//! The whole devnet runbook, on a local Surfpool: deploy with the Solana CLI
//! exactly as `docs/deploy.md` does, initialize, mint the demo husks, release
//! one, sign its receipt with a test keeper, absorb it, and check it is gone.
//! The commands the runbook types are run as they are typed: this tool runs
//! its own subcommands.

use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use reapers_receipt::Feeding;
use reapers_receipt::Grade;
use reapers_receipt::Receipt;
use solana_kit_client::Client;
use solana_kit_client::Limits;
use solana_kit_core::Address;
use solana_kit_core::crypto::Keypair;
use solana_kit_native::HttpConfig;
use solana_kit_native::NativeHttp;
use solana_kit_surfpool::Surfnet;
use solana_kit_surfpool::deploy::DeployOptions;

use crate::chain::Chain;
use crate::chain::Error;
use crate::chain::kit;
use crate::chain::pubkey;
use crate::chain::random_keypair;
use crate::chain::write_keypair;
use crate::demo;
use crate::program;
use crate::program::units;

pub struct Options {
	pub program_so: PathBuf,
	pub program_keypair: PathBuf,
	pub core_so: PathBuf,
	pub port: u16,
	pub work_dir: PathBuf,
}

/// Stops Surfpool however the rehearsal ends.
struct Validator(Child);

impl Drop for Validator {
	fn drop(&mut self) {
		let _ = self.0.kill();
		let _ = self.0.wait();
	}
}

pub fn step(text: &str) {
	println!("==> {text}");
}

