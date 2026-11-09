//! The room's grid drawn over the fight (`REAPERS_ROOM_OVERLAY`): the
//! surfaces a husk can stand on, outlined at their own heights and
//! coloured by level, the part of each a body fits on, the leaps between
//! them as the arcs a husk's feet would follow, and the places it picks to
//! perch, peek, hide and stand in the open. It is how a real desk is
//! checked to have become a surface on the phone.
//!
//! It is worked out once, when the fight begins, from the same
//! [`RoomNav`] the hunt's husk walks; each render only places its points
//! on the screen, as a handful of paths: one for every pen and colour, so
//! that a room of a thousand spots costs a frame no more than a few lines.
//! Like the drawn room, it has no words.

use std::collections::BTreeMap;

use reapers_rules::HuntRules;
use reapers_rules::RoomGrid;
use reapers_rules::RoomNav;
use reapers_rules::RoomSpotKind;
use reapers_rules::Vec3;
use reapers_rules::Vessel;

use crate::hunt::overlay::Lens;
use crate::hunt::overlay::Sketch;
use crate::hunt::overlay::colour;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;

/// Each level's colour, from the floor up: the floor in ash, the rest in
/// colours that stand apart from each other and from the room's own lines.
const LEVELS: [Rgb; 5] = [
	ASH,
	Rgb::from_hex(0x00F2_B84B),
	Rgb::from_hex(0x0055_D6C2),
	Rgb::from_hex(0x00E8_6FA8),
	Rgb::from_hex(0x009B_8CF2),
];
/// A leap's colour.
const LEAP: Rgb = Rgb::from_hex(0x00F4_EBD0);
/// The longest piece a line is drawn in, in metres, so that a line running
/// past the player is drawn wherever it is in front.
const PIECE: f64 = 0.4;
/// How far over a surface its outline is drawn, so it is not lost in it.
const RAISE: f64 = 0.01;
/// How many pieces a leap's arc is drawn in.
const ARC_PIECES: usize = 8;

/// How a line of the overlay is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pen {
	/// The edge of a surface.
	Surface,
	/// The edge of the part of a surface a body fits on.
	Footing,
	/// A leap's arc.
	Leap,
}

impl Pen {
	const fn alpha(self) -> f64 {
		match self {
			Self::Surface => 0.8,
			Self::Footing => 0.45,
			Self::Leap => 0.55,
		}
	}

	const fn width(self) -> f64 {
		match self {
			Self::Surface => 1.6,
			Self::Footing => 1.0,
			Self::Leap => 1.2,
		}
	}
}

/// Every line drawn with one pen in one colour, drawn as one path: the
/// points of each line in the hunt's frame, with a gap between one line
/// and the next.
#[derive(Clone, Debug, PartialEq)]
struct Stroke {
	pen: Pen,
	tint: Rgb,
	points: Vec<Option<Vec3>>,
}

/// The room's grid as a husk of one vessel reads it, ready to draw.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomOverlay {
	strokes: Vec<Stroke>,
	/// The spots of each colour.
	spots: Vec<(Rgb, Vec<Vec3>)>,
}

impl RoomOverlay {
	/// The overlay of `grid` for a husk of `vessel`, read as the hunt
	/// reads it.
	pub fn new(grid: RoomGrid, vessel: Vessel) -> Self {
		let profile = vessel.locomotion();
		let nav = RoomNav::for_hunt(
			grid,
			profile,
			HuntRules::STAGING_FAR_METRES,
			HuntRules::NEAREST_METRES,
		);
		let grid = &nav.grid;
		let floor = -f64::from(grid.eye_height_centimetres) / 100.0;
		let mut overlay = Self {
			strokes: Vec::new(),
			spots: Vec::new(),
		};
		for region in &nav.regions {
			let tint = LEVELS[usize::from(region.level_index) % LEVELS.len()];
			let height = floor + region.height_metres() + RAISE;
			let surface = overlay.stroke(Pen::Surface, tint);
			outline(grid, &region.cells, height, surface);
			let footing = overlay.stroke(Pen::Footing, tint);
			outline(grid, &region.standable_cells, height, footing);
		}
		// A leap is drawn where the feet go, half a body under its middle.
		let feet = Vec3::new(0.0, -profile.half_height(), 0.0);
		let leaps = overlay.stroke(Pen::Leap, LEAP);
		for link in &nav.links {
			let seconds = link.leap.flight_seconds();
			leaps.extend((0..=ARC_PIECES).map(|piece| {
				let at = seconds * piece as f64 / ARC_PIECES as f64;
				Some(link.leap.position_at_seconds(at) + feet)
			}));
			leaps.push(None);
		}
		for spot in &nav.spots {
			let tint = match spot.kind {
				RoomSpotKind::Perch => LEVELS[1],
				RoomSpotKind::Peek => LEVELS[2],
				RoomSpotKind::Hide => LEVELS[3],
				RoomSpotKind::Open => BONE,
			};
			match overlay.spots.iter_mut().find(|(each, _)| *each == tint) {
				Some((_, points)) => points.push(spot.point),
				None => overlay.spots.push((tint, vec![spot.point])),
			}
		}
		overlay
	}

