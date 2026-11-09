//! The page: its frame, and its sections in the order they are read.

pub mod art;
pub mod chain;
pub mod closing;
pub mod hero;
pub mod hunt;
pub mod husks;
pub mod orb;
pub mod reapers;
pub mod ui;

use leptoskit::native::Role;
use leptoskit::prelude::*;

use self::ui::ASH;
use self::ui::BONE;
use self::ui::COLUMN;
use self::ui::INK;
use self::ui::Weight;
use self::ui::label;
use self::ui::link;
use self::ui::paragraph;
use self::ui::rule;
use self::ui::section;
use crate::config::app_url;
use crate::l10n::use_l10n;

/// The mark, as the game shows it.
const MARK: &str = "asset://branding/mark.png";

/// The frame around the page: the page's ink.
pub fn frame(children: &Children) -> Element {
	View()
		.class(format!("w-full min-h-screen bg-[{INK}] text-[{BONE}]"))
		.children(children.elements().iter().cloned())
}

/// The page, from its banner to its colophon.
pub fn page() -> Element {
	View()
		.class("w-full")
		.child(banner())
		.child(
			View()
				.role(Role::Main)
				.class("w-full")
				.child(hero::hero())
				.child(premise())
				.child(hunt::hunt())
				.child(orb::capture())
				.child(reapers::reapers())
				.child(husks::husks())
				.child(chain::chain())
				.child(closing::play()),
		)
		.child(closing::colophon())
}

/// The banner: the mark and the name, and the way into the game.
fn banner() -> Element {
	let words = use_l10n();
	View().role(Role::Banner).class("w-full").child(
		View()
			.class(format!(
				"{COLUMN} pt-safe-offset-4 pb-2 flex-row items-center justify-between"
			))
			.child(
				View()
					.class("flex-row items-center gap-3")
					.child(Image(MARK).class("w-8 h-8").aria_hidden(true))
					.child(
						Text(words.brand_name())
							.id("wordmark")
							.class(format!("text-[22px] text-[{BONE}]")),
					),
			)
			.child(link(words.header_play(), app_url(), Weight::Quiet)),
	)
}

/// Why the game exists, in one sentence and then a few.
fn premise() -> Element {
	let words = use_l10n();
	let statement = paragraph(words.premise_statement(), 30, BONE)
		.id("premise-statement")
		.motion(Motion::enter(Pose::new().y(28.0).opacity(0.0)).in_view(ViewRange::entry()));
	section("why", words.premise_label()).child(
		View()
			.class(format!("{COLUMN} pt-10 pb-6 md:pt-16 md:pb-8 gap-8"))
			.child(rule())
			.child(label(words.premise_label(), ASH))
			.child(statement)
			.child(
				View()
					.class("w-full md:w-[52%] md:self-end")
					.child(paragraph(words.premise_detail(), 17, ASH)),
			),
	)
}
