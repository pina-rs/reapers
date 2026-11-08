//! The bodies dead NFTs wear in the spirit realm.

use crate::named::named_enum;
use crate::vec3::Vec3;

/// A place on a husk that a strike can land on.
///
/// The position is in the husk's own frame, in metres from the middle of its
/// body: `x` toward its left side (the right, as someone it is facing sees
/// it), `y` up, and `z` forward, out of its chest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SealSpec {
	pub offset: Vec3,
}

const fn seal(x: f64, y: f64, z: f64) -> SealSpec {
	SealSpec {
		offset: Vec3::new(x, y, z),
	}
}

/// How a husk gets about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gait {
	/// It walks on the floor, and runs across it when it bolts.
	Ground,

	/// It holds the air: it drifts, and darts when it bolts.
	Air,

	/// It stays where it is.
	Still,
}

named_enum!(Gait {
	Ground => "ground",
	Air => "air",
	Still => "still",
});

/// The body a dead NFT wears in the spirit realm.
///
/// Every supported collection has a creature of its own: an ape for a
/// collection of apes, a whale for a collection of whales. The creature is
/// an original model; an NFT's own colours and traits dress it. The numbers
/// here are the single source for the rules and for the models: the art
/// build reads them when it shapes each body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Vessel {
	/// A heavy ape that walks on its knuckles and charges off when startled.
	Ape,

	/// A hunched bird that hangs in the air at about head height.
	Bird,

	/// An upright penguin that waddles, and is easy to keep up with.
	Penguin,

	/// A long-necked llama that trots, and gallops when it bolts.
	Llama,

	/// A robot put together from parts that do not match, which walks on two
	/// legs and is in no hurry.
	Robot,

	/// A tower of image cubes held together by light. Too much for one
	/// reaper: it only appears in a rift, and its seals are the seams
	/// between cubes.
	ReliquaryStack,
}

named_enum!(Vessel {
	Ape => "ape",
	Bird => "bird",
	Penguin => "penguin",
	Llama => "llama",
	Robot => "robot",
	ReliquaryStack => "reliquaryStack",
});

/// The shape and habits of one vessel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VesselSpec {
	pub gait: Gait,

	/// Half the body's width, in metres, taken as the ellipse a strike can
	/// land in.
	pub half_width: f64,

	/// Half the body's height, in metres.
	pub half_height: f64,

	/// The nearest the husk comes to the player, measured along the floor.
	pub near_metres: f64,

	/// The farthest it goes.
	pub far_metres: f64,

	/// The lowest the middle of an airborne body flies, in metres above the
	/// player's eye. A body on the ground ignores this: it stands on the
	/// floor.
	pub low_metres: f64,

	/// The highest it flies.
	pub high_metres: f64,

	/// How fast it ambles when nothing has alarmed it, in metres a second.
	pub walk_speed: f64,

	/// How fast it flees.
	pub bolt_speed: f64,

	/// The highest ledge it will jump up onto, in metres above the floor: an
	/// ape takes a kitchen counter in one bound, a penguin only a low step.
	/// For a flier, the highest it will come down to perch.
	pub leap_metres: f64,

	pub seals: &'static [SealSpec],
}

const APE: VesselSpec = VesselSpec {
	gait: Gait::Ground,
	half_width: 0.55,
	half_height: 0.55,
	near_metres: 2.6,
	far_metres: 4.6,
	low_metres: 0.0,
	high_metres: 0.0,
	walk_speed: 0.7,
	bolt_speed: 2.6,
	leap_metres: 1.2,
	seals: &[
		seal(0.0, 0.38, 0.30),
		seal(0.0, 0.02, 0.36),
		seal(0.36, 0.30, 0.05),
		seal(-0.36, 0.30, 0.05),
		seal(0.0, 0.36, -0.28),
	],
};

const BIRD: VesselSpec = VesselSpec {
	gait: Gait::Air,
	half_width: 0.45,
	half_height: 0.28,
	near_metres: 2.2,
	far_metres: 3.6,
	low_metres: -0.15,
	high_metres: 0.85,
	walk_speed: 0.5,
	bolt_speed: 3.2,
	leap_metres: 2.2,
	seals: &[
		seal(0.0, 0.22, 0.08),
		seal(0.0, -0.04, 0.2),
		seal(0.2, 0.06, 0.0),
		seal(-0.2, 0.06, 0.0),
		seal(0.0, 0.1, -0.2),
	],
};

