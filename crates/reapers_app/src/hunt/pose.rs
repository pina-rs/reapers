//! Where the phone points during a hunt: the platform's own tracking where
//! it answers, and a dragging finger where it does not. The Flutter
//! `AdaptivePoseSource`, with `leptoskit_xr` in place of the gyroscope.
//!
//! The platform's frames arrive on their own task ([`super::xr`]) and are
//! kept here, with the touches and the room they bring, for the hunt's
//! clock to read. Its pose is kept in the platform's own space, because the
//! scene and the camera picture behind it are drawn in that space; a source
//! that stops answering leaves the view where it stood, and a hunt hangs
//! its fixed lights only on the first pose that source actually gives.

use reapers_rules::Vec3;
use reapers_rules::wrap_angle;
use reapers_world::ArPose;
use reapers_world::DragPose;
use reapers_world::PoseSource;

/// How long the platform may stay silent before the drag takes over again,
/// in seconds.
const SILENCE: f64 = 1.5;

/// Whether the platform's own tracking has answered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sensor {
	/// The platform has been asked for its pose, and nothing has come back.
	#[default]
	Waiting,
	/// A frame from the platform has been taken, or is being waited on while
	/// the drag stands in for it.
	Tracking,
	/// There is no platform tracking to be had: a browser without motion
	/// sensors, a simulator, or a refused permission.
	Missing,
}

/// What the platform's tracking says, as the hunt reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DevicePose {
	pub pose: ArPose,
	/// When it was taken, in seconds on the platform's clock.
	pub time: f64,
}

/// How the phone's view is being followed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tracking {
	/// By a finger: a browser, a simulator, a phone that cannot feel
	/// itself turn.
	#[default]
	Drag,
	/// By the phone's rotation sensors: it turns, but does not know where
	/// it is.
	Turning,
	/// By the platform's AR: the camera's picture and where the phone is.
	World,
}

/// The phone's pose, wherever it comes from.
#[derive(Clone, Debug, Default)]
pub struct LivePose {
	drag: DragPose,
	device: Option<DevicePose>,
	silent_for: f64,
	tracking: Tracking,
	sensor: Sensor,
	/// Yaw and pitch added to the drag pose so the view keeps the heading it
	/// had when the platform last let go.
	drag_yaw: f64,
	drag_pitch: f64,
	/// Roll carried by the drag pose, so the horizon does not snap level
	/// when the platform falls silent mid-hunt.
	drag_roll: f64,
	/// The vertical field of view of the platform's view, when it reports
	/// one: the camera's own lens.
	pub fov: Option<f64>,
	/// Whether a finger is resting on the screen long enough to be holding
	/// ORB ready.
	pub holding: bool,
	/// How far above the floor the phone is, from the platform's planes.
	pub eye_height: Option<f64>,
	/// Whether the platform measures the room's depth.
	pub depth: bool,
	/// Where the platform says the phone is, in its own space, in metres.
	pub position: Option<[f64; 3]>,
	/// Whether the platform draws its camera's picture behind the hunt.
	pub camera: bool,
}

impl LivePose {
	/// Where the phone points now.
	pub fn pose(&self) -> ArPose {
		match self.device {
			Some(device) if self.follows_device() => device.pose,
			_ => self.drag_pose(),
		}
	}

	/// The drag's pose, turned to meet the view where the platform left it.
	fn drag_pose(&self) -> ArPose {
		let pose = self.drag.pose();
		let level = ArPose::looking(
			Vec3::from_angles(
				wrap_angle(pose.forward.yaw() + self.drag_yaw),
				pose.forward.pitch() + self.drag_pitch,
			),
			pose.turn_speed,
		);

		if self.drag_roll == 0.0 {
			return level;
		}

		let (sine, cosine) = (self.drag_roll.sin(), self.drag_roll.cos());

		ArPose {
			forward: level.forward,
			up: level.up * cosine + level.right * sine,
			right: level.right * cosine - level.up * sine,
			turn_speed: level.turn_speed,
		}
	}

	/// Whether the view follows the phone itself rather than a drag.
	pub fn follows_device(&self) -> bool {
		self.device.is_some() && self.silent_for < SILENCE
	}

	/// How the view is being followed now.
	pub fn tracking(&self) -> Tracking {
		if self.follows_device() {
			self.tracking
		} else {
			Tracking::Drag
		}
	}

	/// What the platform's tracking has answered, if anything.
	pub const fn sensor(&self) -> Sensor {
		self.sensor
	}

