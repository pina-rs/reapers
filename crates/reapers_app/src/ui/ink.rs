//! Second Ink's hand: every rough stroke the interface prints, drawn by
//! Skribble's own generator (`skribble_rough`, through
//! `leptoskit_skribble`) and kept.
//!
//! A stroke is a [`Recipe`]: a [`Mark`] (a frame, a line, a ring, a hatch),
//! the seed its wobble is drawn from, and how many passes of the pen it
//! takes. The same recipe in the same hand always draws the same line, so a
//! stroke is generated once, the first time anything asks for it, and every
//! later render takes the kept path: ink never crawls, and a screen that is
//! built again costs a lookup. [`drawn`] counts the strokes generated, so a
//! test can hold a second build to drawing none.
//!
//! Strokes are drawn in points in a box of their own and printed as a
//! [`Plate`]: one `Vector` layer in one ink, stretched over its parent, so
//! the spot plate can sit out of register by a translate without redrawing
//! anything (§8).

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;

use leptoskit::native::vector::PathCommand;
use leptoskit::native::vector::Point;
use leptoskit::prelude::*;
use leptoskit_skribble::Palette;
use leptoskit_skribble::Theme;
use leptoskit_skribble::colorway::Rgba;
use leptoskit_skribble::rough;

/// The Second Ink palette as a Skribble theme, for the hand that draws on
/// the vigil page: paper ink on vigil, ash for what is muted, raised for
/// rules. There is one appearance (the bible's §16, "Vigil always").
const PALETTE: Palette = Palette {
	ink: Rgba::hex(0x00EC_E3CF),
	paper: Rgba::hex(0x0034_2F33),
	muted_ink: Rgba::hex(0x00A3_9C93),
	marker: Rgba::hex(0x0045_4042),
	accent: Rgba::hex(0x00EC_E3CF),
	washes: &[Rgba::hex(0x0045_4042)],
};

/// Second Ink's Skribble theme: its palette, its outline width, and how far
/// its hand wanders. Every stroke is kept against the theme it was drawn
/// in, so a change of hand redraws everything once.
pub const SECOND_INK: Theme = Theme {
	light: PALETTE,
	dark: PALETTE,
	stroke_width: crate::theme::stroke::SLIP,
	roughness: 0.8,
};

/// How much a line bows from straight in Second Ink's hand: less than
/// Skribble's default, so a long rule reads as pulled, not wavering.
const BOWING: f64 = 0.7;
/// How far a line's ends wander, in points.
const WANDER: f64 = 0.9;

/// A coordinate kept to a quarter point, so a recipe can be hashed and the
/// same shape asked for twice is the same key.
type Quarter = i32;

fn quarter(value: f32) -> Quarter {
	(value * 4.0).round() as Quarter
}

fn points(value: Quarter) -> f64 {
	f64::from(value) / 4.0
}

/// A shape the hand draws, in points, in its plate's own box.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
	/// A box from the origin, `width` by `height`: four edges, each pulled
	/// `over` past its corners as a printer's rule is.
	Frame {
		width: Quarter,
		height: Quarter,
		over: Quarter,
	},
	/// A straight stroke.
	Line {
		from: (Quarter, Quarter),
		to: (Quarter, Quarter),
	},
	/// A ring round `centre`.
	Ellipse {
		centre: (Quarter, Quarter),
		radii: (Quarter, Quarter),
	},
	/// Part of a ring, from `start` to `stop` in thousandths of a radian,
	/// clockwise from three o'clock.
	Arc {
		centre: (Quarter, Quarter),
		radii: (Quarter, Quarter),
		start: i32,
		stop: i32,
	},
	/// A line through `points`, back to the first when `closed`.
	Path {
		points: Vec<(Quarter, Quarter)>,
		closed: bool,
	},
	/// Hachure inside the polygon through `points`: lines `gap` apart at
	/// `angle` degrees.
	Hatch {
		points: Vec<(Quarter, Quarter)>,
		gap: Quarter,
		angle: i32,
	},
	/// A halftone: dots of radius `dot` on a grid `pitch` apart, turned
	/// `angle` degrees, inside the circle of `radius` round `centre`. It is
	/// a fill, never stroked, and has no wobble.
	Halftone {
		centre: (Quarter, Quarter),
		radius: Quarter,
		pitch: Quarter,
		dot: Quarter,
		angle: i32,
	},
	/// The lit part of a waxing disc round `centre`: the right half of the
	/// circle, closed by an elliptical terminator that is `phase`
	/// thousandths of the way from new (none lit) to full. A fill.
	Waxing {
		centre: (Quarter, Quarter),
		radius: Quarter,
		phase: i32,
	},
}

