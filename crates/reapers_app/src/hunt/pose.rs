//! Where the phone points during a hunt: the platform's own tracking where
//! it answers, and a dragging finger where it does not. The Flutter
//! `AdaptivePoseSource`, with `leptoskit_xr` in place of the gyroscope.
//!
//! The platform's frames arrive on their own task ([`super::xr`]) and are
//! kept here, with the touches and the room they bring, for the hunt's
//! clock to read.

use reapers_rules::Vec3;
use reapers_world::ArPose;
use reapers_world::DragPose;
use reapers_world::PoseSource;

/// How long the platform may stay silent before the drag takes over again,
/// in seconds.
const SILENCE: f64 = 1.5;

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
	/// The vertical field of view of the platform's view, when it reports
	/// one: the camera's own lens.
	pub fov: Option<f64>,
	/// Strikes the platform's touches made since they were last taken, as
	/// directions out from the player.
	taps: Vec<Vec3>,
	/// Whether a finger is resting on the screen long enough to be holding
	/// the art ready.
	pub holding: bool,
	/// How far above the floor the phone is, from the platform's planes.
	pub eye_height: Option<f64>,
	/// Whether the platform measures the room's depth.
	pub depth: bool,
}

impl LivePose {
	/// Where the phone points now.
	pub fn pose(&self) -> ArPose {
		match self.device {
			Some(device) if self.follows_device() => device.pose,
			_ => self.drag.pose(),
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

	/// A finger moved by `dx` and `dy` radians across and up the view.
	pub fn drag(&mut self, dx: f64, dy: f64) {
		self.drag.drag(dx, dy);
	}

	/// Moves the drag's clock on, and counts how long the platform has been
	/// silent.
	pub fn advance(&mut self, delta: f64) {
		self.drag.advance(delta);
		self.silent_for += delta;
	}

	/// Takes a frame of the platform's tracking: `rotation` is the viewer's
	/// orientation, as the platform reports it (`[x, y, z, w]`), at `time`.
	/// The turn speed is measured from the frame before.
	pub fn track(&mut self, rotation: [f64; 4], time: f64, tracking: Tracking) {
		let mut pose = ArPose::from_quaternion(rotation, 0.0);
		if let Some(before) = self.device {
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
	}

	/// Forgets the platform: its session ended.
	pub fn lose_device(&mut self) {
		self.device = None;
		self.fov = None;
		self.depth = false;
	}

	/// A touch the platform reported, aiming along `direction`.
	pub fn touch(&mut self, direction: Vec3) {
		self.taps.push(direction);
	}

	/// The touches since the last call.
	pub fn take_taps(&mut self) -> Vec<Vec3> {
		std::mem::take(&mut self.taps)
	}
}

#[cfg(test)]
mod tests {
	use std::f64::consts::FRAC_PI_2;

	use super::*;

	/// A turn of `angle` about the platform's vertical, as a quaternion.
	fn turned(angle: f64) -> [f64; 4] {
		[0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()]
	}

	#[test]
	fn the_drag_leads_until_the_platform_answers_and_again_when_it_falls_silent() {
		let mut live = LivePose::default();
		live.drag(0.3, 0.0);
		assert!((live.pose().forward.yaw() - 0.3).abs() < 1e-9);
		assert_eq!(live.tracking(), Tracking::Drag);
		live.track(turned(0.0), 0.0, Tracking::World);
		assert_eq!(live.tracking(), Tracking::World);
		assert!(live.pose().forward.yaw().abs() < 1e-9);
		live.advance(2.0);
		assert_eq!(live.tracking(), Tracking::Drag);
		assert!((live.pose().forward.yaw() - 0.3).abs() < 1e-9);
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
}
