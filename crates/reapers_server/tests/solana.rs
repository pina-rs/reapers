//! The Rust side of `reapers_server/test/integration/solana_endpoint_test.dart`
//! (wallet proof, account restoration, NFT reaper ownership and growth),
//! and the escrow and v2 receipt the Reapers program adds, against a real
//! PostgreSQL and controlled chain responses.

mod support;

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use leptoskit_auth::Account;
use leptoskit_db::Id;
use reapers_api::auth::WalletSignIn;
use reapers_api::chain::ChainStatus;
use reapers_api::chain::ImportReaper;
use reapers_api::chain::WalletChallenge;
use reapers_api::player::Acquire;
use reapers_api::player::Load;
use reapers_api::player::Release;
use reapers_api::player::SelectReaper;
use reapers_api::player::SettleHunt;
use reapers_api::player::TakeOath;
use reapers_rules::HuntBot;
use reapers_rules::HuntLog;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Judging;
use reapers_rules::Nft;
use reapers_rules::ReaperGenome;
use reapers_rules::Refusal;
use reapers_rules::market_listings;
use reapers_server::books::breach;
use reapers_server::config::RunMode;
use reapers_server::escrow;
use reapers_server::escrow::Escrow;
use reapers_server::escrow::HuskKind;
use reapers_server::keeper::now;
use reapers_server::receipt;
use reapers_server::receipt::Receipt;
use reapers_server::sign_in::sign_in_wallet;
use reapers_server::tables::HuskEscrow;
use reapers_server::tables::ReaperRecord;
use solana_kit_core::Address;
use solana_kit_core::Signature;
use solana_kit_core::crypto::Keypair;
use solana_kit_core::transaction::Transaction;
use solana_kit_mpl_core::UpdateAuthority;
use support::*;

const SAINT: &str = "degenApes:1";

struct Wallet(Keypair);

impl Wallet {
	fn new(seed: u8) -> Self {
		Self(Keypair::from_seed(&[seed; 32]))
	}

	fn address(&self) -> Address {
		self.0.address()
	}

	fn sign(&self, message: &str) -> String {
		STANDARD.encode(self.0.sign(message.as_bytes()).as_bytes())
	}
}

fn genome() -> ReaperGenome {
	ReaperGenome::shuffle(11)
}

/// The NFT reaper the tests trade.
fn reaper_asset() -> String {
	Address::from_bytes([9; 32]).to_string()
}

/// Puts the reaper asset on the chain, owned by `owner`.
fn own(chain: &Chain, owner: Address) {
	chain.put(
		&reaper_asset(),
		&solana_kit_mpl_core::PROGRAM.to_string(),
		&asset(
			owner,
			UpdateAuthority::Collection(COLLECTION.parse().expect("an address")),
			&format!("https://reapers.example/reapers/{}.json", genome().code()),
		),
	);
}

/// Proves `wallet` for `account`, and returns the account it signs in.
async fn prove(world: &World, account: Id<Account>, wallet: &Wallet) -> String {
	let challenge = world
		.call(
			account,
			WalletChallenge {
				address: wallet.address().to_string(),
			},
		)
		.await
		.expect("a challenge");

	sign_in_wallet(
		&world.keeper,
		account,
		&WalletSignIn {
			challenge_id: challenge.challenge_id,
			signature: wallet.sign(&challenge.message),
		},
	)
	.await
	.expect("the proof holds")
	.account
}