impl Mark {
	/// A box `width` by `height` with edges pulled `over` past the corners.
	pub fn frame(width: f32, height: f32, over: f32) -> Self {
		Self::Frame {
			width: quarter(width),
			height: quarter(height),
			over: quarter(over),
		}
	}

	/// A stroke from `(x1, y1)` to `(x2, y2)`.
	pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
		Self::Line {
			from: (quarter(x1), quarter(y1)),
			to: (quarter(x2), quarter(y2)),
		}
	}

	/// A ring of radius `r` round `(cx, cy)`.
	pub fn circle(cx: f32, cy: f32, r: f32) -> Self {
		Self::ellipse(cx, cy, r, r)
	}

	/// A ring with radii `rx` and `ry` round `(cx, cy)`.
	pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
		Self::Ellipse {
			centre: (quarter(cx), quarter(cy)),
			radii: (quarter(rx), quarter(ry)),
		}
	}

	/// Part of a ring of radius `r` round `(cx, cy)`, from `start` to
	/// `stop` radians.
	pub fn arc(cx: f32, cy: f32, r: f32, start: f32, stop: f32) -> Self {
		Self::Arc {
			centre: (quarter(cx), quarter(cy)),
			radii: (quarter(r), quarter(r)),
			start: (start * 1000.0).round() as i32,
			stop: (stop * 1000.0).round() as i32,
		}
	}

	/// A line through `points`, closed back to the first when `closed`.
	pub fn path(points: &[(f32, f32)], closed: bool) -> Self {
		Self::Path {
			points: points
				.iter()
				.map(|&(x, y)| (quarter(x), quarter(y)))
				.collect(),
			closed,
		}
	}

	/// Hachure inside the polygon through `points`.
	pub fn hatch(points: &[(f32, f32)], gap: f32, angle: i32) -> Self {
		Self::Hatch {
			points: points
				.iter()
				.map(|&(x, y)| (quarter(x), quarter(y)))
				.collect(),
			gap: quarter(gap),
			angle,
		}
	}

	/// Hachure inside the box at `(x, y)`, `width` by `height`.
	pub fn hatch_box(x: f32, y: f32, width: f32, height: f32, gap: f32, angle: i32) -> Self {
		Self::hatch(
			&[
				(x, y),
				(x + width, y),
				(x + width, y + height),
				(x, y + height),
			],
			gap,
			angle,
		)
	}

	/// The lamp's halftone: the bible's 4.4 pt pitch at 22°, dots r 1.05.
	pub fn halftone(cx: f32, cy: f32, radius: f32) -> Self {
		Self::Halftone {
			centre: (quarter(cx), quarter(cy)),
			radius: quarter(radius),
			pitch: quarter(4.4),
			dot: quarter(1.05),
			angle: 22,
		}
	}

	/// A disc of `radius` round `(cx, cy)`, lit `fraction` of the way from
	/// new to full.
	pub fn waxing(cx: f32, cy: f32, radius: f32, fraction: f64) -> Self {
		Self::Waxing {
			centre: (quarter(cx), quarter(cy)),
			radius: quarter(radius),
			phase: (fraction.clamp(0.0, 1.0) * 1000.0).round() as i32,
		}
	}

	/// Whether the mark is a filled area rather than a stroke.
	pub fn is_fill(&self) -> bool {
		matches!(self, Self::Halftone { .. } | Self::Waxing { .. })
	}
}

