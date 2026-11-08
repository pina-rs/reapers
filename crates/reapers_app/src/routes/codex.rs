//! `/codex`: everything this reaper has absorbed.

use leptoskit::prelude::*;

/// The codex.
#[component]
pub fn Page() -> Element {
	crate::screens::codex::page()
}
