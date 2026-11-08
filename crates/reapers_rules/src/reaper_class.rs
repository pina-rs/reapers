//! The six kinds of reaper, what they are rated on, and the weapon shapes.

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::FormatError;
use crate::named::named_enum;

/// The four numbers a reaper is rated on, each from 1 to 5.
///
/// Every class totals [`ReaperStats::CLASS_TOTAL`], so no shuffle result is
/// simply better than another: a class is a lean, never a lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReaperStats {
	/// Ward removed by one landed strike.
	pub strike: i64,

	/// How far from a seal a tap may land and still count.
	pub reach: i64,

	/// How long a broken husk stays open to the reap.
	pub capture: i64,

	/// How many flinches a reaper survives before the husk slips away.
	pub resilience: i64,
}

impl ReaperStats {
	/// What every class's four ratings add up to.
	pub const CLASS_TOTAL: i64 = 13;

	pub const fn total(self) -> i64 {
		self.strike + self.reach + self.capture + self.resilience
	}
}

/// The one rule each class bends during a hunt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassPerk {
	/// A strike also lands on every other open seal near the tap.
	Cleave,

	/// The first flinch of a hunt costs no resolve.
	Steadfast,

	/// Seals are numbered, and striking them in order doubles the strike.
	Measure,

	/// The strike after an unflinching hold lands three times as hard.
	ChosenBlow,

	/// Cadence builds to double strength instead of one and a half.
	Chain,

	/// Each unflinching hold lengthens the reap window.
	Toll,
}

named_enum!(ClassPerk {
	Cleave => "cleave",
	Steadfast => "steadfast",
	Measure => "measure",
	ChosenBlow => "chosenBlow",
	Chain => "chain",
	Toll => "toll",
});

/// The six kinds of reaper the shuffle can bind a player to.
///
/// The names are job titles on purpose: the classes borrow roles and tools
/// from death figures across cultures, never a named figure's likeness. See
/// docs/research/dark-games-and-ar-hunting.md.
///
/// On the wire a class is its three-letter [`code`](Self::code).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReaperClass {
	/// The field hand of death. A wide hat over a tall, narrow body.
	Harvester,

	/// Carries the dead across and cannot be moved. A heavy cowl and a pole.
	Ferryman,

	/// Measures a spirit before taking it. A crested mask and a balance.
	Weigher,

	/// Picks who falls, then strikes once. A one-sided feather mantle.
	Chooser,

	/// The rider with no head. An empty collar and a lantern at the hip.
	Hollow,

	/// Announces a death before it arrives. A floor-length veil and a bell.
	Mourner,
}

impl ReaperClass {
	pub const ALL: [Self; 6] = [
		Self::Harvester,
		Self::Ferryman,
		Self::Weigher,
		Self::Chooser,
		Self::Hollow,
		Self::Mourner,
	];

	/// Its position in [`ReaperClass::ALL`].
	pub const fn index(self) -> usize {
		self as usize
	}

	/// Its name, as the Dart rules spelled it.
	pub const fn name(self) -> &'static str {
		match self {
			Self::Harvester => "harvester",
			Self::Ferryman => "ferryman",
			Self::Weigher => "weigher",
			Self::Chooser => "chooser",
			Self::Hollow => "hollow",
			Self::Mourner => "mourner",
		}
	}

	/// The three-letter tag used in a genome code.
	pub const fn code(self) -> &'static str {
		match self {
			Self::Harvester => "har",
			Self::Ferryman => "fer",
			Self::Weigher => "wei",
			Self::Chooser => "cho",
			Self::Hollow => "hol",
			Self::Mourner => "mou",
		}
	}

	pub const fn stats(self) -> ReaperStats {
		let (strike, reach, capture, resilience) = match self {
			Self::Harvester => (3, 5, 3, 2),
			Self::Ferryman => (3, 2, 3, 5),
			Self::Weigher => (4, 2, 4, 3),
			Self::Chooser => (5, 4, 2, 2),
			Self::Hollow => (4, 3, 2, 4),
			Self::Mourner => (1, 4, 5, 3),
		};

		ReaperStats {
			strike,
			reach,
			capture,
			resilience,
		}
	}

	pub const fn perk(self) -> ClassPerk {
		match self {
			Self::Harvester => ClassPerk::Cleave,
			Self::Ferryman => ClassPerk::Steadfast,
			Self::Weigher => ClassPerk::Measure,
			Self::Chooser => ClassPerk::ChosenBlow,
			Self::Hollow => ClassPerk::Chain,
			Self::Mourner => ClassPerk::Toll,
		}
	}

	/// The kind of weapon this class starts with.
	pub const fn weapon(self) -> WeaponKind {
		match self {
			Self::Harvester => WeaponKind::Scythe,
			Self::Ferryman => WeaponKind::LanternPole,
			Self::Weigher => WeaponKind::BalanceFlail,
			Self::Chooser => WeaponKind::Spear,
			Self::Hollow => WeaponKind::ChainWhip,
			Self::Mourner => WeaponKind::Bell,
		}
	}

	/// The class whose [`code`](Self::code) is `code`.
	pub fn from_code(code: &str) -> Result<Self, FormatError> {
		Self::ALL
			.into_iter()
			.find(|class| class.code() == code)
			.ok_or_else(|| FormatError::new(format!("Unknown reaper class \"{code}\"")))
	}
}

impl Serialize for ReaperClass {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.code())
	}
}

impl<'de> Deserialize<'de> for ReaperClass {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let code = String::deserialize(deserializer)?;

		Self::from_code(&code).map_err(serde::de::Error::custom)
	}
}

/// The weapon shapes. Each class starts with one of the first six, and any
/// reaper may buy any of them. The greatsword and the axe are no class's
/// own: they are heavy, and every reaper comes to them from something else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum WeaponKind {
	Scythe,
	LanternPole,
	BalanceFlail,
	Spear,
	ChainWhip,
	Bell,
	Greatsword,
	Axe,
}

named_enum!(WeaponKind {
	Scythe => "scythe",
	LanternPole => "lanternPole",
	BalanceFlail => "balanceFlail",
	Spear => "spear",
	ChainWhip => "chainWhip",
	Bell => "bell",
	Greatsword => "greatsword",
	Axe => "axe",
});

impl WeaponKind {
	/// Whether this is one of the heavy weapons, which no class starts with.
	pub const fn is_heavy(self) -> bool {
		matches!(self, Self::Greatsword | Self::Axe)
	}
}
