//! The Reapers keeper.
//!
//! | Variable                      | Default                                  | What it is                                   |
//! | ----------------------------- | ---------------------------------------- | -------------------------------------------- |
//! | `REAPERS_RUN_MODE`            | `development`                            | `development`, `test`, `staging`, `production` |
//! | `REAPERS_LISTEN`              | `127.0.0.1:8080`                         | Where to listen                              |
//! | `REAPERS_ORIGIN`              | none (required in staging/production)    | The site's origin, for CORS and live sockets |
//! | `REAPERS_DB_POOL`             | `32`                                     | Maximum PostgreSQL pool connections (`4..=256`) |
//! | `REAPERS_SECRET_<name>`       | `config/passwords.toml` in development   | Each secret; see `config::REQUIRED_SECRETS`  |
//! | `REAPERS_SOLANA_*`, …         | devnet                                   | The chain; see `config::SolanaConfig`        |

use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use leptoskit_auth::Keys;
use leptoskit_axum::GracefulShutdown;
use leptoskit_axum::Readiness;
use leptoskit_db::Connection;
use leptoskit_db::Db;
use leptoskit_db::PoolOptions;
use leptoskit_db::Read;
use leptoskit_db::Statement;
use leptoskit_db::Value;
use reapers_server::chain::Gateway;
use reapers_server::config::ConfigError;
use reapers_server::config::RunMode;
use reapers_server::config::Secrets;
use reapers_server::config::SolanaConfig;
use reapers_server::config::retired_keepers;
use reapers_server::config::validate_origin;
use reapers_server::rift::RiftHall;
use reapers_server::server::Options;
use reapers_server::signer::ReceiptSigner;
use reapers_server::tables::MIGRATIONS;
use sha2::Digest;

/// What a deployed keeper exits with when a secret is missing:
/// `EX_CONFIG`, as the keeper always has.
const MISSING_SECRET: u8 = 78;

/// The database a developer's keeper uses when none is configured.
const DEVELOPMENT_DATABASE: &str = "postgres://localhost:5432/reapers?sslmode=disable";

/// Default PostgreSQL pool size (higher than `leptoskit_db`'s default of 8 so
/// concurrent hunts, live sockets, and RPC reads do not starve).
const DEFAULT_DB_POOL: usize = 32;

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

	secrets.validate_strength(mode)?;

	let origin = validate_origin(std::env::var("REAPERS_ORIGIN").ok().as_deref(), mode)?;
	let solana = SolanaConfig::from_env()?;
	solana.validate_for_mode(mode)?;

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
	let signer = signer.with_retired(retired_keepers(
		std::env::var("REAPERS_RETIRED_KEEPERS").ok().as_deref(),
		mode,
	)?);

	if signer.is_development() && !mode.simulates() {
		return Err(ConfigError(format!(
			"refusing to run in \"{mode}\" mode with the development receipt signing key"
		))
		.into());
	}

	let keys = match secrets.get("authKeys") {
		Some(keys) => Keys::parse(keys)?,
		None => development_keys(),
	};
	let url = secrets
		.get("database")
		.map_or_else(|| DEVELOPMENT_DATABASE.to_owned(), str::to_owned);
	let pool_options = pool_options_from_env()?;
	let readiness = Readiness::new();
	let migrated = readiness.check("migrations");
	let db_ready = readiness.check("database");
	let live_ready = readiness.check("live");
	let address: SocketAddr = std::env::var("REAPERS_LISTEN")
		.unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
		.parse()
		.map_err(|_| ConfigError("REAPERS_LISTEN: not an address".to_owned()))?;
	let db = Db::connect_with(&url, pool_options).await?;

	for name in db.migrate(MIGRATIONS).await? {
		println!("applied migration {name}");
	}

	migrated.pass();
	db_ready.pass();

	let reapers = reapers_server::server::start(
		&db,
		keys,
		Options {
			mode,
			gateway,
			signer,
			hall: RiftHall::default(),
			origin,
		},
	)?;

	reapers.live.start().await?;
	live_ready.pass();
	println!("keeper key {} ({mode})", reapers.keeper.signer().address());

	let maintenance_db = db.system("background maintenance");
	let probe_db = db.system("database readiness probe");
	let maintenance = tokio::spawn(run_maintenance(maintenance_db));
	let probe = tokio::spawn(run_db_probe(probe_db, db_ready));

	let app = reapers.router.clone().merge(readiness.routes());
	let listener = tokio::net::TcpListener::bind(address).await?;
	let live = reapers.live.clone();
	let keeper = Arc::clone(&reapers.keeper);
	let shutdown = GracefulShutdown::new()
		.readiness(&readiness)
		.signal(async move {
			leptoskit_axum::shutdown_signal().await;
			keeper.hall().drain();
			live.drain();
			maintenance.abort();
			probe.abort();
		});

	println!("listening on http://{address}");
	leptoskit_axum::serve_listener_with(listener, app, shutdown).await?;

	Ok(ExitCode::SUCCESS)
}

