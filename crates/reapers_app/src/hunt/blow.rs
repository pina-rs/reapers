//! The marks of a blow that strikes the husk, as the art bible sets them out
//! (§11.1, §11.2, §11.5, §11.6, §12.2): the ink splat it leaves in the
//! reaper's spot and the scrawl across it, the sight's arms stepping out,
//! and the slip its figure is printed on.
//!
//! Like the shot's marks ([`super::marks`]), each is a function of the frame
//! it is asked about and of where it falls on the screen, and nothing reads a
//! clock. The bible counts a hit's frames from the press, the splat landing
//! one frame after the burst. The rules land a shot and its blow on one tick,
//! and the blow's sound plays on that tick, so here the hit's table runs a
//! frame earlier ([`beat`]): the splat lands with the burst, on the sound.

use std::f64::consts::PI;
use std::f64::consts::TAU;

use leptoskit::native::vector::PathCommand;
use leptoskit::prelude::*;
use reapers_rules::SeededRandom;

use super::marks::At;
use super::marks::CLEAR;
use super::marks::outline;
use super::marks::point;
use super::overlay::Sketch;
use super::overlay::colour;
use crate::ink;
use crate::theme::Rgb;
use crate::ui::display;
use crate::ui::label;

/// The frames a second the bible's tables count in.
const FRAMES_PER_SECOND: f64 = 60.0;

/// The frame of the bible's hit table (§12.2) that a blow's `frame`, as
/// [`super::marks::frames_since`] counts it, is: one on, since the splat
/// lands on the shot's own frame, where its sound is.
pub const fn beat(frame: i64) -> i64 {
	frame + 1
}

/// A splat's radius on a seal and on the body, and its satellites' (§11.1).
pub const SEAL_SPLAT: f64 = 21.0;
pub const BODY_SPLAT: f64 = 12.0;
const SATELLITE: (f64, f64) = (2.4, 6.0);
/// The splat's key edge, and how far out of register it prints: 2 pt left
/// and 1.5 pt down.
const SPLAT_EDGE: f64 = 1.6;
pub const MISREGISTER: (f64, f64) = (-2.0, 1.5);
/// The last beat the edge is held out of register; from the next it springs
/// home.
const HELD_UNTIL: i64 = 5;
/// The splat's wipe, in three bites on twos: from each beat, how much of
/// it is left. 400 ms is beat 24.
const WIPE: [(i64, f64); 3] = [(24, 0.75), (29, 0.45), (34, 0.0)];
/// The scrawl: three key strokes, from beat 2 (§11.6).
const SCRAWL_FROM: i64 = 2;
const SCRAWL_STROKES: usize = 3;
const SCRAWL_WIDTH: f64 = 2.4;

/// How much of a blow's splat is printed on `beat`, from 1 (all of it) down
/// to 0 (none): none before it lands, then all, then wiped off in bites.
/// For a player who asks for less motion (`lively` false) it is all there
/// until it goes, in one step, when the bites would have finished (§14).
pub fn coverage(beat: i64, lively: bool) -> f64 {
	if beat < 1 {
		return 0.0;
	}
	if !lively {
		return if beat < WIPE[2].0 { 1.0 } else { 0.0 };
	}
	WIPE.iter()
		.rev()
		.find(|(from, _)| beat >= *from)
		.map_or(1.0, |(_, left)| *left)
}

/// How far a splat's key edge prints from where it belongs on `beat`: out
/// of register and held until beat 5, then sprung home on the snap spring.
/// In register throughout for a player who asks for less motion.
pub fn misregistration(beat: i64, lively: bool) -> (f64, f64) {
	let left = if !lively {
		0.0
	} else if beat <= HELD_UNTIL {
		1.0
	} else {
		snap((beat - HELD_UNTIL) as f64 / FRAMES_PER_SECOND)
	};
	(MISREGISTER.0 * left, MISREGISTER.1 * left)
}

/// Whether a blow's scrawl is printed on `beat`: from beat 2, and gone with
/// the splat.
pub fn scrawl_shows(beat: i64, lively: bool) -> bool {
	beat >= SCRAWL_FROM && coverage(beat, lively) > 0.0
}

