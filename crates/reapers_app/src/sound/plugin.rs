//! The real engine, on `leptoskit_plugin_audio`.
//!
//! One piece plays at a time, on a loop, streamed as it goes so that a
//! ten-minute recording never has to sit in memory whole. A new piece fades
//! in while the old one falls away. Everything passes through one low-pass
//! filter on the master bus, which is what the veil is.
//!
//! The browser's `web::Audio` and a native guest's `guest::Audio` speak the
//! same model through slightly different calls, so the engine is written
//! once against [`Device`], which each of them implements.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use leptoskit_plugin_audio::Filter;
use leptoskit_plugin_audio::Play;
use leptoskit_plugin_audio::Sound;
use leptoskit_plugin_audio::Target;
use leptoskit_plugin_audio::Voice;

use super::Mix;
use super::Score;
use super::ScoreEngine;
use crate::platform;

/// A call the device could not carry out. The music is not worth more than
/// staying quiet.
#[derive(Debug)]
pub struct Unheard;

/// The plugin's calls, as the engine needs them.
pub trait Device: 'static {
	/// Where an asset may be, before its path inside `assets/`: the engine
	/// tries each in turn.
	const ASSET_ROOTS: &'static [&'static str];

	/// Waits until the device may make a sound. Cheap once it has.
	async fn ready(&self);

	/// Loads the asset at `path` to be streamed as it plays.
	async fn stream(&self, path: &str) -> Result<Sound, Unheard>;

	async fn unload(&self, sound: Sound);

	async fn play(&self, sound: Sound, play: Play) -> Result<Voice, Unheard>;

	async fn set_volume(&self, target: Target, volume: f32, ramp: Duration);

	async fn stop(&self, target: Target, fade: Duration);

	async fn pause(&self, target: Target, paused: bool, fade: Duration);

	async fn set_filter(&self, target: Target, filter: Filter, glide: Duration);
}

/// How quickly a change of [`Mix`] is eased in, so that nothing clicks.
const EASE: Duration = Duration::from_millis(180);

/// How quickly the music is held and let go.
const HOLD: Duration = Duration::from_millis(150);

/// Plays the music through a [`Device`].
pub struct PluginEngine<D: Device>(Rc<Inner<D>>);

struct Inner<D> {
	device: D,
	sounds: RefCell<HashMap<Score, Sound>>,
	playing: Cell<Option<Voice>>,
	/// How the music was last asked to be heard, so a change asked for while
	/// a piece is still coming in is not lost.
	wanted: Cell<Mix>,
	asked: Cell<u64>,
}

impl<D: Device> PluginEngine<D> {
	pub fn new(device: D) -> Self {
		Self(Rc::new(Inner {
			device,
			sounds: RefCell::default(),
			playing: Cell::new(None),
			wanted: Cell::new(Mix::OPEN),
			asked: Cell::new(0),
		}))
	}
}

impl<D: Device> ScoreEngine for PluginEngine<D> {
	fn play(&self, piece: Score, mix: Mix, fade: Duration) {
		let asked = self.0.asked.get() + 1;
		self.0.asked.set(asked);
		self.0.wanted.set(mix);
		platform::spawn(self.0.clone().bring_in(asked, piece, mix, fade));
	}

	fn shape(&self, mix: Mix) {
		self.0.wanted.set(mix);

		if let Some(voice) = self.0.playing.get() {
			platform::spawn(self.0.clone().shape(voice, mix));
		}
	}

	fn silence(&self, fade: Duration) {
		self.0.asked.set(self.0.asked.get() + 1);

		if let Some(voice) = self.0.playing.take() {
			let inner = self.0.clone();
			platform::spawn(async move { inner.device.stop(Target::Voice(voice), fade).await });
		}
	}

	fn hold(&self, held: bool) {
		let inner = self.0.clone();
		platform::spawn(async move { inner.device.pause(Target::Master, held, HOLD).await });
	}
}

impl<D: Device> Inner<D> {
	fn still_wanted(&self, asked: u64) -> bool {
		self.asked.get() == asked
	}

