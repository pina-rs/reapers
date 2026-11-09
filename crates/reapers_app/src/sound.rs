//! What the game sounds like, and the seam the audio engine plugs into.
//!
//! [`Soundtrack`] decides which piece plays and how it is heard. The sound
//! itself is a [`ScoreEngine`]'s: [`device_engine`] is the real one, on
//! `leptoskit_plugin_audio`, which [`Game::start`](crate::state::Game::start)
//! hands in; until then, and in every test, the engine is [`Silence`].
//! [`hunt_mix`] and [`sigil_mix`] are the whole of how a hunt sounds: plain
//! functions from the hunt's and the sigil's state to a [`Mix`].
//!
//! The recordings are Kevin `MacLeod`'s, under CC BY 4.0, credited on the
//! colophon page and in `assets/music/CREDITS.md`.

use std::cell::Cell;
use std::time::Duration;

use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;
use reapers_rules::SigilTrace;

mod effects;
pub use effects::Effect;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod browser;
#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
mod guest;
#[cfg(any(
	test,
	all(target_arch = "wasm32", any(feature = "web", feature = "native"))
))]
mod plugin;
#[cfg(test)]
mod tests;

/// A piece of the game's music. Each is one recording, played on a loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Score {
	/// The reaper alone in the dark: a slow drone with something thin and
	/// pale singing over it.
	Sanctum,
	/// A hunt: something in the shadows that cannot quite be heard.
	Hunt,
	/// A rift: a dark march.
	Rift,
}

impl Score {
	pub const fn name(self) -> &'static str {
		match self {
			Self::Sanctum => "sanctum",
			Self::Hunt => "hunt",
			Self::Rift => "rift",
		}
	}

	/// The recording, as a path inside the app's `assets/` folder, which
	/// `build.rs` fills from the repository's `assets/music`.
	pub const fn asset(self) -> &'static str {
		match self {
			Self::Sanctum => "music/sanctum.ogg",
			Self::Hunt => "music/hunt.ogg",
			Self::Rift => "music/rift.ogg",
		}
	}

	/// How the piece is heard when it comes in: the sanctum as recorded, a
	/// hunt or a rift a long way off, as their screens begin them.
	pub const fn first_heard(self) -> Mix {
		match self {
			Self::Sanctum => Mix::OPEN,
			Self::Hunt | Self::Rift => Mix::WAITING,
		}
	}
}

/// How a piece is heard at one moment.
///
/// The music is never switched for another tune as a hunt goes on. The same
/// recording is brought nearer or pushed away: louder or quieter, and clear
/// or muffled, as if heard through the veil the hunt takes place behind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mix {
	/// From 0 (silent) to 1 (as loud as the game gets).
	pub volume: f64,
	/// From 0 (clear) to 1 (muffled until only the low strings come
	/// through).
	pub veil: f64,
}

impl Mix {
	const CLEAR: f64 = 16_000.0;
	const MUFFLED: f64 = 320.0;
	/// The piece as recorded, at a level that sits under everything else.
	pub const OPEN: Self = Self {
		volume: 0.55,
		veil: 0.0,
	};
	/// The husk has shown itself and not yet seen the reaper: nearer, still
	/// muffled.
	pub const UNSEEN: Self = Self {
		volume: 0.3,
		veil: 0.8,
	};
	/// Something is nearby and has not shown itself: the music is a long
	/// way off.
	pub const WAITING: Self = Self {
		volume: 0.22,
		veil: 0.9,
	};

	/// The frequency, in hertz, above which a veil of this weight lets
	/// nothing through. Each step of the veil takes off the same share of
	/// what is left, which is how the ear measures it.
	pub fn cutoff(self) -> f64 {
		Self::CLEAR * (Self::MUFFLED / Self::CLEAR).powf(self.veil.clamp(0.0, 1.0))
	}

