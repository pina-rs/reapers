//! `/rift/:site`: a seat in the rift at one site.

use leptoskit::prelude::*;
use leptoskit_router::Route;

/// The rift's site, by the rules' name for it.
#[derive(Route, Clone, Debug, PartialEq)]
pub struct Rift {
	pub site: String,
}

/// The rift.
#[component]
pub fn Page(route: Rift) -> Element {
	crate::screens::rift::page(&route.site)
}
