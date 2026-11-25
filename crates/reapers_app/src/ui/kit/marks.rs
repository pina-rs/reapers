//! The marks (§8): what a screen counts and measures with. The drawn arrow,
//! the class mark and the rarity pips; the tally of the taken; the level
//! stamp; the soul dial; the lamp; and the meter.

use leptoskit::prelude::*;
use reapers_rules::CaptureRules;

use super::steady;
use crate::l10n::use_l10n;
use crate::theme::ASH;
use crate::theme::KEY;
use crate::theme::PAPER;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::WOUND;
use crate::theme::fit_display_line;
use crate::theme::stroke;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::text::display_words;
use crate::ui::text::figure;
use crate::ui::text::label;
use crate::ui::text::wound_words;

/// An arrow drawn by hand, for "2 nearby →": a shaft and an open head, in
/// the icons' stroke (§8, "Arrows and swap marks are drawn").
pub fn arrow(colour: Rgb, seed: u32) -> Element {
	Plate::new(16.0, 12.0)
		.stroke(
			&Recipe::twice(Mark::line(1.5, 6.0, 14.0, 6.2), seed),
			colour,
			stroke::ICON,
		)
		.stroke(
			&Recipe::once(
				Mark::path(&[(9.5, 1.8), (14.4, 6.1), (9.5, 10.4)], false),
				seed.wrapping_add(1),
			),
			colour,
			stroke::ICON,
		)
		.sized()
}

/// A registration mark: a circle of `radius` crossed by a hair `span` long.
fn registration(radius: f32, span: f32, colour: Rgb, width: f32, seed: u32) -> Element {
	let side = span + 2.0;
	let centre = side / 2.0;
	Plate::new(side, side)
		.stroke(
			&Recipe::once(Mark::circle(centre, centre, radius), seed),
			colour,
			width,
		)
		.stroke(
			&Recipe::once(
				Mark::line(1.0, centre, side - 1.0, centre),
				seed.wrapping_add(1),
			),
			colour,
			width,
		)
		.stroke(
			&Recipe::once(
				Mark::line(centre, 1.0, centre, side - 1.0),
				seed.wrapping_add(2),
			),
			colour,
			width,
		)
		.sized()
}

/// The class mark: a registration mark in the spot, a circle of 6 points
/// crossed by an 18-point hair (§8).
pub fn class_mark(spot: Rgb, seed: u32) -> Element {
	registration(6.0, 18.0, spot, stroke::MARK, seed)
}

/// Rarity pips: `count` registration marks beside a name, one for common up
/// to four for epic, in paper (§8). There is no rarity colour.
pub fn pips(count: u8, seed: u32) -> Element {
	(0..count.clamp(1, 4)).fold(
		View().class("flex-row gap-[3px]").aria_hidden(true),
		|row, index| {
			row.child(registration(
				3.6,
				11.0,
				PAPER,
				1.1,
				seed.wrapping_add(u32::from(index) * 17),
			))
		},
	)
}

/// The tally of the taken: gates of five rough paper strokes, the newest
/// stroke in the spot at 2.2 points (§8). At most `gates` gates are drawn;
/// the figure beside the tally always says the whole count.
pub fn tally(count: u32, gates: u32, spot: Rgb, seed: u32) -> Element {
	const STROKE: f32 = 5.2;
	const GATE: f32 = 29.0;
	const HEIGHT: f32 = 24.0;
	let shown = count.min(gates.max(1) * 5);
	let drawn_gates = shown.div_ceil(5).max(1);
	let width = drawn_gates as f32 * GATE;
	let mut plate = Plate::new(width, HEIGHT);
	for index in 0..shown {
		let gate = index / 5;
		let place = index % 5;
		let left = gate as f32 * GATE + 2.0;
		let newest = index + 1 == shown;
		let (colour, width) = if newest {
			(spot, stroke::EMPHASIS)
		} else {
			(PAPER, stroke::TALLY)
		};
		let mark = if place == 4 {
			// The fifth closes the gate, across the other four.
			Mark::line(left - 2.0, HEIGHT - 6.0, left + 3.0 * STROKE + 3.0, 6.0)
		} else {
			let x = left + place as f32 * STROKE;
			Mark::line(x, 2.0, x + 0.6, HEIGHT - 2.0)
		};
		plate = plate.stroke(
			&Recipe::once(mark, seed.wrapping_add(index * 31)),
			colour,
			width,
		);
	}
	plate.sized()
}

