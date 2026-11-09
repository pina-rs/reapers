//! The soul capture orb, from the throw to the seal or the break: where it
//! is, how open its talons are and how bright its light, at any moment.
//!
//! Its story is the rules' ([`OrbView`], on [`CaptureRules`]' ticks from the
//! release): flight to 27; at 27 it strikes and the husk is drawn in as
//! light until 48; it drops onto the ground under the husk and lands at 66;
//! it pulses at 66, 114 and 168; it holds its breath; and at 222 it seals. A
//! soul that breaks out does so on its pulse, in place of it. An orb that
//! misses comes down where it was thrown and goes out. [`look`] says what
//! any moment of that looks like.

use std::f32::consts::PI;
use std::f32::consts::TAU;

use reapers_rules::CaptureRules;
use reapers_rules::OrbStage;
use reapers_rules::OrbView;

use super::arms::TICKS_PER_SECOND;
use super::camera::Point;
use super::curves::Ease;
use super::curves::knock;
use super::turn::add;
use super::turn::lerp;
use super::turn::scale;

/// How long an orb that missed takes to go out, in ticks.
const MISSED_FOR: f32 = 30.0;

/// The tick, from the release, that the stage `orb` is in began on.
pub fn stage_began(orb: &OrbView) -> i64 {
	let pulse = |nth: i64| CaptureRules::PULSE_TICKS[(nth.clamp(1, 3) - 1) as usize];
	match orb.stage {
		OrbStage::Flight => 0,
		OrbStage::Drawing | OrbStage::Missed => CaptureRules::IMPACT_TICK,
		OrbStage::Falling => CaptureRules::DRAWN_TICK,
		OrbStage::Pulsing | OrbStage::Breath => pulse(orb.pulses),
		OrbStage::Captured => CaptureRules::RESULT_TICK,
		OrbStage::BrokeFree => pulse(orb.broke_free_on.unwrap_or(1)),
	}
}

/// Where the orb's story happens, in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbPlace {
	/// Where it left the hand.
	pub from: Point,
	/// The middle of the husk, where it strikes.
	pub husk: Point,
	/// The ground under the husk, where it lands.
	pub ground: Point,
}

/// The orb at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbLook {
	pub at: Point,
	/// Its tumble or rock, as a turn.
	pub turn: [f32; 4],
	/// How far its talons are open, and how far flung out.
	pub open: f32,
	pub burst: f32,
	/// The soul light: how bright, from 0, and how big, as a share of rest.
	pub core: f32,
	pub core_size: f32,
	/// The halo round it, as a share of full.
	pub halo: f32,
	/// The soul drawn in from the husk: how far gathered, and how bright.
	pub stream: Option<(f32, f32)>,
	/// The latest ring wave: how far it has spread (0 to 1) and how bright.
	pub ring: Option<(f32, f32)>,
	/// How far the sigil has burned in.
	pub sigil: f32,
	/// A flash of light at the orb, from 0.
	pub flare: f32,
	/// Whether the orb is drawn at all.
	pub shown: bool,
}

