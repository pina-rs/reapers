//! Where a husk may go: what the phone made of the room before the hunt.

use core::fmt;

use crate::hunt::hunt_stats::HuntRules;
use crate::math::PI;
use crate::math::{self};
use crate::vec3::Vec3;
use crate::vec3::degrees;
use crate::vec3::radians;
use crate::vec3::wrap_angle;

/// Where a husk may go: what the phone made of the room before the hunt.
///
/// Before a hunt the phone reads the surroundings while the player looks
/// around. What it has read becomes the plan, and the husk appears inside it
/// and never leaves it, so it is never somewhere the phone knows nothing
/// about. A plan says:
///
/// - an arc around the player that the husk keeps to;
/// - how far below the player's eye the floor is;
/// - how far the way is clear in each direction, when the phone could tell;
/// - what stands at the end of that way: how tall it is, whether there is
///   room to hide behind it, and whether its top is level enough to stand
///   on.
///
/// A plan never says a hunt cannot be played. Where the room is tight the
/// husk stands closer; it does not stay away.
///
/// Everything is stored in whole numbers so that a hunt replays exactly.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HuntPlan {
	/// The middle of the arc, to the right of where the player first faced.
	pub centre_centidegrees: i32,

	/// Half the arc's width. 18000 is the whole circle.
	pub half_width_centidegrees: i32,

	/// How far the floor is below the player's eye.
	pub eye_centimetres: i32,

	/// How far the way is clear along each slice, from the player to the
	/// first thing standing in it, or to the end of what the phone saw.
	///
	/// Empty when the phone could not tell, and the husk is then free to
	/// keep whatever distance it likes. Otherwise there is one entry for
	/// each of the [`SLICES`](Self::SLICES), starting directly behind the
	/// player and going round to the right.
	pub reach_decimetres: Vec<i32>,

	/// How tall the thing standing at the end of each slice's reach is, in
	/// centimetres above the floor: a counter, a sofa, a table. Zero where
	/// the slice ends because the floor that was seen ends, not at anything.
	///
	/// Empty when the phone could not tell. Otherwise there is one entry for
	/// each slice, in the order of `reach_decimetres`.
	pub cover_centimetres: Vec<i32>,

	/// Where a husk can hide behind the cover along each slice, in
	/// decimetres from the player: in the gap between its far side and
	/// whatever stands behind it. Zero where there is no room behind it.
	pub hide_decimetres: Vec<i32>,

	/// Whether the top of the cover along each slice is level enough to
	/// stand on: a table or a counter, not the back of a chair.
	pub ledges: Vec<bool>,
}

impl Default for HuntPlan {
	fn default() -> Self {
		Self::OPEN
	}
}

impl HuntPlan {
	/// A bearing brought to the edge by [`clamp`](Self::clamp) lands on it
	/// to within the last bit of a double, and still counts as inside.
	const EDGE: f64 = 1e-9;
	/// The tallest cover is recorded as: anything taller is as good as a
	/// wall.
	pub const MAX_COVER_CENTIMETRES: i32 = 255;
	pub const MAX_EYE_CENTIMETRES: i32 = 220;
	/// The farthest a reach is recorded: what fits in a byte.
	pub const MAX_REACH_DECIMETRES: i32 = 255;
	/// The lowest and highest a phone can plausibly be held above a floor.
	pub const MIN_EYE_CENTIMETRES: i32 = 50;
	/// The narrowest plan a hunt may be played in: a hundred degrees across.
	/// A narrower one would pen the husk in where it is too easy to follow.
	pub const MIN_HALF_WIDTH_CENTIDEGREES: i32 = 5000;
	/// The whole circle, with nothing known about the room: for a husk that
	/// may go anywhere, as in practice against the rules alone.
	pub const OPEN: Self = Self::new(0, 18000);
	/// The directions `reach_decimetres` is measured in: one every five
	/// degrees, all the way round.
	pub const SLICES: usize = 72;
	/// The width of one slice, in radians.
	pub const SLICE_WIDTH: f64 = 2.0 * PI / Self::SLICES as f64;

