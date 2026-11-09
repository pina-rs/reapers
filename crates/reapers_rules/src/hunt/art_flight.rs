//! A thrown art on its way.

use crate::arts::Art;
use crate::arts::ArtRules;
use crate::math;
use crate::vec3::Vec3;

/// A thrown art on its way.
///
/// The rules decide when it lands and what it does there. This is also all
/// the view needs to draw it in flight: where it left the hand, where it is
/// going, and how far along it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArtFlight {
	pub art: Art,

	/// Where it left the reaper's hand, in metres from the hunt origin.
	pub from: Vec3,

	/// Where it was thrown, in metres from the hunt origin. A homing art goes to
	/// wherever the husk is instead.
	pub target: Vec3,

	/// Whether it follows the husk.
	pub homing: bool,

	/// The tick it was thrown on.
	pub thrown_at: i64,

	/// How many ticks it is in the air.
	pub ticks: i64,
}

impl ArtFlight {
	/// The tick it lands on.
	pub const fn lands_at(&self) -> i64 {
		self.thrown_at + self.ticks
	}

	/// How far along its flight it is at `tick`, from 0 to 1.
	pub fn progress(&self, tick: i64) -> f64 {
		math::clamp((tick - self.thrown_at) as f64 / self.ticks as f64, 0.0, 1.0)
	}

	/// Where it is at `tick`, going to `goal`: the husk it follows, or the
	/// place it was thrown. Fire is lobbed, so it rises and falls; the
	/// others fly straight.
	pub fn position(&self, tick: i64, goal: Vec3) -> Vec3 {
		let along = self.progress(tick);
		let straight = self.from + (goal - self.from) * along;

		if self.art != Art::Cinder {
			return straight;
		}

		let lob = ArtRules::LOB_METRES + ArtRules::LOB_PER_METRE * (goal - self.from).length();

		straight + Vec3::UP * (lob * 4.0 * along * (1.0 - along))
	}
}