#[tokio::test]
async fn a_wallet_challenge_is_signed_expires_and_cannot_be_replayed_or_used_by_another_guest() {
	let world = World::new().await;
	let first = world.player().await;
	let second = world.player().await;
	let first_wallet = Wallet::new(1);
	let second_wallet = Wallet::new(2);
	let challenge = world
		.call(
			first,
			WalletChallenge {
				address: first_wallet.address().to_string(),
			},
		)
		.await
		.unwrap();
	let login = |account, signature| {
		let keeper = Arc::clone(&world.keeper);
		let challenge_id = challenge.challenge_id.clone();

		async move {
			sign_in_wallet(
				&keeper,
				account,
				&WalletSignIn {
					challenge_id,
					signature,
				},
			)
			.await
		}
	};

	assert!(challenge.message.contains("Chain ID: solana:devnet"));
	assert!(challenge.message.contains("does not authorize a payment"));
	assert!(
		challenge
			.message
			.starts_with("reapers.example wants you to sign in")
	);

	assert_refused(
		login(second, first_wallet.sign(&challenge.message)).await,
		Refusal::WalletUnproven,
	);
	assert_refused(
		login(first, second_wallet.sign(&challenge.message)).await,
		Refusal::WalletUnproven,
	);

	let signed_in = login(first, first_wallet.sign(&challenge.message))
		.await
		.unwrap();

	assert_eq!(signed_in.account, first.uuid().to_string());
	assert_refused(
		login(first, first_wallet.sign(&challenge.message)).await,
		Refusal::WalletUnproven,
	);

	let next = world
		.call(
			first,
			WalletChallenge {
				address: first_wallet.address().to_string(),
			},
		)
		.await
		.unwrap();

	reapers_server::tables::WalletChallenge::update()
		.filter(|row| row.challenge_id.eq(next.challenge_id.clone()))
		.set(|row| row.expires_at, chrono::DateTime::UNIX_EPOCH)
		.run(&world.db)
		.await
		.unwrap();

	assert_refused(
		sign_in_wallet(
			&world.keeper,
			first,
			&WalletSignIn {
				challenge_id: next.challenge_id,
				signature: first_wallet.sign(&next.message),
			},
		)
		.await,
		Refusal::WalletUnproven,
	);
}

#[tokio::test]
async fn a_wallet_challenge_needs_a_real_identity_origin_and_an_address() {
	let placeholder =
		reapers_server::config::SolanaConfig::from_vars(|_| None).expect("the defaults");
	let unconfigured = World::reading(RunMode::Test, quick_hall(), placeholder).await;
	let guest = unconfigured.player().await;

	assert_refused(
		unconfigured
			.call(
				guest,
				WalletChallenge {
					address: Wallet::new(1).address().to_string(),
				},
			)
			.await,
		Refusal::ChainUnconfigured,
	);

	let world = World::new().await;
	let player = world.player().await;

	assert_refused(
		world
			.call(
				player,
				WalletChallenge {
					address: "not an address".into(),
				},
			)
			.await,
		Refusal::WalletUnproven,
	);
}

#[tokio::test]
async fn a_wallet_restores_its_original_player_instead_of_merging_guest_progress() {
	let world = World::new().await;
	let first = world.player().await;
	let second = world.player().await;
	let wallet = Wallet::new(1);

	prove(&world, first, &wallet).await;
	world.call(first, TakeOath {}).await.unwrap();

	assert_eq!(
		prove(&world, second, &wallet).await,
		first.uuid().to_string()
	);
	assert!(!world.call(second, Load {}).await.unwrap().oath_taken());

	let status = world.call(first, ChainStatus {}).await.unwrap();

	assert_eq!(status.lamports, Some(2_000_000_000));
	assert_eq!(status.wallet, Some(wallet.address().to_string()));
	assert_eq!(status.chain, "solana:devnet");
	assert_eq!(status.collection.as_deref(), Some(COLLECTION));
	assert_eq!(status.program, Some(program().to_string()));
	assert_eq!(
		world.call(second, ChainStatus {}).await.unwrap().wallet,
		None
	);
}

#[tokio::test]
async fn a_linked_account_cannot_link_a_second_wallet() {
	let world = World::new().await;
	let player = world.player().await;

	prove(&world, player, &Wallet::new(1)).await;

	let other = Wallet::new(2);
	let challenge = world
		.call(
			player,
			WalletChallenge {
				address: other.address().to_string(),
			},
		)
		.await
		.unwrap();

	assert_refused(
		sign_in_wallet(
			&world.keeper,
			player,
			&WalletSignIn {
				challenge_id: challenge.challenge_id,
				signature: other.sign(&challenge.message),
			},
		)
		.await,
		Refusal::AlreadyOwned,
	);
}

