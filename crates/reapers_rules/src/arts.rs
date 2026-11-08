//! A reaper's second way to fight: something it throws.

use core::fmt;

use crate::hunt::hunt_stats::HuntRules;
use crate::named::named_enum;
use crate::vec3::radians;
use crate::vessel::Vessel;

/// A reaper's second way to fight: something it throws.
///
/// A reaper learns an art by absorbing the creatures that carry it. The
/// first husk of one teaches it, and more of them deepen it. Nothing about
/// an art is bought: what a reaper can throw is a record of what it has
/// hunted.
///
/// An art is thrown, not tapped. The reaper holds a thumb on it to gather
/// it while keeping the phone steady on where it should go, then flicks the
/// phone to let it fly. Each one answers the room differently: fire is
/// lobbed over whatever a husk hides behind, ice flies straight and breaks
/// on it, and wind blows a husk out of cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Art {
	/// Fire. A lobbed ember that bursts where it lands and sets a husk
	/// burning. It arcs over cover, and a burning husk runs into the open.
	Cinder,

	/// Ice. A shard that flies straight at the husk and freezes it where it
	/// stands, every seal in view open. The next blow shatters the ice.
	Rime,

	/// Wind. A wide gust that knocks a husk off whatever it stands on or out
	/// from behind whatever it hides behind, and stuns it there.
	Gale,
}

named_enum!(Art {
	Cinder => "cinder",
	Rime => "rime",
	Gale => "gale",
});

impl Art {
	/// The creatures that carry this art. Absorbing one teaches it.
	pub const fn vessels(self) -> &'static [Vessel] {
		match self {
			Self::Cinder => &[Vessel::Ape, Vessel::Robot],
			Self::Rime => &[Vessel::Penguin, Vessel::Llama],
			Self::Gale => &[Vessel::Bird],
		}
	}

	/// Whether `vessel` carries this art.
	pub fn carried_by_vessel(self, vessel: Vessel) -> bool {
		self.vessels().contains(&vessel)
	}

	/// How fast it flies, in metres a second.
	pub const fn speed(self) -> f64 {
		match self {
			Self::Cinder => 6.0,
			Self::Rime => 14.0,
			Self::Gale => 12.0,
		}
	}

	/// Whether, thrown at a husk, it follows the husk in flight. Fire does
	/// not: it is lobbed at a place, and bursts there.
	pub const fn homing(self) -> bool {
		!matches!(self, Self::Cinder)
	}

	/// Ward removed when it lands on the husk, in strikes of the reaper's
	/// own weapon, at each rank.
	pub const fn strike(self) -> [f64; 3] {
		match self {
			Self::Cinder => [2.0, 2.5, 3.0],
			Self::Rime => [1.5, 1.5, 1.5],
			Self::Gale => [0.8, 1.0, 1.2],
		}
	}

	/// The art a husk on `vessel` carries, or `None` for one that carries
	/// none.
	pub fn carried_by(vessel: Vessel) -> Option<Self> {
		Self::ALL
			.into_iter()
			.find(|art| art.carried_by_vessel(vessel))
	}

	/// How deep this art runs in a reaper that has absorbed `absorbed` husks
	/// carrying it: 0 when it has absorbed none, up to
	/// [`ArtRules::MAX_RANK`].
	pub fn rank_for(absorbed: usize) -> i64 {
		ArtRules::ABSORBED_FOR_RANK
			.iter()
			.filter(|&&needed| absorbed >= needed)
			.count() as i64
	}
}

/// One art as a reaper knows it: which, and how deep.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArtSkill {
	pub art: Art,

	/// From 1, when one husk carrying it has been absorbed, to
	/// [`ArtRules::MAX_RANK`].
	pub rank: i64,
}

impl ArtSkill {
	/// The art at `rank`, or `None` for a rank no reaper can hold.
	pub const fn new(art: Art, rank: i64) -> Option<Self> {
		if rank >= 1 && rank <= ArtRules::MAX_RANK {
			Some(Self { art, rank })
		} else {
			None
		}
	}

	const fn step(&self) -> i64 {
		self.rank - 1
	}

	/// Ward removed when it lands, in strikes.
	pub const fn strike(&self) -> f64 {
		self.art.strike()[self.step() as usize]
	}

	/// The most throws a reaper can have ready at once.
	pub const fn max_charges(&self) -> i64 {
		if self.rank >= ArtRules::MAX_RANK {
			3
		} else {
			2
		}
	}

	/// Ticks until another throw is ready.
	pub const fn recharge_ticks(&self) -> i64 {
		ArtRules::RECHARGE_TICKS - ArtRules::RECHARGE_TICKS_PER_RANK * self.step()
	}

