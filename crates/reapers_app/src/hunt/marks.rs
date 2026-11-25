//! The marks printed flat over a fight, as the art bible sets them out
//! (§11, §12.2): the burst at the pistol's muzzle and the smear of its kick,
//! which stand in for the flash the pistol once drew in light, printed with
//! the splats of its blows ([`super::blow`]) and the sight over them.
//!
//! Each is a function of the frame it is asked about, counted at sixty a
//! second from the first frame shown after its event ([`frames_since`]),
//! and of where it falls on the screen, in the view's own points. Nothing
//! reads a clock and nothing is remembered, so the same frame prints the
//! same marks: a bench can step through them and a test can hold them to
//! the bible's tables. The viewmodel seam works out where they fall and
//! prints them ([`crate::seams::viewmodel_marks`]).

use std::f64::consts::PI;
use std::f64::consts::TAU;

use leptoskit::native::vector::PathCommand;
use leptoskit::native::vector::Point;
use leptoskit::prelude::*;
use reapers_rules::SeededRandom;

use super::blow::Plates;
use super::blow::Splat;
use super::blow::splat;
use super::overlay::Sight;
use super::overlay::Sketch;
use super::overlay::colour;
use super::overlay::crosshair;
use crate::ink;
use crate::theme::Rgb;

/// A place on the screen, in points from its top left.
pub type At = (f64, f64);

/// How far round the sight nothing printed for a shot may come, in points
/// (§6.5, §11.4).
pub const CLEAR: f64 = 60.0;

/// The sixty-a-second frame `shown`, a tick of the hunt's clock as the
/// display shows it, is in, counted from the first frame shown after an
/// event on the rules' `tick`. An event is heard once its tick is past, so
/// its first frame is the one at `tick + 1`. Negative before then.
pub fn frames_since(shown: f64, tick: i64) -> i64 {
	(shown - tick as f64 - 1.0 + 1.0e-6).floor() as i64
}

/// The muzzle burst's star: how many tips, and the radii of its tips and of
/// the notches between them, with its key edge (§11.6).
const BURST_TIPS: usize = 10;
const BURST_OUTER: f64 = 17.0;
const BURST_INNER: f64 = 6.0;
const BURST_EDGE: f64 = 1.8;
/// How far past the pistol's muzzle, along its barrel, the burst's middle
/// is printed: clear of the barrel's flared mouth, so the star is seen
/// whole rather than half over the iron.
pub const BURST_AHEAD: f64 = 18.0;
/// The smear's edge in paper (§11.4), and how wide it is from the muzzle
/// back along the three frames before, each side of its middle: about the
/// barrel's width at the muzzle, about 3 pt across at its tail.
const SMEAR_EDGE: f64 = 1.25;
const SMEAR_WIDTHS: [f64; 4] = [12.0, 8.5, 5.0, 1.5];

/// Whether a shot's burst is printed on its `frame`: the first two (§11.6),
/// or only the first for a player who asks for less motion (§14).
pub const fn burst_shows(frame: i64, lively: bool) -> bool {
	frame == 0 || (lively && frame == 1)
}

/// Whether the smear of a shot's kick is printed on its `frame`: the first
/// (§11.4), and never for a player who asks for less motion (§14).
pub const fn smear_shows(frame: i64, lively: bool) -> bool {
	lively && frame == 0
}

/// What is printed over a fight on one frame, in the view's points.
#[derive(Clone, Debug, PartialEq)]
pub struct Print {
	/// The view's width and height.
	pub view: At,
	/// The reaper's spot: the second ink.
	pub spot: Rgb,
	/// The splats of the blows that struck, oldest first.
	pub splats: Vec<Splat>,
	/// The burst at the muzzle, and the seed that turns it.
	pub burst: Option<(At, i64)>,
	/// The smear of the pistol's kick: the muzzle now, then where it was the
	/// three frames before.
	pub smear: Option<[At; 4]>,
	/// The sight at the middle of the view, while there is one: what a press
	/// would do, and how far its arms stand out.
	pub sight: Option<(Sight, f64)>,
}

impl Print {
	/// Nothing printed, on a view `view` big.
	pub const fn blank(view: At, spot: Rgb) -> Self {
		Self {
			view,
			spot,
			splats: Vec::new(),
			burst: None,
			smear: None,
			sight: None,
		}
	}

	/// Whether nothing is printed.
	pub const fn is_blank(&self) -> bool {
		self.splats.is_empty()
			&& self.burst.is_none()
			&& self.smear.is_none()
			&& self.sight.is_none()
	}
}

/// The marks of `print`, filling their parent: the splats on the husk, the
/// smear and the burst at the muzzle, and the sight over them all.
pub fn print_view(print: &Print) -> Element {
	let mut sketch = Sketch::default();
	let centre = (print.view.0 / 2.0, print.view.1 / 2.0);
	let mut plates = Plates::default();
	for each in &print.splats {
		splat(&mut plates, each);
	}
	plates.print(&mut sketch, print.spot);
	if let Some(trail) = print.smear {
		smear(&mut sketch, trail, centre);
	}
	if let Some((at, seed)) = print.burst {
		burst(&mut sketch, at, seed, print.spot);
	}
	if let Some((sight, out)) = print.sight {
		crosshair(&mut sketch, centre, sight, out);
	}
	sketch.into_view(print.view.0, print.view.1)
}

/// `at`, as a point of a path.
pub(super) fn point(at: At) -> Point {
	Point::new(at.0 as f32, at.1 as f32)
}

