//! The engine in a native app: the host's `audio` plugin, through
//! `guest::Audio`.

use std::cell::Cell;
use std::time::Duration;

use leptoskit_plugin_audio::Filter;
use leptoskit_plugin_audio::LoadMode;
use leptoskit_plugin_audio::Play;
use leptoskit_plugin_audio::Session;
use leptoskit_plugin_audio::Sound;
use leptoskit_plugin_audio::Source;
use leptoskit_plugin_audio::Target;
use leptoskit_plugin_audio::Voice;
use leptoskit_plugin_audio::guest::Audio;

use super::plugin::Device;
use super::plugin::PluginEngine;
use super::plugin::Unheard;

/// The engine, or `None` when the host has no `audio` plugin.
pub fn engine() -> Option<PluginEngine<Guest>> {
	let audio = Audio::new().ok()?;
	Some(PluginEngine::new(Guest {
		audio,
		configured: Cell::new(false),
	}))
}

pub struct Guest {
	audio: Audio,
	configured: Cell<bool>,
}

impl Device for Guest {
	// iOS reads assets from the bundle's `assets` folder, which leptoskit
	// fills with the app's own, so `music/hunt.ogg` is found there. Android
	// opens the path inside the APK's assets, where leptoskit puts the
	// app's folder whole, as `assets/music/hunt.ogg`.
	const ASSET_ROOTS: &'static [&'static str] = &["", "assets/"];

	async fn ready(&self) {
		// Ambient: game music, which mixes with other apps' audio and keeps
		// to the silent switch.
		if !self.configured.replace(true) {
			let _ = self.audio.configure(Session::default()).await;
		}
	}

	async fn stream(&self, path: &str) -> Result<Sound, Unheard> {
		self.audio
			.load(Source::Asset(path), LoadMode::Stream)
			.await
			.map(|info| info.sound)
			.map_err(|_| Unheard)
	}

	async fn sample(&self, path: &str) -> Result<Sound, Unheard> {
		self.audio
			.load(Source::Asset(path), LoadMode::Memory)
			.await
			.map(|info| info.sound)
			.map_err(|_| Unheard)
	}

	async fn unload(&self, sound: Sound) {
		let _ = self.audio.unload(sound).await;
	}

	async fn play(&self, sound: Sound, play: Play) -> Result<Voice, Unheard> {
		self.audio.play(sound, play).await.map_err(|_| Unheard)
	}

	async fn set_volume(&self, target: Target, volume: f32, ramp: Duration) {
		let _ = self.audio.set_volume(target, volume, ramp).await;
	}

	async fn stop(&self, target: Target, fade: Duration) {
		let _ = self.audio.stop(target, fade).await;
	}

	async fn pause(&self, target: Target, paused: bool, fade: Duration) {
		let _ = self.audio.pause(target, paused, fade).await;
	}

	async fn set_filter(&self, target: Target, filter: Filter, glide: Duration) {
		let _ = self.audio.set_filter(target, 0, Some(filter), glide).await;
	}
}