	/// How long a frozen husk stays frozen.
	pub const fn freeze_ticks(&self) -> i64 {
		ArtRules::FREEZE_TICKS + ArtRules::FREEZE_TICKS_PER_RANK * self.step()
	}

	/// How much harder the blow that shatters a frozen husk lands.
	pub const fn shatter(&self) -> f64 {
		ArtRules::SHATTER + ArtRules::SHATTER_PER_RANK * self.step() as f64
	}

	/// How long a burning husk burns.
	pub const fn burn_ticks(&self) -> i64 {
		ArtRules::BURN_TICKS
	}

	/// The ward the burning takes in all, in strikes.
	pub const fn burn_strike(&self) -> f64 {
		ArtRules::BURN_STRIKE + ArtRules::BURN_STRIKE_PER_RANK * self.step() as f64
	}

	/// How long a husk the wind caught stands stunned.
	pub const fn gust_stun_ticks(&self) -> i64 {
		ArtRules::GUST_STUN_TICKS + ArtRules::GUST_STUN_TICKS_PER_RANK * self.step()
	}
}

impl fmt::Display for ArtSkill {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "ArtSkill({}, {})", self.art.name(), self.rank)
	}
}

/// The fixed numbers of the arts. Playtest starting points, written down in
/// docs/gameplay.md.
pub struct ArtRules;

impl ArtRules {
	/// Husks carrying an art a reaper must have absorbed for each rank.
	pub const ABSORBED_FOR_RANK: [usize; 3] = [1, 4, 10];
	/// A burning husk bolts this soon, and never into cover.
	pub const BURN_BOLT_TICKS: i64 = 20;
	/// The ward its burning takes in all, in strikes, at the first rank and
	/// for each rank after.
	pub const BURN_STRIKE: f64 = 1.5;
	pub const BURN_STRIKE_PER_RANK: f64 = 0.5;
	/// How long a husk burns.
	pub const BURN_TICKS: i64 = 150;
	// Cinder ------------------------------------------------------------

	/// How far from where it bursts an ember catches a husk, in metres
	/// beyond the husk's own half width.
	pub const BURST_METRES: f64 = 0.8;
	/// The farthest an art thrown at nothing flies, in metres.
	pub const FARTHEST_METRES: f64 = 8.0;
	// Rime --------------------------------------------------------------

	/// How long a frozen husk stays frozen, and how much longer each rank.
	pub const FREEZE_TICKS: i64 = 150;
	pub const FREEZE_TICKS_PER_RANK: i64 = 40;
	/// How long a thumb must rest on the art before a flick throws it.
	pub const GATHER_TICKS: i64 = 24;
	// Gale --------------------------------------------------------------

	/// How far off the husk the gust may be aimed and still catch it.
	pub const GUST_ANGLE: f64 = radians(22.0);
	/// How long a husk the gust caught stands stunned, and how much longer
	/// each rank.
	pub const GUST_STUN_TICKS: i64 = 90;
	pub const GUST_STUN_TICKS_PER_RANK: i64 = 20;
	/// Where an art leaves the reaper's hand, in metres from the eye: low
	/// and a little ahead.
	pub const HAND_DROP: f64 = 0.25;
	/// How high the ember's lob rises above the straight line, in metres,
	/// and how much more for every metre it is thrown.
	pub const LOB_METRES: f64 = 0.5;
	pub const LOB_PER_METRE: f64 = 0.18;
	pub const MAX_RANK: i64 = 3;
	/// Ticks for another throw to be ready, at the first rank, and how much
	/// sooner at each rank after.
	pub const RECHARGE_TICKS: i64 = 9 * HuntRules::TICK_RATE;
	pub const RECHARGE_TICKS_PER_RANK: i64 = HuntRules::TICK_RATE;
	/// How long a husk the gust caught keeps away from cover after it comes
	/// round.
	pub const SHAKEN_TICKS: i64 = 4 * HuntRules::TICK_RATE;
	/// The blow that shatters the ice lands this much harder, and more for
	/// each rank.
	pub const SHATTER: f64 = 2.0;
	pub const SHATTER_PER_RANK: f64 = 0.25;
	/// A thrown art lands at once on anything nearer than this many ticks.
	pub const SHORTEST_FLIGHT_TICKS: i64 = 8;
	/// Throws ready when a fight begins.
	pub const STARTING_CHARGES: i64 = 1;
	/// Turning faster than this, in radians a second, with the art gathered,
	/// throws it. A flick of the wrist, slower than the swing that turns a
	/// charge aside, so a hard flick does both.
	pub const THROW_SPEED: f64 = 1.5;
}
