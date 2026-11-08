//! The Rust side of `reapers_flutter/test/rift/rift_placer_test.dart`.

mod support;

use reapers_rules::GeoFix;
use reapers_rules::GeoPoint;
use reapers_rules::HuntLoadout;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftReport;
use reapers_rules::RiftRoom;
use reapers_rules::RiftRules;
use reapers_rules::RiftSighting;
use reapers_rules::RiftSite;
use reapers_rules::Timestamp;
use reapers_rules::degrees;
use reapers_rules::radians;
use reapers_rules::wrap_angle;
use reapers_world::RiftPlacer;
use support::assert_close;

const GARDEN: GeoPoint = GeoPoint::new(52.2, 0.12);

/// The time every phone here reads off its clock.
fn now() -> Timestamp {
	Timestamp::utc(2026, 10, 5, 12, 0, 0)
}

fn fix(at: GeoPoint) -> GeoFix {
	GeoFix {
		at,
		accuracy: 4.0,
		observed_at: now(),
	}
}

/// A phone in a rift: its placer, where it is, which way its owner first
/// faced (which is what its own view calls straight ahead), and which way it
/// points now.
struct Phone {
	placer: RiftPlacer,

	/// The compass bearing of what this phone's view calls straight ahead.
	first_faced_degrees: f64,

	/// The compass bearing the phone points along now.
	points_degrees: f64,
}

impl Phone {
	fn new(at: GeoPoint, first_faced_degrees: f64, points_degrees: f64) -> Self {
		let mut placer = RiftPlacer::new();

		placer.fix = Some(fix(at));

		Self {
			placer,
			first_faced_degrees,
			points_degrees,
		}
	}

	/// Where north lies in this phone's view.
	fn north(&self) -> f64 {
		radians(-self.first_faced_degrees)
	}

	/// Which way the phone points, in its own view.
	fn facing(&self) -> f64 {
		radians(self.points_degrees - self.first_faced_degrees)
	}

	/// Shows the phone the rift's frame for `seat`, and passes what it has
	/// to say on to `room`.
	fn see(&mut self, room: &mut RiftRoom, seat: usize) {
		let standing = self.placer.observe(
			&room.frame_for(seat).unwrap(),
			self.facing(),
			Some(self.north()),
			now(),
		);

		if let Some(standing) = standing {
			room.report(
				seat,
				&RiftReport {
					standing: Some(standing),
					..RiftReport::readiness(false)
				},
			);
		}
	}

	/// Where on the earth this phone would be pointing, were it turned to
	/// the husk as it sees it, at `anchor`'s distance.
	fn looking_at(&self, anchor: &GeoPoint) -> GeoPoint {
		let sighting = self.placer.sighting.unwrap();
		let here = self.placer.fix.unwrap().at;

		here.moved(sighting.bearing - self.north(), here.distance_to(anchor))
	}
}

fn room() -> RiftRoom {
	RiftRoom::new(RiftSite::LondonBridge, 3)
}

fn join(room: &mut RiftRoom, name: &str) -> usize {
	room.join(RiftEntrant {
		name: name.to_owned(),
		loadout: HuntLoadout::BORROWED,
	})
	.unwrap()
}

#[test]
fn a_stale_or_coarse_fix_cannot_place_the_shared_husk() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let frame = room.frame_for(seat).unwrap();
	let mut placer = RiftPlacer::new();
	let precise = fix(GARDEN);

	for fix in [
		GeoFix {
			accuracy: 80.0,
			..precise
		},
		GeoFix {
			observed_at: Timestamp::utc(2026, 10, 5, 11, 59, 29),
			..precise
		},
	] {
		placer.fix = Some(fix);

		assert_eq!(placer.observe(&frame, 0.0, Some(0.0), now()), None);
		assert_eq!(placer.sighting, None);
	}

	placer.fix = Some(precise);

	assert!(placer.observe(&frame, 0.0, Some(0.0), now()).is_some());
}

#[test]
fn the_first_phone_to_find_itself_plants_the_rift_ahead_of_it() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut phone = Phone::new(GARDEN, 20.0, 70.0);

	phone.see(&mut room, seat);

	let anchor = room.anchor().unwrap();

	assert_close(GARDEN.distance_to(&anchor), RiftRules::PLANT_METRES, 1e-3);
	assert_close(degrees(GARDEN.bearing_to(&anchor)), 70.0, 1e-3);
}

#[test]
fn it_then_sees_the_rift_where_it_was_pointing() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut phone = Phone::new(GARDEN, 20.0, 70.0);

	phone.see(&mut room, seat);
	phone.see(&mut room, seat);

	let sighting = phone.placer.sighting.unwrap();

	// It was pointing 50° to the right of where it first faced.
	assert_close(degrees(sighting.bearing), 50.0, 0.01);
	assert_close(sighting.metres, 25.0, 0.01);
}

