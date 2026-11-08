//! How a husk acts out what is being done to it: it reels from blows,
//! cowers and trembles as its ward fails, rages while it attacks, struggles
//! against the reap and convulses as it comes apart. The Flutter
//! `HuskReaction`.
//!
//! This is only for show and decides nothing. It is plain arithmetic, with
//! no scene in it, so it can be tested; [`super::husk::HuskRig`] turns what
//! it says into how the husk is placed and lit.
//!
//! Angles are in radians, in the model's frame: x to the creature's left,
//! y up, z behind it.

use std::f64::consts::TAU;

/// What the hunt is doing to a husk, as far as its acting goes. The arena
/// reads it off the simulation each frame; nothing here feeds back into
/// the rules.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(
	clippy::struct_excessive_bools,
	reason = "each is one thing the hunt does to it"
)]
pub struct HuskPlight {
	/// What is left of its ward, from 1 (untouched) to 0 (broken).
	pub ward: f64,
	/// Whether the fight has begun. Before it, the husk is unaware and calm.
	pub fighting: bool,
	/// How far it has drawn breath to shriek or is charging, from 0 to 1. A
	/// husk that is attacking is furious, not afraid.
	pub fury: f64,
	/// Whether it is reeling from a parry, a heavy blow or a gust.
	pub staggered: bool,
	/// Whether its ward is broken and it waits to be reaped.
	pub broken: bool,
	/// How far along the arc the reap has cut, from 0 to 1.
	pub reap: f64,
	/// Whether it has been absorbed and is coming apart.
	pub dying: bool,
	/// Whether it is frozen in ice: then nothing moves at all.
	pub frozen: bool,
}

impl Default for HuskPlight {
	/// A husk at rest, before any hunt.
	fn default() -> Self {
		Self {
			ward: 1.0,
			fighting: false,
			fury: 0.0,
			staggered: false,
			broken: false,
			reap: 0.0,
			dying: false,
			frozen: false,
		}
	}
}

/// A damped spring about zero, stepped in small enough steps to stay steady
/// at any frame rate.
#[derive(Clone, Copy, Debug)]
struct Spring {
	stiffness: f64,
	drag: f64,
	position: f64,
	velocity: f64,
}

impl Spring {
	const SUBSTEP: f64 = 1.0 / 240.0;

	fn new(frequency: f64, damping: f64) -> Self {
		Self {
			stiffness: (TAU * frequency).powi(2),
			drag: 2.0 * damping * TAU * frequency,
			position: 0.0,
			velocity: 0.0,
		}
	}

	fn step(&mut self, delta: f64) {
		let mut left = delta;
		while left > 0.0 {
			let dt = left.min(Self::SUBSTEP);
			self.velocity += (-self.stiffness * self.position - self.drag * self.velocity) * dt;
			self.position += self.velocity * dt;
			left -= dt;
		}
	}
}

/// How a husk acts out its plight.
#[derive(Clone, Copy, Debug)]
pub struct HuskReaction {
	seconds: f64,
	// The body's sway from a blow: a lean (x and z, as a tilt in radians)
	// and a twist about the vertical, each a spring that overshoots and
	// settles. A creature's body rocks at about three times a second:
	// slower and it floats, faster and it buzzes instead of reeling.
	lean_x: Spring,
	lean_z: Spring,
	twist: Spring,
	squash: Spring,
	// The head snaps back further than the body, and comes back later.
	head_back: Spring,
	hit_stop: f64,
	reap: f64,
	wince: f64,
	fear: f64,
	rage: f64,
	cower: f64,
	reel: f64,
	struggle: f64,
	death: f64,
	death_age: f64,
	frozen: bool,
}

impl Default for HuskReaction {
	fn default() -> Self {
		Self {
			seconds: 0.0,
			lean_x: Spring::new(3.2, 0.34),
			lean_z: Spring::new(3.2, 0.34),
			twist: Spring::new(4.0, 0.3),
			squash: Spring::new(7.0, 0.4),
			head_back: Spring::new(2.8, 0.32),
			hit_stop: 0.0,
			reap: 0.0,
			wince: 0.0,
			fear: 0.0,
			rage: 0.0,
			cower: 0.0,
			reel: 0.0,
			struggle: 0.0,
			death: 0.0,
			death_age: 0.0,
			frozen: false,
		}
	}
}

