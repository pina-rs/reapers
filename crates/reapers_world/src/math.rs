//! The transcendental functions this crate uses, from `libm`.
//!
//! As in the rules, every sine, arc tangent and square root comes from the
//! `libm` crate, which computes them the same way on every target. What the
//! room makes of a picture becomes a hunt's plan, and the server replays
//! hunts by their plans.

pub use core::f64::consts::PI;

pub fn sin(x: f64) -> f64 {
	libm::sin(x)
}

pub fn cos(x: f64) -> f64 {
	libm::cos(x)
}

pub fn tan(x: f64) -> f64 {
	libm::tan(x)
}

pub fn atan2(y: f64, x: f64) -> f64 {
	libm::atan2(y, x)
}

pub fn sqrt(x: f64) -> f64 {
	libm::sqrt(x)
}

pub fn exp(x: f64) -> f64 {
	libm::exp(x)
}

/// Dart's `double.floor()` as a whole number, saturating where Dart would
/// throw.
pub fn floor(x: f64) -> i64 {
	libm::floor(x) as i64
}

/// Dart's `double.ceil()` as a whole number.
pub fn ceil(x: f64) -> i64 {
	libm::ceil(x) as i64
}

/// Dart's `double.round()`: half away from zero, to a whole number.
pub fn round(x: f64) -> i64 {
	libm::round(x) as i64
}

/// The length of the vector (`x`, `y`, `z`).
pub fn length(x: f64, y: f64, z: f64) -> f64 {
	sqrt(x * x + y * y + z * z)
}
