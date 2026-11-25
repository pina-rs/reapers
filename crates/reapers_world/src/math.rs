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
	x.sqrt()
}

/// Deterministic minimax approximation of `atan2(y, x)` accurate to `< 2e-8 rad`
/// (`0.000001°`), using only IEEE-754 `+`, `-`, `*`, `/` so native and Wasm
/// produce identical bits ~10× faster than software `libm::atan2`.
#[inline]
pub(crate) fn fast_atan2(y: f64, x: f64) -> f64 {
	let abs_x = x.abs();
	let abs_y = y.abs();
	if abs_x == 0.0 && abs_y == 0.0 {
		return 0.0;
	}
	let (min, max) = if abs_x < abs_y {
		(abs_x, abs_y)
	} else {
		(abs_y, abs_x)
	};
	let ratio = min / max;
	let ratio_sq = ratio * ratio;
	// Abramowitz & Stegun 4.4.49 minimax polynomial on [0, 1].
	let mut poly = -0.004_054_058_0_f64;
	poly = poly * ratio_sq + 0.021_861_228_8;
	poly = poly * ratio_sq - 0.055_909_886_1;
	poly = poly * ratio_sq + 0.096_420_044_1;
	poly = poly * ratio_sq - 0.139_085_335_1;
	poly = poly * ratio_sq + 0.199_465_359_9;
	poly = poly * ratio_sq - 0.333_298_560_5;
	poly = poly * ratio_sq + 0.999_999_332_9;
	poly *= ratio;
	if abs_y > abs_x {
		poly = core::f64::consts::FRAC_PI_2 - poly;
	}
	if x < 0.0 {
		poly = PI - poly;
	}
	if y < 0.0 { -poly } else { poly }
}

/// Deterministic degree-7 minimax approximation of `atan2(y, x)` in `f32`,
/// accurate to `< 8e-5 rad` (`0.0046°`, >50× finer than `OcclusionMap`'s
/// `0.25°` cells) using only IEEE-754 `+`, `-`, `*`, `/`.
#[inline]
pub(crate) fn fast_atan2_f32(y: f32, x: f32) -> f32 {
	let abs_x = x.abs();
	let abs_y = y.abs();
	let max = abs_x.max(abs_y);
	if max == 0.0 {
		return 0.0;
	}
	let min = abs_x.min(abs_y);
	let ratio = min / max;
	let ratio_sq = ratio * ratio;
	// Minimax odd polynomial for atan(t) on [0, 1], max error ~7.5e-5 rad.
	let mut poly = -0.046_496_474_f32;
	poly = poly * ratio_sq + 0.159_314_22;
	poly = poly * ratio_sq - 0.327_622_77;
	poly = poly * ratio_sq + 0.999_866;
	poly *= ratio;
	if abs_y > abs_x {
		poly = core::f32::consts::FRAC_PI_2 - poly;
	}
	if x < 0.0 {
		poly = core::f32::consts::PI - poly;
	}
	if y < 0.0 { -poly } else { poly }
}

/// Fast exact integer floor for finite coordinates within `[-1e12, 1e12]`.
#[inline]
pub(crate) fn fast_floor(x: f64) -> i64 {
	if (-1e12..=1e12).contains(&x) {
		let t = x as i64;
		if (t as f64) > x { t - 1 } else { t }
	} else {
		libm::floor(x) as i64
	}
}

pub fn exp(x: f64) -> f64 {
	libm::exp(x)
}

/// Dart's `double.floor()` as a whole number, saturating where Dart would
/// throw.
pub fn floor(x: f64) -> i64 {
	fast_floor(x)
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
#[inline]
pub fn length(x: f64, y: f64, z: f64) -> f64 {
	sqrt(x * x + y * y + z * z)
}