/// One stroke: a mark, the seed its wobble is drawn from, and the passes
/// of the pen. Two passes for outlines that carry state, one for rules,
/// ticks and hatches, never three (§8).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Recipe {
	pub mark: Mark,
	pub seed: u32,
	pub passes: u8,
}

impl Recipe {
	/// `mark` drawn in two passes.
	pub fn twice(mark: Mark, seed: u32) -> Self {
		Self {
			mark,
			seed,
			passes: 2,
		}
	}

	/// `mark` drawn in one pass.
	pub fn once(mark: Mark, seed: u32) -> Self {
		Self {
			mark,
			seed,
			passes: 1,
		}
	}
}

/// The key a stroke is kept under: its recipe, and the hand it was drawn
/// in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Kept {
	recipe: Recipe,
	roughness: u64,
}

thread_local! {
	static KEPT: RefCell<HashMap<Kept, PathData>> = RefCell::new(HashMap::new());
	static DRAWN: Cell<u64> = const { Cell::new(0) };
	static ASKED: Cell<u64> = const { Cell::new(0) };
}

/// How many strokes the hand has generated since the app started. A screen
/// built a second time adds none.
pub fn drawn() -> u64 {
	DRAWN.get()
}

/// How many times a stroke was asked for, generated or kept.
pub fn asked() -> u64 {
	ASKED.get()
}

/// The path for `recipe` in Second Ink's hand: generated the first time it
/// is asked for and kept from then on.
pub fn path(recipe: &Recipe) -> PathData {
	ASKED.set(ASKED.get() + 1);
	let key = Kept {
		recipe: recipe.clone(),
		roughness: SECOND_INK.roughness.to_bits(),
	};
	if let Some(path) = KEPT.with_borrow(|kept| kept.get(&key).cloned()) {
		return path;
	}
	let path = generate(recipe, &SECOND_INK);
	DRAWN.set(DRAWN.get() + 1);
	KEPT.with_borrow_mut(|kept| kept.insert(key, path.clone()));
	path
}

/// A generator for one shape, seeded by `seed`, filling with `fill`.
fn generator(seed: u32, theme: &Theme, fill: FillStyleChoice) -> rough::Generator {
	let options = rough::DrawOptions {
		max_randomness_offset: WANDER,
		roughness: theme.roughness,
		bowing: BOWING,
		seed: i64::from(seed),
		pen: rough::RoughPen::UNIFORM,
		..rough::DrawOptions::default()
	};
	let (style, gap, angle) = match fill {
		FillStyleChoice::None => (rough::FillStyle::None, 15.0, 320.0),
		FillStyleChoice::Hachure { gap, angle } => (rough::FillStyle::Hachure, gap, angle),
	};
	let filler = rough::Filler::new(
		style,
		rough::FillOptions {
			hachure_gap: gap,
			hachure_angle: angle,
			..rough::FillOptions::default()
		},
		// The fill draws from a stream of its own, so the outline and the
		// fill of one shape replay independently.
		rough::DrawConfig::new(rough::DrawOptions {
			seed: i64::from(seed.wrapping_add(0x9E37)),
			..options
		}),
	);
	rough::Generator::new(rough::DrawConfig::new(options), filler)
}

#[derive(Clone, Copy)]
enum FillStyleChoice {
	None,
	Hachure { gap: f64, angle: f64 },
}

/// The commands of the op sets of `kind` in `drawable`, keeping the first
/// `passes` passes of the pen.
fn commands(
	drawable: &rough::Drawable,
	kind: rough::OpSetKind,
	passes: u8,
	out: &mut Vec<PathCommand>,
) {
	let point = |point: rough::Point| Point::new(point.x as f32, point.y as f32);
	for set in drawable.sets.iter().filter(|set| set.kind == kind) {
		let mut keep = true;
		for op in &set.ops {
			match *op {
				rough::Op::Move { to, pass } => {
					keep = pass < u32::from(passes);
					if keep {
						out.push(PathCommand::MoveTo(point(to)));
					}
				}
				rough::Op::LineTo(to) if keep => out.push(PathCommand::LineTo(point(to))),
				rough::Op::CurveTo {
					control1,
					control2,
					end,
				} if keep => {
					out.push(PathCommand::CubicTo {
						control1: point(control1),
						control2: point(control2),
						to: point(end),
					});
				}
				rough::Op::LineTo(_) | rough::Op::CurveTo { .. } => {}
			}
		}
	}
}

