//! The devnet flow end to end, through the game's own backend: the client
//! the browser and the apps use (`ServerApi`), the keeper over it
//! (`RemoteBackend`) and the cluster (`Cluster`), against a real keeper and
//! a real validator with the Reapers program on it. Only the transport is
//! the desktop's (`reqwest`) and the wallet a key in a file.
//!
//! It signs in as a guest, takes the oath, proves the wallet, releases one
//! of the wallet's demo husks into the program's escrow, waits for it to
//! surface, and settles a hunt the rules' careful bot played. The wallet
//! declines the burn the server built, as a player might, so the husk stays
//! in escrow; then the game finishes it, with the same receipt on a fresh
//! blockhash, and checks the husk is Core's one-byte tombstone.
//!
//! Ignored by default: it needs Surfpool, the Solana CLI and PostgreSQL.
//!
//! ```sh
//! cargo test -p reapers_app --test devnet -- --ignored --nocapture
//! ```
//!
//! On its own it builds `reapers_server` and `reapers_ops`, starts a
//! PostgreSQL cluster under `target/tmp`, brings up `reapers_ops localnet`
//! (Surfpool with the program, initialized with the development keeper, and
//! the demo husks minted to a funded player), and starts the server on it as
//! `solana:localnet`. It needs `target/deploy/reapers_program.so`,
//! `target/fixtures/mpl_core.so` and the program keypair, as
//! `reapers_ops rehearse` does.
//!
//! Against a deployed keeper on devnet, the post-deploy smoke test, name the
//! keeper and a wallet that owns demo husks and holds a little SOL:
//!
//! ```sh
//! REAPERS_E2E_SERVER_URL=https://keeper.example/ \
//! REAPERS_E2E_WALLET=~/.config/solana/reapers/devnet-player.json \
//! cargo test -p reapers_app --test devnet -- --ignored --nocapture
//! ```
//!
//! `REAPERS_E2E_RPC_URL` names the RPC (devnet's public one by default) and
//! `REAPERS_E2E_CLUSTER` the cluster (`devnet` by default). It burns one husk.

use std::net::TcpListener;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use reapers_app::backend::ChainStatus;
use reapers_app::backend::GameBackend;
use reapers_app::backend::KeeperResult;
use reapers_app::backend::RemoteBackend;
use reapers_app::backend::ServerApi;
use reapers_app::chain::ChainFailure;
use reapers_app::chain::ChainResult;
use reapers_app::chain::Cluster;
use reapers_app::chain::ClusterName;
use reapers_app::chain::HuskPlace;
use reapers_app::platform;
use reapers_app::platform::Header;
use reapers_app::platform::HttpReply;
use reapers_app::platform::HttpTransport;
use reapers_app::platform::MemoryPrefs;
use reapers_app::platform::PlatformError;
use reapers_app::simnet::SimnetInstruction;
use reapers_app::wallet::ChallengeFn;
use reapers_app::wallet::WalletProof;
use reapers_app::wallet::WalletProver;
use reapers_rules::HuntBot;
use reapers_rules::HuntLog;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use solana_kit_core::crypto::Keypair;
use solana_kit_core::transaction::Transaction;

/// The desktop's HTTP, standing in for `fetch` and the HTTP plugin.
struct Desktop(reqwest::Client);

#[async_trait(?Send)]
impl HttpTransport for Desktop {
	async fn post(
		&self,
		url: &str,
		headers: Vec<Header>,
		body: Vec<u8>,
		timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		let mut request = self.0.post(url).timeout(timeout).body(body);
		for (name, value) in headers {
			request = request.header(name, value);
		}
		let reply = request
			.send()
			.await
			.map_err(|error| PlatformError::Failed(error.to_string()))?;
		let status = reply.status().as_u16();
		let body = reply
			.bytes()
			.await
			.map_err(|error| PlatformError::Failed(error.to_string()))?;
		Ok(HttpReply {
			status,
			body: body.to_vec(),
		})
	}
}

