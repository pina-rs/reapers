//! The heavy swings, as keys from the press: a wind-up, a strike that
//! sweeps about 120 degrees in 180 ms, and a recovery. The Flutter
//! `_swingShapes`.

use super::arms::CONTACT;
use super::arms::STRIKE_SECONDS;
use super::arms::Swing;
use super::arms::WIND_UP;
use super::curves::Ease;
use super::curves::Keys;
use super::curves::eased;
use super::curves::key;

/// One swing's curves, from the press.
pub struct SwingShape {
	/// When it is back at rest.
	pub end: f32,
	pub x: Keys,
	pub y: Keys,
	pub z: Keys,
	pub pitch: Keys,
	pub yaw: Keys,
	pub roll: Keys,
	/// Whether its head draws a trail.
	pub trail: bool,
}

const STILL: Keys = &[key(0.0, 0.0)];
const STRIKE: f32 = WIND_UP + STRIKE_SECONDS;
const SETTLED: f32 = 0.62;
const TOLLED: f32 = SETTLED + 0.2;

// The strike is eased in and out: fastest, and halfway round, as it lands
// at the contact, so the blade is over the husk when the hit stops it.
const SWEEP: SwingShape = SwingShape {
	end: SETTLED,
	x: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.07),
		eased(STRIKE, -0.2, Ease::InOut),
		key(SETTLED, 0.0),
	],
	y: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.04),
		eased(STRIKE, 0.02, Ease::InOut),
		key(SETTLED, 0.0),
	],
	z: STILL,
	pitch: STILL,
	// From 1.25 to -0.85 radians: 120 degrees in the strike, the head in
	// the middle of the view halfway.
	yaw: &[
		key(0.0, 0.0),
		eased(WIND_UP, 1.25, Ease::Out),
		eased(STRIKE, -0.85, Ease::InOut),
		key(SETTLED, 0.0),
	],
	roll: &[
		key(0.0, 0.0),
		key(WIND_UP, -0.3),
		eased(STRIKE, 0.5, Ease::InOut),
		key(SETTLED, 0.0),
	],
	trail: true,
};

const THRUST: SwingShape = SwingShape {
	end: SETTLED,
	x: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.02),
		eased(STRIKE, -0.06, Ease::Out),
		key(SETTLED, 0.0),
	],
	y: STILL,
	z: &[
		key(0.0, 0.0),
		key(WIND_UP, -0.12),
		eased(CONTACT, 0.32, Ease::Out),
		key(STRIKE, 0.3),
		key(SETTLED, 0.0),
	],
	pitch: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.1),
		eased(STRIKE, -0.08, Ease::Out),
		key(SETTLED, 0.0),
	],
	yaw: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.1),
		key(STRIKE, -0.15),
		key(SETTLED, 0.0),
	],
	roll: STILL,
	trail: false,
};

const TOLL: SwingShape = SwingShape {
	end: TOLLED,
	x: STILL,
	y: &[
		key(0.0, 0.0),
		eased(WIND_UP, 0.14, Ease::Out),
		eased(STRIKE, -0.04, Ease::In),
		key(TOLLED, 0.0),
	],
	z: &[
		key(0.0, 0.0),
		key(WIND_UP, -0.04),
		key(STRIKE, 0.1),
		key(TOLLED, 0.0),
	],
	pitch: &[
		key(0.0, 0.0),
		eased(WIND_UP, 1.05, Ease::Out),
		eased(STRIKE, -1.05, Ease::In),
		key(TOLLED, 0.0),
	],
	yaw: STILL,
	roll: &[
		key(0.0, 0.0),
		key(STRIKE, 0.0),
		key(STRIKE + 0.06, 0.16),
		key(STRIKE + 0.14, -0.12),
		key(STRIKE + 0.22, 0.08),
		key(TOLLED, 0.0),
	],
	trail: true,
};

