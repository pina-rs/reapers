//! The books kept on the device itself, with no server.
//!
//! The rules are the shared ones, so nothing plays differently offline. The
//! difference is trust: this keeper draws its own seeds and reads its own
//! clock, which is fine for one player's own game and is exactly what the
//! server exists to take away.

use std::cell::Cell;
use std::rc::Rc;

use async_trait::async_trait;
use reapers_rules::Absorption;
use reapers_rules::Art;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntSettlement;
use reapers_rules::Judging;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::RiftRoom;
use reapers_rules::RiftSite;
use reapers_rules::RuleResult;
use reapers_rules::Timestamp;
use reapers_rules::Weapon;

use super::GameBackend;
use super::KeeperError;
use super::KeeperKind;
use super::KeeperResult;
use super::LocalRiftSession;
use super::RiftSession;
use crate::platform;
use crate::platform::Prefs;

/// Where a keeper's seeds come from.
#[async_trait(?Send)]
pub trait Dice {
	/// A fresh 32-bit seed.
	async fn seed(&self) -> KeeperResult<i64>;
}

/// The device's secure random source.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceDice;

#[async_trait(?Send)]
impl Dice for DeviceDice {
	async fn seed(&self) -> KeeperResult<i64> {
		Ok(platform::random_seed().await?)
	}
}

/// Seeds counted up from a start, for tests that must know what is drawn.
#[derive(Debug, Default)]
pub struct SeededDice(Cell<i64>);

impl SeededDice {
	pub const fn new(first: i64) -> Self {
		Self(Cell::new(first))
	}
}

#[async_trait(?Send)]
impl Dice for SeededDice {
	async fn seed(&self) -> KeeperResult<i64> {
		let seed = self.0.get();
		self.0.set(seed.wrapping_add(1));
		Ok(seed)
	}
}

/// Signs an absorption's receipt, for a keeper that vouches for what it
/// settles. The plain offline game signs nothing.
pub type ReceiptSigner = Rc<dyn Fn(&Absorption) -> String>;

/// The device's own keeper.
#[derive(Clone)]
pub struct LocalBackend {
	prefs: Rc<dyn Prefs>,
	storage_key: &'static str,
	clock: Rc<dyn Fn() -> Timestamp>,
	dice: Rc<dyn Dice>,
	sign_receipt: Option<ReceiptSigner>,
}

impl LocalBackend {
	/// Where the player is stored. The name carries the schema's
	/// generation, so a future incompatible format starts fresh instead of
	/// failing to read. It is the Flutter game's key.
	pub const DEFAULT_STORAGE_KEY: &'static str = "reapers.player.v2";

	/// A keeper for the player saved in `prefs`, with the device's clock
	/// and dice.
	pub fn new(prefs: Rc<dyn Prefs>) -> Self {
		Self {
			prefs,
			storage_key: Self::DEFAULT_STORAGE_KEY,
			clock: Rc::new(platform::now),
			dice: Rc::new(DeviceDice),
			sign_receipt: None,
		}
	}

	/// The same keeper with its own clock.
	#[must_use]
	pub fn with_clock(mut self, clock: impl Fn() -> Timestamp + 'static) -> Self {
		self.clock = Rc::new(clock);
		self
	}

	/// The same keeper with its own dice.
	#[must_use]
	pub fn with_dice(mut self, dice: impl Dice + 'static) -> Self {
		self.dice = Rc::new(dice);
		self
	}

	/// The same keeper, storing its player under `key`. The simnet keeps a
	/// player of its own, so a rehearsal never touches the offline save.
	#[must_use]
	pub const fn with_storage_key(mut self, key: &'static str) -> Self {
		self.storage_key = key;
		self
	}

	/// The same keeper, signing every absorption it settles.
	#[must_use]
	pub fn with_receipts(mut self, sign: ReceiptSigner) -> Self {
		self.sign_receipt = Some(sign);
		self
	}

	pub const fn storage_key(&self) -> &'static str {
		self.storage_key
	}

	/// The time by this keeper's clock.
	pub fn now(&self) -> Timestamp {
		(self.clock)()
	}

	/// Forgets the player, as if the game had just been installed.
	pub async fn forget_player(&self) -> KeeperResult<()> {
		Ok(self.prefs.remove(self.storage_key).await?)
	}

	async fn save(&self, state: &PlayerState) -> KeeperResult<()> {
		let value = serde_json::to_value(state).map_err(KeeperError::unavailable)?;
		Ok(self.prefs.set(self.storage_key, value).await?)
	}

	async fn change(
		&self,
		apply: impl FnOnce(&PlayerState) -> RuleResult<PlayerState>,
	) -> KeeperResult<PlayerState> {
		let next = apply(&self.load().await?)?;
		self.save(&next).await?;
		Ok(next)
	}

	/// Pays the reaper `reaper_id` for a sealed rift.
	pub async fn settle_rift(
		&self,
		xp: i64,
		anima: i64,
		reaper_id: Option<String>,
	) -> KeeperResult<PlayerState> {
		self.change(|state| state.settle_rift(xp, anima, reaper_id.as_deref()))
			.await
	}
}

