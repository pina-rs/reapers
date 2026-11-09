//! Weakening a husk and catching it in a soul orb.

use crate::hunt::hunt_event::ReapGrade;
use crate::math;
use crate::named::named_enum;
use crate::nft_catalogue::Rarity;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;
use crate::vec3::Vec3;

/// How well an orb was timed: where the shrinking ring was when ORB was let
/// go.
///
/// A grade raises the odds of the capture and, if it holds, what the husk
/// pays. The two use different scales. Three grades are kept, because the
/// deployed program reads the grade byte of a receipt and refuses anything
/// above two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CaptureGrade {
	/// Thrown without a hold, with the ring in its outer third, or with the
	/// crosshair off the husk.
	#[default]
	Plain,

	/// Let go with the ring in its middle third.
	Great,

	/// In its inner third.
	Excellent,
}

named_enum!(CaptureGrade {
	Plain => "plain",
	Great => "great",
	Excellent => "excellent",
});

impl CaptureGrade {
	/// Multiplier on the capture chance.
	pub const fn chance(self) -> f64 {
		match self {
			Self::Plain => 1.0,
			Self::Great => 1.3,
			Self::Excellent => 1.5,
		}
	}

	/// Multiplier on the experience and anima the husk is worth.
	pub const fn reward(self) -> f64 {
		match self {
			Self::Plain => 1.0,
			Self::Great => 1.25,
			Self::Excellent => 1.5,
		}
	}

	/// The cut this grade counts as in a rift, whose sealing and pay are
	/// graded by cuts.
	pub const fn cut(self) -> ReapGrade {
		match self {
			Self::Plain => ReapGrade::Ragged,
			Self::Great => ReapGrade::Clean,
			Self::Excellent => ReapGrade::Perfect,
		}
	}

	/// Reads a grade saved before husks were caught with an orb, when the
	/// codex kept how cleanly they were reaped. Each old grade becomes the
	/// capture grade that pays the same.
	pub fn from_saved(name: &str) -> Option<Self> {
		match name {
			"ragged" => Some(Self::Plain),
			"clean" => Some(Self::Great),
			"perfect" => Some(Self::Excellent),
			other => Self::from_name(other),
		}
	}
}

/// Why an orb cannot be thrown right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrbBlock {
	/// There is no fight to throw it into: the husk has not been found, or
	/// it is already held, or the hunt is over.
	NotFighting,

	/// The husk's ward is still above [`CaptureRules::WEAKENED_SHARE`] of
	/// what it started with. In a rift, the shared ward has not broken.
	NotWeakened,

	/// The reaper has less than [`CaptureRules::COST`] soul energy.
	LowSoul,

	/// An orb is already in the air.
	InFlight,
}

/// Where a thrown orb is in its short life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrbStage {
	/// In the air, on its way to the husk.
	Flight,

	/// It struck the husk, which is drawn into it as light.
	Drawing,

	/// It drops onto the surface under where the husk was.
	Falling,

	/// It lies there and pulses.
	Pulsing,

	/// After the third pulse: the held breath before the result.
	Breath,

	/// It held: the husk is captured.
	Captured,

	/// It cracked and the husk burst out.
	BrokeFree,

	/// It missed the husk and fell. The soul it cost is spent.
	Missed,
}

impl OrbStage {
	/// The stage of an orb `ticks` after ORB was let go: thrown `on_target`
	/// or not, with `broke_on` the pulse it broke out on once it has, and
	/// `held` once it has held. The hunt reports its own orbs this way
	/// ([`HuntSim::orb`](crate::HuntSim::orb)); a view that plays an orb on
	/// its own, such as a bench, can call it to tell the same story.
	pub const fn at(ticks: i64, on_target: bool, broke_on: Option<i64>, held: bool) -> Self {
		if !on_target {
			return if ticks < CaptureRules::IMPACT_TICK {
				Self::Flight
			} else {
				Self::Missed
			};
		}

		if broke_on.is_some() {
			return Self::BrokeFree;
		}

		if held {
			return Self::Captured;
		}

		if ticks < CaptureRules::IMPACT_TICK {
			Self::Flight
		} else if ticks < CaptureRules::DRAWN_TICK {
			Self::Drawing
		} else if ticks < CaptureRules::LANDED_TICK {
			Self::Falling
		} else if ticks < CaptureRules::PULSE_TICKS[2] {
			Self::Pulsing
		} else {
			Self::Breath
		}
	}
}

/// An orb the reaper threw, as the view sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbView {
	/// The tick ORB was let go.
	pub thrown_at: i64,

	/// Ticks since then.
	pub ticks: i64,

	pub stage: OrbStage,

	pub grade: CaptureGrade,

	/// Whether it was thrown close enough to the husk to strike it.
	pub on_target: bool,

	/// Pulses it has made so far, from 0 to 3.
	pub pulses: i64,

	/// Where it flies to, in metres from the hunt origin: the husk, or where it
	/// falls when it misses.
	pub target: Vec3,

	/// The chance it holds, known from the moment it strikes the husk.
	pub chance: Option<f64>,

	/// The pulse it broke on, once it has broken.
	pub broke_free_on: Option<i64>,
}

/// The fixed numbers and the arithmetic of a capture.
///
/// These are playtest starting points, written down in docs/gameplay.md.
pub struct CaptureRules;

impl CaptureRules {
	/// How far off the husk's body, in degrees, an orb may be thrown and
	/// still strike it. The difficulty widens or narrows it.
	pub const AIM_DEGREES: f64 = 6.0;
	// The odds -----------------------------------------------------------

