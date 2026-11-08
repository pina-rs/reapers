//! Moments in time, as the rules are handed them.
//!
//! The rules read no clock. Whoever calls them says what time it is, as a
//! [`Timestamp`], and that is the only time there is.

use core::fmt;
use core::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::FormatError;

/// A moment, in microseconds since 1970 began in UTC.
///
/// Saves write it as ISO 8601 text, as they always have:
/// `2026-10-01T12:00:00.000Z`, with three more digits when there are
/// microseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Timestamp {
	micros: i64,
}

const MICROS_PER_SECOND: i64 = 1_000_000;
const MICROS_PER_DAY: i64 = 86_400 * MICROS_PER_SECOND;

impl Timestamp {
	/// The moment `micros` microseconds after 1970 began.
	pub const fn from_micros(micros: i64) -> Self {
		Self { micros }
	}

	/// The moment `millis` milliseconds after 1970 began.
	pub const fn from_millis(millis: i64) -> Self {
		Self {
			micros: millis.saturating_mul(1000),
		}
	}

	/// The moment `seconds` seconds after 1970 began.
	pub const fn from_unix_seconds(seconds: i64) -> Self {
		Self {
			micros: seconds.saturating_mul(MICROS_PER_SECOND),
		}
	}

	/// A moment on the calendar, in UTC. Out-of-range parts roll over into
	/// the next: month 13 is January of the year after.
	pub fn utc(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64) -> Self {
		let (year, month) = (
			year + (month - 1).div_euclid(12),
			(month - 1).rem_euclid(12) + 1,
		);
		let days = days_from_civil(year, month, 1) + day - 1;
		let seconds = ((days * 24 + hour) * 60 + minute) * 60 + second;

		Self::from_unix_seconds(seconds)
	}

	/// Microseconds since 1970 began.
	pub const fn micros(self) -> i64 {
		self.micros
	}

	/// Milliseconds since 1970 began, rounded toward zero.
	pub const fn millis(self) -> i64 {
		self.micros / 1000
	}

	/// Whole seconds since 1970 began, rounded toward zero: what a receipt
	/// records.
	pub const fn unix_seconds(self) -> i64 {
		self.millis() / 1000
	}

	/// This moment `seconds` later.
	#[must_use]
	pub const fn plus_seconds(self, seconds: i64) -> Self {
		Self {
			micros: self
				.micros
				.saturating_add(seconds.saturating_mul(MICROS_PER_SECOND)),
		}
	}

	/// This moment `duration` later.
	#[must_use]
	pub fn plus(self, duration: Duration) -> Self {
		let micros = i64::try_from(duration.as_micros()).unwrap_or(i64::MAX);

		Self {
			micros: self.micros.saturating_add(micros),
		}
	}

	/// How long after `earlier` this is, in microseconds: negative when it
	/// came first.
	pub const fn micros_since(self, earlier: Self) -> i64 {
		self.micros.saturating_sub(earlier.micros)
	}

	pub const fn is_before(self, other: Self) -> bool {
		self.micros < other.micros
	}

	/// Reads ISO 8601 text: a date, optionally a time with fractional
	/// seconds, and optionally `Z` or an offset. Text with no offset is
	/// taken as UTC, never as the device's local time.
	pub fn parse(text: &str) -> Result<Self, FormatError> {
		parse_iso(text).ok_or_else(|| FormatError::with_input("Invalid date format", text))
	}

	/// The moment as ISO 8601 text in UTC, as saves write it.
	pub fn to_iso8601(self) -> String {
		let days = self.micros.div_euclid(MICROS_PER_DAY);
		let within = self.micros.rem_euclid(MICROS_PER_DAY);
		let (year, month, day) = civil_from_days(days);
		let seconds = within / MICROS_PER_SECOND;
		let fraction = within % MICROS_PER_SECOND;
		let year = if (0..=9999).contains(&year) {
			format!("{year:04}")
		} else {
			format!("{}{:06}", if year < 0 { '-' } else { '+' }, year.abs())
		};
		let fraction = if fraction % 1000 == 0 {
			format!("{:03}", fraction / 1000)
		} else {
			format!("{fraction:06}")
		};

		format!(
			"{year}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{fraction}Z",
			seconds / 3600,
			seconds / 60 % 60,
			seconds % 60,
		)
	}
}

impl fmt::Display for Timestamp {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str(&self.to_iso8601())
	}
}

impl Serialize for Timestamp {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_iso8601())
	}
}

impl<'de> Deserialize<'de> for Timestamp {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let text = String::deserialize(deserializer)?;

		Self::parse(&text).map_err(serde::de::Error::custom)
	}
}

/// Days since 1970-01-01 of a date on the proleptic Gregorian calendar.
const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
	let year = if month <= 2 { year - 1 } else { year };
	let era = year.div_euclid(400);
	let of_era = year - era * 400;
	let of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
	let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;

	era * 146_097 + of_cycle - 719_468
}