	/// The pose of the source in use, once that source has produced one:
	/// the platform's first frame, or the drag where there is no platform.
	/// The hunt anchors its fixed lights on this and never before it.
	pub fn ready_pose(&self) -> Option<ArPose> {
		(self.sensor != Sensor::Waiting).then(|| self.pose())
	}

	/// The platform is being asked for its pose. Until it answers, the view
	/// has nothing but where it already stood.
	pub fn expect_platform(&mut self) {
		if !self.follows_device() {
			self.sensor = Sensor::Waiting;
		}
	}

	/// There is no platform tracking to be had, now or again. The drag
	/// takes over from wherever the view stood, without a swing. What the
	/// platform last said of the room and its picture stays: a session that
	/// has not answered yet may still do so.
	pub fn no_sensor(&mut self) {
		let shown = self.pose();
		self.device = None;
		self.adopt_drag(shown);
		self.sensor = Sensor::Missing;
	}

	/// The platform says this frame's pose cannot be trusted. The view
	/// stops following it, keeping the heading it was last given.
	pub fn untracked(&mut self) {
		let shown = self.pose();
		self.silent_for = SILENCE;
		self.adopt_drag(shown);
	}

	/// A finger moved by `dx` and `dy` radians across and up the view.
	pub fn drag(&mut self, dx: f64, dy: f64) {
		self.drag.drag(dx, dy);
	}

	/// Moves the drag's clock on, and counts how long the platform has been
	/// silent.
	pub fn advance(&mut self, delta: f64) {
		let shown = self.pose();
		let following = self.follows_device();
		self.drag.advance(delta);
		self.silent_for += delta;

		if following && !self.follows_device() {
			self.adopt_drag(shown);
		}
	}

	/// Puts the drag where the view is, so a source change keeps the
	/// picture still.
	fn adopt_drag(&mut self, shown: ArPose) {
		let raw = self.drag.pose();
		self.drag_yaw = wrap_angle(shown.forward.yaw() - raw.forward.yaw());
		self.drag_pitch = shown.forward.pitch() - raw.forward.pitch();
		let level = ArPose::looking(shown.forward, shown.turn_speed);
		self.drag_roll = shown.up.dot(level.right).atan2(shown.up.dot(level.up));
	}

	/// Takes a frame of the platform's tracking: `rotation` is the viewer's
	/// orientation, as the platform reports it (`[x, y, z, w]`), at `time`.
	/// The turn speed is measured from the frame before.
	///
	/// The pose is the platform's own, in the platform's own space: the
	/// scene, the room and the camera picture are all placed and drawn in
	/// that space, so turning it to a heading of the hunt's own would move
	/// the husk against the room behind it. The first frame of a fresh
	/// stretch takes over from the drag without measuring a turn across the
	/// gap, and the hunt hangs its sigil on the first such pose.
	pub fn track(&mut self, rotation: [f64; 4], time: f64, tracking: Tracking) {
		let Some(mut pose) = pose_of(rotation) else {
			return;
		};
		let take_over = !self.follows_device();

		if take_over {
			pose.turn_speed = 0.0;
		} else if let Some(before) = self.device {
			let elapsed = time - before.time;

			if elapsed > 1e-4 {
				let turned = pose.forward.angle_to(before.pose.forward);
				let measured = turned / elapsed;
				// Eased a little, as the gyroscope's was: one frame's jitter
				// is not a swing.
				pose.turn_speed =
					before.pose.turn_speed + (measured - before.pose.turn_speed) * 0.5;
			} else {
				pose.turn_speed = before.pose.turn_speed;
			}
		}

		self.device = Some(DevicePose { pose, time });
		self.silent_for = 0.0;
		self.tracking = tracking;
		self.sensor = Sensor::Tracking;
	}

	/// Forgets the platform: its session ended.
	pub fn lose_device(&mut self) {
		self.no_sensor();
		self.fov = None;
		self.depth = false;
		self.position = None;
		self.camera = false;
	}
}

/// A finite, non-zero quaternion as a pose, or `None` for a frame that
/// cannot say where the phone points.
fn pose_of(rotation: [f64; 4]) -> Option<ArPose> {
	let [x, y, z, w] = rotation;
	let length = (x * x + y * y + z * z + w * w).sqrt();

	if ![x, y, z, w, length].iter().all(|value| value.is_finite()) || length < 1e-6 {
		return None;
	}

	Some(ArPose::from_quaternion(
		[x / length, y / length, z / length, w / length],
		0.0,
	))
}

