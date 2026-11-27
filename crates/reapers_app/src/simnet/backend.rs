//! The offline game, with every step the chain is meant to take posted to
//! the [`SimnetChain`]: the airdrop when a reaper is bound, a purchase, the
//! escrow that releases a husk, and the burn that absorbs it.
//!
//! The books are kept by a [`LocalBackend`] under a key of their own, so the
//! rules, and the player's progress, are exactly the offline game's. The
//! chain is told what happened after the rules allow it; it never decides.

use std::rc::Rc;

use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use reapers_rules::Absorption;
use reapers_rules::Art;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntSettlement;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::RiftSite;
use reapers_rules::Weapon;

use super::SimnetAccount;
use super::SimnetChain;
use super::SimnetInstruction;
use super::SimnetPost;
use super::SimnetRole;
use crate::backend::GameBackend;
use crate::backend::KeeperError;
use crate::backend::KeeperKind;
use crate::backend::KeeperResult;
use crate::backend::LocalBackend;
use crate::backend::RiftSession;
use crate::receipt;

/// The simnet's keeper.
pub struct SimnetBackend {
	books: LocalBackend,
	chain: Rc<SimnetChain>,
}

impl SimnetBackend {
	/// Where the simnet's player is stored, apart from the offline game's.
	pub const STORAGE_KEY: &'static str = "reapers.simnet.player.v2";

	/// A keeper over `books` (whose clock, dice and store it keeps), posting
	/// to `chain`.
	pub fn new(books: LocalBackend, chain: Rc<SimnetChain>) -> Self {
		let signer = chain.clone();
		let books = books
			.with_storage_key(Self::STORAGE_KEY)
			.with_receipts(Rc::new(move |absorption| {
				BASE64.encode(SimnetChain::sign_receipt(&receipt_for(&signer, absorption)))
			}));
		Self { books, chain }
	}

	pub fn chain(&self) -> &Rc<SimnetChain> {
		&self.chain
	}

	fn wallet(&self) -> SimnetAccount {
		SimnetAccount::new(SimnetRole::Wallet, &self.chain.wallet())
	}

	/// Posts `post` to the chain, and puts the books back as `before` when the
	/// chain refuses it: the ledger and the books must tell the same story,
	/// so a player never keeps a husk the ledger will not release, nor a
	/// settlement the ledger will not burn.
	async fn post(&self, post: SimnetPost, before: PlayerState) -> KeeperResult<()> {
		if let Err(error) = self.chain.submit(post).await {
			self.books.restore(before).await?;
			return Err(error);
		}
		Ok(())
	}
}

/// The receipt for `absorption`: the player is the burner wallet, and the
/// husk the address that stands for its NFT.
fn receipt_for(chain: &SimnetChain, absorption: &Absorption) -> Vec<u8> {
	let wallet = chain
		.wallet()
		.parse::<solana_kit_core::Address>()
		.map(|wallet| *wallet.as_bytes())
		.unwrap_or_default();
	receipt::of(
		wallet,
		*SimnetChain::mint_for(absorption.nft).as_bytes(),
		absorption,
	)
	.map(|receipt| receipt.encode().to_vec())
	.unwrap_or_default()
}

#[async_trait(?Send)]
impl GameBackend for SimnetBackend {
	fn kind(&self) -> KeeperKind {
		KeeperKind::Simnet
	}

	async fn load(&self) -> KeeperResult<PlayerState> {
		self.books.load().await
	}

	async fn locate(&self, fix: Option<GeoFix>) -> KeeperResult<()> {
		self.books.locate(fix).await
	}

	async fn take_oath(&self) -> KeeperResult<PlayerState> {
		let before = self.books.load().await?;
		let state = self.books.take_oath().await?;
		self.post(
			SimnetPost {
				lamports: state.lamports(),
				..SimnetPost::new(
					SimnetInstruction::Airdrop,
					vec![
						SimnetAccount::new(SimnetRole::Faucet, &SimnetChain::faucet()),
						self.wallet(),
					],
				)
			},
			before,
		)
		.await?;
		Ok(state)
	}

	async fn select_reaper(&self, id: &str) -> KeeperResult<PlayerState> {
		self.books.select_reaper(id).await
	}

