//! `/studio`: the reaper studio.

use leptoskit::prelude::*;

/// The studio, opened on the player's reaper.
#[component]
pub fn Page() -> Element {
	crate::screens::studio::page()
}