	/// The capture chance never goes above this.
	pub const CHANCE_CAP: f64 = 0.97;
	/// What an orb costs.
	pub const COST: f64 = 40.0;
	/// The husk has been drawn in.
	pub const DRAWN_TICK: i64 = 48;
	/// How long a husk that broke out stands stunned.
	pub const FAILURE_STUN_TICKS: i64 = 48;
	/// A husk that breaks out has this share of its ward back, but never
	/// more than [`WEAKENED_SHARE`](Self::WEAKENED_SHARE).
	pub const FAILURE_WARD: f64 = 0.1;
	pub const HOLD_SOUL: f64 = 10.0;
	// The timeline, in ticks from the release ----------------------------

	/// It strikes the husk.
	pub const IMPACT_TICK: i64 = 27;
	/// It has dropped onto the surface and lies there.
	pub const LANDED_TICK: i64 = 66;
	/// Soul energy a parry adds, and an unflinching hold.
	pub const PARRY_SOUL: f64 = 10.0;
	pub const PITY_CAP: f64 = 0.24;
	/// Added to the chance for every orb this husk has broken out of, up to
	/// [`PITY_CAP`](Self::PITY_CAP).
	pub const PITY_PER_FAILURE: f64 = 0.08;
	/// The three pulses.
	pub const PULSE_TICKS: [i64; 3] = [66, 114, 168];
	/// The result, after the held breath.
	pub const RESULT_TICK: i64 = 222;
	// The throw ----------------------------------------------------------

	/// How long the ring takes to shrink around the husk while ORB is held.
	/// Then it starts again.
	pub const RING_TICKS: i64 = 72;
	/// A hold shorter than this is no hold: the throw is not graded.
	pub const SHORTEST_HOLD_TICKS: i64 = 6;
	// Soul energy --------------------------------------------------------

	/// The most soul energy a reaper holds.
	pub const SOUL_MAX: f64 = 100.0;
	/// Soul energy a hit adds, for removing the husk's whole ward: a hit
	/// adds this times the share of the ward it removed. Breaking a ward
	/// always pays for at least two orbs.
	pub const SOUL_PER_WARD: f64 = 120.0;
	/// An orb can be thrown once the ward is at or under this share of what
	/// it started with.
	pub const WEAKENED_SHARE: f64 = 0.6;

	/// How a husk's rarity sets its odds.
	pub const fn rarity_factor(rarity: Rarity) -> f64 {
		match rarity {
			Rarity::Common => 0.8,
			Rarity::Uncommon => 0.6,
			Rarity::Rare => 0.4,
			Rarity::Epic => 0.25,
		}
	}

	/// How a class's capture rating, from 1 to 5, sets its odds.
	pub const fn capture_factor(capture: i64) -> f64 {
		0.85 + 0.05 * capture as f64
	}

	/// The grade of a throw held for `held_ticks`, with the crosshair
	/// `on_husk` or not when it was let go.
	#[must_use]
	pub fn grade_for(held_ticks: i64, on_husk: bool) -> CaptureGrade {
		if !on_husk || held_ticks < Self::SHORTEST_HOLD_TICKS {
			return CaptureGrade::Plain;
		}

		let ring = Self::ring_progress(held_ticks);

		if ring >= 2.0 / 3.0 {
			CaptureGrade::Excellent
		} else if ring >= 1.0 / 3.0 {
			CaptureGrade::Great
		} else {
			CaptureGrade::Plain
		}
	}

	/// How far the ring has shrunk after `held_ticks` of holding, from 0
	/// (the outer edge) toward 1 (the husk). It starts again every
	/// [`RING_TICKS`](Self::RING_TICKS).
	pub fn ring_progress(held_ticks: i64) -> f64 {
		(held_ticks as f64 % Self::RING_TICKS as f64) / Self::RING_TICKS as f64
	}

	/// The chance an orb holds a husk of rarity factor `rarity` with
	/// `ward_share` of its ward left, thrown at `grade` by a reaper whose
	/// class rates `capture` for capture, after `failures` orbs it broke
	/// out of.
	pub fn chance(
		rarity: f64,
		ward_share: f64,
		grade: CaptureGrade,
		capture: i64,
		failures: i64,
	) -> f64 {
		let pity = (failures as f64 * Self::PITY_PER_FAILURE).min(Self::PITY_CAP);
		let share = math::clamp(ward_share, 0.0, 1.0);

		math::min(
			Self::CHANCE_CAP,
			rarity * (3.0 - 2.0 * share) / 3.0 * grade.chance() * Self::capture_factor(capture)
				+ pity,
		)
	}

	/// The roll for the `attempt`th orb that struck the husk of the hunt
	/// `seed`, counting from 0. The same hunt always rolls the same.
	pub fn roll(seed: i64, attempt: i64) -> f64 {
		SeededRandom::new(mix_seed(&[seed, 0x0B5, attempt])).next_double()
	}

	/// How many times an orb has pulsed `ticks` after ORB was let go, from 0
	/// to 3: none after the pulse it broke out on, `broke_on`, once it has.
	pub fn pulses_at(ticks: i64, broke_on: Option<i64>) -> i64 {
		let pulses = Self::PULSE_TICKS
			.iter()
			.filter(|&&pulse| ticks >= pulse)
			.count() as i64;

		match broke_on {
			Some(broke) => pulses.min(broke - 1),
			None => pulses,
		}
	}

	/// The pulse an orb with `chance` breaks on, given its `roll`, or `None`
	/// when it holds. The nearer the miss, the later it breaks: the pulses
	/// tell the truth about how close it was.
	pub fn break_pulse(roll: f64, chance: f64) -> Option<i64> {
		if roll < chance {
			return None;
		}

		let miss = (roll - chance) / (1.0 - chance);

		if miss < 1.0 / 3.0 {
			Some(3)
		} else if miss < 2.0 / 3.0 {
			Some(2)
		} else {
			Some(1)
		}
	}
}
