//! What a reaper hunts: the five creatures the dead come back as, each a
//! portrait from the game's own art build.
//!
//! The plates watch the reader go by. Each turns a little towards the middle
//! of the screen as it scrolls up through it, and away again as it leaves,
//! the way a husk's head follows you round the room.

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::HAIRLINE;
use super::ui::label;
use super::ui::paragraph;
use super::ui::prose;
use super::ui::section;
use super::ui::title;
use crate::l10n::use_l10n;

/// Where a plate sits in the grid.
#[derive(Clone, Copy)]
enum Place {
	/// One of a pair on a phone, and one of five across a wider screen.
	Paired,
	/// Alone on its row on a phone, so held in the middle at a pair's
	/// width; one of five across a wider screen.
	Alone,
}

impl Place {
	/// The list item's classes: the cells it takes.
	fn cell(self) -> &'static str {
		match self {
			Self::Paired => "w-full",
			Self::Alone => "w-full col-span-2 md:col-span-1 items-center",
		}
	}

	/// The plate's own width in its cell.
	fn width(self) -> &'static str {
		match self {
			Self::Paired => "w-full",
			// About a pair's cell: half the row, less half the gap.
			Self::Alone => "w-[48%] md:w-full",
		}
	}
}

/// A plate: the portrait, the creature's name, and how it moves.
fn plate(source: &str, name: &str, moves: &str, lean: f32, place: Place) -> Element {
	let watch = Motion::keyframes([
		(0.0, Pose::new().rotate(lean).y(32.0)),
		(0.5, Pose::REST),
		(1.0, Pose::new().rotate(-lean).y(-12.0)),
	])
	.easing(Easing::Linear)
	.in_view(ViewRange::cover());
	let held = View()
		.class(format!("{} gap-3", place.width()))
		.motion(watch)
		.child(
			View()
				.class(format!("w-full aspect-square border border-[{HAIRLINE}]"))
				.child(Image(source).class("w-full h-full").aria_hidden(true)),
		)
		.child(
			View()
				.class("w-full gap-1")
				.child(Text(name).class(format!("text-[16px] font-medium text-[{BONE}]")))
				.child(label(moves, ASH)),
		);
	View().role(Role::ListItem).class(place.cell()).child(held)
}

/// What you hunt.
pub fn husks() -> Element {
	let words = use_l10n();
	let walks = words.husks_walks();
	let plates = View()
		.role(Role::List)
		.class("w-full grid grid-cols-2 md:grid-cols-5 gap-x-4 md:gap-x-6 gap-y-10")
		.child(plate(
			"asset://nfts/degenApes_1.webp",
			words.husks_ape(),
			walks,
			-5.0,
			Place::Paired,
		))
		.child(plate(
			"asset://nfts/peskyPenguins_2.webp",
			words.husks_penguin(),
			walks,
			4.0,
			Place::Paired,
		))
		.child(plate(
			"asset://nfts/sollamas_4.webp",
			words.husks_llama(),
			walks,
			-3.0,
			Place::Paired,
		))
		.child(plate(
			"asset://nfts/solarians_3.webp",
			words.husks_robot(),
			walks,
			5.0,
			Place::Paired,
		))
		.child(plate(
			"asset://nfts/thugbirdz_1.webp",
			words.husks_bird(),
			words.husks_flies(),
			-4.0,
			Place::Alone,
		));
	section("husks", words.husks_title()).child(
		View()
			.class(format!("{COLUMN} py-20 md:py-28 gap-12"))
			.child(
				View()
					.class("w-full gap-6 md:flex-row md:items-end md:justify-between")
					.child(title(words.husks_title()).class("md:w-[40%]"))
					.child(
						View()
							.class("w-full md:w-[52%]")
							.child(prose(words.husks_intro())),
					),
			)
			.child(plates)
			.child(paragraph(words.husks_rarity(), 16, ASH)),
	)
}
