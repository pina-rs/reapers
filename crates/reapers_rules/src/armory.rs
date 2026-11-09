//! What a reaper can buy: weapons at their tempers, and outfits.

use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::FormatError;
use crate::named::named_enum;
use crate::reaper_class::ReaperClass;
use crate::reaper_class::WeaponKind;

/// How worked a weapon is. Each temper is the same weapon after more has
/// happened to it, never a different weapon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Temper {
	/// As issued: wood, rope and plain iron.
	Shrouded,

	/// Dyed dark, shod in iron and carved with marks.
	Sworn,

	/// Charred and pitted, with the edge burning through.
	Scorched,

	/// Barely there: an outline around a lit edge.
	Hollowed,
}

named_enum!(Temper {
	Shrouded => "shrouded",
	Sworn => "sworn",
	Scorched => "scorched",
	Hollowed => "hollowed",
});

impl Temper {
	/// Multiplier on every strike's ward damage.
	pub const fn strike(self) -> f64 {
		match self {
			Self::Shrouded => 1.0,
			Self::Sworn => 1.3,
			Self::Scorched => 1.7,
			Self::Hollowed => 2.2,
		}
	}

	/// The reaper level needed to buy this temper.
	pub const fn level_required(self) -> i64 {
		match self {
			Self::Shrouded => 1,
			Self::Sworn => 4,
			Self::Scorched => 9,
			Self::Hollowed => 15,
		}
	}

	/// Cost in anima. The shrouded temper of a class's own weapon is free.
	pub const fn price(self) -> i64 {
		match self {
			Self::Shrouded => 0,
			Self::Sworn => 220,
			Self::Scorched => 640,
			Self::Hollowed => 1500,
		}
	}

	/// The best temper a reaper of `level` may hold.
	pub fn best_for(level: i64) -> Self {
		Self::ALL
			.into_iter()
			.rev()
			.find(|temper| temper.level_required() <= level)
			.unwrap_or(Self::Shrouded)
	}
}

/// One buyable weapon: a `kind` at a `temper`. On the wire it is its
/// [`id`](Self::id), such as `scythe.sworn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Weapon {
	pub kind: WeaponKind,
	pub temper: Temper,
}

impl Weapon {
	/// What another class pays to pick up this kind for the first time. A
	/// class's own kind costs nothing at the shrouded temper.
	pub const CROSS_CLASS_PRICE: i64 = 120;
	pub const HEAVY_LEVEL: i64 = 3;
	/// What anyone pays for the first of a heavy weapon, and the level they
	/// must be to lift it.
	pub const HEAVY_PRICE: i64 = 320;

	pub const fn new(kind: WeaponKind, temper: Temper) -> Self {
		Self { kind, temper }
	}

	/// Reads a weapon back from its `id`, such as `scythe.sworn`.
	pub fn parse(id: &str) -> Result<Self, FormatError> {
		let parts: Vec<&str> = id.split('.').collect();

		let [kind, temper] = parts[..] else {
			return Err(FormatError::with_input("Not a weapon id", id));
		};

		Ok(Self::new(
			WeaponKind::from_name(kind)
				.ok_or_else(|| FormatError::with_input("No such weapon", id))?,
			Temper::from_name(temper)
				.ok_or_else(|| FormatError::with_input("No such temper", id))?,
		))
	}

	pub fn id(&self) -> String {
		format!("{}.{}", self.kind.name(), self.temper.name())
	}

	pub const fn profile(&self) -> WeaponProfile {
		weapon_profile(self.kind)
	}

	/// The reaper level needed to buy this weapon.
	pub const fn level_required(&self) -> i64 {
		if self.kind.is_heavy() && matches!(self.temper, Temper::Shrouded) {
			Self::HEAVY_LEVEL
		} else {
			self.temper.level_required()
		}
	}

	/// The price in anima for a reaper of `reaper_class`.
	pub fn price_for(&self, reaper_class: ReaperClass) -> i64 {
		if self.temper != Temper::Shrouded {
			return self.temper.price();
		}

		if self.kind.is_heavy() {
			return Self::HEAVY_PRICE;
		}

		if reaper_class.weapon() == self.kind {
			0
		} else {
			Self::CROSS_CLASS_PRICE
		}
	}
}

