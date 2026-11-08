//! `/roster`: every reaper the player holds, and the wallet that proves them.

use leptoskit::prelude::*;

/// The roster.
#[component]
pub fn Page() -> Element {
	crate::screens::roster::page()
}
