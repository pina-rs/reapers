//! Everything one player owns and has done, and every request they can
//! make of the rules.

use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use thiserror::Error;

use crate::armory::Outfit;
use crate::armory::Weapon;
use crate::arts::Art;
use crate::error::FormatError;
use crate::feeding::Feeding;
use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::ReapGrade;
use crate::hunt::hunt_input::HuntLog;
use crate::hunt::hunt_sim::HuntSim;
use crate::hunt::hunt_stats::HuntLoadout;
use crate::hunt::hunt_stats::HuntStats;
use crate::named::named_enum;
use crate::nft_catalogue::Nft;
use crate::nft_catalogue::NftCollection;
use crate::reaper_genome::ReaperGenome;
use crate::reaper_soul::SoulMemory;
use crate::reaper_state::ReaperRecord;
use crate::reaper_state::ReaperState;
use crate::rift::rift_room::RiftEntrant;
use crate::rift::rift_rules::RiftRules;
use crate::rift::rift_site::RiftSite;
use crate::seeded_random::SeededRandom;
use crate::time::Timestamp;

/// Why a request was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Refusal {
	/// The reaper is already bound; the oath is taken once.
	OathAlreadyTaken,

	/// Only an oathbound reaper may do this.
	OathRequired,

	/// The market does not list this NFT.
	NotListed,

	/// The player already holds, has released or has absorbed this NFT.
	AlreadyHeld,

	/// The NFT is not in the player's vault.
	NotInVault,

	/// The wallet does not hold enough SOL.
	InsufficientFunds,

	/// The reaper does not hold enough anima.
	InsufficientAnima,

	/// The reaper's level is too low.
	LevelTooLow,

	/// The player has not bought this item.
	NotOwned,

	/// The player already owns this item.
	AlreadyOwned,

	/// Too many husks are already loose.
	VeilCrowded,

	/// No such husk is loose.
	UnknownSighting,

	/// The husk has not surfaced yet.
	NotSurfaced,

	/// The submitted hunt does not replay to a finished hunt.
	HuntUnproven,

	/// Client and keeper run different combat rules; update before
	/// retrying.
	HuntVersionUnsupported,

	/// The simulated NFT economy is unavailable on a deployed keeper.
	MarketUnavailable,

	/// Tradable reapers require replayed rift results before earning
	/// growth.
	RiftUnproven,

	/// The reaper has absorbed nothing from this rift's collection yet.
	NotAttuned,

	/// A location is malformed, too coarse, stale or future-dated.
	LocationUnusable,

	/// Finish loose hunts before taking another reaper.
	ReaperBusy,

	/// The wallet proof is invalid, expired or already used.
	WalletUnproven,

	/// NFT ownership requires a configured collection and online keeper.
	ChainUnconfigured,

	/// The chain could not be read or returned an invalid response.
	ChainUnavailable,

	/// The NFT roster has reached its bounded ownership-check capacity.
	RosterFull,
}

named_enum!(Refusal {
	OathAlreadyTaken => "oathAlreadyTaken",
	OathRequired => "oathRequired",
	NotListed => "notListed",
	AlreadyHeld => "alreadyHeld",
	NotInVault => "notInVault",
	InsufficientFunds => "insufficientFunds",
	InsufficientAnima => "insufficientAnima",
	LevelTooLow => "levelTooLow",
	NotOwned => "notOwned",
	AlreadyOwned => "alreadyOwned",
	VeilCrowded => "veilCrowded",
	UnknownSighting => "unknownSighting",
	NotSurfaced => "notSurfaced",
	HuntUnproven => "huntUnproven",
	HuntVersionUnsupported => "huntVersionUnsupported",
	MarketUnavailable => "marketUnavailable",
	RiftUnproven => "riftUnproven",
	NotAttuned => "notAttuned",
	LocationUnusable => "locationUnusable",
	ReaperBusy => "reaperBusy",
	WalletUnproven => "walletUnproven",
	ChainUnconfigured => "chainUnconfigured",
	ChainUnavailable => "chainUnavailable",
	RosterFull => "rosterFull",
});

