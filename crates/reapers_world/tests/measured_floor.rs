//! A floor the platform measured: how far below the eye it is, in real
//! metres, anchoring the map of the room.

mod support;

use reapers_rules::HuntRules;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::radians;
use reapers_world::Block;
use reapers_world::DrawnRoom;
use reapers_world::RoomMap;
use reapers_world::RoomSurvey;
use support::Sweep;
use support::assert_close;
use support::pictures;
use support::slice_at;
use support::sweep;
use support::sweep_with;

/// A table right across the view, so near that no floor shows at all, and
/// ending a metre short of the far wall.
fn table() -> DrawnRoom {
	DrawnRoom::with(&[Block {
		left: -3.0,
		right: 3.0,
		near: 0.4,
		far: 3.0,
		height: 0.75,
	}])
}

#[test]
fn makes_the_plans_eye_height_exact_before_any_picture() {
	let mut map = RoomMap::new(Vessel::Penguin);

	assert!(map.measure_floor(1.32));
	assert!(map.floor_is_measured());
	assert_eq!(map.floor_depth(), Some(1.32));
	assert_eq!(map.plan_within(0.0, radians(60.0)).eye_centimetres, 132);
	assert!(!map.plan_within(0.0, radians(60.0)).is_surveyed());
}

#[test]
fn makes_the_plans_eye_height_exact_where_the_pictures_only_come_close() {
	let room = DrawnRoom {
		eye_height: 1.32,
		..DrawnRoom::default()
	};
	let mut found = RoomMap::new(Vessel::Penguin);
	let mut measured = RoomMap::new(Vessel::Penguin);

	measured.measure_floor(1.32);
	sweep(&mut found, &room, -60.0, 60.0, -30.0);
	sweep(&mut measured, &room, -60.0, 60.0, -30.0);

	assert_close(found.floor_depth().unwrap(), 1.32, 0.03);
	assert_eq!(
		measured.plan_within(0.0, radians(60.0)).eye_centimetres,
		132
	);
	assert_close(measured.reach(slice_at(0.0)).unwrap(), 4.0, 0.25);
	assert_eq!(measured.last_picture_floor(), Some(1.32));
}

#[test]
fn charts_a_picture_with_no_floor_in_it() {
	// Read against the floors found in the pictures, a picture of nothing
	// but table top takes the table for the floor, and once a few pictures
	// agree where the floor really is, it is set aside. Against a measured
	// floor, the table is charted as what it is: something 75 cm tall with
	// a level top.
	let mut found = RoomMap::new(Vessel::Penguin);
	let mut measured = RoomMap::new(Vessel::Penguin);

	sweep(&mut found, &DrawnRoom::default(), -40.0, 20.0, -30.0);
	measured.measure_floor(1.45);

	assert!(!found.add(&table().picture(0.0, -30.0)));

	let mut yaw = -10.0;

	while yaw <= 10.0 {
		assert!(measured.add(&table().picture(yaw, -30.0)));
		yaw += 5.0;
	}

	let top = measured.furniture(slice_at(0.0)).unwrap();

	assert_close(top.cover, 0.75, 0.08);
	assert!(top.ledge);
}

#[test]
fn rescales_what_was_charted_before_it() {
	// Pictures that read everything 20% long put the wall 20% too far.
	// Measured, the floor puts it back: the map keeps every length in
	// eye-heights, and the measurement says what an eye-height is.
	let mut map = RoomMap::new(Vessel::Penguin);

	sweep_with(
		&mut map,
		&DrawnRoom::default(),
		-60.0,
		60.0,
		-30.0,
		Sweep {
			stretch: 1.2,
			..Sweep::default()
		},
	);

	assert_close(map.reach(slice_at(0.0)).unwrap(), 4.0 * 1.2, 0.3);

	map.measure_floor(1.45);

	assert_close(map.reach(slice_at(0.0)).unwrap(), 4.0, 0.25);
	assert_eq!(map.plan_within(0.0, radians(60.0)).eye_centimetres, 145);
}

#[test]
fn may_be_measured_again_and_the_last_stands() {
	let mut map = RoomMap::new(Vessel::Penguin);

	map.measure_floor(1.4);
	map.measure_floor(1.46);

	assert_eq!(map.floor_depth(), Some(1.46));
}

#[test]
fn is_not_taken_at_a_height_no_phone_is_held_at() {
	let mut map = RoomMap::new(Vessel::Penguin);

	for nonsense in [f64::NAN, f64::INFINITY, -1.45, 0.1, 3.5] {
		assert!(!map.measure_floor(nonsense), "{nonsense}");
	}

	assert!(!map.floor_is_measured());
	assert_eq!(map.floor_depth(), None);
	assert_eq!(
		map.plan_within(0.0, radians(60.0)).eye_centimetres,
		HuntRules::EYE_CENTIMETRES
	);
}