fn smooth(x: f64) -> f64 {
	x * x * (3.0 - 2.0 * x)
}

fn toward(from: f64, to: f64, rate: f64, dt: f64) -> f64 {
	from + (to - from) * (1.0 - (-rate * dt).exp())
}

impl HuskReaction {
	/// How long time stands still for the husk after a heavy blow, in
	/// seconds: the instant the blade is in it.
	pub const HIT_STOP_SECONDS: f64 = 0.07;

	/// How fast the animation should run: near zero for an instant after a
	/// heavy blow, then 1. The springs keep moving through it, so the blow
	/// lands with a jolt rather than a pause.
	pub fn time_scale(&self) -> f64 {
		if self.hit_stop > 0.0 && !self.frozen {
			0.08
		} else {
			1.0
		}
	}

	/// A blow lands. `away` is the direction it drives the husk, in the
	/// model's frame, as an angle about the vertical from +x towards +z.
	/// `side` is where on the body it struck, from −1 (the creature's
	/// right) to 1 (its left), and twists it the other way. `strength` runs
	/// from 0 to 1; a `heavy` one stops time for an instant.
	pub fn hit(&mut self, away: f64, side: f64, strength: f64, heavy: bool) {
		if self.frozen {
			return;
		}
		let kick = strength.clamp(0.0, 1.0);
		// The spring is given a kick of speed, not of position, so the blow
		// shows as the body being driven back and then rocking forward
		// again. A full blow tips it about an eighth of a radian, for a
		// moment. Seals are struck where the rules hold them, so a husk
		// still being fought must not reel far from there; the wild motion
		// is saved for once its ward has broken, when nothing more is
		// struck.
		let push = 2.6 * kick * if heavy { 1.3 } else { 1.0 };
		self.lean_x.velocity += away.cos() * push;
		self.lean_z.velocity += away.sin() * push;
		self.twist.velocity -= side.clamp(-1.0, 1.0) * 6.0 * kick;
		self.squash.velocity -= 2.5 * kick;
		self.head_back.velocity += 9.0 * kick;
		self.wince = self.wince.max(kick);
		if heavy {
			self.hit_stop = Self::HIT_STOP_SECONDS;
		}
	}

	/// Moves the acting on by `delta` seconds for a husk in `plight`.
	pub fn step(&mut self, delta: f64, plight: HuskPlight) {
		self.frozen = plight.frozen;
		if self.frozen {
			// Ice holds it exactly as it was.
			self.hit_stop = 0.0;
			return;
		}
		self.seconds += delta;
		self.hit_stop = (self.hit_stop - delta).max(0.0);
		for spring in [
			&mut self.lean_x,
			&mut self.lean_z,
			&mut self.twist,
			&mut self.squash,
			&mut self.head_back,
		] {
			spring.step(delta);
		}
		self.wince *= (-6.0 * delta).exp();
		// Fear grows as the ward fails, and fills it once the ward is
		// broken. Fury pushes it aside while the husk attacks.
		let lost = 1.0 - plight.ward.clamp(0.0, 1.0);
		let dread = if plight.dying || plight.broken || plight.reap > 0.0 {
			1.0
		} else if plight.fighting {
			0.25 + 0.75 * smooth(lost)
		} else {
			0.0
		};
		let fury = plight.fury.clamp(0.0, 1.0);
		self.fear = toward(self.fear, dread * (1.0 - fury), 3.5, delta);
		self.rage = toward(self.rage, fury, 9.0, delta);
		let cower = if plight.broken || plight.reap > 0.0 {
			1.0
		} else {
			(self.fear - 0.35).max(0.0) / 0.65 * 0.6
		};
		self.cower = toward(self.cower, cower, 5.0, delta);
		self.reel = toward(
			self.reel,
			f64::from(u8::from(plight.staggered)),
			10.0,
			delta,
		);
		self.struggle = toward(
			self.struggle,
			f64::from(u8::from(plight.reap > 0.0 && !plight.dying)),
			12.0,
			delta,
		);
		if plight.dying {
			self.death_age += delta;
			self.death = toward(self.death, 1.0, 16.0, delta);
		} else {
			self.death_age = 0.0;
			self.death = toward(self.death, 0.0, 6.0, delta);
		}
		self.reap = plight.reap.clamp(0.0, 1.0);
	}