#[cfg(test)]
mod tests {
	use std::f64::consts::FRAC_PI_2;

	use reapers_rules::HuntRules;
	use reapers_rules::Sigil;
	use reapers_rules::SigilLight;
	use reapers_rules::SigilTrace;
	use reapers_rules::radians;

	use super::*;

	/// A turn of `angle` about the platform's vertical, as a quaternion.
	fn turned(angle: f64) -> [f64; 4] {
		[0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()]
	}

	fn yaw(live: &LivePose) -> f64 {
		live.pose().forward.yaw()
	}

	#[test]
	fn the_drag_leads_until_the_platform_answers_and_again_when_it_falls_silent() {
		let mut live = LivePose::default();
		live.drag(0.3, 0.0);
		assert!((yaw(&live) - 0.3).abs() < 1e-9);
		assert_eq!(live.tracking(), Tracking::Drag);

		live.track(turned(-0.3), 0.0, Tracking::World);
		assert_eq!(live.tracking(), Tracking::World);
		assert!((yaw(&live) - 0.3).abs() < 1e-9);

		live.advance(2.0);
		assert_eq!(live.tracking(), Tracking::Drag);
		assert!(
			(yaw(&live) - 0.3).abs() < 1e-9,
			"the drag takes over without swinging the view"
		);
	}

	#[test]
	fn turning_the_platform_left_turns_the_hunt_left_and_is_measured() {
		let mut live = LivePose::default();
		live.track(turned(0.0), 0.0, Tracking::Turning);
		// A right-handed turn about +Y swings −Z towards −X: to the left.
		live.track(turned(FRAC_PI_2 / 10.0), 0.1, Tracking::Turning);
		let pose = live.pose();
		assert!(pose.forward.yaw() < 0.0);
		assert!(pose.turn_speed > 0.5, "{}", pose.turn_speed);
	}

	#[test]
	fn the_platforms_heading_is_taken_whole_when_it_answers() {
		let mut live = LivePose::default();
		live.drag(0.8, 0.0);

		// The platform's own space, not a heading of the hunt's own: the
		// scene and the camera picture are drawn in that space too.
		live.track(turned(-2.4), 0.0, Tracking::Turning);

		assert!((yaw(&live) - 2.4).abs() < 1e-9);
		assert_eq!(live.sensor(), Sensor::Tracking);

		// What happens after the first frame is the phone's turn, measured
		// as it happens.
		live.track(turned(-2.5), 0.1, Tracking::Turning);
		assert!((yaw(&live) - 2.5).abs() < 1e-9);
	}

	#[test]
	fn a_quiet_platform_hands_the_view_to_the_drag_without_a_swing() {
		let mut live = LivePose::default();
		live.drag(0.4, 0.0);
		live.track(turned(-1.2), 0.0, Tracking::Turning);

		// No frames for longer than the platform is allowed to be quiet: the
		// view must not jump back to the drag's own origin.
		live.advance(2.0);
		assert_eq!(live.tracking(), Tracking::Drag);
		assert!((yaw(&live) - 1.2).abs() < 1e-9);

		// A finger may turn it from where it stood.
		live.drag(0.2, 0.0);
		assert!((yaw(&live) - 1.4).abs() < 1e-9);

		// The platform comes back on its own heading. That heading is the
		// space the scene and camera picture are drawn in, so it is taken
		// whole; the fixed lights themselves never move.
		live.track(turned(-2.9), 4.0, Tracking::Turning);
		assert_eq!(live.tracking(), Tracking::Turning);
		assert!((yaw(&live) - 2.9).abs() < 1e-9);
	}

	#[test]
	fn a_frame_that_cannot_say_where_the_phone_points_is_not_taken() {
		let mut live = LivePose::default();
		live.track(turned(0.5), 0.0, Tracking::Turning);
		let before = live.pose();

		for broken in [
			[f64::NAN, 0.0, 0.0, 1.0],
			[0.0, f64::INFINITY, 0.0, 1.0],
			[0.0, 0.0, 0.0, 0.0],
		] {
			live.track(broken, 1.0, Tracking::Turning);
			assert_eq!(live.pose().forward, before.forward);
		}
	}

	#[test]
	fn a_platform_that_has_not_answered_keeps_the_picture_it_announced() {
		let mut live = LivePose {
			fov: Some(1.05),
			camera: true,
			..LivePose::default()
		};

		live.no_sensor();

		assert_eq!(live.sensor(), Sensor::Missing);
		assert_eq!(live.fov, Some(1.05), "the session may still start");
		assert!(live.camera, "the camera picture is the session's");
	}

