//! `/armory`: weapons, arts and outfits.

use leptoskit::prelude::*;

/// The armory.
#[component]
pub fn Page() -> Element {
	crate::screens::armory::page()
}
