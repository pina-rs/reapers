//! The three faces the interface is set in (§7), and the helpers screens
//! set words with.
//!
//! - **Display** ([`display`], [`fitted_display`]): names, stamps, the one
//!   primary action. A heading, so screen readers can move through a page
//!   by its names and titles.
//! - **Text** ([`body`], [`prose`], [`aside`]): sentences. Sized in rem, so
//!   they grow with the reader's text size.
//! - **Figures** ([`figure`], [`label`], [`margin`]): every number, with
//!   tabular figures, and every uppercase label, tracked.
//!
//! Each face is named by a CSS custom property the web shell defines
//! (`assets/app.css`), with its fallback chain, and set with one weight
//! only: nothing is ever made bold.

use std::cell::RefCell;
use std::collections::HashMap;

use leptoskit::native::vector::PathCommand;
use leptoskit::native::vector::Point;
use leptoskit::prelude::*;

use crate::theme::ASH;
use crate::theme::Face;
use crate::theme::PAPER;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::WOUND;
use crate::theme::fit_display;

/// What a run of words is set as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Setting {
	Display,
	Sentence,
	Figure,
	Label,
}

/// The line height for `size` points of type set as `setting`: the
/// scale's own where `size` is on it, and its nearest step's ratio between.
fn leading(setting: Setting, size: u32) -> f32 {
	let step = Step::ALL
		.into_iter()
		.min_by_key(|step| step.size().abs_diff(size))
		.unwrap_or(Step::S3);
	let ratio = step.leading() as f32 / step.size() as f32;
	match setting {
		// Display sets tight: never looser than its size.
		Setting::Display => ratio.min(1.1),
		Setting::Sentence | Setting::Figure | Setting::Label => ratio,
	}
}

/// The class for type of `size` in `colour` set as `setting`, made once
/// per combination and kept: a screen asks for the same few on every
/// render.
fn type_class(setting: Setting, size: u32, colour: Rgb) -> &'static str {
	thread_local! {
		static CLASSES: RefCell<HashMap<(Setting, u32, Rgb), &'static str>> =
			RefCell::new(HashMap::new());
	}
	CLASSES.with_borrow_mut(|classes| {
		*classes.entry((setting, size, colour)).or_insert_with(|| {
			let colour = colour.hex();
			let line = leading(setting, size);
			let class = match setting {
				Setting::Display => {
					format!(
						"{} text-[{size}px] leading-[{:.0}px] text-[{colour}]",
						Face::Display.class(),
						size as f32 * line
					)
				}
				// Sentences scale with the reader's text size: rem, with a
				// line height that is a multiple of it.
				Setting::Sentence => {
					format!(
						"{} text-[{:.4}rem] leading-[{line:.3}] text-[{colour}]",
						Face::Text.class(),
						size as f32 / 16.0
					)
				}
				Setting::Figure => {
					format!(
						"{} tabular-nums text-[{size}px] leading-[{:.0}px] text-[{colour}]",
						Face::Figures.class(),
						size as f32 * line
					)
				}
				Setting::Label => {
					format!(
						"{} tracking-widest text-[{size}px] leading-[{:.0}px] text-[{colour}]",
						Face::Figures.class(),
						size as f32 * line
					)
				}
			};
			// The combinations are the few the screens name, so the set is
			// small and fixed; keeping each for the life of the app is the
			// point of the cache.
			Box::leak(class.into_boxed_str())
		})
	})
}

/// Large words in Display: names, titles, numerals, the primary action.
pub fn display(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Heading(2, text.into()).class(type_class(Setting::Display, size, colour))
}

/// A name or a chop in a box `width` points wide: Display at `step`, shrunk
/// to fit two lines and never under s5 (§7.2).
pub fn fitted_display(text: impl Into<String>, width: f32, step: Step, colour: Rgb) -> Element {
	let text = text.into();
	let size = fit_display(&text, width, step);
	display(text, size, colour)
}

/// An uppercase label in Figures, tracked: captions, tabs, field labels.
pub fn label(text: &str, size: u32, colour: Rgb) -> Element {
	Text(text.to_uppercase()).class(type_class(Setting::Label, size, colour))
}

/// A label in ash at the smallest step.
pub fn margin(text: &str) -> Element {
	label(text, Step::S1.size(), ASH)
}

/// A sentence in Text.
pub fn body(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(type_class(Setting::Sentence, size, colour))
}

/// A sentence at the scale's sentence step, in paper.
pub fn prose(text: impl Into<String>) -> Element {
	body(text, Step::S3.size(), PAPER)
}

/// A sentence that gives way to other words.
pub fn aside(text: impl Into<String>) -> Element {
	body(text, Step::S3.size(), ASH)
}

/// A number in Figures, with tabular figures so a count never jitters.
pub fn figure(text: impl Into<String>, size: u32, colour: Rgb) -> Element {
	Text(text.into()).class(type_class(Setting::Figure, size, colour))
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn each_face_is_named_by_its_variable() {
		assert!(type_class(Setting::Display, 40, PAPER).starts_with("font-[var(--display)]"));
		assert!(type_class(Setting::Sentence, 14, PAPER).starts_with("font-[var(--text)]"));
		assert!(type_class(Setting::Figure, 14, PAPER).contains("tabular-nums"));
		assert!(type_class(Setting::Label, 11, ASH).contains("tracking-widest"));
	}

	#[test]
	fn sentences_scale_with_the_readers_text_size() {
		let class = type_class(Setting::Sentence, 14, PAPER);
		assert!(class.contains("text-[0.8750rem]"), "{class}");
		assert!(class.contains("leading-[1.357]"), "{class}");
	}

	#[test]
	fn the_scale_sets_its_own_line_heights() {
		assert!(type_class(Setting::Display, 40, PAPER).contains("leading-[38px]"));
		assert!(type_class(Setting::Display, 30, PAPER).contains("leading-[30px]"));
		assert!(type_class(Setting::Label, 11, ASH).contains("leading-[14px]"));
		assert!(type_class(Setting::Figure, 12, ASH).contains("leading-[16px]"));
	}

	#[test]
	fn nothing_is_made_bold() {
		for setting in [
			Setting::Display,
			Setting::Sentence,
			Setting::Figure,
			Setting::Label,
		] {
			for step in Step::ALL {
				let class = type_class(setting, step.size(), PAPER);
				assert!(!class.contains("font-semibold"), "{class}");
				assert!(!class.contains("font-bold"), "{class}");
				assert!(!class.contains("font-medium"), "{class}");
			}
		}
	}
}