#[async_trait(?Send)]
impl GameBackend for LocalBackend {
	fn kind(&self) -> KeeperKind {
		KeeperKind::Local
	}

	async fn load(&self) -> KeeperResult<PlayerState> {
		match self.prefs.get(self.storage_key).await? {
			None => Ok(PlayerState::initial()),
			// The Flutter game stored the JSON as text; read either.
			Some(serde_json::Value::String(text)) => {
				serde_json::from_str(&text).map_err(|error| KeeperError::Damaged(error.to_string()))
			}
			Some(value) => {
				serde_json::from_value(value)
					.map_err(|error| KeeperError::Damaged(error.to_string()))
			}
		}
	}

	async fn locate(&self, _fix: Option<GeoFix>) -> KeeperResult<()> {
		Ok(())
	}

	async fn take_oath(&self) -> KeeperResult<PlayerState> {
		let seed = self.dice.seed().await?;
		self.change(|state| state.take_oath(seed)).await
	}

	async fn select_reaper(&self, id: &str) -> KeeperResult<PlayerState> {
		self.change(|state| state.select_reaper(id)).await
	}

	async fn acquire(&self, nft: Nft) -> KeeperResult<PlayerState> {
		self.change(|state| state.acquire(nft)).await
	}

	async fn release(&self, nft: Nft) -> KeeperResult<PlayerState> {
		let seed = self.dice.seed().await?;
		let now = self.now();
		self.change(|state| state.release(nft, seed, now)).await
	}

	async fn settle_hunt(&self, sighting_id: i64, log: &HuntLog) -> KeeperResult<HuntSettlement> {
		let feeding_seed = self.dice.seed().await?;
		let mut settlement = self.load().await?.judge_hunt(
			sighting_id,
			log,
			self.now(),
			feeding_seed,
			&Judging::default(),
		)?;

		if let Some(sign) = &self.sign_receipt
			&& settlement.outcome.absorbed()
			&& let Some(absorption) = settlement
				.state
				.active_reaper()
				.and_then(|reaper| reaper.codex().last())
		{
			let signature = sign(absorption);
			settlement.state = settlement.state.sign_absorption(absorption.nft, &signature);
		}

		self.save(&settlement.state).await?;
		Ok(settlement)
	}

	async fn buy_weapon(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.change(|state| state.buy_weapon(weapon)).await
	}

	async fn wield(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.change(|state| state.wield(weapon)).await
	}

	async fn buy_outfit(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.change(|state| state.buy_outfit(outfit)).await
	}

	async fn wear(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.change(|state| state.wear(outfit)).await
	}

	async fn attune(&self, art: Art) -> KeeperResult<PlayerState> {
		self.change(|state| state.attune(art)).await
	}

	async fn enter_rift(&self, site: RiftSite) -> KeeperResult<Box<dyn RiftSession>> {
		let player = self.load().await?;
		let entrant = player.entrant_for(site)?;
		let mut room = RiftRoom::new(site, self.dice.seed().await?);
		let seat = room
			.join(entrant)
			.map_err(|_| KeeperError::unavailable("the rift has no empty seat"))?;
		let books = self.clone();
		let reaper = player.active_reaper_id().map(str::to_owned);

		Ok(Box::new(LocalRiftSession::open(
			room,
			seat,
			self.now(),
			self.clock.clone(),
			Rc::new(move |reward| {
				let books = books.clone();
				let reaper = reaper.clone();
				Box::pin(async move {
					books
						.settle_rift(reward.xp, reward.anima, reaper)
						.await
						.map(|_| ())
				})
			}),
		)))
	}
}

#[cfg(test)]
mod tests {
	use futures::executor::block_on;
	use reapers_rules::Refusal;
	use reapers_rules::market_listings;

	use super::*;
	use crate::platform::MemoryPrefs;

	fn keeper() -> (Rc<MemoryPrefs>, LocalBackend) {
		let prefs = Rc::new(MemoryPrefs::new());
		let keeper = LocalBackend::new(prefs.clone())
			.with_clock(|| Timestamp::utc(2026, 1, 1, 12, 0, 0))
			.with_dice(SeededDice::new(7));
		(prefs, keeper)
	}

	#[test]
	fn a_new_player_has_taken_no_oath() {
		let (_, keeper) = keeper();
		let player = block_on(keeper.load()).unwrap();
		assert!(!player.oath_taken());
		assert_eq!(player.lamports(), PlayerState::STARTING_LAMPORTS);
	}

	#[test]
	fn the_oath_is_kept_between_launches() {
		let (prefs, keeper) = keeper();
		let bound = block_on(keeper.take_oath()).unwrap();
		assert!(bound.oath_taken());

		let reopened = LocalBackend::new(prefs);
		assert_eq!(block_on(reopened.load()).unwrap(), bound);
	}

