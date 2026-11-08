//! Where the phone is pointing: from its motion sensors, from the
//! platform's AR, or from a finger dragged across the screen.

use reapers_rules::Timestamp;
use reapers_rules::Vec3;
use reapers_rules::radians;
use reapers_rules::wrap_angle;

use crate::math;
use crate::math::PI;

/// Where the phone is pointing and how fast it is turning.
///
/// The three directions are in the hunt's space (x right, y up, z ahead of
/// where the player first faced) and always at right angles to each other.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArPose {
	/// Out through the back of the phone: the middle of the view.
	pub forward: Vec3,

	/// Toward the top edge of the phone.
	pub up: Vec3,

	/// Toward the right edge of the phone.
	pub right: Vec3,

	/// How fast the phone is turning, in radians a second.
	pub turn_speed: f64,
}

impl ArPose {
	/// Looking straight ahead and level, perfectly still.
	pub const AHEAD: Self = Self {
		forward: Vec3::FORWARD,
		up: Vec3::UP,
		right: Vec3::new(1.0, 0.0, 0.0),
		turn_speed: 0.0,
	};

	/// A level view along `forward`: the phone held upright, not rolled.
	pub fn looking(forward: Vec3, turn_speed: f64) -> Self {
		let yaw = forward.yaw();
		let pitch = forward.pitch();

		Self {
			forward: Vec3::from_angles(yaw, pitch),
			up: Vec3::from_angles(yaw, pitch + PI / 2.0),
			right: Vec3::new(math::cos(yaw), 0.0, -math::sin(yaw)),
			turn_speed,
		}
	}

	/// The pose of a viewer the platform's AR reports as the unit
	/// quaternion `[x, y, z, w]`.
	///
	/// The platform's space is `leptoskit_xr`'s, and `ARKit`'s, `ARCore`'s
	/// and `WebXR`'s: right-handed, in metres, +Y up, with a viewer looking down
	/// its own −Z axis and its +X to the right of the view. Its −Z is the
	/// hunt's straight ahead, so the hunt faces wherever the platform's
	/// space does: see [`from_platform`]. Where the viewer stands is left
	/// out: the hunt is played around the player, who only ever turns.
	pub fn from_quaternion(rotation: [f64; 4], turn_speed: f64) -> Self {
		let [x, y, z, w] = rotation;
		let attitude = Rotation::new(w, x, y, z).normalized();

		attitude.pose(turn_speed)
	}

	/// The direction, out from the player, of a touch on the screen.
	///
	/// `x` and `y` run from -1 to 1 across the view, with y up.
	/// `vertical_fov` is the full vertical field of view in radians and
	/// `aspect` is the view's width over its height. This is the inverse of
	/// the projection the 3D view draws with, so a tap lands exactly on what
	/// was drawn there.
	pub fn ray_through(&self, x: f64, y: f64, vertical_fov: f64, aspect: f64) -> Vec3 {
		let half_height = math::tan(vertical_fov / 2.0);

		(self.forward + self.right * (x * half_height * aspect) + self.up * (y * half_height))
			.normalized()
	}

	/// Where `direction` appears on the screen, as x and y from -1 to 1 with
	/// y up, or `None` when it is behind the player.
	pub fn project(&self, direction: Vec3, vertical_fov: f64, aspect: f64) -> Option<(f64, f64)> {
		let depth = direction.dot(self.forward);

		if depth <= 1e-6 {
			return None;
		}

		let half_height = math::tan(vertical_fov / 2.0);

		Some((
			direction.dot(self.right) / depth / (half_height * aspect),
			direction.dot(self.up) / depth / half_height,
		))
	}
}

/// A direction in the hunt's space, as a direction in the platform's: the
/// one to cast a ray along. See [`ArPose::from_quaternion`].
pub fn to_platform(direction: Vec3) -> [f64; 3] {
	[direction.x, direction.y, -direction.z]
}

/// A direction in the platform's space, as a direction in the hunt's: the
/// platform looks down −Z, and the hunt's straight ahead is +Z.
pub fn from_platform([x, y, z]: [f64; 3]) -> Vec3 {
	Vec3::new(x, y, -z)
}

/// Something that knows where the phone is pointing.
///
/// A phone with motion sensors answers from them. Anything else, such as a
/// simulator or a desk, answers from a finger dragged across the screen, so
/// the hunt can be played everywhere it can be drawn.
pub trait PoseSource {
	/// The pose right now.
	fn pose(&self) -> ArPose;

