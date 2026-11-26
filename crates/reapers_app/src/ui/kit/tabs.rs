//! The tab bar (§8): five places, each a drawn icon over a label, under a
//! rough raised rule.

use leptoskit::native::Current;
use leptoskit::prelude::*;

use super::Action;
use super::State;
use super::crop_marks;
use super::with_tap;
use crate::theme::ASH;
use crate::theme::PAPER;
use crate::theme::RAISED;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::space;
use crate::theme::stroke;
use crate::ui::ink::Ink;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::text::label;

/// The icon box: icons are drawn round its centre, in the 24-point box the
/// bible gives them with room for their overshoot.
const ICON: f32 = 32.0;

/// The five icons, one stroke family: rough seeded paths, two passes, round
/// caps, 1.7 points, no fill but the active hachure (§8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabIcon {
	/// The sanctum: a lamp on its hook.
	Lamp,
	/// The vault: a lidded box.
	Box,
	/// The armory: a blade.
	Blade,
	/// The codex: an open book.
	Book,
	/// The rifts: a cracked ring.
	Crack,
}

impl TabIcon {
	/// The icon's strokes, and the outline its active hachure fills.
	fn marks(self) -> (Vec<Mark>, Vec<(f32, f32)>) {
		let (x, y) = (ICON / 2.0, ICON / 2.0);
		match self {
			Self::Lamp => {
				let ring: Vec<(f32, f32)> = (0..16)
					.map(|step| {
						let angle = step as f32 / 16.0 * std::f32::consts::TAU;
						(x + 8.0 * angle.cos(), y + 1.0 + 8.0 * angle.sin())
					})
					.collect();
				(
					vec![
						Mark::circle(x, y + 1.0, 9.0),
						Mark::line(x, y - 14.0, x, y - 9.0),
					],
					ring,
				)
			}
			Self::Box => {
				let outline = [
					(x - 10.0, y - 6.0),
					(x + 10.0, y - 6.0),
					(x + 10.0, y + 9.0),
					(x - 10.0, y + 9.0),
				];
				(
					vec![
						Mark::path(&outline, true),
						Mark::line(x - 11.0, y - 1.0, x + 11.0, y - 1.0),
					],
					outline.to_vec(),
				)
			}
			Self::Blade => {
				let outline = [
					(x - 9.0, y + 9.0),
					(x + 8.0, y - 10.0),
					(x + 10.0, y - 8.0),
					(x - 7.0, y + 11.0),
				];
				(
					vec![
						Mark::path(&outline, true),
						Mark::line(x - 11.0, y + 4.0, x - 3.0, y + 12.0),
					],
					outline.to_vec(),
				)
			}
			Self::Book => {
				let outline = [
					(x - 11.0, y - 7.0),
					(x, y - 4.0),
					(x + 11.0, y - 7.0),
					(x + 11.0, y + 8.0),
					(x, y + 10.0),
					(x - 11.0, y + 8.0),
				];
				(
					vec![
						Mark::path(&outline, true),
						Mark::line(x, y - 4.0, x, y + 10.0),
					],
					outline.to_vec(),
				)
			}
			Self::Crack => {
				let ring: Vec<(f32, f32)> = (0..16)
					.map(|step| {
						let angle = step as f32 / 16.0 * std::f32::consts::TAU;
						(x + 10.0 * angle.cos(), y + 12.0 * angle.sin())
					})
					.collect();
				(
					vec![
						Mark::path(
							&[
								(x - 2.0, y - 12.0),
								(x + 4.0, y - 4.0),
								(x - 3.0, y + 1.0),
								(x + 3.0, y + 11.0),
							],
							false,
						),
						Mark::ellipse(x, y, 11.0, 13.0),
					],
					ring,
				)
			}
		}
	}
}

/// One place on the tab bar.
pub struct Tab {
	pub icon: TabIcon,
	pub label: String,
	pub test_id: String,
	/// Whether it is the place the page is.
	pub active: bool,
	/// How it prints: at rest it answers the hand.
	pub state: State,
	/// Where it goes.
	pub open: Action,
}

/// One tab: 78 by 56 points, its icon and its label.
///
/// - **Inactive:** ash icon and label.
/// - **Active:** paper icon and label, spot hachure inside the icon, and a
///   spot underline 28 points long.
/// - **Pressed:** the hachure prints at once, held.
/// - **Focused:** crop marks.
fn tab(tab: Tab, spot: Rgb, seed: u32) -> Element {
	let Tab {
		icon,
		label: text,
		test_id,
		active,
		state,
		open,
	} = tab;
	let colour = if active { PAPER } else { ASH };
	let (strokes, outline) = icon.marks();
	let hachure = Recipe::once(Mark::hatch(&outline, 2.6, 60), seed.wrapping_add(9));
	let mut plate = Plate::new(ICON, ICON).stroke(&hachure, Ink::Current, 1.4);
	for (index, mark) in strokes.into_iter().enumerate() {
		plate = plate.stroke(
			&Recipe::twice(mark, seed.wrapping_add(index as u32 * 13)),
			colour,
			stroke::ICON,
		);
	}
	let hachure_ink = if active || state == State::Pressed {
		format!("text-[{}]", spot.hex())
	} else {
		format!("text-[#00000000] active:text-[{}]", spot.hex())
	};
	let mut cell = Pressable(text.clone())
		.class(format!(
			"relative w-[{}px] h-[{}px] items-center justify-start pt-[2px] {hachure_ink}",
			space::TAB_WIDTH,
			space::TAB_HEIGHT
		))
		.test_id(test_id)
		.child(plate.sized())
		.child(label(&text, Step::S1.size(), colour));
	if active {
		cell = cell.aria_current(Current::Page).child(
			View()
				.class("absolute left-[25px] bottom-[3px] w-[28px] h-[4px]")
				.child(
					Plate::new(28.0, 4.0)
						.stroke(
							&Recipe::once(Mark::line(0.5, 2.4, 27.5, 1.8), seed.wrapping_add(5)),
							spot,
							stroke::EMPHASIS,
						)
						.over(),
				),
		);
	}
	if state == State::Focused {
		cell = cell.child(crop_marks());
	}
	if state.takes_presses() {
		cell.on_press(with_tap(open))
	} else {
		cell
	}
}

/// The tab bar: a rough raised rule over five tabs, in the spot of the
/// bound reaper's light, padded clear of the home indicator.
pub fn tab_bar(tabs: Vec<Tab>, spot: Rgb) -> Element {
	let width = space::TAB_WIDTH as f32 * 5.0;
	let rule = Plate::new(width, 4.0).stroke(
		&Recipe::once(Mark::line(10.0, 2.0, width - 10.0, 2.4), 7701),
		RAISED,
		stroke::TAB_RULE,
	);
	let mut row = View().class("w-full flex-row justify-center");
	for (index, entry) in tabs.into_iter().enumerate() {
		row = row.child(tab(entry, spot, 7710 + index as u32 * 101));
	}
	View()
		.class("w-full items-center pb-safe")
		.role(leptoskit::native::Role::Navigation)
		.child(
			View()
				.class(format!("relative w-full max-w-[{width}px] h-[4px]"))
				.aria_hidden(true)
				.child(rule.over()),
		)
		.child(row)
}
