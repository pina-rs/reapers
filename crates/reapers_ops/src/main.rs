//! `reapers_ops`: deploys, configures and rehearses the Reapers program.
//!
//! Every command that writes names its cluster with `--url`, and none of them
//! reaches mainnet without `--allow-mainnet`. Keys are read from Solana CLI
//! keypair files and never printed. See `docs/deploy.md` for the runbook.

mod chain;
mod demo;
mod program;
mod rehearse;

use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use clap::Subcommand;
use sha2::Digest;
use sha2::Sha256;
use solana_kit_core::Address;
use solana_kit_core::crypto::Keypair;

use crate::chain::Chain;
use crate::chain::Error;
use crate::chain::read_keypair;
use crate::program::units;

#[derive(Parser)]
#[command(about = "Deploys, configures and rehearses the Reapers program.")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(clap::Args)]
struct Cluster {
	/// The cluster's JSON-RPC URL, or `devnet`, `localhost` or `mainnet`.
	#[arg(long, short = 'u')]
	url: String,
	/// Micro-lamports per compute unit to offer as a priority fee.
	#[arg(long, default_value_t = 10_000)]
	priority_fee: u64,
	/// Allow writing to mainnet.
	#[arg(long)]
	allow_mainnet: bool,
}

impl Cluster {
	fn chain(&self) -> Result<Chain, Error> {
		let url = match self.url.as_str() {
			"devnet" | "d" => "https://api.devnet.solana.com",
			"localhost" | "l" => "http://127.0.0.1:8899",
			"mainnet" | "mainnet-beta" | "m" => "https://api.mainnet-beta.solana.com",
			other => other,
		};
		if url.contains("mainnet") && !self.allow_mainnet {
			return Err("refusing to write to mainnet without --allow-mainnet".into());
		}

		Chain::new(url, self.priority_fee)
	}
}

#[derive(Subcommand)]
enum Commands {
	/// Print the program's configuration on a cluster.
	Status {
		#[command(flatten)]
		cluster: Cluster,
	},
	/// Create the configuration. The signer must be the upgrade authority.
	Initialize {
		#[command(flatten)]
		cluster: Cluster,
		/// The program's upgrade authority, which pays and signs.
		#[arg(long)]
		upgrade_authority: PathBuf,
		/// Who administers the program; the upgrade authority if omitted.
		#[arg(long)]
		authority: Option<Address>,
		/// The keeper's Ed25519 public key (see the `keeper` command).
		#[arg(long)]
		keeper: Address,
	},
	/// Replace the keeper. Receipts from the old one stop working at once.
	SetKeeper {
		#[command(flatten)]
		cluster: Cluster,
		/// The configured authority.
		#[arg(long)]
		authority: PathBuf,
		#[arg(long)]
		keeper: Address,
	},
	/// Pause or resume releasing and absorbing.
	SetPaused {
		#[command(flatten)]
		cluster: Cluster,
		/// The configured authority.
		#[arg(long)]
		authority: PathBuf,
		#[arg(long, action = clap::ArgAction::Set)]
		paused: bool,
	},
	/// Hand the administration to another key. Both keys sign.
	SetAuthority {
		#[command(flatten)]
		cluster: Cluster,
		/// The configured authority.
		#[arg(long)]
		authority: PathBuf,
		/// The new authority.
		#[arg(long)]
		new_authority: PathBuf,
	},
	/// Print the keeper public key the server derives from its secret.
	///
	/// The secret is read from the environment variable named by `--env`
	/// (`SERVERPOD_PASSWORD_receiptSigningKey` by default), never from the
	/// command line, and is not printed.
	Keeper {
		#[arg(long, default_value = "SERVERPOD_PASSWORD_receiptSigningKey")]
		env: String,
	},
	/// Mint the five demo husk collections, eight Core assets each.
	MintDemo {
		#[command(flatten)]
		cluster: Cluster,
		/// Pays, and becomes each collection's update authority.
		#[arg(long)]
		payer: PathBuf,
		/// Who owns the husks; the payer if omitted.
		#[arg(long)]
		owner: Option<Address>,
		/// Where the placeholder metadata JSON will be hosted.
		#[arg(long, default_value = "https://reapers.invalid/husks/")]
		uri_base: String,
		/// Where to write the minted addresses.
		#[arg(long, default_value = "demo-husks.json")]
		out: PathBuf,
	},
	/// Release one demo husk and absorb it with the real keeper: the smoke
	/// test after a deployment. It burns the husk.
	///
	/// The keeper secret is read from the environment variable named by
	/// `--env`, as for `keeper`.
	Smoke {
		#[command(flatten)]
		cluster: Cluster,
		/// The husk's owner, who releases and absorbs it.
		#[arg(long)]
		player: PathBuf,
		/// The Core asset to burn.
		#[arg(long)]
		asset: Address,
		/// Its collection.
		#[arg(long)]
		collection: Address,
		#[arg(long, default_value = "SERVERPOD_PASSWORD_receiptSigningKey")]
		env: String,
	},
	/// Fetch Metaplex Core from mainnet for the tests and the rehearsal.
	Fixtures {
		#[arg(long, default_value = "target/fixtures/mpl_core.so")]
		out: PathBuf,
	},
	/// Run the whole runbook against a local Surfpool.
	Rehearse {
		#[arg(long, default_value = "target/deploy/reapers_program.so")]
		program: PathBuf,
		/// The program keypair; the devnet one by default, used only locally.
		#[arg(long)]
		program_keypair: Option<PathBuf>,
		#[arg(long, default_value = "target/fixtures/mpl_core.so")]
		core: PathBuf,
		#[arg(long, default_value_t = 18_899)]
		port: u16,
		#[arg(long, default_value = "target/rehearsal")]
		work_dir: PathBuf,
	},
}

