//! The body a husk moves in, and what it can jump.

use crate::vessel::Vessel;

/// The numbers the room navigation and the locomotion states read.
///
/// The heights and reach are the creature table's: a walker is cat-to-
/// small-dog sized, so furniture is its terrain rather than an obstacle it
/// goes round. The paces are not repeated here: [`Vessel::locomotion`]
/// fills them from the vessel's own `walk_speed` and `bolt_speed`, so a
/// creature's pace is written down exactly once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocomotionProfile {
	/// The full height of the body, in metres.
	pub body_height: f64,

	/// The radius of the patch a foot stands on, in metres. A standable
	/// cell keeps this clear of every edge.
	pub foot_radius: f64,

	/// The highest step it will jump up onto, in metres.
	pub jumps_up: f64,

	/// The deepest drop it will jump down, in metres.
	pub drops_down: f64,

	/// The farthest horizontal gap it will leap across, in metres.
	pub leaps_across: f64,

	/// Its unhurried pace, in metres a second.
	pub walk_speed: f64,

	/// Its fleeing pace, in metres a second.
	pub run_speed: f64,

	/// For a husk that flies, the lowest its body centre flies, in metres
	/// above the player's eye. Zero for a walker.
	pub air_low_metres: f64,

	/// The highest its body centre flies. Zero for a walker.
	pub air_high_metres: f64,
}

impl LocomotionProfile {
	/// Half the body's height, in metres. The ballistic leap keeps this
	/// much air below the body's centre.
	pub const fn half_height(&self) -> f64 {
		self.body_height / 2.0
	}

	/// Whether this creature keeps to the air rather than the floor.
	pub const fn flies(&self) -> bool {
		self.air_high_metres > self.air_low_metres
	}
}

impl Vessel {
	/// How this body moves across the room, and what it can jump.
	///
	/// The hunt's own numbers on [`Vessel`] are unchanged: the room moves
	/// the smaller, nimbler bodies of the creature table, and the hunt
	/// adopts them when its combat does.
	pub const fn locomotion(self) -> LocomotionProfile {
		let (body_height, foot_radius, jumps_up, drops_down, leaps_across) = match self {
			Self::Ape => (0.60, 0.18, 1.00, 1.50, 1.2),
			Self::Bird => (0.34, 0.12, 0.0, 0.0, 0.0),
			Self::Penguin => (0.48, 0.14, 0.50, 0.90, 0.6),
			Self::Llama => (0.80, 0.20, 0.90, 1.40, 1.1),
			Self::Robot => (0.62, 0.17, 0.80, 1.30, 0.9),
			// The stack never moves. Its body is the tower it stands as.
			Self::ReliquaryStack => (10.0, 1.15, 0.0, 0.0, 0.0),
		};
		let spec = self.spec();

		LocomotionProfile {
			body_height,
			foot_radius,
			jumps_up,
			drops_down,
			leaps_across,
			walk_speed: spec.walk_speed,
			run_speed: spec.bolt_speed,
			air_low_metres: spec.low_metres,
			air_high_metres: spec.high_metres,
		}
	}
}