#[test]
fn two_phones_side_by_side_end_up_pointing_at_the_same_spot() {
	let mut room = room();
	let first = join(&mut room, "One");
	let second = join(&mut room, "Two");
	let mut one = Phone::new(GARDEN, 20.0, 70.0);
	let mut other = Phone::new(GARDEN.moved(radians(150.0), 4.0), -110.0, 200.0);

	for _ in 0..3 {
		one.see(&mut room, first);
		other.see(&mut room, second);
	}

	let anchor = room.anchor().unwrap();

	assert!(one.looking_at(&anchor).distance_to(&anchor) < 0.05);
	assert!(other.looking_at(&anchor).distance_to(&anchor) < 0.05);
	// The second phone started out facing somewhere else entirely, so the
	// rift lies in a different part of its own view.
	assert!(
		wrap_angle(one.placer.sighting.unwrap().bearing - other.placer.sighting.unwrap().bearing)
			.abs()
			> radians(90.0)
	);
}

#[test]
fn once_the_rift_has_its_place_nobody_else_is_asked_for_theirs() {
	let mut room = room();
	let first = join(&mut room, "One");
	let second = join(&mut room, "Two");
	let mut one = Phone::new(GARDEN, 0.0, 0.0);
	let mut other = Phone::new(GARDEN.moved(1.0, 5.0), 0.0, 180.0);

	one.see(&mut room, first);

	let anchor = room.anchor();
	let (facing, north) = (other.facing(), other.north());

	assert_eq!(
		other
			.placer
			.observe(&room.frame_for(second).unwrap(), facing, Some(north), now()),
		None
	);
	assert_eq!(room.anchor(), anchor);
}

#[test]
fn a_phone_that_has_not_found_itself_says_so_then_stops_waiting() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut placer = RiftPlacer::new();

	assert!(placer.still_looking());

	for _ in 0..RiftPlacer::FRAMES_TO_FIND_ITSELF {
		assert_eq!(
			placer.observe(&room.frame_for(seat).unwrap(), 0.0, Some(0.0), now()),
			None
		);
	}

	assert!(!placer.still_looking());
	assert_eq!(placer.sighting, None);
	assert_eq!(room.anchor(), None);
}

#[test]
fn a_phone_with_no_compass_tells_the_rift_nothing() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut placer = RiftPlacer::new();

	placer.fix = Some(fix(GARDEN));

	assert_eq!(
		placer.observe(&room.frame_for(seat).unwrap(), 0.0, None, now()),
		None
	);
	assert_eq!(placer.sighting, None);
}

#[test]
fn it_tells_the_rift_again_only_every_half_second() {
	// A rift that never takes a place: nobody is listening on the other end.
	let mut room = room();
	let seat = join(&mut room, "One");
	let frame = room.frame_for(seat).unwrap();
	let mut placer = RiftPlacer::new();
	let mut told = 0;

	placer.fix = Some(fix(GARDEN));

	for _ in 0..3 * RiftPlacer::TELL_EVERY_FRAMES {
		if placer.observe(&frame, 0.0, Some(0.0), now()).is_some() {
			told += 1;
		}
	}

	assert_eq!(told, 3);
}

#[test]
fn a_fix_that_wanders_moves_the_sighting_gently() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut phone = Phone::new(GARDEN, 0.0, 0.0);

	phone.see(&mut room, seat);
	phone.see(&mut room, seat);

	let before = phone.placer.sighting.unwrap().bearing;

	// The phone now thinks it is five metres to the east.
	phone.placer.fix = Some(fix(GARDEN.moved(radians(90.0), 5.0)));
	phone.see(&mut room, seat);

	let after = phone.placer.sighting.unwrap().bearing;
	let whole = RiftSighting::of(
		&room.anchor().unwrap(),
		&phone.placer.fix.unwrap().at,
		phone.north(),
	)
	.unwrap()
	.bearing;

	assert!(wrap_angle(whole - before).abs() > radians(5.0));
	assert_close(
		wrap_angle(after - before).abs(),
		wrap_angle(whole - before).abs() * RiftPlacer::EASING,
		1e-6,
	);
}

#[test]
fn a_rift_in_another_town_is_seen_dead_ahead() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut frame = room.frame_for(seat).unwrap();
	let mut phone = Phone::new(GARDEN, 0.0, 0.0);

	frame.anchor = Some(RiftSite::LondonBridge.location());
	phone
		.placer
		.observe(&frame, phone.facing(), Some(phone.north()), now());

	assert_eq!(phone.placer.sighting, None);
}

#[test]
fn what_it_tells_the_rift_is_where_it_stands_and_which_way_it_points() {
	let mut room = room();
	let seat = join(&mut room, "One");
	let mut phone = Phone::new(GARDEN, 20.0, 70.0);
	let (facing, north) = (phone.facing(), phone.north());
	let standing = phone
		.placer
		.observe(&room.frame_for(seat).unwrap(), facing, Some(north), now())
		.unwrap();

	assert_eq!(standing.at, GARDEN);
	assert_close(degrees(standing.facing), 70.0, 1e-9);
	assert_close(standing.accuracy, 4.0, 0.0);
	assert_eq!(standing.observed_at, Some(now()));
	assert!(standing.is_usable_at(now()));
}
