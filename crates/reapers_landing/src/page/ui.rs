//! The pieces every section is set with: the game's colours, its two kinds
//! of type, and the link into the game.
//!
//! The page is the game's funeral ledger at a larger size: bone on
//! blue-black ink, ruled with hairlines, names in Fraunces and everything
//! else in DM Sans. leptoskit has no utility for a typeface, a line height
//! or letter spacing, so `assets/app.css` sets those by element: headings in
//! Fraunces, paragraphs in DM Sans with room between their lines. Colours,
//! sizes and layout are leptoskit's own classes.

use leptoskit::native::Role;
use leptoskit::prelude::*;

/// The page: a blue-black, never a neutral one.
pub const INK: &str = "#07080b";
/// A surface raised off the page.
pub const RAISED: &str = "#0f1116";
/// Text and rules.
pub const BONE: &str = "#e9e4d4";
/// Text that gives way to other text.
pub const ASH: &str = "#8e897d";
/// A rule that is barely there: bone at 18%.
pub const HAIRLINE: &str = "#e9e4d42e";
/// A husk drawing breath: hold still.
pub const AMBER: &str = "#ffb347";
/// A husk charging: swing.
pub const WOUND: &str = "#d4553f";
/// A soul, and the light a sigil burns with.
pub const SOUL: &str = "#ffc65a";

/// The column every section's content keeps to.
pub const COLUMN: &str = "w-full max-w-[1120px] mx-auto px-6 md:px-10";

/// A section of the page: a landmark named `name`, which in-page links
/// reach as `#id`.
pub fn section(id: &str, name: &str) -> Element {
	View()
		.role(Role::Region)
		.id(id)
		.aria_label(name)
		.class("relative w-full")
}

/// A section's title: the second level of the page's outline.
pub fn title(text: &str) -> Element {
	Heading(2, text).class(format!(
		"text-[40px] md:text-[64px] font-normal text-[{BONE}]"
	))
}

/// A title inside a section.
pub fn subtitle(text: &str) -> Element {
	Heading(3, text).class(format!(
		"text-[26px] md:text-[30px] font-normal text-[{BONE}]"
	))
}

/// A paragraph of running text, at `size` pixels, in `colour`.
pub fn paragraph(text: &str, size: u32, colour: &str) -> Element {
	View()
		.role(Role::Paragraph)
		.class(format!("text-[{size}px] text-[{colour}]"))
		.child(Text(text))
}

/// Running text at the page's usual size.
pub fn prose(text: &str) -> Element {
	paragraph(text, 17, BONE)
}

/// Small capitals in the margin: what a group of things is.
pub fn label(text: &str, colour: &str) -> Element {
	Text(text.to_uppercase()).class(format!("text-[12px] font-semibold text-[{colour}]"))
}

/// How loud a link into the game is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
	/// Outlined in bone, the page's one call to act.
	Primary,
	/// Words alone.
	Quiet,
}

/// A link: to the game, or to a place on this page. It is an ordinary
/// `<a href>`, so it opens in a new tab and works with the keyboard as any
/// link does.
pub fn link(text: &str, href: &str, weight: Weight) -> Element {
	let frame = match weight {
		Weight::Primary => {
			format!(
				"min-h-[56px] px-6 justify-center items-center border border-[{BONE}b3] \
				 text-[{BONE}]"
			)
		}
		Weight::Quiet => format!("min-h-[56px] justify-center items-center text-[{ASH}]"),
	};
	Pressable(text)
		.href(href)
		.class(frame)
		.child(Text(text).class("text-[16px] font-medium"))
}

/// A thin rule across the column.
pub fn rule() -> Element {
	View()
		.aria_hidden(true)
		.class(format!("w-full h-[1px] bg-[{HAIRLINE}]"))
}