/// The tag: when it appears, popped, and when it is at its size; how far it
/// rises; its three stepped shrinks on twos; and when it is gone (§11.5).
/// 600 ms is beat 36 and 850 ms beat 51.
const TAG_FROM: i64 = 2;
const TAG_SETTLED: i64 = 4;
const TAG_POP: f64 = 1.15;
pub const TAG_RISE: f64 = 18.0;
const SHRINKS: [(i64, f64); 3] = [(36, 0.8), (41, 0.55), (46, 0.3)];
pub const TAG_GONE: i64 = 51;
/// A tag for a blow less than 600 ms after the last stacks this far above
/// it, and no more than three are printed at once.
pub const STACK: f64 = 22.0;
pub const STACK_BEATS: i64 = 36;
pub const MOST_TAGS: usize = 3;

/// How a blow's tag is printed on `beat`: its scale, and how far it has
/// risen, in points; `None` while it is not. For a player who asks for less
/// motion it is printed in place at its size, and goes in one step (§14).
pub fn tag(beat: i64, lively: bool) -> Option<(f64, f64)> {
	if !(TAG_FROM..TAG_GONE).contains(&beat) {
		return None;
	}
	if !lively {
		return Some((1.0, 0.0));
	}
	if beat < TAG_SETTLED {
		return Some((TAG_POP, 0.0));
	}
	let scale = SHRINKS
		.iter()
		.rev()
		.find(|(from, _)| beat >= *from)
		.map_or(1.0, |(_, scale)| *scale);
	let rise = TAG_RISE * settle((beat - TAG_SETTLED) as f64 / FRAMES_PER_SECOND);
	Some((scale, rise))
}

/// How far the sight's arms step out on `beat`: 3 pt on beats 1 and 2
/// (§11.2).
pub const fn sight_out(beat: i64) -> f64 {
	if matches!(beat, 1 | 2) { 3.0 } else { 0.0 }
}

/// The snap spring (§13: k 1537, c 78.4, critically damped), let go at
/// rest from 1: how much is left `seconds` later.
fn snap(seconds: f64) -> f64 {
	let rate = 1537.0_f64.sqrt();
	(1.0 + rate * seconds) * (-rate * seconds).exp()
}

/// The settle spring (§13: k 587, c 36.4), stepped from 0 to 1: where it is
/// `seconds` later. It overshoots a little.
fn settle(seconds: f64) -> f64 {
	let decay: f64 = 36.4 / 2.0;
	let ring = (587.0 - decay * decay).sqrt();
	1.0 - (-decay * seconds).exp()
		* ((ring * seconds).cos() + decay / ring * (ring * seconds).sin())
}

/// The size of a blow's figure on its tag, by its `cadence`, the hits in a
/// row counting it (§7.3): 40 pt with no bonus, 48 from +20%, 56 at +50%.
/// Each hit before it in the row adds a tenth (`HuntRules::CADENCE_STEP`).
pub const fn figure_size(cadence: i64) -> u32 {
	let before = cadence - 1;
	if before >= 5 {
		56
	} else if before >= 2 {
		48
	} else {
		40
	}
}

/// A tag's slip for a figure `characters` long at `size`: 86 × 50 pt, or as
/// much larger as a bigger figure needs.
pub fn slip_size(characters: usize, size: u32) -> (f64, f64) {
	let size = f64::from(size);
	(
		(characters as f64 * 0.55 * size + 18.0).max(86.0),
		(size * 1.2).max(50.0),
	)
}

/// The margins a tag keeps inside the view: from its sides, below the top's
/// caption and above the thumbs' controls.
const TAG_SIDE: f64 = 12.0;
const TAG_TOP: f64 = 96.0;
const TAG_BOTTOM: f64 = 190.0;

/// Where the middle of a blow's tag first stands, for a blow struck at
/// `hit` on a view `view` points across and high, with a slip `slip` big:
/// 60 to 130 pt up and to the right of it, by `seed`, then kept clear
/// ([`keep_clear`]).
pub fn tag_place(hit: At, seed: i64, view: At, slip: At) -> At {
	let mut random = SeededRandom::new(seed ^ 0x7A65);
	let wanted = (
		hit.0 + random.next_range(60.0, 80.0),
		hit.1 - random.next_range(100.0, 126.0),
	);
	keep_clear(wanted, view, slip)
}

/// The turn of a blow's tag, in degrees: −5° ± 3°, by `seed`.
pub fn tag_turn(seed: i64) -> f64 {
	-5.0 + SeededRandom::new(seed ^ 0x7A66).next_range(-3.0, 3.0)
}