/// Draws `recipe` in `theme`'s hand.
fn generate(recipe: &Recipe, theme: &Theme) -> PathData {
	let Recipe { mark, seed, passes } = recipe;
	let (seed, passes) = (*seed, (*passes).clamp(1, 2));
	let mut out = Vec::new();
	match mark {
		Mark::Halftone {
			centre,
			radius,
			pitch,
			dot,
			angle,
		} => {
			halftone(
				(points(centre.0), points(centre.1)),
				points(*radius),
				points(*pitch),
				points(*dot),
				f64::from(*angle),
				&mut out,
			);
		}
		Mark::Waxing {
			centre,
			radius,
			phase,
		} => {
			waxing(
				(points(centre.0), points(centre.1)),
				points(*radius),
				f64::from(*phase) / 1000.0,
				&mut out,
			);
		}
		Mark::Hatch {
			points: through,
			gap,
			angle,
		} => {
			let fill = FillStyleChoice::Hachure {
				gap: points(*gap),
				angle: f64::from(*angle),
			};
			let drawable = generator(seed, theme, fill).polygon(&rough_points(through));
			commands(&drawable, rough::OpSetKind::FillSketch, passes, &mut out);
		}
		Mark::Frame {
			width,
			height,
			over,
		} => {
			let (w, h, o) = (points(*width), points(*height), points(*over));
			let edges = [
				(-o, 0.0, w + o, 0.0),
				(w, -o, w, h + o),
				(w + o, h, -o, h),
				(0.0, h + o, 0.0, -o),
			];
			for (index, (x1, y1, x2, y2)) in edges.into_iter().enumerate() {
				// Each edge is its own stroke of the pen, from a seed of its
				// own, so the four never wobble alike.
				let seed = seed.wrapping_add(index as u32 * 7919);
				let drawable = generator(seed, theme, FillStyleChoice::None).line(x1, y1, x2, y2);
				commands(&drawable, rough::OpSetKind::Path, passes, &mut out);
			}
		}
		Mark::Line { .. } | Mark::Ellipse { .. } | Mark::Arc { .. } | Mark::Path { .. } => {
			let drawable = outline(mark, &mut generator(seed, theme, FillStyleChoice::None));
			commands(&drawable, rough::OpSetKind::Path, passes, &mut out);
		}
	}
	PathData::from_commands(out).expect("a drawn stroke is finite")
}

/// The generator's drawing of a mark that is one outline.
fn outline(mark: &Mark, generator: &mut rough::Generator) -> rough::Drawable {
	match mark {
		Mark::Line { from, to } => {
			generator.line(points(from.0), points(from.1), points(to.0), points(to.1))
		}
		Mark::Ellipse { centre, radii } => {
			generator.ellipse(
				points(centre.0),
				points(centre.1),
				points(radii.0) * 2.0,
				points(radii.1) * 2.0,
			)
		}
		Mark::Arc {
			centre,
			radii,
			start,
			stop,
		} => {
			generator.arc(
				points(centre.0),
				points(centre.1),
				points(radii.0) * 2.0,
				points(radii.1) * 2.0,
				f64::from(*start) / 1000.0,
				f64::from(*stop) / 1000.0,
				false,
			)
		}
		Mark::Path {
			points: through,
			closed,
		} => generator.linear_path(&rough_points(through), *closed),
		Mark::Frame { .. } | Mark::Hatch { .. } | Mark::Halftone { .. } | Mark::Waxing { .. } => {
			unreachable!("{mark:?} is not one outline")
		}
	}
}

fn rough_points(through: &[(Quarter, Quarter)]) -> Vec<rough::Point> {
	through
		.iter()
		.map(|&(x, y)| rough::Point::new(points(x), points(y)))
		.collect()
}

/// The handle length that makes four cubics a circle.
const KAPPA: f64 = 0.552_284_749_830_793_6;

