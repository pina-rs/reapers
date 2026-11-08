//! What the game sounds like, and the seam the audio engine plugs into.
//!
//! [`Soundtrack`] decides which piece plays and whether it is heard. The
//! sound itself is a [`ScoreEngine`]'s: the real one comes with
//! `leptoskit_plugin_audio` (being written beside this), and until it is
//! handed in the engine is silent, as it is in every test. The recordings
//! are Kevin MacLeod's, under CC BY 4.0, credited on the colophon page and
//! in `assets/music/CREDITS.md`.
//!
//! How a hunt sounds (`huntMix`, `sigilMix` in the Flutter game) belongs to
//! the hunt, which the 3D agent ports with it.

use std::cell::Cell;

/// One of the game's three pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Score {
	/// Home: a slow, dark ground under the sanctum.
	Sanctum,
	/// A hunt: something in the shadows that cannot quite be heard.
	Hunt,
	/// A rift: a dark march.
	Rift,
}

impl Score {
	/// Where the recording is bundled, under the repository's `assets/`.
	pub const fn asset(self) -> &'static str {
		match self {
			Self::Sanctum => "assets/music/sanctum.ogg",
			Self::Hunt => "assets/music/hunt.ogg",
			Self::Rift => "assets/music/rift.ogg",
		}
	}
}

/// What makes the sound.
pub trait ScoreEngine {
	/// Starts `score` looping, in place of whatever played.
	fn play(&self, score: Score);
	/// Silences or restores the current piece without losing its place.
	fn hold(&self, held: bool);
}

/// An engine that makes no sound.
#[derive(Debug, Default)]
pub struct Silence;

impl ScoreEngine for Silence {
	fn play(&self, _score: Score) {}

	fn hold(&self, _held: bool) {}
}

/// Which piece plays, and whether it is heard.
pub struct Soundtrack {
	engine: Box<dyn ScoreEngine>,
	playing: Cell<Option<Score>>,
}

impl Soundtrack {
	pub fn new(engine: Box<dyn ScoreEngine>) -> Self {
		Self {
			engine,
			playing: Cell::new(None),
		}
	}

	/// A soundtrack with no engine behind it.
	pub fn silent() -> Self {
		Self::new(Box::new(Silence))
	}

	/// The piece that is playing.
	pub fn playing(&self) -> Option<Score> {
		self.playing.get()
	}

	/// Plays `score` if it is not already playing. It is held when the
	/// player has turned the music off.
	pub fn play(&self, score: Score, enabled: bool) {
		if self.playing.get() != Some(score) {
			self.playing.set(Some(score));
			self.engine.play(score);
		}
		self.engine.hold(!enabled);
	}

	/// Turns the music on or off.
	pub fn enable(&self, enabled: bool) {
		self.engine.hold(!enabled);
	}
}

#[cfg(test)]
mod tests {
	use std::rc::Rc;

	use super::*;

	#[derive(Default)]
	struct Recorder(Rc<std::cell::RefCell<Vec<String>>>);

	impl ScoreEngine for Recorder {
		fn play(&self, score: Score) {
			self.0.borrow_mut().push(format!("play {score:?}"));
		}

		fn hold(&self, held: bool) {
			self.0.borrow_mut().push(format!("hold {held}"));
		}
	}

	#[test]
	fn a_piece_is_started_once_and_held_when_the_music_is_off() {
		let log = Rc::new(std::cell::RefCell::new(Vec::new()));
		let soundtrack = Soundtrack::new(Box::new(Recorder(log.clone())));
		soundtrack.play(Score::Sanctum, true);
		soundtrack.play(Score::Sanctum, false);
		assert_eq!(*log.borrow(), ["play Sanctum", "hold false", "hold true"]);
		assert_eq!(soundtrack.playing(), Some(Score::Sanctum));
	}
}
