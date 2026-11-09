//! Rooms drawn by hand, as a camera or the platform would see them.

use reapers_rules::SeededRandom;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::radians;

use crate::depth_frame::DepthFrame;
use crate::depth_frame::RayGrid;
use crate::math;
use crate::pose::ArPose;
use crate::room_survey::RoomSurvey;

/// Something standing on the floor of a [`DrawnRoom`]: a block `height`
/// metres tall, between `left` and `right` metres to the player's right and
/// between `near` and `far` metres ahead of where they first faced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
	pub left: f64,
	pub right: f64,
	pub near: f64,
	pub far: f64,
	pub height: f64,
}

/// A room drawn by hand, for testing what the map makes of pictures of it,
/// and for playing in on a phone or a simulator that cannot see a real one.
///
/// The player stands in the middle of a square room with their eye
/// `eye_height` metres above the floor. The walls are `wall` metres away on
/// every side and the ceiling is a metre above the eye.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawnRoom {
	pub eye_height: f64,
	pub wall: f64,
	pub blocks: Vec<Block>,

	/// How far the room is turned from the hunt's own axes, in radians to
	/// the right, about the player. A turned room tests that the grid finds
	/// the room's axes instead of the hunt's.
	pub room_yaw: f64,
}

/// How a picture of a [`DrawnRoom`] is taken: how big it is, and how wrong
/// its depths are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
	/// Every distance is this many times too long, as a model that
	/// misjudges scale reads them.
	pub stretch: f64,

	/// How far, as a share of the distance, each pixel may be out on its
	/// own.
	pub noise: f64,

	/// Where the noise starts from.
	pub seed: i64,

	/// How many pixels each silhouette is smeared across, as a depth model
	/// blurs a surface into the one behind it. Zero for a perfect picture.
	pub flying: usize,

	pub width: usize,
	pub height: usize,
}

impl Default for Shot {
	fn default() -> Self {
		Self {
			stretch: 1.0,
			noise: 0.0,
			seed: 1,
			flying: 0,
			width: 252,
			height: 448,
		}
	}
}

impl Shot {
	/// A step of more than this share between two pixels is a silhouette,
	/// so a drawn model smears its depth there rather than reading the
	/// nearer surface clean to its edge.
	pub const FLYING_EDGE: f64 = 0.10;
}

impl Default for DrawnRoom {
	fn default() -> Self {
		Self {
			eye_height: 1.45,
			wall: 4.0,
			blocks: Vec::new(),
			room_yaw: 0.0,
		}
	}
}

impl DrawnRoom {
	/// The Seeker's camera, held upright: how much a picture takes in.
	pub const TAN_HALF_HEIGHT: f64 = 0.7328;
	pub const TAN_HALF_WIDTH: f64 = 0.4122;

	/// An empty room `wall` metres from the player on every side.
	pub fn walled(wall: f64) -> Self {
		Self {
			wall,
			..Self::default()
		}
	}

	/// A room with `blocks` in it, otherwise as usual.
	pub fn with(blocks: &[Block]) -> Self {
		Self {
			blocks: blocks.to_vec(),
			..Self::default()
		}
	}

	/// A kitchen: a counter 92 cm high across the room ahead, 2.2 m away and
	/// 60 cm deep, with the far wall 4.5 m away. Something to hide behind
	/// and to leap onto.
	pub fn kitchen() -> Self {
		Self {
			wall: 4.5,
			blocks: vec![Block {
				left: -1.6,
				right: 1.6,
				near: 2.2,
				far: 2.8,
				height: 0.92,
			}],
			..Self::default()
		}
	}

	/// The pose of a phone held upright, turned `yaw` degrees to the right
	/// and tipped `pitch` degrees up.
	pub fn pose(yaw: f64, pitch: f64) -> ArPose {
		ArPose::looking(Vec3::from_angles(radians(yaw), radians(pitch)), 0.0)
	}

	/// The picture of the room a perfect depth model would hand back with
	/// the phone turned `yaw` degrees to the right and tipped `pitch`
	/// degrees up, taken where the player stands.
	pub fn picture(&self, yaw: f64, pitch: f64) -> DepthFrame {
		self.picture_with(yaw, pitch, &Shot::default())
	}

	/// The picture of the room as [`picture`](Self::picture), taken as
	/// `shot` says.
	pub fn picture_with(&self, yaw: f64, pitch: f64, shot: &Shot) -> DepthFrame {
		self.picture_from_with(Vec3::ZERO, yaw, pitch, shot)
	}

