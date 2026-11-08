//! `/hunt/:sighting`: the hunt for one loose husk.

use leptoskit::prelude::*;
use leptoskit_router::Route;

/// The loose husk being hunted, by its sighting's id.
#[derive(Route, Clone, Debug, PartialEq)]
pub struct Hunt {
	pub sighting: i64,
}

/// The hunt, settled by the keeper when it ends.
#[component]
pub fn Page(route: Hunt) -> Element {
	crate::screens::hunt::page(route.sighting)
}
