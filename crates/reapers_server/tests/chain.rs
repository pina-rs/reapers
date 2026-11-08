//! The Rust side of `reapers_server/test/solana/solana_gateway_test.dart`,
//! with the escrow checks the Reapers program adds: the chain as the keeper
//! reads it, through Solana Kit, against controlled chain responses.

mod support;

use reapers_rules::ReaperGenome;
use reapers_rules::Refusal;
use reapers_server::chain::Gateway;
use reapers_server::config::SolanaConfig;
use reapers_server::escrow;
use reapers_server::escrow::Escrow;
use solana_kit_core::Address;
use solana_kit_mpl_core::UpdateAuthority;
use support::*;

fn genome() -> ReaperGenome {
	ReaperGenome::shuffle(11)
}

fn uri(genome: &ReaperGenome) -> String {
	format!("https://reapers.example/reapers/{}.json", genome.code())
}

fn collection() -> UpdateAuthority {
	UpdateAuthority::Collection(COLLECTION.parse().expect("an address"))
}

fn core() -> String {
	solana_kit_mpl_core::PROGRAM.to_string()
}

#[tokio::test]
async fn only_owned_core_assets_in_the_verified_collection_are_read_at_a_finalized_slot() {
	let chain = Chain::default();
	let gateway = Gateway::new(solana(), chain.clone());
	let wallet = Address::from_bytes([1; 32]);
	let other = Address::from_bytes([2; 32]);
	let addresses: Vec<String> = (0..5_u8)
		.map(|index| Address::from_bytes([index + 8; 32]).to_string())
		.collect();
	let genome = genome();

	chain.put(
		&addresses[0],
		&core(),
		&asset(wallet, collection(), &uri(&genome)),
	);
	chain.put(
		&addresses[1],
		&core(),
		&asset(other, collection(), &uri(&genome)),
	);
	chain.put(
		&addresses[2],
		&core(),
		&asset(wallet, UpdateAuthority::Address(wallet), &uri(&genome)),
	);
	// Owned by another program: not a Core asset, whatever it holds.
	chain.put(
		&addresses[3],
		&Address::from_bytes([3; 32]).to_string(),
		&asset(wallet, collection(), &uri(&genome)),
	);
	// addresses[4] does not exist.

	let owned = gateway
		.owned(&wallet.to_string(), &addresses)
		.await
		.expect("the chain answers");

	assert_eq!(owned.len(), 1);
	assert_eq!(owned.get(&addresses[0]), Some(&genome));

	let calls = chain.with(|state| state.calls.clone());
	let (method, params) = calls.last().expect("a read");

	assert_eq!(method, "getMultipleAccounts");
	assert_eq!(params[0], serde_json::json!(addresses));
	assert_eq!(params[1]["commitment"], "finalized");
	assert_eq!(params[1]["encoding"], "base64");
	assert_eq!(calls[0].0, "getGenesisHash", "the cluster is checked first");
}

#[tokio::test]
async fn the_wrong_cluster_refuses_the_operation_before_reading_assets() {
	let chain = Chain::default();
	let gateway = Gateway::new(solana(), chain.clone());

	chain.with(|state| {
		state.genesis = reapers_server::config::genesis_hash("solana:mainnet")
			.expect("known")
			.to_owned();
	});

	assert_eq!(
		gateway
			.owned(
				&Address::from_bytes([1; 32]).to_string(),
				&[COLLECTION.to_owned()]
			)
			.await,
		Err(Refusal::ChainUnconfigured)
	);
	assert_eq!(
		gateway.balance(&Address::from_bytes([1; 32])).await,
		Err(Refusal::ChainUnconfigured)
	);
	assert!(
		chain.with(|state| {
			state
				.calls
				.iter()
				.all(|(method, _)| method == "getGenesisHash")
		}),
		"nothing but the genesis was read"
	);
}

