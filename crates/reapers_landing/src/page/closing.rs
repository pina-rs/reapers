//! The end of the page: the way into the game, and the colophon.
//!
//! The music is other people's, free to use on the condition that its
//! composer is named in his own words wherever the game is published, and
//! this page publishes it. Its credit lines are `assets/music/CREDITS.md`'s,
//! word for word; the test below holds the two together.

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::HAIRLINE;
use super::ui::Weight;
use super::ui::label;
use super::ui::link;
use super::ui::paragraph;
use super::ui::prose;
use super::ui::section;
use super::ui::title;
use crate::config::app_url;
use crate::l10n::use_l10n;

/// The mark, as the game shows it.
const MARK: &str = "asset://branding/mark.png";

/// The way into the game.
pub fn play() -> Element {
	let words = use_l10n();
	section("play", words.play_title()).child(
		View()
			.class(format!("{COLUMN} py-24 md:py-36 items-center gap-8"))
			.child(Image(MARK).class("w-16 h-16").aria_hidden(true).motion(
				Motion::enter(Pose::new().scale(0.8).opacity(0.0)).in_view(ViewRange::entry()),
			))
			.child(title(words.play_title()).class("text-center"))
			.child(
				View()
					.class("w-full max-w-[560px] items-center")
					.child(paragraph(words.play_text(), 17, ASH).class("text-center")),
			)
			.child(link(words.play_action(), app_url(), Weight::Primary)),
	)
}

/// A group of the colophon's entries, under its name.
fn group(name: &str, entries: Vec<Element>) -> Element {
	View()
		.class(format!("w-full gap-5 pt-5 border-t border-[{HAIRLINE}]"))
		.child(label(name, ASH))
		.children(entries)
}

/// One piece of music: where it plays, what it is, and its credit line.
fn credit(place: &str, piece: &str, line: &str) -> Element {
	View()
		.class("w-full gap-1")
		.child(Text(place).class(format!("text-[13px] text-[{ASH}]")))
		.child(Text(piece).class(format!("text-[16px] font-medium text-[{BONE}]")))
		.child(paragraph(line, 14, ASH))
}

/// The colophon: who made what the game is made of.
pub fn colophon() -> Element {
	let words = use_l10n();
	let music = group(
		words.colophon_music_heading(),
		vec![
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
		],
	);
	let made = View()
		.class("w-full gap-10")
		.child(group(
			words.colophon_type_heading(),
			vec![paragraph(words.colophon_type(), 15, ASH)],
		))
		.child(group(
			words.colophon_made_heading(),
			vec![paragraph(words.colophon_made(), 15, ASH)],
		));
	View()
		.role(Role::ContentInfo)
		.id("colophon")
		.class("w-full")
		.child(
			View()
				.class(format!("{COLUMN} pt-16 pb-safe-offset-12 gap-12"))
				.child(
					Heading(2, words.colophon_title())
						.class(format!("text-[30px] font-normal text-[{BONE}]")),
				)
				.child(
					View()
						.class("w-full gap-10 md:flex-row md:gap-16")
						.child(View().class("w-full md:flex-1").child(music))
						.child(View().class("w-full md:flex-1").child(made)),
				)
				.child(
					View()
						.class(format!(
							"w-full flex-row items-center gap-3 pt-8 border-t border-[{HAIRLINE}]"
						))
						.child(Image(MARK).class("w-6 h-6").aria_hidden(true))
						.child(prose(words.footer_status())),
				),
		)
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
