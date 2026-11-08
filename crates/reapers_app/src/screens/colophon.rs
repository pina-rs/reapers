//! Who made what the game is made of, and the one switch that goes with
//! it: whether the music plays.
//!
//! The recordings are free to use on the condition that their composer is
//! named in his own words, so this page is part of what lets the game use
//! them. Its credit lines are `assets/music/CREDITS.md`'s, word for word;
//! the test below holds the two together.

use leptoskit::prelude::*;

use crate::l10n::use_l10n;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_page;

/// The colophon.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let music = game.music.get();
	ledger_page(
		words.colophon_overline(),
		Some(words.colophon_title()),
		None,
		vec![
			gap(14),
			LedgerButton::new(words.colophon_music())
				.detail(if music {
					words.colophon_music_on()
				} else {
					words.colophon_music_off()
				})
				.test_id("colophon-music")
				.on_press(move || game.toggle_music()),
			ledger_heading(words.colophon_music_heading(), None),
			credit(
				words.colophon_sanctum(),
				words.colophon_sanctum_piece(),
				words.colophon_sanctum_credit(),
			),
			credit(
				words.colophon_hunt(),
				words.colophon_hunt_piece(),
				words.colophon_hunt_credit(),
			),
			credit(
				words.colophon_rift(),
				words.colophon_rift_piece(),
				words.colophon_rift_credit(),
			),
			ledger_heading(words.colophon_sight_heading(), None),
			credit(
				words.colophon_sight(),
				words.colophon_depth_piece(),
				words.colophon_depth_credit(),
			),
			ledger_heading(words.colophon_type_heading(), None),
			credit(
				words.colophon_type(),
				words.colophon_type_piece(),
				words.colophon_type_credit(),
			),
		],
	)
}

/// One thing somebody else made: where it is used, what it is, and whose
/// it is.
fn credit(place: &str, what: &str, whose: &str) -> Element {
	View()
		.class("pb-4 gap-1")
		.child(label(place, 10, ASH))
		.child(body(what, 15, BONE))
		.child(body(whose, 13, ASH))
}

#[cfg(test)]
mod tests {
	use crate::l10n::En;
	use crate::l10n::Messages;

	const CREDITS: &str = include_str!("../../../../assets/music/CREDITS.md");

	#[test]
	fn the_credit_lines_are_the_composers_own_words() {
		for line in [
			En.colophon_sanctum_credit(),
			En.colophon_hunt_credit(),
			En.colophon_rift_credit(),
		] {
			assert!(CREDITS.contains(&format!("- {line}")), "{line}");
		}
	}
}
