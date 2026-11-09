//! The Rust side of `reapers_flutter/test/hunt/depth/room_map_test.dart`.

mod support;

use reapers_rules::Difficulty;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntLog;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use reapers_rules::Vessel;
use reapers_rules::radians;
use reapers_world::Block;
use reapers_world::DrawnRoom;
use reapers_world::RoomMap;
use reapers_world::Shot;
use support::Sweep;
use support::assert_between;
use support::assert_close;
use support::slice_at;
use support::sweep;
use support::sweep_with;

/// The plan for a hunt kept to `degrees_either_side` of straight ahead.
fn ahead(map: &RoomMap, degrees_either_side: f64) -> HuntPlan {
	map.plan_within(0.0, radians(degrees_either_side))
}

/// A sofa: 80 cm tall, a metre and a half ahead and a little to the right.
const SOFA: Block = Block {
	left: 0.4,
	right: 2.0,
	near: 1.5,
	far: 2.3,
	height: 0.8,
};

fn sim(vessel: Vessel, plan: HuntPlan) -> HuntSim {
	HuntSim::new(
		5,
		vessel,
		Rarity::Common,
		Rarity::Common.ward(),
		&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
		plan,
		Difficulty::Normal,
	)
	.unwrap()
}

mod finding_the_floor {
	use super::*;

	#[test]
	fn standing_the_floor_is_where_it_really_is() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &DrawnRoom::default(), -60.0, 60.0, -30.0);

		assert_close(map.floor_depth().unwrap(), 1.45, 0.03);
	}

	#[test]
	fn sitting_down_the_floor_comes_up_to_meet_the_phone() {
		let mut map = RoomMap::new(Vessel::Penguin);
		let room = DrawnRoom {
			eye_height: 0.95,
			..DrawnRoom::default()
		};

		sweep(&mut map, &room, -60.0, 60.0, -30.0);

		assert_close(map.floor_depth().unwrap(), 0.95, 0.03);
		assert_between(f64::from(ahead(&map, 60.0).eye_centimetres), 92.0, 98.0);
	}

	#[test]
	fn a_table_top_is_not_taken_for_the_floor() {
		// A big table across the far side of the view, with floor in front.
		let room = DrawnRoom::with(&[Block {
			left: -2.0,
			right: 2.0,
			near: 1.8,
			far: 3.4,
			height: 0.75,
		}]);
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &room, -40.0, 40.0, -32.0);

		assert_close(map.floor_depth().unwrap(), 1.45, 0.03);
	}

	#[test]
	fn depth_that_wobbles_from_pixel_to_pixel_still_finds_it() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep_with(
			&mut map,
			&DrawnRoom::default(),
			-60.0,
			60.0,
			-30.0,
			Sweep {
				noise: 0.03,
				..Sweep::default()
			},
		);

		assert_close(map.floor_depth().unwrap(), 1.45, 0.06);
		assert!(map.reach(slice_at(0.0)).is_some());
	}

	#[test]
	fn a_picture_that_takes_a_table_for_the_floor_is_set_aside() {
		// A table right across the view, so near that no floor shows at all.
		let table = DrawnRoom::with(&[Block {
			left: -3.0,
			right: 3.0,
			near: 0.4,
			far: 4.0,
			height: 0.75,
		}]);
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &DrawnRoom::default(), -40.0, 20.0, -30.0);

		let before = map.frames();

		assert!(!map.add(&table.picture(40.0, -30.0)));
		assert_eq!(map.frames(), before);
		assert_close(map.floor_depth().unwrap(), 1.45, 0.03);
	}

	#[test]
	fn a_picture_with_no_floor_in_it_teaches_nothing() {
		// Level with a wall a metre away, there is no floor in view at all.
		let room = DrawnRoom::walled(1.0);
		let mut map = RoomMap::new(Vessel::Penguin);

		assert!(!map.add(&room.picture(0.0, 25.0)));
		assert_eq!(map.frames(), 0);
		assert_eq!(map.floor_depth(), None);
		assert!(!ahead(&map, 60.0).is_surveyed());
		assert_eq!(
			ahead(&map, 60.0).eye_centimetres,
			HuntRules::EYE_CENTIMETRES
		);
	}
}

mod how_far_the_way_is_clear {
	use super::*;