#[allow(
	clippy::too_many_lines,
	reason = "the runbook, step by step, reads best in one place"
)]
pub async fn rehearse(options: Options) -> Result<(), Error> {
	for (what, path) in [
		(
			"the program (run `pina build` in crates/reapers_program)",
			&options.program_so,
		),
		("the program keypair", &options.program_keypair),
		(
			"Metaplex Core (run `reapers_ops fixtures`)",
			&options.core_so,
		),
	] {
		if !path.is_file() {
			return Err(format!("missing {what}: {}", path.display()).into());
		}
	}
	std::fs::create_dir_all(&options.work_dir)?;
	let url = format!("http://127.0.0.1:{}", options.port);

	step(&format!("start Surfpool at {url}, offline"));
	let _validator = Validator(
		Command::new("surfpool")
			.args(["start", "--offline", "--no-deploy", "--ci", "--yes"])
			.args(["--port", &options.port.to_string()])
			.args(["--ws-port", &(options.port + 1).to_string()])
			.current_dir(&options.work_dir)
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.map_err(|error| format!("cannot start surfpool: {error}"))?,
	);
	let chain = Chain::new(&url, 1)?;
	wait_until_up(&chain).await?;

	step("load Metaplex Core into the validator (a Surfpool cheatcode; devnet has it already)");
	let core = std::fs::read(&options.core_so)?;
	let transport = NativeHttp::new(&url, HttpConfig::default())?;
	Surfnet::new(Client::new(transport, Limits::default()))
		.deploy(DeployOptions::new(demo::CORE, &core))
		.await
		.map_err(|error| format!("cannot load Core: {error}"))?;

	step("make and fund a deployer, as `solana-keygen new` and a faucet would");
	let deployer_path = options.work_dir.join("deployer.json");
	let (seed, deployer) = random_keypair()?;
	write_keypair(&deployer_path, &seed, &deployer)?;
	chain.airdrop(deployer.address(), 100_000_000_000).await?;
	println!("    deployer {}", deployer.address());

	step("deploy with the Solana CLI, as docs/deploy.md does");
	deploy(&url, &deployer_path, &options)?;
	let program_account = chain
		.account(program::program_id())
		.await?
		.ok_or("the program account does not exist after deploying")?;
	if !program_account.executable {
		return Err("the program account is not executable".into());
	}
	println!("    program {} is executable", program::program_id());

	// The development keeper: the key `ReceiptSigner` derives in development.
	let secret = "reapers-development-receipt-key";
	let keeper = crate::keeper_from_secret(secret);
	let deployer_file = deployer_path.display().to_string();
	let keeper_address = keeper.address().to_string();

	step("reapers_ops status, before initializing");
	ops(&["status", "--url", &url], &[])?;

	step("reapers_ops initialize, with the development keeper");
	ops(
		&[
			"initialize",
			"--url",
			&url,
			"--priority-fee",
			"1",
			"--upgrade-authority",
			&deployer_file,
			"--keeper",
			&keeper_address,
		],
		&[],
	)?;
	let config = program::configuration(&chain)
		.await?
		.ok_or("the config does not exist after initializing")?;
	if config.keeper != keeper.address() || config.authority != deployer.address() || config.paused
	{
		return Err("the config does not hold what was initialized".into());
	}

	step("reapers_ops mint-demo: five collections, eight husks each, to a player");
	let player_path = options.work_dir.join("player.json");
	let (seed, player) = random_keypair()?;
	write_keypair(&player_path, &seed, &player)?;
	chain.airdrop(player.address(), 10_000_000_000).await?;
	let manifest_path = options.work_dir.join("demo-husks.json");
	ops(
		&[
			"mint-demo",
			"--url",
			&url,
			"--priority-fee",
			"1",
			"--payer",
			&deployer_file,
			"--owner",
			&player.address().to_string(),
			"--out",
			&manifest_path.display().to_string(),
		],
		&[],
	)?;
	let manifest: serde_json::Value =
		serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
	let thugbirdz = manifest["collections"]
		.as_array()
		.and_then(|collections| {
			collections
				.iter()
				.find(|collection| collection["slug"] == "thugbirdz")
		})
		.ok_or("no Thugbirdz collection in the manifest")?;
	let collection: Address = thugbirdz["address"]
		.as_str()
		.ok_or("no collection address")?
		.parse()?;
	let assets: Vec<Address> = thugbirdz["assets"]
		.as_array()
		.ok_or("no assets")?
		.iter()
		.filter_map(|asset| asset.as_str()?.parse().ok())
		.collect();
	if assets.len() != usize::from(demo::ASSETS_PER_COLLECTION) {
		return Err(format!("{} Thugbirdz husks, not 8", assets.len()).into());
	}
	let asset = assets[0];

	step("reapers_ops smoke: release one husk and absorb it with the keeper");
	ops(
		&[
			"smoke",
			"--url",
			&url,
			"--priority-fee",
			"1",
			"--player",
			&player_path.display().to_string(),
			"--asset",
			&asset.to_string(),
			"--collection",
			&collection.to_string(),
		],
		&[("REAPERS_SECRET_receiptSigningKey", secret)],
	)?;
	let instructions = absorb(
		&keeper,
		&receipt_for(&player.address(), &asset)?,
		&collection,
	);

	step("try to absorb the burned husk again, with a fresh receipt from the keeper");
	match chain
		.send(&instructions, &[&player], units::ABSORB_CORE)
		.await
	{
		Ok(signature) => {
			return Err(format!("a replayed receipt was accepted: {signature}").into());
		}
		Err(error) => println!("    refused, as it must be: {error}"),
	}

	step("try to absorb another husk with a receipt from another key");
	let other = assets[1];
	chain
		.send(
			&[program::release_core(
				&player.address(),
				&other,
				Some(&collection),
			)],
			&[&player],
			units::RELEASE_CORE,
		)
		.await?;
	let impostor = Keypair::from_seed(&[13; 32]);
	let forged = absorb(
		&impostor,
		&receipt_for(&player.address(), &other)?,
		&collection,
	);
	match chain.send(&forged, &[&player], units::ABSORB_CORE).await {
		Ok(signature) => {
			return Err(format!("a receipt from another key was accepted: {signature}").into());
		}
		Err(error) => println!("    refused, as it must be: {error}"),
	}

	step("reapers_ops set-paused: releasing stops while paused");
	let pause = |paused: &'static str| {
		ops(
			&[
				"set-paused",
				"--url",
				&url,
				"--priority-fee",
				"1",
				"--authority",
				&deployer_file,
				"--paused",
				paused,
			],
			&[],
		)
	};
	pause("true")?;
	let third = assets[2];
	match chain
		.send(
			&[program::release_core(
				&player.address(),
				&third,
				Some(&collection),
			)],
			&[&player],
			units::RELEASE_CORE,
		)
		.await
	{
		Ok(signature) => return Err(format!("released while paused: {signature}").into()),
		Err(error) => {
			println!(
				"    refused, as it must be: {}",
				last_line(&error.to_string())
			);
		}
	}
	pause("false")?;

	step("reapers_ops status, at the end");
	ops(&["status", "--url", &url], &[])?;

	step("rehearsal passed");
	Ok(())
}

/// The receipt a keeper might sign for `player` absorbing `asset` now.
fn receipt_for(player: &Address, asset: &Address) -> Result<Receipt, Error> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

	Ok(Receipt {
		player: player.to_bytes(),
		mint: asset.to_bytes(),
		grade: Grade::Clean,
		feeding: Feeding::Rich,
		xp: 63,
		anima: 20,
		at: i64::try_from(now)?,
		nonce: now,
	})
}

