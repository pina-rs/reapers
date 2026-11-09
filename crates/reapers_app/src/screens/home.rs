//! Home. Which screen it is depends on one thing: whether the player has
//! been bound to a reaper yet.

use leptoskit::prelude::*;

use super::sanctum;
use super::threshold;
use crate::l10n::use_l10n;
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
		// A debug build may open at a bench instead: the gallery, which
		// steps the studio through every reaper and outfit
		// (REAPERS_GALLERY), the depth bench, which shows what the
		// platform's AR measures of the room (REAPERS_DEPTH_PROBE), or the
		// arms bench, which plays every weapon's motions and the orb
		// (REAPERS_ARMS_BENCH).
		Books::Open(player) if game.flags.gallery => crate::studio::gallery(player.genome()),
		Books::Open(_) if game.flags.depth_probe => crate::dev::probe::page(),
		Books::Open(_) if game.flags.arms_bench => crate::dev::arms::page(),
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