	async fn acquire(&self, nft: Nft) -> KeeperResult<PlayerState> {
		let before = self.books.load().await?;
		let state = self.books.acquire(nft).await?;
		self.post(
			SimnetPost {
				lamports: nft.rarity().lamports(),
				nft: Some(nft),
				..SimnetPost::new(
					SimnetInstruction::Buy,
					vec![
						self.wallet(),
						SimnetAccount::new(SimnetRole::Market, &SimnetChain::market()),
						SimnetAccount::new(SimnetRole::Mint, &SimnetChain::mint_for(nft)),
					],
				)
			},
			before,
		)
		.await?;
		Ok(state)
	}

	async fn release(&self, nft: Nft) -> KeeperResult<PlayerState> {
		let before = self.books.load().await?;
		let state = self.books.release(nft).await?;
		self.post(
			SimnetPost {
				nft: Some(nft),
				..SimnetPost::new(
					SimnetInstruction::Escrow,
					vec![
						self.wallet(),
						SimnetAccount::new(SimnetRole::Mint, &SimnetChain::mint_for(nft)),
						SimnetAccount::new(SimnetRole::Escrow, &SimnetChain::escrow_for(nft)),
						SimnetAccount::new(SimnetRole::Program, &SimnetChain::program()),
					],
				)
			},
			before,
		)
		.await?;
		Ok(state)
	}

	/// Settles the hunt. An absorbed husk is burned out of escrow, and the
	/// burn carries the receipt the keeper signed; one that got away stays in
	/// escrow, still huntable.
	async fn settle_hunt(&self, sighting_id: i64, log: &HuntLog) -> KeeperResult<HuntSettlement> {
		let before = self.books.load().await?;
		let settlement = self.books.settle_hunt(sighting_id, log).await?;
		if !settlement.outcome.absorbed() {
			return Ok(settlement);
		}

		let absorption = settlement
			.state
			.active_reaper()
			.and_then(|reaper| reaper.codex().last())
			.ok_or_else(|| KeeperError::unavailable("an absorption the codex does not hold"))?;
		let signature = absorption
			.signature
			.as_deref()
			.and_then(|signature| BASE64.decode(signature).ok())
			.ok_or_else(|| KeeperError::unavailable("an absorption without its receipt"))?;
		let nft = absorption.nft;

		self.post(
			SimnetPost {
				nft: Some(nft),
				receipt: Some(receipt_for(&self.chain, absorption)),
				receipt_signature: Some(signature),
				..SimnetPost::new(
					SimnetInstruction::Burn,
					vec![
						self.wallet(),
						SimnetAccount::new(SimnetRole::Mint, &SimnetChain::mint_for(nft)),
						SimnetAccount::new(SimnetRole::Escrow, &SimnetChain::escrow_for(nft)),
						SimnetAccount::new(SimnetRole::Program, &SimnetChain::program()),
						SimnetAccount::new(SimnetRole::Keeper, &SimnetChain::keeper()),
					],
				)
			},
			before,
		)
		.await?;
		Ok(settlement)
	}

	async fn buy_weapon(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.books.buy_weapon(weapon).await
	}

	async fn wield(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.books.wield(weapon).await
	}

	async fn buy_outfit(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.books.buy_outfit(outfit).await
	}

	async fn wear(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.books.wear(outfit).await
	}

	async fn attune(&self, art: Art) -> KeeperResult<PlayerState> {
		self.books.attune(art).await
	}

	async fn enter_rift(&self, site: RiftSite) -> KeeperResult<Box<dyn RiftSession>> {
		self.books.enter_rift(site).await
	}