/// A request the rules refused.
///
/// The game and the server raise the same breach from the same code, so an
/// offline player and an online one are told the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Error)]
#[error("RuleBreach({})", .refusal.name())]
pub struct RuleBreach {
	pub refusal: Refusal,
}

impl From<Refusal> for RuleBreach {
	fn from(refusal: Refusal) -> Self {
		Self { refusal }
	}
}

/// What every request answers: the state after it, or why it was refused.
pub type RuleResult<T> = Result<T, RuleBreach>;

fn refuse<T>(refusal: Refusal) -> RuleResult<T> {
	Err(RuleBreach { refusal })
}

/// A husk loose in the veil, waiting to be hunted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sighting {
	/// Unique among one player's sightings.
	pub id: i64,

	pub nft: Nft,

	/// The seed of every hunt for this husk.
	pub seed: i64,

	/// When the husk shows itself. Before then it cannot be hunted.
	pub surfaces_at: Timestamp,
}

impl Sighting {
	pub const fn has_surfaced(&self, now: Timestamp) -> bool {
		!now.is_before(self.surfaces_at)
	}
}

/// An NFT the reaper has absorbed, kept in the codex.
///
/// It records everything the keeper decided at that moment, so the entry is
/// also what a signed receipt is made from: see docs/nfts.md.
///
/// The receipt's byte layout lives in the `reapers_receipt` crate, which the
/// server and the on-chain program share. Every field it signs is here:
/// `grade` and `feeding` as their indices, `xp` and `anima`, `at` in whole
/// seconds ([`Timestamp::unix_seconds`]) and `nonce`. The player's and the
/// mint's 32-byte addresses come from the keeper, which knows them; the
/// integration builds the receipt from this entry and those two.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Absorption {
	pub nft: Nft,

	pub grade: ReapGrade,

	/// How well the reaper fed, as the keeper rolled it.
	pub feeding: Feeding,

	/// Experience the absorption paid.
	pub xp: i64,

	/// Anima the absorption paid.
	pub anima: i64,

	/// The seed the feeding was rolled from. With the NFT it makes this
	/// absorption unlike any other, which is what lets a receipt be used
	/// once.
	pub nonce: i64,

	pub at: Timestamp,

	/// The keeper's signature over this absorption, in base64, or `None`
	/// when the keeper was the phone itself: an offline game signs nothing.
	#[serde(default)]
	pub signature: Option<String>,
}

impl Absorption {
	/// This absorption, signed.
	#[must_use]
	pub fn signed_with(&self, signature: impl Into<String>) -> Self {
		Self {
			signature: Some(signature.into()),
			..self.clone()
		}
	}
}

/// What a judged hunt came to: the verdict, what it paid, and the player
/// afterwards.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HuntSettlement {
	pub state: PlayerState,

	pub outcome: HuntOutcome,

	/// Experience the hunt paid.
	pub xp: i64,

	/// Anima the hunt paid.
	pub anima: i64,

	/// How well the reaper fed, or `None` when the husk got away.
	#[serde(default)]
	pub feeding: Option<Feeding>,
}

/// How a keeper judges a submitted hunt.
#[derive(Clone, Debug, PartialEq)]
pub struct Judging {
	/// How much wider every window is than the phone played with. The
	/// server passes a little above 1, so a hunt won by a hair on the phone
	/// is not judged lost over a rounding difference.
	pub leniency: f64,

	/// The difficulties a hunt may have been played at. A keeper that pays
	/// for real allows only the game's own.
	pub difficulties: Vec<Difficulty>,
}

impl Default for Judging {
	fn default() -> Self {
		Self {
			leniency: 1.0,
			difficulties: Difficulty::ALL.to_vec(),
		}
	}
}

/// Everything one player owns and has done.
///
/// The state is immutable. Each request is a method that returns the state
/// after it, or a [`RuleBreach`]; nothing here reads a clock or rolls a die
/// on its own. The offline game and the server both keep a player as one of
/// these and change it only through these methods, which is what keeps the
/// two from drifting apart.
///
/// Saves are versioned. This writes schema three; schema two's one reaper
/// migrates intact into the oathbound roster seat.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "PlayerRecord")]
pub struct PlayerState {
	reapers: Vec<ReaperState>,
	active_reaper_id: Option<String>,
	anima: i64,
	lamports: i64,
	vault: Vec<Nft>,
	sightings: Vec<Sighting>,
	next_sighting_id: i64,
}

