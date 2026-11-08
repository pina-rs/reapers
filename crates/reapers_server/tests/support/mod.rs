//! What every keeper test shares: a real PostgreSQL, a keeper on a fresh
//! database of it, a chain answering what each test says, and players.
//!
//! Each test binary starts one PostgreSQL cluster with `initdb` and
//! `pg_ctl` under Cargo's test directory, migrates a template database once,
//! and gives every test a copy of it, so tests run side by side without
//! seeing each other's rows. A watchdog stops the cluster and deletes it
//! once the test binary exits. No Docker, no shared server.

#![allow(dead_code, reason = "each test binary uses some of this")]

use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures::future::BoxFuture;
use leptoskit_auth::Aal;
use leptoskit_auth::Account;
use leptoskit_auth::SignedIn;
use leptoskit_db::Db;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use leptoskit_rpc::Identity;
use leptoskit_rpc::Operation;
use reapers_api::keeper::KeeperHandle;
use reapers_rules::HuntBot;
use reapers_rules::HuntLog;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::Sighting;
use reapers_server::chain::Gateway;
use reapers_server::chain::Rpc;
use reapers_server::config::RunMode;
use reapers_server::config::SolanaConfig;
use reapers_server::keeper::Keeper;
use reapers_server::keeper::Shared;
use reapers_server::rift::RiftHall;
use reapers_server::signer::ReceiptSigner;
use reapers_server::tables::MIGRATIONS;
use serde_json::Value;
use serde_json::json;
use solana_kit_client::Reply;
use solana_kit_client::Request;
use solana_kit_client::TransportError;
use solana_kit_client::TransportFailureKind;

/// The test cluster: where it listens.
struct Cluster {
	port: u16,
}

const TEMPLATE: &str = "reapers_template";

fn cluster() -> &'static Cluster {
	static CLUSTER: OnceLock<Cluster> = OnceLock::new();

	CLUSTER.get_or_init(|| {
		let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
			.join(format!("reapers-postgres-{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&directory);
		let port = TcpListener::bind("127.0.0.1:0")
			.and_then(|listener| listener.local_addr())
			.map(|address| address.port())
			.expect("a free port");

		run(Command::new(tool("initdb"))
			.arg("-D")
			.arg(&directory)
			.args([
				"-U",
				"postgres",
				"--auth=trust",
				"--encoding=UTF8",
				"--no-locale",
				"--no-sync",
			]));
		run(Command::new(tool("pg_ctl"))
			.arg("-D")
			.arg(&directory)
			.arg("-l")
			.arg(directory.join("log"))
			.arg("-o")
			.arg(format!(
				"-p {port} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' -c \
				 fsync=off -c synchronous_commit=off -c full_page_writes=off -c \
				 max_connections=500"
			))
			.args(["-w", "start"]));

		// Stops the cluster and removes it once this test binary is gone.
		#[allow(
			clippy::zombie_processes,
			reason = "the watchdog outlives this process on purpose"
		)]
		Command::new("sh")
			.arg("-c")
			.arg(format!(
				"while kill -0 {pid} 2>/dev/null; do sleep 1; done; '{pg_ctl}' -D '{dir}' stop -m \
				 immediate >/dev/null 2>&1; rm -rf '{dir}'",
				pid = std::process::id(),
				pg_ctl = tool("pg_ctl").display(),
				dir = directory.display(),
			))
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.expect("the watchdog starts");

		run(Command::new(tool("createdb")).args([
			"-h",
			"127.0.0.1",
			"-p",
			&port.to_string(),
			"-U",
			"postgres",
			TEMPLATE,
		]));

		// The template is migrated once, on a runtime of its own, so no
		// test's runtime holds its connections.
		std::thread::spawn(move || {
			let runtime = tokio::runtime::Runtime::new().expect("a runtime");

			runtime.block_on(async {
				let db = Db::connect(&url(port, TEMPLATE))
					.await
					.expect("the template connects");

				db.migrate(MIGRATIONS).await.expect("the migrations apply");
			});
		})
		.join()
		.expect("the template migrates");

		Cluster { port }
	})
}

