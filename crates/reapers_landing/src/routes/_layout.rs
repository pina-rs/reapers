//! The frame around the page.

use leptoskit::prelude::*;

/// Wraps the matched route in the page's ink.
#[component]
pub fn Layout(children: Children) -> Element {
	crate::page::frame(&children)
}