impl Default for PlayerState {
	fn default() -> Self {
		Self::initial()
	}
}

/// A player as any schema of save writes it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerRecord {
	schema: i64,
	anima: i64,
	lamports: i64,
	vault: Vec<Nft>,
	sightings: Vec<Sighting>,
	next_sighting_id: i64,

	// Schema three.
	#[serde(default)]
	reapers: Option<Vec<ReaperState>>,
	#[serde(default)]
	active_reaper_id: Option<String>,

	// Schema two: the one reaper, at the top level.
	#[serde(default)]
	genome: Option<String>,
	#[serde(default)]
	xp: Option<i64>,
	#[serde(default)]
	weapons: Option<Vec<Weapon>>,
	#[serde(default)]
	outfits: Option<Vec<Outfit>>,
	#[serde(default)]
	weapon: Option<Weapon>,
	#[serde(default)]
	outfit: Option<Outfit>,
	#[serde(default)]
	codex: Option<Vec<Absorption>>,
	#[serde(default)]
	hunts_lost: Option<i64>,
	#[serde(default)]
	rifts_sealed: Option<i64>,
	#[serde(default)]
	memory: Option<SoulMemory>,
	#[serde(default)]
	art: Option<Art>,
}

impl PlayerRecord {
	/// Schema two's one reaper, as a schema-three roster seat.
	fn oathbound(&mut self) -> Result<Vec<ReaperState>, FormatError> {
		let Some(genome) = self.genome.take() else {
			return Ok(Vec::new());
		};
		let missing = || FormatError::new("Incomplete schema two reaper");
		let record = ReaperRecord {
			id: ReaperState::OATH_ID.to_owned(),
			genome,
			xp: self.xp.ok_or_else(missing)?,
			weapons: self.weapons.take().ok_or_else(missing)?,
			outfits: self.outfits.take().ok_or_else(missing)?,
			weapon: self.weapon.ok_or_else(missing)?,
			outfit: self.outfit.ok_or_else(missing)?,
			codex: self.codex.take().ok_or_else(missing)?,
			hunts_lost: self.hunts_lost.ok_or_else(missing)?,
			rifts_sealed: self.rifts_sealed.ok_or_else(missing)?,
			memory: self.memory,
			art: self.art,
		};

		Ok(vec![ReaperState::try_from(record)?])
	}
}

impl TryFrom<PlayerRecord> for PlayerState {
	type Error = FormatError;

	fn try_from(mut record: PlayerRecord) -> Result<Self, FormatError> {
		let (roster, selected) = match record.schema {
			2 => {
				let roster = record.oathbound()?;
				let selected = roster.first().map(|reaper| reaper.id().to_owned());

				(roster, selected)
			}
			PlayerState::SCHEMA => {
				(
					record
						.reapers
						.take()
						.ok_or_else(|| FormatError::new("A save with no reapers"))?,
					record.active_reaper_id.take(),
				)
			}
			schema => {
				return Err(FormatError::with_input(
					"Unknown player schema",
					schema.to_string(),
				));
			}
		};

		let unique = roster.iter().enumerate().all(|(index, reaper)| {
			roster[..index]
				.iter()
				.all(|other| other.id() != reaper.id())
		});
		let selection_valid = match &selected {
			None => roster.is_empty(),
			Some(selected) => roster.iter().any(|reaper| reaper.id() == selected),
		};

		if !unique || !selection_valid {
			return Err(FormatError::new(
				"Invalid active reaper or duplicate roster seat",
			));
		}

		Ok(Self {
			reapers: roster,
			active_reaper_id: selected,
			anima: record.anima,
			lamports: record.lamports,
			vault: record.vault,
			sightings: record.sightings,
			next_sighting_id: record.next_sighting_id,
		})
	}
}