fn pool_options_from_env() -> Result<PoolOptions, ConfigError> {
	let max_connections = match std::env::var("REAPERS_DB_POOL") {
		Ok(raw) if !raw.trim().is_empty() => {
			let parsed: usize = raw
				.trim()
				.parse()
				.map_err(|_| ConfigError("REAPERS_DB_POOL: not a positive integer".to_owned()))?;
			parsed.clamp(4, 256)
		}
		_ => DEFAULT_DB_POOL,
	};
	Ok(PoolOptions { max_connections })
}

/// Periodically prunes expired wallet challenges, stale/revoked auth sessions,
/// and old idempotency receipts so unbounded table growth never degrades Postgres.
async fn run_maintenance(db: Db) {
	let mut interval = tokio::time::interval(Duration::from_mins(15));
	interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
	// Skip immediate tick at startup.
	interval.tick().await;

	loop {
		interval.tick().await;
		let now = leptoskit_db::now();
		let challenge_cutoff = now - chrono::TimeDelta::hours(1);
		let session_cutoff = now - chrono::TimeDelta::days(7);
		let receipt_cutoff = now - chrono::TimeDelta::hours(24);

		let statements = [
			Statement {
				sql: "DELETE FROM \"wallet_challenges\" WHERE \"consumed\" = TRUE OR \"expires_at\" < $1"
					.to_owned(),
				args: vec![Value::Timestamp(challenge_cutoff)],
			},
			Statement {
				sql: "DELETE FROM \"auth\".\"sessions\" WHERE (\"revoked_at\" IS NOT NULL AND \"revoked_at\" < $1) OR \"expires_at\" < $1"
					.to_owned(),
				args: vec![Value::Timestamp(session_cutoff)],
			},
			Statement {
				sql: "DELETE FROM \"rpc\".\"receipts\" WHERE \"created_at\" < $1".to_owned(),
				args: vec![Value::Timestamp(receipt_cutoff)],
			},
		];

		for statement in statements {
			if let Err(error) = db.execute(statement).await {
				eprintln!("reapers_server: maintenance prune failed: {error}");
			}
		}
	}
}

/// Periodically checks PostgreSQL connectivity and flips the `database`
/// readiness probe if connectivity is lost or restored.
async fn run_db_probe(db: Db, handle: leptoskit_axum::Check) {
	let mut interval = tokio::time::interval(Duration::from_secs(30));
	interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
	interval.tick().await;

	loop {
		interval.tick().await;
		let result = db
			.fetch(Statement {
				sql: "SELECT 1".to_owned(),
				args: Vec::new(),
			})
			.await;

		if let Err(error) = result {
			eprintln!("reapers_server: database readiness probe failed: {error}");
			handle.fail();
		} else {
			handle.pass();
		}
	}
}

/// Sign-in's keys for a developer's keeper: derived from a public name, so
/// its sessions mean nothing anywhere else. A deployed keeper must be given
/// `authKeys`.
fn development_keys() -> Keys {
	let secret: [u8; 32] = sha2::Sha256::digest(b"reapers-development-auth-key").into();

	Keys::single("development", secret)
}