/// What `orb` looks like `ticks` (fractional) after it left the hand, at
/// `place`. For an orb that missed, `place`'s husk and ground are both where
/// it comes down.
pub fn look(orb: &OrbView, ticks: f32, place: &OrbPlace) -> OrbLook {
	let since = (ticks - stage_began(orb) as f32).max(0.0);
	let seconds = since / TICKS_PER_SECOND;
	let calm = [0.0, 0.0, 0.0, 1.0];
	let mut look = OrbLook {
		at: place.ground,
		turn: calm,
		core: 1.0,
		core_size: 1.0,
		halo: 1.0,
		shown: true,
		..OrbLook::default()
	};
	match orb.stage {
		OrbStage::Flight => {
			let share = (since / CaptureRules::IMPACT_TICK as f32).clamp(0.0, 1.0);
			// A low arc, and a tumble.
			look.at = arc(place.from, place.husk, share, 0.35);
			look.turn = super::turn::axis_angle([0.6, 0.0, 0.8], share * TAU * 1.5);
		}
		OrbStage::Drawing => {
			let share = (since / (CaptureRules::DRAWN_TICK - CaptureRules::IMPACT_TICK) as f32)
				.clamp(0.0, 1.0);
			look.at = place.husk;
			look.open = Ease::Out.apply(share / 0.25);
			look.flare = knock(1.0, 30.0, seconds);
			look.core = 1.0 + 1.5 * share;
			look.core_size = 1.0 + 0.4 * share;
			look.stream = Some((
				Ease::In.apply(share),
				1.0 - Ease::In.apply((share - 0.7) / 0.3),
			));
		}
		OrbStage::Falling => {
			let share = (since / (CaptureRules::LANDED_TICK - CaptureRules::DRAWN_TICK) as f32)
				.clamp(0.0, 1.0);
			look.at = lerp(place.husk, place.ground, Ease::In.apply(share));
			look.open = 1.0 - Ease::Out.apply(share / 0.5);
			look.core = 2.5 - 1.3 * share;
		}
		OrbStage::Pulsing | OrbStage::Breath => {
			// Each pulse is stronger. After the third the orb holds its
			// breath: the last pulse dies away and nothing more comes.
			let strength = 0.6 + 0.2 * orb.pulses.clamp(1, 3) as f32;
			let beat = knock(1.0, 9.0, seconds);
			// It rocks where it lies, and the soul inside swells and pushes
			// at the talons.
			let rock = beat * 0.35 * strength * (seconds * 22.0).sin();
			look.turn = super::turn::axis_angle([1.0, 0.0, 0.3], rock);
			look.at = add(place.ground, [0.0, 0.03 * beat * strength, 0.0]);
			look.open = 0.22 * beat * strength;
			look.core = 1.0 + 1.8 * beat * strength;
			look.core_size = 1.0 + 0.3 * beat;
			look.halo = 0.8 + 0.6 * beat;
			look.ring = ring(seconds);
		}
		OrbStage::Captured => {
			let share = (seconds / 0.6).clamp(0.0, 1.0);
			look.sigil = Ease::Out.apply(share);
			look.flare = knock(0.8, 12.0, seconds);
			// The soul settles to a steady, dimmer light.
			look.core = 1.6 - 0.9 * Ease::InOut.apply(share);
			look.halo = 1.0 - 0.5 * share;
			look.ring = ring(seconds);
		}
		OrbStage::BrokeFree => {
			look.burst = Ease::Out.apply(seconds / 0.15);
			look.open = 1.0;
			look.flare = knock(1.6, 25.0, seconds);
			look.core = (1.0 - seconds / 0.2).max(0.0) * 3.0;
			look.core_size = 1.0 + 2.0 * Ease::Out.apply(seconds / 0.2);
			look.halo = (1.0 - seconds / 0.3).max(0.0);
			look.ring = ring(seconds);
			// The cracked shell lies there a moment, then is gone.
			look.shown = seconds < 1.2;
		}
		OrbStage::Missed => {
			// It comes down shut, with a dull knock of light, and its soul
			// goes out: nothing was caught.
			let share = (since / MISSED_FOR).clamp(0.0, 1.0);
			look.flare = knock(0.5, 18.0, seconds);
			look.core = 1.0 - Ease::In.apply(share);
			look.halo = 1.0 - share;
			look.shown = share < 1.0;
		}
	}
	look
}

/// A ring wave `seconds` after it set out: half a second to spread.
fn ring(seconds: f32) -> Option<(f32, f32)> {
	let share = seconds / 0.5;
	(share < 1.0).then(|| (Ease::Out.apply(share), 1.0 - share))
}

/// The point `share` of the way from `from` to `to`, raised by `lift`
/// halfway, for anything else thrown on the orb's arc.
pub fn arc(from: Point, to: Point, share: f32, lift: f32) -> Point {
	add(
		lerp(from, to, share),
		scale([0.0, 1.0, 0.0], lift * (PI * share).sin()),
	)
}

#[cfg(test)]
mod tests {
	use reapers_rules::CaptureGrade;
	use reapers_rules::Vec3;

	use super::super::turn::length;
	use super::super::turn::sub;
	use super::*;

	const PLACE: OrbPlace = OrbPlace {
		from: [0.0, 0.0, 0.0],
		husk: [0.0, 1.0, -3.0],
		ground: [0.0, 0.0, -3.0],
	};