/// A player as this version writes it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerSave<'a> {
	schema: i64,
	reapers: &'a [ReaperState],
	active_reaper_id: &'a Option<String>,
	anima: i64,
	lamports: i64,
	vault: &'a [Nft],
	sightings: &'a [Sighting],
	next_sighting_id: i64,
}

impl Serialize for PlayerState {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		PlayerSave {
			schema: Self::SCHEMA,
			reapers: &self.reapers,
			active_reaper_id: &self.active_reaper_id,
			anima: self.anima,
			lamports: self.lamports,
			vault: &self.vault,
			sightings: &self.sightings,
			next_sighting_id: self.next_sighting_id,
		}
		.serialize(serializer)
	}
}

impl PlayerState {
	/// Keeps one ownership read bounded; the oathbound starter is
	/// additional.
	pub const MAX_NFT_REAPERS: usize = 32;
	/// Husks that may be loose at once.
	pub const MAX_SIGHTINGS: usize = 3;
	/// The save schema this version writes.
	pub const SCHEMA: i64 = 3;
	/// The stand-in wallet a new player starts with: three SOL, which is
	/// enough to buy every NFT the market lists.
	pub const STARTING_LAMPORTS: i64 = 3_000_000_000;
	pub const SURFACE_MAX_SECONDS: i64 = 14;
	/// The shortest and longest wait, in seconds, before a released husk
	/// surfaces.
	pub const SURFACE_MIN_SECONDS: i64 = 6;

	/// A new player: no reaper, no anima, three stand-in SOL.
	pub const fn initial() -> Self {
		Self {
			reapers: Vec::new(),
			active_reaper_id: None,
			anima: 0,
			lamports: Self::STARTING_LAMPORTS,
			vault: Vec::new(),
			sightings: Vec::new(),
			next_sighting_id: 1,
		}
	}

	/// Each reaper owns its growth, codex and equipment. Anima and the vault
	/// belong to the player; switching never copies experience between
	/// reapers.
	pub fn reapers(&self) -> &[ReaperState] {
		&self.reapers
	}

	pub fn active_reaper_id(&self) -> Option<&str> {
		self.active_reaper_id.as_deref()
	}

	pub const fn anima(&self) -> i64 {
		self.anima
	}

	/// Offline simulation balance; a linked Solana balance is read
	/// separately.
	pub const fn lamports(&self) -> i64 {
		self.lamports
	}

	pub fn vault(&self) -> &[Nft] {
		&self.vault
	}

	pub fn sightings(&self) -> &[Sighting] {
		&self.sightings
	}

	pub const fn next_sighting_id(&self) -> i64 {
		self.next_sighting_id
	}

	pub fn active_reaper(&self) -> Option<&ReaperState> {
		let id = self.active_reaper_id.as_deref()?;

		self.reapers.iter().find(|reaper| reaper.id() == id)
	}

	pub fn genome_code(&self) -> Option<String> {
		self.active_reaper().map(ReaperState::genome_code)
	}

	pub fn xp(&self) -> i64 {
		self.active_reaper().map_or(0, ReaperState::xp)
	}

	pub fn weapons(&self) -> &[Weapon] {
		self.active_reaper().map_or(&[], ReaperState::weapons)
	}

	pub fn outfits(&self) -> &[Outfit] {
		self.active_reaper()
			.map_or(&[Outfit::Shrouded], ReaperState::outfits)
	}

	pub fn weapon(&self) -> Option<Weapon> {
		self.active_reaper().map(ReaperState::weapon)
	}

	pub fn outfit(&self) -> Outfit {
		self.active_reaper()
			.map_or(Outfit::Shrouded, ReaperState::outfit)
	}

	pub fn codex(&self) -> &[Absorption] {
		self.active_reaper().map_or(&[], ReaperState::codex)
	}

	pub fn hunts_lost(&self) -> i64 {
		self.active_reaper().map_or(0, ReaperState::hunts_lost)
	}

	pub fn rifts_sealed(&self) -> i64 {
		self.active_reaper().map_or(0, ReaperState::rifts_sealed)
	}

	pub fn oath_taken(&self) -> bool {
		self.active_reaper().is_some()
	}