#[test]
fn reads_the_platforms_rays_against_it() {
	let mut survey = RoomSurvey::new(Vessel::Ape);
	let sweep = Sweep {
		every: 10.0,
		rays: true,
		..Sweep::default()
	};

	survey.measure_floor(1.45);

	for frame in pictures(&DrawnRoom::kitchen(), -60.0, 60.0, -30.0, sweep) {
		survey.add(&frame);
	}

	let counter = survey.map().furniture(slice_at(0.0)).unwrap();
	let hides = |point: Vec3| {
		survey
			.occlusion()
			.hides(point.yaw(), point.pitch(), point.length())
	};

	assert_close(counter.cover, 0.92, 0.08);
	assert!(counter.ledge);
	assert_eq!(survey.plan_within(0.0, radians(60.0)).eye_centimetres, 145);
	assert!(hides(Vec3::new(0.0, -1.45 + 0.3, 3.4)));
	assert!(!hides(Vec3::new(0.0, -1.45 + 0.02, 1.8)));
}

#[test]
fn estimates_use_their_own_floor_with_a_measured_anchor() {
	// ARKit/ARCore can track a floor while the camera model reads the
	// entire room long or short. The wall and counter must still stand
	// in real metres, and occlusion must use the same calibration.
	for stretch in [0.7, 1.0, 1.3] {
		let mut survey = RoomSurvey::new(Vessel::Ape);
		survey.measure_floor(1.45);
		for frame in pictures(
			&DrawnRoom::kitchen(),
			-60.0,
			60.0,
			-30.0,
			Sweep {
				stretch,
				every: 10.0,
				..Sweep::default()
			},
		) {
			assert!(survey.add_estimated(&frame));
		}
		assert_eq!(survey.map().floor_depth(), Some(1.45));
		assert_close(
			survey.map().last_picture_floor().unwrap(),
			1.45 * stretch,
			0.05,
		);
		assert_close(survey.reach(slice_at(0.0)).unwrap(), 2.2, 0.25);
		let counter = survey.map().furniture(slice_at(0.0)).unwrap();
		assert_close(counter.cover, 0.92, 0.08);
		let point = Vec3::new(0.0, -1.45 + 0.3, 3.4);
		assert!(
			survey
				.occlusion()
				.hides(point.yaw(), point.pitch(), point.length())
		);
		let point = Vec3::new(0.0, -1.45 + 0.02, 1.8);
		assert!(
			!survey
				.occlusion()
				.hides(point.yaw(), point.pitch(), point.length())
		);
	}
}

#[test]
fn estimates_without_a_floor_do_not_borrow_the_measured_floor() {
	let mut survey = RoomSurvey::new(Vessel::Penguin);
	survey.measure_floor(1.45);
	assert!(!survey.add_estimated(&DrawnRoom::walled(1.0).picture(0.0, 25.0)));
	assert_eq!(survey.pictures(), 0);
	assert_eq!(survey.occlusion().pictures(), 0);
	assert_eq!(survey.map().floor_depth(), Some(1.45));
}

#[test]
fn measured_and_estimated_pictures_share_the_same_map() {
	let mut map = RoomMap::new(Vessel::Penguin);
	map.measure_floor(1.45);
	let room = DrawnRoom::default();
	for frame in pictures(&room, -60.0, -10.0, -30.0, Sweep::default()) {
		assert!(map.add(&frame));
	}
	for frame in pictures(
		&room,
		10.0,
		60.0,
		-30.0,
		Sweep {
			stretch: 1.3,
			..Sweep::default()
		},
	) {
		assert!(map.add_estimated(&frame));
	}
	let mut measured = RoomMap::new(Vessel::Penguin);
	measured.measure_floor(1.45);
	sweep(&mut measured, &room, -60.0, 60.0, -30.0);
	for bearing in [-30.0, 30.0] {
		let slice = slice_at(bearing);
		assert_close(
			map.reach(slice).unwrap(),
			measured.reach(slice).unwrap(),
			0.1,
		);
	}
}

#[test]
fn estimated_air_is_calibrated_to_the_tracked_floor_too() {
	let room = DrawnRoom::default();
	let mut measured = RoomMap::new(Vessel::Bird);
	measured.measure_floor(1.45);
	sweep(&mut measured, &room, -60.0, 60.0, -20.0);
	for stretch in [0.7, 1.3] {
		let mut estimated = RoomMap::new(Vessel::Bird);
		estimated.measure_floor(1.45);
		for frame in pictures(
			&room,
			-60.0,
			60.0,
			-20.0,
			Sweep {
				stretch,
				..Sweep::default()
			},
		) {
			assert!(estimated.add_estimated(&frame));
		}
		for bearing in [-30.0, 0.0, 30.0] {
			let slice = slice_at(bearing);
			assert_close(
				estimated.reach(slice).unwrap(),
				measured.reach(slice).unwrap(),
				0.1,
			);
		}
	}
}
