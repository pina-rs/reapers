//! A reaper's birth gift and temperament, and the deeds that awaken it.

use serde::Deserialize;
use serde::Serialize;

use crate::error::FormatError;
use crate::hunt::hunt_event::HuntOutcome;
use crate::named::named_enum;
use crate::reaper_class::ReaperClass;
use crate::reaper_genome::ReaperGenome;

/// The kind of successful hunt that teaches a birth gift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum SoulPath {
	Unseen,
	Stillness,
	Rhythm,
}

named_enum!(SoulPath {
	Unseen => "unseen",
	Stillness => "stillness",
	Rhythm => "rhythm",
});

/// A fixed, common disposition, with a benefit and a cost. Rare omens and
/// colours never improve these odds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Temperament {
	Patient,
	Keen,
	Stalwart,
	Hungry,
}

named_enum!(Temperament {
	Patient => "patient",
	Keen => "keen",
	Stalwart => "stalwart",
	Hungry => "hungry",
});

/// Original powers belonging to the six offices of death. The ordering is
/// part of the R1 genome contract: append a new version, never reorder this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ReaperGift {
	Furrow,
	Aftermath,
	Gathering,
	Passage,
	Anchor,
	ReturnLight,
	Foreknowledge,
	Verdict,
	Equilibrium,
	FirstLight,
	Reprisal,
	LastWord,
	Hunger,
	Echo,
	Unbroken,
	Hush,
	Keening,
	Refrain,
}

named_enum!(ReaperGift {
	Furrow => "furrow",
	Aftermath => "aftermath",
	Gathering => "gathering",
	Passage => "passage",
	Anchor => "anchor",
	ReturnLight => "returnLight",
	Foreknowledge => "foreknowledge",
	Verdict => "verdict",
	Equilibrium => "equilibrium",
	FirstLight => "firstLight",
	Reprisal => "reprisal",
	LastWord => "lastWord",
	Hunger => "hunger",
	Echo => "echo",
	Unbroken => "unbroken",
	Hush => "hush",
	Keening => "keening",
	Refrain => "refrain",
});

impl ReaperGift {
	/// The class whose office this gift belongs to: three gifts each, in
	/// class order.
	pub const fn reaper_class(self) -> ReaperClass {
		ReaperClass::ALL[self as usize / SoulRules::GIFTS_PER_CLASS]
	}

	/// The kind of hunt that teaches it.
	pub const fn path(self) -> SoulPath {
		SoulPath::ALL[self as usize % SoulRules::GIFTS_PER_CLASS]
	}
}

/// Birth identity derived only from permanent R1 fields. No extra random
/// roll, client claim or NFT metadata can change a reaper's power.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReaperSoul {
	pub gift: ReaperGift,
	pub temperament: Temperament,
}

impl ReaperSoul {
	pub fn from_genome(genome: &ReaperGenome) -> Self {
		let per_class = SoulRules::GIFTS_PER_CLASS as u32;
		let offset = (genome.given_name * 17 + genome.epithet * 31 + genome.gaze) % per_class;
		let temperaments = Temperament::ALL.len() as u32;

		Self {
			gift: ReaperGift::ALL
				[genome.reaper_class.index() * SoulRules::GIFTS_PER_CLASS + offset as usize],
			temperament: Temperament::ALL
				[((genome.stature + genome.gauntness + genome.tatter) % temperaments) as usize],
		}
	}
}

/// Replay observations, never submitted separately by the player. Counters
/// describe an encounter; a memory awards at most one mark per path per win.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "HuntDeedsRecord", rename_all = "camelCase")]
pub struct HuntDeeds {
	pub holds: i64,
	pub parries: i64,
	pub max_cadence: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HuntDeedsRecord {
	holds: i64,
	parries: i64,
	max_cadence: i64,
}

impl TryFrom<HuntDeedsRecord> for HuntDeeds {
	type Error = FormatError;

	fn try_from(record: HuntDeedsRecord) -> Result<Self, FormatError> {
		if record.holds < 0 || record.parries < 0 || record.max_cadence < 0 {
			return Err(FormatError::new("Negative hunt deed"));
		}

		Ok(Self {
			holds: record.holds,
			parries: record.parries,
			max_cadence: record.max_cadence,
		})
	}
}

/// Earned history belongs to the reaper and travels with its canonical NFT
/// record. Old saves begin without invented deeds; XP and equipment survive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "SoulMemoryRecord")]
pub struct SoulMemory {
	pub unseen: i64,
	pub stillness: i64,
	pub rhythm: i64,
}

#[derive(Deserialize)]
struct SoulMemoryRecord {
	unseen: i64,
	stillness: i64,
	rhythm: i64,
}

impl TryFrom<SoulMemoryRecord> for SoulMemory {
	type Error = FormatError;