	/// Starts over with a new wallet and a new player, for the next person
	/// to pick the device up.
	async fn forget(&self) -> KeeperResult<()> {
		self.books.forget_player().await?;
		self.chain.reset().await
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;
	use std::time::Duration;

	use futures::executor::block_on;
	use reapers_rules::HuntBot;
	use reapers_rules::HuntSim;
	use reapers_rules::HuntStats;
	use reapers_rules::NftCollection;
	use reapers_rules::Refusal;
	use reapers_rules::Timestamp;
	use solana_kit_core::Address;

	use super::*;
	use crate::backend::SeededDice;
	use crate::chain::ChainFailure;
	use crate::platform::MemoryPrefs;
	use crate::platform::PlatformError;
	use crate::platform::Prefs;

	const SAINT: Nft = Nft::new(NftCollection::DegenApes, 1);

	struct World {
		prefs: Rc<MemoryPrefs>,
		now: Rc<Cell<Timestamp>>,
		/// Whether the simnet's ledger still takes new transactions.
		ledger_open: Rc<Cell<bool>>,
	}

	impl World {
		fn new() -> Self {
			Self {
				prefs: Rc::new(MemoryPrefs::new()),
				now: Rc::new(Cell::new(Timestamp::utc(2026, 11, 4, 12, 0, 0))),
				ledger_open: Rc::new(Cell::new(true)),
			}
		}

		fn clock(&self) -> Rc<dyn Fn() -> Timestamp> {
			let now = self.now.clone();
			Rc::new(move || now.get())
		}

		fn chain(&self) -> Rc<SimnetChain> {
			Rc::new(
				block_on(SimnetChain::open(
					Rc::new(Ledger {
						inner: self.prefs.clone(),
						open: self.ledger_open.clone(),
					}),
					self.clock(),
					Duration::ZERO,
				))
				.unwrap(),
			)
		}

		/// Stops the ledger taking transactions, as a store that is full or
		/// locked does.
		fn close_ledger(&self) {
			self.ledger_open.set(false);
		}

		fn books(&self) -> LocalBackend {
			let clock = self.clock();
			LocalBackend::new(self.prefs.clone())
				.with_clock(move || clock())
				.with_dice(SeededDice::new(5))
		}

		fn keeper(&self, chain: Rc<SimnetChain>) -> SimnetBackend {
			SimnetBackend::new(self.books(), chain)
		}

		fn later(&self, seconds: i64) {
			self.now.set(self.now.get().plus_seconds(seconds));
		}
	}

	/// The device's preferences, with the simnet's ledger refusing new
	/// transactions once it is closed.
	struct Ledger {
		inner: Rc<MemoryPrefs>,
		open: Rc<Cell<bool>>,
	}

	#[async_trait::async_trait(?Send)]
	impl Prefs for Ledger {
		async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, PlatformError> {
			self.inner.get(key).await
		}

		async fn set(&self, key: &str, value: serde_json::Value) -> Result<(), PlatformError> {
			if key == SimnetChain::STORAGE_KEY && !self.open.get() {
				return Err(PlatformError::Unavailable("the ledger is closed"));
			}
			self.inner.set(key, value).await
		}

		async fn remove(&self, key: &str) -> Result<(), PlatformError> {
			self.inner.remove(key).await
		}
	}

	/// Plays the one loose husk with the shared bot and settles it.
	fn play(keeper: &SimnetBackend) -> KeeperResult<HuntSettlement> {
		let state = block_on(keeper.load())?;
		let [sighting] = state.sightings() else {
			return Err(KeeperError::unavailable("expected one sighting"));
		};
		let mut sim = HuntSim::open(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity(),
			sighting.nft.rarity().ward(),
			&HuntStats::derive(&state.loadout(), 1.0).expect("a loadout the stats can read"),
		);
		let mut bot = HuntBot::careful(&sim);
		bot.play(&mut sim);
		block_on(keeper.settle_hunt(sighting.id, &HuntLog::new(bot.recorded)))
	}

	/// Plays the one loose husk with the shared bot and settles it.
	fn hunt(keeper: &SimnetBackend) -> HuntSettlement {
		play(keeper).expect("the hunt settles")
	}

	#[test]
	fn binding_a_reaper_airdrops_the_starting_sol_into_a_burner_wallet_with_a_real_address() {
		let world = World::new();
		let chain = world.chain();
		let state = block_on(world.keeper(chain.clone()).take_oath()).unwrap();
		let [airdrop] = &chain.transactions()[..] else {
			panic!("expected one transaction");
		};

		assert!(chain.wallet().parse::<Address>().is_ok());
		assert_eq!(airdrop.instruction, SimnetInstruction::Airdrop);
		assert_eq!(airdrop.lamports, state.lamports());
		assert_eq!(
			airdrop.account(SimnetRole::Wallet),
			Some(chain.wallet().as_str())
		);
		assert!(
			airdrop.wire.is_none(),
			"a faucet pays an airdrop; no wallet signs one"
		);
		assert!(
			airdrop
				.signature
				.parse::<solana_kit_core::Signature>()
				.is_ok()
		);
	}

	#[test]
	fn a_purchase_a_release_and_an_absorption_are_a_buy_an_escrow_and_a_burn_that_carries_a_receipt_the_keeper_signed()
	 {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		let settlement = hunt(&keeper);

		assert!(settlement.outcome.absorbed());
		let transactions = chain.transactions();
		let instructions: Vec<_> = transactions
			.iter()
			.map(|transaction| transaction.instruction)
			.collect();
		assert_eq!(
			instructions,
			[
				SimnetInstruction::Airdrop,
				SimnetInstruction::Buy,
				SimnetInstruction::Escrow,
				SimnetInstruction::Burn
			]
		);

		let [_, buy, escrow, burn] = &transactions[..] else {
			unreachable!()
		};
		let escrow_address = SimnetChain::escrow_for(SAINT).to_string();

		assert_eq!(buy.lamports, SAINT.rarity().lamports());
		assert_eq!(buy.nft, Some(SAINT));
		assert_eq!(
			escrow.account(SimnetRole::Escrow),
			Some(escrow_address.as_str())
		);
		assert_eq!(
			burn.account(SimnetRole::Escrow),
			Some(escrow_address.as_str())
		);
		assert!(burn.slot > escrow.slot, "a minute later");

		let absorption = settlement
			.state
			.active_reaper()
			.unwrap()
			.codex()
			.last()
			.unwrap();
		assert!(absorption.signature.is_some(), "the codex keeps it");
		assert_eq!(
			burn.receipt.as_deref(),
			Some(&receipt_for(&chain, absorption)[..])
		);
		assert!(burn.receipt_verifies());
	}

	#[test]
	fn every_transaction_the_wallet_signs_is_a_real_wire_transaction() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		hunt(&keeper);

		for transaction in &chain.transactions()[1..] {
			assert!(transaction.wire_verifies(), "{:?}", transaction.instruction);
			let decoded = solana_kit_core::transaction::Transaction::decode(
				transaction.wire.as_deref().unwrap(),
			)
			.unwrap();
			assert_eq!(
				decoded.signatures()[0].map(|signature| signature.to_string()),
				Some(transaction.signature.clone())
			);
		}

		let rebuilt = chain
			.rebuild(
				&chain.transactions()[3],
				solana_kit_core::Hash::from_bytes([9; 32]),
			)
			.unwrap()
			.unwrap();
		assert!(
			solana_kit_core::transaction::Transaction::decode(&rebuilt)
				.unwrap()
				.verify()
				.is_ok()
		);
	}

