//! `/`: the landing page.

use leptoskit::prelude::*;

/// The page.
#[component]
pub fn Page() -> Element {
	crate::page::page()
}