	/// Whether going from `other` to this would be heard: a change too small
	/// to notice is not worth telling the engine about.
	pub fn differs_from(self, other: Self) -> bool {
		(self.volume - other.volume).abs() > 0.01 || (self.veil - other.veil).abs() > 0.01
	}
}

/// How the hunt's music is heard while its sigil is traced: far off at
/// first, and nearer with every light that catches, until it is where the
/// stalk takes it up.
pub fn sigil_mix(trace: &SigilTrace) -> Mix {
	let drawn = (trace.caught() as f64 + trace.fill()) / trace.sigil.lights.len() as f64;

	Mix {
		volume: Mix::WAITING.volume + (Mix::UNSEEN.volume - Mix::WAITING.volume) * drawn,
		veil: Mix::WAITING.veil + (Mix::UNSEEN.veil - Mix::WAITING.veil) * drawn,
	}
}

/// How the hunt's music is heard at this moment of `sim`.
///
/// The music follows what the hunt asks of the player. It comes clear when
/// the fight is on, drains away as the husk draws breath so that the shriek
/// is met in near silence, swells as the husk charges, and is gone by the
/// time the husk has come apart: the quiet after the reap is part of the
/// hunt.
pub fn hunt_mix(sim: &HuntSim) -> Mix {
	match sim.phase() {
		// Unseen, the music is far off. Holding steady on the husk draws it
		// in.
		HuntPhase::Stalk => {
			Mix {
				volume: Mix::UNSEEN.volume + 0.15 * sim.steadiness(),
				veil: Mix::UNSEEN.veil - 0.3 * sim.steadiness(),
			}
		}
		HuntPhase::Fight => {
			Mix {
				volume: if sim.staggered() { 1.0 } else { 0.8 },
				veil: 0.0,
			}
		}
		HuntPhase::Windup => {
			Mix {
				volume: 0.8 - 0.68 * sim.shriek_progress(),
				veil: 0.9 * sim.shriek_progress(),
			}
		}
		HuntPhase::Shriek => {
			Mix {
				volume: 0.06,
				veil: 1.0,
			}
		}
		HuntPhase::Charge => {
			Mix {
				volume: 1.0,
				veil: 0.0,
			}
		}
		HuntPhase::Broken | HuntPhase::Capture => {
			Mix {
				volume: 0.45,
				veil: 0.45,
			}
		}
		HuntPhase::Absorbed => {
			Mix {
				volume: 0.0,
				veil: 0.6,
			}
		}
		HuntPhase::Escaped => {
			Mix {
				volume: 0.25,
				veil: 0.85,
			}
		}
	}
}

/// Whatever actually makes the sound.
///
/// The game decides which piece plays and how it is heard; an engine does
/// as it is told, in its own time. Keeping the two apart is what lets the
/// deciding be tested with nothing audible at all.
pub trait ScoreEngine {
	fn prepare_effects(&self) {}
	fn cue(&self, _effect: Effect) {}
	fn stop_effects(&self) {}
	/// Brings `piece` in over `fade`, heard as `mix`, and lets whatever was
	/// playing fall away over the same time.
	fn play(&self, piece: Score, mix: Mix, fade: Duration);

	/// Changes how the piece now playing is heard.
	fn shape(&self, mix: Mix);

	/// Lets whatever is playing fall away over `fade`.
	fn silence(&self, fade: Duration);

	/// Holds the music where it is, or lets it carry on: for when the game
	/// leaves the screen and comes back.
	fn hold(&self, held: bool);
}

/// An engine that makes no sound: what the game has until one is given,
/// and all a test needs.
#[derive(Debug, Default)]
pub struct Silence;

impl ScoreEngine for Silence {
	fn play(&self, _piece: Score, _mix: Mix, _fade: Duration) {}

	fn shape(&self, _mix: Mix) {}

	fn silence(&self, _fade: Duration) {}

	fn hold(&self, _held: bool) {}
}

