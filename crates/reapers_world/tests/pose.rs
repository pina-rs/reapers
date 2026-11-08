//! The Rust side of `reapers_flutter/test/hunt/pose_test.dart`, and the pose
//! of a platform's AR frame.

mod support;

use core::f64::consts::PI;

use reapers_rules::Timestamp;
use reapers_rules::Vec3;
use reapers_rules::degrees;
use reapers_rules::radians;
use reapers_world::ArPose;
use reapers_world::AttitudeFilter;
use reapers_world::DragPose;
use reapers_world::PoseSource;
use reapers_world::SensorPose;
use reapers_world::from_platform;
use reapers_world::to_platform;
use support::assert_close;
use support::assert_near;

const GRAVITY: f64 = AttitudeFilter::STANDARD_GRAVITY;
const FOV: f64 = 62.0 * PI / 180.0;
const ASPECT: f64 = 9.0 / 19.5;
const RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);

/// A filter that has settled on a phone held upright in portrait.
fn upright() -> AttitudeFilter {
	let mut filter = AttitudeFilter::new();

	filter.add_acceleration(0.0, GRAVITY, 0.0);
	filter
}

/// Turns `filter` about a device axis at (`x`, `y`, `z`) radians a second
/// for `seconds`, in small steps as the sensor would deliver them.
fn turn(filter: &mut AttitudeFilter, (x, y, z): (f64, f64, f64), seconds: f64) {
	let step = 0.01;
	let mut elapsed = 0.0;

	while elapsed < seconds - 1e-9 {
		filter.add_rotation(x, y, z, step);
		elapsed += step;
	}
}

mod where_north_is {
	use super::*;

	/// A field that points north along the ground and dips into it, as the
	/// earth's does away from the equator, for a phone held upright with
	/// north lying `degrees_right` to the right of where it faces.
	fn feel(filter: &mut AttitudeFilter, degrees_right: f64) {
		let turn = radians(degrees_right);

		// Device axes: x right, y up, z toward the player.
		for _ in 0..400 {
			filter.add_magnetic_field(20.0 * turn.sin(), -40.0, -20.0 * turn.cos());
		}
	}

	#[test]
	fn is_not_known_until_the_compass_has_been_heard() {
		assert_eq!(upright().north(), None);
	}

	#[test]
	fn is_straight_ahead_when_the_phone_faces_north() {
		let mut filter = upright();

		feel(&mut filter, 0.0);

		assert_close(degrees(filter.north().unwrap()), 0.0, 0.1);
	}

	#[test]
	fn is_to_the_right_when_the_phone_faces_west() {
		let mut filter = upright();

		feel(&mut filter, 90.0);

		assert_close(degrees(filter.north().unwrap()), 90.0, 0.1);
	}

	#[test]
	fn stays_where_it_is_as_the_phone_turns() {
		let mut filter = upright();

		feel(&mut filter, 0.0);
		// Turn a quarter turn to the right: about the device's up axis, the
		// other way round from how angles are counted.
		turn(&mut filter, (0.0, -PI / 2.0, 0.0), 1.0);

		assert_close(degrees(filter.pose().forward.yaw()), 90.0, 0.5);

		// North is now on the phone's left, and the compass says so.
		feel(&mut filter, -90.0);

		assert_close(degrees(filter.north().unwrap()), 0.0, 1.0);
	}

	#[test]
	fn is_not_thrown_by_one_wild_reading() {
		let mut filter = upright();

		feel(&mut filter, 0.0);
		filter.add_magnetic_field(-300.0, -40.0, 5.0);

		assert!(degrees(filter.north().unwrap()).abs() < 4.0);
	}

	#[test]
	fn ignores_a_field_with_no_heading_in_it() {
		let mut filter = upright();

		filter.add_magnetic_field(0.1, -50.0, 0.1);

		assert_eq!(filter.north(), None);
	}
}

mod attitude_filter {
	use super::*;