/// The level stamp, a chop: a rough spot ring of two passes and a broken
/// inner ring, turned −6 to −9° by `seed`, round the word "Level" in paper
/// and the numeral in Display in the spot (§8, §9). It is pressed in: it
/// lands from 1.15 times its size, and appears in place under reduced
/// motion.
pub fn stamp(numeral: &str, spot: Rgb, seed: u32) -> Element {
	const SIDE: f32 = 70.0;
	let words = use_l10n();
	let turn = 6 + seed % 4;
	let centre = SIDE / 2.0;
	let ring = Plate::new(SIDE, SIDE)
		.stroke(
			&Recipe::twice(Mark::circle(centre, centre, 32.0), seed),
			spot,
			stroke::STAMP,
		)
		.stroke(
			&Recipe::once(
				Mark::arc(centre, centre, 27.0, 2.3, 7.6),
				seed.wrapping_add(1),
			),
			spot,
			1.1,
		);
	let size = fit_display_line(numeral, 44.0, Step::S6);
	let press = if steady() {
		String::new()
	} else {
		"starting:scale-[1.15] transition-transform duration-[166ms] ease-out".to_owned()
	};
	View()
		.class(format!(
			"relative w-[{SIDE}px] h-[{SIDE}px] items-center justify-center -rotate-[{turn}deg] {press}"
		))
		.accessibility_label(format!("{} {numeral}", words.common_level()))
		.child(ring.over())
		.child(label(words.common_level(), Step::S1.size(), PAPER).class("mt-[4px]"))
		.child(display_words(numeral, size, spot).class("mt-[-4px]"))
}

/// The soul dial round the orb (§8): a ring of 7.5 points in key under the
/// spot's arc of what is held, and a paper notch at 40, an orb's cost.
/// Under 40 the arc prints in ash and the dial says what it needs.
pub fn soul_dial(soul: f64, spot: Rgb, seed: u32) -> Element {
	const SIDE: f32 = 72.0;
	const RADIUS: f32 = 26.0;
	let words = use_l10n();
	let centre = SIDE / 2.0;
	let soul = soul.clamp(0.0, CaptureRules::SOUL_MAX);
	let ready = soul >= CaptureRules::COST;
	// Kept to whole hundredths, so the dial draws at most a hundred arcs.
	let share = (soul / CaptureRules::SOUL_MAX * 100.0).round() as f32 / 100.0;
	let start = -std::f32::consts::FRAC_PI_2;
	let tau = std::f32::consts::TAU;
	let notch = start + tau * (CaptureRules::COST / CaptureRules::SOUL_MAX) as f32;
	let (cos, sin) = (notch.cos(), notch.sin());
	let mut plate = Plate::new(SIDE, SIDE).stroke(
		&Recipe::once(Mark::circle(centre, centre, RADIUS), seed),
		KEY,
		7.5,
	);
	if share > 0.0 {
		plate = plate.stroke(
			&Recipe::twice(
				Mark::arc(centre, centre, RADIUS, start, start + tau * share),
				seed.wrapping_add(1),
			),
			if ready { spot } else { ASH },
			4.0,
		);
	}
	plate = plate.stroke(
		&Recipe::once(
			Mark::line(
				centre + cos * (RADIUS - 6.0),
				centre + sin * (RADIUS - 6.0),
				centre + cos * (RADIUS + 6.0),
				centre + sin * (RADIUS + 6.0),
			),
			seed.wrapping_add(2),
		),
		PAPER,
		stroke::EMPHASIS,
	);
	let figure_text = format!("{}", soul.floor() as i64);
	let mut dial = View()
		.class("items-center gap-1")
		.role(leptoskit::native::Role::Meter)
		.accessibility_label(words.dial_soul())
		.aria_value_now(soul)
		.aria_value_min(0.0)
		.aria_value_max(CaptureRules::SOUL_MAX)
		.child(
			View()
				.class(format!(
					"relative w-[{SIDE}px] h-[{SIDE}px] items-center justify-center"
				))
				.child(plate.over())
				.child(figure(figure_text, Step::S3.size(), PAPER)),
		);
	if !ready {
		dial = dial.child(label(
			&words.dial_needs(CaptureRules::COST as i64),
			Step::S1.size(),
			ASH,
		));
	}
	dial
}

