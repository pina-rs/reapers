//! One clock for a reaper's clips, and what the host is told to play.
//!
//! The host plays a clip on its own display clock: it is told which clip,
//! how fast, whether it loops, and how long to crossfade from the one
//! before. [`ReaperPlayback`] keeps the same clock on this side, as the
//! Flutter `ReaperPlayback` did, so the studio can show where a clip is and
//! the cloth can stir in time with it.

use leptoskit::scene3d::Play;
use reapers_rules::MotionGroup;
use reapers_rules::ReaperMotion;

use super::appearance::ReaperAppearance;

/// Why a playback setting was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PlaybackError {
	#[error("a progress must be a finite number")]
	Progress,
	#[error("a speed must be finite and above zero")]
	Speed,
}

/// Where a reaper's motion has got to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReaperPlayback {
	motion: ReaperMotion,
	seconds: f64,
	playing: bool,
	speed: f64,
}

impl Default for ReaperPlayback {
	fn default() -> Self {
		Self {
			motion: ReaperMotion::Idle,
			seconds: 0.0,
			playing: true,
			speed: 1.0,
		}
	}
}

impl ReaperPlayback {
	pub const fn motion(&self) -> ReaperMotion {
		self.motion
	}

	pub const fn seconds(&self) -> f64 {
		self.seconds
	}

	pub const fn playing(&self) -> bool {
		self.playing
	}

	pub const fn speed(&self) -> f64 {
		self.speed
	}

	/// How far through the clip, from 0 to 1.
	pub fn progress(&self) -> f64 {
		self.seconds / self.motion.seconds()
	}

	/// Starts `motion` from its beginning, playing or held.
	pub fn select(&mut self, motion: ReaperMotion, play: bool) {
		self.motion = motion;
		self.seconds = 0.0;
		self.playing = play;
	}

	/// Pauses or plays on; a finished one-shot plays again from the start.
	pub fn toggle(&mut self) {
		if !self.playing && self.seconds >= self.motion.seconds() {
			self.seconds = 0.0;
		}
		self.playing = !self.playing;
	}

	/// Plays the motion again from its beginning.
	pub fn replay(&mut self) {
		self.select(self.motion, true);
	}

	/// Holds the clip at `progress`, from 0 to 1.
	pub fn seek(&mut self, progress: f64) -> Result<(), PlaybackError> {
		if !progress.is_finite() {
			return Err(PlaybackError::Progress);
		}
		self.seconds = progress.clamp(0.0, 1.0) * self.motion.seconds();
		self.playing = false;
		Ok(())
	}

	pub fn set_speed(&mut self, speed: f64) -> Result<(), PlaybackError> {
		if !speed.is_finite() || speed <= 0.0 {
			return Err(PlaybackError::Speed);
		}
		self.speed = speed;
		Ok(())
	}

	/// Moves the clock on by `delta` seconds. Renderers call this only when
	/// motion is permitted by accessibility settings.
	pub fn advance(&mut self, delta: f64) {
		if !self.playing || !delta.is_finite() || delta <= 0.0 {
			return;
		}
		self.seconds += delta * self.speed;
		if self.motion.loops() {
			self.seconds %= self.motion.seconds();
		} else if self.seconds >= self.motion.seconds() {
			self.seconds = self.motion.seconds();
			self.playing = false;
		}
	}

	/// What the host is told to play for this clock, in `appearance`'s form.
	///
	/// Gaits loop; everything else plays once and holds its last frame,
	/// which every clip authors as rest. A held clock plays at no speed.
	pub fn play(&self, appearance: &ReaperAppearance) -> Play {
		let play = Play::new(appearance.clip_name(self.motion))
			.speed(if self.playing { self.speed as f32 } else { 0.0 })
			.fade(crossfade(self.motion));
		if self.motion.loops() {
			play
		} else {
			play.once()
		}
	}
}

/// How long the host blends into `motion` from whatever played: a gait
/// eases in, a strike or a gesture snaps.
pub const fn crossfade(motion: ReaperMotion) -> f32 {
	match motion.group() {
		MotionGroup::Movement if motion.loops() => 0.35,
		MotionGroup::Movement => 0.15,
		MotionGroup::Combat | MotionGroup::Finisher => 0.1,
		MotionGroup::Reaction | MotionGroup::Meme => 0.25,
	}
}

