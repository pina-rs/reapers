//! The Rust side of `reapers_flutter/test/hunt/depth/room_cover_test.dart`.

mod support;

use std::collections::HashSet;

use reapers_rules::Difficulty;
use reapers_rules::Footing;
use reapers_rules::HuntBot;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntLog;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use reapers_rules::Vessel;
use reapers_rules::radians;
use reapers_world::Block;
use reapers_world::DrawnRoom;
use reapers_world::RoomMap;
use support::Sweep;
use support::assert_close;
use support::slice_at;
use support::sweep_with;

fn swept(room: &DrawnRoom, vessel: Vessel, pitch: f64) -> RoomMap {
	let mut map = RoomMap::new(vessel);

	sweep_with(
		&mut map,
		room,
		-70.0,
		70.0,
		pitch,
		Sweep {
			every: 10.0,
			..Sweep::default()
		},
	);

	map
}

/// The kitchen, swept for an ape.
fn kitchen() -> RoomMap {
	swept(&DrawnRoom::kitchen(), Vessel::Ape, -30.0)
}

/// A room with a block as wide as the counter, `height` tall, and the far
/// wall `wall` metres away.
fn counter_room(wall: f64, near: f64, far: f64, half_width: f64, height: f64) -> DrawnRoom {
	DrawnRoom {
		wall,
		blocks: vec![Block {
			left: -half_width,
			right: half_width,
			near,
			far,
			height,
		}],
		..DrawnRoom::default()
	}
}

mod what_stands_in_the_room {
	use super::*;

	#[test]
	fn a_counter_is_as_tall_as_it_is_and_its_top_is_a_ledge() {
		let counter = kitchen().furniture(slice_at(0.0)).unwrap();

		assert_close(counter.front, 2.2, 0.2);
		assert_close(counter.cover, 0.92, 0.08);
		assert!(counter.ledge);
	}

	#[test]
	fn there_is_room_to_hide_behind_it_short_of_the_wall() {
		let hide = kitchen().furniture(slice_at(0.0)).unwrap().hide.unwrap();

		assert!(hide > 2.8 + Vessel::Ape.half_width() - 0.15);
		assert!(hide < 4.5 - Vessel::Ape.half_width() + 0.1);
	}

	#[test]
	fn there_is_no_room_behind_a_counter_against_the_wall() {
		let map = swept(&counter_room(3.0, 2.2, 2.8, 1.6, 0.92), Vessel::Ape, -30.0);

		assert_eq!(map.furniture(slice_at(0.0)).unwrap().hide, None);
	}

	#[test]
	fn nothing_is_known_to_be_behind_a_cupboard_taller_than_the_eye() {
		let map = swept(&counter_room(4.5, 2.0, 2.6, 0.8, 2.1), Vessel::Ape, -30.0);
		let cupboard = map.furniture(slice_at(0.0)).unwrap();

		// Nothing above the eye is charted: it is as tall as a wall.
		assert!(cupboard.cover > 1.3);
		assert!(!cupboard.ledge);
		assert_eq!(cupboard.hide, None);
	}

	#[test]
	fn where_nothing_stands_the_way_ends_at_a_wall_no_ledge_no_room() {
		let map = swept(&DrawnRoom::walled(4.5), Vessel::Ape, -30.0);
		let wall = map.furniture(slice_at(0.0)).unwrap();

		assert_close(wall.front, 4.5, 0.3);
		assert!(wall.cover > 1.3);
		assert!(!wall.ledge);
		assert_eq!(wall.hide, None);
	}

	#[test]
	fn the_plan_carries_all_of_it_and_a_hunt_can_be_played_in_it() {
		let plan = kitchen().plan_within(0.0, radians(70.0));

		assert!(plan.is_well_formed());
		assert!(plan.knows_cover());
		assert_close(plan.cover_at(0.0), 0.92, 0.08);
		assert!(plan.ledge_at(slice_at(0.0)).is_some());
		assert!(plan.hide_at(slice_at(0.0)).unwrap() > plan.reach(0.0));

		let decoded = HuntLog::decode(
			&HuntLog {
				plan: plan.clone(),
				..HuntLog::new(Vec::new())
			}
			.encode(),
		)
		.unwrap();

		assert_eq!(decoded.plan, plan);
	}

	#[test]
	fn a_husk_in_the_kitchen_leaps_onto_the_counter_and_behind_it() {
		let plan = kitchen().plan_within(0.0, radians(70.0));
		let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
		let mut footings = HashSet::new();

		for seed in 1..=10 {
			let mut sim = HuntSim::new(
				seed,
				Vessel::Ape,
				Rarity::Epic,
				Rarity::Epic.ward(),
				&stats,
				plan.clone(),
				Difficulty::Normal,
			)
			.unwrap();
			let mut bot = HuntBot::human(&sim);

			while !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
				bot.step_once(&mut sim);
				footings.insert(sim.footing());
			}
		}

		for footing in Footing::ALL {
			assert!(footings.contains(&footing), "never {footing:?}");
		}
	}
}

mod for_a_husk_that_flies {
	use super::*;

	#[test]
	fn a_counter_top_is_somewhere_to_perch_and_it_keeps_in_front() {
		let map = swept(&DrawnRoom::kitchen(), Vessel::Bird, -12.0);
		let counter = map.furniture(slice_at(0.0)).unwrap();

		assert!(counter.ledge);
		assert_close(counter.cover, 0.92, 0.08);
		assert_eq!(counter.hide, None);
		assert_close(map.reach(slice_at(0.0)).unwrap(), counter.front, 1e-9);
		assert!(map.plan_within(0.0, radians(70.0)).is_well_formed());
	}
}
