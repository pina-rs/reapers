//! The capture, as the page scrolls: the orb pulses once, twice, three
//! times, and seals.
//!
//! The section is a stage. Where the browser can drive animations from the
//! scroll, `assets/app.css` makes it three screens tall and pins its middle,
//! so the reader's own scrolling counts the pulses: each one swells the orb,
//! sends a ring out across the floor and lights its word. Anywhere else, and
//! for anyone who asks for less motion, it is one screen with the orb, its
//! rings and the words all at rest: "One. Two. Three. Sealed."

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::art;
use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::SOUL;
use super::ui::label;
use super::ui::paragraph;
use super::ui::section;
use crate::l10n::use_l10n;

/// The stage's scroll, which everything in it follows.
const TIMELINE: &str = "capture";

/// Where in the stage's scroll each pulse lands.
const PULSES: [f32; 3] = [0.3, 0.5, 0.7];
/// Where the orb seals.
const SEALED: f32 = 0.86;

/// `poses` played as the stage scrolls past.
fn scrolled(poses: impl IntoIterator<Item = (f32, Pose)>) -> Motion {
	Motion::keyframes(poses)
		.easing(Easing::Linear)
		.follow(TIMELINE, ViewRange::contain())
}

/// The orb: it gathers, swells at each pulse, and settles as it seals.
fn orb() -> Element {
	let swell = Pose::new().scale(1.2);
	let mut poses = vec![
		(0.0, Pose::new().scale(0.3).opacity(0.0)),
		(0.14, Pose::REST),
	];
	for pulse in PULSES {
		poses.extend([
			(pulse - 0.02, Pose::REST),
			(pulse + 0.01, swell),
			(pulse + 0.06, Pose::REST),
		]);
	}
	poses.extend([
		(SEALED - 0.03, Pose::REST),
		(SEALED, Pose::new().scale(0.86)),
		(1.0, Pose::new().scale(0.86)),
	]);
	View()
		.id("capture-orb")
		.aria_hidden(true)
		.class("absolute inset-0 items-center justify-center")
		.motion(scrolled(poses))
		.child(art::orb(SOUL).class("w-full h-full"))
}

/// The ring a pulse sends out: at rest, one of three ripples round the orb.
fn ripple(index: usize, pulse: f32) -> Element {
	let size = 118 + 26 * index;
	let quiet = Pose::new().scale(0.62).opacity(0.0);
	View()
		.aria_hidden(true)
		.class(format!(
			"absolute left-1/2 top-1/2 w-[{size}%] h-[{size}%] -translate-x-1/2 -translate-y-1/2 \
			 rounded-full border border-[{SOUL}66]"
		))
		.motion(scrolled([
			(0.0, quiet),
			(pulse, quiet),
			(pulse + 0.01, Pose::new().scale(0.62).opacity(1.0)),
			(pulse + 0.14, Pose::new().scale(1.5).opacity(0.0)),
			(1.0, Pose::new().scale(1.5).opacity(0.0)),
		]))
}

/// A word of the count, ghosted until its pulse lands.
fn word(text: &str, at: f32, colour: &str) -> Element {
	let ghost = Pose::new().opacity(0.14);
	Text(text)
		.class(format!("text-[44px] md:text-[76px] text-[{colour}]"))
		.motion(scrolled([
			(0.0, ghost),
			(at - 0.02, ghost),
			(at + 0.02, Pose::REST),
			(1.0, Pose::REST),
		]))
}

/// The capture.
pub fn capture() -> Element {
	let words = use_l10n();
	let count = View()
		.id("capture-words")
		.role(Role::Paragraph)
		.class("flex-row flex-wrap justify-center gap-x-5 md:gap-x-8")
		.child(word(words.orb_one(), PULSES[0], BONE))
		.child(word(words.orb_two(), PULSES[1], BONE))
		.child(word(words.orb_three(), PULSES[2], BONE))
		.child(word(words.orb_sealed(), SEALED, SOUL));
	let mut stage = View()
		.class("relative w-[200px] h-[200px] md:w-[260px] md:h-[260px]")
		.child(orb());
	for (index, pulse) in PULSES.iter().enumerate() {
		stage = stage.child(ripple(index, *pulse));
	}
	let note = paragraph(words.orb_fail_note(), 16, ASH)
		.class("text-center")
		.motion(scrolled([
			(0.0, Pose::new().opacity(0.0)),
			(SEALED + 0.03, Pose::new().opacity(0.0)),
			(SEALED + 0.08, Pose::REST),
			(1.0, Pose::REST),
		]));
	section("capture", words.orb_label())
		.timeline(TIMELINE)
		.child(
			View()
				.id("capture-pin")
				.class(format!(
					"{COLUMN} min-h-screen py-24 items-center justify-center gap-14"
				))
				.child(label(words.orb_label(), ASH))
				.child(stage)
				.child(View().class("items-center gap-6").child(count).child(note)),
		)
}