#[tokio::test]
async fn verified_nft_growth_follows_a_transfer_and_the_old_owner_loses_use_of_it() {
	let world = World::new().await;
	let first = world.player().await;
	let second = world.player().await;
	let first_wallet = Wallet::new(1);
	let second_wallet = Wallet::new(2);
	let asset = reaper_asset();

	own(&world.chain, first_wallet.address());

	for (player, wallet) in [(first, &first_wallet), (second, &second_wallet)] {
		prove(&world, player, wallet).await;
		world.call(player, TakeOath {}).await.unwrap();
	}

	world
		.call(
			first,
			ImportReaper {
				asset: asset.clone(),
			},
		)
		.await
		.unwrap();
	world
		.call(first, SelectReaper { id: asset.clone() })
		.await
		.unwrap();

	// The NFT reaper absorbs a husk, so it has growth to carry.
	world
		.keeper
		.change_alone(first, |state| {
			let nft = market_listings()[0];
			let loose = breach(
				state
					.acquire(nft)
					.and_then(|state| state.release(nft, 19, now())),
			)?;
			let sighting = loose.sightings()[0];
			let mut sim = HuntSim::open(
				19,
				nft.collection.vessel(),
				nft.rarity().ward(),
				&HuntStats::derive(&loose.loadout(), 1.0).expect("a loadout"),
			);
			let mut bot = HuntBot::careful(&sim);

			bot.play(&mut sim);

			let judged = breach(loose.judge_hunt(
				sighting.id,
				&HuntLog::new(bot.recorded),
				sighting.surfaces_at.plus_seconds(30),
				23,
				&Judging::default(),
			))?;

			Ok(judged.state)
		})
		.await
		.unwrap();

	let before = world.call(first, Load {}).await.unwrap();
	let grown = before.active_reaper().unwrap().clone();

	assert!(before.xp() > 0);
	assert_eq!(grown.memory().unseen, 1);

	own(&world.chain, second_wallet.address());

	let imported = world
		.call(
			second,
			ImportReaper {
				asset: asset.clone(),
			},
		)
		.await
		.unwrap();

	assert_eq!(imported.reapers().last().unwrap().xp(), before.xp());
	assert_eq!(imported.reapers().last().unwrap().memory(), grown.memory());

	let selected = world
		.call(second, SelectReaper { id: asset.clone() })
		.await
		.unwrap();

	assert_eq!(selected.genome(), genome());
	assert_eq!(selected.xp(), before.xp());
	assert_eq!(selected.active_reaper().unwrap().memory(), grown.memory());
	assert_eq!(selected.anima(), 0, "anima stays with the account");

	assert_refused(
		world.call(first, SelectReaper { id: asset }).await,
		Refusal::NotOwned,
	);
	assert_eq!(world.call(first, Load {}).await.unwrap().reapers().len(), 1);

	let records = ReaperRecord::query().all(&world.db).await.unwrap();

	assert_eq!(records.len(), 1);
	assert_eq!(records[0].state.0.xp(), before.xp());
}

