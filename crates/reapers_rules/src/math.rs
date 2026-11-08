//! The transcendental functions the rules use, and Dart's rounding.
//!
//! Every sine, cosine, arc tangent and square root in the rules goes through
//! here. By default they come from the `libm` crate, which computes them the
//! same way on every target: the server runs natively and the game runs as
//! Wasm, and a replay must land on the same bits on both. With the
//! `system-math` feature they come from the platform's own library instead,
//! which is what the Dart rules used.

#![allow(clippy::float_cmp, reason = "Dart's exact comparisons, kept exact")]

#[cfg(not(feature = "system-math"))]
mod imp {
	pub fn sin(x: f64) -> f64 {
		libm::sin(x)
	}
	pub fn cos(x: f64) -> f64 {
		libm::cos(x)
	}
	pub fn atan2(y: f64, x: f64) -> f64 {
		libm::atan2(y, x)
	}
	pub fn acos(x: f64) -> f64 {
		libm::acos(x)
	}
	pub fn asin(x: f64) -> f64 {
		libm::asin(x)
	}
	pub fn sqrt(x: f64) -> f64 {
		libm::sqrt(x)
	}
	pub fn floor(x: f64) -> f64 {
		libm::floor(x)
	}
	pub fn ceil(x: f64) -> f64 {
		libm::ceil(x)
	}
	pub fn round(x: f64) -> f64 {
		libm::round(x)
	}
}

#[cfg(feature = "system-math")]
mod imp {
	pub fn sin(x: f64) -> f64 {
		x.sin()
	}
	pub fn cos(x: f64) -> f64 {
		x.cos()
	}
	pub fn atan2(y: f64, x: f64) -> f64 {
		y.atan2(x)
	}
	pub fn acos(x: f64) -> f64 {
		x.acos()
	}
	pub fn asin(x: f64) -> f64 {
		x.asin()
	}
	pub fn sqrt(x: f64) -> f64 {
		x.sqrt()
	}
	// Rounding is exact everywhere, so it comes from libm either way.
	pub fn floor(x: f64) -> f64 {
		libm::floor(x)
	}
	pub fn ceil(x: f64) -> f64 {
		libm::ceil(x)
	}
	pub fn round(x: f64) -> f64 {
		libm::round(x)
	}
}

pub use imp::acos;
pub use imp::asin;
pub use imp::atan2;
pub use imp::cos;
pub use imp::sin;
pub use imp::sqrt;

/// Pi, as Dart's `math.pi`.
pub const PI: f64 = core::f64::consts::PI;

/// Dart's `double.round()`: half away from zero, to a whole number. Where
/// Dart would throw on a value that is not finite, this saturates.
pub fn round(x: f64) -> i64 {
	imp::round(x) as i64
}

/// Dart's `double.floor()`, saturating where Dart would throw.
pub fn floor_int(x: f64) -> i64 {
	imp::floor(x) as i64
}

/// Dart's `double.ceil()`, saturating where Dart would throw.
pub fn ceil_int(x: f64) -> i64 {
	imp::ceil(x) as i64
}

/// Dart's `num.clamp` on doubles. Unlike `f64::clamp` it never panics, and,
/// as in Dart, a NaN comes out as the upper limit.
pub fn clamp(x: f64, low: f64, high: f64) -> f64 {
	if x < low {
		low
	} else if x > high || x.is_nan() {
		high
	} else {
		x
	}
}

/// Dart's `math.max` on doubles: a NaN on either side wins, and positive
/// zero beats negative zero.
pub fn max(a: f64, b: f64) -> f64 {
	if a.is_nan() || b.is_nan() {
		f64::NAN
	} else if a > b || (a == b && b.is_sign_negative()) {
		a
	} else {
		b
	}
}

/// Dart's `math.min` on doubles: a NaN on either side wins, and negative
/// zero beats positive zero.
pub fn min(a: f64, b: f64) -> f64 {
	if a.is_nan() || b.is_nan() {
		f64::NAN
	} else if a < b || (a == b && a.is_sign_negative()) {
		a
	} else {
		b
	}
}

/// Dart's `%` on doubles, whose result is never negative.
pub fn modulo(x: f64, divisor: f64) -> f64 {
	let remainder = x % divisor;

	if remainder < 0.0 {
		remainder + divisor.abs()
	} else {
		remainder
	}
}
