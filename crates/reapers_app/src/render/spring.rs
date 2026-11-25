//! Motion that follows other motion, the same at any frame rate: a
//! critically damped spring that trails what it follows and settles without
//! a wobble.
//!
//! It is exact over the time it is stepped across, so a motion stepped
//! thirty, sixty or a hundred and twenty times a second lands in the same
//! place. It decides nothing: it is for show, never for the rules.

use std::f64::consts::TAU;

/// A critically damped spring following a target: it trails a target that
/// moves, never overshoots one that stops, and settles as fast as anything
/// that does not overshoot.
///
/// Each step is solved exactly for a target that moves in a straight line
/// from where it was at the last step to where it is now, so the frame rate
/// it is stepped at changes nothing but how finely that line follows the
/// target's own path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
	/// Where it is.
	pub value: f64,
	/// How fast it is moving, a second.
	pub velocity: f64,
	/// How stiff it is: its angular frequency, in radians a second.
	omega: f64,
	/// What it followed at its last step.
	target: Option<f64>,
}

impl Spring {
	/// A spring at rest at `value`, as stiff as one that would ring
	/// `frequency` times a second were it not damped. It lags a target
	/// moving at a steady speed by `speed / (π · frequency)`.
	pub fn new(frequency: f64, value: f64) -> Self {
		Self {
			value,
			velocity: 0.0,
			omega: TAU * frequency,
			target: None,
		}
	}

	/// Follows `target` for `delta` seconds, and answers where it got to. A
	/// step of no time, or of a time that is not a number, moves nothing but
	/// remembers the target.
	pub fn step(&mut self, target: f64, delta: f64) -> f64 {
		if !delta.is_finite() || delta <= 0.0 {
			self.target = Some(target);
			return self.value;
		}
		let from = self.target.unwrap_or(target);
		let slope = (target - from) / delta;
		let omega = self.omega;
		// A target moving at a steady `slope` is trailed by 2·slope/ω; what is
		// left over dies away as (c1 + c2·t)·e^(−ωt).
		let lag = 2.0 * slope / omega;
		let c1 = self.value - (from - lag);
		let c2 = self.velocity - slope + omega * c1;
		let decay = (-omega * delta).exp();
		self.value = target - lag + (c1 + c2 * delta) * decay;
		self.velocity = slope + (c2 - omega * (c1 + c2 * delta)) * decay;
		self.target = Some(target);
		self.value
	}

	/// Puts it at rest on `value` at once.
	pub fn settle(&mut self, value: f64) {
		self.value = value;
		self.velocity = 0.0;
		self.target = Some(value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Steps a spring after a target that swings back and forth and then
	/// stops, `rate` times a second, and answers where it is at each of the
	/// times in `at`.
	fn follow(rate: f64, at: &[f64]) -> Vec<(f64, f64)> {
		let target = |t: f64| (t.min(0.6) * 9.0).sin();
		let mut spring = Spring::new(4.0, 0.0);
		let mut shown = Vec::new();
		let steps = (1.0 * rate).round() as usize;
		for step in 1..=steps {
			let t = step as f64 / rate;
			spring.step(target(t), 1.0 / rate);
			if at.iter().any(|when| (when - t).abs() < 1e-9) {
				shown.push((spring.value, spring.velocity));
			}
		}
		shown
	}

	#[test]
	fn a_spring_lands_in_the_same_place_at_any_frame_rate() {
		let at = [0.2, 0.4, 0.6, 0.8, 1.0];
		let thirty = follow(30.0, &at);
		let sixty = follow(60.0, &at);
		let one_twenty = follow(120.0, &at);
		assert_eq!(thirty.len(), at.len());
		for ((slow, fast), fastest) in thirty.iter().zip(&sixty).zip(&one_twenty) {
			// A thirtieth of a second is a coarse line through a sine of
			// nine radians a second; the spring through it is within a
			// hundredth of the finer ones.
			assert!((slow.0 - fastest.0).abs() < 0.01, "{slow:?} {fastest:?}");
			assert!((fast.0 - fastest.0).abs() < 0.003, "{fast:?} {fastest:?}");
		}
	}

	#[test]
	fn a_spring_after_a_straight_line_is_exact_at_any_step() {
		// A target moving at a steady pace, and one that stops: a line, which
		// each step solves exactly, so no frame rate differs at all.
		let mut coarse = Spring::new(6.0, 0.0);
		let mut fine = Spring::new(6.0, 0.0);
		for step in 1..=30 {
			coarse.step(0.5 * f64::from(step) / 30.0, 1.0 / 30.0);
		}
		for step in 1..=120 {
			fine.step(0.5 * f64::from(step) / 120.0, 1.0 / 120.0);
		}
		assert!((coarse.value - fine.value).abs() < 1e-9);
		assert!((coarse.velocity - fine.velocity).abs() < 1e-9);
		// It trails a steady target by 2·speed/ω.
		let lag = 2.0 * 0.5 / (TAU * 6.0);
		assert!((0.5 - lag - fine.value).abs() < 1e-3, "{}", fine.value);
	}

	#[test]
	fn a_spring_settles_on_a_still_target_without_overshooting() {
		let mut spring = Spring::new(3.0, 0.0);
		let mut most = 0.0_f64;
		for _ in 0..120 {
			most = most.max(spring.step(1.0, 1.0 / 60.0));
		}
		assert!(most <= 1.0 + 1e-12, "critically damped: {most}");
		assert!((spring.value - 1.0).abs() < 1e-6);
		spring.step(2.0, 0.0);
		assert!((spring.value - 1.0).abs() < 1e-6, "no time, no move");
		spring.settle(-0.5);
		assert_eq!(spring.step(-0.5, 1.0 / 60.0), -0.5);
	}
}