	/// The points of the stroke drawn with `pen` in `tint`, begun if there
	/// is none yet.
	fn stroke(&mut self, pen: Pen, tint: Rgb) -> &mut Vec<Option<Vec3>> {
		let index = self
			.strokes
			.iter()
			.position(|stroke| stroke.pen == pen && stroke.tint == tint)
			.unwrap_or_else(|| {
				self.strokes.push(Stroke {
					pen,
					tint,
					points: Vec::new(),
				});
				self.strokes.len() - 1
			});
		&mut self.strokes[index].points
	}

	/// Draws the overlay where it falls on the screen.
	pub fn draw(&self, sketch: &mut Sketch, lens: &Lens) {
		for stroke in &self.strokes {
			let points: Vec<_> = stroke
				.points
				.iter()
				.map(|point| point.and_then(|point| lens.place(point)))
				.collect();
			sketch.line_through(&points, stroke.tint, stroke.pen.alpha(), stroke.pen.width());
		}
		for (tint, points) in &self.spots {
			sketch.dots(
				points.iter().filter_map(|&point| lens.place(point)),
				2.5,
				colour(*tint, 0.9),
			);
		}
	}
}

/// Outlines the cells of `cells` at `height` into `stroke`: every side of
/// a cell whose neighbour is not among them, joined into straight runs and
/// cut into pieces no longer than [`PIECE`].
fn outline(grid: &RoomGrid, cells: &[usize], height: f64, stroke: &mut Vec<Option<Vec3>>) {
	if cells.is_empty() {
		return;
	}
	let mut inside = vec![false; grid.cell_count()];
	for &cell in cells {
		if let Some(slot) = inside.get_mut(cell) {
			*slot = true;
		}
	}
	let member = |x: i64, z: i64| grid.contains(x, z) && inside[grid.index_of(x, z)];
	// The sides that face out, by the line they lie on: rows of sides
	// along x at each z, and columns along z at each x, both on the
	// doubled grid so a side between cells has a whole number.
	let mut rows: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
	let mut columns: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
	for &cell in cells {
		let (x, z) = (grid.x_of(cell), grid.z_of(cell));
		for (dz, row) in [(-1, 2 * z - 1), (1, 2 * z + 1)] {
			if !member(x, z + dz) {
				rows.entry(row).or_default().push(x);
			}
		}
		for (dx, column) in [(-1, 2 * x - 1), (1, 2 * x + 1)] {
			if !member(x + dx, z) {
				columns.entry(column).or_default().push(z);
			}
		}
	}
	let point = |x: f64, z: f64| {
		let at = grid.world_direction(Vec3::new(
			(x - f64::from(grid.player_x)) * RoomGrid::CELL_METRES,
			0.0,
			(z - f64::from(grid.player_z)) * RoomGrid::CELL_METRES,
		));
		Vec3::new(at.x, height, at.z)
	};
	let mut add = |from: Vec3, to: Vec3| {
		let pieces = ((to - from).length() / PIECE).ceil().max(1.0) as usize;
		stroke.extend(
			(0..=pieces).map(|piece| Some(from + (to - from) * (piece as f64 / pieces as f64))),
		);
		stroke.push(None);
	};
	for (row, mut xs) in rows {
		let z = row as f64 / 2.0;
		for (first, last) in runs(&mut xs) {
			add(point(first as f64 - 0.5, z), point(last as f64 + 0.5, z));
		}
	}
	for (column, mut zs) in columns {
		let x = column as f64 / 2.0;
		for (first, last) in runs(&mut zs) {
			add(point(x, first as f64 - 0.5), point(x, last as f64 + 0.5));
		}
	}
}

/// The unbroken runs of `values`, as their first and last.
fn runs(values: &mut [i64]) -> Vec<(i64, i64)> {
	values.sort_unstable();
	let mut runs: Vec<(i64, i64)> = Vec::new();
	for &value in values.iter() {
		match runs.last_mut() {
			Some((_, last)) if value <= *last + 1 => *last = value.max(*last),
			_ => runs.push((value, value)),
		}
	}
	runs
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_kitchen_is_drawn_with_a_leap_for_every_link() {
		let mut grid = reapers_world::RoomGridBuilder::new();
		let room = crate::dev::drawn_room::kitchen();
		crate::dev::drawn_room::read_grid(&room, &mut grid);
		let grid = grid.finish();
		let nav = RoomNav::for_hunt(
			grid.clone(),
			Vessel::Penguin.locomotion(),
			HuntRules::STAGING_FAR_METRES,
			HuntRules::NEAREST_METRES,
		);
		let overlay = RoomOverlay::new(grid, Vessel::Penguin);
		// Each line ends in a gap, so a pen draws as many lines as gaps.
		let lines = |pen: Pen| {
			overlay
				.strokes
				.iter()
				.filter(|stroke| stroke.pen == pen)
				.map(|stroke| stroke.points.iter().filter(|point| point.is_none()).count())
				.sum::<usize>()
		};
		assert_eq!(lines(Pen::Leap), nav.links.len());
		assert!(lines(Pen::Surface) > 0, "the surfaces are outlined");
		assert!(lines(Pen::Footing) > 0, "the footing is outlined");
		let spots: usize = overlay.spots.iter().map(|(_, points)| points.len()).sum();
		assert_eq!(spots, nav.spots.len());
		assert!(overlay.strokes.len() <= 2 * LEVELS.len() + 1);
	}

	#[test]
	fn runs_join_neighbours_and_break_at_gaps() {
		assert_eq!(runs(&mut [3, 1, 2, 7, 8, 5]), [(1, 3), (5, 5), (7, 8)]);
		assert_eq!(runs(&mut []), []);
	}
}