	#[test]
	fn a_phone_held_upright_looks_straight_ahead_and_level() {
		let pose = upright().pose();

		assert_near(pose.forward, Vec3::FORWARD, 1e-3);
		assert_near(pose.up, Vec3::UP, 1e-3);
		assert_near(pose.right, RIGHT, 1e-3);
	}

	#[test]
	fn is_not_ready_until_gravity_has_been_felt() {
		let mut filter = AttitudeFilter::new();

		assert!(!filter.is_ready());

		filter.add_rotation(0.0, 1.0, 0.0, 0.01);

		assert!(!filter.is_ready());

		filter.add_acceleration(0.0, GRAVITY, 0.0);

		assert!(filter.is_ready());
	}

	#[test]
	fn a_phone_lying_face_up_looks_at_the_ground() {
		let mut filter = AttitudeFilter::new();

		filter.add_acceleration(0.0, 0.0, GRAVITY);

		assert_near(filter.pose().forward, Vec3::new(0.0, -1.0, 0.0), 1e-3);
	}

	#[test]
	fn turning_left_swings_the_view_to_the_left() {
		let mut filter = upright();

		// A quarter turn anticlockwise seen from above is a positive turn
		// about the phone's up axis.
		turn(&mut filter, (0.0, PI / 2.0, 0.0), 1.0);

		assert_near(filter.pose().forward, Vec3::new(-1.0, 0.0, 0.0), 0.01);
		assert_near(filter.pose().up, Vec3::UP, 0.01);
	}

	#[test]
	fn tilting_the_top_of_the_phone_back_looks_up() {
		let mut filter = upright();

		turn(&mut filter, (PI / 4.0, 0.0, 0.0), 1.0);

		assert_close(filter.pose().forward.pitch(), PI / 4.0, 0.01);
		assert_close(filter.pose().forward.yaw(), 0.0, 0.01);
	}

	#[test]
	fn the_three_directions_stay_at_right_angles() {
		let mut filter = upright();

		turn(&mut filter, (0.7, -1.3, 0.4), 2.0);

		let pose = filter.pose();

		assert_close(pose.forward.dot(pose.up), 0.0, 1e-6);
		assert_close(pose.forward.dot(pose.right), 0.0, 1e-6);
		assert_close(pose.up.dot(pose.right), 0.0, 1e-6);
		assert_close(pose.forward.length(), 1.0, 1e-6);
	}

	#[test]
	fn gravity_slowly_corrects_a_tilt_the_gyroscope_got_wrong() {
		let mut filter = upright();

		// The gyroscope claims a tilt that never happened.
		turn(&mut filter, (0.3, 0.0, 0.0), 1.0);

		let wrong = filter.pose().forward.pitch().abs();

		for _ in 0..400 {
			filter.add_acceleration(0.0, GRAVITY, 0.0);
		}

		assert!(wrong > 0.25);
		assert!(filter.pose().forward.pitch().abs() < 0.01);
	}

	#[test]
	fn gravity_never_changes_the_heading() {
		let mut filter = upright();

		turn(&mut filter, (0.0, 1.0, 0.0), 1.0);

		let heading = filter.pose().forward.yaw();

		for _ in 0..400 {
			filter.add_acceleration(0.0, GRAVITY, 0.0);
		}

		assert_close(filter.pose().forward.yaw(), heading, 1e-3);
	}

	#[test]
	fn a_shake_is_not_mistaken_for_gravity() {
		let mut filter = upright();

		for _ in 0..200 {
			filter.add_acceleration(14.0, 3.0, -6.0);
		}

		assert_near(filter.pose().forward, Vec3::FORWARD, 1e-3);
	}

	#[test]
	fn the_turn_speed_follows_the_gyroscope_and_settles_when_it_stops() {
		let mut filter = upright();

		turn(&mut filter, (0.0, 2.0, 0.0), 0.5);

		assert_close(filter.pose().turn_speed, 2.0, 0.05);

		turn(&mut filter, (0.0, 0.0, 0.0), 0.5);

		assert!(filter.pose().turn_speed < 0.01);
	}
}

mod sensor_pose {
	use super::*;