	/// Whether the pose follows the phone itself rather than a drag.
	fn follows_device(&self) -> bool;

	/// Where north lies in the view, as a bearing in radians to the right of
	/// where the view first faced, or `None` for a source with no compass.
	fn north(&self) -> Option<f64>;

	/// Advances the source by `delta_seconds`. Called once a frame.
	fn advance(&mut self, delta_seconds: f64);

	/// Turns the view by a drag of (`dx`, `dy`) radians. Only a source that
	/// does not follow the device responds.
	fn drag(&mut self, dx: f64, dy: f64);
}

/// Works out which way a phone is pointing from its gyroscope, corrected by
/// its accelerometer.
///
/// The gyroscope is fast and smooth but drifts; the accelerometer always
/// knows which way is down but is noisy. Each gyroscope sample turns the
/// estimate, and each steady accelerometer sample pulls its sense of "down"
/// a little back toward the truth. Heading is never corrected: it only has
/// to stay put for the length of one hunt, and a compass would bring more
/// noise than it removes.
///
/// A compass is still listened to, for one thing: where north lies in the
/// view. That is what lets something be drawn at a real place on the earth,
/// where everyone standing nearby sees it in the same direction. It is kept
/// beside the pose, never mixed into it.
///
/// Readings use the sensor convention shared by Android and iOS: x to the
/// right of the screen, y toward its top, z out toward the player, with a
/// resting accelerometer reading pointing up.
#[derive(Clone, Debug)]
pub struct AttitudeFilter {
	/// Device to world. The world's y axis is up; its heading is wherever
	/// the phone pointed when the first accelerometer sample arrived.
	attitude: Rotation,
	levelled: bool,
	turn_speed: f64,
	north: Option<f64>,
}

impl Default for AttitudeFilter {
	fn default() -> Self {
		Self::new()
	}
}

impl AttitudeFilter {
	/// A field pointing this nearly straight up or down has no heading in it
	/// worth reading.
	const FLAT_FIELD_SHARE: f64 = 0.2;
	/// How strongly one accelerometer sample corrects the estimate.
	const GRAVITY_GAIN: f64 = 0.02;
	/// Accelerometer samples further than this from one g are the player
	/// moving the phone, not gravity, and are ignored.
	const GRAVITY_TOLERANCE: f64 = 0.18;
	/// How strongly one compass sample moves the sense of where north is. A
	/// compass jumps about, and north does not move, so each sample counts
	/// for little.
	const NORTH_GAIN: f64 = 0.03;
	/// How quickly the reported turn speed follows the gyroscope, as the time
	/// in seconds to cover most of a change.
	const SPEED_SMOOTHING: f64 = 0.07;
	/// One g, in metres a second squared.
	pub const STANDARD_GRAVITY: f64 = 9.80665;

	/// A filter that has heard nothing yet.
	pub const fn new() -> Self {
		Self {
			attitude: Rotation::IDENTITY,
			levelled: false,
			turn_speed: 0.0,
			north: None,
		}
	}

	/// Whether enough has been heard from the sensors to trust
	/// [`pose`](Self::pose).
	pub const fn is_ready(&self) -> bool {
		self.levelled
	}

	/// Feeds one gyroscope sample, in radians a second, taken
	/// `delta_seconds` after the last.
	pub fn add_rotation(&mut self, x: f64, y: f64, z: f64, delta_seconds: f64) {
		let speed = math::length(x, y, z);
		let blend = 1.0 - math::exp(-delta_seconds / Self::SPEED_SMOOTHING);

		self.turn_speed += (speed - self.turn_speed) * blend;

		if speed < 1e-9 {
			return;
		}

		// The gyroscope measures in the phone's own frame, so its turn is
		// applied before the attitude so far.
		let turn = Rotation::about(Vec3::new(x, y, z) / speed, speed * delta_seconds);

		self.attitude = (self.attitude * turn).normalized();
	}

	/// Feeds one accelerometer sample, in metres a second squared.
	pub fn add_acceleration(&mut self, x: f64, y: f64, z: f64) {
		let size = math::length(x, y, z);
		let steady = (size / Self::STANDARD_GRAVITY - 1.0).abs() <= Self::GRAVITY_TOLERANCE;

		if !steady {
			return;
		}

		let measured_up = Vec3::new(x, y, z) / size;

		if !self.levelled {
			// Start level with gravity and facing wherever the phone points.
			self.attitude = Rotation::between(measured_up, Vec3::UP, 1.0);
			self.levelled = true;
			return;
		}

		let believed_up = self.attitude.rotate(measured_up);
		let correction = Rotation::between(believed_up, Vec3::UP, Self::GRAVITY_GAIN);

		self.attitude = (correction * self.attitude).normalized();
	}

