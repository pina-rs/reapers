//! Points on the earth, and bearings and distances between them.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use reapers_rules::*;

/// London Bridge, where one of the rifts is.
const BRIDGE: GeoPoint = GeoPoint::new(51.5079, -0.0877);

fn standing(at: GeoPoint, facing_degrees: f64) -> GeoStanding {
	GeoStanding {
		at,
		facing: radians(facing_degrees),
		accuracy: 4.0,
		observed_at: None,
	}
}

fn close_to(actual: f64, expected: f64, delta: f64) -> bool {
	(actual - expected).abs() <= delta
}

/// A value written to JSON and read back.
fn sent<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
	serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}

fn entrant(name: &str) -> RiftEntrant {
	RiftEntrant {
		name: name.to_owned(),
		loadout: HuntLoadout::BORROWED,
	}
}

fn reporting(standing: GeoStanding) -> RiftReport {
	RiftReport {
		standing: Some(standing),
		..RiftReport::readiness(false)
	}
}

mod places_on_the_earth {
	use super::*;

	#[test]
	fn a_degree_of_latitude_is_about_111_km() {
		let distance = GeoPoint::new(51.0, 0.0).distance_to(&GeoPoint::new(52.0, 0.0));

		assert!(close_to(distance, 111_195.0, 60.0), "{distance}");
	}

	#[test]
	fn a_place_is_no_distance_from_itself() {
		assert_eq!(BRIDGE.distance_to(&BRIDGE), 0.0);
	}

	#[test]
	fn bearings_run_clockwise_from_north() {
		let north = degrees(BRIDGE.bearing_to(&GeoPoint::new(51.6, -0.0877)));
		let east = degrees(BRIDGE.bearing_to(&GeoPoint::new(51.5079, 0.1)));
		let south = degrees(BRIDGE.bearing_to(&GeoPoint::new(51.4, -0.0877))).abs();
		let west = degrees(BRIDGE.bearing_to(&GeoPoint::new(51.5079, -0.3)));

		assert!(close_to(north, 0.0, 1e-6), "{north}");
		assert!(close_to(east, 90.0, 0.2), "{east}");
		assert!(close_to(south, 180.0, 1e-6), "{south}");
		assert!(close_to(west, -90.0, 0.2), "{west}");
	}

	#[test]
	fn moving_somewhere_and_measuring_back_agree() {
		for bearing in [0.0, 37.0, 90.0, 180.0, -120.0] {
			for metres in [5.0, 25.0, 140.0, 2500.0] {
				let there = BRIDGE.moved(radians(bearing), metres);
				let distance = BRIDGE.distance_to(&there);

				assert!(
					close_to(distance, metres, metres * 1e-6 + 1e-6),
					"{metres} m at {bearing}°: {distance}"
				);
				assert!(
					wrap_angle(BRIDGE.bearing_to(&there) - radians(bearing)).abs() < 1e-6,
					"{metres} m at {bearing}°"
				);
			}
		}
	}

	#[test]
	fn a_place_survives_being_written_down() {
		let read = sent(&standing(BRIDGE, 33.0));

		assert_eq!(read.at, BRIDGE);
		assert!(close_to(read.facing, radians(33.0), 1e-12));
		assert_eq!(read.accuracy, 4.0);
	}
}

mod where_a_rift_stands {
	use super::*;

	const GARDEN: GeoPoint = GeoPoint::new(52.2, 0.12);

	#[test]
	fn at_its_site_for_a_reaper_who_has_come_to_it() {
		let nearby = BRIDGE.moved(1.0, 120.0);

		assert_eq!(
			RiftSite::LondonBridge.anchor_for(&standing(nearby, 0.0)),
			BRIDGE
		);
	}

	#[test]
	fn in_front_of_the_first_reaper_anywhere_else() {
		let anchor = RiftSite::LondonBridge.anchor_for(&standing(GARDEN, 70.0));

		assert!(close_to(
			GARDEN.distance_to(&anchor),
			RiftRules::PLANT_METRES,
			1e-3
		));
		assert!(close_to(degrees(GARDEN.bearing_to(&anchor)), 70.0, 1e-3));
	}

	#[test]
	fn the_first_reaper_to_say_where_they_are_decides_it_for_the_circle() {
		let mut room = RiftRoom::new(RiftSite::LondonBridge, 3);
		let first = room.join(entrant("One")).unwrap();
		let second = room.join(entrant("Two")).unwrap();

		assert_eq!(room.frame_for(first).unwrap().anchor, None);

		room.report(second, &reporting(standing(GARDEN, 70.0)));

		let anchor = room.anchor().unwrap();

		// Someone else, standing a few metres off and facing another way,
		// does not move it.
		room.report(first, &reporting(standing(GARDEN.moved(2.0, 6.0), -140.0)));

		assert_eq!(room.anchor(), Some(anchor));
		assert_eq!(room.frame_for(first).unwrap().anchor, Some(anchor));
		assert_eq!(room.frame_for(second).unwrap().anchor, Some(anchor));
	}