impl fmt::Display for Weapon {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "Weapon({})", self.id())
	}
}

impl Serialize for Weapon {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.id())
	}
}

impl<'de> Deserialize<'de> for Weapon {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let id = String::deserialize(deserializer)?;

		Self::parse(&id).map_err(serde::de::Error::custom)
	}
}

/// How one kind of weapon handles, whatever its temper. Swung as a reaper's
/// slot-three heavy weapon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponProfile {
	/// How many hits a swing does.
	pub strike: f64,

	/// Added to the seal cone, in degrees of view, and to how far off the
	/// husk's body a swing may be and still reach it.
	pub reach_degrees: f64,

	/// Ticks a swing takes.
	pub swing_ticks: i64,

	/// How long a swing that lands stops the husk in its tracks, when that
	/// is longer than every heavy swing does. Only the greatsword's is.
	pub knock_ticks: i64,

	/// The share of a hit on an open seal that a swing on the body does. An
	/// axe bites wherever it lands.
	pub body_share: f64,
}

impl WeaponProfile {
	/// The share of a hit on an open seal that a hit on the body does, for
	/// every weapon but one that bites anywhere.
	pub const PLAIN_BODY_SHARE: f64 = 0.5;
}

const fn profile(strike: f64, reach_degrees: f64, swing_ticks: i64) -> WeaponProfile {
	WeaponProfile {
		strike,
		reach_degrees,
		swing_ticks,
		knock_ticks: 0,
		body_share: WeaponProfile::PLAIN_BODY_SHARE,
	}
}

/// The handling of each weapon kind.
///
/// Damage per second is kept close across kinds: a slow weapon hits harder,
/// a fast one reaches less. What differs is the rhythm a hunt is played at.
/// The two heavy weapons do a little more, and pay for it: the greatsword
/// swings slowest of all, and the axe has almost no reach, so it lands on
/// the body as often as on a seal.
pub const fn weapon_profile(kind: WeaponKind) -> WeaponProfile {
	match kind {
		WeaponKind::Scythe => profile(1.15, 1.6, 26),
		WeaponKind::LanternPole => profile(1.0, 0.8, 22),
		WeaponKind::BalanceFlail => profile(1.0, 0.4, 24),
		WeaponKind::Spear => profile(1.45, 0.0, 32),
		WeaponKind::ChainWhip => profile(0.8, 0.6, 16),
		WeaponKind::Bell => profile(1.3, 2.4, 28),
		WeaponKind::Greatsword => {
			WeaponProfile {
				knock_ticks: 28,
				..profile(1.85, 1.2, 40)
			}
		}
		WeaponKind::Axe => {
			WeaponProfile {
				body_share: 0.7,
				..profile(1.5, 0.2, 32)
			}
		}
	}
}

/// Every weapon the armory sells: each kind at each temper.
pub fn armory_weapons() -> Vec<Weapon> {
	WeaponKind::ALL
		.into_iter()
		.flat_map(|kind| {
			Temper::ALL
				.into_iter()
				.map(move |temper| Weapon::new(kind, temper))
		})
		.collect()
}

/// The four states of a reaper's outfit.
///
/// "Darker" means the same outfit after something happened to it: the cloth
/// is dyed, then charred, then partly gone, while the reaper's one light
/// burns brighter. An outfit never changes how a hunt plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Outfit {
	/// Undyed cloth, rope and wood.
	Shrouded,

	/// Dyed dark, with iron fittings and carved marks.
	Sworn,

	/// Charred cloth and pitted metal.
	Scorched,

	/// Parts of the figure are absent; the light is at its brightest.
	Hollowed,
}

named_enum!(Outfit {
	Shrouded => "shrouded",
	Sworn => "sworn",
	Scorched => "scorched",
	Hollowed => "hollowed",
});

impl Outfit {
	/// The reaper level needed to buy this outfit.
	pub const fn level_required(self) -> i64 {
		match self {
			Self::Shrouded => 1,
			Self::Sworn => 5,
			Self::Scorched => 12,
			Self::Hollowed => 20,
		}
	}

	/// Cost in anima.
	pub const fn price(self) -> i64 {
		match self {
			Self::Shrouded => 0,
			Self::Sworn => 300,
			Self::Scorched => 900,
			Self::Hollowed => 2400,
		}
	}
}