/// Releases `asset` from `player` into escrow, unless it is there already,
/// absorbs it with a receipt `keeper` signs, and checks that it is burned and
/// its escrow closed. Returns the absorbing instructions, so a caller can try
/// them again.
pub async fn release_and_absorb(
	chain: &Chain,
	player: &Keypair,
	keeper: &Keypair,
	asset: &Address,
	collection: &Address,
) -> Result<Vec<solana_kit_core::Instruction>, Error> {
	let escrow = crate::chain::address(&reapers_program_client::absorb::escrow(&pubkey(asset)));

	match asset_owner(chain, asset).await? {
		Some((1, owner)) if owner == escrow => {
			step(&format!("{asset} is already in escrow"));
		}
		Some((1, owner)) if owner == player.address() => {
			step(&format!("release {asset} into escrow"));
			let signature = chain
				.send(
					&[program::release_core(
						&player.address(),
						asset,
						Some(collection),
					)],
					&[player],
					units::RELEASE_CORE,
				)
				.await?;
			let owner = asset_owner(chain, asset).await?;
			if owner != Some((1, escrow)) {
				return Err(format!("the asset is not in escrow: {owner:?}").into());
			}
			println!("    {signature}");
			println!("    the asset's owner is now the escrow {escrow}");
		}
		other => {
			return Err(format!("{asset} is neither the player's nor in escrow: {other:?}").into());
		}
	}

	step("sign a receipt with the keeper and absorb");
	let instructions = absorb(keeper, &receipt_for(&player.address(), asset)?, collection);
	let before = chain.balance(player.address()).await?;
	let signature = chain
		.send(&instructions, &[player], units::ABSORB_CORE)
		.await?;
	println!("    {signature}");

	step("check the husk is gone and the escrow closed");
	let tombstone = chain
		.account(*asset)
		.await?
		.ok_or("the asset account vanished entirely")?;
	if tombstone.data != [0] {
		return Err(format!(
			"the asset was not burned: {} bytes left",
			tombstone.data.len()
		)
		.into());
	}
	if chain.account(escrow).await?.is_some() {
		return Err("the escrow still exists".into());
	}
	let after = chain.balance(player.address()).await?;
	println!("    the asset is a one-byte tombstone; the escrow is closed");
	println!(
		"    the player got back {} lamports of rent, less fees",
		after.saturating_sub(before)
	);

	Ok(instructions)
}

/// Runs this tool's own subcommand, as the runbook would, and indents what it
/// prints.
fn ops(args: &[&str], env: &[(&str, &str)]) -> Result<(), Error> {
	let output = Command::new(std::env::current_exe()?)
		.args(args)
		.envs(env.iter().copied())
		.output()?;

	for line in String::from_utf8_lossy(&output.stdout).lines() {
		println!("    {line}");
	}
	if !output.status.success() {
		return Err(format!(
			"reapers_ops {} failed: {}",
			args.first().unwrap_or(&""),
			String::from_utf8_lossy(&output.stderr)
		)
		.into());
	}

	Ok(())
}

fn last_line(text: &str) -> &str {
	text.lines().last().unwrap_or(text)
}

async fn wait_until_up(chain: &Chain) -> Result<(), Error> {
	for _ in 0..120 {
		if chain.balance(Address::from_bytes([0; 32])).await.is_ok() {
			return Ok(());
		}
		tokio::time::sleep(Duration::from_millis(500)).await;
	}

	Err("Surfpool did not answer within a minute".into())
}

fn deploy(url: &str, deployer: &Path, options: &Options) -> Result<(), Error> {
	let output = Command::new("solana")
		.args(["program", "deploy", "--url", url])
		.arg("--keypair")
		.arg(deployer)
		.arg("--program-id")
		.arg(&options.program_keypair)
		.arg("--upgrade-authority")
		.arg(deployer)
		.args([
			"--with-compute-unit-price",
			"1",
			"--max-sign-attempts",
			"5",
			"--use-rpc",
		])
		.arg(&options.program_so)
		.output()
		.map_err(|error| format!("cannot run the solana CLI: {error}"))?;

	for line in String::from_utf8_lossy(&output.stdout).lines() {
		println!("    {line}");
	}
	if !output.status.success() {
		return Err(format!(
			"solana program deploy failed: {}",
			String::from_utf8_lossy(&output.stderr)
		)
		.into());
	}

	Ok(())
}

fn absorb(
	keeper: &Keypair,
	receipt: &Receipt,
	collection: &Address,
) -> Vec<solana_kit_core::Instruction> {
	let signature = keeper.sign(&receipt.encode()).to_bytes();

	reapers_program_client::absorb::absorb_core(
		&pubkey(&keeper.address()),
		&signature,
		receipt,
		Some(pubkey(collection)),
	)
	.into_iter()
	.map(kit)
	.collect()
}

async fn asset_owner(chain: &Chain, asset: &Address) -> Result<Option<(u8, Address)>, Error> {
	Ok(chain
		.account(*asset)
		.await?
		.and_then(|account| demo::asset_owner(&account.data)))
}
