//! Authored motion: a value at a few times and how it travels between
//! them, the way an animator keys a curve. The first-person arms and the
//! orb are animated with these, sampled at any moment, so a hit-stop can
//! hold a pose and a bench can freeze any frame.

/// How a curve moves from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ease {
	/// At an even pace.
	Linear,
	/// Slow away, slow in: a settle.
	InOut,
	/// Fast away, slowing into the key: a strike that lands.
	Out,
	/// Slow away, fast into the key: a wind-up let go.
	In,
	/// Past the key and back: a snap that overshoots.
	OutBack,
}

impl Ease {
	/// `t` from 0 to 1, eased.
	pub fn apply(self, t: f32) -> f32 {
		let t = t.clamp(0.0, 1.0);
		match self {
			Self::Linear => t,
			Self::InOut => t * t * (3.0 - 2.0 * t),
			Self::Out => 1.0 - (1.0 - t).powi(3),
			Self::In => t * t * t,
			Self::OutBack => 1.0 + 2.4 * (t - 1.0).powi(3) + 1.4 * (t - 1.0).powi(2),
		}
	}
}

/// One key: `value` at `time` seconds, reached from the key before with
/// `ease`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
	pub time: f32,
	pub value: f32,
	pub ease: Ease,
}

/// A key reached with a settle.
pub const fn key(time: f32, value: f32) -> Key {
	Key {
		time,
		value,
		ease: Ease::InOut,
	}
}

/// A key reached with `ease`.
pub const fn eased(time: f32, value: f32, ease: Ease) -> Key {
	Key { time, value, ease }
}

/// A curve: before its first key it holds the first value, after its last
/// the last. Keys are in order of time.
pub type Keys = &'static [Key];

/// The curve `keys` at `time`.
pub fn sample(keys: Keys, time: f32) -> f32 {
	let Some(first) = keys.first() else {
		return 0.0;
	};
	if time <= first.time {
		return first.value;
	}
	for pair in keys.windows(2) {
		let [from, to] = [pair[0], pair[1]];
		if time < to.time {
			let span = to.time - from.time;
			let share = if span <= 0.0 {
				1.0
			} else {
				(time - from.time) / span
			};
			return from.value + (to.value - from.value) * to.ease.apply(share);
		}
	}
	keys.last().map_or(0.0, |last| last.value)
}

/// When the curve's last key is.
pub fn end(keys: Keys) -> f32 {
	keys.last().map_or(0.0, |last| last.time)
}

/// The response of a critically damped spring knocked from rest, `since`
/// seconds after the knock: it rises to `peak` after `1 / rate` seconds and
/// settles back. A kick that recovers, without a spring to keep.
pub fn knock(peak: f32, rate: f32, since: f32) -> f32 {
	if since <= 0.0 {
		return 0.0;
	}
	let x = rate * since;
	peak * std::f32::consts::E * x * (-x).exp()
}

#[cfg(test)]
mod tests {
	use super::*;

	const RISE: Keys = &[key(0.0, 0.0), eased(1.0, 2.0, Ease::Out), key(2.0, 0.0)];

	#[test]
	fn a_curve_holds_its_ends_and_passes_through_its_keys() {
		assert_eq!(sample(RISE, -1.0), 0.0);
		assert_eq!(sample(RISE, 1.0), 2.0);
		assert_eq!(sample(RISE, 5.0), 0.0);
		assert_eq!(end(RISE), 2.0);
		// Easing out covers most of the way early.
		assert!(sample(RISE, 0.5) > 1.5);
	}

	#[test]
	fn every_ease_runs_from_nothing_to_all() {
		for ease in [
			Ease::Linear,
			Ease::InOut,
			Ease::Out,
			Ease::In,
			Ease::OutBack,
		] {
			assert!(ease.apply(0.0).abs() < 1.0e-6, "{ease:?}");
			assert!((ease.apply(1.0) - 1.0).abs() < 1.0e-6, "{ease:?}");
		}
		assert!(Ease::OutBack.apply(0.7) > 1.0, "a snap overshoots");
	}

	#[test]
	fn a_knock_peaks_once_and_settles() {
		let peak = knock(1.0, 25.0, 1.0 / 25.0);
		assert!((peak - 1.0).abs() < 1.0e-5);
		assert!(knock(1.0, 25.0, 0.3) < 0.05);
		assert_eq!(knock(1.0, 25.0, 0.0), 0.0);
	}
}