	async fn bring_in(self: Rc<Self>, asked: u64, piece: Score, mix: Mix, fade: Duration) {
		self.device.ready().await;

		// Loading takes a moment. If another piece, or silence, was asked
		// for meanwhile, this one is no longer wanted.
		let Some(sound) = self.sound(piece).await else {
			return;
		};

		if !self.still_wanted(asked) {
			return;
		}

		let leaving = self.playing.take();
		self.veil(mix, Duration::ZERO).await;
		let play = Play::new()
			.volume(loudness(mix))
			.looping(true)
			.fade_in(fade);
		let voice = self.device.play(sound, play).await;

		if let Some(leaving) = leaving {
			self.device.stop(Target::Voice(leaving), fade).await;
		}

		let Ok(voice) = voice else {
			return;
		};

		if !self.still_wanted(asked) {
			self.device.stop(Target::Voice(voice), fade).await;
			return;
		}

		self.playing.set(Some(voice));

		// A line for whoever is wondering why there is no sound. Never in a
		// release build.
		if cfg!(debug_assertions) {
			platform::log(&format!("Music: {} is playing", piece.name()));
		}

		let wanted = self.wanted.get();

		if wanted.differs_from(mix) {
			self.shape(voice, wanted).await;
		}
	}

	async fn shape(self: Rc<Self>, voice: Voice, mix: Mix) {
		self.device
			.set_volume(Target::Voice(voice), loudness(mix), EASE)
			.await;
		self.veil(mix, EASE).await;
	}

	async fn veil(&self, mix: Mix, glide: Duration) {
		let filter = Filter::low_pass(mix.cutoff() as f32);
		self.device.set_filter(Target::Master, filter, glide).await;
	}

	/// `piece`, loaded once and kept.
	async fn sound(&self, piece: Score) -> Option<Sound> {
		if let Some(&sound) = self.sounds.borrow().get(&piece) {
			return Some(sound);
		}

		let mut loaded = None;

		for root in D::ASSET_ROOTS {
			if let Ok(sound) = self
				.device
				.stream(&format!("{root}{}", piece.asset()))
				.await
			{
				loaded = Some(sound);
				break;
			}
		}

		let loaded = loaded?;
		let kept = *self.sounds.borrow_mut().entry(piece).or_insert(loaded);

		// Asked for twice while it loaded: keep one.
		if kept != loaded {
			self.device.unload(loaded).await;
		}

		Some(kept)
	}
}

