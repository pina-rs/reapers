//! Every screen that is not a 3D or camera view, each a port of its
//! Flutter namesake. The routes under `src/routes/` are one line each and
//! call these, so a screen's code reads the same whichever URL shows it.

// A screen is one builder, read top to bottom as the page reads.
#![allow(clippy::too_many_lines)]

pub mod armory;
pub mod codex;
pub mod colophon;
pub mod frame;
pub mod home;
pub mod hunt;
pub mod oath;
pub mod rift;
pub mod rifts;
pub mod roster;
pub mod sanctum;
pub mod simnet;
pub mod soul;
pub mod studio;
pub mod threshold;
pub mod vault;

use std::cell::RefCell;

use crate::routes::Navigation;
use crate::routes::Route;

thread_local! {
	/// The app's router. A press runs outside the reactive owner, where no
	/// context can be read, so the router is kept here for handlers and
	/// tasks to reach.
	static NAVIGATION: RefCell<Option<Navigation>> = const { RefCell::new(None) };
}

/// Makes `navigation` the router every screen moves through.
pub fn install_navigation(navigation: Navigation) {
	NAVIGATION.with_borrow_mut(|installed| *installed = Some(navigation));
}

/// The app's router, once one is installed.
pub fn navigation() -> Option<Navigation> {
	NAVIGATION.with_borrow(Clone::clone)
}

/// Opens `route` over the current screen.
pub fn open(route: impl Into<Route>) {
	if let Some(navigation) = navigation() {
		navigation.navigate(route);
	}
}

/// Goes back to the screen before.
pub fn back() {
	if let Some(navigation) = navigation() {
		navigation.back();
	}
}

/// Goes home: the sanctum, or the threshold for an unbound player.
pub fn home() {
	if let Some(navigation) = navigation() {
		navigation.replace(Route::Index);
	}
}