	/// How afraid it is, from 0 to 1.
	pub fn fear(&self) -> f64 {
		self.fear
	}

	/// How furious it is, from 0 to 1: drawing breath to shriek, or
	/// charging.
	pub fn rage(&self) -> f64 {
		self.rage
	}

	/// How far it shrinks down into itself, from 0 to 1.
	pub fn cower(&self) -> f64 {
		self.cower
	}

	/// How far it has gone in its death throes, from 0 to 1.
	pub fn death(&self) -> f64 {
		self.death
	}

	/// The body's tilt about the z axis, in radians: positive tips the top
	/// towards +x.
	pub fn lean_x(&self) -> f64 {
		self.lean_x.position
			+ self.wobble(1.3, 0.12) * self.reel
			+ self.thrash(0.11) * self.struggle
	}

	/// The body's tilt about the x axis: positive tips it towards +z. In its
	/// death throes it arches back.
	pub fn lean_z(&self) -> f64 {
		self.lean_z.position
			+ self.wobble(1.7, 0.1) * self.reel
			+ self.thrash(0.08) * self.struggle
			+ 0.45 * self.arch()
	}

	/// How far it is pulled over towards the reaper, in radians: dragged up
	/// the arc by the reap, fighting it all the way.
	pub fn drag(&self) -> f64 {
		0.4 * self.reap * self.struggle
	}

	/// The body's twist about the vertical, in radians.
	pub fn twist(&self) -> f64 {
		self.twist.position + self.wobble(2.1, 0.18) * self.reel + self.thrash(0.22) * self.struggle
	}

	/// How much the body is squashed down (negative) or stretched
	/// (positive), as a share of its height.
	pub fn squash(&self) -> f64 {
		self.squash.position * 0.06 - 0.08 * self.cower
	}

	/// How far it is lifted off where it stands, as a share of its height:
	/// the reap draws it up towards the blade.
	pub fn rise(&self) -> f64 {
		0.35 * self.reap * self.struggle + 0.18 * self.death
	}

	/// How wide its jaw hangs, from 0 to 1: teeth chatter in fear, and it
	/// screams while it is reaped and as it dies.
	pub fn gape(&self) -> f64 {
		let chatter =
			self.fear * (1.0 - self.struggle) * 0.16 * (0.5 + 0.5 * (self.seconds * 41.0).sin());
		chatter
			.max(
				(0.35 * self.wince).max(0.75 * self.struggle + 0.25 * self.thrash(0.4).abs())
					+ self.death,
			)
			.clamp(0.0, 1.0)
	}

	/// How far its limbs flail beyond their gait, in radians.
	pub fn flail(&self) -> f64 {
		self.thrash(0.9) * self.struggle
	}

	/// A tremble to lay on any angle, in radians.
	pub fn shiver(&self) -> f64 {
		self.tremble(0.012 + 0.05 * self.fear * self.fear + 0.08 * self.death)
	}

	/// How tall its eyes are, as a scale: wide in terror, squeezed shut by a
	/// blow, narrowed in fury, and blown wide as it dies.
	pub fn eye_height(&self) -> f64 {
		((1.0 + 0.75 * self.fear - 0.55 * self.rage + 0.9 * self.death) * (1.0 - 0.88 * self.wince))
			.clamp(0.08, 2.4)
	}

	/// How wide its eyes are, as a scale.
	pub fn eye_width(&self) -> f64 {
		(1.0 + 0.55 * self.fear - 0.2 * self.rage + 0.7 * self.death).clamp(0.4, 2.2)
	}

	/// The brows' roll, in radians: positive lifts the inner ends, as in
	/// fear; negative drives them down, as in fury or a wince.
	pub fn brow_tilt(&self) -> f64 {
		0.5 * self.fear - 0.6 * self.rage - 0.35 * self.wince + 0.3 * self.death
	}

	/// How bright its eyes burn, as a multiple of their usual glow: they
	/// flicker in fear, blaze in fury and flare out as it dies.
	pub fn eye_glow(&self) -> f64 {
		let flicker = 0.5 + 0.5 * (self.seconds * 23.0 + (self.seconds * 7.0).sin()).sin();
		let flare = self.death * (-2.2 * self.death_age).exp() * 2.5;
		(1.0 + 0.5 * self.fear * flicker + 0.9 * self.rage + flare)
			* (1.0 - 0.85 * self.death * (1.0 - (-2.2 * self.death_age).exp()))
	}

