//! Levels and what they are worth.

/// Levels and what they are worth.
pub struct Progression;

impl Progression {
	/// The highest level a reaper reaches.
	pub const LEVEL_CAP: i64 = 30;
	/// Each level adds this fraction of a level-one strike.
	pub const STRIKE_PER_LEVEL: f64 = 0.07;
	/// Each completed collection set adds this fraction to every strike.
	pub const STRIKE_PER_SET: f64 = 0.03;

	/// Experience needed to go from `level` to the next one.
	pub const fn xp_to_next(level: i64) -> i64 {
		let step = level - 1;

		90 + 55 * step + 6 * step * step
	}

	/// Total experience a reaper holds on reaching `level`.
	pub fn xp_for_level(level: i64) -> i64 {
		(1..level).map(Self::xp_to_next).sum()
	}

	/// The level reached with `xp` total experience.
	pub fn level_for_xp(xp: i64) -> i64 {
		let mut level = 1;
		let mut remaining = xp;

		while level < Self::LEVEL_CAP && remaining >= Self::xp_to_next(level) {
			remaining -= Self::xp_to_next(level);
			level += 1;
		}

		level
	}

	/// How far `xp` has carried a reaper through its current level, from 0
	/// to 1. A reaper at the cap is always at 1.
	pub fn level_progress(xp: i64) -> f64 {
		let level = Self::level_for_xp(xp);

		if level >= Self::LEVEL_CAP {
			return 1.0;
		}

		(xp - Self::xp_for_level(level)) as f64 / Self::xp_to_next(level) as f64
	}
}
