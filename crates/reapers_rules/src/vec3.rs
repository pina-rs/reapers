//! The rules' own small vector, and the angle arithmetic around it.

use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Neg;
use core::ops::Sub;

use serde::Deserialize;
use serde::Serialize;

use crate::math::PI;
use crate::math::{self};

/// An immutable 3D vector in the hunt's space: `x` to the player's right,
/// `y` up and `z` straight ahead of where the player first faced.
///
/// The rules use their own small vector so that they depend on nothing else
/// and stay in double precision on every platform.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Vec3 {
	pub const FORWARD: Self = Self::new(0.0, 0.0, 1.0);
	pub const UP: Self = Self::new(0.0, 1.0, 0.0);
	pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

	pub const fn new(x: f64, y: f64, z: f64) -> Self {
		Self { x, y, z }
	}

	/// The unit vector that points `yaw` radians to the right of straight
	/// ahead and `pitch` radians above the horizon.
	pub fn from_angles(yaw: f64, pitch: f64) -> Self {
		let horizontal = math::cos(pitch);

		Self::new(
			math::sin(yaw) * horizontal,
			math::sin(pitch),
			math::cos(yaw) * horizontal,
		)
	}

	pub fn length(self) -> f64 {
		math::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
	}

	/// This vector scaled to length 1, or [`Vec3::FORWARD`] when it has no
	/// length.
	#[must_use]
	pub fn normalized(self) -> Self {
		let size = self.length();

		if size == 0.0 {
			Self::FORWARD
		} else {
			self / size
		}
	}

	/// How far to the right of straight ahead this vector points, in
	/// radians.
	pub fn yaw(self) -> f64 {
		math::atan2(self.x, self.z)
	}

	/// How far above the horizon this vector points, in radians.
	pub fn pitch(self) -> f64 {
		math::atan2(self.y, math::sqrt(self.x * self.x + self.z * self.z))
	}

	pub fn dot(self, other: Self) -> f64 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	/// The vector at right angles to both this and `other`.
	#[must_use]
	pub fn cross(self, other: Self) -> Self {
		Self::new(
			self.y * other.z - self.z * other.y,
			self.z * other.x - self.x * other.z,
			self.x * other.y - self.y * other.x,
		)
	}

	/// The angle between this vector and `other`, in radians.
	pub fn angle_to(self, other: Self) -> f64 {
		let cosine = self.normalized().dot(other.normalized());

		math::acos(math::clamp(cosine, -1.0, 1.0))
	}
}

impl Add for Vec3 {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
	}
}

impl Sub for Vec3 {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
	}
}

impl Mul<f64> for Vec3 {
	type Output = Self;

	fn mul(self, scale: f64) -> Self {
		Self::new(self.x * scale, self.y * scale, self.z * scale)
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;

	fn div(self, scale: f64) -> Self {
		Self::new(self.x / scale, self.y / scale, self.z / scale)
	}
}

impl Neg for Vec3 {
	type Output = Self;

	fn neg(self) -> Self {
		Self::new(-self.x, -self.y, -self.z)
	}
}

/// Wraps `radians` into `(-pi, pi]`.
pub fn wrap_angle(radians: f64) -> f64 {
	let mut wrapped = math::modulo(radians, 2.0 * PI);

	if wrapped > PI {
		wrapped -= 2.0 * PI;
	}

	wrapped
}

/// Degrees to radians.
pub const fn radians(degrees: f64) -> f64 {
	degrees * PI / 180.0
}

/// Radians to degrees.
pub const fn degrees(radians: f64) -> f64 {
	radians * 180.0 / PI
}

/// Hermite easing from 0 to 1 as `t` goes from 0 to 1.
pub fn smoothstep(t: f64) -> f64 {
	let clamped = math::clamp(t, 0.0, 1.0);

	clamped * clamped * (3.0 - 2.0 * clamped)
}