	/// The arch of the death throes: a hard first spasm that slackens.
	fn arch(&self) -> f64 {
		self.death * (0.75 + 0.25 * (self.death_age * 30.0).sin() * (-3.0 * self.death_age).exp())
	}

	/// A fast, uneven shake of `amplitude`: incommensurate waves, so it
	/// never settles into a pattern.
	fn tremble(&self, amplitude: f64) -> f64 {
		let t = self.seconds;
		amplitude
			* (0.55 * (t * 37.0).sin()
				+ 0.3 * (t * 53.3 + 1.1).sin()
				+ 0.15 * (t * 71.7 + 2.3).sin())
	}

	/// A slower, heavier lurch of `amplitude`, for limbs and a body fighting
	/// to get free.
	fn thrash(&self, amplitude: f64) -> f64 {
		let t = self.seconds;
		amplitude * (0.6 * (t * 13.1).sin() + 0.4 * (t * 19.7 + 0.7).sin())
	}

	/// A drunken sway of `amplitude` at about `rate` cycles a second.
	fn wobble(&self, rate: f64, amplitude: f64) -> f64 {
		amplitude * (self.seconds * rate * TAU).sin()
	}
}

#[cfg(test)]
mod tests {
	use std::f64::consts::FRAC_PI_2;

	use super::*;

	fn fighting() -> HuskPlight {
		HuskPlight {
			fighting: true,
			..HuskPlight::default()
		}
	}

	/// Steps `reaction` for `seconds` at sixty frames a second.
	fn play(reaction: &mut HuskReaction, plight: HuskPlight, seconds: f64) {
		let mut t = 0.0;
		while t < seconds {
			reaction.step(1.0 / 60.0, plight);
			t += 1.0 / 60.0;
		}
	}

	#[test]
	fn an_unaware_husk_is_calm() {
		let mut reaction = HuskReaction::default();
		play(&mut reaction, HuskPlight::default(), 2.0);
		assert_eq!(reaction.fear(), 0.0);
		assert_eq!(reaction.rage(), 0.0);
		assert!((reaction.eye_width() - 1.0).abs() < 1e-9);
		assert!(reaction.brow_tilt().abs() < 1e-9);
	}

	#[test]
	fn fear_grows_as_the_ward_fails_and_fills_it_once_broken() {
		let fear_at = |plight| {
			let mut reaction = HuskReaction::default();
			play(&mut reaction, plight, 3.0);
			reaction.fear()
		};
		let fresh = fear_at(fighting());
		let worn = fear_at(HuskPlight {
			ward: 0.4,
			..fighting()
		});
		let broken = fear_at(HuskPlight {
			ward: 0.0,
			broken: true,
			..fighting()
		});
		assert!(fresh < worn && worn < broken && broken > 0.95);
	}

	#[test]
	fn a_terrified_husk_cowers_and_a_furious_one_narrows_its_eyes() {
		let mut terrified = HuskReaction::default();
		let mut furious = HuskReaction::default();
		play(
			&mut terrified,
			HuskPlight {
				ward: 0.0,
				broken: true,
				..fighting()
			},
			3.0,
		);
		play(
			&mut furious,
			HuskPlight {
				fury: 1.0,
				..fighting()
			},
			3.0,
		);
		assert!(terrified.cower() > 0.9);
		assert!(terrified.squash() < 0.0, "it shrinks into itself");
		assert!(terrified.eye_height() > 1.5);
		assert!(terrified.brow_tilt() > 0.4);
		assert!(furious.fear() < 0.05, "fury pushes fear aside");
		assert!(furious.rage() > 0.95);
		assert!(furious.eye_height() < 0.6);
		assert!(furious.brow_tilt() < -0.5);
	}