/// `wanted`, the middle of a slip `slip` big, kept inside the view's margins
/// and out of the sight's clear circle: above it, or below it where there is
/// no room above.
pub fn keep_clear(wanted: At, view: At, slip: At) -> At {
	let half = (slip.0 / 2.0, slip.1 / 2.0);
	let within = |value: f64, low: f64, high: f64| value.max(low).min(high);
	let mut at = (
		within(wanted.0, TAG_SIDE + half.0, view.0 - TAG_SIDE - half.0),
		within(wanted.1, TAG_TOP + half.1, view.1 - TAG_BOTTOM - half.1),
	);
	let centre = (view.0 / 2.0, view.1 / 2.0);
	let nearest = (
		within(centre.0, at.0 - half.0, at.0 + half.0),
		within(centre.1, at.1 - half.1, at.1 + half.1),
	);
	if (nearest.0 - centre.0).hypot(nearest.1 - centre.1) < CLEAR {
		let above = centre.1 - CLEAR - half.1;
		at.1 = if above >= TAG_TOP + half.1 {
			above
		} else {
			centre.1 + CLEAR + half.1
		};
	}
	at
}

/// A blow's splat on one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Splat {
	/// Where it struck, on the screen.
	pub at: At,
	/// Its blob's radius: [`SEAL_SPLAT`] or [`BODY_SPLAT`].
	pub radius: f64,
	/// The way the shot travelled across the screen, as a unit vector: its
	/// drops are thrown this way, and it wipes off from this side.
	pub toward: At,
	/// What shapes it, the same every frame.
	pub seed: i64,
	/// The bible's frame it is on.
	pub beat: i64,
	/// Whether it moves: not for a player who asks for less motion.
	pub lively: bool,
}

/// How many corners a splat's blob has.
const BLOB_CORNERS: usize = 28;

/// The part of `corners` on the near side of the line across `toward` at
/// `cut` from `from`: what is past it has been wiped off.
fn wiped(corners: &[At], from: At, toward: At, cut: f64) -> Vec<At> {
	let along = |at: At| (at.0 - from.0) * toward.0 + (at.1 - from.1) * toward.1;
	let mut kept = Vec::with_capacity(corners.len() + 2);
	for (index, &now) in corners.iter().enumerate() {
		let next = corners[(index + 1) % corners.len()];
		let (here, there) = (along(now) - cut, along(next) - cut);
		if here <= 0.0 {
			kept.push(now);
		}
		if (here <= 0.0) != (there <= 0.0) {
			let share = here / (here - there);
			kept.push((
				now.0 + (next.0 - now.0) * share,
				now.1 + (next.1 - now.1) * share,
			));
		}
	}
	kept
}

/// The area inside `corners`.
fn area(corners: &[At]) -> f64 {
	let twice: f64 = corners
		.iter()
		.zip(corners.iter().cycle().skip(1))
		.map(|(a, b)| a.0 * b.1 - b.0 * a.1)
		.sum();
	twice.abs() / 2.0
}

/// Where the wipe's line stands, along `toward` from `from`, for `left` of
/// `blob` to be left.
fn wipe_cut(blob: &[At], from: At, toward: At, left: f64) -> f64 {
	let along = |at: &At| (at.0 - from.0) * toward.0 + (at.1 - from.1) * toward.1;
	let (mut low, mut high) = blob
		.iter()
		.map(along)
		.fold((f64::MAX, f64::MIN), |(low, high), value| {
			(low.min(value), high.max(value))
		});
	if left >= 1.0 {
		return high;
	}
	let whole = area(blob);
	for _ in 0..32 {
		let middle = f64::midpoint(low, high);
		if area(&wiped(blob, from, toward, middle)) < left * whole {
			low = middle;
		} else {
			high = middle;
		}
	}
	f64::midpoint(low, high)
}

/// The three plates every splat on a frame is printed in, a path each: the
/// spot, the key edge and the scrawl.
#[derive(Default)]
pub struct Plates {
	spot: Vec<PathCommand>,
	edge: Vec<PathCommand>,
	scrawl: Vec<PathCommand>,
}

impl Plates {
	/// Prints the plates, the spot in `spot`.
	pub fn print(self, sketch: &mut Sketch, spot: Rgb) {
		let key = colour(ink::KEY, 1.0);
		sketch.push_commands(self.spot, Some(colour(spot, 1.0)), None);
		sketch.push_commands(self.edge, None, Some((key, SPLAT_EDGE)));
		sketch.push_commands(self.scrawl, None, Some((key, SCRAWL_WIDTH)));
	}
}