/// The keeper `ReceiptSigner` in `reapers_server` derives from `secret`: the
/// seed is the secret's SHA-256.
pub fn keeper_from_secret(secret: &str) -> Keypair {
	Keypair::from_seed(&Sha256::digest(secret.as_bytes()).into())
}

fn home() -> Result<PathBuf, Error> {
	std::env::var_os("HOME")
		.map(PathBuf::from)
		.ok_or_else(|| "HOME is not set".into())
}

#[tokio::main]
async fn main() {
	if let Err(error) = run(Cli::parse()).await {
		eprintln!("error: {error}");
		std::process::exit(1);
	}
}

#[allow(clippy::too_many_lines, reason = "one arm per command")]
async fn run(cli: Cli) -> Result<(), Error> {
	match cli.command {
		Commands::Status { cluster } => {
			let chain = cluster.chain()?;
			println!("program {}", program::program_id());
			match chain.account(program::program_id()).await? {
				Some(account) => println!("  deployed, executable: {}", account.executable),
				None => println!("  not deployed on {}", chain.url),
			}
			match program::configuration(&chain).await? {
				Some(config) => {
					println!("config {}", program::config());
					println!("  authority {}", config.authority);
					println!("  keeper    {}", config.keeper);
					println!("  paused    {}", config.paused);
				}
				None => println!("config {}: not initialized", program::config()),
			}
		}
		Commands::Initialize {
			cluster,
			upgrade_authority,
			authority,
			keeper,
		} => {
			let chain = cluster.chain()?;
			let signer = read_keypair(&upgrade_authority)?;
			let authority = authority.unwrap_or_else(|| signer.address());
			let signature = chain
				.send(
					&[program::initialize(&signer.address(), &authority, &keeper)],
					&[&signer],
					units::INITIALIZE,
				)
				.await?;
			println!("initialized: {signature}");
			println!("  authority {authority}");
			println!("  keeper    {keeper}");
		}
		Commands::SetKeeper {
			cluster,
			authority,
			keeper,
		} => {
			let chain = cluster.chain()?;
			let signer = read_keypair(&authority)?;
			let signature = chain
				.send(
					&[program::set_keeper(&signer.address(), &keeper)],
					&[&signer],
					units::ADMIN,
				)
				.await?;
			println!("keeper is now {keeper}: {signature}");
		}
		Commands::SetPaused {
			cluster,
			authority,
			paused,
		} => {
			let chain = cluster.chain()?;
			let signer = read_keypair(&authority)?;
			let signature = chain
				.send(
					&[program::set_paused(&signer.address(), paused)],
					&[&signer],
					units::ADMIN,
				)
				.await?;
			println!("paused is now {paused}: {signature}");
		}
		Commands::SetAuthority {
			cluster,
			authority,
			new_authority,
		} => {
			let chain = cluster.chain()?;
			let signer = read_keypair(&authority)?;
			let successor = read_keypair(&new_authority)?;
			let signature = chain
				.send(
					&[program::set_authority(
						&signer.address(),
						&successor.address(),
					)],
					&[&signer, &successor],
					units::ADMIN,
				)
				.await?;
			println!("authority is now {}: {signature}", successor.address());
		}
		Commands::Keeper { env } => {
			let secret = std::env::var(&env).map_err(|_| format!("{env} is not set"))?;
			println!("{}", keeper_from_secret(&secret).address());
		}
		Commands::MintDemo {
			cluster,
			payer,
			owner,
			uri_base,
			out,
		} => {
			let chain = cluster.chain()?;
			let payer = read_keypair(&payer)?;
			let owner = owner.unwrap_or_else(|| payer.address());
			let minted = demo::mint_all(&chain, &payer, &owner, &uri_base, |line| {
				println!("{line}");
			})
			.await?;
			std::fs::write(
				&out,
				serde_json::to_string_pretty(&demo::manifest(&chain.url, &minted))?,
			)?;
			println!("wrote {}", out.display());
		}
		Commands::Smoke {
			cluster,
			player,
			asset,
			collection,
			env,
		} => {
			let chain = cluster.chain()?;
			let player = read_keypair(&player)?;
			let secret = std::env::var(&env).map_err(|_| format!("{env} is not set"))?;
			let keeper = keeper_from_secret(&secret);
			let config = program::configuration(&chain)
				.await?
				.ok_or("the program is not initialized")?;
			if config.keeper != keeper.address() {
				return Err(format!(
					"{env} derives keeper {}, but the program trusts {}",
					keeper.address(),
					config.keeper
				)
				.into());
			}
			rehearse::release_and_absorb(&chain, &player, &keeper, &asset, &collection).await?;
			rehearse::step("smoke test passed");
		}
		Commands::Fixtures { out } => {
			if let Some(parent) = out.parent() {
				std::fs::create_dir_all(parent)?;
			}
			let status = Command::new("solana")
				.args(["program", "dump", "--url", "mainnet-beta"])
				.arg(demo::CORE.to_string())
				.arg(&out)
				.status()
				.map_err(|error| format!("cannot run the solana CLI: {error}"))?;
			if !status.success() {
				return Err("solana program dump failed".into());
			}
		}
		Commands::Rehearse {
			program,
			program_keypair,
			core,
			port,
			work_dir,
		} => {
			let program_keypair = match program_keypair {
				Some(path) => path,
				None => home()?.join(".config/solana/reapers/program-devnet-keypair.json"),
			};
			rehearse::rehearse(rehearse::Options {
				program_so: program,
				program_keypair,
				core_so: core,
				port,
				work_dir,
			})
			.await?;
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The development keeper, as `reapers_receipt`'s vector has it.
	#[test]
	fn derives_the_development_keeper() {
		assert_eq!(
			keeper_from_secret("reapers-development-receipt-key")
				.address()
				.to_string(),
			"5tTh2Cc1DkXv1KDhB2XnCkwsN1GuvuZU9C21yCo7BCrd"
		);
	}

	#[test]
	fn refuses_mainnet_without_the_flag() {
		let cluster = Cluster {
			url: "mainnet".into(),
			priority_fee: 0,
			allow_mainnet: false,
		};
		assert!(cluster.chain().is_err());
	}

	#[test]
	fn names_the_compute_budget_program() {
		let instruction = program::set_paused(&Address::from_bytes([1; 32]), true);
		assert_eq!(instruction.data, [3, 1]);
	}
}
