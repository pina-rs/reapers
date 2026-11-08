//! The frame around every screen: the page's ink, the arrival, the slips
//! and anything laid over the screen.

use leptoskit::prelude::*;

/// Wraps the matched route in the game's frame.
#[component]
pub fn Layout(children: Children) -> Element {
	crate::screens::frame::frame(children)
}