	#[test]
	fn looks_ahead_until_gravity_has_been_felt() {
		let mut source = SensorPose::new();

		source.rotation(0.0, 1.0, 0.0, Timestamp::from_millis(0));
		source.rotation(0.0, 1.0, 0.0, Timestamp::from_millis(10));

		assert!(!source.is_ready());
		assert_eq!(source.pose(), ArPose::AHEAD);
		assert!(source.follows_device());
	}

	#[test]
	fn turns_by_the_time_between_readings() {
		let mut source = SensorPose::new();

		source.acceleration(0.0, GRAVITY, 0.0);

		for step in 0..=100 {
			source.rotation(0.0, PI / 2.0, 0.0, Timestamp::from_millis(step * 10));
		}

		assert_near(source.pose().forward, Vec3::new(-1.0, 0.0, 0.0), 0.01);
	}

	#[test]
	fn a_stall_is_not_a_turn() {
		let mut source = SensorPose::new();

		source.acceleration(0.0, GRAVITY, 0.0);
		source.rotation(0.0, PI / 2.0, 0.0, Timestamp::from_millis(0));
		// A second's silence, then a reading: not a second of turning.
		source.rotation(0.0, PI / 2.0, 0.0, Timestamp::from_millis(1000));

		assert_near(source.pose().forward, Vec3::FORWARD, 1e-3);
	}
}

mod ar_pose {
	use super::*;

	#[test]
	fn the_middle_of_the_screen_is_where_the_phone_points() {
		assert_near(
			ArPose::AHEAD.ray_through(0.0, 0.0, FOV, ASPECT),
			Vec3::FORWARD,
			1e-3,
		);
	}

	#[test]
	fn the_top_edge_of_the_screen_is_half_the_field_of_view_up() {
		let ray = ArPose::AHEAD.ray_through(0.0, 1.0, FOV, ASPECT);

		assert_close(ray.pitch(), FOV / 2.0, 1e-9);
	}

	#[test]
	fn projecting_a_ray_gives_back_the_point_it_came_through() {
		let mut source = DragPose::new();

		source.drag(0.7, -0.3);

		let pose = source.pose();

		for (x, y) in [(0.0, 0.0), (0.6, -0.4), (-0.9, 0.8)] {
			let ray = pose.ray_through(x, y, FOV, ASPECT);
			let (across, up) = pose.project(ray, FOV, ASPECT).unwrap();

			assert_close(across, x, 1e-9);
			assert_close(up, y, 1e-9);
		}
	}

	#[test]
	fn what_is_behind_the_player_is_not_on_the_screen() {
		assert_eq!(
			ArPose::AHEAD.project(Vec3::new(0.0, 0.0, -1.0), FOV, ASPECT),
			None
		);
	}

	#[test]
	fn a_level_view_is_upright_and_not_rolled() {
		let pose = ArPose::looking(Vec3::new(1.0, -1.0, 1.0), 0.5);

		assert_close(pose.right.y, 0.0, 1e-12);
		assert_close(degrees(pose.forward.yaw()), 45.0, 1e-9);
		assert_close(pose.forward.dot(pose.up), 0.0, 1e-12);
		assert!(pose.up.y > 0.0);
		assert_close(pose.turn_speed, 0.5, 0.0);
	}
}

mod platform_pose {
	use super::*;

	/// A turn of `angle` radians about `axis`, as the platform writes it:
	/// `[x, y, z, w]`.
	fn about(axis: [f64; 3], angle: f64) -> [f64; 4] {
		let sine = (angle / 2.0).sin();

		[
			axis[0] * sine,
			axis[1] * sine,
			axis[2] * sine,
			(angle / 2.0).cos(),
		]
	}

	#[test]
	fn a_viewer_not_turned_at_all_looks_straight_ahead() {
		let pose = ArPose::from_quaternion([0.0, 0.0, 0.0, 1.0], 0.0);

		assert_near(pose.forward, Vec3::FORWARD, 1e-12);
		assert_near(pose.up, Vec3::UP, 1e-12);
		assert_near(pose.right, RIGHT, 1e-12);
	}

