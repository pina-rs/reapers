//! The reaper studio, opened on the player's reaper, or on one the shuffle
//! draws for a player who has none.

use leptoskit::prelude::*;
use reapers_rules::ReaperGenome;

use crate::seams::StudioViewProps;
use crate::seams::studio_view;
use crate::state::Game;

/// The studio.
pub fn page() -> Element {
	let game = Game::get();
	let genome = game
		.player()
		.filter(reapers_rules::PlayerState::oath_taken)
		.map_or_else(|| ReaperGenome::shuffle(1), |player| player.genome());
	studio_view(&StudioViewProps { genome })
}
