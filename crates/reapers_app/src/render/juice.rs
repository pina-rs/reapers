//! The weight of a blow: a hit-stop that holds the reaper's own arms still
//! for a few frames, a trauma shake, and the dive lens.
//!
//! [`Juice`] is a small value the hunt keeps and hands to the views. It
//! reads no clock: it is told the tick of every blow and asked about any
//! tick after, so the arms, the effects and a bench all agree.
//!
//! The shake moves only what the reaper's blow moves: its own arms, the
//! HUD and the husk. Never the camera's picture, and never more than
//! [`MAX_SHAKE_PIXELS`] on the world.

use reapers_rules::WeaponKind;
use reapers_rules::weapon_profile;

use super::arms::TICKS_PER_SECOND;

/// How fast trauma drains, a share of the most there can be each second.
pub const TRAUMA_DECAY: f32 = 1.5;
/// The furthest the shake moves the world, in pixels.
pub const MAX_SHAKE_PIXELS: f32 = 6.0;
/// The furthest the dive lens zooms.
pub const MAX_ZOOM: f32 = 3.0;
/// How close the dive lens brings the husk, in apparent metres.
const DIVE_DISTANCE: f32 = 1.0;

/// A blow, and how hard it lands on the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jolt {
	Shot,
	/// A shot that struck an open seal.
	SealShot,
	Stab,
	/// A heavy swing with the class weapon. The harder the rules say it
	/// strikes, the harder it lands.
	Heavy(WeaponKind),
	Parry,
	Kill,
	OrbImpact,
	/// The soul breaking out of the orb.
	OrbBreak,
	/// The reaper struck.
	Struck,
}

impl Jolt {
	/// How much trauma it adds, out of 1.
	pub const fn trauma(self) -> f32 {
		match self {
			Self::Shot => 0.1,
			Self::SealShot => 0.2,
			Self::Stab | Self::OrbImpact => 0.4,
			Self::Heavy(kind) if heft(kind) == 0 => 0.45,
			Self::Heavy(_) => 0.55,
			Self::Parry => 0.3,
			Self::Kill | Self::OrbBreak => 0.7,
			Self::Struck => 0.5,
		}
	}

	/// How many frames, at sixty a second, the arms hold still.
	pub const fn stop_frames(self) -> i64 {
		match self {
			Self::Shot | Self::Struck | Self::OrbImpact => 0,
			Self::SealShot => 2,
			Self::Stab => 5,
			Self::Heavy(kind) => 6 + heft(kind),
			Self::Parry => 4,
			Self::Kill | Self::OrbBreak => 10,
		}
	}
}

/// How heavy a class weapon lands, from 0 to 2, by how many hits the rules
/// say a swing of it does: up to the scythe's, up to the axe's, and more.
const fn heft(kind: WeaponKind) -> i64 {
	let strike = weapon_profile(kind).strike;
	if strike < 1.2 {
		0
	} else if strike < 1.6 {
		1
	} else {
		2
	}
}

/// Where the shake has the world this moment, each from -1 to 1 of the
/// most it may move: across, up, and a turn clockwise.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shake {
	pub x: f32,
	pub y: f32,
	pub roll: f32,
}

/// Every blow's lasting effect: trauma, and the latest hit-stop.
///
/// A blow may be landed ahead of time, at the tick it will strike: until
/// then the trauma is what it was before it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Juice {
	trauma: f32,
	trauma_tick: i64,
	/// The trauma before the latest blow, and when that was.
	earlier: f32,
	earlier_tick: i64,
	stop_from: i64,
	stop_ticks: i64,
}

impl Juice {
	/// `jolt` landed at tick `now`.
	pub fn land(&mut self, jolt: Jolt, now: i64) {
		let before = self.trauma_at(now as f64);
		self.earlier = self.trauma;
		self.earlier_tick = self.trauma_tick;
		self.trauma = (before + jolt.trauma()).min(1.0);
		self.trauma_tick = now;
		let frames = jolt.stop_frames();
		if frames > 0 {
			let held_until = self.stop_from + self.stop_ticks;
			// A stop during a stop lengthens it; it never shortens one.
			if now < held_until {
				self.stop_ticks = self.stop_ticks.max(now + frames - self.stop_from);
			} else {
				self.stop_from = now;
				self.stop_ticks = frames;
			}
		}
	}

	/// The trauma left at `tick`, from 0 to 1.
	pub fn trauma_at(&self, tick: f64) -> f32 {
		let (trauma, from) = if tick < self.trauma_tick as f64 {
			(self.earlier, self.earlier_tick)
		} else {
			(self.trauma, self.trauma_tick)
		};
		let seconds = ((tick - from as f64) as f32 / TICKS_PER_SECOND).max(0.0);
		(trauma - TRAUMA_DECAY * seconds).max(0.0)
	}

	/// The shake at `tick`: trauma squared, so small blows barely move and
	/// big ones throw, on smooth noise that never repeats in a fight.
	pub fn shake_at(&self, tick: f64) -> Shake {
		let amount = self.trauma_at(tick).powi(2);
		if amount <= 0.0 {
			return Shake::default();
		}
		let t = (tick / f64::from(TICKS_PER_SECOND)) as f32;
		let noise = |phase: f32| {
			((t * 37.0 + phase).sin() * 0.6 + (t * 61.0 + phase * 2.3).sin() * 0.4).clamp(-1.0, 1.0)
		};
		Shake {
			x: amount * noise(0.0),
			y: amount * noise(1.7),
			roll: amount * noise(4.1),
		}
	}