/// A blow's splat, added to `plates`: a blob with drops thrown from it the
/// way the shot went, its key edge out of register until it springs home,
/// its scrawl across it, wiped off from the far side in bites.
#[allow(clippy::too_many_lines, reason = "one splat, plate by plate")]
pub fn splat(plates: &mut Plates, splat: &Splat) {
	let left = coverage(splat.beat, splat.lively);
	if left <= 0.0 {
		return;
	}
	let Splat {
		at,
		radius,
		toward,
		seed,
		beat,
		lively,
	} = *splat;
	let mut random = SeededRandom::new(seed);
	// The blob: round, with a few lumps, the same each frame.
	let lumps = [
		(
			3.0,
			random.next_range(0.05, 0.1),
			random.next_range(0.0, TAU),
		),
		(
			5.0,
			random.next_range(0.03, 0.07),
			random.next_range(0.0, TAU),
		),
	];
	let blob: Vec<At> = (0..BLOB_CORNERS)
		.map(|corner| {
			let angle = corner as f64 / BLOB_CORNERS as f64 * TAU;
			let swell: f64 = lumps
				.iter()
				.map(|(waves, depth, phase)| depth * (waves * angle + phase).sin())
				.sum();
			let reach = radius * (1.0 + swell + random.next_range(-0.03, 0.03));
			(at.0 + reach * angle.cos(), at.1 + reach * angle.sin())
		})
		.collect();
	// Its drops, thrown away from the shot.
	let heading = toward.1.atan2(toward.0);
	let drops: Vec<(At, f64)> = (0..3 + random.next_int(4))
		.map(|_| {
			let angle = heading + random.next_range(-0.9, 0.9);
			let out = radius * random.next_range(1.3, 2.1);
			let size = random.next_range(SATELLITE.0, SATELLITE.1);
			((at.0 + out * angle.cos(), at.1 + out * angle.sin()), size)
		})
		.collect();
	// The scrawl: three quick strokes across it, roughly square to the shot.
	let strokes: Vec<[At; 3]> = (0..SCRAWL_STROKES)
		.map(|_| {
			let angle = heading + PI / 2.0 + random.next_range(-0.5, 0.5);
			let (dx, dy) = (angle.cos(), angle.sin());
			let reach = radius * random.next_range(0.75, 1.0);
			let shift = radius * random.next_range(-0.45, 0.45);
			let middle = (at.0 + toward.0 * shift, at.1 + toward.1 * shift);
			let bend = radius * random.next_range(-0.15, 0.15);
			[
				(middle.0 - dx * reach, middle.1 - dy * reach),
				(middle.0 + toward.0 * bend, middle.1 + toward.1 * bend),
				(middle.0 + dx * reach, middle.1 + dy * reach),
			]
		})
		.collect();
	let cut = wipe_cut(&blob, at, toward, left);
	let along = |point: At| (point.0 - at.0) * toward.0 + (point.1 - at.1) * toward.1;
	let kept = wiped(&blob, at, toward, cut);
	let drops: Vec<(At, f64)> = drops
		.into_iter()
		.filter(|(drop, _)| left >= 1.0 || along(*drop) <= cut)
		.collect();
	let shapes = |offset: At| {
		let mut commands = Vec::with_capacity(kept.len() + 2 + drops.len() * 6);
		let moved: Vec<At> = kept
			.iter()
			.map(|corner| (corner.0 + offset.0, corner.1 + offset.1))
			.collect();
		outline(&mut commands, &moved);
		for (drop, size) in &drops {
			Sketch::append_circle(&mut commands, drop.0 + offset.0, drop.1 + offset.1, *size);
		}
		commands
	};
	plates.spot.extend(shapes((0.0, 0.0)));
	plates.edge.extend(shapes(misregistration(beat, lively)));
	if !scrawl_shows(beat, lively) {
		return;
	}
	let commands = &mut plates.scrawl;
	for line in &strokes {
		// Each stroke is cut where the wipe has reached.
		let mut drawn: Vec<At> = Vec::new();
		for pair in line.windows(2) {
			let (from, to) = (pair[0], pair[1]);
			let (here, there) = (along(from) - cut, along(to) - cut);
			if here > 0.0 && there > 0.0 {
				stroke(commands, &drawn);
				drawn.clear();
				continue;
			}
			let clip = |inside: At, outside: At, a: f64, b: f64| {
				let share = a / (a - b);
				(
					inside.0 + (outside.0 - inside.0) * share,
					inside.1 + (outside.1 - inside.1) * share,
				)
			};
			let start = if here > 0.0 {
				clip(from, to, here, there)
			} else {
				from
			};
			let end = if there > 0.0 {
				clip(from, to, here, there)
			} else {
				to
			};
			if drawn.last() != Some(&start) {
				stroke(commands, &drawn);
				drawn.clear();
				drawn.push(start);
			}
			drawn.push(end);
		}
		stroke(commands, &drawn);
	}
}