	#[test]
	fn an_open_floor_is_clear_as_far_as_the_wall() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &DrawnRoom::default(), -60.0, 60.0, -30.0);

		assert_close(map.reach(slice_at(0.0)).unwrap(), 4.0, 0.25);
		assert_eq!(map.reach(slice_at(-120.0)), None, "never looked there");
		assert_close(map.read(slice_at(-120.0)), 0.0, 0.0);
		assert!(!map.tight(slice_at(0.0)));
	}

	#[test]
	fn a_sofa_is_as_far_as_the_way_is_clear_in_its_direction() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &DrawnRoom::with(&[SOFA]), -150.0, 60.0, -30.0);

		// Straight at the sofa, the way is clear up to its front.
		assert_between(map.reach(slice_at(30.0)).unwrap(), 1.5, 1.9);
		// To the left of it, all the way to the wall.
		assert!(map.reach(slice_at(-40.0)).unwrap() > 3.5);
	}

	#[test]
	fn where_there_is_no_room_the_husk_may_still_go_and_stands_close() {
		// The sofa is too near for a penguin to keep its distance from.
		let mut map = RoomMap::new(Vessel::Penguin);
		let room = DrawnRoom::with(&[Block {
			left: 0.3,
			right: 2.0,
			near: 0.9,
			far: 2.0,
			height: 0.8,
		}]);

		sweep(&mut map, &room, -70.0, 70.0, -30.0);

		let plan = ahead(&map, 70.0);

		assert!(map.tight(slice_at(35.0)));
		assert!(!map.tight(slice_at(-40.0)));
		assert!(plan.contains(radians(35.0)));
		assert!(plan.reach(radians(35.0)) < 1.2);
		assert!(plan.is_well_formed());
	}

	#[test]
	fn with_room_in_front_of_it_the_husk_may_go_there_and_no_farther() {
		let room = DrawnRoom::with(&[Block {
			left: 0.4,
			right: 2.4,
			near: 2.6,
			far: 3.4,
			height: 0.8,
		}]);
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &room, -70.0, 70.0, -30.0);

		let plan = ahead(&map, 70.0);

		assert!(plan.contains(radians(25.0)));
		assert_between(plan.reach(radians(25.0)), 2.5, 3.0);
		// Well to the side of it there is more room, though the edge of what
		// was swept, where only the nearest floor was seen, holds it in a
		// little.
		assert!(plan.reach(radians(-40.0)) > 3.2);
	}

	#[test]
	fn a_model_that_reads_everything_long_still_lands_on_the_floor() {
		// Every distance 20% too long. The floor and the wall are both out by
		// the same share, so the wall is still where the picture shows it:
		// the same number of eye-heights away.
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

		let floor = map.floor_depth().unwrap();

		assert_close(floor, 1.45 * 1.2, 0.04);
		assert_close(map.reach(slice_at(0.0)).unwrap() / floor, 4.0 / 1.45, 0.2);
	}

	#[test]
	fn pictures_that_disagree_about_scale_are_brought_into_step() {
		// One picture reads long and the next reads short. Measured against
		// its own floor, each puts the wall the same number of eye-heights
		// away.
		let mut map = RoomMap::new(Vessel::Penguin);
		let room = DrawnRoom::default();

		for (yaw, stretch) in [
			(-40.0, 1.15),
			(-20.0, 0.9),
			(0.0, 1.1),
			(20.0, 0.92),
			(40.0, 1.0),
		] {
			map.add(&room.picture_with(
				yaw,
				-30.0,
				&Shot {
					stretch,
					..Shot::default()
				},
			));
		}

		for bearing in [-30.0_f64, 0.0, 30.0] {
			// The wall is 4 m ahead, so farther off to either side.
			let wall = 4.0 / radians(bearing).cos();
			let reach = map.reach(slice_at(bearing)).unwrap() / map.floor_depth().unwrap();

			assert!(
				(reach - wall / 1.45).abs() <= 0.3,
				"at {bearing}°: {reach} against {}",
				wall / 1.45
			);
		}
	}
}

mod the_plan {
	use super::*;

	#[test]
	fn it_keeps_to_the_arc_it_is_asked_for() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep(&mut map, &DrawnRoom::default(), -60.0, 60.0, -30.0);