	pub fn genome(&self) -> ReaperGenome {
		self.active_reaper()
			.map_or(ReaperGenome::BORROWED, |reaper| *reaper.genome())
	}

	pub fn level(&self) -> i64 {
		self.active_reaper().map_or(1, ReaperState::level)
	}

	/// Registers a reaper whose NFT ownership has already been verified by
	/// the caller. There is deliberately no simulated SOL purchase here.
	pub fn bind_reaper(&self, reaper: ReaperState) -> RuleResult<Self> {
		if self.reapers.iter().any(|owned| owned.id() == reaper.id()) {
			return refuse(Refusal::AlreadyOwned);
		}

		if reaper.is_nft()
			&& self.reapers.iter().filter(|owned| owned.is_nft()).count() >= Self::MAX_NFT_REAPERS
		{
			return refuse(Refusal::RosterFull);
		}

		let mut roster = self.reapers.clone();

		roster.push(reaper);

		Ok(self.with_roster(roster))
	}

	/// Loose husks belong to the current reaper until their hunts are
	/// finished.
	pub fn select_reaper(&self, id: &str) -> RuleResult<Self> {
		if !self.reapers.iter().any(|reaper| reaper.id() == id) {
			return refuse(Refusal::NotOwned);
		}

		if Some(id) != self.active_reaper_id() && !self.sightings.is_empty() {
			return refuse(Refusal::ReaperBusy);
		}

		Ok(Self {
			active_reaper_id: Some(id.to_owned()),
			..self.clone()
		})
	}

	/// Refreshes NFT-backed seats from the authoritative asset ledger while
	/// retaining the non-tradable oathbound starter. A transferred seat
	/// leaves this roster; its progression stays in the server's asset
	/// ledger.
	pub fn reconcile_reapers(&self, owned: Vec<ReaperState>) -> RuleResult<Self> {
		let mut roster: Vec<ReaperState> = self
			.reapers
			.iter()
			.filter(|reaper| !reaper.is_nft())
			.cloned()
			.collect();

		roster.extend(owned);

		let duplicated = roster.iter().enumerate().any(|(index, reaper)| {
			roster[..index]
				.iter()
				.any(|other| other.id() == reaper.id())
		});

		if duplicated {
			return refuse(Refusal::AlreadyOwned);
		}

		// A transferred reaper cannot keep hunting under another reaper's
		// stats. Return its unfinished husks to this player's vault without
		// rewarding it.
		let lost_active = !roster
			.iter()
			.any(|reaper| Some(reaper.id()) == self.active_reaper_id());
		let mut next = self.with_roster(roster);

		if lost_active {
			next.vault
				.extend(self.sightings.iter().map(|sighting| sighting.nft));
			next.sightings.clear();
		}

		Ok(next)
	}

	/// Collections with a full set absorbed.
	pub fn completed_sets(&self) -> i64 {
		NftCollection::ALL
			.into_iter()
			.filter(|&collection| self.absorbed_from(collection) >= NftCollection::SET_SIZE)
			.count() as i64
	}

	/// How many of `collection` the reaper has absorbed.
	pub fn absorbed_from(&self, collection: NftCollection) -> usize {
		self.codex()
			.iter()
			.filter(|entry| entry.nft.collection == collection)
			.count()
	}

	/// What the reaper takes into a hunt.
	pub fn loadout(&self) -> HuntLoadout {
		let Some(reaper) = self.active_reaper() else {
			return HuntLoadout::BORROWED;
		};

		HuntLoadout {
			reaper_class: reaper.genome().reaper_class,
			level: reaper.level(),
			weapon: reaper.weapon(),
			completed_sets: self.completed_sets(),
			soul: Some(reaper.soul()),
			soul_rank: reaper.soul_rank(),
			art: reaper.art_skill(),
		}
	}

	/// Whether the player holds `nft` in any form.
	pub fn holds(&self, nft: Nft) -> bool {
		self.vault.contains(&nft)
			|| self.sightings.iter().any(|sighting| sighting.nft == nft)
			|| self
				.reapers
				.iter()
				.any(|reaper| reaper.codex().iter().any(|entry| entry.nft == nft))
	}