/// The date `days` after 1970-01-01.
const fn civil_from_days(days: i64) -> (i64, i64, i64) {
	let days = days + 719_468;
	let era = days.div_euclid(146_097);
	let of_era = days - era * 146_097;
	let of_cycle = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
	let of_year = of_era - (365 * of_cycle + of_cycle / 4 - of_cycle / 100);
	let shifted = (5 * of_year + 2) / 153;
	let day = of_year - (153 * shifted + 2) / 5 + 1;
	let month = if shifted < 10 {
		shifted + 3
	} else {
		shifted - 9
	};
	let year = of_cycle + era * 400 + if month <= 2 { 1 } else { 0 };

	(year, month, day)
}

/// A cursor over ISO 8601 text.
struct Reader<'a> {
	text: &'a [u8],
	at: usize,
}

impl Reader<'_> {
	fn peek(&self) -> Option<u8> {
		self.text.get(self.at).copied()
	}

	fn eat(&mut self, byte: u8) -> bool {
		if self.peek() == Some(byte) {
			self.at += 1;
			true
		} else {
			false
		}
	}

	fn digits(&mut self, count: usize) -> Option<i64> {
		let digits = self.text.get(self.at..self.at + count)?;

		if !digits.iter().all(u8::is_ascii_digit) {
			return None;
		}

		self.at += count;

		Some(
			digits
				.iter()
				.fold(0, |value, digit| value * 10 + i64::from(digit - b'0')),
		)
	}
}

fn parse_iso(text: &str) -> Option<Timestamp> {
	let mut reader = Reader {
		text: text.trim().as_bytes(),
		at: 0,
	};
	let year = match reader.peek()? {
		sign @ (b'+' | b'-') => {
			reader.at += 1;
			let year = reader.digits(6)?;
			if sign == b'-' { -year } else { year }
		}
		_ => reader.digits(4)?,
	};
	let dashed = reader.eat(b'-');
	let month = reader.digits(2)?;
	if dashed && !reader.eat(b'-') {
		return None;
	}
	let day = reader.digits(2)?;

	if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
		return None;
	}

	let (mut hour, mut minute, mut second, mut micros) = (0, 0, 0, 0);

	if reader.eat(b'T') || reader.eat(b't') || reader.eat(b' ') {
		hour = reader.digits(2)?;
		let colons = reader.eat(b':');
		if let Some(value) = reader.digits(2) {
			minute = value;
			if colons {
				reader.eat(b':');
			}
			if let Some(value) = reader.digits(2) {
				second = value;
				if reader.eat(b'.') || reader.eat(b',') {
					let start = reader.at;
					while reader.peek().is_some_and(|byte| byte.is_ascii_digit()) {
						reader.at += 1;
					}
					let fraction = &reader.text[start..reader.at];
					if fraction.is_empty() {
						return None;
					}
					for place in 0..6 {
						let digit = fraction
							.get(place)
							.map_or(0, |digit| i64::from(digit - b'0'));
						micros = micros * 10 + digit;
					}
				}
			}
		}
	}

	if hour > 24 || minute > 59 || second > 59 {
		return None;
	}

	let mut offset = 0;

	match reader.peek() {
		None => {}
		Some(b'Z' | b'z') => reader.at += 1,
		Some(sign @ (b'+' | b'-')) => {
			reader.at += 1;
			let hours = reader.digits(2)?;
			reader.eat(b':');
			let minutes = reader.digits(2).unwrap_or(0);
			offset = (hours * 60 + minutes) * 60 * if sign == b'-' { -1 } else { 1 };
		}
		Some(_) => return None,
	}

	if reader.peek().is_some() {
		return None;
	}

	let days = days_from_civil(year, month, day);
	let seconds = ((days * 24 + hour) * 60 + minute) * 60 + second - offset;

	Some(Timestamp::from_micros(
		seconds
			.checked_mul(MICROS_PER_SECOND)?
			.checked_add(micros)?,
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn writes_and_reads_what_dart_wrote() {
		let at = Timestamp::utc(2026, 10, 1, 12, 0, 0);

		assert_eq!(at.to_iso8601(), "2026-10-01T12:00:00.000Z");
		assert_eq!(Timestamp::parse("2026-10-01T12:00:00.000Z"), Ok(at));
		assert_eq!(Timestamp::parse("2026-10-01T13:00:00+01:00"), Ok(at));
		assert_eq!(Timestamp::parse("2026-10-01 12:00:00"), Ok(at));

		let fine = Timestamp::from_micros(at.micros() + 1_234_567);

		assert_eq!(fine.to_iso8601(), "2026-10-01T12:00:01.234567Z");
		assert_eq!(Timestamp::parse(&fine.to_iso8601()), Ok(fine));
		assert_eq!(
			Timestamp::from_micros(-1).to_iso8601(),
			"1969-12-31T23:59:59.999999Z"
		);
		assert!(Timestamp::parse("yesterday").is_err());
	}
}
