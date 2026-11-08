//! The Rust side of `reapers_flutter/test/hunt/depth/occlusion_map_test.dart`.

mod support;

use core::f64::consts::PI;

use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::degrees;
use reapers_rules::radians;
use reapers_world::DrawnRoom;
use reapers_world::OcclusionMap;
use reapers_world::RoomSurvey;
use reapers_world::Shot;
use support::assert_close;

const EYE: f64 = 1.45;

/// Whether the room hides `point`, in metres from the eye.
fn hides(map: &OcclusionMap, point: Vec3) -> bool {
	map.hides(point.yaw(), point.pitch(), point.length())
}

/// Reads `room` the way a hunt does, at a model that reads everything
/// `stretch` times too long.
fn read(room: &DrawnRoom, stretch: f64) -> OcclusionMap {
	let mut survey = RoomSurvey::new(Vessel::Ape);
	let mut yaw = -60.0;

	while yaw <= 60.0 {
		survey.add(&room.picture_with(
			yaw,
			-30.0,
			&Shot {
				stretch,
				..Shot::default()
			},
		));
		yaw += 10.0;
	}

	survey.occlusion().clone()
}

fn kitchen() -> OcclusionMap {
	read(&DrawnRoom::kitchen(), 1.0)
}

mod what_the_room_hides {
	use super::*;

	#[test]
	fn a_husk_behind_the_counter_is_hidden_below_its_top() {
		let room = kitchen();

		assert!(hides(&room, Vec3::new(0.0, -EYE + 0.3, 3.4)));
		assert!(hides(&room, Vec3::new(0.6, -EYE + 0.6, 3.4)));
	}

	#[test]
	fn what_rises_above_the_counter_is_seen_over_it() {
		assert!(!hides(&kitchen(), Vec3::new(0.0, -EYE + 1.4, 3.4)));
	}

	#[test]
	fn in_front_of_the_counter_nothing_is_hidden() {
		let room = kitchen();

		assert!(!hides(&room, Vec3::new(0.0, -EYE + 0.05, 1.8)));
		assert!(!hides(&room, Vec3::new(-0.5, -EYE + 0.5, 1.6)));
	}

	#[test]
	fn the_floor_never_hides_a_husk_standing_on_it() {
		let room = read(&DrawnRoom::walled(4.5), 1.0);

		assert!(!hides(&room, Vec3::new(0.0, -EYE + 0.02, 3.5)));
		assert!(!hides(&room, Vec3::new(1.0, -EYE + 0.02, 2.5)));
	}

	#[test]
	fn a_model_that_reads_long_still_puts_the_counter_in_front() {
		let room = read(&DrawnRoom::kitchen(), 1.25);

		assert!(hides(&room, Vec3::new(0.0, -EYE + 0.3, 3.4)));
		assert!(!hides(&room, Vec3::new(0.0, -EYE + 0.05, 1.8)));
	}

	#[test]
	fn where_nothing_was_seen_nothing_is_hidden() {
		let room = kitchen();

		assert_eq!(room.distance_at(PI, 0.0), None);
		assert!(!hides(&room, Vec3::new(0.0, -EYE + 0.3, -3.4)));
	}
}

#[test]
fn the_shader_reads_the_same_distances_the_map_holds() {
	let room = kitchen();
	let pixels = room.to_pixels();
	let towards = Vec3::new(0.0, -EYE + 0.4, 2.5);
	let column = ((degrees(towards.yaw()) + 180.0) / OcclusionMap::CELL_DEGREES).floor() as usize;
	let row = ((OcclusionMap::HIGHEST - degrees(towards.pitch())) / OcclusionMap::CELL_DEGREES)
		.floor() as usize;
	let at = (row * OcclusionMap::COLUMNS + column) * 4;
	let centimetres = u16::from_be_bytes([pixels[at], pixels[at + 1]]);

	assert_eq!(pixels.len(), OcclusionMap::COLUMNS * OcclusionMap::ROWS * 4);
	assert_eq!(pixels[at + 2], 255, "something stands there");
	assert_close(
		f64::from(centimetres) / 100.0,
		room.distance_at(towards.yaw(), towards.pitch()).unwrap(),
		0.01,
	);
}

#[test]
fn the_shader_is_told_where_only_floor_was_seen_and_where_nothing_was() {
	let room = kitchen();
	let pixels = room.to_pixels();
	let cell = |yaw: f64, pitch: f64| {
		let column = ((yaw + 180.0) / OcclusionMap::CELL_DEGREES).floor() as usize;
		let row = ((OcclusionMap::HIGHEST - pitch) / OcclusionMap::CELL_DEGREES).floor() as usize;

		(row * OcclusionMap::COLUMNS + column) * 4
	};
	// The floor a metre and a half ahead, in front of the counter.
	let floor = cell(0.0, degrees((-EYE / 1.5_f64).atan()));
	let behind = cell(180.0 - 0.1, 0.0);

	assert_eq!(
		room.distance_at(0.0, radians(degrees((-EYE / 1.5_f64).atan()))),
		Some(f64::INFINITY)
	);
	assert_eq!(&pixels[floor..floor + 4], &[0xFF, 0xFF, 128, 255]);
	assert_eq!(&pixels[behind..behind + 4], &[0, 0, 0, 255]);
}

#[test]
fn a_room_drawn_by_hand_is_read_like_a_real_one() {
	let survey = DrawnRoom::kitchen().survey(Vessel::Ape);
	let plan = survey.plan_within(0.0, radians(80.0));

	assert!(plan.knows_cover());
	assert!(plan.is_well_formed());
	assert!(survey.occlusion().pictures() > 10);
	assert!(hides(survey.occlusion(), Vec3::new(0.0, -EYE + 0.3, 3.4)));
}
