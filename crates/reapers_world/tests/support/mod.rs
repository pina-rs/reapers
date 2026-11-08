//! What the tests share: sweeping a drawn room into a map, and comparing
//! numbers.

#![allow(dead_code, reason = "each test file uses some of it")]

use reapers_rules::HuntPlan;
use reapers_rules::Vec3;
use reapers_rules::radians;
use reapers_world::DepthFrame;
use reapers_world::DrawnRoom;
use reapers_world::RoomMap;
use reapers_world::Shot;

/// How a sweep takes its pictures.
#[derive(Clone, Copy)]
pub struct Sweep {
	/// Degrees between one picture and the next.
	pub every: f64,
	pub stretch: f64,
	pub noise: f64,
	/// Pictures made from the platform's rays, not a depth model's whole
	/// picture.
	pub rays: bool,
}

impl Default for Sweep {
	fn default() -> Self {
		Self {
			every: 20.0,
			stretch: 1.0,
			noise: 0.0,
			rays: false,
		}
	}
}

/// Takes pictures of `room` into `map` as a player would: sweeping from
/// `from` to `to` degrees with the phone tipped `pitch` degrees up, a
/// picture every 20 degrees.
pub fn sweep(map: &mut RoomMap, room: &DrawnRoom, from: f64, to: f64, pitch: f64) {
	sweep_with(map, room, from, to, pitch, Sweep::default());
}

/// [`sweep`], with the pictures taken as `how` says.
pub fn sweep_with(map: &mut RoomMap, room: &DrawnRoom, from: f64, to: f64, pitch: f64, how: Sweep) {
	for frame in pictures(room, from, to, pitch, how) {
		map.add(&frame);
	}
}

/// The pictures a sweep takes.
pub fn pictures(room: &DrawnRoom, from: f64, to: f64, pitch: f64, how: Sweep) -> Vec<DepthFrame> {
	let mut frames = Vec::new();
	let mut picture = 0;
	let mut yaw = from;

	while yaw <= to + 1e-9 {
		picture += 1;
		frames.push(if how.rays {
			room.ray_picture(yaw, pitch)
		} else {
			room.picture_with(
				yaw,
				pitch,
				&Shot {
					stretch: how.stretch,
					noise: how.noise,
					seed: picture,
					..Shot::default()
				},
			)
		});
		yaw += how.every;
	}

	frames
}

/// The slice that holds the bearing `degrees_right`.
pub fn slice_at(degrees_right: f64) -> usize {
	HuntPlan::slice_of(radians(degrees_right))
}

/// Whether `actual` is within `delta` of `expected`.
#[track_caller]
pub fn assert_close(actual: f64, expected: f64, delta: f64) {
	assert!(
		(actual - expected).abs() <= delta,
		"{actual} is not within {delta} of {expected}"
	);
}

/// Whether `actual` is within `tolerance` of `expected`.
#[track_caller]
pub fn assert_near(actual: Vec3, expected: Vec3, tolerance: f64) {
	assert!(
		(actual - expected).length() <= tolerance,
		"{actual:?} is not within {tolerance} of {expected:?}"
	);
}

/// Whether `actual` lies between `low` and `high`, both included.
#[track_caller]
pub fn assert_between(actual: f64, low: f64, high: f64) {
	assert!(
		(low..=high).contains(&actual),
		"{actual} is not between {low} and {high}"
	);
}
