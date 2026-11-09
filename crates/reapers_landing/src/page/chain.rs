//! Built on Solana: what happens to a husk's NFT, as four ruled entries in
//! the ledger, and an honest line on where that stands.

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::art::colour;
use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::HAIRLINE;
use super::ui::SOUL;
use super::ui::paragraph;
use super::ui::prose;
use super::ui::section;
use super::ui::title;
use crate::l10n::use_l10n;

/// The dotted leader a ledger runs from a name to its figure.
fn leader() -> Element {
	let dots = Stroke::new(colour(BONE, 0x55), 1.0)
		.expect("a positive width")
		.round()
		.dashes(&[0.1, 6.0], 0.0)
		.expect("positive dashes");
	Vector(ViewBox::new(0.0, 0.0, 100.0, 2.0).expect("a positive size"))
		.aria_hidden(true)
		.scaling(VectorScaling::Stretch)
		.path(VectorPath::new(PathData::parse("M0 1 L100 1").expect("a line")).stroke(dots))
		.class("hidden md:flex flex-1 min-w-[24px] h-[2px] mx-4")
}

/// One entry: a diamond, what happened to the NFT, and how.
fn entry(step: &str, detail: &str) -> Element {
	View()
		.role(Role::ListItem)
		.class(format!(
			"w-full flex-row flex-wrap md:flex-nowrap items-center gap-x-4 gap-y-1 py-5 \
			 border-b border-[{HAIRLINE}]"
		))
		.child(
			View()
				.aria_hidden(true)
				.class(format!("w-2 h-2 rotate-45 bg-[{SOUL}]")),
		)
		.child(Heading(3, step).class(format!(
			"w-[150px] text-[26px] md:text-[30px] font-normal text-[{BONE}]"
		)))
		.child(leader())
		.child(Text(detail).class(format!(
			"w-full md:w-auto md:shrink pl-6 md:pl-0 text-[16px] text-[{ASH}]"
		)))
}

/// The dead stay dead.
pub fn chain() -> Element {
	let words = use_l10n();
	let entries = View()
		.role(Role::List)
		.class(format!("w-full border-t border-[{HAIRLINE}]"))
		.child(entry(
			words.solana_released(),
			words.solana_released_detail(),
		))
		.child(entry(words.solana_hunted(), words.solana_hunted_detail()))
		.child(entry(words.solana_signed(), words.solana_signed_detail()))
		.child(entry(words.solana_burned(), words.solana_burned_detail()));
	section("solana", words.solana_title()).child(
		View()
			.class(format!(
				"{COLUMN} py-20 md:py-28 gap-12 md:flex-row md:gap-16 md:items-start"
			))
			.child(
				View()
					.class("w-full md:w-[42%] gap-6")
					.child(title(words.solana_title()))
					.child(prose(words.solana_intro())),
			)
			.child(
				View()
					.class("w-full md:flex-1 gap-8 md:pt-3")
					.child(entries)
					.child(paragraph(words.solana_status(), 15, ASH)),
			),
	)
}
