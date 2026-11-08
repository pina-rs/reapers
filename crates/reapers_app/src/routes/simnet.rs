//! `/simnet`: the simnet's explorer.

use leptoskit::prelude::*;

/// The simnet ledger.
#[component]
pub fn Page() -> Element {
	crate::screens::simnet::page()
}