	#[test]
	fn it_does_not_move_once_the_fight_has_begun() {
		let mut room = RiftRoom::new(RiftSite::LondonBridge, 3);
		let seat = room.join(entrant("One")).unwrap();

		room.report(seat, &RiftReport::readiness(true));
		room.advance(RiftRules::COUNTDOWN_TICKS + 2);

		assert_eq!(room.phase(), RiftPhase::Fight);

		room.report(
			seat,
			&RiftReport {
				standing: Some(standing(GARDEN, 0.0)),
				..RiftReport::readiness(true)
			},
		);

		assert_eq!(room.anchor(), None);
	}

	#[test]
	fn a_frame_carries_it_across_the_wire() {
		let mut room = RiftRoom::new(RiftSite::LondonBridge, 3);
		let seat = room.join(entrant("One")).unwrap();

		room.report(seat, &reporting(standing(GARDEN, 0.0)));

		let frame = sent(&room.frame_for(seat).unwrap());
		let report = sent(&RiftReport {
			standing: Some(standing(GARDEN, 12.0)),
			..RiftReport::readiness(true)
		});

		assert_eq!(frame.anchor, room.anchor());
		assert_eq!(report.standing.unwrap().at, GARDEN);
		assert_eq!(sent(&RiftReport::readiness(true)).standing, None);
	}
}

mod where_a_reaper_sees_it {
	use super::*;

	const GARDEN: GeoPoint = GeoPoint::new(52.2, 0.12);

	fn anchor() -> GeoPoint {
		GARDEN.moved(radians(70.0), 25.0)
	}

	#[test]
	fn the_one_who_planted_it_sees_it_where_they_were_pointing() {
		// Their phone pointed 70° east of north, which in their own view was
		// straight ahead: so north is 70° to their left.
		let sighting = RiftSighting::of(&anchor(), &GARDEN, radians(-70.0)).unwrap();

		assert!(close_to(degrees(sighting.bearing), 0.0, 1e-3));
		assert!(close_to(sighting.metres, 25.0, 1e-3));
	}

	#[test]
	fn two_reapers_side_by_side_point_at_the_same_place() {
		let anchor = anchor();
		let beside = GARDEN.moved(radians(160.0), 3.0);

		// Each has north somewhere different in their own view, because each
		// started facing a different way.
		for north in [-70.0, 15.0, 170.0] {
			let one = RiftSighting::of(&anchor, &GARDEN, radians(north)).unwrap();
			let other = RiftSighting::of(&anchor, &beside, radians(north)).unwrap();

			// On the earth, the two directions meet at the rift.
			let meet = GARDEN.moved(one.bearing - radians(north), 25.0);
			let meet_other =
				beside.moved(other.bearing - radians(north), beside.distance_to(&anchor));

			assert!(meet.distance_to(&anchor) < 0.01, "{north}");
			assert!(meet_other.distance_to(&anchor) < 0.01, "{north}");
		}
	}

	#[test]
	fn from_across_the_rift_it_lies_the_other_way() {
		let beyond = GARDEN.moved(radians(70.0), 50.0);
		let sighting = RiftSighting::of(&anchor(), &beyond, 0.0).unwrap();

		assert!(close_to(degrees(sighting.bearing), -110.0, 0.01));
	}

	#[test]
	fn it_is_never_drawn_too_near_or_too_far_to_fight() {
		let anchor = anchor();
		let close = anchor.moved(0.0, 3.0);
		let far = anchor.moved(0.0, 120.0);

		assert_eq!(
			RiftSighting::of(&anchor, &close, 0.0).unwrap().metres,
			RiftRules::NEAREST_METRES
		);
		assert_eq!(
			RiftSighting::of(&anchor, &far, 0.0).unwrap().metres,
			RiftRules::FARTHEST_METRES
		);
	}

	#[test]
	fn a_reaper_in_another_town_is_not_there_at_all() {
		assert_eq!(RiftSighting::of(&anchor(), &BRIDGE, 0.0), None);
	}

	#[test]
	fn the_husk_stands_where_it_is_sighted_facing_the_reaper() {
		let sim = HuntSim::rift(
			3,
			Vessel::ReliquaryStack,
			1000.0,
			&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
			Some(RiftSighting {
				bearing: radians(40.0),
				metres: 30.0,
			}),
		);
		let pose = sim.pose();
		let flat = (pose.position.x * pose.position.x + pose.position.z * pose.position.z).sqrt();

		assert!(close_to(degrees(pose.yaw()), 40.0, 1e-9));
		assert!(close_to(flat, 30.0, 1e-9));
		assert!(pose.faces(reapers_rules::Vec3::ZERO));
	}

	#[test]
	fn with_no_sighting_it_stands_dead_ahead_as_before() {
		let sim = HuntSim::rift(
			3,
			Vessel::ReliquaryStack,
			1000.0,
			&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
			None,
		);

		assert_eq!(sim.pose().yaw(), 0.0);
		assert!(close_to(sim.pose().position.z, 17.0, 1e-9));
	}
}