	#[test]
	fn the_oath_is_taken_once() {
		let (_, keeper) = keeper();
		block_on(keeper.take_oath()).unwrap();
		let again = block_on(keeper.take_oath()).unwrap_err();
		assert_eq!(again.refusal(), Some(Refusal::OathAlreadyTaken));
	}

	#[test]
	fn a_refusal_leaves_the_save_as_it_was() {
		let (_, keeper) = keeper();
		let before = block_on(keeper.load()).unwrap();
		let refused = block_on(keeper.acquire(market_listings()[0])).unwrap_err();
		assert_eq!(refused.refusal(), Some(Refusal::OathRequired));
		assert_eq!(block_on(keeper.load()).unwrap(), before);
	}

	#[test]
	fn buying_and_releasing_makes_a_husk() {
		let (_, keeper) = keeper();
		block_on(keeper.take_oath()).unwrap();
		let nft = market_listings()[0];
		let bought = block_on(keeper.acquire(nft)).unwrap();
		assert_eq!(bought.vault(), &[nft]);
		assert_eq!(
			bought.lamports(),
			PlayerState::STARTING_LAMPORTS - nft.rarity().lamports()
		);

		let released = block_on(keeper.release(nft)).unwrap();
		assert_eq!(released.vault(), []);
		assert_eq!(released.sightings().len(), 1);
		assert_eq!(released.sightings()[0].nft, nft);
	}

	#[test]
	fn a_save_written_as_text_still_reads() {
		let (prefs, keeper) = keeper();
		let bound = block_on(keeper.take_oath()).unwrap();
		let text = serde_json::to_string(&bound).unwrap();
		block_on(prefs.set(
			LocalBackend::DEFAULT_STORAGE_KEY,
			serde_json::Value::String(text),
		))
		.unwrap();
		assert_eq!(block_on(keeper.load()).unwrap(), bound);
	}

	#[test]
	fn a_damaged_save_is_told_apart_from_an_unreachable_keeper() {
		let (prefs, keeper) = keeper();
		block_on(prefs.set(
			LocalBackend::DEFAULT_STORAGE_KEY,
			serde_json::json!({ "schema": "nonsense" }),
		))
		.unwrap();
		assert!(matches!(
			block_on(keeper.load()),
			Err(KeeperError::Damaged(_))
		));
	}

	#[test]
	fn forgetting_starts_over() {
		let (_, keeper) = keeper();
		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.forget_player()).unwrap();
		assert!(!block_on(keeper.load()).unwrap().oath_taken());
	}

	#[test]
	fn a_rift_is_shut_to_a_reaper_who_carries_none_of_its_dead() {
		let (_, keeper) = keeper();
		block_on(keeper.take_oath()).unwrap();
		let refused = block_on(keeper.enter_rift(RiftSite::ALL[0])).err().unwrap();
		assert_eq!(refused.refusal(), Some(Refusal::NotAttuned));
	}

	#[test]
	fn an_absorbed_husk_pays_is_written_in_the_codex_and_attunes_its_rift() {
		let prefs = Rc::new(MemoryPrefs::new());
		let now = Rc::new(Cell::new(Timestamp::utc(2026, 10, 1, 12, 0, 0)));
		let clock = now.clone();
		let keeper = LocalBackend::new(prefs)
			.with_clock(move || clock.get())
			.with_dice(SeededDice::new(5));
		let saint = Nft::new(reapers_rules::NftCollection::DegenApes, 1);

		block_on(keeper.take_oath()).unwrap();
		block_on(keeper.acquire(saint)).unwrap();
		let released = block_on(keeper.release(saint)).unwrap();
		let sighting = released.sightings()[0];
		let unproven =
			block_on(keeper.settle_hunt(sighting.id, &HuntLog::new(Vec::new()))).unwrap_err();
		assert_eq!(unproven.refusal(), Some(Refusal::NotSurfaced));

		now.set(now.get().plus_seconds(60));
		let mut sim = reapers_rules::HuntSim::open(
			sighting.seed,
			saint.collection.vessel(),
			saint.rarity().ward(),
			&reapers_rules::HuntStats::derive(&released.loadout(), 1.0).unwrap(),
		);
		let mut bot = reapers_rules::HuntBot::careful(&sim);
		bot.play(&mut sim);
		let settlement =
			block_on(keeper.settle_hunt(sighting.id, &HuntLog::new(bot.recorded))).unwrap();

		assert!(settlement.outcome.absorbed());
		assert!(settlement.xp > 0);
		let codex = settlement.state.active_reaper().unwrap().codex();
		assert_eq!(codex.last().map(|entry| entry.nft), Some(saint));
		assert!(
			codex.last().unwrap().signature.is_none(),
			"an offline keeper signs nothing"
		);
		assert_eq!(block_on(keeper.load()).unwrap(), settlement.state);

		let site = RiftSite::ALL
			.into_iter()
			.find(|site| site.collection() == saint.collection)
			.unwrap();
		assert!(settlement.state.attuned_to(site));
		assert!(block_on(keeper.enter_rift(site)).is_ok());
	}
}
