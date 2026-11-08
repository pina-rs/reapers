//! What the tests share: comparing numbers.

#![allow(dead_code, reason = "each test file uses some of it")]

use reapers_rules::Vec3;

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
