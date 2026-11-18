//! Places on the earth, and where a phone says it is.

use core::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::math;
use crate::rift::rift_rules::RiftRules;
use crate::time::Timestamp;
use crate::vec3::degrees;
use crate::vec3::radians;
use crate::vec3::wrap_angle;

/// A place on the earth, in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
	/// Degrees north of the equator.
	pub latitude: f64,

	/// Degrees east of Greenwich.
	pub longitude: f64,
}

impl GeoPoint {
	/// The earth's mean radius, in metres. The distances the game deals in
	/// are tens of metres, over which the earth is as good as a sphere.
	pub const EARTH_RADIUS: f64 = 6_371_008.8;

	pub const fn new(latitude: f64, longitude: f64) -> Self {
		Self {
			latitude,
			longitude,
		}
	}

	/// Whether these are finite coordinates on the earth.
	pub fn is_valid(&self) -> bool {
		self.latitude.is_finite()
			&& self.longitude.is_finite()
			&& self.latitude.abs() <= 90.0
			&& self.longitude.abs() <= 180.0
	}

	/// How far it is to `other` over the ground, in metres.
	pub fn distance_to(&self, other: &Self) -> f64 {
		let from_latitude = radians(self.latitude);
		let to_latitude = radians(other.latitude);
		let across_latitude = to_latitude - from_latitude;
		let across_longitude = radians(other.longitude - self.longitude);
		let half_latitude = math::sin(across_latitude / 2.0);
		let half_longitude = math::sin(across_longitude / 2.0);
		let chord = half_latitude * half_latitude
			+ math::cos(from_latitude) * math::cos(to_latitude) * (half_longitude * half_longitude);

		2.0 * Self::EARTH_RADIUS * math::asin(math::min(1.0, math::sqrt(chord)))
	}

	/// Which way `other` lies from here: radians clockwise from north.
	pub fn bearing_to(&self, other: &Self) -> f64 {
		let from_latitude = radians(self.latitude);
		let to_latitude = radians(other.latitude);
		let across_longitude = radians(other.longitude - self.longitude);

		wrap_angle(math::atan2(
			math::sin(across_longitude) * math::cos(to_latitude),
			math::cos(from_latitude) * math::sin(to_latitude)
				- math::sin(from_latitude) * math::cos(to_latitude) * math::cos(across_longitude),
		))
	}

	/// The place `metres` from here along `bearing`, in radians clockwise
	/// from north.
	#[must_use]
	pub fn moved(&self, bearing: f64, metres: f64) -> Self {
		let arc = metres / Self::EARTH_RADIUS;
		let from_latitude = radians(self.latitude);
		let to_latitude = math::asin(
			math::sin(from_latitude) * math::cos(arc)
				+ math::cos(from_latitude) * math::sin(arc) * math::cos(bearing),
		);
		let across_longitude = math::atan2(
			math::sin(bearing) * math::sin(arc) * math::cos(from_latitude),
			math::cos(arc) - math::sin(from_latitude) * math::sin(to_latitude),
		);

		Self::new(
			degrees(to_latitude),
			degrees(wrap_angle(radians(self.longitude) + across_longitude)),
		)
	}
}

impl fmt::Display for GeoPoint {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "GeoPoint({}, {})", self.latitude, self.longitude)
	}
}

/// One measured location. Times are supplied by the device; trust and
/// freshness are checked again by the server, with its own supplied time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoFix {
	pub at: GeoPoint,
	pub accuracy: f64,
	pub observed_at: Timestamp,
}

impl GeoFix {
	pub fn is_precise(&self) -> bool {
		self.at.is_valid()
			&& self.accuracy.is_finite()
			&& self.accuracy > 0.0
			&& self.accuracy <= RiftRules::LOCATION_ACCURACY_METRES
	}

	pub fn is_fresh_at(&self, now: Timestamp) -> bool {
		let age = now.micros_since(self.observed_at);

		age <= micros(RiftRules::LOCATION_MAX_AGE) && age >= -micros(RiftRules::LOCATION_CLOCK_SKEW)
	}

	pub fn is_usable_at(&self, now: Timestamp) -> bool {
		self.is_precise() && self.is_fresh_at(now)
	}

	/// Whether this reading should replace `before` in the player's books.
	///
	/// A later observation replaces an earlier one, so a delayed upload
	/// never moves a player back. Two observations of the same instant keep
	/// the more precise: a phone often tightens a fix without moving its
	/// timestamp, and a delayed coarser copy of that instant must not win.
	pub fn replaces(&self, before: &Self) -> bool {
		self.observed_at > before.observed_at
			|| (self.observed_at == before.observed_at && self.accuracy < before.accuracy)
	}
}

fn micros(duration: core::time::Duration) -> i64 {
	i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
}

/// Where a reaper is standing and which way their phone points, as the
/// phone itself measured it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoStanding {
	pub at: GeoPoint,

	/// The way the phone points, in radians clockwise from north by its
	/// compass.
	pub facing: f64,

	/// How far out `at` may be, in metres, by the phone's own reckoning.
	pub accuracy: f64,

	/// Older recordings may omit this. Live servers require a timestamp.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub observed_at: Option<Timestamp>,
}

impl GeoStanding {
	pub fn is_usable_at(&self, now: Timestamp) -> bool {
		self.facing.is_finite()
			&& self.observed_at.is_some_and(|observed_at| {
				GeoFix {
					at: self.at,
					accuracy: self.accuracy,
					observed_at,
				}
				.is_usable_at(now)
			})
	}
}