/// The lamp (§8): experience to the next level as a waxing disc of
/// `radius` points, the earned part solid spot, the rest a spot halftone,
/// inside a rough two-pass ring. Full, the ring doubles.
pub fn lamp(radius: f32, fraction: f64, spot: Rgb, seed: u32) -> Element {
	let side = 2.0 * radius + 16.0;
	let centre = side / 2.0;
	let full = fraction >= 1.0;
	let mut plate = Plate::new(side, side)
		.fill(
			&Recipe::once(Mark::halftone(centre, centre, radius), 0),
			spot,
		)
		.fill(
			&Recipe::once(Mark::waxing(centre, centre, radius, fraction), 0),
			spot,
		)
		.stroke(
			&Recipe::twice(Mark::circle(centre, centre, radius + 3.0), seed),
			spot,
			1.4,
		);
	if full {
		plate = plate.stroke(
			&Recipe::twice(
				Mark::circle(centre, centre, radius + 6.5),
				seed.wrapping_add(1),
			),
			spot,
			1.4,
		);
	}
	plate.sized()
}

/// A meter (§8): a paper strip with a key outline, what remains hatched in
/// key at 60°, what is gone bare paper, and the figure beside it. When the
/// meter is a loss (resolve) and under 15%, the figure is struck in wound
/// and `warning` says so in words; a husk's ward is never struck.
pub fn meter(
	fraction: f64,
	figure_text: &str,
	warning: Option<&str>,
	width: f32,
	seed: u32,
) -> Element {
	const HEIGHT: f32 = 12.0;
	let fraction = fraction.clamp(0.0, 1.0);
	// Kept to whole percents, so a draining meter draws at most a hundred
	// hatches.
	let remaining = ((fraction * 100.0).round() / 100.0) as f32 * width;
	let mut plate = Plate::new(width, HEIGHT).flood(0.0, 0.0, width, HEIGHT, PAPER);
	if remaining > 1.0 {
		plate = plate.stroke(
			&Recipe::once(
				Mark::hatch_box(0.0, 0.0, remaining, HEIGHT, 2.4, 60),
				seed.wrapping_add(1),
			),
			KEY,
			1.0,
		);
	}
	plate = plate.stroke(
		&Recipe::once(Mark::frame(width, HEIGHT, 1.5), seed),
		KEY,
		stroke::METER,
	);
	let critical = fraction < 0.15;
	let mut reading =
		View()
			.class("relative")
			.child(figure(figure_text.to_owned(), Step::S2.size(), PAPER));
	let mut row = View()
		.class("flex-row items-center gap-3")
		.role(leptoskit::native::Role::Meter)
		.aria_value_now(fraction * 100.0)
		.aria_value_min(0.0)
		.aria_value_max(100.0)
		.aria_value_text(figure_text.to_owned())
		.child(
			View()
				.class(format!("relative w-[{width}px] h-[{HEIGHT}px]"))
				.child(plate.over()),
		);
	match warning {
		Some(words) if critical => {
			reading = reading.child(
				View()
					.class("absolute left-[-2px] right-[-2px] top-[7px] h-[4px]")
					.child(
						Plate::new(40.0, 4.0)
							.stroke(
								&Recipe::once(
									Mark::line(0.5, 2.6, 39.5, 1.6),
									seed.wrapping_add(2),
								),
								WOUND,
								stroke::EMPHASIS,
							)
							.over(),
					),
			);
			row = row
				.child(reading)
				.child(wound_words(label(words, Step::S1.size(), PAPER), 11));
		}
		_ => row = row.child(reading),
	}
	row
}
