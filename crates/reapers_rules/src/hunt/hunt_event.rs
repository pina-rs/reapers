//! Where an encounter is, what happens in it, and how it ends.

use serde::Deserialize;
use serde::Serialize;

use crate::arts::Art;
use crate::error::FormatError;
use crate::feeding::Feeding;
use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::husk_pose::Footing;
use crate::math;
use crate::named::named_enum;
use crate::nft_catalogue::Rarity;
use crate::reaper_soul::HuntDeeds;
use crate::reaper_soul::ReaperGift;
use crate::vec3::Vec3;

/// Where an encounter is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HuntPhase {
	/// The husk has not noticed the reaper. Moving slowly earns an ambush.
	#[default]
	Stalk,

	/// Seals open and shut, and the reaper strikes them.
	Fight,

	/// The husk is drawing breath. A shriek follows.
	Windup,

	/// The husk is shrieking. The reaper holds still or loses resolve.
	Shriek,

	/// The husk is running at the reaper. A swing of the phone turns it
	/// aside; without one it strikes, and the reaper loses resolve.
	Charge,

	/// The ward is gone and the husk hangs open, waiting to be reaped.
	Broken,

	/// The husk was reaped and absorbed. The hunt is over.
	Absorbed,

	/// The husk slipped away. The hunt is over.
	Escaped,
}

named_enum!(HuntPhase {
	Stalk => "stalk",
	Fight => "fight",
	Windup => "windup",
	Shriek => "shriek",
	Charge => "charge",
	Broken => "broken",
	Absorbed => "absorbed",
	Escaped => "escaped",
});

/// How cleanly the reap was cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReapGrade {
	/// The cut tore: too fast, or too far from the arc.
	Ragged,

	/// A steady cut along the arc.
	Clean,

	/// A cut that barely left the arc at all.
	Perfect,
}

named_enum!(ReapGrade {
	Ragged => "ragged",
	Clean => "clean",
	Perfect => "perfect",
});

impl ReapGrade {
	/// Multiplier on the experience and anima the husk is worth.
	pub const fn reward(self) -> f64 {
		match self {
			Self::Ragged => 1.0,
			Self::Clean => 1.25,
			Self::Perfect => 1.5,
		}
	}
}

/// Why a husk got away.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EscapeCause {
	/// The reaper flinched until no resolve was left.
	Resolve,

	/// The fight ran past its limit.
	Time,

	/// Too many reap windows closed unfinished.
	Reap,
}

named_enum!(EscapeCause {
	Resolve => "resolve",
	Time => "time",
	Reap => "reap",
});

/// Something that happened during one tick, for the view to show and sound.
///
/// Events describe results the rules have already decided. The view never
/// feeds anything back from them.
#[derive(Clone, Debug, PartialEq)]
pub enum HuntEvent {
	/// A birth gift changed this moment of the hunt.
	GiftStirred(ReaperGift),

	/// The stalk is over and the fight has begun.
	StalkEnded {
		/// Whether the reaper steadied on the husk before it noticed.
		ambush: bool,
	},

	/// A strike landed on an open seal.
	StrikeLanded {
		seal: usize,

		/// Ward removed, including anything the strike cleaved.
		damage: f64,

		/// Hits in a row, counting this one.
		cadence: i64,

		/// Whether an ambush, a perk or striking in order strengthened it.
		empowered: bool,

		/// Further seals the same strike caught.
		cleaved: Vec<usize>,
	},

	/// A strike hit the husk's body, away from any open seal.
	StrikeGlanced { damage: f64 },

	/// A strike hit nothing.
	StrikeWhiffed,

	/// A seal opened.
	SealOpened(usize),

	/// The husk is drawing breath to shriek.
	ShriekWindup,

	/// The shriek has started: hold still.
	ShriekBegan,

	/// The reaper moved or struck during a shriek.
	Flinched {
		resolve_left: i64,

		/// Whether a perk absorbed the flinch, so no resolve was lost.
		forgiven: bool,
	},

	/// The reaper held still through a whole shriek. Every seal opens.
	HeldFirm,

	/// The husk has turned on the reaper and is coming at them.
	ChargeBegan,

	/// The reaper swung in time and turned the charge aside. The husk is
	/// stunned, with every seal in view open.
	Parried,

	/// The charge reached the reaper.
	Struck { resolve_left: i64 },

	/// The husk is darting to a new place.
	Relocating,

	/// The husk has sprung into the air: up onto a ledge, down from one,
	/// over whatever it hid behind or behind it. It will land on the
	/// footing given.
	Leapt(Footing),

	/// The reaper flicked the phone and their art is in the air.
	ArtThrown(Art),

	/// A thrown art came down.
	ArtLanded {
		art: Art,

		/// Where it came down, in metres from the player's eye.
		at: Vec3,

		/// Whether it caught the husk.
		hit: bool,

		/// Ward it removed as it landed. Burning takes more after.
		damage: f64,
	},

	/// The blow that broke a frozen husk's ice.
	IceShattered,

	/// The ward broke. The reap arc is showing.
	Broken,

	/// The reaper's aim reached the start of the arc and the cut has begun.
	ReapBegan,

	/// The reap window closed unfinished and the husk pulled itself
	/// together.
	ReapRecovered { attempts_left: i64 },

	/// The husk was reaped.
	HuskAbsorbed(ReapGrade),

	/// The husk slipped away.
	HuskEscaped(EscapeCause),
}

