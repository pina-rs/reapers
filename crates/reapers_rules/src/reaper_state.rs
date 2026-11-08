//! The progression that travels with one reaper.

use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use crate::armory::Outfit;
use crate::armory::Temper;
use crate::armory::Weapon;
use crate::arts::Art;
use crate::arts::ArtSkill;
use crate::error::FormatError;
use crate::player_state::Absorption;
use crate::progression::Progression;
use crate::reaper_genome::ReaperGenome;
use crate::reaper_soul::ReaperSoul;
use crate::reaper_soul::SoulMemory;
use crate::reaper_soul::SoulRules;

/// The progression that travels with one reaper, never with a selected
/// slot. NFT-backed ids are their Core asset addresses; the starter has a
/// local id.
///
/// A reaper record is always one a reaper could have: it has an id, no
/// count is negative, it holds the weapon and outfit it wears, and the art
/// it has ready is one it knows.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "ReaperRecord")]
pub struct ReaperState {
	id: String,
	genome: ReaperGenome,
	xp: i64,
	weapons: Vec<Weapon>,
	outfits: Vec<Outfit>,
	weapon: Weapon,
	outfit: Outfit,
	codex: Vec<Absorption>,
	hunts_lost: i64,
	rifts_sealed: i64,
	memory: SoulMemory,
	art: Option<Art>,
}

/// A reaper as saves write it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReaperRecord {
	pub id: String,
	pub genome: String,
	pub xp: i64,
	pub weapons: Vec<Weapon>,
	pub outfits: Vec<Outfit>,
	pub weapon: Weapon,
	pub outfit: Outfit,
	pub codex: Vec<Absorption>,
	pub hunts_lost: i64,
	pub rifts_sealed: i64,
	#[serde(default)]
	pub memory: Option<SoulMemory>,
	#[serde(default)]
	pub art: Option<Art>,
}

impl TryFrom<ReaperRecord> for ReaperState {
	type Error = FormatError;

	fn try_from(record: ReaperRecord) -> Result<Self, FormatError> {
		Self::new(
			record.id,
			ReaperGenome::parse(&record.genome)?,
			record.xp,
			record.weapons,
			record.outfits,
			record.weapon,
			record.outfit,
			record.codex,
			record.hunts_lost,
			record.rifts_sealed,
			record.memory.unwrap_or_default(),
			record.art,
		)
	}
}

impl Serialize for ReaperState {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		ReaperRecord {
			id: self.id.clone(),
			genome: self.genome.code(),
			xp: self.xp,
			weapons: self.weapons.clone(),
			outfits: self.outfits.clone(),
			weapon: self.weapon,
			outfit: self.outfit,
			codex: self.codex.clone(),
			hunts_lost: self.hunts_lost,
			rifts_sealed: self.rifts_sealed,
			memory: Some(self.memory),
			art: self.art,
		}
		.serialize(serializer)
	}
}

/// Keeps the first of each item, in the order given: a set that remembers
/// the order things were added to it.
fn unique<T: PartialEq>(items: Vec<T>) -> Vec<T> {
	let mut kept: Vec<T> = Vec::with_capacity(items.len());

	for item in items {
		if !kept.contains(&item) {
			kept.push(item);
		}
	}

	kept
}

impl ReaperState {
	/// The id of the non-tradable starter reaper bound by the oath.
	pub const OATH_ID: &'static str = "oathbound";