/// The plugin's volume for `mix`, which never asks for more than the
/// recording's own level.
fn loudness(mix: Mix) -> f32 {
	mix.volume.clamp(0.0, 1.0) as f32
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use leptoskit_plugin_audio::FilterKind;

	use super::*;
	use crate::sound::Soundtrack;

	/// A device that writes down what it is told, and finds only the assets
	/// it is given.
	#[derive(Clone, Default)]
	struct Recording {
		told: Rc<RefCell<Vec<String>>>,
		voices: Rc<Cell<u32>>,
	}

	impl Recording {
		fn told(&self) -> Vec<String> {
			self.told.borrow().clone()
		}

		fn note(&self, line: String) {
			self.told.borrow_mut().push(line);
		}
	}

	fn target(target: Target) -> String {
		match target {
			Target::Master => "master".into(),
			Target::Voice(voice) => format!("voice {}", voice.0),
		}
	}

	#[allow(
		clippy::unused_async_trait_impl,
		reason = "a device answers in its own time; this one at once"
	)]
	impl Device for Recording {
		const ASSET_ROOTS: &'static [&'static str] = &["", "assets/"];

		async fn ready(&self) {}

		async fn stream(&self, path: &str) -> Result<Sound, Unheard> {
			// As a native app finds them on Android: under `assets/`.
			let Some(name) = path.strip_prefix("assets/music/") else {
				return Err(Unheard);
			};
			self.note(format!("load {name}"));
			Ok(Sound(name.len() as u32))
		}

		async fn unload(&self, sound: Sound) {
			self.note(format!("unload {}", sound.0));
		}

		async fn play(&self, _sound: Sound, play: Play) -> Result<Voice, Unheard> {
			let voice = self.voices.get() + 1;
			self.voices.set(voice);
			self.note(format!(
				"play voice {voice} at {} over {}ms, looping {}",
				play.volume,
				play.fade_in.as_millis(),
				play.looping
			));
			Ok(Voice(voice))
		}

		async fn set_volume(&self, target_: Target, volume: f32, ramp: Duration) {
			self.note(format!(
				"volume {} to {volume} over {}ms",
				target(target_),
				ramp.as_millis()
			));
		}

		async fn stop(&self, target_: Target, fade: Duration) {
			self.note(format!(
				"stop {} over {}ms",
				target(target_),
				fade.as_millis()
			));
		}

		async fn pause(&self, target_: Target, paused: bool, _fade: Duration) {
			self.note(format!("pause {} {paused}", target(target_)));
		}

		async fn set_filter(&self, target_: Target, filter: Filter, glide: Duration) {
			assert_eq!(filter.kind, FilterKind::LowPass);
			self.note(format!(
				"veil {} at {}Hz over {}ms",
				target(target_),
				filter.cutoff.round(),
				glide.as_millis()
			));
		}
	}

	fn engine() -> (PluginEngine<Recording>, Recording) {
		let device = Recording::default();
		(PluginEngine::new(device.clone()), device)
	}

	const FADE: Duration = Soundtrack::CROSSFADE;

	#[test]
	fn a_piece_loops_in_over_the_fade_behind_its_veil() {
		let (engine, device) = engine();
		engine.play(Score::Hunt, Mix::WAITING, FADE);
		platform::run_until_stalled();
		assert_eq!(
			device.told(),
			[
				"load hunt.ogg",
				"veil master at 473Hz over 0ms",
				"play voice 1 at 0.22 over 1600ms, looping true",
			]
		);
	}

	#[test]
	fn another_piece_comes_in_as_the_last_falls_away_and_each_loads_once() {
		let (engine, device) = engine();
		engine.play(Score::Sanctum, Mix::OPEN, FADE);
		platform::run_until_stalled();
		engine.play(Score::Hunt, Mix::OPEN, FADE);
		platform::run_until_stalled();
		engine.play(Score::Sanctum, Mix::OPEN, FADE);
		platform::run_until_stalled();
		assert_eq!(
			device.told(),
			[
				"load sanctum.ogg",
				"veil master at 16000Hz over 0ms",
				"play voice 1 at 0.55 over 1600ms, looping true",
				"load hunt.ogg",
				"veil master at 16000Hz over 0ms",
				"play voice 2 at 0.55 over 1600ms, looping true",
				"stop voice 1 over 1600ms",
				"veil master at 16000Hz over 0ms",
				"play voice 3 at 0.55 over 1600ms, looping true",
				"stop voice 2 over 1600ms",
			]
		);
	}

	#[test]
	fn a_change_eases_the_voice_and_glides_the_veil() {
		let (engine, device) = engine();
		engine.play(Score::Hunt, Mix::WAITING, FADE);
		platform::run_until_stalled();
		engine.shape(Mix {
			volume: 1.0,
			veil: 1.0,
		});
		platform::run_until_stalled();
		assert_eq!(
			device.told()[3..],
			[
				"volume voice 1 to 1 over 180ms",
				"veil master at 320Hz over 180ms",
			]
		);
	}

	#[test]
	fn a_change_asked_for_while_the_piece_comes_in_is_heard_once_it_has() {
		let (engine, device) = engine();
		engine.play(Score::Hunt, Mix::WAITING, FADE);
		engine.shape(Mix {
			volume: 0.8,
			veil: 0.0,
		});
		platform::run_until_stalled();
		assert_eq!(
			device.told()[3..],
			[
				"volume voice 1 to 0.8 over 180ms",
				"veil master at 16000Hz over 180ms",
			]
		);
	}

	#[test]
	fn a_piece_silenced_before_it_loaded_never_plays() {
		let (engine, device) = engine();
		engine.play(Score::Rift, Mix::WAITING, FADE);
		engine.silence(FADE);
		platform::run_until_stalled();
		assert_eq!(device.told(), ["load rift.ogg"]);
	}

	#[test]
	fn silence_lets_the_piece_fall_away_and_holding_rests_the_whole_output() {
		let (engine, device) = engine();
		engine.play(Score::Sanctum, Mix::OPEN, FADE);
		platform::run_until_stalled();
		engine.silence(FADE);
		engine.hold(true);
		engine.hold(false);
		engine.shape(Mix::WAITING);
		platform::run_until_stalled();
		assert_eq!(
			device.told()[3..],
			[
				"stop voice 1 over 1600ms",
				"pause master true",
				"pause master false",
			]
		);
	}

	#[test]
	fn the_soundtrack_tells_the_engine_only_what_would_be_heard() {
		let device = Recording::default();
		let soundtrack = Soundtrack::new(Box::new(PluginEngine::new(device.clone())), true);
		for _ in 0..30 {
			soundtrack.play(Score::Hunt, Mix::WAITING);
			soundtrack.shape(Mix::WAITING);
		}
		platform::run_until_stalled();
		assert_eq!(device.told().len(), 3);
	}
}