	#[test]
	fn the_pitch_and_roll_of_the_phone_survive_the_handover() {
		let mut live = LivePose::default();
		let pitch: f64 = 0.3;
		live.track(
			[(pitch / 2.0).sin(), 0.0, 0.0, (pitch / 2.0).cos()],
			0.0,
			Tracking::Turning,
		);
		assert!(live.pose().forward.pitch() > 0.2);

		let mut live = LivePose::default();
		let roll: f64 = 0.4;
		live.track(
			[0.0, 0.0, (roll / 2.0).sin(), (roll / 2.0).cos()],
			0.0,
			Tracking::Turning,
		);
		assert!(live.pose().up.x < -0.2, "the roll is kept");
	}

	#[test]
	fn a_quiet_platform_does_not_snap_a_rolled_horizon_level() {
		let mut live = LivePose::default();
		let roll: f64 = 0.4;
		live.track(
			[0.0, 0.0, (roll / 2.0).sin(), (roll / 2.0).cos()],
			0.0,
			Tracking::Turning,
		);
		let shown = live.pose();

		live.advance(2.0);
		assert_eq!(live.tracking(), Tracking::Drag);
		assert!(
			(live.pose().up - shown.up).length() < 1e-9,
			"the drag keeps the horizon where the platform left it"
		);
		assert!((live.pose().forward - shown.forward).length() < 1e-9);
	}

	#[test]
	fn a_small_hand_motion_carries_a_whole_sigil_without_a_swing() {
		// Three level lights, so a pure yaw stream can trace them: what is
		// under test is the pose and the trace, not the sigil's own shape.
		let sigil = Sigil {
			lights: vec![
				SigilLight {
					yaw: 0.0,
					pitch: 0.0,
				},
				SigilLight {
					yaw: 0.4,
					pitch: 0.0,
				},
				SigilLight {
					yaw: -0.25,
					pitch: 0.0,
				},
			],
			radius: radians(8.0),
			hold_seconds: 0.6,
			fray_seconds: None,
		};
		let mut trace = SigilTrace::new(sigil, 0.0);
		let mut live = LivePose::default();
		live.track(turned(0.0), 0.0, Tracking::Turning);

		let dt = 1.0 / 30.0;
		let mut time = 0.0;
		let mut origin = 0.0;
		let mut shown_yaw = 0.0;
		let mut frame = 0;

		for light in 0..3 {
			let target = trace.direction_of(light).yaw();

			// A hand turning at about a third of a radian a second, with a
			// third of a degree of tremor each way.
			while (target - shown_yaw).abs() > 0.01 {
				let step = (target - shown_yaw).clamp(-0.3 * dt, 0.3 * dt);
				shown_yaw += step;
				time += dt;
				frame += 1;
				let tremor = if frame % 2 == 0 { 0.006 } else { -0.006 };
				live.track(
					turned(origin - (shown_yaw + tremor)),
					time,
					Tracking::Turning,
				);
				assert!(
					live.pose().turn_speed <= HuntRules::SIGIL_STEADY_SPEED,
					"a hand tremor is not a swing: {}",
					live.pose().turn_speed
				);
				trace.advance(live.pose().forward, live.pose().turn_speed, dt);
			}

			// Hold until it catches.
			for _ in 0..60 {
				if trace.caught() > light {
					break;
				}

				time += dt;
				frame += 1;
				let tremor = if frame % 2 == 0 { 0.006 } else { -0.006 };
				live.track(
					turned(origin - (shown_yaw + tremor)),
					time,
					Tracking::Turning,
				);
				assert!(live.pose().turn_speed <= HuntRules::SIGIL_STEADY_SPEED);
				trace.advance(live.pose().forward, live.pose().turn_speed, dt);
			}

			assert_eq!(trace.caught(), light + 1, "light {light} catches");

			if light == 0 {
				// The platform goes quiet long enough for the drag to take
				// over, and comes back still facing the light being held.
				live.advance(2.0);
				assert_eq!(live.tracking(), Tracking::Drag);
				let held = live.pose().forward.yaw();
				origin = 0.0;
				time += dt;
				live.track(turned(-held), time, Tracking::Turning);
				assert!(
					(live.pose().forward.yaw() - held).abs() < 1e-9,
					"a returning platform does not move the light being held"
				);
			}
		}

		assert!(trace.is_closed(), "the whole sigil is traced");
	}
}