	/// Feeds one magnetometer sample. Only its direction is used, so the
	/// unit does not matter.
	pub fn add_magnetic_field(&mut self, x: f64, y: f64, z: f64) {
		let size = math::length(x, y, z);

		if !self.levelled || size < 1e-9 {
			return;
		}

		let field = from_platform_vec(self.attitude.rotate(Vec3::new(x, y, z) / size));
		let flat = math::sqrt(field.x * field.x + field.z * field.z);

		if flat < Self::FLAT_FIELD_SHARE {
			return;
		}

		let measured = math::atan2(field.x, field.z);

		self.north = Some(match self.north {
			None => measured,
			Some(north) => wrap_angle(north + wrap_angle(measured - north) * Self::NORTH_GAIN),
		});
	}

	/// Where magnetic north lies in the hunt's space, as a bearing in radians
	/// to the right of where the view first faced. `None` until the compass
	/// has been heard from.
	pub const fn north(&self) -> Option<f64> {
		self.north
	}

	/// How fast the phone is turning, in radians a second, smoothed.
	pub const fn turn_speed(&self) -> f64 {
		self.turn_speed
	}

	/// The current pose, in the hunt's space.
	pub fn pose(&self) -> ArPose {
		self.attitude.pose(self.turn_speed)
	}
}

/// Follows the phone itself, through its gyroscope, accelerometer and
/// compass, given each reading as it arrives.
///
/// It is what a sensor stream feeds: it works out how long passed between
/// one gyroscope reading and the next from their times, and answers
/// [`ArPose::AHEAD`] until gravity has been felt.
#[derive(Clone, Debug, Default)]
pub struct SensorPose {
	filter: AttitudeFilter,
	last_rotation: Option<Timestamp>,
}

impl SensorPose {
	/// Gaps longer than this between gyroscope samples, in microseconds, are
	/// a stall, not a turn, and are not integrated.
	const LONGEST_STEP_MICROS: i64 = 100_000;

	/// A source that has heard nothing yet.
	pub const fn new() -> Self {
		Self {
			filter: AttitudeFilter::new(),
			last_rotation: None,
		}
	}

	/// Whether the sensors have produced a usable pose yet.
	pub const fn is_ready(&self) -> bool {
		self.filter.is_ready()
	}

	/// Feeds one gyroscope sample, in radians a second, taken `at`.
	pub fn rotation(&mut self, x: f64, y: f64, z: f64, at: Timestamp) {
		let last = self.last_rotation.replace(at);
		let Some(last) = last else {
			return;
		};
		let step = at.micros_since(last);

		if step <= 0 || step > Self::LONGEST_STEP_MICROS {
			return;
		}

		self.filter.add_rotation(x, y, z, step as f64 / 1_000_000.0);
	}

	/// Feeds one accelerometer sample, in metres a second squared.
	pub fn acceleration(&mut self, x: f64, y: f64, z: f64) {
		self.filter.add_acceleration(x, y, z);
	}

	/// Feeds one magnetometer sample.
	pub fn magnetic_field(&mut self, x: f64, y: f64, z: f64) {
		self.filter.add_magnetic_field(x, y, z);
	}
}

impl PoseSource for SensorPose {
	fn pose(&self) -> ArPose {
		if self.filter.is_ready() {
			self.filter.pose()
		} else {
			ArPose::AHEAD
		}
	}

	fn follows_device(&self) -> bool {
		true
	}

	fn north(&self) -> Option<f64> {
		self.filter.north()
	}

	fn advance(&mut self, _delta_seconds: f64) {}

	fn drag(&mut self, _dx: f64, _dy: f64) {}
}

/// Follows a finger dragged across the screen, for anywhere the phone's own
/// motion cannot be read: a simulator, a desktop, a device with no
/// gyroscope.
#[derive(Clone, Debug, Default)]
pub struct DragPose {
	yaw: f64,
	pitch: f64,
	pending_turn: f64,
	turn_speed: f64,
}

impl DragPose {
	/// How far up or down the view can be tilted.
	const PITCH_LIMIT: f64 = radians(80.0);
	/// How quickly the reported turn speed settles once the drag stops, as
	/// the time in seconds to lose most of it.
	const SPEED_DECAY: f64 = 0.09;