	/// The arc centred `centre_centidegrees` to the right, `half_width`
	/// either side, at the usual eye height and with nothing known about
	/// the room.
	pub const fn new(centre_centidegrees: i32, half_width_centidegrees: i32) -> Self {
		Self {
			centre_centidegrees,
			half_width_centidegrees,
			eye_centimetres: HuntRules::EYE_CENTIMETRES,
			reach_decimetres: Vec::new(),
			cover_centimetres: Vec::new(),
			hide_decimetres: Vec::new(),
			ledges: Vec::new(),
		}
	}

	/// The plan covering the arc from `from_radians` round to the right as
	/// far as `to_radians`, with nothing known about the room. Set the other
	/// fields on the result for what is known.
	pub fn between(from_radians: f64, to_radians: f64) -> Self {
		let mut width = math::modulo(to_radians - from_radians, 2.0 * PI);

		#[allow(clippy::float_cmp, reason = "a whole turn wraps to exactly zero")]
		if width == 0.0 && to_radians != from_radians {
			width = 2.0 * PI;
		}

		let centre = math::round(degrees(wrap_angle(from_radians + width / 2.0)) * 100.0);
		let half_width = math::round(degrees(width / 2.0) * 100.0).clamp(0, 18000);

		Self::new(centre as i32, half_width as i32)
	}

	pub fn centre(&self) -> f64 {
		radians(f64::from(self.centre_centidegrees) / 100.0)
	}

	pub fn half_width(&self) -> f64 {
		radians(f64::from(self.half_width_centidegrees) / 100.0)
	}

	/// How far the floor is below the player's eye, in metres.
	pub fn eye_height(&self) -> f64 {
		f64::from(self.eye_centimetres) / 100.0
	}

	pub const fn is_whole_circle(&self) -> bool {
		self.half_width_centidegrees >= 18000
	}

	/// Whether the phone measured how far the way is clear.
	pub fn is_surveyed(&self) -> bool {
		!self.reach_decimetres.is_empty()
	}

	/// Whether the phone also measured what stands in the way.
	pub fn knows_cover(&self) -> bool {
		!self.cover_centimetres.is_empty()
	}

	/// The slice the bearing `yaw` falls in.
	pub fn slice_of(yaw: f64) -> usize {
		math::floor_int((wrap_angle(yaw) + PI) / Self::SLICE_WIDTH)
			.clamp(0, Self::SLICES as i64 - 1) as usize
	}

	/// The bearing of the middle of slice `index`.
	pub fn bearing_of(index: usize) -> f64 {
		wrap_angle(index as f64 * Self::SLICE_WIDTH - PI + Self::SLICE_WIDTH / 2.0)
	}

	/// Whether the plan is one a phone could have made.
	pub fn is_well_formed(&self) -> bool {
		self.half_width_centidegrees >= Self::MIN_HALF_WIDTH_CENTIDEGREES
			&& self.half_width_centidegrees <= 18000
			&& self.centre_centidegrees >= -18000
			&& self.centre_centidegrees <= 18000
			&& self.eye_centimetres >= Self::MIN_EYE_CENTIMETRES
			&& self.eye_centimetres <= Self::MAX_EYE_CENTIMETRES
			&& (self.reach_decimetres.is_empty() || self.reach_decimetres.len() == Self::SLICES)
			&& self
				.reach_decimetres
				.iter()
				.all(|&reach| (0..=Self::MAX_REACH_DECIMETRES).contains(&reach))
			&& self.cover_is_well_formed()
	}

	/// Cover is all there or not there at all, and only ever with a reach to
	/// stand at the end of. A husk hides behind cover, never in front of it.
	fn cover_is_well_formed(&self) -> bool {
		if self.cover_centimetres.is_empty() {
			return self.hide_decimetres.is_empty() && self.ledges.is_empty();
		}

		if !self.is_surveyed()
			|| self.cover_centimetres.len() != Self::SLICES
			|| self.hide_decimetres.len() != Self::SLICES
			|| self.ledges.len() != Self::SLICES
		{
			return false;
		}

		(0..Self::SLICES).all(|slice| {
			let cover = self.cover_centimetres[slice];
			let hide = self.hide_decimetres[slice];

			(0..=Self::MAX_COVER_CENTIMETRES).contains(&cover)
				&& (0..=Self::MAX_REACH_DECIMETRES).contains(&hide)
				&& (hide == 0 || (cover != 0 && hide > self.reach_decimetres[slice]))
		})
	}