impl HuntEvent {
	/// The name of the event's kind, as the Dart rules named the class.
	pub const fn kind(&self) -> &'static str {
		match self {
			Self::GiftStirred(_) => "GiftStirred",
			Self::StalkEnded { .. } => "StalkEnded",
			Self::StrikeLanded { .. } => "StrikeLanded",
			Self::StrikeGlanced { .. } => "StrikeGlanced",
			Self::StrikeWhiffed => "StrikeWhiffed",
			Self::SealOpened(_) => "SealOpened",
			Self::ShriekWindup => "ShriekWindup",
			Self::ShriekBegan => "ShriekBegan",
			Self::Flinched { .. } => "Flinched",
			Self::HeldFirm => "HeldFirm",
			Self::ChargeBegan => "ChargeBegan",
			Self::Parried => "Parried",
			Self::Struck { .. } => "Struck",
			Self::Relocating => "Relocating",
			Self::Leapt(_) => "Leapt",
			Self::ArtThrown(_) => "ArtThrown",
			Self::ArtLanded { .. } => "ArtLanded",
			Self::IceShattered => "IceShattered",
			Self::Broken => "Broken",
			Self::ReapBegan => "ReapBegan",
			Self::ReapRecovered { .. } => "ReapRecovered",
			Self::HuskAbsorbed(_) => "HuskAbsorbed",
			Self::HuskEscaped(_) => "HuskEscaped",
		}
	}
}

/// How a hunt ended: the husk absorbed with a grade, or escaped for a cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HuntEnd {
	Absorbed(ReapGrade),
	Escaped(EscapeCause),
}

/// How a hunt ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "HuntOutcomeRecord", into = "HuntOutcomeRecord")]
pub struct HuntOutcome {
	pub end: HuntEnd,

	/// Whether the fight opened with an ambush.
	pub ambushed: bool,

	/// How long the hunt ran.
	pub ticks: i64,
	pub deeds: HuntDeeds,
}

/// A hunt outcome as saves and the wire write it.
#[derive(Clone, Copy, Serialize, Deserialize)]
struct HuntOutcomeRecord {
	grade: Option<ReapGrade>,
	escape: Option<EscapeCause>,
	ambushed: bool,
	ticks: i64,
	#[serde(default)]
	deeds: Option<HuntDeeds>,
}

impl TryFrom<HuntOutcomeRecord> for HuntOutcome {
	type Error = FormatError;

	fn try_from(record: HuntOutcomeRecord) -> Result<Self, FormatError> {
		let end = match (record.grade, record.escape) {
			(Some(grade), _) => HuntEnd::Absorbed(grade),
			(None, Some(escape)) => HuntEnd::Escaped(escape),
			(None, None) => {
				return Err(FormatError::new(
					"A hunt outcome needs a grade or an escape",
				));
			}
		};

		Ok(Self {
			end,
			ambushed: record.ambushed,
			ticks: record.ticks,
			deeds: record.deeds.unwrap_or_default(),
		})
	}
}

impl From<HuntOutcome> for HuntOutcomeRecord {
	fn from(outcome: HuntOutcome) -> Self {
		Self {
			grade: outcome.grade(),
			escape: outcome.escape(),
			ambushed: outcome.ambushed,
			ticks: outcome.ticks,
			deeds: Some(outcome.deeds),
		}
	}
}

impl HuntOutcome {
	pub const fn absorbed_with(
		grade: ReapGrade,
		ambushed: bool,
		ticks: i64,
		deeds: HuntDeeds,
	) -> Self {
		Self {
			end: HuntEnd::Absorbed(grade),
			ambushed,
			ticks,
			deeds,
		}
	}

	pub const fn escaped_by(
		escape: EscapeCause,
		ambushed: bool,
		ticks: i64,
		deeds: HuntDeeds,
	) -> Self {
		Self {
			end: HuntEnd::Escaped(escape),
			ambushed,
			ticks,
			deeds,
		}
	}

	/// The grade of the reap, or `None` when the husk escaped.
	pub const fn grade(&self) -> Option<ReapGrade> {
		match self.end {
			HuntEnd::Absorbed(grade) => Some(grade),
			HuntEnd::Escaped(_) => None,
		}
	}

	/// Why the husk escaped, or `None` when it was absorbed.
	pub const fn escape(&self) -> Option<EscapeCause> {
		match self.end {
			HuntEnd::Absorbed(_) => None,
			HuntEnd::Escaped(escape) => Some(escape),
		}
	}

	pub const fn absorbed(&self) -> bool {
		matches!(self.end, HuntEnd::Absorbed(_))
	}

	/// Experience earned from a husk of `rarity` that fed the reaper as
	/// `feeding` says.
	pub fn xp_for(&self, rarity: Rarity, feeding: Feeding) -> i64 {
		math::round(self.reward(rarity.xp()) * feeding.share())
	}

	/// Anima earned from a husk of `rarity`. The feeding does not change it.
	pub fn anima_for(&self, rarity: Rarity) -> i64 {
		math::round(self.reward(rarity.anima()))
	}

	fn reward(&self, base: i64) -> f64 {
		let Some(grade) = self.grade() else {
			return 0.0;
		};
		let ambush = if self.ambushed {
			HuntRules::AMBUSH_REWARD
		} else {
			1.0
		};

		base as f64 * grade.reward() * ambush
	}
}