	/// How far the arms' own clock has got by `tick`: the hunt's ticks, less
	/// the ones a hit-stop held.
	pub fn motion_tick(&self, tick: f64) -> f64 {
		let from = self.stop_from as f64;
		let held = (tick - from).clamp(0.0, self.stop_ticks as f64);
		tick - held
	}

	/// Whether the arms are held still at `tick`.
	pub fn holding(&self, tick: f64) -> bool {
		let from = self.stop_from as f64;
		tick >= from && tick < from + self.stop_ticks as f64
	}
}

/// The dive lens's zoom: as the dive goes in (`dive` from 0 to 1) it
/// brings a husk `distance` metres away to about a metre, never past
/// [`MAX_ZOOM`]. The camera's picture and the 3D field of view zoom
/// together about the middle of the screen.
pub fn dive_zoom(dive: f32, distance: f32) -> f32 {
	let full = (distance / DIVE_DISTANCE).clamp(1.0, MAX_ZOOM);
	1.0 + (full - 1.0) * super::curves::Ease::InOut.apply(dive)
}

/// A field of view narrowed by `zoom`: the same middle, `zoom` times
/// larger.
pub fn zoomed_fov(fov_y: f32, zoom: f32) -> f32 {
	2.0 * ((fov_y * 0.5).tan() / zoom.max(1.0)).atan()
}

/// The turn, in radians, that moves the world `pixels` across a view
/// `height` pixels tall seen through `fov_y`.
pub fn pixels_to_angle(pixels: f32, height: f32, fov_y: f32) -> f32 {
	(pixels / height.max(1.0) * 2.0 * (fov_y * 0.5).tan()).atan()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn trauma_stacks_drains_and_shakes_with_its_square() {
		let mut juice = Juice::default();
		juice.land(Jolt::Stab, 0);
		juice.land(Jolt::Stab, 0);
		assert!((juice.trauma_at(0.0) - 0.8).abs() < 1.0e-6);
		// Drains at 1.5 a second: gone in under 0.6 s.
		assert!(juice.trauma_at(36.0) < 1.0e-6);
		let shake = juice.shake_at(3.0);
		assert!(shake.x.abs() <= 0.8 * 0.8 && shake.y.abs() <= 0.64 && shake.roll.abs() <= 0.64);
		assert_eq!(juice.shake_at(100.0), Shake::default());
	}

	#[test]
	fn a_hit_stop_holds_the_arms_clock_and_only_lengthens() {
		let mut juice = Juice::default();
		juice.land(Jolt::Stab, 100);
		assert!(juice.holding(102.0));
		assert_eq!(juice.motion_tick(103.0), 100.0);
		assert_eq!(juice.motion_tick(110.0), 105.0);
		juice.land(Jolt::SealShot, 101);
		assert_eq!(
			juice.motion_tick(110.0),
			105.0,
			"a shorter stop inside a longer one changes nothing"
		);
		juice.land(Jolt::Kill, 104);
		assert_eq!(juice.motion_tick(120.0), 106.0);
	}

	#[test]
	fn a_heavier_class_weapon_holds_the_arms_longer() {
		let frames = |kind| Jolt::Heavy(kind).stop_frames();
		assert_eq!(frames(WeaponKind::Scythe), 6);
		assert_eq!(frames(WeaponKind::ChainWhip), 6);
		assert_eq!(frames(WeaponKind::Axe), 7);
		assert_eq!(frames(WeaponKind::Greatsword), 8);
		for kind in WeaponKind::ALL {
			assert!((6..=8).contains(&frames(kind)), "{kind:?}");
		}
	}

	#[test]
	fn a_blow_landed_ahead_of_time_waits_for_its_tick() {
		let mut juice = Juice::default();
		juice.land(Jolt::Shot, 0);
		juice.land(Jolt::Kill, 30);
		assert!(
			(juice.trauma_at(3.0) - 0.025).abs() < 1.0e-6,
			"only the shot shakes before the kill lands"
		);
		assert!(juice.trauma_at(29.0) < 1.0e-6);
		assert!((juice.trauma_at(30.0) - 0.7).abs() < 1.0e-6);
		assert!(!juice.holding(29.0));
		assert!(juice.holding(30.0));
	}

	#[test]
	fn the_lens_is_capped_and_the_shake_is_small() {
		assert_eq!(dive_zoom(1.0, 10.0), MAX_ZOOM);
		assert_eq!(dive_zoom(1.0, 0.5), 1.0);
		assert_eq!(dive_zoom(0.0, 10.0), 1.0);
		let fov = 60_f32.to_radians();
		assert!((zoomed_fov(fov, 1.0) - fov).abs() < 1.0e-6);
		assert!(zoomed_fov(fov, 3.0) < fov / 2.5);
		// Six pixels of an 800-pixel view is under half a degree.
		assert!(pixels_to_angle(MAX_SHAKE_PIXELS, 800.0, fov).to_degrees() < 0.5);
	}
}
