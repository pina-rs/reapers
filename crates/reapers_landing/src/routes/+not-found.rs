//! Anything no route matches: a static host that answers every path with
//! the page's `index.html` shows the page.

use leptoskit::prelude::*;

/// The page.
#[component]
pub fn Page() -> Element {
	crate::page::page()
}