	#[test]
	fn a_blow_drives_the_body_back_overshoots_and_settles() {
		let mut reaction = HuskReaction::default();
		reaction.hit(FRAC_PI_2, 0.0, 1.0, false);
		play(&mut reaction, fighting(), 0.06);
		assert!(reaction.lean_z() > 0.05, "driven back");
		assert!(
			reaction.lean_z() < 0.16,
			"not so far that its seals leave where they are struck"
		);
		assert!(reaction.lean_x().abs() < 0.02);
		let mut forward = false;
		let mut t = 0.0;
		while t < 0.4 {
			reaction.step(1.0 / 60.0, fighting());
			forward |= reaction.lean_z() < 0.0;
			t += 1.0 / 60.0;
		}
		assert!(forward, "it rocks forward again");
		play(&mut reaction, fighting(), 2.0);
		assert!(reaction.lean_z().abs() < 0.01, "and comes to rest");
	}

	#[test]
	fn a_blow_on_one_side_twists_the_body_away_from_it() {
		let mut left = HuskReaction::default();
		left.hit(0.0, 1.0, 1.0, false);
		let mut right = HuskReaction::default();
		right.hit(0.0, -1.0, 1.0, false);
		play(&mut left, fighting(), 0.05);
		play(&mut right, fighting(), 0.05);
		assert!(left.twist() < 0.0);
		assert!(right.twist() > 0.0);
	}

	#[test]
	fn a_blow_squeezes_the_eyes_shut_for_a_moment() {
		let mut reaction = HuskReaction::default();
		play(&mut reaction, fighting(), 1.0);
		let open = reaction.eye_height();
		reaction.hit(0.0, 0.0, 1.0, false);
		reaction.step(1.0 / 60.0, fighting());
		assert!(reaction.eye_height() < open * 0.3);
		play(&mut reaction, fighting(), 1.5);
		assert!(reaction.eye_height() > open * 0.9);
	}

	#[test]
	fn a_heavy_blow_stops_time_for_an_instant_and_a_light_one_does_not() {
		let mut reaction = HuskReaction::default();
		reaction.hit(0.0, 0.0, 1.0, true);
		assert!(reaction.time_scale() < 0.2);
		play(
			&mut reaction,
			fighting(),
			HuskReaction::HIT_STOP_SECONDS + 0.02,
		);
		assert_eq!(reaction.time_scale(), 1.0);
		let mut light = HuskReaction::default();
		light.hit(0.0, 0.0, 0.3, false);
		assert_eq!(light.time_scale(), 1.0);
	}

	#[test]
	fn ice_holds_a_husk_exactly_as_it_was() {
		let worn = HuskPlight {
			ward: 0.3,
			..fighting()
		};
		let mut reaction = HuskReaction::default();
		play(&mut reaction, worn, 2.0);
		let fear = reaction.fear();
		reaction.hit(0.0, 1.0, 1.0, true);
		play(
			&mut reaction,
			HuskPlight {
				frozen: true,
				..worn
			},
			1.0,
		);
		assert!(reaction.lean_x().abs() < 0.02);
		assert_eq!(reaction.fear(), fear);
		assert_eq!(reaction.time_scale(), 1.0);
	}

	#[test]
	fn the_reap_drags_it_up_towards_the_blade_screaming_and_flailing() {
		let mut reaction = HuskReaction::default();
		let mut flailed = 0.0_f64;
		let mut t = 0.0;
		while t < 1.5 {
			reaction.step(
				1.0 / 60.0,
				HuskPlight {
					ward: 0.0,
					broken: true,
					reap: 0.7,
					..fighting()
				},
			);
			flailed = flailed.max(reaction.flail().abs());
			t += 1.0 / 60.0;
		}
		assert!(reaction.rise() > 0.2);
		assert!(reaction.drag() > 0.25);
		assert!(reaction.gape() > 0.7);
		assert!(flailed > 0.5);
	}

	#[test]
	fn as_it_dies_its_eyes_flare_and_then_go_out() {
		let mut reaction = HuskReaction::default();
		play(
			&mut reaction,
			HuskPlight {
				ward: 0.0,
				broken: true,
				..fighting()
			},
			1.0,
		);
		let living = reaction.eye_glow();
		let dying = HuskPlight {
			ward: 0.0,
			dying: true,
			..fighting()
		};
		play(&mut reaction, dying, 0.15);
		assert!(reaction.eye_glow() > living);
		assert!(reaction.gape() > 0.85);
		assert!(reaction.death() > 0.85);
		play(&mut reaction, dying, 1.5);
		assert!(reaction.eye_glow() < 0.5);
	}
}
