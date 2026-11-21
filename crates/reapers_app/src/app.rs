//! The game, started: the state, the router, and the arrival.

use std::cell::RefCell;
use std::time::Duration;

use leptoskit::prelude::*;
use reactive_graph::owner::Owner;
use reactive_graph::owner::provide_context;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use reactive_graph::traits::GetUntracked;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use reactive_graph::traits::Set;

use crate::l10n::AppTranslations;
use crate::l10n::L10n;
use crate::l10n::Messages;
use crate::platform;
use crate::routes::Navigation;
use crate::routes::Route;
use crate::sound::Score;
use crate::state::Books;
use crate::state::Game;

/// How long the arrival's sweep takes, and how long a reduced-motion
/// arrival holds its still.
const ARRIVAL: Duration = Duration::from_millis(1800);
const STILL: Duration = Duration::from_millis(250);

/// The arrival never hides loading or recovery for longer than this.
const CEILING: Duration = Duration::from_secs(4);

thread_local! {
	/// The active locale's messages, for non-render callbacks (tickers, async
	/// handlers) that run outside a reactive owner.
	static MESSAGES: RefCell<&'static dyn Messages> = RefCell::new(&crate::l10n::En);
}

/// The active locale's messages, for code running outside a component render.
pub fn messages() -> &'static dyn Messages {
	MESSAGES.with_borrow(|messages| *messages)
}

/// Owns each render's [`on_cleanup`](reactive_graph::owner::on_cleanup) hooks
/// as a child of the root [`Owner`].
///
/// The next render runs under a fresh child owner before the previous child is
/// cleaned up: a screen that stays mounted keeps its reference count above
/// zero (no ticker restart or closure leak), while a screen left behind drops
/// to zero and shuts down its ticker and XR session.
pub struct RenderScope {
	root: Owner,
	current: RefCell<Option<Owner>>,
}

impl RenderScope {
	pub const fn new(root: Owner) -> Self {
		Self {
			root,
			current: RefCell::new(None),
		}
	}

	pub fn render(&self, build: impl FnOnce() -> Element) -> Element {
		let next = self.root.child();
		let element = next.with(build);
		if let Some(previous) = self.current.borrow_mut().replace(next) {
			previous.cleanup();
		}
		element
	}
}

/// Starts the game in the current reactive owner: the strings, the state,
/// the router (from the address bar in a browser), and the arrival.
pub fn start(navigation: Navigation) -> Game {
	use leptoskit_l10n::Translations as _;

	let preferred = preferred_locales();
	let l10n = L10n::new(&preferred);
	let chosen = AppTranslations::messages(&l10n.locale());
	MESSAGES.with_borrow_mut(|messages| *messages = chosen);
	l10n.provide();
	crate::screens::install_navigation(navigation.clone());
	provide_context(navigation);
	let game = Game::start();
	install_escape_handler(game);
	game.play(Score::Sanctum);
	platform::spawn(arrive(game));
	game
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn install_escape_handler(game: Game) {
	use std::cell::Cell;

	use wasm_bindgen::JsCast as _;
	use wasm_bindgen::prelude::Closure;

	thread_local! {
		static INSTALLED: Cell<bool> = const { Cell::new(false) };
	}
	if INSTALLED.replace(true) {
		return;
	}
	let Some(window) = web_sys::window() else {
		return;
	};
	let on_key =
		Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
			if event.key() == "Escape" && game.overlay.get_untracked().is_some() {
				game.overlay.set(None);
				event.prevent_default();
			}
		});
	let _ = window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
	on_key.forget();
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
fn install_escape_handler(_game: Game) {}

/// Lifts the arrival once its sweep has run and the books are open, or at
/// the ceiling, whichever comes first.
async fn arrive(game: Game) {
	let sweep = if platform::reduced_motion() {
		STILL
	} else {
		ARRIVAL
	};
	let step = Duration::from_millis(100);
	let mut waited = Duration::ZERO;
	loop {
		let opened = !matches!(game.books.get_untracked(), Books::Opening);
		if (waited >= sweep && opened) || waited >= CEILING {
			break;
		}
		if !platform::sleep(step).await {
			break;
		}
		waited += step;
	}
	game.arrived.set(true);
}

/// The router for this host: the browser's address, or home.
pub fn navigation() -> Navigation {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	{
		Navigation::from_browser().unwrap_or_else(|_| Navigation::new(Route::Index))
	}
	#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
	{
		Navigation::new(Route::Index)
	}
}

fn preferred_locales() -> Vec<leptoskit_l10n::Locale> {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	{
		leptoskit_l10n::browser_locales()
	}
	#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
	{
		Vec::new()
	}
}

/// The whole interface: the current route inside its layouts.
pub fn screen(navigation: &Navigation) -> Element {
	navigation.render()
}