/// A closed ellipse round `(cx, cy)` as four cubics.
fn ellipse_commands(cx: f64, cy: f64, rx: f64, ry: f64, out: &mut Vec<PathCommand>) {
	let p = |x: f64, y: f64| Point::new(x as f32, y as f32);
	let (kx, ky) = (rx * KAPPA, ry * KAPPA);
	out.push(PathCommand::MoveTo(p(cx + rx, cy)));
	out.push(PathCommand::CubicTo {
		control1: p(cx + rx, cy + ky),
		control2: p(cx + kx, cy + ry),
		to: p(cx, cy + ry),
	});
	out.push(PathCommand::CubicTo {
		control1: p(cx - kx, cy + ry),
		control2: p(cx - rx, cy + ky),
		to: p(cx - rx, cy),
	});
	out.push(PathCommand::CubicTo {
		control1: p(cx - rx, cy - ky),
		control2: p(cx - kx, cy - ry),
		to: p(cx, cy - ry),
	});
	out.push(PathCommand::CubicTo {
		control1: p(cx + kx, cy - ry),
		control2: p(cx + rx, cy - ky),
		to: p(cx + rx, cy),
	});
	out.push(PathCommand::Close);
}

/// Dots on a turned grid inside a circle.
fn halftone(
	centre: (f64, f64),
	radius: f64,
	pitch: f64,
	dot: f64,
	angle: f64,
	out: &mut Vec<PathCommand>,
) {
	let (sin, cos) = angle.to_radians().sin_cos();
	let steps = (radius / pitch).ceil() as i32 + 1;
	for row in -steps..=steps {
		for column in -steps..=steps {
			let (u, v) = (f64::from(column) * pitch, f64::from(row) * pitch);
			let (x, y) = (u * cos - v * sin, u * sin + v * cos);
			if x.hypot(y) + dot <= radius {
				ellipse_commands(centre.0 + x, centre.1 + y, dot, dot, out);
			}
		}
	}
}

/// The lit part of a waxing disc: the right half-circle, closed by a
/// half-ellipse terminator whose width follows the phase.
fn waxing(centre: (f64, f64), radius: f64, phase: f64, out: &mut Vec<PathCommand>) {
	let (cx, cy) = centre;
	let p = |x: f64, y: f64| Point::new(x as f32, y as f32);
	let k = radius * KAPPA;
	// The terminator's half-width: at a quarter the disc is a thin crescent
	// bowing right, at a half a straight edge, past a half it bows left.
	let terminator = radius * (2.0 * phase - 1.0);
	let t = terminator * KAPPA;
	out.push(PathCommand::MoveTo(p(cx, cy - radius)));
	// The lit limb, top to bottom down the right.
	out.push(PathCommand::CubicTo {
		control1: p(cx + k, cy - radius),
		control2: p(cx + radius, cy - k),
		to: p(cx + radius, cy),
	});
	out.push(PathCommand::CubicTo {
		control1: p(cx + radius, cy + k),
		control2: p(cx + k, cy + radius),
		to: p(cx, cy + radius),
	});
	// The terminator, bottom to top.
	out.push(PathCommand::CubicTo {
		control1: p(cx - t, cy + radius),
		control2: p(cx - terminator, cy + k),
		to: p(cx - terminator, cy),
	});
	out.push(PathCommand::CubicTo {
		control1: p(cx - terminator, cy - k),
		control2: p(cx - t, cy - radius),
		to: p(cx, cy - radius),
	});
	out.push(PathCommand::Close);
}

/// How one layer of a plate is inked.
#[derive(Clone, Copy, Debug)]
pub enum Ink {
	/// A colour of its own.
	Colour(Color),
	/// The plate's text colour, so a pressed control's `active:text-[…]`
	/// can reveal a layer that is clear at rest.
	Current,
}

impl From<crate::theme::Rgb> for Ink {
	fn from(colour: crate::theme::Rgb) -> Self {
		Self::Colour(colour.color())
	}
}

impl From<Color> for Ink {
	fn from(colour: Color) -> Self {
		Self::Colour(colour)
	}
}