	#[test]
	fn a_receipt_changed_after_signing_fails_the_check() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		hunt(&keeper);

		let mut forged = chain.transactions().last().unwrap().clone();
		forged.receipt.as_mut().unwrap()[73] ^= 1;
		assert!(!forged.receipt_verifies());
	}

	#[test]
	fn a_refused_request_posts_nothing() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		let refused = block_on(keeper.acquire(SAINT)).unwrap_err();
		assert_eq!(refused.refusal(), Some(Refusal::AlreadyHeld));
		assert_eq!(chain.transactions().len(), 2);
	}

	#[test]
	fn the_wallet_and_its_ledger_survive_a_restart() {
		let world = World::new();
		let first = world.chain();
		block_on(world.keeper(first.clone()).take_oath()).unwrap();

		let reopened = world.chain();
		assert_eq!(reopened.wallet(), first.wallet());
		assert_eq!(reopened.transactions(), first.transactions());
	}

	#[test]
	fn starting_over_makes_a_new_wallet_and_a_new_player_and_leaves_the_offline_game_alone() {
		let world = World::new();
		let offline = world.books();
		block_on(offline.take_oath()).unwrap();
		let offline_save = block_on(world.prefs.get(LocalBackend::DEFAULT_STORAGE_KEY)).unwrap();

		let chain = world.chain();
		let keeper = world.keeper(chain.clone());
		let before = chain.wallet();
		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.forget()).unwrap();

		assert_ne!(chain.wallet(), before);
		assert_eq!(chain.transactions(), []);
		assert!(!block_on(keeper.load()).unwrap().oath_taken());
		assert_eq!(
			block_on(world.prefs.get(LocalBackend::DEFAULT_STORAGE_KEY)).unwrap(),
			offline_save
		);
	}

	#[test]
	fn simnet_submit_refuses_burn_with_forged_receipt_signature() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		hunt(&keeper);

		let valid_burn = chain.transactions().last().unwrap().clone();
		let mut forged_sig = valid_burn.receipt_signature.clone().unwrap();
		forged_sig[0] ^= 0xFF;

		let result = block_on(chain.submit(SimnetPost {
			nft: Some(SAINT),
			receipt: valid_burn.receipt.clone(),
			receipt_signature: Some(forged_sig),
			..SimnetPost::new(SimnetInstruction::Burn, valid_burn.accounts.clone())
		}));
		assert!(
			result.is_err(),
			"SimnetChain::submit must verify the Ed25519 receipt signature before recording a burn (docs/solana.md:78)"
		);
	}

	#[test]
	fn simnet_submit_refuses_a_burn_whose_receipt_is_not_the_ninety_eight_bytes() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		hunt(&keeper);

		let valid_burn = chain.transactions().last().unwrap().clone();
		let mut malformed = valid_burn.receipt.clone().unwrap();
		malformed.truncate(crate::receipt::LEN - 1);
		let refused = block_on(chain.submit(SimnetPost {
			nft: Some(SAINT),
			receipt: Some(malformed),
			receipt_signature: valid_burn.receipt_signature.clone(),
			..SimnetPost::new(SimnetInstruction::Burn, valid_burn.accounts.clone())
		}))
		.unwrap_err();

		assert!(
			matches!(refused, KeeperError::Chain(ChainFailure::Rejected(_))),
			"a receipt the program cannot read must be refused, not recorded: {refused}"
		);
		assert_eq!(
			chain.transactions().len(),
			4,
			"the refused burn is not on the ledger"
		);
	}

	#[test]
	fn simnet_submit_refuses_a_burn_carrying_another_husks_receipt() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		hunt(&keeper);

		let valid_burn = chain.transactions().last().unwrap().clone();
		let other = Nft::new(NftCollection::DegenApes, 2);
		let refused = block_on(chain.submit(SimnetPost {
			nft: Some(other),
			receipt: valid_burn.receipt.clone(),
			receipt_signature: valid_burn.receipt_signature.clone(),
			..SimnetPost::new(SimnetInstruction::Burn, valid_burn.accounts.clone())
		}))
		.unwrap_err();

		assert!(
			matches!(refused, KeeperError::Chain(ChainFailure::Rejected(_))),
			"the receipt names its husk, so another husk's receipt must be refused: {refused}"
		);
	}

	#[test]
	fn a_ledger_that_refuses_the_burn_leaves_the_books_as_they_were() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		block_on(keeper.release(SAINT)).unwrap();
		world.later(60);
		let before = block_on(keeper.load()).unwrap();

		world.close_ledger();
		let refused = play(&keeper).unwrap_err();

		assert!(matches!(refused, KeeperError::Unavailable(_)), "{refused}");
		assert_eq!(
			block_on(keeper.load()).unwrap(),
			before,
			"the books must not keep a settlement the ledger refused"
		);
		assert_eq!(
			chain.transactions().len(),
			3,
			"the ledger keeps no transaction it could not write"
		);
	}

	#[test]
	fn a_ledger_that_refuses_the_escrow_leaves_the_books_as_they_were() {
		let world = World::new();
		let chain = world.chain();
		let keeper = world.keeper(chain.clone());

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(SAINT)).unwrap();
		let before = block_on(keeper.load()).unwrap();

		world.close_ledger();
		let refused = block_on(keeper.release(SAINT)).unwrap_err();

		assert!(matches!(refused, KeeperError::Unavailable(_)), "{refused}");
		assert_eq!(
			block_on(keeper.load()).unwrap(),
			before,
			"the husk stays in the vault while the ledger has no escrow for it"
		);
		assert_eq!(chain.transactions().len(), 2);
	}
}