#[tokio::test]
async fn chain_failures_fail_closed() {
	let chain = Chain::default();
	let gateway = Gateway::new(solana(), chain.clone());

	chain.with(|state| state.fail = true);

	assert_eq!(
		gateway
			.owned(
				&Address::from_bytes([1; 32]).to_string(),
				&[COLLECTION.to_owned()]
			)
			.await,
		Err(Refusal::ChainUnavailable)
	);
	assert_eq!(
		gateway.balance(&Address::from_bytes([1; 32])).await,
		Err(Refusal::ChainUnavailable)
	);
	assert_eq!(
		gateway
			.escrowed(&Address::from_bytes([1; 32]), &Address::from_bytes([2; 32]))
			.await,
		Err(Refusal::ChainUnavailable)
	);
}

#[tokio::test]
async fn without_a_collection_no_reaper_can_be_owned() {
	let config = SolanaConfig::new(
		"solana:devnet",
		"https://api.devnet.solana.com",
		"https://reapers.example/",
		None,
		None,
		None,
	)
	.expect("valid");
	let gateway = Gateway::new(config, Chain::default());

	assert_eq!(
		gateway
			.owned(
				&Address::from_bytes([1; 32]).to_string(),
				&[COLLECTION.to_owned()]
			)
			.await,
		Err(Refusal::ChainUnconfigured)
	);
	assert_eq!(
		gateway
			.escrowed(&Address::from_bytes([1; 32]), &Address::from_bytes([2; 32]))
			.await,
		Err(Refusal::ChainUnconfigured),
		"no program, no escrow"
	);
}

#[test]
fn genome_uris_must_use_the_configured_origin_and_a_valid_permanent_code() {
	let gateway = Gateway::new(solana(), Chain::default());
	let genome = genome();

	assert_eq!(gateway.genome_from_uri(&uri(&genome)), Some(genome));

	for uri in [
		format!("https://evil.example/reapers/{}.json", genome.code()),
		format!(
			"https://reapers.example/reapers/../../{}.json",
			genome.code()
		),
		"https://reapers.example/reapers/R1-invalid.json".to_owned(),
		format!("https://reapers.example/reapers/{}", genome.code()),
	] {
		assert_eq!(gateway.genome_from_uri(&uri), None, "{uri}");
	}
}

#[tokio::test]
async fn an_escrow_holds_only_when_the_program_records_this_wallet_and_this_mint() {
	let chain = Chain::default();
	let gateway = Gateway::new(solana(), chain.clone());
	let wallet = Address::from_bytes([1; 32]);
	let other = Address::from_bytes([2; 32]);
	let mint = Address::from_bytes([5; 32]);
	let (address, bump) = escrow::address(&program(), &mint).expect("an escrow address");
	let record = |wallet: Address, mint: Address, bump: u8| Escrow { wallet, mint, bump }.encode();

	assert_eq!(
		gateway.escrowed(&wallet, &mint).await,
		Err(Refusal::NotInVault),
		"no escrow"
	);

	chain.put(
		&address.to_string(),
		&program().to_string(),
		&record(wallet, mint, bump),
	);

	assert_eq!(gateway.escrowed(&wallet, &mint).await, Ok(()));
	assert_eq!(
		gateway.escrowed(&other, &mint).await,
		Err(Refusal::NotInVault),
		"another wallet's escrow"
	);

	let calls = chain.with(|state| state.calls.clone());
	let (method, params) = calls.last().expect("a read");

	assert_eq!(method, "getAccountInfo");
	assert_eq!(params[0], address.to_string());
	assert_eq!(params[1]["commitment"], "finalized");

	for (label, owner, data) in [
		(
			"owned by another program",
			core(),
			record(wallet, mint, bump),
		),
		(
			"recording another mint",
			program().to_string(),
			record(wallet, other, bump),
		),
		(
			"with another bump",
			program().to_string(),
			record(wallet, mint, bump.wrapping_sub(1)),
		),
		("not an escrow", program().to_string(), [0; escrow::LEN]),
	] {
		chain.put(&address.to_string(), &owner, &data);

		assert_eq!(
			gateway.escrowed(&wallet, &mint).await,
			Err(Refusal::NotInVault),
			"{label}"
		);
	}
}
