//! The game, started: the state, the router, and the arrival.

use std::time::Duration;

use leptoskit::prelude::*;
use reactive_graph::owner::provide_context;

use crate::l10n::L10n;
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

/// Starts the game in the current reactive owner: the strings, the state,
/// the router (from the address bar in a browser), and the arrival.
pub fn start(navigation: Navigation) -> Game {
	let preferred = preferred_locales();
	L10n::new(&preferred).provide();
	crate::screens::install_navigation(navigation.clone());
	provide_context(navigation);
	let game = Game::start();
	game.play(Score::Sanctum);
	platform::spawn(arrive(game));
	game
}

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
