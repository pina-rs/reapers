//! Where a husk is and which way it is turned.

use crate::math;
use crate::named::named_enum;
use crate::vec3::Vec3;
use crate::vec3::wrap_angle;
use crate::vessel::Vessel;

/// What a husk stands on, or behind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Footing {
	/// The open floor, or for a husk that flies, its band of air.
	#[default]
	Open,

	/// The level top of something in the room: a table, a counter.
	Ledge,

	/// Behind something in the room, with only what rises above it showing.
	Hidden,
}

named_enum!(Footing {
	Open => "open",
	Ledge => "ledge",
	Hidden => "hidden",
});

/// Where a husk is and which way it is turned at one instant.
///
/// The rules own this, and the view draws exactly what it says: a model is
/// placed at `position` and turned to `heading`. Nothing about a hit is
/// decided by what was drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuskPose {
	pub vessel: Vessel,

	/// The middle of the husk's body, in metres from the player's eye.
	pub position: Vec3,

	/// The way the husk's body faces, as a bearing in radians: 0 is the way
	/// the player first faced, and it grows to the right.
	pub heading: f64,

	/// What it stands on or behind. While `leaping`, what it is leaping to.
	pub footing: Footing,

	/// Whether it is in the air between one footing and the next.
	pub leaping: bool,

	/// How high the surface under it is, in metres from the player's eye:
	/// the floor, or the ledge it stands on. `None` for a husk in the air
	/// with nothing under it near enough to matter.
	pub ground: Option<f64>,
}

impl HuskPose {
	/// A seal is hidden only once it is well round the far side: more than
	/// about 105 degrees from the nearest point of the body. One on the very
	/// edge of the silhouette can still be seen, and struck.
	const FACING_SHARE: f64 = -0.25;

	/// A husk standing in the open at `position`, turned to `heading`.
	pub const fn new(vessel: Vessel, position: Vec3, heading: f64) -> Self {
		Self {
			vessel,
			position,
			heading,
			footing: Footing::Open,
			leaping: false,
			ground: None,
		}
	}

	/// The husk's bearing around the player, in radians.
	pub fn yaw(&self) -> f64 {
		self.position.yaw()
	}

	/// The husk's elevation above the horizon, in radians.
	pub fn pitch(&self) -> f64 {
		self.position.pitch()
	}

	/// How far away the middle of the husk is, in metres.
	pub fn distance(&self) -> f64 {
		self.position.length()
	}

	/// The direction from the player to the husk's centre.
	pub fn direction(&self) -> Vec3 {
		self.position.normalized()
	}

	/// The way the husk's chest points.
	pub fn forward(&self) -> Vec3 {
		Vec3::new(math::sin(self.heading), 0.0, math::cos(self.heading))
	}

	/// The way the husk's left side points.
	pub fn left(&self) -> Vec3 {
		Vec3::new(-math::cos(self.heading), 0.0, math::sin(self.heading))
	}

	/// Whether the husk is turned toward the player, give or take a quarter
	/// turn.
	pub fn faces_player(&self) -> bool {
		self.forward().dot(-self.direction()) > 0.0
	}

	/// Whether `ray`, a direction out from the player, passes through the
	/// husk's body, taken as an ellipse and widened by `pad` radians.
	///
	/// A hit that covers the body but misses every open seal is a body hit:
	/// half a hit on a seal.
	pub fn covers(&self, ray: Vec3, pad: f64) -> bool {
		let distance = self.distance();
		let across = wrap_angle(ray.yaw() - self.yaw()) * math::cos(self.pitch())
			/ (math::atan2(self.vessel.half_width(), distance) + pad);
		let along =
			(ray.pitch() - self.pitch()) / (math::atan2(self.vessel.half_height(), distance) + pad);

		across * across + along * along <= 1.0
	}

	/// Where seal `index` is, in metres from the player.
	pub fn seal_position(&self, index: usize) -> Vec3 {
		self.position + self.seal_offset(index)
	}

	/// The direction from the player to seal `index`.
	pub fn seal_direction(&self, index: usize) -> Vec3 {
		self.seal_position(index).normalized()
	}

	/// Whether seal `index` is on a side of the husk the player can see.
	///
	/// A husk hides the seals on its far side, so one that turns its back
	/// shows different seals from one that faces the reaper.
	pub fn seal_faces_player(&self, index: usize) -> bool {
		let outward = self.seal_offset(index);

		outward.dot(-self.direction()) >= Self::FACING_SHARE * outward.length()
	}

	/// Seal `index`'s place relative to the middle of the body, in the
	/// hunt's space.
	fn seal_offset(&self, index: usize) -> Vec3 {
		let offset = self
			.vessel
			.seals()
			.get(index)
			.map_or(Vec3::ZERO, |seal| seal.offset);

		self.left() * offset.x + Vec3::UP * offset.y + self.forward() * offset.z
	}
}