#[tokio::test]
async fn unlinked_wallets_wrong_owners_and_chain_failures_cannot_import_or_erase_growth() {
	let world = World::new().await;
	let player = world.player().await;
	let wallet = Wallet::new(1);
	let asset = reaper_asset();

	world.call(player, TakeOath {}).await.unwrap();
	own(&world.chain, wallet.address());

	assert_refused(
		world
			.call(
				player,
				ImportReaper {
					asset: asset.clone(),
				},
			)
			.await,
		Refusal::WalletUnproven,
	);

	prove(&world, player, &wallet).await;
	own(&world.chain, Wallet::new(2).address());

	assert_refused(
		world
			.call(
				player,
				ImportReaper {
					asset: asset.clone(),
				},
			)
			.await,
		Refusal::NotOwned,
	);

	own(&world.chain, wallet.address());
	world
		.call(
			player,
			ImportReaper {
				asset: asset.clone(),
			},
		)
		.await
		.unwrap();

	assert_refused(
		world
			.call(
				player,
				ImportReaper {
					asset: asset.clone(),
				},
			)
			.await,
		Refusal::AlreadyOwned,
	);
	assert_refused(
		world
			.call(
				player,
				ImportReaper {
					asset: "not an address".into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);

	world.chain.with(|state| state.fail = true);

	assert_refused(world.call(player, Load {}).await, Refusal::ChainUnavailable);
	assert_eq!(world.stored(player).await.reapers().len(), 2);
	assert_eq!(ReaperRecord::query().count(&world.db).await.unwrap(), 1);
}

/// The husk mint the escrow tests release.
fn mint() -> Address {
	Address::from_bytes([5; 32])
}

/// Escrows `mint` on the chain, released by `wallet`: the program's escrow
/// account, and the Core asset it now owns, in a demo husk collection.
fn escrow_on_chain(chain: &Chain, wallet: Address, mint: Address) {
	let (address, bump) = escrow::address_of(&mint);

	chain.put(
		&address.to_string(),
		&program().to_string(),
		&Escrow {
			wallet,
			mint,
			kind: HuskKind::Core,
			bump,
		}
		.encode(),
	);
	chain.put(
		&mint.to_string(),
		&solana_kit_mpl_core::PROGRAM.to_string(),
		&asset(
			address,
			UpdateAuthority::Collection(husk_collection()),
			"https://reapers.example/husks/1.json",
		),
	);
}

/// The demo husk collection the escrow tests' husk belongs to.
fn husk_collection() -> Address {
	Address::from_bytes([6; 32])
}

/// A proved, oathbound player with the saint bought.
async fn holder(world: &World, wallet: &Wallet) -> Id<Account> {
	let player = world.player().await;

	prove(world, player, wallet).await;
	world.call(player, TakeOath {}).await.unwrap();
	world
		.call(player, Acquire { nft: SAINT.into() })
		.await
		.unwrap();
	player
}

#[tokio::test]
async fn an_escrowed_husk_earns_a_receipt_the_program_can_burn_it_with() {
	let world = World::new().await;
	let wallet = Wallet::new(1);
	let player = holder(&world, &wallet).await;

	escrow_on_chain(&world.chain, wallet.address(), mint());

	let released = world
		.call(
			player,
			Release {
				nft: SAINT.into(),
				escrow: Some(mint().to_string()),
			},
		)
		.await
		.unwrap();
	let sighting = released.sightings()[0];
	let bound = HuskEscrow::query().all(&world.db).await.unwrap();

	assert_eq!(bound.len(), 1);
	assert_eq!(bound[0].sighting_id, sighting.id);
	assert_eq!(bound[0].mint, mint().to_string());

	world.surface_everything(player).await;

	let before = world.call(player, Load {}).await.unwrap();
	let settled = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &sighting)),
			},
		)
		.await
		.unwrap();
	let entry = settled.settlement.state.codex()[0].clone();
	let chain_receipt = settled.receipt.expect("a receipt for the chain");
	let message = STANDARD.decode(&chain_receipt.message).unwrap();
	let receipt = Receipt::decode(&message).expect("a v2 receipt");
	let signature = Signature::from_bytes(
		STANDARD
			.decode(&chain_receipt.signature)
			.unwrap()
			.try_into()
			.unwrap(),
	);
	let keeper = world.keeper.signer();

	assert_eq!(
		receipt,
		receipt::of(*wallet.address().as_bytes(), *mint().as_bytes(), &entry).unwrap()
	);
	assert_eq!(
		receipt.player,
		*wallet.address().as_bytes(),
		"made out to the wallet"
	);
	assert_eq!(receipt.mint, *mint().as_bytes(), "for the real mint");
	assert!(keeper.verifies(&message, &signature));
	assert_eq!(
		entry.signature.as_deref(),
		Some(chain_receipt.signature.as_str())
	);
	assert_eq!(chain_receipt.keeper, keeper.address().to_string());
	assert_eq!(chain_receipt.wallet, wallet.address().to_string());
	assert_eq!(chain_receipt.mint, mint().to_string());

	let transaction = Transaction::decode(
		&STANDARD
			.decode(chain_receipt.transaction.expect("an absorb transaction"))
			.unwrap(),
	)
	.expect("a transaction");
	let instructions = transaction.message().instructions();

	assert_eq!(transaction.message().signers(), &[wallet.address()]);
	assert_eq!(instructions.len(), 2);
	assert!(
		instructions[0].data().ends_with(&message),
		"the Ed25519 check carries the receipt"
	);
	assert!(
		transaction
			.message()
			.static_accounts()
			.contains(&husk_collection()),
		"the burn writes to the husk's collection"
	);
	assert!(transaction.sign(&wallet.0).unwrap().verify().is_ok());
}