/// A wallet whose key is in a file, as a wallet app holds one. Told to,
/// it declines the next transaction, and keeps what it was handed.
struct FileWallet {
	keypair: Keypair,
	decline_next: std::cell::Cell<bool>,
	declined: std::cell::RefCell<Option<Vec<u8>>>,
}

impl FileWallet {
	fn new(keypair: Keypair) -> Self {
		Self {
			keypair,
			decline_next: std::cell::Cell::new(false),
			declined: std::cell::RefCell::new(None),
		}
	}
}

#[async_trait(?Send)]
impl WalletProver for FileWallet {
	fn available(&self) -> bool {
		true
	}

	async fn prove(
		&self,
		_status: &ChainStatus,
		challenge: ChallengeFn<'_>,
	) -> KeeperResult<WalletProof> {
		let issued = challenge(self.keypair.address().to_string()).await?;
		Ok(WalletProof {
			challenge_id: issued.challenge_id,
			signature: BASE64.encode(self.keypair.sign(issued.message.as_bytes()).to_bytes()),
		})
	}

	async fn sign_transaction(
		&self,
		_status: &ChainStatus,
		transaction: Vec<u8>,
	) -> ChainResult<Vec<u8>> {
		if self.decline_next.replace(false) {
			*self.declined.borrow_mut() = Some(transaction);
			return Err(ChainFailure::Declined);
		}
		Transaction::decode(&transaction)
			.and_then(|transaction| transaction.sign(&self.keypair))
			.and_then(|signed| signed.encode(true))
			.map_err(|_| ChainFailure::WrongWallet)
	}
}

/// A Solana CLI keypair file: 64 bytes as a JSON array, the seed first.
fn read_keypair(path: &Path) -> Keypair {
	let bytes: Vec<u8> = serde_json::from_str(
		&std::fs::read_to_string(path)
			.unwrap_or_else(|error| panic!("{}: {error}", path.display())),
	)
	.expect("a keypair file is a JSON array of bytes");
	Keypair::from_seed(&bytes[..32].try_into().expect("a 32-byte seed"))
}

/// A process that is stopped when the test ends, however it ends.
struct Running {
	child: Child,
	/// Held open: `reapers_ops localnet` stops when it closes.
	stdin: Option<ChildStdin>,
}