	#[test]
	fn turning_anticlockwise_about_up_looks_left() {
		// Right-handed, +Y up: a positive turn about +Y is anticlockwise
		// seen from above.
		let pose = ArPose::from_quaternion(about([0.0, 1.0, 0.0], PI / 2.0), 0.0);

		assert_near(pose.forward, Vec3::new(-1.0, 0.0, 0.0), 1e-12);
		assert_near(pose.up, Vec3::UP, 1e-12);
		assert_close(degrees(pose.forward.yaw()), -90.0, 1e-9);
	}

	#[test]
	fn tipping_about_right_looks_up() {
		let pose = ArPose::from_quaternion(about([1.0, 0.0, 0.0], PI / 6.0), 0.0);

		assert_close(degrees(pose.forward.pitch()), 30.0, 1e-9);
		assert_close(pose.forward.yaw(), 0.0, 1e-12);
	}

	#[test]
	fn agrees_with_the_attitude_filter() {
		// The filter, turned left a quarter and tipped up an eighth, in the
		// phone's own frame: the same as the platform's turn about +Y, then
		// about the turned +X.
		let mut filter = upright();

		turn(&mut filter, (0.0, PI / 2.0, 0.0), 1.0);
		turn(&mut filter, (PI / 4.0, 0.0, 0.0), 1.0);

		let [ax, ay, az, aw] = about([0.0, 1.0, 0.0], PI / 2.0);
		let [bx, by, bz, bw] = about([1.0, 0.0, 0.0], PI / 4.0);
		// a * b, applying b first.
		let both = [
			aw * bx + ax * bw + ay * bz - az * by,
			aw * by - ax * bz + ay * bw + az * bx,
			aw * bz + ax * by - ay * bx + az * bw,
			aw * bw - ax * bx - ay * by - az * bz,
		];
		let pose = ArPose::from_quaternion(both, 0.0);

		assert_near(pose.forward, filter.pose().forward, 0.01);
		assert_near(pose.up, filter.pose().up, 0.01);
	}

	#[test]
	fn a_quaternion_not_quite_of_length_one_is_taken_as_one() {
		let [x, y, z, w] = about([0.0, 1.0, 0.0], 0.4);
		let pose = ArPose::from_quaternion([x * 1.01, y * 1.01, z * 1.01, w * 1.01], 0.0);

		assert_close(pose.forward.length(), 1.0, 1e-12);
	}

	#[test]
	fn a_direction_goes_to_the_platform_and_back() {
		let ahead = to_platform(Vec3::FORWARD);

		assert_eq!(ahead, [0.0, 0.0, -1.0]);
		assert_eq!(from_platform(ahead), Vec3::FORWARD);
	}
}

mod drag_pose {
	use super::*;

	#[test]
	fn a_drag_to_the_right_turns_the_view_to_the_right() {
		let mut source = DragPose::new();

		source.drag(PI / 2.0, 0.0);

		assert_near(source.pose().forward, RIGHT, 1e-3);
		assert_near(source.pose().right, Vec3::new(0.0, 0.0, -1.0), 1e-3);
		assert!(!source.follows_device());
		assert_eq!(source.north(), None);
	}

	#[test]
	fn the_view_cannot_be_tipped_over_backwards() {
		let mut source = DragPose::new();

		source.drag(0.0, 10.0);

		assert!(source.pose().forward.pitch() < PI / 2.0);
		assert!(source.pose().up.y > 0.0);
	}

	#[test]
	fn reports_how_fast_it_was_dragged_then_comes_to_rest() {
		let mut source = DragPose::new();

		for _ in 0..30 {
			source.drag(0.03, 0.0);
			source.advance(1.0 / 60.0);
		}

		assert_close(source.pose().turn_speed, 1.8, 0.1);

		for _ in 0..60 {
			source.advance(1.0 / 60.0);
		}

		assert!(source.pose().turn_speed < 0.01);
	}
}