	/// Binds a reaper drawn from `seed`.
	///
	/// The caller supplies the seed, so the server decides it for an online
	/// player and nobody picks their own reaper.
	pub fn take_oath(&self, seed: i64) -> RuleResult<Self> {
		if self.oath_taken() {
			return refuse(Refusal::OathAlreadyTaken);
		}

		let genome = ReaperGenome::shuffle(seed);

		Ok(self.with_roster(vec![ReaperState::bound(ReaperState::OATH_ID, genome)]))
	}

	/// Buys `nft` from the stand-in market into the vault.
	pub fn acquire(&self, nft: Nft) -> RuleResult<Self> {
		self.require_oath()?;

		if self.holds(nft) {
			return refuse(Refusal::AlreadyHeld);
		}

		if self.lamports < nft.rarity().lamports() {
			return refuse(Refusal::InsufficientFunds);
		}

		let mut next = self.clone();

		next.lamports -= nft.rarity().lamports();
		next.vault.push(nft);

		Ok(next)
	}

	/// Releases `nft` from the vault into the veil, where it becomes a husk.
	///
	/// `seed` decides when the husk surfaces and how every hunt for it
	/// plays.
	pub fn release(&self, nft: Nft, seed: i64, now: Timestamp) -> RuleResult<Self> {
		self.require_oath()?;

		if !self.vault.contains(&nft) {
			return refuse(Refusal::NotInVault);
		}

		if self.sightings.len() >= Self::MAX_SIGHTINGS {
			return refuse(Refusal::VeilCrowded);
		}

		let wait = Self::SURFACE_MIN_SECONDS
			+ SeededRandom::new(seed)
				.next_int(Self::SURFACE_MAX_SECONDS - Self::SURFACE_MIN_SECONDS + 1);
		let mut next = self.clone();

		next.vault.retain(|&held| held != nft);
		next.sightings.push(Sighting {
			id: self.next_sighting_id,
			nft,
			seed,
			surfaces_at: now.plus_seconds(wait),
		});
		next.next_sighting_id = self.next_sighting_id.saturating_add(1);

		Ok(next)
	}

	/// The loose husk with `sighting_id`, ready to be hunted at `now`.
	pub fn huntable(&self, sighting_id: i64, now: Timestamp) -> RuleResult<Sighting> {
		let Some(sighting) = self
			.sightings
			.iter()
			.find(|candidate| candidate.id == sighting_id)
		else {
			return refuse(Refusal::UnknownSighting);
		};

		if !sighting.has_surfaced(now) {
			return refuse(Refusal::NotSurfaced);
		}

		Ok(*sighting)
	}

	/// Records how the hunt for `sighting_id` ended.
	///
	/// An absorbed husk leaves the veil for the codex and pays out. An
	/// escaped one stays loose, to be hunted again.
	///
	/// `feeding_seed` decides how well the reaper feeds on an absorbed husk.
	/// The caller supplies it, so the server rolls for an online player and
	/// nobody picks their own luck.
	pub fn settle_hunt(
		&self,
		sighting_id: i64,
		outcome: &HuntOutcome,
		now: Timestamp,
		feeding_seed: i64,
	) -> RuleResult<Self> {
		let sighting = self.huntable(sighting_id, now)?;

		let Some(grade) = outcome.grade() else {
			return Ok(self.with_active(|reaper| {
				let lost = reaper.hunts_lost().saturating_add(1);

				reaper.with_hunts_lost(lost)
			}));
		};

		let Some(reaper) = self.active_reaper() else {
			return refuse(Refusal::OathRequired);
		};

		let rarity = sighting.nft.rarity();
		let feeding = Feeding::roll(feeding_seed);
		let absorption = Absorption {
			nft: sighting.nft,
			grade,
			feeding,
			xp: outcome.xp_for(rarity, feeding),
			anima: outcome.anima_for(rarity),
			nonce: feeding_seed,
			at: now,
			signature: None,
		};

		// The first art a reaper learns is made ready at once: absorbing the
		// husk is how it learns, and throwing it is how it finds out.
		let learnt = Art::carried_by(sighting.nft.collection.vessel());
		let art = reaper.art().or(learnt);
		let memory = reaper.memory().remember(outcome);
		let xp = reaper.xp().saturating_add(absorption.xp);
		let anima = self.anima.saturating_add(absorption.anima);
		let mut codex = reaper.codex().to_vec();

		codex.push(absorption);

		let mut next = self.with_active(|reaper| {
			let reaper = reaper.with_xp(xp).with_codex(codex).with_memory(memory);

			match art {
				Some(art) => reaper.with_art(art),
				None => reaper,
			}
		});

		next.anima = anima;
		next.sightings.retain(|loose| loose.id != sighting_id);

		Ok(next)
	}