/// An open line through `points`, added to `commands`.
fn stroke(commands: &mut Vec<PathCommand>, points: &[At]) {
	let Some((first, rest)) = points.split_first() else {
		return;
	};
	commands.push(PathCommand::MoveTo(point(*first)));
	commands.extend(rest.iter().map(|at| PathCommand::LineTo(point(*at))));
}

/// A blow's tag as it is printed on one frame.
#[derive(Clone, Debug, PartialEq)]
pub struct TagLook {
	/// The tick of its blow, which tells tags apart and shapes its slip.
	pub tick: i64,
	/// Where its middle is, on the screen, and how big its slip is.
	pub centre: At,
	pub slip: At,
	/// Its turn, in degrees, and its scale.
	pub turn: f64,
	pub scale: f64,
	/// The figure, and its size.
	pub figure: String,
	pub size: u32,
	/// The chip under it on a seal: "Seal ×2".
	pub chip: Option<String>,
	pub spot: Rgb,
}

/// The slip's rough key edge, and its spot underline's two passes.
const SLIP_EDGE: f64 = 1.8;
const UNDERLINE: f64 = 3.0;

/// A blow's tag: a paper slip with a rough key edge, the figure in key
/// over a spot underline drawn in two passes, and on a seal a chip under it.
/// It is placed by its middle, and turned and scaled about it.
pub fn tag_view(look: TagLook) -> Element {
	let (width, height) = look.slip;
	let mut random = SeededRandom::new(look.tick ^ 0x511F);
	let mut jitter = |reach: f64| random.next_range(-reach, reach);
	let inset = SLIP_EDGE * 2.0;
	let (right, bottom) = (width - inset, height - inset);
	let corners: Vec<At> = [
		(inset, inset),
		(width / 2.0, inset),
		(right, inset),
		(right, height / 2.0),
		(right, bottom),
		(width / 2.0, bottom),
		(inset, bottom),
		(inset, height / 2.0),
	]
	.into_iter()
	.map(|(x, y)| (x + jitter(1.4), y + jitter(1.4)))
	.collect();
	let mut sketch = Sketch::default();
	let mut slip = Vec::with_capacity(corners.len() + 1);
	outline(&mut slip, &corners);
	sketch.push_commands(
		slip,
		Some(colour(ink::PAPER, 1.0)),
		Some((colour(ink::KEY, 1.0), SLIP_EDGE)),
	);
	let size = f64::from(look.size);
	let text = (look.figure.chars().count() as f64 * 0.55 * size).min(width - 12.0);
	let line = height / 2.0 + size * 0.36;
	let mut passes = Vec::new();
	for pass in 0..2 {
		let lift = f64::from(pass) * 1.6;
		stroke(
			&mut passes,
			&[
				(
					(width - text) / 2.0 + jitter(2.0),
					line + lift + jitter(0.8),
				),
				(
					f64::midpoint(width, text) + jitter(2.0),
					line + lift + jitter(0.8),
				),
			],
		);
	}
	sketch.push_commands(passes, None, Some((colour(look.spot, 1.0), UNDERLINE)));
	let drawing = match ViewBox::new(0.0, 0.0, width as f32, height as f32) {
		Ok(view_box) => {
			Vector(view_box)
				.scaling(VectorScaling::Stretch)
				.paths(sketch.into_paths())
				.class("absolute inset-0 w-full h-full")
		}
		Err(_) => View(),
	};
	let mut slip = View()
		.class(format!(
			"absolute left-[{:.1}px] top-[{:.1}px] w-[{width:.1}px] h-[{height:.1}px] items-center justify-center",
			look.centre.0 - width / 2.0,
			look.centre.1 - height / 2.0,
		))
		.transform(
			leptoskit::native::Transform::rotate(look.turn as f32)
				.scaled(look.scale as f32, look.scale as f32),
		)
		.child(drawing)
		.child(display(look.figure, look.size, ink::KEY));
	if let Some(chip) = look.chip {
		slip = slip.child(
			View()
				.class(format!(
					"absolute left-0 right-0 top-[{:.1}px] items-center",
					height - inset
				))
				.child(
					View()
						.class(format!(
							"px-[6px] py-[2px] border border-[{}] bg-[{}]",
							ink::KEY.hex(),
							ink::PAPER.hex()
						))
						.child(label(&chip, 12, ink::KEY)),
				),
		);
	}
	slip
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_splat_lands_holds_springs_home_and_wipes_off_in_three_bites() {
		assert_eq!(coverage(0, true), 0.0);
		for beat in 1..24 {
			assert_eq!(coverage(beat, true), 1.0, "f{beat}");
		}
		assert_eq!(coverage(24, true), 0.75, "400 ms");
		assert_eq!(coverage(28, true), 0.75, "held on twos");
		assert_eq!(coverage(29, true), 0.45);
		assert_eq!(coverage(33, true), 0.45);
		assert_eq!(coverage(34, true), 0.0, "gone");
		// Out of register, held, from f1 to f5.
		for beat in 1..=5 {
			assert_eq!(misregistration(beat, true), MISREGISTER, "f{beat}");
		}
		// From f6 it springs home on the snap spring: moving at once, never
		// past where it belongs, and home in about 120 ms.
		let left: Vec<f64> = (5..=14)
			.map(|beat| misregistration(beat, true).0 / MISREGISTER.0)
			.collect();
		assert!(
			left[1] < 1.0 && left[1] > 0.5,
			"f6 is the first step: {left:?}"
		);
		assert!(
			left.windows(2)
				.all(|pair| pair[1] < pair[0] && pair[1] >= 0.0)
		);
		assert!(
			misregistration(5 + 8, true).0.abs() < 0.05 * MISREGISTER.0.abs(),
			"home in about 120 ms"
		);
		// The scrawl from f2, gone with the splat.
		let scrawl: Vec<i64> = (0..40).filter(|beat| scrawl_shows(*beat, true)).collect();
		assert_eq!(scrawl, (2..34).collect::<Vec<_>>());
	}

	#[test]
	fn the_tag_pops_settles_rises_and_shrinks_out_in_steps() {
		assert_eq!(tag(1, true), None);
		assert_eq!(tag(2, true), Some((1.15, 0.0)), "f2 at 1.15×");
		assert_eq!(tag(3, true), Some((1.15, 0.0)));
		assert_eq!(tag(4, true), Some((1.0, 0.0)), "f4 at 1.0×");
		// It rises 18 pt on the settle spring, smoothly, and is there well
		// before 600 ms.
		let rises: Vec<f64> = (4..36).map(|beat| tag(beat, true).unwrap().1).collect();
		assert!(rises.windows(2).take(10).all(|pair| pair[1] > pair[0]));
		assert!((rises.last().unwrap() - TAG_RISE).abs() < 0.2, "{rises:?}");
		assert!(rises.iter().all(|rise| *rise < TAG_RISE * 1.05));
		// Three stepped shrinks on twos from 600 ms, and gone at 850 ms.
		let scale = |beat| tag(beat, true).unwrap().0;
		assert_eq!(scale(35), 1.0);
		assert_eq!((scale(36), scale(40)), (0.8, 0.8));
		assert_eq!((scale(41), scale(45)), (0.55, 0.55));
		assert_eq!((scale(46), scale(50)), (0.3, 0.3));
		assert_eq!(tag(51, true), None);
	}

	#[test]
	fn with_less_motion_a_blow_prints_in_place_and_leaves_in_one_step() {
		assert_eq!(beat(0), 1, "the splat lands on the shot's own frame");
		let left: Vec<f64> = (0..40).map(|beat| coverage(beat, false)).collect();
		assert_eq!(left[0], 0.0);
		assert_eq!(left[1..34], [1.0; 33], "held: {left:?}");
		assert!(
			left[34..].iter().all(|left| *left == 0.0),
			"then gone at once"
		);
		assert!((1..40).all(|beat| misregistration(beat, false) == (0.0, 0.0)));
		let scrawl: Vec<i64> = (0..40).filter(|beat| scrawl_shows(*beat, false)).collect();
		assert_eq!(scrawl, (2..34).collect::<Vec<_>>());
		assert_eq!(tag(1, false), None);
		assert!((2..51).all(|beat| tag(beat, false) == Some((1.0, 0.0))));
		assert_eq!(tag(51, false), None);
	}

	#[test]
	fn the_sight_steps_out_for_two_frames() {
		let out: Vec<i64> = (0..8).filter(|beat| sight_out(*beat) > 0.0).collect();
		assert_eq!(out, [1, 2]);
		assert_eq!(sight_out(1), 3.0);
	}

	#[test]
	fn a_figure_grows_with_cadence() {
		// The first hit of a row has no bonus; the third has +20%, the sixth
		// +50%.
		assert_eq!(figure_size(1), 40);
		assert_eq!(figure_size(2), 40, "+10%");
		assert_eq!(figure_size(3), 48, "+20%");
		assert_eq!(figure_size(5), 48);
		assert_eq!(figure_size(6), 56, "+50%");
		assert_eq!(figure_size(11), 56, "a chain past the cap");
		assert_eq!(slip_size(3, 40), (86.0, 50.0));
		assert!(
			slip_size(3, 56).0 > 86.0,
			"a bigger figure gets a bigger slip"
		);
	}

	#[test]
	fn a_tag_stands_up_and_right_of_its_blow_clear_of_the_sight() {
		let view = (390.0, 844.0);
		let slip = slip_size(3, 40);
		for seed in 0..40 {
			let hit = (190.0 + seed as f64, 470.0 - seed as f64 * 2.0);
			let at = tag_place(hit, seed, view, slip);
			let half = (slip.0 / 2.0, slip.1 / 2.0);
			assert!(at.0 - half.0 >= TAG_SIDE && at.0 + half.0 <= view.0 - TAG_SIDE);
			assert!(at.1 - half.1 >= TAG_TOP && at.1 + half.1 <= view.1 - TAG_BOTTOM);
			let nearest = (
				(view.0 / 2.0).clamp(at.0 - half.0, at.0 + half.0),
				(view.1 / 2.0).clamp(at.1 - half.1, at.1 + half.1),
			);
			assert!(
				(nearest.0 - view.0 / 2.0).hypot(nearest.1 - view.1 / 2.0) >= CLEAR - 1.0e-9,
				"seed {seed}: {at:?}"
			);
		}
		// Far from the sight, it is 60–130 pt up and right of the blow.
		let hit = (120.0, 700.0);
		let at = tag_place(hit, 7, view, slip);
		let (right, up) = (at.0 - hit.0, hit.1 - at.1);
		assert!(right > 0.0 && up > 0.0);
		assert!(
			(60.0..=130.0).contains(&right) && (60.0..=130.0).contains(&up),
			"{right}, {up}"
		);
		for seed in 0..40 {
			assert!((-8.0..=-2.0).contains(&tag_turn(seed)));
		}
	}

	#[test]
	fn a_splat_is_three_paths_and_its_wipe_leaves_what_it_says() {
		let splat_at = |beat| {
			Splat {
				at: (200.0, 300.0),
				radius: SEAL_SPLAT,
				toward: (0.6, -0.8),
				seed: 42,
				beat,
				lively: true,
			}
		};
		let printed = |beats: &[i64]| {
			let mut plates = Plates::default();
			for beat in beats {
				splat(&mut plates, &splat_at(*beat));
			}
			let mut sketch = Sketch::default();
			plates.print(&mut sketch, ink::LANTERN);
			sketch.into_paths().len()
		};
		assert_eq!(printed(&[1]), 2, "no scrawl on f1");
		assert_eq!(printed(&[2]), 3, "spot, key edge, scrawl");
		assert_eq!(
			printed(&[2, 10, 20]),
			3,
			"three splats print in the same three"
		);
		assert_eq!(printed(&[34]), 0, "gone");
		// The wipe's line leaves the share of the blob it says.
		let blob: Vec<At> = (0..BLOB_CORNERS)
			.map(|corner| {
				let angle = corner as f64 / BLOB_CORNERS as f64 * TAU;
				(20.0 * angle.cos(), 20.0 * angle.sin())
			})
			.collect();
		for left in [0.75, 0.45] {
			let cut = wipe_cut(&blob, (0.0, 0.0), (1.0, 0.0), left);
			let kept = area(&wiped(&blob, (0.0, 0.0), (1.0, 0.0), cut)) / area(&blob);
			assert!((kept - left).abs() < 1.0e-3, "{kept} for {left}");
		}
	}
}
