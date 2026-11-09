//! The page's drawings: a sigil, a crosshair, the ring under a husk, a soul
//! orb and the six callings' weapons, each drawn as a [`Vector`] in fine
//! strokes, as the game's ledger is ruled. Every drawing is decoration: the
//! text beside it says what it shows.

use std::fmt::Write as _;

use leptoskit::prelude::*;

/// The lights of a sigil, in the order they are traced, in a 100 by 75 box:
/// one to a column, each at one of three heights and never at the height of
/// the one beside it, as the game lays them out.
pub const SIGIL: [(f32, f32); 5] = [
	(10.0, 30.0),
	(29.0, 12.0),
	(50.0, 22.0),
	(71.0, 8.0),
	(90.0, 26.0),
];

/// `#rrggbb` at `alpha`.
pub fn colour(hex: &str, alpha: u8) -> Color {
	let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
	Color::rgba((value >> 16) as u8, (value >> 8) as u8, value as u8, alpha)
}

/// Bone, for the ring a player aims with.
const BONE_HEX: &str = "#e9e4d4";

fn path(data: &str) -> PathData {
	PathData::parse(data).expect("the drawing's path is valid")
}

/// A circle, as path data.
fn circle(cx: f32, cy: f32, r: f32) -> String {
	format!(
		"M{} {cy} A{r} {r} 0 1 0 {} {cy} A{r} {r} 0 1 0 {} {cy} Z",
		cx - r,
		cx + r,
		cx - r
	)
}

/// An ellipse lying on the floor, as path data.
fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> String {
	format!(
		"M{} {cy} A{rx} {ry} 0 1 0 {} {cy} A{rx} {ry} 0 1 0 {} {cy} Z",
		cx - rx,
		cx + rx,
		cx - rx
	)
}

fn line(hex: &str, alpha: u8, width: f32) -> Stroke {
	Stroke::new(colour(hex, alpha), width)
		.expect("a positive width")
		.round()
}

fn drawing(width: f32, height: f32) -> Element {
	Vector(ViewBox::new(0.0, 0.0, width, height).expect("a positive size")).aria_hidden(true)
}

/// Adds a point to a polyline's path data.
fn push_point(data: &mut String, (x, y): (f32, f32)) {
	data.push_str(if data.is_empty() { "M" } else { " L" });
	write!(data, "{x} {y}").expect("writing to a String cannot fail");
}

/// The thread a traced sigil leaves, through `points` in a 100 by 75 box.
pub fn thread(points: &[(f32, f32)], hex: &str) -> Element {
	let mut data = String::new();
	for point in points {
		push_point(&mut data, *point);
	}
	drawing(100.0, 75.0).path(VectorPath::new(path(&data)).stroke(line(hex, 0xb0, 1.2)))
}

/// A sigil part traced: the first `caught` of [`SIGIL`]'s lights caught
/// and threaded, the next one ringed, and the rest faint, with the way to
/// them dashed. It is drawn in the top 100 by 40 of the sigil's box, the band
/// its lights hang in.
pub fn sigil(caught: usize, hex: &str) -> Element {
	let (mut done, mut ahead) = (String::new(), String::new());
	for (index, point) in SIGIL.iter().enumerate() {
		if index < caught {
			push_point(&mut done, *point);
		}
		if index + 1 >= caught {
			push_point(&mut ahead, *point);
		}
	}
	let faint = Stroke::new(colour(hex, 0x55), 1.0)
		.expect("a positive width")
		.dashes(&[2.0, 4.0], 0.0)
		.expect("positive dashes");
	let mut art = drawing(100.0, 40.0).path(VectorPath::new(path(&ahead)).stroke(faint));
	if caught > 1 {
		art = art.path(VectorPath::new(path(&done)).stroke(line(hex, 0xc0, 1.4)));
	}
	for (index, (x, y)) in SIGIL.iter().enumerate() {
		let alpha = if index < caught { 0xff } else { 0x55 };
		art = art
			.path(VectorPath::new(path(&circle(*x, *y, 4.5))).fill(colour(hex, alpha / 5)))
			.path(VectorPath::new(path(&circle(*x, *y, 1.8))).fill(colour(hex, alpha)));
		if index == caught {
			art = art.path(
				VectorPath::new(path(&circle(*x, *y, 7.5))).stroke(line(BONE_HEX, 0xdd, 1.3)),
			);
		}
	}
	art
}

/// The crosshair at the middle of the screen: a ring, four ticks and a dot.
pub fn crosshair(hex: &str) -> Element {
	drawing(100.0, 100.0)
		.path(VectorPath::new(path(&circle(50.0, 50.0, 30.0))).stroke(line(hex, 0xee, 1.4)))
		.path(
			VectorPath::new(path(
				"M50 8 L50 26 M50 74 L50 92 M8 50 L26 50 M74 50 L92 50",
			))
			.stroke(line(hex, 0xee, 1.4)),
		)
		.path(VectorPath::new(path(&circle(50.0, 50.0, 1.4))).fill(colour(hex, 0xff)))
}

/// The ring on the floor under a husk, in the colour of what it is doing.
pub fn floor_ring(hex: &str) -> Element {
	drawing(120.0, 44.0)
		.path(VectorPath::new(path(&ellipse(60.0, 22.0, 54.0, 17.0))).fill(colour(hex, 0x14)))
		.path(VectorPath::new(path(&ellipse(60.0, 22.0, 54.0, 17.0))).stroke(line(hex, 0xff, 2.0)))
		.path(VectorPath::new(path(&ellipse(60.0, 22.0, 40.0, 12.0))).stroke(line(hex, 0x66, 1.0)))
}

/// A soul orb, and the three rings its pulses leave.
pub fn orb(hex: &str) -> Element {
	let mut art = drawing(100.0, 100.0);
	for (radius, alpha) in [(46.0, 0x22), (38.0, 0x44), (30.0, 0x77)] {
		art = art
			.path(VectorPath::new(path(&circle(50.0, 50.0, radius))).stroke(line(hex, alpha, 1.0)));
	}
	art.path(VectorPath::new(path(&circle(50.0, 50.0, 18.0))).fill(colour(hex, 0x30)))
		.path(VectorPath::new(path(&circle(50.0, 50.0, 12.0))).fill(colour(hex, 0xff)))
		.path(VectorPath::new(path(&circle(46.0, 46.0, 4.0))).fill(colour("#fff6e0", 0xcc)))
}