impl Ink {
	fn paint(self) -> Paint {
		match self {
			Self::Colour(colour) => Paint::Color(colour),
			Self::Current => Paint::CurrentColor,
		}
	}
}

/// One printed layer: strokes and fills in a box `width` by `height`
/// points, stretched over its parent and lying beneath its content.
pub struct Plate {
	width: f32,
	height: f32,
	paths: Vec<VectorPath>,
}

impl Plate {
	/// An empty plate whose box is `width` by `height` points.
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			width: width.max(1.0),
			height: height.max(1.0),
			paths: Vec::new(),
		}
	}

	/// Strokes `recipe` in `ink` at `width` points.
	#[must_use]
	pub fn stroke(mut self, recipe: &Recipe, ink: impl Into<Ink>, width: f32) -> Self {
		let stroke = leptoskit::prelude::Stroke::new(ink.into().paint(), width)
			.expect("a positive width")
			.round();
		self.paths
			.push(VectorPath::new(path(recipe)).stroke(stroke));
		self
	}

	/// Strokes `recipe` in `ink` at `width` points, dashed `on` and `off`.
	#[must_use]
	pub fn dashed(
		mut self,
		recipe: &Recipe,
		ink: impl Into<Ink>,
		width: f32,
		on: f32,
		off: f32,
	) -> Self {
		let stroke = leptoskit::prelude::Stroke::new(ink.into().paint(), width)
			.expect("a positive width")
			.round()
			.dashes(&[on, off], 0.0)
			.expect("positive dashes");
		self.paths
			.push(VectorPath::new(path(recipe)).stroke(stroke));
		self
	}

	/// Fills `recipe` (a halftone, a waxing disc, or any closed mark) in
	/// `ink`.
	#[must_use]
	pub fn fill(mut self, recipe: &Recipe, ink: impl Into<Ink>) -> Self {
		self.paths
			.push(VectorPath::new(path(recipe)).fill(ink.into().paint()));
		self
	}

	/// Fills a plain box at `(x, y)`, `width` by `height`, in `ink`: a
	/// flood, not a stroke, so it has no wobble.
	#[must_use]
	pub fn flood(mut self, x: f32, y: f32, width: f32, height: f32, ink: impl Into<Ink>) -> Self {
		let flood = PathData::rect(x, y, width, height).expect("a finite box");
		self.paths
			.push(VectorPath::new(flood).fill(ink.into().paint()));
		self
	}

	/// The plate, stretched over its parent: lay it first inside a
	/// `relative` box so it lies beneath the content and takes no room. It
	/// never takes a press.
	pub fn over(self) -> Element {
		self.vector().class("absolute inset-0").enabled(false)
	}

	/// The plate at its own size, `width` by `height` points.
	pub fn sized(self) -> Element {
		let class = format!("w-[{}px] h-[{}px]", self.width, self.height);
		self.vector().class(class).enabled(false)
	}

	fn vector(self) -> Element {
		let mut vector =
			Vector(ViewBox::new(0.0, 0.0, self.width, self.height).expect("a positive size"))
				.scaling(VectorScaling::Stretch);
		for path in self.paths {
			vector = vector.path(path);
		}
		vector.aria_hidden(true)
	}
}

/// A rough box over its parent in `colour` at `alpha`, drawn twice: the
/// ledger's box, for the screens not yet rebuilt on the kit. Lay it first
/// inside a `relative` container.
pub fn sketch(seed: u32, colour: crate::theme::Rgb, alpha: f64) -> Element {
	Plate::new(SKETCH_WIDTH, SKETCH_HEIGHT)
		.stroke(
			&Recipe::twice(Mark::frame(SKETCH_WIDTH, SKETCH_HEIGHT, 1.5), seed),
			colour.color_alpha(alpha),
			1.2,
		)
		.over()
}

