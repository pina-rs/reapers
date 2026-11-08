//! The Rust side of `reapers_shared/test/geo_fix_test.dart`.

#![allow(clippy::float_cmp, reason = "the Dart tests compare these exactly")]

mod support;

use core::time::Duration;

use reapers_rules::*;

fn now() -> Timestamp {
	Timestamp::utc(2026, 10, 5, 12, 0, 0)
}

fn fix() -> GeoFix {
	GeoFix {
		at: GeoPoint::new(51.5, -0.08),
		accuracy: 4.0,
		observed_at: now(),
	}
}

/// `at` moved `duration` earlier: Dart's `DateTime.subtract`.
fn minus(at: Timestamp, duration: Duration) -> Timestamp {
	Timestamp::from_micros(at.micros() - i64::try_from(duration.as_micros()).unwrap())
}

#[test]
fn measured_fix_survives_wire_and_database_serialization() {
	let fix = fix();
	let read: GeoFix = serde_json::from_value(serde_json::to_value(fix).unwrap()).unwrap();

	assert_eq!(read.at, fix.at);
	assert_eq!(read.accuracy, fix.accuracy);
	assert_eq!(read.observed_at, now());
	assert!(read.is_usable_at(now()));
}

#[test]
fn freshness_includes_the_boundary_and_tolerates_only_small_clock_skew() {
	let fix = fix();
	let tick = Duration::from_micros(1);

	assert!(fix.is_usable_at(now().plus(RiftRules::LOCATION_MAX_AGE)));
	assert!(!fix.is_usable_at(now().plus(RiftRules::LOCATION_MAX_AGE + tick)));
	assert!(fix.is_usable_at(minus(now(), RiftRules::LOCATION_CLOCK_SKEW)));
	assert!(!fix.is_usable_at(minus(now(), RiftRules::LOCATION_CLOCK_SKEW + tick)));
}

#[test]
fn coarse_missing_nonfinite_and_out_of_range_coordinates_are_unusable() {
	let fix = fix();

	for at in [
		GeoPoint::new(91.0, 0.0),
		GeoPoint::new(0.0, 181.0),
		GeoPoint::new(f64::NAN, 0.0),
		GeoPoint::new(0.0, f64::INFINITY),
	] {
		assert!(!GeoFix { at, ..fix }.is_usable_at(now()), "{at}");
	}

	for accuracy in [0.0, -1.0, f64::NAN, f64::INFINITY, 25.01] {
		assert!(
			!GeoFix { accuracy, ..fix }.is_usable_at(now()),
			"{accuracy}"
		);
	}

	assert!(
		GeoFix {
			accuracy: 25.0,
			..fix
		}
		.is_usable_at(now())
	);
}

#[test]
fn a_live_standing_needs_a_fresh_fix_and_a_finite_compass_bearing() {
	let fix = fix();
	let standing = |facing: f64, time: Option<Timestamp>| {
		GeoStanding {
			at: fix.at,
			accuracy: fix.accuracy,
			facing,
			observed_at: time,
		}
	};

	assert!(standing(0.0, Some(now())).is_usable_at(now()));
	assert!(!standing(f64::NAN, Some(now())).is_usable_at(now()));
	assert!(!standing(0.0, None).is_usable_at(now()));

	let read: GeoStanding =
		serde_json::from_value(serde_json::to_value(standing(1.0, Some(now()))).unwrap()).unwrap();

	assert_eq!(read.observed_at, Some(now()));
}
