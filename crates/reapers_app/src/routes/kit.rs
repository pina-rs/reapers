//! `/kit`: every control and mark of Second Ink in every state. Debug builds
//! only; a release sends it home.

use leptoskit::prelude::*;

/// The kit page.
#[component]
pub fn Page() -> Element {
	crate::dev::kit::page()
}
