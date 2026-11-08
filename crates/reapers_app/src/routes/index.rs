//! `/`: the threshold for an unbound player, the sanctum for a bound one.

use leptoskit::prelude::*;

/// Home: which screen depends on whether the player has taken the oath.
#[component]
pub fn Page() -> Element {
	crate::screens::home::page()
}
