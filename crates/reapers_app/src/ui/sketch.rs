//! The hand that rules every box in the ledger.
//!
//! The Flutter game drew its boxes with Skribble: two passes of a fine pen
//! that bow away from the corner and never quite meet. This draws the same
//! hand as one [`Vector`] path: each edge twice, as a cubic whose ends wander
//! and whose middle bows, from a generator seeded by the box's label, so a
//! rebuild redraws the same line and the ink does not crawl.
//!
//! The path is made once per seed and kept, so a box costs a pointer
//! compare on every render after its first. It is drawn in a 100 by 100 box
//! stretched over the frame, with a stroke that keeps its width in layout
//! units, and it sits beneath the content so it never takes a press.

use std::cell::RefCell;
use std::collections::HashMap;

use leptoskit::native::vector::PathCommand;
use leptoskit::native::vector::Point;
use leptoskit::prelude::*;

use crate::theme::Rgb;

/// The width of a hand-drawn rule, in layout units.
pub const THICKNESS: f32 = 1.15;

/// How far a line's ends wander from the corner, in hundredths of the box.
const WANDER: f32 = 1.4;

/// How far a line bows from straight, in hundredths of the box.
const BOW: f32 = 0.9;

/// A small, fixed generator: the same seed always draws the same line.
struct Hand(u32);

impl Hand {
	fn next(&mut self) -> f32 {
		// Mulberry32.
		self.0 = self.0.wrapping_add(0x6D2B_79F5);
		let mut t = self.0;
		t = (t ^ (t >> 15)).wrapping_mul(t | 1);
		t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 0x3D));
		((t ^ (t >> 14)) as f32) / 4_294_967_296.0
	}

	/// A value between `-amount` and `amount`.
	fn wobble(&mut self, amount: f32) -> f32 {
		(self.next() * 2.0 - 1.0) * amount
	}
}

/// The commands for one stroke of the pen from `from` to `to`.
fn stroke(hand: &mut Hand, from: Point, to: Point) -> [PathCommand; 2] {
	let (dx, dy) = (to.x - from.x, to.y - from.y);
	let length = dx.hypot(dy).max(1.0);
	let (ux, uy) = (dx / length, dy / length);
	// Run a little short of the corner or a little past it.
	let start = Point::new(
		from.x - ux * hand.wobble(WANDER) + hand.wobble(WANDER * 0.5),
		from.y - uy * hand.wobble(WANDER) + hand.wobble(WANDER * 0.5),
	);
	let end = Point::new(
		to.x + ux * hand.wobble(WANDER) + hand.wobble(WANDER * 0.5),
		to.y + uy * hand.wobble(WANDER) + hand.wobble(WANDER * 0.5),
	);
	let bow = hand.wobble(BOW);
	let (nx, ny) = (-uy * bow, ux * bow);
	[
		PathCommand::MoveTo(start),
		PathCommand::CubicTo {
			control1: Point::new(start.x + dx / 3.0 + nx, start.y + dy / 3.0 + ny),
			control2: Point::new(
				start.x + dx * 2.0 / 3.0 + nx * 0.6,
				start.y + dy * 2.0 / 3.0 + ny * 0.6,
			),
			to: end,
		},
	]
}

/// The box for `seed`: every edge drawn twice.
fn path(seed: u32) -> PathData {
	thread_local! {
		static PATHS: RefCell<HashMap<u32, PathData>> = RefCell::new(HashMap::new());
	}
	PATHS.with_borrow_mut(|paths| {
		paths
			.entry(seed)
			.or_insert_with(|| {
				let mut hand = Hand(seed);
				let corners = [
					Point::new(0.0, 0.0),
					Point::new(100.0, 0.0),
					Point::new(100.0, 100.0),
					Point::new(0.0, 100.0),
				];
				let mut commands = Vec::with_capacity(16);
				for _pass in 0..2 {
					for edge in 0..4 {
						commands.extend(stroke(&mut hand, corners[edge], corners[(edge + 1) % 4]));
					}
				}
				PathData::from_commands(commands).expect("a hand-drawn box is finite")
			})
			.clone()
	})
}

/// A sketch of a box, to lay first inside a `relative` container so it is
/// drawn beneath the content and takes up none of its room.
pub fn sketch(seed: u32, colour: Rgb, alpha: f64) -> Element {
	let ink = Color::rgba(
		colour.0,
		colour.1,
		colour.2,
		(alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
	);
	Vector(ViewBox::new(0.0, 0.0, 100.0, 100.0).expect("a positive size"))
		.scaling(VectorScaling::Stretch)
		.path(
			VectorPath::new(path(seed)).stroke(
				Stroke::new(ink, THICKNESS)
					.expect("a positive width")
					.round(),
			),
		)
		.class("absolute inset-0")
		.aria_hidden(true)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_same_seed_draws_the_same_line() {
		assert_eq!(path(7), path(7));
		assert_ne!(path(7), path(8));
	}

	#[test]
	fn the_hand_wanders_a_little_and_no_more() {
		let mut hand = Hand(42);
		for _ in 0..1000 {
			let value = hand.wobble(WANDER);
			assert!(value.abs() <= WANDER);
		}
	}
}
