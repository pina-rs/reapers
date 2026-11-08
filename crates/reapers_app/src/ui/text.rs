//! The four kinds of type the ledger is set in, after the Flutter game's
//! `Veil.display`, `label`, `body` and `figure`.
//!
//! Display type is a heading, so the browser sets it in Fraunces (see
//! `assets/app.css`) and screen readers can move through a page by its
//! names and titles. Everything else is DM Sans on the web and the system
//! face on a phone.

use leptoskit::prelude::*;

use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;

/// Large words: names, titles, numerals.
pub fn display(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Heading(2, text.into()).class(format!(
		"text-[{size}px] font-normal text-[{}]",
		colour.hex()
	))
}

/// Small capitals in the margin: labels, units, column heads.
pub fn label(text: &str, size: u32, colour: Rgb) -> Element {
	Text(text.to_uppercase()).class(format!(
		"text-[{size}px] font-semibold text-[{}]",
		colour.hex()
	))
}

/// A label in ash at the usual size.
pub fn margin(text: &str) -> Element {
	label(text, 11, ASH)
}

/// Running text.
pub fn body(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(format!("text-[{size}px] text-[{}]", colour.hex()))
}

/// Running text at the usual size, in bone.
pub fn prose(text: impl Into<String>) -> Element {
	body(text, 15, BONE)
}

/// Running text that gives way to other text.
pub fn aside(text: impl Into<String>) -> Element {
	body(text, 13, ASH)
}

/// A figure in a table.
pub fn figure(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(format!(
		"text-[{size}px] font-medium text-[{}]",
		colour.hex()
	))
}