	fn try_from(record: SoulMemoryRecord) -> Result<Self, FormatError> {
		if record.unseen < 0 || record.stillness < 0 || record.rhythm < 0 {
			return Err(FormatError::new("Negative soul memory"));
		}

		Ok(Self {
			unseen: record.unseen,
			stillness: record.stillness,
			rhythm: record.rhythm,
		})
	}
}

impl SoulMemory {
	pub const fn marks(&self, path: SoulPath) -> i64 {
		match path {
			SoulPath::Unseen => self.unseen,
			SoulPath::Stillness => self.stillness,
			SoulPath::Rhythm => self.rhythm,
		}
	}

	/// Losing or abandoning a hunt grants no marks. Waiting for repeated
	/// shrieks cannot farm more marks than a single well-played absorption.
	#[must_use]
	pub fn remember(&self, outcome: &HuntOutcome) -> Self {
		if !outcome.absorbed() {
			return *self;
		}

		Self {
			unseen: self.unseen.saturating_add(i64::from(outcome.ambushed)),
			stillness: self
				.stillness
				.saturating_add(i64::from(outcome.deeds.holds > 0)),
			rhythm: self.rhythm.saturating_add(i64::from(
				outcome.deeds.max_cadence >= SoulRules::CADENCE_FOR_MARK,
			)),
		}
	}
}

/// Every new progression tunable. Read docs/gameplay.md and the identity
/// research before changing these playtest starting values.
pub struct SoulRules;

impl SoulRules {
	pub const CADENCE_FOR_MARK: i64 = 5;
	pub const EQUILIBRIUM_HITS: i64 = 3;
	pub const EQUILIBRIUM_TICKS_PER_STRENGTH: i64 = 24;
	pub const EXECUTION_CADENCE: i64 = 3;
	pub const EXECUTION_WARD_SHARE: f64 = 0.35;
	pub const FOREKNOWLEDGE_TICKS_PER_STRENGTH: i64 = 30;
	pub const FURROW_CLEAVE_PER_STRENGTH: f64 = 0.15;
	pub const GATHERING_HITS: i64 = 3;
	pub const GATHERING_TICKS_PER_STRENGTH: i64 = 24;
	pub const GIFTS_PER_CLASS: usize = 3;
	pub const GIFT_STRIKE_PER_STRENGTH: f64 = 0.25;
	pub const HUNGRY_FLINCH: f64 = 0.92;
	pub const HUNGRY_STRIKE: f64 = 1.08;
	pub const HUSH_TICKS_PER_STRENGTH: i64 = 30;
	pub const KEEN_BROKEN_TICKS: i64 = 24;
	pub const KEEN_REACH_DEGREES: f64 = 0.6;
	pub const MAX_RANK: i64 = 3;
	pub const PASSAGE_STAGGER_TICKS_PER_STRENGTH: i64 = 30;
	pub const PATIENT_BROKEN_TICKS: i64 = 30;
	pub const PATIENT_SWING_TICKS: i64 = 2;
	pub const RANK_LEVELS: [i64; 4] = [1, 5, 12, 20];
	pub const RANK_MARKS: [i64; 4] = [0, 3, 12, 30];
	pub const REFRAIN_HITS: i64 = 3;
	pub const REFRAIN_MAX_TOLLS: i64 = 6;
	pub const REFRAIN_TICKS_PER_STRENGTH: i64 = 15;
	pub const RETURN_CADENCE: i64 = 5;
	pub const STALWART_STRIKE: f64 = 0.95;

	/// Level and demonstrated practice are both needed. Money, rarity and
	/// feeding luck alone cannot awaken a gift's deeper behaviour.
	pub fn rank(level: i64, memory: &SoulMemory, gift: ReaperGift) -> i64 {
		let mut rank = 0;

		for next in 1..=Self::MAX_RANK as usize {
			if level < Self::RANK_LEVELS[next] || memory.marks(gift.path()) < Self::RANK_MARKS[next]
			{
				break;
			}

			rank = next as i64;
		}

		rank
	}
}
