//! The Reapers keeper.
//!
//! | Variable                      | Default                                  | What it is                                   |
//! | ----------------------------- | ---------------------------------------- | -------------------------------------------- |
//! | `REAPERS_RUN_MODE`            | `development`                            | `development`, `test`, `staging`, `production` |
//! | `REAPERS_LISTEN`              | `127.0.0.1:8080`                         | Where to listen                              |
//! | `REAPERS_ORIGIN`              | none                                     | The site's origin, for the live socket       |
//! | `REAPERS_SECRET_<name>`       | `config/passwords.toml` in development   | Each secret; see `config::REQUIRED_SECRETS`  |
//! | `REAPERS_SOLANA_*`, …         | devnet                                   | The chain; see `config::SolanaConfig`        |

use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;

use leptoskit_auth::Keys;
use leptoskit_axum::GracefulShutdown;
use leptoskit_axum::Readiness;
use leptoskit_db::Db;
use reapers_server::chain::Gateway;
use reapers_server::config::ConfigError;
use reapers_server::config::RunMode;
use reapers_server::config::Secrets;
use reapers_server::config::SolanaConfig;
use reapers_server::rift::RiftHall;
use reapers_server::server::Options;
use reapers_server::signer::ReceiptSigner;
use reapers_server::tables::MIGRATIONS;
use sha2::Digest;

/// What a deployed keeper exits with when a secret is missing:
/// `EX_CONFIG`, as the Dart keeper did.
const MISSING_SECRET: u8 = 78;

/// The database a developer's keeper uses when none is configured.
const DEVELOPMENT_DATABASE: &str = "postgres://localhost:5432/reapers?sslmode=disable";

#[tokio::main]
async fn main() -> ExitCode {
	match run().await {
		Ok(code) => code,
		Err(error) => {
			eprintln!("{error}");
			ExitCode::FAILURE
		}
	}
}

async fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
	let mode = RunMode::from_env()?;
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));
	let secrets = Secrets::load(mode, root)?;
	let missing = secrets.missing(mode);

	if !missing.is_empty() {
		eprintln!(
			"Refusing to start in \"{mode}\" mode without: {}. Set {}<name> for each one.",
			missing.join(", "),
			Secrets::ENV_PREFIX,
		);

		return Ok(ExitCode::from(MISSING_SECRET));
	}

	let solana = SolanaConfig::from_env()?;
	let gateway = Gateway::connect(solana)?;
	let signer = if let Some(secret) = secrets.get(ReceiptSigner::SECRET) {
		ReceiptSigner::from_secret(secret)
	} else {
		eprintln!(
			"Signing receipts with the development key anyone can derive: they prove \
 				 nothing. A deployed keeper needs {}{}.",
			Secrets::ENV_PREFIX,
			ReceiptSigner::SECRET,
		);
		ReceiptSigner::development()
	};
	let keys = match secrets.get("authKeys") {
		Some(keys) => Keys::parse(keys)?,
		None => development_keys(),
	};
	let url = secrets
		.get("database")
		.map_or_else(|| DEVELOPMENT_DATABASE.to_owned(), str::to_owned);
	let readiness = Readiness::new();
	let migrated = readiness.check("migrations");
	let live_ready = readiness.check("live");
	let address: SocketAddr = std::env::var("REAPERS_LISTEN")
		.unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
		.parse()
		.map_err(|_| ConfigError("REAPERS_LISTEN: not an address".to_owned()))?;
	let db = Db::connect(&url).await?;

	for name in db.migrate(MIGRATIONS).await? {
		println!("applied migration {name}");
	}

	migrated.pass();

	let reapers = reapers_server::server::start(
		&db,
		keys,
		Options {
			mode,
			gateway,
			signer,
			hall: RiftHall::default(),
			origin: std::env::var("REAPERS_ORIGIN").ok(),
		},
	)?;

	reapers.live.start().await?;
	live_ready.pass();
	println!("keeper key {} ({mode})", reapers.keeper.signer().address());

	let app = reapers.router.clone().merge(readiness.routes());
	let listener = tokio::net::TcpListener::bind(address).await?;
	let live = reapers.live.clone();
	let shutdown = GracefulShutdown::new()
		.readiness(&readiness)
		.signal(async move {
			leptoskit_axum::shutdown_signal().await;
			live.drain();
		});

	println!("listening on http://{address}");
	leptoskit_axum::serve_listener_with(listener, app, shutdown).await?;

	Ok(ExitCode::SUCCESS)
}

/// Sign-in's keys for a developer's keeper: derived from a public name, so
/// its sessions mean nothing anywhere else. A deployed keeper must be given
/// `authKeys`.
fn development_keys() -> Keys {
	let secret: [u8; 32] = sha2::Sha256::digest(b"reapers-development-auth-key").into();

	Keys::single("development", secret)
}
