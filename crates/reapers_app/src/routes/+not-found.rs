//! Anything no route matches.

use leptoskit::prelude::*;

/// Sends the player home.
#[component]
pub fn Page() -> Element {
	crate::screens::home::page()
}
