//! How a hunt plays, in four beats: draw it out, fight it, read the ring,
//! throw the orb. Each beat is a plate in the ledger with a drawing of what
//! the player sees.

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::art;
use super::ui::AMBER;
use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::HAIRLINE;
use super::ui::RAISED;
use super::ui::SOUL;
use super::ui::WOUND;
use super::ui::label;
use super::ui::paragraph;
use super::ui::prose;
use super::ui::section;
use super::ui::subtitle;
use super::ui::title;
use crate::l10n::use_l10n;

/// The husk the crosshair is on.
const QUARRY: &str = "asset://nfts/peskyPenguins_2.webp";

/// The ledger counts its beats in Roman numerals.
const NUMERALS: [&str; 4] = ["I", "II", "III", "IV"];

/// The box a beat's drawing sits in.
fn plate() -> Element {
	View().aria_hidden(true).class(format!(
		"relative w-full aspect-[4/3] overflow-hidden items-center justify-center \
			 bg-[{RAISED}] border border-[{HAIRLINE}]"
	))
}

/// I: the sigil, three lights caught and the ring on the fourth.
fn drawing_out() -> Element {
	plate().child(art::sigil(3, SOUL).class("w-[86%] h-[46%]"))
}

/// II: a husk in the crosshair, and the pistol's rounds.
fn fighting() -> Element {
	let rounds = View()
		.class("absolute right-4 bottom-4 flex-row items-center gap-1")
		.children((0..6).map(|_| View().class(format!("w-[6px] h-[10px] bg-[{BONE}cc]"))));
	plate()
		.child(Image(QUARRY).class("w-[54%] h-[72%]"))
		.child(art::crosshair(BONE).class("absolute left-[30%] top-[18%] w-[40%] h-[64%]"))
		.child(rounds)
}

/// III: the ring under a husk, in its two warnings.
fn ring(colour: &str, caption: &str) -> Element {
	View()
		.class("flex-1 items-center gap-3")
		.child(art::floor_ring(colour).class("w-full h-[64px]"))
		.child(label(caption, colour))
}

fn reading_the_ring(hold: &str, swing: &str) -> Element {
	plate()
		.class("flex-row px-6 gap-6")
		.child(ring(AMBER, hold))
		.child(ring(WOUND, swing))
}

/// IV: the orb, mid-pulse.
fn throwing() -> Element {
	plate().child(art::orb(SOUL).class("w-[60%] h-[80%]"))
}

/// One beat: its numeral, its name, what to do, and the drawing.
fn beat(index: usize, name: &str, text: &str, drawing: Element) -> Element {
	let arrive = Motion::enter(Pose::new().y(36.0).opacity(0.0)).in_view(ViewRange::during(
		ViewPhase::Entry,
		0.0,
		0.8,
	));
	View()
		.role(Role::ListItem)
		.class("w-full gap-5")
		.motion(arrive)
		.child(drawing)
		.child(
			View()
				.class("flex-row items-center gap-3")
				.child(
					Text(NUMERALS[index])
						.id(format!("numeral-{}", index + 1))
						.class(format!("text-[26px] md:text-[30px] text-[{SOUL}]")),
				)
				.child(subtitle(name)),
		)
		.child(paragraph(text, 16, ASH))
}

/// How a hunt plays.
pub fn hunt() -> Element {
	let words = use_l10n();
	let beats = View()
		.role(Role::List)
		.class("w-full grid grid-cols-1 md:grid-cols-2 gap-x-10 gap-y-16")
		.child(beat(
			0,
			words.hunt_draw_title(),
			words.hunt_draw_text(),
			drawing_out(),
		))
		.child(beat(
			1,
			words.hunt_fight_title(),
			words.hunt_fight_text(),
			fighting(),
		))
		.child(beat(
			2,
			words.hunt_ring_title(),
			words.hunt_ring_text(),
			reading_the_ring(words.hunt_ring_hold(), words.hunt_ring_swing()),
		))
		.child(beat(
			3,
			words.hunt_orb_title(),
			words.hunt_orb_text(),
			throwing(),
		));
	section("hunt", words.hunt_title()).child(
		View()
			.class(format!("{COLUMN} py-20 md:py-28 gap-12 md:gap-16"))
			.child(
				View()
					.class("w-full gap-6 md:flex-row md:items-end md:justify-between")
					.child(title(words.hunt_title()).class("md:w-[44%]"))
					.child(
						View()
							.class("w-full md:w-[48%]")
							.child(prose(words.hunt_intro())),
					),
			)
			.child(beats),
	)
}
