//! The kinds of type the interface is set in, after the Flutter game's
//! `Veil.display`, `label`, `body` and `figure`.
//!
//! Display type is a heading, so screen readers can move through a page by
//! its names and titles.

use std::cell::RefCell;
use std::collections::HashMap;

use leptoskit::native::vector::PathCommand;
use leptoskit::native::vector::Point;
use leptoskit::prelude::*;

use crate::theme::ASH;
use crate::theme::PAPER;
use crate::theme::Rgb;
use crate::theme::WOUND;

/// The class for type of `size` in `colour` after `prefix`, made once per
/// combination and kept: a screen asks for the same few on every render.
fn text_class(prefix: &'static str, size: u32, colour: Rgb) -> &'static str {
	thread_local! {
		static CLASSES: RefCell<HashMap<(&'static str, u32, Rgb), &'static str>> =
			RefCell::new(HashMap::new());
	}
	CLASSES.with_borrow_mut(|classes| {
		*classes.entry((prefix, size, colour)).or_insert_with(|| {
			// The combinations are the few the screens name, so the set is
			// small and fixed; keeping each for the life of the app is the
			// point of the cache.
			Box::leak(format!("text-[{size}px] {prefix}text-[{}]", colour.hex()).into_boxed_str())
		})
	})
}

/// Large words: names, titles, numerals.
pub fn display(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Heading(2, text.into()).class(text_class("font-normal ", size, colour))
}

/// Small capitals in the margin: labels, units, column heads.
pub fn label(text: &str, size: u32, colour: Rgb) -> Element {
	Text(text.to_uppercase()).class(text_class("font-semibold ", size, colour))
}

/// A label in ash at the usual size.
pub fn margin(text: &str) -> Element {
	label(text, 11, ASH)
}

/// Running text.
pub fn body(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(text_class("", size, colour))
}

/// Running text at the usual size, in paper.
pub fn prose(text: impl Into<String>) -> Element {
	body(text, 15, PAPER)
}

/// Running text that gives way to other text.
pub fn aside(text: impl Into<String>) -> Element {
	body(text, 13, ASH)
}

/// A figure in a table.
pub fn figure(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(text_class("font-medium ", size, colour))
}

/// A loss or an error on a dark ground: a cross in wound beside `words`,
/// which are set in paper at `size`. Wound never carries words on the page
/// (§15.1), so this is how a screen says something went wrong.
pub fn wound_words(words: Element, size: u32) -> Element {
	View()
		.class("flex-row items-start gap-2")
		.child(wound_cross(size))
		.child(View().class("flex-1 min-w-0").child(words))
}

/// A small drawn cross in wound, the height of a line of `size` type.
fn wound_cross(size: u32) -> Element {
	let side = size.max(11) as f32 * 0.8;
	let path = PathData::from_commands([
		PathCommand::MoveTo(Point::new(2.0, 2.0)),
		PathCommand::LineTo(Point::new(10.0, 10.0)),
		PathCommand::MoveTo(Point::new(10.4, 1.6)),
		PathCommand::LineTo(Point::new(1.8, 10.2)),
	])
	.expect("a cross is finite");
	Vector(ViewBox::new(0.0, 0.0, 12.0, 12.0).expect("a positive size"))
		.path(
			VectorPath::new(path).stroke(
				Stroke::new(WOUND.color(), 2.2)
					.expect("a positive width")
					.round(),
			),
		)
		.class(format!("w-[{side:.0}px] h-[{side:.0}px] mt-[3px]"))
		.aria_hidden(true)
}