/// The box a legacy sketch is drawn in before it is stretched over its
/// parent: about a ledger row's shape.
const SKETCH_WIDTH: f32 = 320.0;
const SKETCH_HEIGHT: f32 = 60.0;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_stroke_is_drawn_once_and_kept() {
		let recipe = Recipe::twice(Mark::frame(350.0, 54.0, 4.0), 71);
		let before = drawn();
		let first = path(&recipe);
		assert_eq!(drawn(), before + 1, "the first ask draws it");
		let second = path(&recipe);
		assert_eq!(drawn(), before + 1, "a second ask must not draw it again");
		assert_eq!(first, second);
	}

	#[test]
	fn the_same_seed_draws_the_same_line_and_another_does_not() {
		let one = generate(
			&Recipe::once(Mark::line(0.0, 0.0, 200.0, 0.0), 7),
			&SECOND_INK,
		);
		let again = generate(
			&Recipe::once(Mark::line(0.0, 0.0, 200.0, 0.0), 7),
			&SECOND_INK,
		);
		let other = generate(
			&Recipe::once(Mark::line(0.0, 0.0, 200.0, 0.0), 8),
			&SECOND_INK,
		);
		assert_eq!(one, again);
		assert_ne!(one, other);
	}

	#[test]
	fn passes_are_the_pens_and_never_three() {
		let moves = |recipe: &Recipe| {
			generate(recipe, &SECOND_INK)
				.commands()
				.iter()
				.filter(|command| matches!(command, PathCommand::MoveTo(_)))
				.count()
		};
		let line = Mark::line(0.0, 0.0, 120.0, 4.0);
		assert_eq!(moves(&Recipe::once(line.clone(), 3)), 1);
		assert_eq!(moves(&Recipe::twice(line.clone(), 3)), 2);
		let three = Recipe {
			mark: line,
			seed: 3,
			passes: 3,
		};
		assert_eq!(moves(&three), 2, "a third pass is never drawn");
		// A frame is four edges, each its own stroke.
		assert_eq!(moves(&Recipe::twice(Mark::frame(100.0, 40.0, 2.5), 5)), 8);
	}

	#[test]
	fn a_frame_runs_past_its_corners() {
		let frame = generate(
			&Recipe::once(Mark::frame(100.0, 40.0, 4.0), 11),
			&SECOND_INK,
		);
		let xs: Vec<f32> = frame
			.commands()
			.iter()
			.filter_map(|command| {
				match command {
					PathCommand::MoveTo(point) | PathCommand::LineTo(point) => Some(point.x),
					PathCommand::CubicTo { to, .. } => Some(to.x),
					_ => None,
				}
			})
			.collect();
		let least = xs.iter().copied().fold(f32::INFINITY, f32::min);
		let most = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
		assert!(least < -1.5, "{least}");
		assert!(most > 101.5, "{most}");
	}

	#[test]
	fn a_hatch_stays_inside_its_box() {
		let hatch = generate(
			&Recipe::once(Mark::hatch_box(0.0, 0.0, 60.0, 10.0, 4.0, 60), 2),
			&SECOND_INK,
		);
		assert!(!hatch.is_empty());
		for command in hatch.commands() {
			if let PathCommand::MoveTo(point) | PathCommand::LineTo(point) = command {
				assert!(
					(-3.0..=63.0).contains(&point.x) && (-3.0..=13.0).contains(&point.y),
					"{point:?}"
				);
			}
		}
	}

	#[test]
	fn the_halftone_keeps_inside_the_lamp() {
		let dots = generate(
			&Recipe::once(Mark::halftone(132.0, 132.0, 132.0), 0),
			&SECOND_INK,
		);
		let count = dots
			.commands()
			.iter()
			.filter(|command| matches!(command, PathCommand::MoveTo(_)))
			.count();
		// About the disc's area over the pitch squared.
		let expected = std::f64::consts::PI * 132.0 * 132.0 / (4.4 * 4.4);
		assert!(
			(count as f64 - expected).abs() < expected * 0.1,
			"{count} dots"
		);
	}

	#[test]
	fn the_theme_is_second_ink() {
		assert_eq!(SECOND_INK.light.paper, Rgba::hex(0x0034_2F33));
		assert_eq!(SECOND_INK.light.ink, Rgba::hex(0x00EC_E3CF));
		assert_eq!(SECOND_INK.light, SECOND_INK.dark, "vigil, always");
	}
}