const LASH: SwingShape = SwingShape {
	end: SETTLED,
	x: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.06),
		eased(STRIKE, -0.17, Ease::OutBack),
		key(SETTLED, 0.0),
	],
	y: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.09),
		eased(STRIKE, -0.06, Ease::OutBack),
		key(SETTLED, 0.0),
	],
	z: STILL,
	pitch: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.4),
		eased(STRIKE, -0.5, Ease::OutBack),
		key(SETTLED, 0.0),
	],
	yaw: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.5),
		eased(STRIKE, -0.9, Ease::OutBack),
		key(SETTLED, 0.0),
	],
	roll: &[
		key(0.0, 0.0),
		key(WIND_UP, -0.7),
		eased(STRIKE, 1.4, Ease::OutBack),
		key(SETTLED, 0.0),
	],
	trail: true,
};

const CHOP: SwingShape = SwingShape {
	end: SETTLED,
	x: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.04),
		eased(STRIKE, -0.12, Ease::Out),
		key(SETTLED, 0.0),
	],
	y: &[
		key(0.0, 0.0),
		key(WIND_UP, 0.12),
		eased(STRIKE, -0.07, Ease::Out),
		key(SETTLED, 0.0),
	],
	z: STILL,
	pitch: &[
		key(0.0, 0.0),
		eased(WIND_UP, 1.1, Ease::Out),
		eased(STRIKE, -0.95, Ease::Out),
		key(SETTLED, 0.0),
	],
	yaw: STILL,
	roll: &[
		key(0.0, 0.0),
		key(WIND_UP, -0.25),
		eased(STRIKE, 0.55, Ease::Out),
		key(SETTLED, 0.0),
	],
	trail: true,
};

const SHOOT: SwingShape = SwingShape {
	end: 0.45,
	x: STILL,
	y: &[
		key(0.0, 0.0),
		key(0.08, 0.05),
		eased(0.16, -0.04, Ease::Out),
		key(0.45, 0.0),
	],
	z: STILL,
	pitch: &[
		key(0.0, 0.0),
		key(0.08, 0.6),
		eased(0.16, -0.5, Ease::Out),
		key(0.45, 0.0),
	],
	yaw: STILL,
	roll: STILL,
	trail: false,
};

const STAB: SwingShape = SwingShape {
	end: 0.45,
	x: &[
		key(0.0, 0.0),
		key(0.08, 0.04),
		eased(0.16, -0.1, Ease::Out),
		key(0.45, 0.0),
	],
	y: STILL,
	z: STILL,
	pitch: STILL,
	yaw: &[
		key(0.0, 0.0),
		key(0.08, 0.5),
		eased(0.16, -0.8, Ease::Out),
		key(0.45, 0.0),
	],
	roll: &[key(0.0, 0.0), eased(0.16, 0.6, Ease::Out), key(0.45, 0.0)],
	trail: false,
};

/// The curves of `manner`.
pub const fn shape(manner: Swing) -> &'static SwingShape {
	match manner {
		Swing::Sweep => &SWEEP,
		Swing::Thrust => &THRUST,
		Swing::Toll => &TOLL,
		Swing::Lash => &LASH,
		Swing::Chop => &CHOP,
		Swing::Shoot => &SHOOT,
		Swing::Stab => &STAB,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::render::curves::sample;

	#[test]
	fn the_sweep_turns_120_degrees_in_180_ms() {
		let turned = sample(SWEEP.yaw, WIND_UP) - sample(SWEEP.yaw, STRIKE);
		assert!(
			(turned.to_degrees() - 120.3).abs() < 1.0,
			"{}",
			turned.to_degrees()
		);
		assert!((STRIKE - WIND_UP - 0.18).abs() < 1.0e-6);
	}

	#[test]
	fn every_swing_comes_back_to_rest() {
		for manner in [
			Swing::Sweep,
			Swing::Thrust,
			Swing::Toll,
			Swing::Lash,
			Swing::Chop,
			Swing::Shoot,
			Swing::Stab,
		] {
			let shape = shape(manner);
			for curve in [
				shape.x,
				shape.y,
				shape.z,
				shape.pitch,
				shape.yaw,
				shape.roll,
			] {
				assert_eq!(sample(curve, shape.end + 0.01), 0.0, "{manner:?}");
			}
		}
	}
}