	/// The orb as the rules report it `ticks` after the release.
	fn orb(stage: OrbStage, ticks: i64, pulses: i64, broke_free_on: Option<i64>) -> OrbView {
		OrbView {
			thrown_at: 0,
			ticks,
			stage,
			grade: CaptureGrade::Plain,
			on_target: stage != OrbStage::Missed,
			pulses,
			target: Vec3::new(0.0, 1.0, 3.0),
			chance: None,
			broke_free_on,
		}
	}

	fn near(a: Point, b: Point) -> bool {
		length(sub(a, b)) < 1.0e-5
	}

	#[test]
	fn each_stage_begins_on_the_rules_tick() {
		let at = |stage, pulses, broke| stage_began(&orb(stage, 0, pulses, broke));
		assert_eq!(at(OrbStage::Flight, 0, None), 0);
		assert_eq!(at(OrbStage::Drawing, 0, None), CaptureRules::IMPACT_TICK);
		assert_eq!(at(OrbStage::Missed, 0, None), CaptureRules::IMPACT_TICK);
		assert_eq!(at(OrbStage::Falling, 0, None), CaptureRules::DRAWN_TICK);
		for pulse in 1..=3 {
			let tick = CaptureRules::PULSE_TICKS[pulse as usize - 1];
			assert_eq!(at(OrbStage::Pulsing, pulse, None), tick);
			assert_eq!(at(OrbStage::BrokeFree, pulse - 1, Some(pulse)), tick);
		}
		assert_eq!(at(OrbStage::Breath, 3, None), CaptureRules::PULSE_TICKS[2]);
		assert_eq!(at(OrbStage::Captured, 3, None), CaptureRules::RESULT_TICK);
	}

	#[test]
	fn the_orb_flies_to_the_husk_and_lands_under_it() {
		let look_at = |stage, ticks: i64, pulses, broke| {
			look(&orb(stage, ticks, pulses, broke), ticks as f32, &PLACE)
		};
		assert_eq!(look_at(OrbStage::Flight, 0, 0, None).at, PLACE.from);
		let struck = look_at(OrbStage::Flight, CaptureRules::IMPACT_TICK, 0, None);
		assert!(near(struck.at, PLACE.husk));
		let landed = look_at(OrbStage::Falling, CaptureRules::LANDED_TICK, 0, None);
		assert!(near(landed.at, PLACE.ground));
		assert!(look_at(OrbStage::Drawing, 40, 0, None).stream.is_some());
		assert!(look_at(OrbStage::Pulsing, 120, 2, None).ring.is_some());
		assert!(look_at(OrbStage::Captured, 262, 3, None).sigil > 0.99);
		let burst = look_at(OrbStage::BrokeFree, 126, 0, Some(1));
		assert!(burst.burst > 0.99 && burst.core < 0.1);
	}

	#[test]
	fn a_break_bursts_on_its_pulse() {
		for pulse in 1..=3 {
			let tick = CaptureRules::PULSE_TICKS[pulse as usize - 1];
			let broke = orb(OrbStage::BrokeFree, tick, pulse - 1, Some(pulse));
			let cracking = look(&broke, tick as f32 + 3.0, &PLACE);
			assert!(cracking.burst > 0.5, "pulse {pulse} cracks at once");
			assert!(cracking.flare > 0.5);
			let gone = look(&broke, tick as f32 + 90.0, &PLACE);
			assert!(!gone.shown, "the shell of pulse {pulse} is gone");
		}
	}

	#[test]
	fn a_missed_orb_comes_down_and_goes_out() {
		let missed = orb(OrbStage::Missed, CaptureRules::IMPACT_TICK, 0, None);
		let down = look(&missed, CaptureRules::IMPACT_TICK as f32, &PLACE);
		assert!(down.shown && near(down.at, PLACE.ground));
		assert_eq!(down.open, 0.0, "it never opens");
		assert!(down.stream.is_none() && down.ring.is_none());
		let out = look(
			&missed,
			CaptureRules::IMPACT_TICK as f32 + MISSED_FOR,
			&PLACE,
		);
		assert!(!out.shown);
	}
}