/// The PostgreSQL tool `name`, found on `PATH` (or in `REAPERS_PG_BIN`) and
/// followed through symbolic links, so that one installed through a profile
/// of links still finds the files it was built beside.
fn tool(name: &str) -> PathBuf {
	let directories = std::env::var_os("REAPERS_PG_BIN")
		.into_iter()
		.chain(std::env::var_os("PATH"))
		.flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>());

	for directory in directories {
		let candidate = directory.join(name);

		if candidate.is_file() {
			return std::fs::canonicalize(&candidate).unwrap_or(candidate);
		}
	}

	panic!("PostgreSQL's `{name}` is not on PATH; install PostgreSQL or set REAPERS_PG_BIN")
}

fn run(command: &mut Command) {
	let output = command.output().expect("a PostgreSQL tool runs");

	assert!(
		output.status.success(),
		"{command:?} failed:\n{}{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);
}

fn url(port: u16, database: &str) -> String {
	format!("postgres://postgres@127.0.0.1:{port}/{database}?sslmode=disable")
}

/// A fresh, migrated database of its own.
pub async fn database() -> Db {
	static NEXT: AtomicU32 = AtomicU32::new(0);

	let cluster = cluster();
	let name = format!("reapers_{}", NEXT.fetch_add(1, Ordering::Relaxed));
	let port = cluster.port.to_string();

	for attempt in 0.. {
		let output = Command::new(tool("createdb"))
			.args([
				"-h",
				"127.0.0.1",
				"-p",
				&port,
				"-U",
				"postgres",
				"-T",
				TEMPLATE,
				&name,
			])
			.output()
			.expect("createdb runs");

		if output.status.success() {
			break;
		}

		// The template's last connections may still be closing.
		assert!(
			attempt < 50,
			"createdb failed: {}",
			String::from_utf8_lossy(&output.stderr)
		);
		tokio::time::sleep(Duration::from_millis(100)).await;
	}

	Db::connect(&url(cluster.port, &name))
		.await
		.expect("the test database connects")
}

/// The chain each test controls: its genesis, its accounts, and whether it
/// answers at all.
#[derive(Clone)]
pub struct Chain {
	inner: Arc<Mutex<ChainState>>,
}

pub struct ChainState {
	pub genesis: String,
	pub balance: u64,
	pub accounts: HashMap<String, Value>,
	pub fail: bool,
	/// Every request, by method name, with its parameters.
	pub calls: Vec<(String, Value)>,
}

impl Default for Chain {
	fn default() -> Self {
		Self {
			inner: Arc::new(Mutex::new(ChainState {
				genesis: SolanaConfig::new(
					"solana:devnet",
					"https://api.devnet.solana.com",
					"https://reapers.example/",
					None,
					None,
					None,
				)
				.expect("valid")
				.genesis_hash()
				.to_owned(),
				balance: 2_000_000_000,
				accounts: HashMap::new(),
				fail: false,
				calls: Vec::new(),
			})),
		}
	}
}

impl Chain {
	pub fn with<T>(&self, change: impl FnOnce(&mut ChainState) -> T) -> T {
		change(&mut self.inner.lock().expect("the chain"))
	}

	/// Puts an account at `address`, owned by `owner`, holding `data`.
	pub fn put(&self, address: &str, owner: &str, data: &[u8]) {
		let account = json!({
			"lamports": 1_000_000,
			"owner": owner,
			"executable": false,
			"rentEpoch": 0,
			"space": data.len(),
			"data": [STANDARD.encode(data), "base64"],
		});

		self.with(|state| state.accounts.insert(address.to_owned(), account));
	}

	pub fn remove(&self, address: &str) {
		self.with(|state| state.accounts.remove(address));
	}

	fn answer(&self, method: &str, params: &Value) -> Option<Value> {
		let mut state = self.inner.lock().expect("the chain");

		state.calls.push((method.to_owned(), params.clone()));

		if state.fail {
			return None;
		}

		let context = json!({ "slot": 10 });
		let account = |address: &Value| {
			address
				.as_str()
				.and_then(|address| state.accounts.get(address))
				.cloned()
				.unwrap_or(Value::Null)
		};

		Some(match method {
			"getGenesisHash" => json!(state.genesis),
			"getBalance" => json!({ "context": context, "value": state.balance }),
			"getMultipleAccounts" => {
				let addresses = params[0].as_array().cloned().unwrap_or_default();

				json!({ "context": context, "value": addresses.iter().map(account).collect::<Vec<_>>() })
			}
			"getAccountInfo" => json!({ "context": context, "value": account(&params[0]) }),
			"getLatestBlockhash" => {
				json!({
					"context": context,
					"value": {
						"blockhash": "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG",
						"lastValidBlockHeight": 1000,
					},
				})
			}
			_ => return None,
		})
	}
}

impl Rpc for Chain {
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>> {
		let reply = (|| {
			let body: Value = serde_json::from_slice(&request.body).ok()?;
			let result = self.answer(body["method"].as_str()?, &body["params"])?;
			let reply = json!({ "jsonrpc": "2.0", "id": body["id"], "result": result });

			Some(Reply {
				status: 200,
				body: serde_json::to_vec(&reply).ok()?,
			})
		})();

		Box::pin(async move { reply.ok_or_else(|| TransportError::new(TransportFailureKind::Io)) })
	}
}

/// A Core asset's account data, as Metaplex Core writes it.
pub fn asset(
	owner: solana_kit_core::Address,
	authority: solana_kit_mpl_core::UpdateAuthority,
	uri: &str,
) -> Vec<u8> {
	use solana_kit_core::codecs::Encoder;

	solana_kit_mpl_core::AssetV1Codec
		.encode(
			&solana_kit_mpl_core::AssetV1 {
				owner,
				update_authority: authority,
				name: "Vesper".into(),
				uri: uri.into(),
				seq: None,
			},
			32 * 1024,
		)
		.expect("an asset encodes")
}

/// The collection test reapers belong to.
pub const COLLECTION: &str = "11111111111111111111111111111111";

/// The program test husks are escrowed with.
pub fn program() -> solana_kit_core::Address {
	reapers_server::escrow::program()
}

/// The chain settings tests read by.
pub fn solana() -> SolanaConfig {
	SolanaConfig::new(
		"solana:devnet",
		"https://api.devnet.solana.com",
		"https://reapers.example/",
		Some(COLLECTION),
		None,
		Some(&program().to_string()),
	)
	.expect("valid test settings")
}

/// A keeper on a database of its own.
pub struct World {
	pub db: Db,
	pub keeper: Arc<Keeper>,
	pub chain: Chain,
}

impl World {
	/// A development keeper with rifts that play through in well under a
	/// second.
	pub async fn new() -> Self {
		Self::with(RunMode::Test, quick_hall()).await
	}

	pub async fn with(mode: RunMode, hall: RiftHall) -> Self {
		Self::reading(mode, hall, solana()).await
	}

	/// A keeper in `mode` reading the chain `config` describes.
	pub async fn reading(mode: RunMode, hall: RiftHall, config: SolanaConfig) -> Self {
		let db = database().await;
		let chain = Chain::default();
		let keeper = Arc::new(Keeper::new(
			mode,
			&db,
			Gateway::new(config, chain.clone()),
			ReceiptSigner::development(),
			hall,
		));

		Self { db, keeper, chain }
	}

	/// A new player: an account in sign-in's tables.
	pub async fn player(&self) -> Id<Account> {
		reapers_server::sign_in::new_account(&self.db)
			.await
			.expect("an account")
	}

	/// Who `account` is, signed in, as an operation sees them.
	pub fn identity(&self, account: Option<Id<Account>>) -> Identity {
		let signed_in = account.map(|account| {
			SignedIn {
				account,
				session: Id::new(),
				aal: Aal::One,
				authenticated_at: leptoskit_db::now(),
				mfa_at: None,
			}
		});
		let context = leptoskit_auth::context(signed_in)
			.with(KeeperHandle(Arc::new(Shared(Arc::clone(&self.keeper)))));

		Identity::new(context)
	}

	/// Runs `op` as `account`, as the API would.
	pub async fn call<O: Operation>(&self, account: Id<Account>, op: O) -> rpc::Result<O::Output> {
		rpc::server::call_as(&self.db, self.identity(Some(account)), None, op).await
	}

	/// Runs `op` as nobody.
	pub async fn anonymous<O: Operation>(&self, op: O) -> rpc::Result<O::Output> {
		rpc::server::call_as(&self.db, self.identity(None), None, op).await
	}

	/// The stored books of `account`.
	pub async fn stored(&self, account: Id<Account>) -> PlayerState {
		reapers_server::keeper::opened(&self.db, account)
			.await
			.expect("the books")
	}

	/// Stores `state` as `account`'s books, as a test reaching into the
	/// database does.
	pub async fn store(&self, account: Id<Account>, state: PlayerState) {
		use reapers_server::tables::PlayerBooks;

		PlayerBooks::update()
			.filter(|books| books.account_id.eq(account))
			.set(|books| books.state, leptoskit_db::Json(state))
			.run(&self.db)
			.await
			.expect("the books are stored");
	}

	/// Moves every loose husk's surfacing time into the past, so a test does
	/// not have to wait out the real delay.
	pub async fn surface_everything(&self, account: Id<Account>) {
		let mut json = serde_json::to_value(self.stored(account).await).expect("a save");

		for sighting in json["sightings"].as_array_mut().expect("sightings") {
			sighting["surfacesAt"] = json!("2026-01-01T00:00:00.000Z");
		}

		self.store(account, serde_json::from_value(json).expect("a save"))
			.await;
	}
}

/// A hall that plays a rift through in well under a second, by moving it on
/// half a second at every beat. A real rift is two minutes long.
pub fn quick_hall() -> RiftHall {
	RiftHall::new(Duration::from_millis(4), Some(30))
}

/// The hunt for `sighting`, played by the shared bot as `state` would.
pub fn play(state: &PlayerState, sighting: &Sighting) -> HuntLog {
	let mut sim = HuntSim::open(
		sighting.seed,
		sighting.nft.collection.vessel(),
		sighting.nft.rarity().ward(),
		&HuntStats::derive(&state.loadout(), 1.0).expect("a valid loadout"),
	);
	let mut bot = HuntBot::careful(&sim);

	bot.play(&mut sim);

	HuntLog::new(bot.recorded)
}

/// `log`, packed for the `settle_hunt` operation.
pub fn packed(log: &HuntLog) -> String {
	reapers_api::player::encode_log(&log.encode())
}

/// Asserts `result` is the rules' `refusal`.
#[track_caller]
pub fn assert_refused<T: std::fmt::Debug>(result: rpc::Result<T>, refusal: Refusal) {
	match result {
		Err(error) => {
			assert_eq!(
				reapers_api::refusal_of(error.message()),
				Some(refusal),
				"refused with {error:?}"
			);
		}
		Ok(value) => panic!("expected {refusal:?}, got {value:?}"),
	}
}

/// Asserts `result` is unauthorized.
#[track_caller]
#[allow(
	clippy::needless_pass_by_value,
	reason = "it takes the call's result as it comes"
)]
pub fn assert_unauthorized<T: std::fmt::Debug>(result: rpc::Result<T>) {
	assert!(
		matches!(result, Err(rpc::Error::Unauthorized(_))),
		"expected unauthorized, got {result:?}"
	);
}
