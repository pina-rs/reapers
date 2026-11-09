//! Where an encounter is, what happens in it, and how it ends.

use serde::Deserialize;
use serde::Serialize;

use crate::arts::Art;
use crate::error::FormatError;
use crate::feeding::Feeding;
use crate::hunt::arsenal::Archetype;
use crate::hunt::capture::CaptureGrade;
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
	/// The husk has not noticed the reaper. Holding still on it earns an
	/// ambush.
	#[default]
	Stalk,

	/// Seals open and shut, and the reaper fights.
	Fight,

	/// The husk is drawing breath. A shriek follows.
	Windup,

	/// The husk is shrieking. The reaper holds still or loses resolve.
	Shriek,

	/// The husk is running at the reaper. A swing of the phone turns it
	/// aside; without one it strikes, and the reaper loses resolve.
	Charge,

	/// The ward is gone and the husk has collapsed, waiting for an orb.
	Broken,

	/// An orb struck the husk and holds it. It can do nothing until the orb
	/// holds or breaks.
	Capture,

	/// The husk was captured and absorbed. The hunt is over.
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
	Capture => "capture",
	Absorbed => "absorbed",
	Escaped => "escaped",
});

/// What the coloured ring under the husk says, and the caption with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HuskRing {
	/// White: nothing is coming. Attack.
	Open,

	/// Amber: it is drawing breath, or shrieking. Hold still.
	DrawingBreath,

	/// Red: it is charging. Swing the phone to parry.
	Charging,

	/// Blue: it is stunned and close. Dive in.
	Stunned,

	/// Gold: it is weak enough for an orb, or broken. Throw one.
	Weakened,
}

/// How cleanly a reaper cut a rift's husk.
///
/// A rift is sealed by cuts, and pays by them. A landed orb is a cut: its
/// [`CaptureGrade::cut`] says which.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReapGrade {
	/// An orb thrown without a hold, or off the husk.
	Ragged,

	/// A nice or a great throw.
	Clean,

	/// An excellent throw.
	Perfect,
}

named_enum!(ReapGrade {
	Ragged => "ragged",
	Clean => "clean",
	Perfect => "perfect",
});

impl ReapGrade {
	/// Multiplier on what sealing the rift pays.
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
	/// The reaper flinched, or was struck, until no resolve was left.
	Resolve,

	/// The fight ran past its limit.
	Time,

	/// Too many broken windows passed without an orb.
	Lapsed,
}