	/// Attaches the keeper's `signature` to the absorption of `nft`.
	#[must_use]
	pub fn sign_absorption(&self, nft: Nft, signature: &str) -> Self {
		self.with_active(|reaper| {
			let codex = reaper
				.codex()
				.iter()
				.map(|entry| {
					if entry.nft == nft {
						entry.signed_with(signature)
					} else {
						entry.clone()
					}
				})
				.collect();

			reaper.with_codex(codex)
		})
	}

	/// Judges the hunt recorded in `log` and settles it.
	///
	/// The log is replayed through the rules, so the verdict is whatever the
	/// rules say those inputs earn; nothing the player claims is believed. A
	/// log that stops before the hunt ends is refused as
	/// [`Refusal::HuntUnproven`], and so is one played at a difficulty
	/// `judging` does not allow.
	pub fn judge_hunt(
		&self,
		sighting_id: i64,
		log: &HuntLog,
		now: Timestamp,
		feeding_seed: i64,
		judging: &Judging,
	) -> RuleResult<HuntSettlement> {
		let sighting = self.huntable(sighting_id, now)?;
		let provable = log.inputs.len() <= HuntLog::MAX_TICKS
			&& judging.difficulties.contains(&log.difficulty);

		if !provable {
			return refuse(Refusal::HuntUnproven);
		}

		let Ok(stats) = HuntStats::derive(&self.loadout(), judging.leniency) else {
			return refuse(Refusal::HuntUnproven);
		};
		let Some(outcome) = HuntSim::replay(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity().ward(),
			&stats,
			log,
		) else {
			return refuse(Refusal::HuntUnproven);
		};

		let settled = self.settle_hunt(sighting_id, &outcome, now, feeding_seed)?;
		let absorption = if outcome.absorbed() {
			settled.codex().last().cloned()
		} else {
			None
		};

		Ok(HuntSettlement {
			state: settled,
			outcome,
			xp: absorption.as_ref().map_or(0, |absorption| absorption.xp),
			anima: absorption.as_ref().map_or(0, |absorption| absorption.anima),
			feeding: absorption.map(|absorption| absorption.feeding),
		})
	}

	/// Whether the rift at `site` opens for this reaper.
	///
	/// A rift answers only a reaper who already carries one of its dead, so
	/// the way to a collection's rift runs through that collection's NFTs.
	pub fn attuned_to(&self, site: RiftSite) -> bool {
		self.absorbed_from(site.collection()) >= RiftRules::ATTUNEMENT
	}

	/// This reaper as they arrive at the rift at `site`.
	pub fn entrant_for(&self, site: RiftSite) -> RuleResult<RiftEntrant> {
		let Some(reaper) = self.active_reaper() else {
			return refuse(Refusal::OathRequired);
		};

		if reaper.is_nft() {
			return refuse(Refusal::RiftUnproven);
		}

		if !self.attuned_to(site) {
			return refuse(Refusal::NotAttuned);
		}

		Ok(RiftEntrant {
			name: self.genome().name(),
			loadout: self.loadout(),
		})
	}