	/// Whether the bearing `yaw` lies inside the plan.
	pub fn contains(&self, yaw: f64) -> bool {
		self.is_whole_circle()
			|| wrap_angle(yaw - self.centre()).abs() <= self.half_width() + Self::EDGE
	}

	/// `yaw`, or the nearest bearing to it that lies inside the plan.
	pub fn clamp(&self, yaw: f64) -> f64 {
		if self.is_whole_circle() {
			return wrap_angle(yaw);
		}

		let half_width = self.half_width();
		let offset = math::clamp(wrap_angle(yaw - self.centre()), -half_width, half_width);

		wrap_angle(self.centre() + offset)
	}

	/// How far a husk may go along the bearing `yaw`, in metres, or infinity
	/// when the phone could not tell.
	///
	/// That is as far as the way is clear there, and no farther than it is
	/// clear nearby with [`HuntRules::REACH_EASE`] added for every radian
	/// between: a husk coming up on a sofa starts to draw in before it
	/// reaches it, and is round the front of it by the time it gets there.
	pub fn reach(&self, yaw: f64) -> f64 {
		let mut least = f64::INFINITY;

		for (slice, &reach) in self.reach_decimetres.iter().enumerate() {
			let away = math::max(
				0.0,
				wrap_angle(yaw - Self::bearing_of(slice)).abs() - Self::SLICE_WIDTH / 2.0,
			);

			least = math::min(
				least,
				f64::from(reach) / 10.0 + HuntRules::REACH_EASE * away,
			);
		}

		least
	}

	/// How far the way is clear along slice `index`, in metres, or zero when
	/// the phone could not tell.
	pub fn front_at(&self, index: usize) -> f64 {
		self.reach_decimetres
			.get(index)
			.map_or(0.0, |&reach| f64::from(reach) / 10.0)
	}

	/// How tall the cover along the bearing `yaw` is, in metres above the
	/// floor, or zero where nothing stands at the end of the way.
	pub fn cover_at(&self, yaw: f64) -> f64 {
		self.cover_centimetres
			.get(Self::slice_of(yaw))
			.map_or(0.0, |&cover| f64::from(cover) / 100.0)
	}

	/// Whether the line from the player's eye to `point`, in metres from the
	/// eye, passes through the cover standing along its bearing: whether a
	/// counter, a sofa or a table is in front of it.
	pub fn hides(&self, point: Vec3) -> bool {
		if !self.knows_cover() {
			return false;
		}

		let slice = Self::slice_of(point.yaw());
		let cover = self.cover_at(point.yaw());

		if cover == 0.0 {
			return false;
		}

		let front = self.front_at(slice);
		let away = math::sqrt(point.x * point.x + point.z * point.z);

		if away <= front {
			return false;
		}

		// Where the line of sight crosses the front of the cover, and the
		// top of the cover, both measured from the eye.
		let crossing = point.y * front / away;

		crossing < cover - self.eye_height()
	}

	/// How far out along slice `index` a husk can hide behind the cover
	/// there, in metres, or `None` when there is no room behind it.
	pub fn hide_at(&self, index: usize) -> Option<f64> {
		match self.hide_decimetres.get(index) {
			Some(&hide) if self.knows_cover() && hide != 0 => Some(f64::from(hide) / 10.0),
			_ => None,
		}
	}

	/// How high above the floor the level top of the cover along slice
	/// `index` is, in metres, or `None` when there is nothing there to stand
	/// on.
	pub fn ledge_at(&self, index: usize) -> Option<f64> {
		match (self.ledges.get(index), self.cover_centimetres.get(index)) {
			(Some(true), Some(&cover)) if cover != 0 => Some(f64::from(cover) / 100.0),
			_ => None,
		}
	}
}

impl fmt::Display for HuntPlan {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"HuntPlan({} ± {}, eye {} cm, {}{})",
			self.centre_centidegrees,
			self.half_width_centidegrees,
			self.eye_centimetres,
			if self.is_surveyed() {
				"surveyed"
			} else {
				"not surveyed"
			},
			if self.knows_cover() {
				", with cover"
			} else {
				""
			},
		)
	}
}