#[tokio::test]
async fn a_release_into_an_escrow_that_is_not_there_changes_nothing() {
	let world = World::new().await;
	let wallet = Wallet::new(1);
	let player = holder(&world, &wallet).await;
	let before = world.stored(player).await;

	assert_refused(
		world
			.call(
				player,
				Release {
					nft: SAINT.into(),
					escrow: Some(mint().to_string()),
				},
			)
			.await,
		Refusal::NotInVault,
	);

	escrow_on_chain(&world.chain, Wallet::new(2).address(), mint());

	assert_refused(
		world
			.call(
				player,
				Release {
					nft: SAINT.into(),
					escrow: Some(mint().to_string()),
				},
			)
			.await,
		Refusal::NotInVault,
	);
	assert_refused(
		world
			.call(
				player,
				Release {
					nft: SAINT.into(),
					escrow: Some("not a mint".into()),
				},
			)
			.await,
		Refusal::NotInVault,
	);
	assert_eq!(world.stored(player).await, before);
	assert_eq!(HuskEscrow::query().count(&world.db).await.unwrap(), 0);
}

#[tokio::test]
async fn an_escrow_needs_a_proved_wallet_and_binds_one_husk() {
	let world = World::new().await;
	let guest = world.player().await;

	world.call(guest, TakeOath {}).await.unwrap();
	world
		.call(guest, Acquire { nft: SAINT.into() })
		.await
		.unwrap();
	escrow_on_chain(&world.chain, Wallet::new(1).address(), mint());

	assert_refused(
		world
			.call(
				guest,
				Release {
					nft: SAINT.into(),
					escrow: Some(mint().to_string()),
				},
			)
			.await,
		Refusal::WalletUnproven,
	);

	let wallet = Wallet::new(1);
	let player = holder(&world, &wallet).await;
	let other = "degenApes:2";

	world
		.call(player, Acquire { nft: other.into() })
		.await
		.unwrap();
	world
		.call(
			player,
			Release {
				nft: SAINT.into(),
				escrow: Some(mint().to_string()),
			},
		)
		.await
		.unwrap();

	assert_refused(
		world
			.call(
				player,
				Release {
					nft: other.into(),
					escrow: Some(mint().to_string()),
				},
			)
			.await,
		Refusal::AlreadyHeld,
	);
	assert!(
		world
			.stored(player)
			.await
			.vault()
			.contains(&Nft::parse(other).unwrap()),
		"the second release changed nothing"
	);
}

#[tokio::test]
async fn a_husk_that_left_its_escrow_cannot_be_settled() {
	let world = World::new().await;
	let wallet = Wallet::new(1);
	let player = holder(&world, &wallet).await;
	let (address, _) = escrow::address_of(&mint());

	escrow_on_chain(&world.chain, wallet.address(), mint());

	let released = world
		.call(
			player,
			Release {
				nft: SAINT.into(),
				escrow: Some(mint().to_string()),
			},
		)
		.await
		.unwrap();
	let sighting = released.sightings()[0];

	world.surface_everything(player).await;

	let before = world.call(player, Load {}).await.unwrap();

	world.chain.remove(&address.to_string());

	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: sighting.id,
					log: packed(&play(&before, &sighting)),
				},
			)
			.await,
		Refusal::NotInVault,
	);
	assert_eq!(world.stored(player).await, before);
}