	/// The picture a perfect depth model would hand back from `eye`, turned
	/// `yaw` degrees to the right and tipped `pitch` degrees up, with the
	/// room's origin where the player first stood.
	pub fn picture_from(&self, eye: Vec3, yaw: f64, pitch: f64) -> DepthFrame {
		self.picture_from_with(eye, yaw, pitch, &Shot::default())
	}

	/// The picture of the room as [`picture_from`](Self::picture_from), taken
	/// as `shot` says.
	pub fn picture_from_with(&self, eye: Vec3, yaw: f64, pitch: f64, shot: &Shot) -> DepthFrame {
		let pose = Self::pose(yaw, pitch);
		let mut frame = DepthFrame {
			width: shot.width,
			height: shot.height,
			depths: vec![0.0; shot.width * shot.height],
			pose,
			origin: eye,
			tan_half_width: Self::TAN_HALF_WIDTH,
			tan_half_height: Self::TAN_HALF_HEIGHT,
		};
		let mut random = SeededRandom::new(shot.seed);

		for row in 0..shot.height {
			for column in 0..shot.width {
				let (across, up) = frame.tangents(column, row);
				let ray = pose.forward + pose.right * across + pose.up * up;
				let wobble = 1.0 + shot.noise * (random.next_double() * 2.0 - 1.0);

				frame.depths[row * shot.width + column] =
					(self.depth_along_from(eye, ray) * shot.stretch * wobble) as f32;
			}
		}

		if shot.flying > 0 {
			Self::smear_flying(&mut frame.depths, shot.width, shot.height, shot.flying);
		}

		frame
	}

	/// Blurs the depth across each silhouette by `pixels` pixels, the way a
	/// depth model reads a surface into the one behind it, so a filter can
	/// be held to what it leaves behind.
	///
	/// A step of more than [`Shot::FLYING_EDGE`] between two neighbours is a
	/// silhouette, and the midpoint of the two depths is written into the
	/// farther side of it. Both directions are swept, so a desk's front and
	/// its side are both smeared.
	fn smear_flying(depths: &mut [f32], width: usize, height: usize, pixels: usize) {
		let source = depths.to_vec();
		let mut smear = |one: usize, other: usize| {
			let (near, far) = (source[one], source[other]);

			if !(near > 0.0 && near.is_finite() && far > 0.0 && far.is_finite()) {
				return;
			}

			let step = (f64::from(near) - f64::from(far)).abs() / f64::from(near.min(far));

			if step <= Shot::FLYING_EDGE {
				return;
			}

			let midpoint = near.midpoint(far);

			// The band lies on the farther side of the silhouette, running
			// away from the nearer pixel.
			let (at, stride) = if near < far { (other, 1) } else { (one, -1) };

			for offset in 0..pixels {
				let at = at as i64 + stride * offset as i64;

				if at < 0 || at >= depths.len() as i64 {
					break;
				}

				depths[at as usize] = midpoint;
			}
		};

		for row in 0..height {
			for column in 0..width - 1 {
				smear(row * width + column, row * width + column + 1);
			}
		}

		for column in 0..width {
			for row in 0..height - 1 {
				smear(row * width + column, (row + 1) * width + column);
			}
		}
	}

	/// How far along `direction` the first surface is, in metres, as the
	/// platform's hit test against its depth would answer from where the
	/// player stands: `None` where it is not hit.
	pub fn hit(&self, direction: Vec3) -> Option<f64> {
		self.hit_from(Vec3::ZERO, direction)
	}

	/// How far along `direction` the first surface is from `eye`, in metres,
	/// as the platform's hit test against its depth would answer: `None`
	/// where it is not hit.
	pub fn hit_from(&self, eye: Vec3, direction: Vec3) -> Option<f64> {
		let metres = self.depth_along_from(eye, direction.normalized());

		metres.is_finite().then_some(metres)
	}

	/// The answers the platform would give to every ray of `grid`, in the
	/// order of [`RayGrid::rays`], from where the player stands.
	pub fn hits(&self, grid: &RayGrid) -> Vec<Option<f64>> {
		self.hits_from(Vec3::ZERO, grid)
	}