		let plan = map.plan_within(radians(90.0), radians(65.0));

		assert_close(plan.centre(), radians(90.0), 1e-3);
		assert_close(plan.half_width(), radians(65.0), 1e-3);
	}

	#[test]
	fn a_direction_never_looked_in_is_left_open() {
		let mut map = RoomMap::new(Vessel::Penguin);

		map.add(&DrawnRoom::default().picture(0.0, -30.0));

		let plan = ahead(&map, 90.0);

		assert!(plan.is_surveyed());
		assert_eq!(
			plan.reach_decimetres[slice_at(-80.0)],
			HuntPlan::MAX_REACH_DECIMETRES
		);
		assert!(plan.reach_decimetres[slice_at(0.0)] < 50);
	}

	#[test]
	fn a_swept_room_gives_a_plan_a_hunt_can_be_played_in() {
		let mut map = RoomMap::new(Vessel::Ape);

		sweep(&mut map, &DrawnRoom::default(), -70.0, 70.0, -30.0);

		let plan = ahead(&map, 70.0);

		assert!(plan.is_surveyed());
		assert!(plan.is_well_formed());
		assert_between(f64::from(plan.eye_centimetres), 142.0, 148.0);
		assert!(plan.contains(radians(50.0)));
		assert!(plan.contains(radians(-50.0)));
		assert!(!plan.contains(radians(150.0)), "not asked for");

		// The rules take it, and it survives being written down and read
		// back.
		let log = HuntLog::decode(
			&HuntLog {
				plan: plan.clone(),
				..HuntLog::new(Vec::new())
			}
			.encode(),
		)
		.unwrap();

		assert_eq!(log.plan, plan);

		sim(Vessel::Ape, log.plan);
	}

	#[test]
	fn after_a_single_picture_a_hunt_still_gets_the_floor() {
		let mut map = RoomMap::new(Vessel::Penguin);

		assert_eq!(
			ahead(&map, 60.0).eye_centimetres,
			HuntRules::EYE_CENTIMETRES
		);

		let room = DrawnRoom {
			eye_height: 1.1,
			..DrawnRoom::default()
		};

		map.add(&room.picture(0.0, -30.0));

		assert_between(f64::from(ahead(&map, 60.0).eye_centimetres), 107.0, 113.0);
	}

	#[test]
	fn a_room_too_small_all_round_still_gives_a_plan_all_of_it_tight() {
		let mut map = RoomMap::new(Vessel::Ape);

		sweep(&mut map, &DrawnRoom::walled(1.1), -70.0, 70.0, -35.0);

		let plan = ahead(&map, 70.0);

		assert!(map.tight(slice_at(0.0)));
		assert!(plan.reach(0.0) < 1.3);

		// The rules take it, and the husk stands as close as it ever does.
		let position = sim(Vessel::Ape, plan).pose().position;
		let flat = (position.x * position.x + position.z * position.z).sqrt();

		assert_close(flat, HuntRules::CLOSEST_METRES, 1e-9);
	}
}

mod a_husk_that_flies {
	use super::*;

	#[test]
	fn is_given_the_air_as_far_as_the_walls() {
		let mut map = RoomMap::new(Vessel::Bird);

		sweep(&mut map, &DrawnRoom::walled(3.0), -70.0, 70.0, 5.0);

		let plan = ahead(&map, 70.0);

		assert_close(map.reach(slice_at(0.0)).unwrap(), 3.0, 0.2);
		assert!(plan.is_well_formed());
		assert_close(plan.reach(0.0), 3.0, 0.2);
	}

	#[test]
	fn is_kept_from_a_cupboard_that_reaches_up_into_its_air() {
		let room = DrawnRoom::with(&[Block {
			left: -0.6,
			right: 0.6,
			near: 1.3,
			far: 2.0,
			height: 2.2,
		}]);
		let mut map = RoomMap::new(Vessel::Bird);

		sweep(&mut map, &room, -130.0, 70.0, 5.0);

		assert_close(map.reach(slice_at(0.0)).unwrap(), 1.3, 0.15);
		assert!(!map.tight(slice_at(0.0)));
		assert_close(ahead(&map, 70.0).reach(0.0), 1.3, 0.2);
	}
}
