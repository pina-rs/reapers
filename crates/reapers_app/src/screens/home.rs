//! Home. Which screen it is depends on one thing: whether the player has
//! been bound to a reaper yet.

use leptoskit::prelude::*;

use super::sanctum;
use super::threshold;
use crate::l10n::use_l10n;
use crate::seams::StudioViewProps;
use crate::seams::studio_view;
use crate::state::Books;
use crate::state::BooksFailure;
use crate::state::Game;
use crate::theme::ASH;
use crate::ui::ledger_notice;

/// The home screen for the books as they stand.
pub fn page() -> Element {
	let game = Game::get();
	match game.books.get() {
		Books::Opening => opening_books(),
		Books::Unreadable(failure) => unreadable(game, &failure),
		// A debug build may open at a bench instead (REAPERS_GALLERY,
		// REAPERS_DEPTH_PROBE); both are the 3D port's, so both open the
		// studio's seam for now.
		Books::Open(player) if game.flags.gallery || game.flags.depth_probe => {
			studio_view(&StudioViewProps {
				genome: player.genome(),
			})
		}
		Books::Open(player) if player.oath_taken() => sanctum::page(&player),
		Books::Open(_) => threshold::page(),
	}
}

fn opening_books() -> Element {
	let words = use_l10n();
	View()
		.class("flex-1 min-h-screen items-center justify-center p-7")
		.test_id("opening-books")
		.child(
			Status(words.startup_opening_books())
				.class(format!("text-[15px] text-[{}]", ASH.hex())),
		)
}

/// Shown when the player could not be read at all: a damaged save, or a
/// keeper that cannot be reached.
fn unreadable(game: Game, failure: &BooksFailure) -> Element {
	let words = use_l10n();
	let text = match failure {
		BooksFailure::Unreachable => words.common_unreachable(),
		BooksFailure::Damaged | BooksFailure::Misbuilt(_) => words.startup_unreadable(),
	};
	if let BooksFailure::Misbuilt(error) = failure {
		// A build whose flags cannot work says why, for whoever built it.
		crate::platform::log(&error.to_string());
	}
	ledger_notice(
		words.common_failed_title(),
		text,
		words.common_retry(),
		move || game.retry(),
	)
}