/// A closed shape through `corners`, added to `commands`.
pub(super) fn outline(commands: &mut Vec<PathCommand>, corners: &[At]) {
	let Some((first, rest)) = corners.split_first() else {
		return;
	};
	commands.push(PathCommand::MoveTo(point(*first)));
	commands.extend(rest.iter().map(|at| PathCommand::LineTo(point(*at))));
	commands.push(PathCommand::Close);
}

/// `at`, moved straight out from `centre` to the edge of the clear circle
/// if it was inside.
pub(super) fn clear_of(at: At, centre: At) -> At {
	let (dx, dy) = (at.0 - centre.0, at.1 - centre.1);
	let distance = dx.hypot(dy);
	if distance >= CLEAR {
		return at;
	}
	if distance < 1.0e-9 {
		return (centre.0, centre.1 + CLEAR);
	}
	(
		centre.0 + dx / distance * CLEAR,
		centre.1 + dy / distance * CLEAR,
	)
}

/// The burst of a shot at `at`, the muzzle: a ten-point star in the
/// reaper's `spot`, edged in key, turned by `seed`.
pub fn burst(sketch: &mut Sketch, at: At, seed: i64, spot: Rgb) {
	let turn = SeededRandom::new(seed).next_range(0.0, TAU / BURST_TIPS as f64);
	let corners: Vec<At> = (0..BURST_TIPS * 2)
		.map(|corner| {
			let radius = if corner % 2 == 0 {
				BURST_OUTER
			} else {
				BURST_INNER
			};
			let angle = turn + corner as f64 * PI / BURST_TIPS as f64;
			(at.0 + radius * angle.cos(), at.1 + radius * angle.sin())
		})
		.collect();
	let mut commands = Vec::with_capacity(corners.len() + 1);
	outline(&mut commands, &corners);
	sketch.push_commands(
		commands,
		Some(colour(spot, 1.0)),
		Some((colour(ink::KEY, 1.0), BURST_EDGE)),
	);
}

/// The smear of the pistol's kick: a ribbon from the muzzle, `trail[0]`,
/// back through where it was the three frames before, filled key and edged
/// in paper, kept out of the sight's clear circle round `centre`. A muzzle
/// that has not moved leaves none.
pub fn smear(sketch: &mut Sketch, trail: [At; 4], centre: At) {
	let (dx, dy) = (trail[0].0 - trail[3].0, trail[0].1 - trail[3].1);
	let length = dx.hypot(dy);
	if length < 1.5 {
		return;
	}
	let across = (-dy / length, dx / length);
	let side = |sign: f64| {
		trail
			.iter()
			.zip(SMEAR_WIDTHS)
			.map(move |(at, width)| {
				clear_of(
					(
						at.0 + across.0 * width * sign,
						at.1 + across.1 * width * sign,
					),
					centre,
				)
			})
			.collect::<Vec<At>>()
	};
	let mut corners = side(1.0);
	corners.extend(side(-1.0).into_iter().rev());
	let mut commands = Vec::with_capacity(corners.len() + 1);
	outline(&mut commands, &corners);
	sketch.push_commands(
		commands,
		Some(colour(ink::KEY, 1.0)),
		Some((colour(ink::PAPER, 1.0), SMEAR_EDGE)),
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_first_frame_after_an_event_is_its_frame_zero() {
		assert_eq!(frames_since(100.0, 100), -1, "not heard yet");
		assert_eq!(frames_since(101.0, 100), 0);
		assert_eq!(frames_since(101.5, 100), 0, "a second frame at 120 Hz");
		assert_eq!(frames_since(102.0, 100), 1);
		// A clock that shows its ticks a rounding short still counts them.
		assert_eq!(frames_since(101.999_999_9, 100), 1);
	}

	#[test]
	fn the_burst_and_the_smear_keep_to_the_bibles_frames() {
		let burst: Vec<i64> = (-2..10).filter(|frame| burst_shows(*frame, true)).collect();
		assert_eq!(burst, [0, 1], "f0–f1");
		let smear: Vec<i64> = (-2..10).filter(|frame| smear_shows(*frame, true)).collect();
		assert_eq!(smear, [0], "one frame");
		// With less motion: a burst of one frame, and no smear.
		let burst: Vec<i64> = (-2..10)
			.filter(|frame| burst_shows(*frame, false))
			.collect();
		assert_eq!(burst, [0]);
		assert!((-2..10).all(|frame| !smear_shows(frame, false)));
	}

	#[test]
	fn the_burst_is_one_path_and_the_smear_stays_out_of_the_sights_circle() {
		let mut sketch = Sketch::default();
		burst(&mut sketch, (230.0, 560.0), 9, ink::LANTERN);
		assert_eq!(sketch.into_paths().len(), 1);
		let mut sketch = Sketch::default();
		let centre = (195.0, 422.0);
		smear(
			&mut sketch,
			[
				(200.0, 470.0),
				(205.0, 500.0),
				(210.0, 530.0),
				(215.0, 560.0),
			],
			centre,
		);
		assert_eq!(sketch.into_paths().len(), 1);
		for at in [(200.0, 470.0), (195.0, 430.0), (195.0, 422.0)] {
			let kept = clear_of(at, centre);
			assert!((kept.0 - centre.0).hypot(kept.1 - centre.1) >= CLEAR - 1.0e-9);
		}
		let mut still = Sketch::default();
		smear(&mut still, [(230.0, 560.0); 4], centre);
		assert!(
			still.is_empty(),
			"a muzzle that has not moved leaves no smear"
		);
	}
}
