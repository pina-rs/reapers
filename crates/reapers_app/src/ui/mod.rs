//! The "funeral ledger" interface kit: sketchy boxes over straight rules,
//! rows and marks, in bone on tinted black, lit by the reaper's own light.

mod ledger;
mod overlay;
mod sketch;
mod text;

pub use ledger::LedgerButton;
pub use ledger::gap;
pub use ledger::ledger_back;
pub use ledger::ledger_heading;
pub use ledger::ledger_meter;
pub use ledger::ledger_notice;
pub use ledger::ledger_page;
pub use ledger::ledger_pip;
pub use ledger::ledger_row;
pub use overlay::dialog;
pub use overlay::sheet;
pub use sketch::sketch;
pub use text::aside;
pub use text::body;
pub use text::display;
pub use text::figure;
pub use text::label;
pub use text::margin;
pub use text::prose;

use crate::state::Game;
use crate::theme::Rgb;
use crate::theme::UNBOUND;
use crate::theme::reaper_light;

/// The light the interface is lit with: the bound reaper's, or the pale
/// light of one nobody has bound yet.
pub fn light() -> Rgb {
	reactive_graph::owner::use_context::<Game>()
		.and_then(|game| game.player())
		.filter(reapers_rules::PlayerState::oath_taken)
		.map_or(UNBOUND, |player| reaper_light(&player.genome()))
}