/// The face a motion wears at its height, as a morph target and weight:
/// what the studio holds while it inspects an expression.
pub const fn expression(motion: ReaperMotion) -> Option<(&'static str, f32)> {
	match motion {
		ReaperMotion::Laugh | ReaperMotion::Victory => Some(("joy", 1.0)),
		ReaperMotion::Cry => Some(("sorrow", 1.0)),
		ReaperMotion::Bow => Some(("sorrow", 0.6)),
		ReaperMotion::Rage => Some(("anger", 1.0)),
		ReaperMotion::Fear => Some(("fear", 1.0)),
		ReaperMotion::Gigachad
		| ReaperMotion::Sigma
		| ReaperMotion::Shrug
		| ReaperMotion::TPose => Some(("smirk", 1.0)),
		ReaperMotion::AbsoluteCinema => Some(("awe", 1.0)),
		_ => None,
	}
}

/// Every facial morph target the reaper models carry.
pub const EXPRESSIONS: [&str; 6] = ["anger", "awe", "fear", "joy", "smirk", "sorrow"];

#[cfg(test)]
mod tests {
	use leptoskit::scene3d::LoopMode;
	use reapers_rules::WeaponKind;

	use super::*;

	#[test]
	fn one_shots_stop_scrub_stays_still_and_replay_starts_at_zero() {
		let mut clock = ReaperPlayback::default();
		clock.select(ReaperMotion::FinisherPalm, true);
		clock.advance(10.0);
		assert_eq!(clock.progress(), 1.0);
		assert!(!clock.playing());
		clock.seek(0.5).unwrap();
		clock.advance(1.0);
		assert_eq!(clock.progress(), 0.5);
		clock.replay();
		assert_eq!(clock.seconds(), 0.0);
		assert!(clock.playing());
		clock.set_speed(0.5).unwrap();
		clock.advance(1.0);
		assert_eq!(clock.seconds(), 0.5);
		clock.select(ReaperMotion::Jump, false);
		assert_eq!(clock.seconds(), 0.0);
		assert!(!clock.playing());
	}

	#[test]
	fn loops_retain_overflow_and_invalid_time_cannot_corrupt_the_clock() {
		let mut clock = ReaperPlayback::default();
		clock.select(ReaperMotion::Walk, true);
		clock.advance(2.0 * ReaperMotion::Walk.seconds() + 0.5);
		assert!((clock.seconds() - 0.5).abs() < 1e-10);
		clock.advance(f64::NAN);
		clock.advance(-1.0);
		assert!((clock.seconds() - 0.5).abs() < 1e-10);
		assert_eq!(clock.seek(f64::NAN), Err(PlaybackError::Progress));
		assert_eq!(clock.set_speed(0.0), Err(PlaybackError::Speed));
		assert_eq!(clock.set_speed(f64::INFINITY), Err(PlaybackError::Speed));
		clock.seek(2.0).unwrap();
		assert_eq!(clock.progress(), 1.0);
		clock.toggle();
		assert_eq!(clock.seconds(), 0.0);
	}

	#[test]
	fn weapon_finishers_have_explicit_capabilities_and_hand_moves_work_for_all() {
		assert_eq!(ReaperMotion::ALL.len(), 32);
		assert!(ReaperMotion::FinisherImpale.supports(Some(WeaponKind::Spear)));
		assert!(!ReaperMotion::FinisherImpale.supports(Some(WeaponKind::Bell)));
		for weapon in WeaponKind::ALL {
			for motion in ReaperMotion::ALL
				.into_iter()
				.filter(|motion| motion.unarmed())
			{
				assert!(motion.supports(Some(weapon)));
			}
		}
		for motion in ReaperMotion::ALL {
			if let Some(impact) = motion.impact() {
				assert!(impact > 0.0 && impact < 1.0);
			}
		}
	}

	#[test]
	fn the_host_is_told_to_loop_gaits_and_hold_the_rest() {
		let mut clock = ReaperPlayback::default();
		clock.select(ReaperMotion::Run, true);
		let play = clock.play(&ReaperAppearance::default());
		assert_eq!(play.clip, "run");
		assert_eq!(play.mode, LoopMode::Loop);
		clock.select(ReaperMotion::Slash, true);
		let play = clock.play(&ReaperAppearance::default());
		assert_eq!(play.mode, LoopMode::Once);
		assert!(play.fade > 0.0 && play.fade < crossfade(ReaperMotion::Walk));
		clock.toggle();
		assert_eq!(clock.play(&ReaperAppearance::default()).speed, 0.0);
	}

	#[test]
	fn every_reaction_and_meme_has_a_face() {
		for motion in ReaperMotion::ALL {
			if matches!(motion.group(), MotionGroup::Reaction | MotionGroup::Meme) {
				let (target, weight) = expression(motion).expect("a face");
				assert!(EXPRESSIONS.contains(&target));
				assert!(weight > 0.0);
			}
		}
	}
}
