//! What is laid over a screen: a dialog in the middle, or a sheet at the
//! foot. leptoskit has no modal presentation yet, so both are views in the
//! root layout, over a scrim that takes the press that dismisses them.

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::sketch::sketch;
use crate::state::Game;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::seed_for;

/// The dark the page falls into under an overlay. Pressing it puts the
/// overlay away.
fn scrim(game: Game) -> Element {
	Pressable(crate::l10n::use_l10n().common_dismiss().to_owned())
		.class("absolute inset-0 bg-[#07080bcc]")
		.test_id("overlay-scrim")
		.on_press(move || game.overlay.set(None))
}

/// A dialog: a ruled panel in the middle of the page.
pub fn dialog(game: Game, seed_text: &str, children: Vec<Element>) -> Element {
	View()
		.class("absolute inset-0 items-center justify-center px-6")
		.child(scrim(game))
		.child(
			View()
				.class(format!(
					"relative w-full {CONTENT_WIDTH} bg-[#0f1116] p-6 gap-3"
				))
				.role(Role::Dialog)
				.aria_modal(true)
				.test_id("dialog")
				.child(sketch(seed_for(seed_text), BONE, 0.18))
				.children(children),
		)
}

/// A sheet: a panel risen from the foot of the page.
pub fn sheet(game: Game, children: Vec<Element>) -> Element {
	View()
		.class("absolute inset-0 justify-end")
		.child(scrim(game))
		.child(
			ScrollView()
				.class("relative w-full max-h-[86vh] bg-[#0f1116] border-t border-[#e9e4d42e]")
				.role(Role::Dialog)
				.aria_modal(true)
				.test_id("sheet")
				.child(
					View()
						.class(format!(
							"w-full {CONTENT_WIDTH} self-center p-6 pb-safe-offset-6"
						))
						.children(children),
				),
		)
}