/// The engine this host plays through: Web Audio in a browser, the app's
/// `audio` plugin in a native guest, and [`Silence`] where there is
/// neither, or where it cannot be opened.
pub fn device_engine() -> Box<dyn ScoreEngine> {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	if let Some(engine) = browser::engine() {
		return Box::new(engine);
	}
	#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
	if let Some(engine) = guest::engine() {
		return Box::new(engine);
	}
	Box::new(Silence)
}

/// Calls `hold` with `true` when the game goes out of sight and `false`
/// when it comes back. Only the browser says so yet: leptoskit tells a
/// native guest nothing of the app going to the background.
pub fn when_hidden(hold: impl Fn(bool) + 'static) {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	browser::when_hidden(hold);
	#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
	let _ = hold;
}

/// Decides what the engine is told. Screens say which piece belongs to them
/// and how it should be heard; this remembers it, so that music turned off
/// and on again comes back as it should be, and says nothing to the engine
/// that would not be heard.
pub struct Soundtrack {
	engine: Box<dyn ScoreEngine>,
	enabled: Cell<bool>,
	effects_enabled: Cell<bool>,
	piece: Cell<Option<Score>>,
	mix: Cell<Mix>,
}

impl Soundtrack {
	/// How long one piece takes to give way to another.
	pub const CROSSFADE: Duration = Duration::from_millis(1600);

	pub fn new(engine: Box<dyn ScoreEngine>, enabled: bool) -> Self {
		Self {
			engine,
			enabled: Cell::new(enabled),
			effects_enabled: Cell::new(true),
			piece: Cell::new(None),
			mix: Cell::new(Mix::OPEN),
		}
	}

	pub fn prepare_effects(&self) {
		self.engine.prepare_effects();
	}

	pub fn combat(
		&self,
		events: &[(i64, reapers_rules::HuntEvent)],
		vessel: reapers_rules::Vessel,
		heavy: reapers_rules::WeaponKind,
	) {
		if !self.effects_enabled.get() {
			return;
		}
		for (tick, event) in events {
			if let Some(cue) = Effect::event(event, *tick, vessel, heavy) {
				self.engine.cue(cue);
			}
		}
	}

	pub fn set_effects_enabled(&self, enabled: bool) {
		self.effects_enabled.set(enabled);
		if !enabled {
			self.engine.stop_effects();
		}
	}

	/// The piece that is playing, or would be if music were on.
	pub fn piece(&self) -> Option<Score> {
		self.piece.get()
	}

	/// How the piece is being heard.
	pub fn mix(&self) -> Mix {
		self.mix.get()
	}

	/// Makes `piece` the music, heard as `mix`. Asking for the piece already
	/// playing only changes how it is heard.
	pub fn play(&self, piece: Score, mix: Mix) {
		if self.piece.get() == Some(piece) {
			self.shape(mix);
			return;
		}

		self.piece.set(Some(piece));
		self.mix.set(mix);

		if self.enabled.get() {
			self.engine.play(piece, mix, Self::CROSSFADE);
		}
	}

	/// Changes how the piece now playing is heard. Cheap enough for every
	/// tick: only a change that would be heard reaches the engine.
	pub fn shape(&self, mix: Mix) {
		if !mix.differs_from(self.mix.get()) {
			return;
		}

		self.mix.set(mix);

		if self.enabled.get() && self.piece.get().is_some() {
			self.engine.shape(mix);
		}
	}

	pub fn set_enabled(&self, enabled: bool) {
		if enabled == self.enabled.get() {
			return;
		}

		self.enabled.set(enabled);

		if !enabled {
			self.engine.silence(Self::CROSSFADE);
		} else if let Some(piece) = self.piece.get() {
			self.engine.play(piece, self.mix.get(), Self::CROSSFADE);
		}
	}

	/// Holds the music while the game is off the screen.
	pub fn hold(&self, held: bool) {
		self.engine.hold(held);
	}
}