named_enum!(EscapeCause {
	Resolve => "resolve",
	Time => "time",
	Lapsed => "lapsed",
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

	/// A newly selected weapon is being drawn.
	WeaponDrawn(usize),

	/// The gun fired.
	ShotFired {
		/// Rounds left in the magazine after this one.
		rounds_left: i64,

		/// Whether it came from a magazine an active reload loaded.
		boosted: bool,
	},

	/// A shot, a stab or a swing landed on the husk.
	HitLanded {
		/// What landed it.
		weapon: Archetype,

		/// Ward removed, including anything the hit cleaved.
		damage: f64,

		/// Hits in a row, counting this one.
		cadence: i64,

		/// Whether it struck an open seal, or a stunned husk with the blade,
		/// and so landed twice as hard.
		critical: bool,

		/// Whether an ambush, a perk or a gift strengthened it.
		empowered: bool,

		/// The open seal it struck, or `None` for a hit on the body.
		seal: Option<usize>,

		/// Further seals the same hit caught.
		cleaved: Vec<usize>,
	},

	/// A shot, a stab or a swing hit nothing.
	Missed(Archetype),

	/// FIRE on an empty gun began a reload.
	ReloadStarted,

	/// FIRE came too early in a reload and jammed it.
	ReloadJammed,

	/// The gun is loaded again.
	ReloadFinished {
		/// Whether an active reload finished it early, so the magazine hits
		/// harder.
		perfect: bool,
	},

	/// The reaper dove in at the husk.
	DiveStarted(Archetype),

	/// The reaper is back from a dive.
	DiveEnded,

	/// A seal opened.
	SealOpened(usize),

	/// The husk is drawing breath to shriek.
	ShriekWindup,

	/// The shriek has started: hold still.
	ShriekBegan,

	/// The reaper moved, fired or threw during a shriek.
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
	/// stunned, close, with every seal in view open.
	Parried,

	/// The charge reached the reaper.
	Struck {
		resolve_left: i64,

		/// Whether it caught the reaper mid-dive, and so struck for double.
		diving: bool,
	},

	/// The husk is darting to a new place.
	Relocating,

	/// The husk has sprung into the air: up onto a ledge, down from one,
	/// over whatever it hid behind or behind it. It will land on the
	/// footing given.
	Leapt(Footing),

	/// The reaper's art is in the air.
	ArtThrown(Art),

	/// A thrown art came down.
	ArtLanded {
		art: Art,

		/// Where it came down, in metres from the hunt origin.
		at: Vec3,

		/// Whether it caught the husk.
		hit: bool,

		/// Ward it removed as it landed. Burning takes more after.
		damage: f64,
	},

	/// The blow that broke a frozen husk's ice.
	IceShattered,

	/// The ward broke. The husk has collapsed for an orb.
	Broken {
		/// How long it stays down.
		window_ticks: i64,
	},

	/// A broken window passed without an orb, and the husk pulled itself
	/// together and ran.
	Recovered {
		/// Broken windows it will still let pass before it escapes.
		lapses_left: i64,
	},

	/// ORB was let go and the orb is in the air.
	OrbThrown {
		grade: CaptureGrade,

		/// Whether it will strike the husk.
		on_target: bool,
	},

	/// The orb missed the husk and fell. Its soul is spent.
	OrbMissed,

	/// The orb struck the husk.
	OrbImpact {
		/// The chance it holds.
		chance: f64,
	},

	/// The husk has been drawn into the orb as light.
	OrbAbsorbed,

	/// The orb dropped onto the surface and lies there.
	OrbLanded,

	/// The orb pulsed: the husk inside it is fighting.
	OrbPulse(i64),

	/// The orb cracked on `pulse` and the husk burst out, stunned.
	OrbBrokeFree(i64),

	/// The orb held. The husk is captured.
	HuskCaptured(CaptureGrade),

	/// The husk slipped away.
	HuskEscaped(EscapeCause),
}

impl HuntEvent {
	/// The name of the event's kind, as logs and fixtures name it.
	pub const fn kind(&self) -> &'static str {
		match self {
			Self::GiftStirred(_) => "GiftStirred",
			Self::StalkEnded { .. } => "StalkEnded",
			Self::WeaponDrawn(_) => "WeaponDrawn",
			Self::ShotFired { .. } => "ShotFired",
			Self::HitLanded { .. } => "HitLanded",
			Self::Missed(_) => "Missed",
			Self::ReloadStarted => "ReloadStarted",
			Self::ReloadJammed => "ReloadJammed",
			Self::ReloadFinished { .. } => "ReloadFinished",
			Self::DiveStarted(_) => "DiveStarted",
			Self::DiveEnded => "DiveEnded",
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
			Self::Broken { .. } => "Broken",
			Self::Recovered { .. } => "Recovered",
			Self::OrbThrown { .. } => "OrbThrown",
			Self::OrbMissed => "OrbMissed",
			Self::OrbImpact { .. } => "OrbImpact",
			Self::OrbAbsorbed => "OrbAbsorbed",
			Self::OrbLanded => "OrbLanded",
			Self::OrbPulse(_) => "OrbPulse",
			Self::OrbBrokeFree(_) => "OrbBrokeFree",
			Self::HuskCaptured(_) => "HuskCaptured",
			Self::HuskEscaped(_) => "HuskEscaped",
		}
	}
}

/// How a hunt ended: the husk captured with a grade, or escaped for a cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HuntEnd {
	Absorbed(CaptureGrade),
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
	grade: Option<CaptureGrade>,
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
		grade: CaptureGrade,
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

	/// How well the capturing orb was thrown, or `None` when the husk
	/// escaped.
	pub const fn grade(&self) -> Option<CaptureGrade> {
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