const PENGUIN: VesselSpec = VesselSpec {
	gait: Gait::Ground,
	half_width: 0.32,
	half_height: 0.45,
	near_metres: 2.2,
	far_metres: 4.2,
	low_metres: 0.0,
	high_metres: 0.0,
	walk_speed: 0.45,
	bolt_speed: 1.8,
	leap_metres: 0.55,
	seals: &[
		seal(0.0, 0.3, 0.2),
		seal(0.0, -0.04, 0.27),
		seal(0.28, 0.08, 0.02),
		seal(-0.28, 0.08, 0.02),
		seal(0.0, 0.1, -0.25),
	],
};

const LLAMA: VesselSpec = VesselSpec {
	gait: Gait::Ground,
	half_width: 0.5,
	half_height: 0.72,
	near_metres: 2.8,
	far_metres: 4.8,
	low_metres: 0.0,
	high_metres: 0.0,
	walk_speed: 0.6,
	bolt_speed: 2.8,
	leap_metres: 1.0,
	seals: &[
		seal(0.0, 0.6, 0.3),
		seal(0.0, 0.22, 0.32),
		seal(0.22, -0.06, -0.08),
		seal(-0.22, -0.06, -0.08),
		seal(0.0, 0.1, -0.34),
	],
};

const ROBOT: VesselSpec = VesselSpec {
	gait: Gait::Ground,
	half_width: 0.36,
	half_height: 0.56,
	near_metres: 2.4,
	far_metres: 4.4,
	low_metres: 0.0,
	high_metres: 0.0,
	walk_speed: 0.5,
	bolt_speed: 2.0,
	leap_metres: 0.8,
	seals: &[
		seal(0.0, 0.42, 0.17),
		seal(0.0, 0.08, 0.2),
		seal(0.3, 0.2, 0.0),
		seal(-0.3, 0.2, 0.0),
		seal(0.0, 0.1, -0.2),
	],
};

const RELIQUARY_STACK: VesselSpec = VesselSpec {
	gait: Gait::Still,
	half_width: 1.15,
	half_height: 5.0,
	near_metres: 17.0,
	far_metres: 17.0,
	low_metres: 1.9,
	high_metres: 1.9,
	walk_speed: 0.0,
	bolt_speed: 0.0,
	leap_metres: 0.0,
	seals: &[
		seal(0.0, -3.0, 1.05),
		seal(0.0, -1.0, 1.05),
		seal(0.0, 1.0, 1.05),
		seal(0.0, 3.0, 1.05),
	],
};

impl Vessel {
	/// Everything about this vessel's body and habits.
	pub const fn spec(self) -> &'static VesselSpec {
		match self {
			Self::Ape => &APE,
			Self::Bird => &BIRD,
			Self::Penguin => &PENGUIN,
			Self::Llama => &LLAMA,
			Self::Robot => &ROBOT,
			Self::ReliquaryStack => &RELIQUARY_STACK,
		}
	}

	pub const fn gait(self) -> Gait {
		self.spec().gait
	}

	pub const fn half_width(self) -> f64 {
		self.spec().half_width
	}

	pub const fn half_height(self) -> f64 {
		self.spec().half_height
	}

	pub const fn near_metres(self) -> f64 {
		self.spec().near_metres
	}

	pub const fn far_metres(self) -> f64 {
		self.spec().far_metres
	}

	pub const fn low_metres(self) -> f64 {
		self.spec().low_metres
	}

	pub const fn high_metres(self) -> f64 {
		self.spec().high_metres
	}

	pub const fn walk_speed(self) -> f64 {
		self.spec().walk_speed
	}

	pub const fn bolt_speed(self) -> f64 {
		self.spec().bolt_speed
	}

	pub const fn leap_metres(self) -> f64 {
		self.spec().leap_metres
	}

	pub const fn seals(self) -> &'static [SealSpec] {
		self.spec().seals
	}

	/// Whether a single reaper may hunt this vessel.
	pub const fn is_solo(self) -> bool {
		!matches!(self, Self::ReliquaryStack)
	}
}