	/// The answers the platform would give to every ray of `grid`, in the
	/// order of [`RayGrid::rays`], cast from `eye`.
	pub fn hits_from(&self, eye: Vec3, grid: &RayGrid) -> Vec<Option<f64>> {
		grid.rays().map(|ray| self.hit_from(eye, ray)).collect()
	}

	/// The picture of the room a phone held as [`picture`](Self::picture)
	/// says would make from the platform's depth: a grid of rays, through
	/// the same view, made into a frame by [`DepthFrame::from_rays`].
	pub fn ray_picture(&self, yaw: f64, pitch: f64) -> DepthFrame {
		self.ray_picture_from(Vec3::ZERO, yaw, pitch)
	}

	/// The picture of the room a phone held as
	/// [`picture_from`](Self::picture_from) says, from `eye`, would make from
	/// the platform's depth: a grid of rays through the same view, made into
	/// a frame by [`DepthFrame::from_rays_at`].
	pub fn ray_picture_from(&self, eye: Vec3, yaw: f64, pitch: f64) -> DepthFrame {
		let grid = RayGrid::new(
			Self::pose(yaw, pitch),
			Self::TAN_HALF_WIDTH,
			Self::TAN_HALF_HEIGHT,
		);

		DepthFrame::from_rays_at(&grid, &self.hits_from(eye, &grid), eye)
	}

	/// The room read as a careful player tracing a long sigil would read
	/// it: looked at all the way round, low and level.
	///
	/// It is read the way a real room is, so a husk in it takes cover, leaps
	/// and hides exactly as it would in a real one.
	pub fn survey(&self, vessel: Vessel) -> RoomSurvey {
		let mut survey = RoomSurvey::new(vessel);

		for pitch in [-30.0, 5.0] {
			let mut yaw = -180.0;

			while yaw < 180.0 {
				survey.add(&self.picture(yaw, pitch));
				yaw += 15.0;
			}
		}

		survey
	}

	/// How far along `ray`, in multiples of its length, the first surface is
	/// from `eye`.
	fn depth_along_from(&self, eye: Vec3, ray: Vec3) -> f64 {
		// The room's own coordinates, when it is turned away from the
		// hunt's axes: the same turn a
		// [`RoomGrid`](reapers_rules::RoomGrid) stores as `grid_yaw`,
		// applied backwards, to the eye as well as the ray.
		let (eye, ray) = if self.room_yaw == 0.0 {
			(eye, ray)
		} else {
			let cosine = math::cos(self.room_yaw);
			let sine = math::sin(self.room_yaw);
			let turn = |x: f64, z: f64| (x * cosine - z * sine, x * sine + z * cosine);

			let (x, z) = turn(eye.x, eye.z);
			let (ray_x, ray_z) = turn(ray.x, ray.z);

			(Vec3::new(x, eye.y, z), Vec3::new(ray_x, ray.y, ray_z))
		};
		let mut nearest = f64::INFINITY;
		let mut consider = |distance: f64| {
			if distance > 1e-6 && distance < nearest {
				nearest = distance;
			}
		};

		if ray.y != 0.0 {
			consider((-self.eye_height - eye.y) / ray.y);
			consider((1.0 - eye.y) / ray.y);
		}

		if ray.x != 0.0 {
			consider((self.wall - eye.x) / ray.x);
			consider((-self.wall - eye.x) / ray.x);
		}

		if ray.z != 0.0 {
			consider((self.wall - eye.z) / ray.z);
			consider((-self.wall - eye.z) / ray.z);
		}

		for block in &self.blocks {
			if let Some(distance) = self.block_along(eye, ray, block) {
				consider(distance);
			}
		}

		nearest
	}

	/// Where a ray from `eye` enters `block`, or `None` when it misses.
	fn block_along(&self, eye: Vec3, ray: Vec3, block: &Block) -> Option<f64> {
		let mut enter: f64 = 0.0;
		let mut leave = f64::INFINITY;

		for (origin, direction, low, high) in [
			(eye.x, ray.x, block.left, block.right),
			(
				eye.y,
				ray.y,
				-self.eye_height,
				-self.eye_height + block.height,
			),
			(eye.z, ray.z, block.near, block.far),
		] {
			if direction == 0.0 {
				if low - origin > 0.0 || high - origin < 0.0 {
					return None;
				}

				continue;
			}

			let one = (low - origin) / direction;
			let other = (high - origin) / direction;

			enter = enter.max(one.min(other));
			leave = leave.min(one.max(other));
		}

		(enter < leave).then_some(enter)
	}
}
