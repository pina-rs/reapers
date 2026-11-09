//! The top of the page: a husk drawn out.
//!
//! As the page opens, a sigil's lights catch one after another and its
//! thread runs between them, and then the husk gathers itself into view in
//! the crosshair, as a hunt opens in the game. The words beside it say what
//! the game is. Under reduced motion it is all already there; scrolled
//! away, the husk bolts.

use std::time::Duration;

use leptoskit::prelude::*;

use super::art;
use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::SOUL;
use super::ui::Weight;
use super::ui::label;
use super::ui::link;
use super::ui::paragraph;
use super::ui::section;
use crate::config::app_url;
use crate::l10n::use_l10n;

/// The husk the page opens on.
const HUSK: &str = "asset://nfts/degenApes_1.webp";

/// How long a sigil's light waits after the one before it.
const CATCH: u64 = 420;
/// When the first light catches.
const FIRST: u64 = 350;

fn at(milliseconds: u64) -> Duration {
	Duration::from_millis(milliseconds)
}

/// When the last light has caught and the husk may appear.
fn drawn_out() -> u64 {
	FIRST + CATCH * (art::SIGIL.len() as u64)
}

/// One light of the sigil, catching at its turn.
fn light(index: usize, (x, y): (f32, f32)) -> Element {
	// The 100 by 75 box the sigil is laid out in, as percentages of the art.
	let (left, top) = (x, y / 75.0 * 100.0);
	let catch = Motion::enter(Pose::new().scale(0.2).opacity(0.0))
		.spring(Spring::BOUNCY)
		.delay(at(FIRST + CATCH * index as u64));
	View()
		.class(format!(
			"absolute left-[{left}%] top-[{top}%] w-6 h-6 -translate-x-1/2 -translate-y-1/2 \
			 items-center justify-center rounded-full bg-[{SOUL}26]"
		))
		.motion(catch)
		.child(View().class(format!("w-2 h-2 rounded-full bg-[{SOUL}]")))
}

/// The husk, its plate, and the sigil that drew it out.
fn drawing_out() -> Element {
	let thread = art::thread(&art::SIGIL, SOUL)
		.class("absolute inset-0 w-full h-full")
		.motion(
			Motion::enter(Pose::new().clip(Inset::from_left(0.0)))
				.duration(at(CATCH * (art::SIGIL.len() as u64 - 1)))
				.delay(at(FIRST))
				.easing(Easing::Linear),
		);
	let gather = Motion::enter(Pose::new().blur(18.0).opacity(0.0).scale(0.9))
		.duration(at(900))
		.delay(at(drawn_out()));
	let plate = View()
		.class(format!(
			"absolute left-[33%] top-[36%] w-[34%] aspect-square -rotate-3 border \
			 border-[{BONE}33]"
		))
		.motion(gather)
		.child(Image(HUSK).class("w-full h-full").aria_hidden(true));
	// Scrolled away, the husk bolts.
	let bolts = Motion::keyframes([
		(0.0, Pose::REST),
		(1.0, Pose::new().y(-48.0).rotate(-9.0).opacity(0.0)),
	])
	.easing(Easing::In)
	.in_view(ViewRange::exit());
	let sights = Motion::enter(Pose::new().scale(1.6).opacity(0.0))
		.spring(Spring::SNAPPY)
		.delay(at(drawn_out() + 450));
	let crosshair = View()
		.class("absolute left-[27%] top-[28%] w-[46%] aspect-square")
		.motion(sights)
		.child(art::crosshair(BONE).class("w-full h-full"));
	let mut art_box = View()
		.id("hero-art")
		.aria_hidden(true)
		.class("relative w-full md:w-[46%] aspect-[4/3] shrink-0")
		.child(View().class("absolute inset-0").motion(bolts).child(plate))
		.child(thread);
	for (index, point) in art::SIGIL.iter().enumerate() {
		art_box = art_box.child(light(index, *point));
	}
	art_box.child(crosshair)
}

/// The top of the page.
pub fn hero() -> Element {
	let words = use_l10n();
	let rise = |step: u64| {
		Motion::enter(Pose::new().y(18.0).opacity(0.0))
			.duration(at(700))
			.delay(at(120 * step))
	};
	let actions = View()
		.class("flex-row flex-wrap gap-x-4 gap-y-2 items-center")
		.motion(rise(3))
		.child(link(words.hero_play(), app_url(), Weight::Primary))
		.child(link(words.hero_how(), "#hunt", Weight::Quiet));
	let text = View()
		.class("w-full gap-6 md:flex-1")
		.child(label(words.hero_kicker(), ASH).motion(rise(0)))
		.child(
			Heading(1, words.hero_title())
				.class(format!(
					"text-[52px] md:text-[88px] font-normal text-[{BONE}]"
				))
				.motion(rise(1)),
		)
		.child(paragraph(words.hero_lead(), 19, BONE).motion(rise(2)))
		.child(actions)
		.child(paragraph(words.hero_status(), 14, ASH).motion(rise(4)));
	section("top", words.hero_title()).child(
		View()
			.class(format!(
				"{COLUMN} pt-4 pb-10 md:pt-16 md:pb-20 gap-8 md:gap-16 md:flex-row-reverse \
				 md:items-center"
			))
			.child(drawing_out())
			.child(text),
	)
}