	/// Pays a reaper for helping to seal a rift: the reaper `reaper_id`, or
	/// the active one when that is `None`.
	pub fn settle_rift(&self, xp: i64, anima: i64, reaper_id: Option<&str>) -> RuleResult<Self> {
		self.require_oath()?;

		let id = reaper_id.or(self.active_reaper_id());

		if !self.reapers.iter().any(|reaper| Some(reaper.id()) == id) {
			return refuse(Refusal::NotOwned);
		}

		let mut next = self.clone();

		for reaper in &mut next.reapers {
			if Some(reaper.id()) == id {
				let paid = reaper.xp().saturating_add(xp);
				let sealed = reaper.rifts_sealed().saturating_add(1);

				*reaper = reaper.clone().with_xp(paid).with_rifts_sealed(sealed);
			}
		}

		next.anima = self.anima.saturating_add(anima);

		Ok(next)
	}

	/// Buys `weapon` with anima.
	pub fn buy_weapon(&self, weapon: Weapon) -> RuleResult<Self> {
		self.require_oath()?;

		if self.weapons().contains(&weapon) {
			return refuse(Refusal::AlreadyOwned);
		}

		let price = weapon.price_for(self.genome().reaper_class);

		self.require_level(weapon.level_required())?;
		self.require_anima(price)?;

		let mut next = self.with_active(|reaper| reaper.with_weapon(weapon));

		next.anima -= price;

		Ok(next)
	}

	/// Takes a bought `weapon` in hand.
	pub fn wield(&self, weapon: Weapon) -> RuleResult<Self> {
		if !self.weapons().contains(&weapon) {
			return refuse(Refusal::NotOwned);
		}

		Ok(self.with_active(|reaper| reaper.wielding(weapon)))
	}

	/// Makes `art` the one the reaper throws. Only an art it has learned, by
	/// absorbing a husk that carries it, can be made ready.
	pub fn attune(&self, art: Art) -> RuleResult<Self> {
		let Some(reaper) = self.active_reaper() else {
			return refuse(Refusal::OathRequired);
		};

		if reaper.art_rank(art) == 0 {
			return refuse(Refusal::NotOwned);
		}

		Ok(self.with_active(|reaper| reaper.with_art(art)))
	}

	/// Buys `outfit` with anima.
	pub fn buy_outfit(&self, outfit: Outfit) -> RuleResult<Self> {
		self.require_oath()?;

		if self.outfits().contains(&outfit) {
			return refuse(Refusal::AlreadyOwned);
		}

		self.require_level(outfit.level_required())?;
		self.require_anima(outfit.price())?;

		let mut next = self.with_active(|reaper| reaper.with_outfit(outfit));

		next.anima -= outfit.price();

		Ok(next)
	}

	/// Puts on a bought `outfit`.
	pub fn wear(&self, outfit: Outfit) -> RuleResult<Self> {
		if !self.outfits().contains(&outfit) {
			return refuse(Refusal::NotOwned);
		}

		Ok(self.with_active(|reaper| reaper.wearing(outfit)))
	}

	fn require_oath(&self) -> RuleResult<()> {
		if self.oath_taken() {
			Ok(())
		} else {
			refuse(Refusal::OathRequired)
		}
	}

	fn require_level(&self, required: i64) -> RuleResult<()> {
		if self.level() < required {
			refuse(Refusal::LevelTooLow)
		} else {
			Ok(())
		}
	}

	fn require_anima(&self, price: i64) -> RuleResult<()> {
		if self.anima < price {
			refuse(Refusal::InsufficientAnima)
		} else {
			Ok(())
		}
	}

	/// This state with the active reaper changed by `change`.
	fn with_active(&self, change: impl FnOnce(ReaperState) -> ReaperState) -> Self {
		let mut next = self.clone();

		if let Some(reaper) = next
			.reapers
			.iter_mut()
			.find(|reaper| Some(reaper.id()) == self.active_reaper_id.as_deref())
		{
			*reaper = change(reaper.clone());
		}

		next
	}

	/// This state with `roster` as its reapers, keeping the active reaper
	/// when it is still in the roster and taking the first otherwise.
	fn with_roster(&self, roster: Vec<ReaperState>) -> Self {
		let selected = match &self.active_reaper_id {
			Some(id) if roster.iter().any(|reaper| reaper.id() == id) => Some(id.clone()),
			_ => roster.first().map(|reaper| reaper.id().to_owned()),
		};

		Self {
			reapers: roster,
			active_reaper_id: selected,
			..self.clone()
		}
	}
}