	/// A reaper record, refused when it is not one a reaper could have.
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		id: String,
		genome: ReaperGenome,
		xp: i64,
		weapons: Vec<Weapon>,
		outfits: Vec<Outfit>,
		weapon: Weapon,
		outfit: Outfit,
		codex: Vec<Absorption>,
		hunts_lost: i64,
		rifts_sealed: i64,
		memory: SoulMemory,
		art: Option<Art>,
	) -> Result<Self, FormatError> {
		let reaper = Self {
			id,
			genome,
			xp,
			weapons: unique(weapons),
			outfits: unique(outfits),
			weapon,
			outfit,
			codex,
			hunts_lost,
			rifts_sealed,
			memory,
			art,
		};

		if reaper.id.is_empty()
			|| reaper.xp < 0
			|| reaper.hunts_lost < 0
			|| reaper.rifts_sealed < 0
			|| !reaper.weapons.contains(&reaper.weapon)
			|| !reaper.outfits.contains(&reaper.outfit)
			|| reaper.art.is_some_and(|art| reaper.art_rank(art) == 0)
		{
			return Err(FormatError::new("Invalid reaper record"));
		}

		Ok(reaper)
	}

	/// A newly bound reaper: its class's own weapon, undyed cloth, and
	/// nothing else.
	pub fn bound(id: impl Into<String>, genome: ReaperGenome) -> Self {
		let weapon = Weapon::new(genome.reaper_class.weapon(), Temper::Shrouded);

		Self {
			id: id.into(),
			genome,
			xp: 0,
			weapons: vec![weapon],
			outfits: vec![Outfit::Shrouded],
			weapon,
			outfit: Outfit::Shrouded,
			codex: Vec::new(),
			hunts_lost: 0,
			rifts_sealed: 0,
			memory: SoulMemory::default(),
			art: None,
		}
	}

	pub fn id(&self) -> &str {
		&self.id
	}

	/// The genome's permanent code.
	pub fn genome_code(&self) -> String {
		self.genome.code()
	}

	pub const fn genome(&self) -> &ReaperGenome {
		&self.genome
	}

	pub const fn xp(&self) -> i64 {
		self.xp
	}

	/// Every weapon the reaper owns, in the order it came by them.
	pub fn weapons(&self) -> &[Weapon] {
		&self.weapons
	}

	/// Every outfit the reaper owns, in the order it came by them.
	pub fn outfits(&self) -> &[Outfit] {
		&self.outfits
	}

	/// The weapon in hand.
	pub const fn weapon(&self) -> Weapon {
		self.weapon
	}

	/// The outfit worn.
	pub const fn outfit(&self) -> Outfit {
		self.outfit
	}

	/// Every husk the reaper has absorbed, oldest first.
	pub fn codex(&self) -> &[Absorption] {
		&self.codex
	}

	pub const fn hunts_lost(&self) -> i64 {
		self.hunts_lost
	}

	pub const fn rifts_sealed(&self) -> i64 {
		self.rifts_sealed
	}

	pub const fn memory(&self) -> &SoulMemory {
		&self.memory
	}

	/// The art the reaper has made ready to throw, out of those it knows.
	pub const fn art(&self) -> Option<Art> {
		self.art
	}

	pub fn is_nft(&self) -> bool {
		self.id != Self::OATH_ID
	}

	pub fn level(&self) -> i64 {
		Progression::level_for_xp(self.xp)
	}

	pub fn soul(&self) -> ReaperSoul {
		ReaperSoul::from_genome(&self.genome)
	}

	pub fn soul_rank(&self) -> i64 {
		SoulRules::rank(self.level(), &self.memory, self.soul().gift)
	}

	/// How deep `art` runs in this reaper: 0 until it has absorbed a husk
	/// that carries it. It is read from the codex, so it travels with the
	/// reaper and cannot be bought.
	pub fn art_rank(&self, art: Art) -> i64 {
		Art::rank_for(
			self.codex
				.iter()
				.filter(|entry| art.carried_by_vessel(entry.nft.collection.vessel()))
				.count(),
		)
	}

	/// The art the reaper throws, at the rank it knows it.
	pub fn art_skill(&self) -> Option<ArtSkill> {
		self.art
			.and_then(|art| ArtSkill::new(art, self.art_rank(art)))
	}

	pub(crate) fn with_xp(mut self, xp: i64) -> Self {
		self.xp = xp;
		self
	}

	pub(crate) fn with_weapon(mut self, weapon: Weapon) -> Self {
		if !self.weapons.contains(&weapon) {
			self.weapons.push(weapon);
		}
		self
	}

	pub(crate) fn wielding(mut self, weapon: Weapon) -> Self {
		self.weapon = weapon;
		self
	}

	pub(crate) fn with_outfit(mut self, outfit: Outfit) -> Self {
		if !self.outfits.contains(&outfit) {
			self.outfits.push(outfit);
		}
		self
	}

	pub(crate) fn wearing(mut self, outfit: Outfit) -> Self {
		self.outfit = outfit;
		self
	}

	pub(crate) fn with_codex(mut self, codex: Vec<Absorption>) -> Self {
		self.codex = codex;
		self
	}

	pub(crate) fn with_hunts_lost(mut self, hunts_lost: i64) -> Self {
		self.hunts_lost = hunts_lost;
		self
	}

	pub(crate) fn with_rifts_sealed(mut self, rifts_sealed: i64) -> Self {
		self.rifts_sealed = rifts_sealed;
		self
	}

	pub(crate) fn with_memory(mut self, memory: SoulMemory) -> Self {
		self.memory = memory;
		self
	}

	pub(crate) fn with_art(mut self, art: Art) -> Self {
		self.art = Some(art);
		self
	}
}
