//! The four kinds of type the ledger is set in, after the Flutter game's
//! `Veil.display`, `label`, `body` and `figure`.
//!
//! Display type is a heading, so the browser sets it in Fraunces (see
//! `assets/app.css`) and screen readers can move through a page by its
//! names and titles. Everything else is DM Sans on the web and the system
//! face on a phone.

use std::borrow::Cow;

use leptoskit::prelude::*;

use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::INK;
use crate::theme::Rgb;
use crate::theme::WOUND;

fn text_class(prefix: &str, size: u32, colour: Rgb) -> Cow<'static, str> {
	match (prefix, size, colour) {
		("", 12, ASH) => Cow::Borrowed("text-[12px] text-[#8e897d]"),
		("", 12, BONE) => Cow::Borrowed("text-[12px] text-[#e9e4d4]"),
		("", 13, ASH) => Cow::Borrowed("text-[13px] text-[#8e897d]"),
		("", 13, BONE) => Cow::Borrowed("text-[13px] text-[#e9e4d4]"),
		("", 14, ASH) => Cow::Borrowed("text-[14px] text-[#8e897d]"),
		("", 14, BONE) => Cow::Borrowed("text-[14px] text-[#e9e4d4]"),
		("", 15, ASH) => Cow::Borrowed("text-[15px] text-[#8e897d]"),
		("", 15, BONE) => Cow::Borrowed("text-[15px] text-[#e9e4d4]"),
		("", 15, WOUND) => Cow::Borrowed("text-[15px] text-[#d4553f]"),
		("", 18, BONE) => Cow::Borrowed("text-[18px] text-[#e9e4d4]"),
		("font-semibold ", 9, ASH) => Cow::Borrowed("text-[9px] font-semibold text-[#8e897d]"),
		("font-semibold ", 9, BONE) => Cow::Borrowed("text-[9px] font-semibold text-[#e9e4d4]"),
		("font-semibold ", 10, ASH) => Cow::Borrowed("text-[10px] font-semibold text-[#8e897d]"),
		("font-semibold ", 10, BONE) => Cow::Borrowed("text-[10px] font-semibold text-[#e9e4d4]"),
		("font-semibold ", 11, ASH) => Cow::Borrowed("text-[11px] font-semibold text-[#8e897d]"),
		("font-semibold ", 11, BONE) => Cow::Borrowed("text-[11px] font-semibold text-[#e9e4d4]"),
		("font-semibold ", 12, ASH) => Cow::Borrowed("text-[12px] font-semibold text-[#8e897d]"),
		("font-semibold ", 12, BONE) => Cow::Borrowed("text-[12px] font-semibold text-[#e9e4d4]"),
		("font-semibold ", 13, ASH) => Cow::Borrowed("text-[13px] font-semibold text-[#8e897d]"),
		("font-semibold ", 13, BONE) => Cow::Borrowed("text-[13px] font-semibold text-[#e9e4d4]"),
		("font-semibold ", 13, INK) => Cow::Borrowed("text-[13px] font-semibold text-[#07080b]"),
		("font-semibold ", 15, BONE) => Cow::Borrowed("text-[15px] font-semibold text-[#e9e4d4]"),
		("font-medium ", 12, BONE) => Cow::Borrowed("text-[12px] font-medium text-[#e9e4d4]"),
		("font-medium ", 13, ASH) => Cow::Borrowed("text-[13px] font-medium text-[#8e897d]"),
		("font-medium ", 13, BONE) => Cow::Borrowed("text-[13px] font-medium text-[#e9e4d4]"),
		("font-medium ", 14, BONE) => Cow::Borrowed("text-[14px] font-medium text-[#e9e4d4]"),
		("font-medium ", 15, BONE) => Cow::Borrowed("text-[15px] font-medium text-[#e9e4d4]"),
		("font-normal ", 22, BONE) => Cow::Borrowed("text-[22px] font-normal text-[#e9e4d4]"),
		("font-normal ", 24, BONE) => Cow::Borrowed("text-[24px] font-normal text-[#e9e4d4]"),
		("font-normal ", 28, BONE) => Cow::Borrowed("text-[28px] font-normal text-[#e9e4d4]"),
		("font-normal ", 36, BONE) => Cow::Borrowed("text-[36px] font-normal text-[#e9e4d4]"),
		("font-normal ", 40, BONE) => Cow::Borrowed("text-[40px] font-normal text-[#e9e4d4]"),
		_ => {
			Cow::Owned(format!(
				"text-[{size}px] {prefix}text-[#{:02x}{:02x}{:02x}]",
				colour.0, colour.1, colour.2
			))
		}
	}
}

/// Large words: names, titles, numerals.
pub fn display(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Heading(2, text.into()).class(text_class("font-normal ", size, colour).as_ref())
}

/// Small capitals in the margin: labels, units, column heads.
pub fn label(text: &str, size: u32, colour: Rgb) -> Element {
	Text(text.to_uppercase()).class(text_class("font-semibold ", size, colour).as_ref())
}

/// A label in ash at the usual size.
pub fn margin(text: &str) -> Element {
	label(text, 11, ASH)
}

/// Running text.
pub fn body(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(text_class("", size, colour).as_ref())
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
	Text(text.into()).class(text_class("font-medium ", size, colour).as_ref())
}
