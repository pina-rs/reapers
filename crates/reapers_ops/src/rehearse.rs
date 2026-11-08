//! The whole devnet runbook, on a local Surfpool: deploy with the Solana CLI
//! exactly as `docs/deploy.md` does, initialize, mint the demo husks, release
//! one, sign its receipt with a test keeper, absorb it, and check it is gone.

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

fn step(text: &str) {
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
	let keeper = crate::keeper_from_secret("reapers-development-receipt-key");
	step("initialize the config, with the development keeper");
	let signature = chain
		.send(
			&[program::initialize(
				&deployer.address(),
				&deployer.address(),
				&keeper.address(),
			)],
			&[&deployer],
			units::INITIALIZE,
		)
		.await?;
	let config = program::configuration(&chain)
		.await?
		.ok_or("the config does not exist after initializing")?;
	if config.keeper != keeper.address() || config.authority != deployer.address() || config.paused
	{
		return Err("the config does not hold what was initialized".into());
	}
	println!("    {signature}");
	println!(
		"    config {}: authority {}, keeper {}, paused {}",
		program::config(),
		config.authority,
		config.keeper,
		config.paused
	);

	step("mint the five demo collections, eight husks each, to a player");
	let (_, player) = random_keypair()?;
	chain.airdrop(player.address(), 10_000_000_000).await?;
	let minted = demo::mint_all(
		&chain,
		&deployer,
		&player.address(),
		"https://reapers.invalid/husks/",
		|_| {},
	)
	.await?;
	for collection in &minted {
		println!(
			"    {} {} ({} husks)",
			collection.name,
			collection.address,
			collection.assets.len()
		);
	}
	std::fs::write(
		options.work_dir.join("demo-husks.json"),
		serde_json::to_string_pretty(&demo::manifest(&url, &minted))?,
	)?;

	let collection = minted
		.iter()
		.find(|collection| collection.slug == "thugbirdz")
		.ok_or("no Thugbirdz collection")?;
	let asset = collection.assets[0];
	let escrow = crate::chain::address(&reapers_program_client::absorb::escrow(&pubkey(&asset)));

	step(&format!("release {asset} into escrow"));
	let signature = chain
		.send(
			&[program::release_core(
				&player.address(),
				&asset,
				Some(&collection.address),
			)],
			&[&player],
			units::RELEASE_CORE,
		)
		.await?;
	let owner = asset_owner(&chain, &asset).await?;
	if owner != Some((1, escrow)) {
		return Err(format!("the asset is not in escrow: {owner:?}").into());
	}
	println!("    {signature}");
	println!("    the asset's owner is now the escrow {escrow}");

	step("sign a receipt with the keeper and absorb");
	let receipt = Receipt {
		player: player.address().to_bytes(),
		mint: asset.to_bytes(),
		grade: Grade::Clean,
		feeding: Feeding::Rich,
		xp: 63,
		anima: 20,
		at: i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())?,
		nonce: 0x5eed,
	};
	let instructions = absorb(&keeper, &receipt, &collection.address);
	let before = chain.balance(player.address()).await?;
	let signature = chain
		.send(&instructions, &[&player], units::ABSORB_CORE)
		.await?;
	println!("    {signature}");

	step("check the husk is gone and the escrow closed");
	let tombstone = chain
		.account(asset)
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

	step("try to absorb it again with the same receipt");
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
	let other = collection.assets[1];
	chain
		.send(
			&[program::release_core(
				&player.address(),
				&other,
				Some(&collection.address),
			)],
			&[&player],
			units::RELEASE_CORE,
		)
		.await?;
	let impostor = Keypair::from_seed(&[13; 32]);
	let forged = absorb(
		&impostor,
		&Receipt {
			mint: other.to_bytes(),
			..receipt
		},
		&collection.address,
	);
	match chain.send(&forged, &[&player], units::ABSORB_CORE).await {
		Ok(signature) => {
			return Err(format!("a receipt from another key was accepted: {signature}").into());
		}
		Err(error) => println!("    refused, as it must be: {error}"),
	}

	step("rehearsal passed");
	Ok(())
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
