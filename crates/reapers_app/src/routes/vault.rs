//! `/vault`: the vault and the market in one ledger.

use leptoskit::prelude::*;

/// The vault.
#[component]
pub fn Page() -> Element {
	crate::screens::vault::page()
}
