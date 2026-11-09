//! The engine in a browser: Web Audio, through the plugin's `web::Audio`.
//!
//! A browser keeps a page silent until it has been pressed. The plugin
//! resumes its audio context on that first press; the engine also waits for
//! it before starting a piece, because a streamed piece plays from an
//! `<audio>` element, and an element the page starts before then is refused
//! for good.

use std::cell::RefCell;
use std::time::Duration;

use futures::FutureExt as _;
use futures::channel::oneshot;
use futures::future::Shared;
use leptoskit_plugin_audio::Filter;
use leptoskit_plugin_audio::LoadMode;
use leptoskit_plugin_audio::Play;
use leptoskit_plugin_audio::Sound;
use leptoskit_plugin_audio::Source;
use leptoskit_plugin_audio::Target;
use leptoskit_plugin_audio::Voice;
use leptoskit_plugin_audio::web::Audio;
use wasm_bindgen::JsCast as _;
use wasm_bindgen::closure::Closure;

use super::plugin::Device;
use super::plugin::PluginEngine;
use super::plugin::Unheard;

/// What counts as the page being pressed: the plugin's own list.
const GESTURES: [&str; 3] = ["pointerdown", "keydown", "touchend"];

/// The engine, or `None` when the browser has no Web Audio.
pub fn engine() -> Option<PluginEngine<Browser>> {
	let audio = Audio::new().ok()?;
	let (pressed, first_press) = oneshot::channel();
	let pressed = RefCell::new(Some(pressed));
	let listener = Closure::<dyn FnMut()>::new(move || {
		if let Some(pressed) = pressed.borrow_mut().take() {
			let _ = pressed.send(());
		}
	});
	if let Some(document) = document() {
		for gesture in GESTURES {
			let _ = document
				.add_event_listener_with_callback(gesture, listener.as_ref().unchecked_ref());
		}
	}
	Some(PluginEngine::new(Browser {
		audio,
		first_press: first_press.shared(),
		_listener: listener,
	}))
}

/// Calls `hold` as the page is hidden and shown again.
pub fn when_hidden(hold: impl Fn(bool) + 'static) {
	let Some(document) = document() else {
		return;
	};
	let listener = Closure::<dyn FnMut()>::new(move || {
		if let Some(document) = self::document() {
			hold(document.hidden());
		}
	});
	let _ = document
		.add_event_listener_with_callback("visibilitychange", listener.as_ref().unchecked_ref());
	// The page lives as long as the game: so does the listener.
	listener.forget();
}

fn document() -> Option<web_sys::Document> {
	web_sys::window()?.document()
}

pub struct Browser {
	audio: Audio,
	first_press: Shared<oneshot::Receiver<()>>,
	_listener: Closure<dyn FnMut()>,
}

#[allow(
	clippy::unused_async_trait_impl,
	reason = "Web Audio answers at once where the native plugin answers in its own time"
)]
impl Device for Browser {
	// The web build serves the app's `assets/` at `/assets/`.
	const ASSET_ROOTS: &'static [&'static str] = &[""];

	async fn ready(&self) {
		if !self.audio.unlocked() {
			let _ = self.first_press.clone().await;
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
		let _ = self.audio.unload(sound);
	}

	async fn play(&self, sound: Sound, play: Play) -> Result<Voice, Unheard> {
		self.audio.play(sound, play).map_err(|_| Unheard)
	}

	async fn set_volume(&self, target: Target, volume: f32, ramp: Duration) {
		let _ = self.audio.set_volume(target, volume, ramp);
	}

	async fn stop(&self, target: Target, fade: Duration) {
		let _ = self.audio.stop(target, fade);
	}

	async fn pause(&self, target: Target, paused: bool, fade: Duration) {
		let _ = self.audio.pause(target, paused, fade);
	}

	async fn set_filter(&self, target: Target, filter: Filter, glide: Duration) {
		let _ = self.audio.set_filter(target, 0, Some(filter), glide);
	}
}
