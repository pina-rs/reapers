//! The frame every screen is shown in: the page's ink, the slip for each
//! simnet transaction as it lands, anything laid over the screen, and the
//! arrival that opens the game.

use leptoskit::prelude::*;

use super::simnet;
use super::vault;
use crate::brand::reapers_mark;
use crate::l10n::use_l10n;
use crate::lookups::refusal_message;
use crate::lookups::simnet_instruction_name;
use crate::state::Game;
use crate::state::Overlay;
use crate::state::Slip;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::WOUND;
use crate::theme::seed_for;
use crate::theme::short_address;
use crate::theme::sol_amount;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::dialog;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::label;
use crate::ui::light;
use crate::ui::sketch;

/// The frame around `children`, the matched route.
pub fn frame(children: &Children) -> Element {
	let game = Game::get();
	// The frame is exactly the screen's height and the route scrolls inside
	// it, so what is laid over it (a sheet, a dialog, a slip) covers the
	// screen rather than the whole length of a long ledger: leptoskit has no
	// fixed positioning.
	let mut root = View()
		.class("relative w-full h-screen overflow-hidden bg-[#07080b] text-[#e9e4d4]")
		.child(
			ScrollView()
				.class("flex-1 w-full")
				.test_id("frame-scroll")
				.children(children.elements().iter().cloned()),
		);
	if let Some(overlay) = game.overlay.get() {
		root = root.child(overlay_view(game, overlay));
	}
	// The slip sits over everything but the arrival, a sheet included.
	if let Some(slip) = game.slip.get() {
		root = root.child(
			View()
				.class("absolute left-0 right-0 top-0 items-center px-4 pt-safe-offset-2")
				.child(slip_view(&slip)),
		);
	}
	if !game.arrived.get() {
		root = root.child(arrival());
	}
	root
}

fn overlay_view(game: Game, overlay: Overlay) -> Element {
	let words = use_l10n();
	match overlay {
		Overlay::Refused(refusal) => {
			notice(
				game,
				words.common_refused_title(),
				refusal_message(&*words, refusal),
			)
		}
		Overlay::Unreachable => {
			notice(
				game,
				words.common_failed_title(),
				words.common_unreachable(),
			)
		}
		Overlay::Nft(nft) => vault::nft_sheet(game, nft),
		Overlay::Transaction(signature) => simnet::transaction_sheet(game, &signature),
		Overlay::ConfirmReset => simnet::reset_dialog(game),
	}
}

/// Why a request did not go through, and the way out.
fn notice(game: Game, title: &str, text: &str) -> Element {
	let words = use_l10n();
	dialog(
		game,
		title,
		vec![
			label(title, 11, WOUND),
			display(text, 24, BONE),
			View().class("h-2"),
			LedgerButton::new(words.common_dismiss())
				.test_id("refusal-dismiss")
				.on_press(move || game.overlay.set(None)),
		],
	)
}

/// One transaction, as a slip of paper pinned to the top of the screen.
fn slip_view(slip: &Slip) -> Element {
	let words = use_l10n();
	let transaction = &slip.transaction;
	let instruction = simnet_instruction_name(&*words, transaction.instruction);
	let status = if slip.finalized {
		words.simnet_finalized()
	} else {
		words.simnet_confirmed()
	};
	let mut subject = Vec::new();
	if let Some(nft) = transaction.nft {
		subject.push(nft.title());
	}
	if transaction.lamports > 0 {
		subject.push(words.common_sol(&sol_amount(transaction.lamports)));
	}
	let mut detail = vec![status.to_owned(), words.simnet_slot_value(transaction.slot)];
	if transaction.instruction == crate::simnet::SimnetInstruction::Burn {
		detail.push(
			if transaction.receipt_verifies() {
				words.simnet_receipt_passes()
			} else {
				words.simnet_receipt_fails()
			}
			.to_owned(),
		);
	}
	View()
		.class(format!(
			"relative w-full {CONTENT_WIDTH} flex-row items-center gap-3 px-4 py-3 bg-[#0f1116]"
		))
		.test_id("simnet-slip")
		.aria_live(leptoskit::native::Live::Polite)
		.child(sketch(seed_for(&transaction.signature), BONE, 0.18))
		.child(
			View()
				.class("flex-1 gap-1")
				.child(label(
					&format!("{} · {instruction}", words.simnet_overline()),
					11,
					light(),
				))
				.child(body(subject.join(" · "), 14, BONE))
				.child(body(detail.join(" · "), 12, ASH)),
		)
		.child(figure(short_address(&transaction.signature), 12, ASH))
}

/// The arrival: the mark at the launch screen's own position, and the name
/// beneath it. The game beneath cannot be touched until it lifts.
fn arrival() -> Element {
	let words = use_l10n();
	View()
		.class("absolute inset-0 items-center justify-center bg-[#07080b] gap-6")
		.accessibility_label(words.app_title())
		.test_id("arrival")
		.child(reapers_mark(288, true))
		.child(
			display(words.app_title(), 42, BONE)
				.class("starting:opacity-0 transition-opacity duration-700"),
		)
}