impl Drop for Running {
	/// Closes standard input first, so `reapers_ops localnet` stops its
	/// Surfpool itself, and kills what does not stop.
	fn drop(&mut self) {
		if self.stdin.take().is_some() {
			let started = Instant::now();
			while started.elapsed() < Duration::from_secs(10) {
				if matches!(self.child.try_wait(), Ok(Some(_))) {
					return;
				}
				std::thread::sleep(Duration::from_millis(100));
			}
		}
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

/// A PostgreSQL cluster of its own, stopped and deleted at the end.
struct Postgres {
	directory: PathBuf,
	port: u16,
}

impl Drop for Postgres {
	fn drop(&mut self) {
		let _ = Command::new(tool("pg_ctl"))
			.arg("-D")
			.arg(&self.directory)
			.args(["stop", "-m", "immediate"])
			.output();
		let _ = std::fs::remove_dir_all(&self.directory);
	}
}

/// The PostgreSQL tool `name`, found on `PATH` (or in `REAPERS_PG_BIN`) and
/// followed through symbolic links, so one installed through a profile of
/// links still finds the files it was built beside, as the server's tests
/// find it.
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

fn free_port() -> u16 {
	TcpListener::bind("127.0.0.1:0")
		.and_then(|listener| listener.local_addr())
		.map(|address| address.port())
		.expect("a free port")
}

fn run(command: &mut Command) {
	let output = command.output().expect("the command runs");
	assert!(
		output.status.success(),
		"{command:?} failed:\n{}{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);
}

fn workspace() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn postgres() -> Postgres {
	let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
		.join(format!("reapers-e2e-postgres-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&directory);
	let port = free_port();
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
	let postgres = Postgres { directory, port };
	run(Command::new(tool("pg_ctl"))
		.arg("-D")
		.arg(&postgres.directory)
		.arg("-l")
		.arg(postgres.directory.join("log"))
		.arg("-o")
		.arg(format!(
			"-p {port} -c listen_addresses=127.0.0.1 -c unix_socket_directories='' -c fsync=off"
		))
		.args(["-w", "start"]));
	run(Command::new(tool("createdb")).args([
		"-h",
		"127.0.0.1",
		"-p",
		&port.to_string(),
		"-U",
		"postgres",
		"reapers",
	]));
	postgres
}

/// Everything the local run starts, and where the game finds it.
struct Local {
	server_url: String,
	rpc_url: String,
	// Dropped in reverse: the keeper, then the validator, then the database.
	_server: Running,
	_localnet: Running,
	_postgres: Postgres,
}

async fn wait_for(what: &str, mut ready: impl AsyncFnMut() -> bool) {
	let started = Instant::now();
	while !ready().await {
		assert!(
			started.elapsed() < Duration::from_secs(300),
			"{what} did not come up"
		);
		tokio::time::sleep(Duration::from_millis(500)).await;
	}
}

async fn local(http: &reqwest::Client) -> (Local, Keypair) {
	let root = workspace();
	println!("==> build reapers_server and reapers_ops");
	run(Command::new(env!("CARGO")).current_dir(&root).args([
		"build",
		"-q",
		"-p",
		"reapers_server",
		"-p",
		"reapers_ops",
	]));
	let binaries = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("../debug");

	println!("==> PostgreSQL");
	let postgres = postgres();

	println!("==> reapers_ops localnet");
	let rpc_port = loop {
		let port = free_port();
		if TcpListener::bind(("127.0.0.1", port + 1)).is_ok() {
			break port;
		}
	};
	let work = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
		.join(format!("reapers-e2e-localnet-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&work);
	let mut child = Command::new(binaries.join("reapers_ops"))
		.current_dir(&root)
		.args(["localnet", "--port", &rpc_port.to_string(), "--work-dir"])
		.arg(&work)
		.stdin(Stdio::piped())
		.spawn()
		.expect("reapers_ops starts");
	let stdin = child.stdin.take();
	let localnet = Running { child, stdin };
	let ready = work.join("localnet.json");
	wait_for("the localnet", async || ready.is_file()).await;
	let described: serde_json::Value =
		serde_json::from_str(&std::fs::read_to_string(&ready).unwrap()).unwrap();
	let rpc_url = described["url"].as_str().unwrap().to_owned();
	let wallet = read_keypair(Path::new(described["player_keypair"].as_str().unwrap()));

	println!("==> reapers_server on solana:localnet");
	let port = free_port();
	let server = Running {
		child: Command::new(binaries.join("reapers_server"))
			.env("REAPERS_RUN_MODE", "development")
			.env("REAPERS_LISTEN", format!("127.0.0.1:{port}"))
			.env("REAPERS_SOLANA_CHAIN", "solana:localnet")
			.env("REAPERS_SOLANA_RPC_URL", &rpc_url)
			.env("REAPERS_WALLET_IDENTITY_URI", "https://reapers.localhost/")
			.env("REAPERS_PROGRAM_ID", described["program"].as_str().unwrap())
			.env(
				"REAPERS_DEMO_HUSKS",
				described["manifest"].as_str().unwrap(),
			)
			.env(
				"REAPERS_SECRET_database",
				format!(
					"postgres://postgres@127.0.0.1:{}/reapers?sslmode=disable",
					postgres.port
				),
			)
			// The development keeper, which `localnet` initialized the
			// program with, whatever a local passwords file says.
			.env(
				"REAPERS_SECRET_receiptSigningKey",
				"reapers-development-receipt-key",
			)
			.spawn()
			.expect("reapers_server starts"),
		stdin: None,
	};
	let server_url = format!("http://127.0.0.1:{port}");
	let readyz = format!("{server_url}/readyz");
	wait_for("the keeper", async || {
		http.get(&readyz)
			.send()
			.await
			.is_ok_and(|reply| reply.status().is_success())
	})
	.await;

	(
		Local {
			server_url,
			rpc_url,
			_server: server,
			_localnet: localnet,
			_postgres: postgres,
		},
		wallet,
	)
}

#[test]
#[ignore = "needs Surfpool, the Solana CLI and PostgreSQL, or a deployed keeper"]
#[allow(clippy::too_many_lines, reason = "the flow, step by step")]
fn a_wallet_releases_a_real_husk_and_burns_it_after_the_hunt() {
	let runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.expect("a runtime");
	runtime.block_on(async {
		let http = reqwest::Client::new();
		let deployed = std::env::var("REAPERS_E2E_SERVER_URL").ok();
		let (local, server_url, rpc_url, wallet, cluster) = if let Some(server_url) = deployed {
			let cluster = std::env::var("REAPERS_E2E_CLUSTER")
				.ok()
				.and_then(|name| ClusterName::from_name(&name))
				.unwrap_or(ClusterName::Devnet);
			let rpc_url = std::env::var("REAPERS_E2E_RPC_URL")
				.unwrap_or_else(|_| cluster.default_rpc().to_owned());
			let wallet = read_keypair(Path::new(
				&std::env::var("REAPERS_E2E_WALLET")
					.expect("REAPERS_E2E_WALLET names the wallet's keypair file"),
			));
			(None, server_url, rpc_url, wallet, cluster)
		} else {
			let (local, wallet) = local(&http).await;
			let server_url = local.server_url.clone();
			let rpc_url = local.rpc_url.clone();
			(
				Some(local),
				server_url,
				rpc_url,
				wallet,
				ClusterName::Localnet,
			)
		};
		let address = wallet.address();
		println!("==> the game, against {server_url} on {rpc_url}, wallet {address}");

		let desktop: Rc<dyn HttpTransport> = Rc::new(Desktop(http.clone()));
		let pause: reapers_app::chain::Pause = Rc::new(|duration| {
			Box::pin(async move {
				tokio::time::sleep(duration).await;
				true
			})
		});
		let chain = Rc::new(Cluster::new(cluster, &rpc_url, desktop.clone()).with_pause(pause));
		let landed = Rc::new(std::cell::RefCell::new(Vec::new()));
		let failed = Rc::new(std::cell::RefCell::new(Vec::new()));
		chain.subscribe({
			let landed = landed.clone();
			move |transaction| landed.borrow_mut().push(transaction.clone())
		});
		chain.on_failure({
			let failed = failed.clone();
			move |failure| failed.borrow_mut().push(failure.clone())
		});
		let wallet = Rc::new(FileWallet::new(wallet));
		let keeper = RemoteBackend::new(ServerApi::new(
			&server_url,
			desktop,
			Rc::new(MemoryPrefs::new()),
		))
		.with_cluster(chain.clone(), wallet.clone());

		println!("==> sign in as a guest and take the oath");
		let player = keeper.load().await.expect("a guest's books");
		if !player.oath_taken() {
			keeper.take_oath().await.expect("the oath");
		}

		println!("==> prove the wallet");
		let status = keeper
			.chain_status()
			.await
			.expect("the chain's status")
			.expect("a server reads a chain");
		assert_eq!(status.chain, cluster.chain());
		assert!(!status.husks.is_empty(), "the keeper names the demo husks");
		keeper
			.connect_wallet(&status, wallet.as_ref())
			.await
			.expect("the wallet is proved");
		let status = keeper.chain_status().await.unwrap().unwrap();
		assert_eq!(status.wallet, Some(address.to_string()));

		println!("==> the wallet's husks");
		let husks = keeper.wallet_husks().await.expect("the wallet's husks");
		let husk = husks
			.iter()
			.filter(|husk| husk.place == HuskPlace::InWallet)
			.find(|husk| husk.nft.rarity() == Rarity::Common)
			.or_else(|| husks.iter().find(|husk| husk.place == HuskPlace::InWallet))
			.copied()
			.expect("the wallet holds a demo husk");
		println!("    {} is {}", husk.nft.title(), husk.asset);

		println!("==> release it: ReleaseCore, finalized, then the keeper");
		let released = keeper.release(husk.nft).await.expect("the release");
		let sighting = *released
			.sightings()
			.iter()
			.find(|sighting| sighting.nft == husk.nft)
			.expect("the husk is loose");
		let escrowed = keeper.wallet_husks().await.unwrap();
		assert!(
			escrowed
				.iter()
				.any(|held| held.asset == husk.asset && held.place == HuskPlace::Escrowed),
			"the husk is in the program's escrow"
		);
		assert_eq!(landed.borrow().len(), 1);
		assert_eq!(landed.borrow()[0].instruction, SimnetInstruction::Escrow);
		println!("    escrowed: {}", landed.borrow()[0].signature);

		println!("==> wait for it to surface");
		while platform::now() < sighting.surfaces_at {
			tokio::time::sleep(Duration::from_millis(500)).await;
		}
		tokio::time::sleep(Duration::from_secs(1)).await;

		println!("==> the rules' careful bot hunts it");
		let before = keeper.load().await.unwrap();
		let mut sim = HuntSim::open(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity(),
			sighting.nft.rarity().ward(),
			&HuntStats::derive(&before.loadout(), 1.0).expect("a valid loadout"),
		);
		let mut bot = HuntBot::careful(&sim);
		bot.play(&mut sim);
		let log = HuntLog::new(bot.recorded);

		println!("==> settle it; the wallet declines the burn");
		wallet.decline_next.set(true);
		let settlement = keeper
			.settle_hunt(sighting.id, &log)
			.await
			.expect("the settlement");
		assert!(settlement.outcome.absorbed(), "the bot reaps the husk");
		assert_eq!(*failed.borrow(), [ChainFailure::Declined]);
		assert_eq!(landed.borrow().len(), 1, "no burn landed");
		let entry = settlement
			.state
			.codex()
			.iter()
			.find(|entry| entry.nft == husk.nft)
			.cloned()
			.expect("the husk is in the codex");
		let declined = Transaction::decode(
			&wallet
				.declined
				.borrow()
				.clone()
				.expect("the wallet was handed the burn"),
		)
		.expect("a transaction");
		let stranded = keeper.wallet_husks().await.unwrap();
		assert!(
			stranded
				.iter()
				.any(|held| held.asset == husk.asset && held.place == HuskPlace::Escrowed),
			"the husk is still in escrow"
		);

		println!("==> finish the burn: the same receipt, built again");
		// The chain moves on, so the burn is built on a later blockhash.
		tokio::time::sleep(Duration::from_secs(2)).await;
		keeper
			.finish_burn(husk.nft)
			.await
			.expect("the burn is finished");
		assert_eq!(failed.borrow().len(), 1, "{:?}", failed.borrow());
		assert_eq!(landed.borrow().len(), 2);
		let burn = landed.borrow()[1].clone();
		assert_eq!(burn.instruction, SimnetInstruction::Burn);
		assert!(burn.receipt_verifies(), "the keeper's receipt verifies");
		assert!(burn.wire_verifies(), "the wallet signed the burn");
		assert_eq!(
			burn.receipt_signature
				.as_deref()
				.map(|signature| BASE64.encode(signature)),
			entry.signature,
			"the signature the codex holds"
		);
		let sent = Transaction::decode(burn.wire.as_deref().expect("the wire")).unwrap();
		assert_eq!(
			sent.message().instructions(),
			declined.message().instructions(),
			"the same receipt check and the same burn"
		);
		assert_ne!(
			sent.message().lifetime(),
			declined.message().lifetime(),
			"on a fresh blockhash"
		);
		println!("    burned: {}", burn.signature);

		println!("==> the husk is Core's one-byte tombstone");
		let after = keeper.wallet_husks().await.unwrap();
		assert!(
			after
				.iter()
				.any(|held| held.asset == husk.asset && held.place == HuskPlace::Burned),
			"the husk is burned"
		);
		println!("==> passed");
		drop(local);
	});
}
