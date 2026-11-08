//! `/oath`: where a player is bound to a reaper they did not choose.

use leptoskit::prelude::*;

/// The oath, the shuffle, and the reaper it drew.
#[component]
pub fn Page() -> Element {
	crate::screens::oath::page()
}