	/// Looking straight ahead and level.
	pub const fn new() -> Self {
		Self {
			yaw: 0.0,
			pitch: 0.0,
			pending_turn: 0.0,
			turn_speed: 0.0,
		}
	}
}

impl PoseSource for DragPose {
	fn pose(&self) -> ArPose {
		ArPose::looking(Vec3::from_angles(self.yaw, self.pitch), self.turn_speed)
	}

	fn follows_device(&self) -> bool {
		false
	}

	fn north(&self) -> Option<f64> {
		None
	}

	fn advance(&mut self, delta_seconds: f64) {
		if delta_seconds <= 0.0 {
			return;
		}

		let measured = self.pending_turn / delta_seconds;
		let blend = 1.0 - math::exp(-delta_seconds / Self::SPEED_DECAY);

		self.pending_turn = 0.0;
		self.turn_speed += (measured - self.turn_speed) * blend;
	}

	fn drag(&mut self, dx: f64, dy: f64) {
		self.yaw = wrap_angle(self.yaw + dx);
		self.pitch = (self.pitch + dy).clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT);
		self.pending_turn += math::sqrt(dx * dx + dy * dy);
	}
}

/// The sensors' and the platform's world has z toward the player; the
/// hunt's has z ahead.
fn from_platform_vec(world: Vec3) -> Vec3 {
	Vec3::new(world.x, world.y, -world.z)
}

/// A rotation as a unit quaternion, with the textbook conventions: a
/// positive angle turns anticlockwise about its axis, and `a * b` applies
/// `b` first.
///
/// The filter keeps its own because the sign conventions of a rotation are
/// exactly what it must not get wrong, and they are easier to trust when
/// they are written down next to the code that depends on them.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rotation {
	w: f64,
	x: f64,
	y: f64,
	z: f64,
}

impl Rotation {
	const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0);

	const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
		Self { w, x, y, z }
	}

	/// A turn of `angle` radians about `axis`, which must have length 1.
	fn about(axis: Vec3, angle: f64) -> Self {
		let sine = math::sin(angle / 2.0);

		Self::new(
			math::cos(angle / 2.0),
			axis.x * sine,
			axis.y * sine,
			axis.z * sine,
		)
	}

	/// The shortest turn that carries `from` onto `to`, or the first `share`
	/// of that turn. Both must have length 1.
	fn between(from: Vec3, to: Vec3, share: f64) -> Self {
		let angle = from.angle_to(to);
		let axis = from.cross(to);

		if axis.length() < 1e-9 {
			// Already aligned, or exactly opposed. Opposed has no single
			// shortest turn, so pick the one about the x axis.
			return if angle < 1.0 {
				Self::IDENTITY
			} else {
				Self::about(Vec3::new(1.0, 0.0, 0.0), PI * share)
			};
		}

		Self::about(axis / axis.length(), angle * share)
	}

	fn normalized(self) -> Self {
		let size =
			math::sqrt(self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z);

		Self::new(self.w / size, self.x / size, self.y / size, self.z / size)
	}

	/// Turns `vector` by this rotation.
	fn rotate(self, vector: Vec3) -> Vec3 {
		let Self { w, x, y, z } = self;
		// v + 2w(q x v) + 2 q x (q x v), with q the vector part.
		let cx = y * vector.z - z * vector.y;
		let cy = z * vector.x - x * vector.z;
		let cz = x * vector.y - y * vector.x;

		Vec3::new(
			vector.x + 2.0 * (w * cx + y * cz - z * cy),
			vector.y + 2.0 * (w * cy + z * cx - x * cz),
			vector.z + 2.0 * (w * cz + x * cy - y * cx),
		)
	}

	/// The pose of a viewer turned by this rotation from looking down −Z
	/// with +Y up, in the hunt's space.
	fn pose(self, turn_speed: f64) -> ArPose {
		ArPose {
			forward: from_platform_vec(self.rotate(Vec3::new(0.0, 0.0, -1.0))),
			up: from_platform_vec(self.rotate(Vec3::UP)),
			right: from_platform_vec(self.rotate(Vec3::new(1.0, 0.0, 0.0))),
			turn_speed,
		}
	}
}

impl core::ops::Mul for Rotation {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		let Self { w, x, y, z } = self;

		Self::new(
			w * other.w - x * other.x - y * other.y - z * other.z,
			w * other.x + x * other.w + y * other.z - z * other.y,
			w * other.y - x * other.z + y * other.w + z * other.x,
			w * other.z + x * other.y - y * other.x + z * other.w,
		)
	}
}
