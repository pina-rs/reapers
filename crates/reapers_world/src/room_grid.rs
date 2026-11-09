//! Reading a room into the grid a husk walks through.
//!
//! [`RoomGridBuilder`] takes the depth pictures [`RoomMap`] already reads
//! and turns them into a [`RoomGrid`]: the gravity-aligned grid of 10 cm
//! cells the rules move a husk through. It is the phone's half of the
//! extraction described in the research's section 3.2 and in
//! `docs/environment.md`.
//!
//! The steps, in order:
//!
//! 1. Drop flying pixels, the depth a model blurs across a silhouette.
//! 2. Unproject with the picture's pose, and scale every length by the
//!    room's floor: one picture's feet are not trusted, only its shape.
//! 3. Chart the points into 5 cm cells, each holding a histogram of what
//!    stands above the floor. Carve the cells a ray crossed before it hit
//!    something, so a point above free space is dropped.
//! 4. Find the top of every fine cell, then whether a 3 by 3 patch of tops
//!    lies on a level plane.
//! 5. Snap every top to the few levels the room has, and leave the rest as
//!    obstacles with their own height.
//! 6. Turn the grid to the room's axes, folded to one quarter turn.
//! 7. Downsample to the game's 10 cm grid by majority vote.
//! 8. Mark the floor no picture could see, directly behind something that
//!    stands up, as hidden rather than unknown.
//!
//! It holds no camera, no clock and no source of chance, so it is tested
//! against rooms drawn by hand. Every number it reads is a constant here
//! with the reason it was chosen.

use reapers_rules::HuntRules;
use reapers_rules::RoomCell;
use reapers_rules::RoomCellKind;
use reapers_rules::RoomGrid;
use reapers_rules::Vessel;

use crate::depth_frame::DepthFrame;
use crate::math;
use crate::math::PI;
use crate::room_map::Cloud;
use crate::room_map::RoomMap;
use crate::room_map::SELF_METRES;
use crate::room_map::STRIDE;
use crate::room_map::surface_normals;

/// How far the grid reaches either side of the player, in metres. A phone
/// reads a few metres around itself, and every drawn room's walls come in
/// inside five metres, so this holds the room and a margin.
const EXTENT: f64 = 10.0;

/// Fine cells per side: 10 m at 5 cm.
const FINE_SIDE: usize = 200;

/// Grid cells per side: the same 10 m at the game's 10 cm.
const GRID_SIDE: usize = 100;

/// The player's cell, in the middle of the grid.
const PLAYER_CELL: u8 = (GRID_SIDE / 2) as u8;

/// One fine cell is 5 cm across: finer than the game's 10 cm, so a desk's
/// top is a run of cells and not a staircase.
const FINE_CELL: f64 = 0.05;

/// The fine grid is offset half a cell from the 10 cm grid, so a wall that
/// stands on a 10 cm boundary falls inside one fine cell instead of being
/// split between two.
const FINE_OFFSET: f64 = FINE_CELL / 2.0;

/// Height bins are 4 cm deep. Sixty-three of them reach 2.52 m, past the
/// tallest obstacle the grid's five-bit payload can carry (2.48 m).
const HEIGHT_BIN: f64 = 0.04;
const HEIGHT_BINS: usize = 63;

/// Two sampled points in one bin make a top; one is a stray reading.
const MIN_POINTS_OF_TOP: u8 = 2;

/// A step of more than 5% between one pixel and a 4-neighbour is the model
/// blurring across a silhouette, not a surface.
const FLYING_PIXEL_SHARE: f64 = 0.05;

/// One carving ray in every four each way: the research's stride, against a
/// few thousand rays a picture rather than a hundred thousand pixels.
const CARVE_STRIDE: usize = 4;

/// A carving ray is sampled every 4 cm, less than a fine cell, so no cell it
/// crosses is missed.
const CARVE_STEP: f64 = 0.04;

/// A carving ray stops this far before the surface it hit: two fine cells,
/// so neither the surface's own cell nor the cell before it is carved. A
/// wall standing on a cell boundary shares that last cell with the floor
/// in front of it, and carving it would cut the wall's top away.
const CARVE_MARGIN: f64 = 2.0 * FINE_CELL;

/// A point less than 6 cm above the lowest ray that crossed its cell is a
/// grazing reading of the surface; above that it stands in free space.
const CARVE_SLACK: f64 = 0.06;

/// A point up to 7 cm below the floor is a reading of the floor itself: a
/// picture's own floor is measured to a few millimetres, and every point
/// below that is not a mistake. `RoomMap` uses the same slack.
const FLOOR_SLACK: f64 = 0.07;

/// A plane through a 3 by 3 patch of tops is level within 3 cm.
const PLANE_RESIDUAL: f64 = 0.03;

/// A level plane is no steeper than 10°, the research's kernel test.
const PLANE_SLOPE: f64 = 0.1763;

/// The level histogram has one bin per centimetre of height.
const LEVEL_BIN: f64 = 0.01;
const LEVEL_BINS: usize = 252;

/// The histogram is smoothed with a 2 cm standard deviation, so one surface
/// that wobbles gives one peak.
const LEVEL_BLUR: f64 = 0.02;

/// A peak's area, and the smoothing, are counted over this many centimetres
/// either side of it: five is two and a half deviations, enough for one
/// surface and not the next.
const LEVEL_WINDOW: i64 = 5;

/// A peak needs 0.08 m² of level top before it is a level.
const LEVEL_PEAK_AREA: f64 = 0.08;

/// Two peaks closer than 8 cm are one level.
const LEVEL_MERGE: f64 = 0.08;

/// A cell snaps to a level within 6 cm of it, and 1 cm more for every metre
/// away, where depth is less sure.
const LEVEL_SNAP: f64 = 0.06;
const LEVEL_SNAP_PER_METRE: f64 = 0.01;

/// A top seen within 10 cm of the eye is a wall: nothing above the eye is
/// charted, so its top was never seen and no husk crosses it.
const WALL_BELOW_EYE: f64 = 0.10;

/// The room-axis histogram has one bin per degree of one quarter turn.
const YAW_BINS: usize = 90;

/// A yaw peak is smoothed over this many degrees either side, so one wall
/// seen from several angles gives one peak.
const YAW_SMOOTH_DEGREES: i64 = 2;

/// A yaw peak's direction is averaged over this many degrees either side of
/// it, where the wall's normals really are.
const YAW_PEAK_DEGREES: i64 = 4;

/// A `RoomGrid` stores its turn in one of 256 steps of a full turn.
const YAW_STEPS: i64 = 256;

/// A surface within 60° of vertical is vertical structure for the room's
/// axes: a wall, a desk's front.
const YAW_HORIZONTAL_SHARE: f64 = 0.5;

/// Fewer sampled points than this on vertical structure and the room has no
/// axis worth turning to.
const YAW_MIN_WEIGHT: f64 = 50.0;

/// How many rays are cast from the player to find floor hidden behind what
/// stands up: half a degree apart, about 4 cm at five metres.
const HIDDEN_RAYS: usize = 720;

/// A hidden ray is sampled every 4 cm.
const HIDDEN_STEP: f64 = 0.04;

/// The most levels a grid's five-bit payload carries.
const MAX_LEVELS: usize = 32;

/// The floor the fine grid keeps its heights against: the hunt's usual eye
/// height. Every picture's heights are divided by its own floor and then
/// multiplied by this, so a picture that reads the whole room long or short
/// leaves no trace; the final floor turns them back into metres.
const NOMINAL_FLOOR: f64 = HuntRules::EYE_HEIGHT;

/// The fine cells per side, squared.
const FINE_CELLS: usize = FINE_SIDE * FINE_SIDE;

/// One height histogram per fine cell.
const FINE_HEIGHT_COUNTS: usize = FINE_CELLS * HEIGHT_BINS;

/// The grid's own cell, in metres.
const GRID_CELL: f64 = RoomGrid::CELL_METRES;

/// A grid read from depth pictures, before it is finished.
///
/// Feed it every picture the survey takes with
/// [`add_frame`](Self::add_frame), then call [`finish`](Self::finish). It
/// keeps its own [`RoomMap`] for the floor and the per-picture scale, so
/// what it reads agrees with the plan a hunt plays inside.
#[derive(Clone, Debug)]
pub struct RoomGridBuilder {
	/// The floor and the picture scale, as the hunt's map finds them.
	map: RoomMap,

	/// The count of points in each height bin of each fine cell.
	counts: Vec<u8>,

	/// The lowest height above the floor at which a carving ray crossed each
	/// fine cell, or infinity where none did.
	clear: Vec<f32>,

	/// How much vertical structure faces each degree of one quarter turn.
	yaw_weights: [f64; YAW_BINS],

	/// The sum of every picture's own floor, in metres, and how many were
	/// added: their mean is the room's floor.
	floor_sum: f64,
	floor_pictures: usize,
}

impl Default for RoomGridBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl RoomGridBuilder {
	/// A builder that has read nothing yet.
	pub fn new() -> Self {
		Self {
			// The map is only asked where the floor is; a ground body is all
			// it needs, and the smallest keeps its charting cheap.
			map: RoomMap::new(Vessel::Penguin),
			counts: vec![0; FINE_HEIGHT_COUNTS],
			clear: vec![f32::INFINITY; FINE_CELLS],
			yaw_weights: [0.0; YAW_BINS],
			floor_sum: 0.0,
			floor_pictures: 0,
		}
	}

	/// Takes the floor as measured by the platform: see
	/// [`RoomMap::measure_floor`]. Every picture is then charted against it,
	/// in real metres, instead of against its own floor.
	pub fn measure_floor(&mut self, eye_height: f64) -> bool {
		self.map.measure_floor(eye_height)
	}

	/// Pictures charted so far.
	pub fn frames(&self) -> usize {
		self.map.frames()
	}

	/// How far the floor is below the player's eye, in metres, as the
	/// pictures so far have it.
	pub fn floor_depth(&self) -> Option<f64> {
		self.room_floor()
	}

	/// Takes in one picture. Answers whether it was charted: a picture with
	/// no floor of its own, or one that disagrees with the floor the others
	/// found, teaches the grid nothing.
	pub fn add_frame(&mut self, frame: &DepthFrame) -> bool {
		if !self.map.add(frame) {
			return false;
		}

		let Some(picture_floor) = self.map.last_picture_floor() else {
			return false;
		};

		self.floor_sum += picture_floor;
		self.floor_pictures += 1;

		// The pictures' mean floor is the room's metres; until it is known,
		// the first picture is charted against itself.
		let scale = self.room_floor().unwrap_or(picture_floor) / picture_floor;
		let keep = flying_pixels(frame);
		let cloud = Cloud::of_filtered(frame, |column, row| keep[row * frame.width + column]);
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;

		self.chart(&cloud, columns, rows, scale, picture_floor);
		self.carve(frame, scale, picture_floor);

		true
	}

	/// How far below the eye the room's floor is, in metres: the mean of
	/// every picture's own floor, or the platform's measurement.
	fn room_floor(&self) -> Option<f64> {
		if self.map.floor_is_measured() {
			return self.map.floor_depth();
		}

		if self.floor_pictures == 0 {
			return None;
		}

		Some(self.floor_sum / self.floor_pictures as f64)
	}

	/// Charts one picture's points, and the directions its vertical
	/// structure faces, into the fine grid.
	///
	/// `scale` turns the picture's own metres into the room's, so its
	/// furniture stands where the other pictures put it. A height is kept as
	/// a share of the hunt's usual eye height, so the picture's scale is
	/// divided out of it.
	fn chart(
		&mut self,
		cloud: &Cloud,
		columns: usize,
		rows: usize,
		scale: f64,
		picture_floor: f64,
	) {
		let normals = surface_normals(cloud, columns, rows);

		for (index, normal) in normals.into_iter().enumerate() {
			if let Some(normal) = normal {
				let horizontal = normal.horizontal_share();

				if horizontal >= YAW_HORIZONTAL_SHARE {
					let bin = (normal.folded_angle() / (PI / 2.0) * YAW_BINS as f64) as usize;

					self.yaw_weights[bin.min(YAW_BINS - 1)] += horizontal;
				}
			}

			let x = f64::from(cloud.xs[index]) * scale;
			let y = f64::from(cloud.ys[index]);
			let z = f64::from(cloud.zs[index]) * scale;

			// A point that was not seen is NaN, and fails this.
			if !(x.is_finite() && y.is_finite() && z.is_finite()) {
				continue;
			}

			// The player's own hands and feet, and anything above the eye:
			// a ceiling, or a wall the grid calls a wall.
			if math::sqrt(x * x + z * z) < SELF_METRES || y > 0.0 {
				continue;
			}

			let height = (y + picture_floor) / picture_floor * NOMINAL_FLOOR;

			// A point at or a hair under the floor is the floor.
			if height < -FLOOR_SLACK || height >= HEIGHT_BIN * HEIGHT_BINS as f64 {
				continue;
			}

			let height = height.max(0.0);

			let Some(cell) = fine_cell(x, z) else {
				continue;
			};
			let bin = (height / HEIGHT_BIN) as usize;
			let at = cell * HEIGHT_BINS + bin;

			self.counts[at] = self.counts[at].saturating_add(1);
		}
	}

	/// Records, for every fine cell a ray crossed before it hit something,
	/// the lowest height above the floor at which it crossed. A point well
	/// above that flew, since the ray passed beneath it.
	fn carve(&mut self, frame: &DepthFrame, scale: f64, picture_floor: f64) {
		for row in (0..frame.height).step_by(CARVE_STRIDE) {
			for column in (0..frame.width).step_by(CARVE_STRIDE) {
				let depth = f64::from(frame.depths[row * frame.width + column]);

				if !(depth > 0.0 && depth.is_finite()) {
					continue;
				}

				let (across, up) = frame.tangents(column, row);
				let direction = frame.pose.forward + frame.pose.right * across + frame.pose.up * up;
				let stretch = direction.length();

				// A ray that climbs sees the ceiling, and nothing the grid
				// charts lies above the eye.
				if direction.y >= 0.0 {
					continue;
				}

				let ray = direction / stretch;
				let hit = depth * stretch;
				let hit_cell = fine_cell(ray.x * hit * scale, ray.z * hit * scale);
				let mut along = SELF_METRES;

				while along < hit - CARVE_MARGIN {
					let x = ray.x * along * scale;
					let y = ray.y * along;
					let z = ray.z * along * scale;
					let height = ((y + picture_floor) / picture_floor * NOMINAL_FLOOR).max(0.0);

					if let Some(cell) = fine_cell(x, z) {
						// The surface's own cell is not free space: the ray
						// ends in it.
						if Some(cell) == hit_cell {
							break;
						}

						if height < f64::from(self.clear[cell]) {
							self.clear[cell] = height as f32;
						}
					}

					along += CARVE_STEP;
				}
			}
		}
	}

	/// Finishes the room as the grid the rules move a husk through.
	///
	/// A builder that has read nothing still gives a well-formed grid: every
	/// cell unknown, the floor at the hunt's usual eye height.
	pub fn finish(self) -> RoomGrid {
		let room_floor = self.room_floor().unwrap_or(HuntRules::EYE_HEIGHT);
		let eye_centimetres = math::round(room_floor * 100.0).clamp(30, 300) as u16;
		let wall_centimetres = (room_floor - WALL_BELOW_EYE) * 100.0;

		// The fine grid's heights are shares of the nominal floor; the room's
		// own floor turns them into its metres.
		let scale = room_floor / NOMINAL_FLOOR;
		let (tops, walls) = tops_of(&self.counts, &self.clear, wall_centimetres, scale);
		let level = level_cells(&tops, &walls);
		let levels = levels_from(&tops, &level);
		let yaw = room_yaw(&self.yaw_weights);
		let kinds = kinds_of(&tops, &walls, &levels);
		let mut cells = downsample(&kinds, &levels, yaw);

		hide(&mut cells, &levels, room_floor);

		RoomGrid::new(
			eye_centimetres,
			0,
			yaw,
			GRID_SIDE as u8,
			GRID_SIDE as u8,
			PLAYER_CELL,
			PLAYER_CELL,
			levels,
			cells,
		)
	}
}

/// What one fine cell holds, before the grid is downsampled.
#[derive(Clone, Copy, Debug, PartialEq)]
enum FineKind {
	/// Nothing was seen there.
	Unknown,

	/// A level top, at an index into the grid's level table.
	Walkable(u8),

	/// Something standing up, at its top's height above the floor.
	Obstacle(f32),

	/// Something that reaches the eye, and so a wall.
	Wall,
}

/// The top of every fine cell, or NaN where it holds none, and whether that
/// top is a wall. `scale` turns the nominal heights into the room's metres.
fn tops_of(
	counts: &[u8],
	clear: &[f32],
	wall_centimetres: f64,
	scale: f64,
) -> (Vec<f32>, Vec<bool>) {
	let mut tops = vec![f32::NAN; FINE_CELLS];
	let mut walls = vec![false; FINE_CELLS];

	for cell in 0..FINE_CELLS {
		let lowest_clear = f64::from(clear[cell]) * scale;

		for bin in (0..HEIGHT_BINS).rev() {
			if counts[cell * HEIGHT_BINS + bin] < MIN_POINTS_OF_TOP {
				continue;
			}

			let height = (bin as f64 + 0.5) * HEIGHT_BIN * scale;

			// A ray crossed the cell beneath this height, so nothing was
			// there and the points are a silhouette the model smeared.
			if height > lowest_clear + CARVE_SLACK {
				continue;
			}

			tops[cell] = height as f32;
			walls[cell] = height * 100.0 >= wall_centimetres;
			break;
		}
	}

	(tops, walls)
}

/// Which fine cells lie on a surface level enough to stand on.
fn level_cells(tops: &[f32], walls: &[bool]) -> Vec<bool> {
	let mut level = vec![false; FINE_CELLS];

	for row in 1..FINE_SIDE - 1 {
		for column in 1..FINE_SIDE - 1 {
			let cell = row * FINE_SIDE + column;

			if walls[cell] || tops[cell].is_nan() {
				continue;
			}

			let Some((slope, residual)) = plane_fit(tops, walls, row, column) else {
				continue;
			};

			level[cell] = residual <= PLANE_RESIDUAL && slope <= PLANE_SLOPE;
		}
	}

	level
}

/// The slope and worst residual of the plane through the 3 by 3 patch of
/// tops around `row`, `column`, or `None` when any of the nine is missing or
/// a wall.
fn plane_fit(tops: &[f32], walls: &[bool], row: usize, column: usize) -> Option<(f64, f64)> {
	let mut sum = 0.0;

	for down in -1i64..=1 {
		for across in -1i64..=1 {
			let at = (row as i64 + down) as usize * FINE_SIDE + (column as i64 + across) as usize;

			if walls[at] || tops[at].is_nan() {
				return None;
			}

			sum += f64::from(tops[at]);
		}
	}

	let mean = sum / 9.0;
	let mut across_sum = 0.0;
	let mut down_sum = 0.0;
	let mut square_sum = 0.0;

	for down in -1i64..=1 {
		for across in -1i64..=1 {
			let at = (row as i64 + down) as usize * FINE_SIDE + (column as i64 + across) as usize;
			let across_metres = across as f64 * FINE_CELL;
			let down_metres = down as f64 * FINE_CELL;
			let rise = f64::from(tops[at]) - mean;

			across_sum += across_metres * rise;
			down_sum += down_metres * rise;
			square_sum += across_metres * across_metres;
		}
	}

	let slope_across = across_sum / square_sum;
	let slope_down = down_sum / square_sum;
	let mut residual: f64 = 0.0;

	for down in -1i64..=1 {
		for across in -1i64..=1 {
			let at = (row as i64 + down) as usize * FINE_SIDE + (column as i64 + across) as usize;
			let predicted = mean
				+ slope_across * (across as f64 * FINE_CELL)
				+ slope_down * (down as f64 * FINE_CELL);

			residual = residual.max((f64::from(tops[at]) - predicted).abs());
		}
	}

	Some((
		math::sqrt(slope_across * slope_across + slope_down * slope_down),
		residual,
	))
}

/// The few heights the room's level tops stand at, in centimetres above the
/// floor, beginning with the floor itself.
fn levels_from(tops: &[f32], level: &[bool]) -> Vec<u8> {
	let mut histogram = vec![0.0f64; LEVEL_BINS];

	for (cell, top) in tops.iter().enumerate() {
		if !level[cell] || top.is_nan() {
			continue;
		}

		let bin = math::round(f64::from(*top) / LEVEL_BIN).clamp(0, LEVEL_BINS as i64 - 1) as usize;

		histogram[bin] += FINE_CELL * FINE_CELL;
	}

	let smoothed = smooth_levels(&histogram);
	let mut peaks = peaks_of(&histogram, &smoothed);

	peaks.sort_by(|one, other| one.0.total_cmp(&other.0));

	let mut merged: Vec<(f64, f64)> = Vec::new();

	for (height, area) in peaks {
		match merged.last_mut() {
			Some((last_height, last_area)) if height - *last_height < LEVEL_MERGE => {
				let total = *last_area + area;

				*last_height = (*last_height * *last_area + height * area) / total;
				*last_area = total;
			}
			_ => merged.push((height, area)),
		}
	}

	// The floor is a level the format insists on, and it is zero.
	let mut levels: Vec<(f64, f64)> = merged
		.into_iter()
		.filter(|(height, _)| *height >= LEVEL_MERGE)
		.collect();

	levels.insert(0, (0.0, 0.0));

	// Whole centimetres are what the grid stores; two levels that round to
	// one height are one level.
	let mut table: Vec<(i64, f64)> = Vec::new();

	for (height, area) in levels {
		let centimetres = math::round(height * 100.0).clamp(0, 255);

		match table.last_mut() {
			Some((last, last_area)) if *last == centimetres => *last_area += area,
			_ => table.push((centimetres, area)),
		}
	}

	if table.len() > MAX_LEVELS {
		let mut rest = table.split_off(1);

		rest.sort_by(|one, other| other.1.total_cmp(&one.1).then(one.0.cmp(&other.0)));
		rest.truncate(MAX_LEVELS - 1);
		rest.sort_by_key(|(centimetres, _)| *centimetres);
		table.append(&mut rest);
	}

	table
		.into_iter()
		.map(|(centimetres, _)| centimetres as u8)
		.collect()
}

/// Smooth a level histogram with a Gaussian, clamped at either end.
fn smooth_levels(histogram: &[f64]) -> Vec<f64> {
	let sigma = LEVEL_BLUR / LEVEL_BIN;
	let radius = LEVEL_WINDOW;

	(0..histogram.len())
		.map(|bin| {
			(-radius..=radius)
				.map(|offset| {
					let at = bin as i64 + offset;

					if at < 0 || at >= histogram.len() as i64 {
						return 0.0;
					}

					let distance = offset as f64 / sigma;

					histogram[at as usize] * math::exp(-0.5 * distance * distance)
				})
				.sum()
		})
		.collect()
}

/// Every peak of the smoothed histogram that covers at least
/// [`LEVEL_PEAK_AREA`], as its mean height and the area of its cells.
fn peaks_of(histogram: &[f64], smoothed: &[f64]) -> Vec<(f64, f64)> {
	let mut peaks = Vec::new();

	for bin in 0..LEVEL_BINS {
		let left = if bin == 0 { 0.0 } else { smoothed[bin - 1] };
		let right = if bin + 1 == LEVEL_BINS {
			0.0
		} else {
			smoothed[bin + 1]
		};

		if smoothed[bin] < left || smoothed[bin] <= right {
			continue;
		}

		let low = bin.saturating_sub(LEVEL_WINDOW as usize);
		let high = (bin + LEVEL_WINDOW as usize).min(LEVEL_BINS - 1);
		let area: f64 = histogram[low..=high].iter().sum();

		if area < LEVEL_PEAK_AREA {
			continue;
		}

		let mut total = 0.0;
		let mut weighted = 0.0;

		for (at, count) in histogram.iter().enumerate().take(high + 1).skip(low) {
			total += count;
			weighted += count * ((at as f64 + 0.5) * LEVEL_BIN);
		}

		if total > 0.0 {
			peaks.push((weighted / total, area));
		}
	}

	peaks
}

/// What every fine cell is, once its top is held to the room's levels.
fn kinds_of(tops: &[f32], walls: &[bool], levels: &[u8]) -> Vec<FineKind> {
	let mut kinds = vec![FineKind::Unknown; FINE_CELLS];

	for cell in 0..FINE_CELLS {
		let top = tops[cell];

		if top.is_nan() {
			continue;
		}

		if walls[cell] {
			kinds[cell] = FineKind::Wall;
			continue;
		}

		let height = f64::from(top);
		let (x, z) = fine_centre(cell);
		let tolerance = LEVEL_SNAP + LEVEL_SNAP_PER_METRE * math::sqrt(x * x + z * z);
		let mut nearest: Option<(u8, f64)> = None;

		for (index, &centimetres) in levels.iter().enumerate() {
			let distance = (height - f64::from(centimetres) / 100.0).abs();

			if nearest.is_none_or(|(_, best)| distance < best) {
				nearest = Some((index as u8, distance));
			}
		}

		kinds[cell] = match nearest {
			Some((level, distance)) if distance <= tolerance => FineKind::Walkable(level),
			_ => FineKind::Obstacle(top),
		};
	}

	kinds
}

/// The room's own turn from the hunt's axes, in the grid's 1/256 steps, or
/// zero where the pictures hold no vertical structure worth turning to.
fn room_yaw(weights: &[f64; YAW_BINS]) -> u8 {
	let total: f64 = weights.iter().sum();

	if total < YAW_MIN_WEIGHT {
		return 0;
	}

	let smoothed: Vec<f64> = (0..YAW_BINS)
		.map(|bin| {
			(-YAW_SMOOTH_DEGREES..=YAW_SMOOTH_DEGREES)
				.map(|offset| {
					let at = (bin as i64 + offset).rem_euclid(YAW_BINS as i64) as usize;

					weights[at] * (3 - offset.abs()) as f64
				})
				.sum()
		})
		.collect();
	let peak = (0..YAW_BINS)
		.max_by(|one, other| smoothed[*one].total_cmp(&smoothed[*other]))
		.unwrap_or(0);

	// Fold the spread to one quarter turn, where every wall is one axis.
	let mut cosine = 0.0;
	let mut sine = 0.0;

	for offset in -YAW_PEAK_DEGREES..=YAW_PEAK_DEGREES {
		let at = (peak as i64 + offset).rem_euclid(YAW_BINS as i64) as usize;
		let angle = (at as f64 + 0.5) * (PI / 2.0) / YAW_BINS as f64;

		cosine += weights[at] * math::cos(4.0 * angle);
		sine += weights[at] * math::sin(4.0 * angle);
	}

	let angle = (math::atan2(sine, cosine) / 4.0).rem_euclid(PI / 2.0);
	let steps = math::round(angle * YAW_STEPS as f64 / (2.0 * PI));

	steps.rem_euclid(YAW_STEPS) as u8
}

/// The coarse grid's votes, one set per cell.
struct CoarseVotes {
	/// Walkable votes, by cell and then by level index.
	walk: Vec<u16>,
	unknown: Vec<u16>,
	obstacle: Vec<u16>,
	wall: Vec<u16>,
	/// The highest obstacle top voted for each cell.
	top: Vec<f32>,
}

impl CoarseVotes {
	fn new() -> Self {
		let cells = GRID_SIDE * GRID_SIDE;

		Self {
			walk: vec![0; cells * MAX_LEVELS],
			unknown: vec![0; cells],
			obstacle: vec![0; cells],
			wall: vec![0; cells],
			top: vec![0.0; cells],
		}
	}

	/// The majority vote in every cell: wall, then unknown, then the best
	/// level, then an obstacle, then unknown again.
	fn choose(&self, levels: &[u8]) -> Vec<u8> {
		let mut cells = vec![RoomCell::UNKNOWN.byte(); GRID_SIDE * GRID_SIDE];

		for cell in 0..cells.len() {
			let (best_level, walk) = (0..levels.len()).fold((0usize, 0u16), |best, level| {
				let votes = self.walk[cell * MAX_LEVELS + level];

				if votes >= best.1 {
					(level, votes)
				} else {
					best
				}
			});
			let unknown = self.unknown[cell];
			let obstacle = self.obstacle[cell];
			let wall = self.wall[cell];
			let known = walk + obstacle + wall;

			if wall > 0 && wall >= walk && wall >= obstacle && wall >= unknown {
				cells[cell] = RoomCell::WALL.byte();
			} else if unknown > known {
				// Nothing was seen in most of it, so nothing is known.
			} else if walk > 0 && walk >= obstacle {
				cells[cell] = RoomCell::walkable(best_level as u8).byte();
			} else if obstacle > 0 {
				let centimetres = math::round(f64::from(self.top[cell]) * 100.0)
					.clamp(1, RoomCell::MAX_OBSTACLE_CENTIMETRES);

				cells[cell] = RoomCell::obstacle(centimetres).byte();
			}
		}

		cells
	}
}

/// Votes every fine cell into the 10 cm grid, turned by `yaw`.
fn downsample(kinds: &[FineKind], levels: &[u8], yaw: u8) -> Vec<u8> {
	let mut votes = CoarseVotes::new();
	let angle = f64::from(yaw) * 2.0 * PI / 256.0;
	let sine = math::sin(angle);
	let cosine = math::cos(angle);

	for (cell, kind) in kinds.iter().enumerate() {
		let (x, z) = fine_centre(cell);
		let grid_x = x * cosine - z * sine;
		let grid_z = x * sine + z * cosine;
		let column = math::floor((grid_x + EXTENT / 2.0) / GRID_CELL);
		let row = math::floor((grid_z + EXTENT / 2.0) / GRID_CELL);

		if column < 0 || row < 0 || column >= GRID_SIDE as i64 || row >= GRID_SIDE as i64 {
			continue;
		}

		let coarse = row as usize * GRID_SIDE + column as usize;

		match *kind {
			FineKind::Unknown => votes.unknown[coarse] += 1,
			FineKind::Walkable(level) => votes.walk[coarse * MAX_LEVELS + level as usize] += 1,
			FineKind::Obstacle(top) => {
				votes.obstacle[coarse] += 1;
				votes.top[coarse] = votes.top[coarse].max(top);
			}
			FineKind::Wall => votes.wall[coarse] += 1,
		}
	}

	votes.choose(levels)
}

/// Marks the floor no picture saw, directly behind something standing up,
/// as hidden rather than unknown.
///
/// The player stands at the grid's middle. Each ray out from them records
/// how far a top above the floor hides what is behind it: a blocker `b` out
/// and `height` up hides the floor from `b` to `b / (1 - height / eye)`,
/// where the line of sight from the eye over that blocker meets the floor.
/// An unknown cell inside that reach is floor the player cannot see.
fn hide(cells: &mut [u8], levels: &[u8], eye_height: f64) {
	let hidden = RoomCell::hidden(0).byte();

	for ray in 0..HIDDEN_RAYS {
		let angle = ray as f64 * 2.0 * PI / HIDDEN_RAYS as f64;
		let sine = math::sin(angle);
		let cosine = math::cos(angle);
		let mut shadow_end = 0.0;
		let mut last = usize::MAX;
		let mut along = HIDDEN_STEP;

		while along <= EXTENT / 2.0 {
			let column = math::floor((cosine * along + EXTENT / 2.0) / GRID_CELL);
			let row = math::floor((sine * along + EXTENT / 2.0) / GRID_CELL);

			if column < 0 || row < 0 || column >= GRID_SIDE as i64 || row >= GRID_SIDE as i64 {
				break;
			}

			let cell = row as usize * GRID_SIDE + column as usize;

			if cell != last {
				last = cell;

				let Some(value) = RoomCell::from_byte(cells[cell]) else {
					along += HIDDEN_STEP;
					continue;
				};

				match value.kind {
					RoomCellKind::Wall => break,
					RoomCellKind::Unknown => {
						if along <= shadow_end {
							cells[cell] = hidden;
						}
					}
					RoomCellKind::Walkable | RoomCellKind::Hidden | RoomCellKind::Obstacle => {
						let centimetres: i64 = match value.kind {
							RoomCellKind::Obstacle => value.top_centimetres(),
							_ => i64::from(levels[value.payload as usize]),
						};
						let top = centimetres as f64 / 100.0;

						if top > 0.0 && top < eye_height {
							shadow_end = shadow_end
								.max((along + GRID_CELL / 2.0) / (1.0 - top / eye_height));
						}
					}
				}
			}

			along += HIDDEN_STEP;
		}
	}
}

/// Which pixels are not flying: a pixel whose depth differs from a
/// 4-neighbour by more than [`FLYING_PIXEL_SHARE`] is across a silhouette.
fn flying_pixels(frame: &DepthFrame) -> Vec<bool> {
	let mut keep = vec![false; frame.width * frame.height];

	for row in 0..frame.height {
		for column in 0..frame.width {
			let at = row * frame.width + column;
			let depth = frame.depths[at];

			if !(depth > 0.0 && depth.is_finite()) {
				continue;
			}

			let mut flying = false;

			for (across, down) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
				let other_column = column as i64 + across;
				let other_row = row as i64 + down;

				if other_column < 0
					|| other_row < 0
					|| other_column >= frame.width as i64
					|| other_row >= frame.height as i64
				{
					continue;
				}

				let other = frame.depths[other_row as usize * frame.width + other_column as usize];

				if other > 0.0
					&& other.is_finite()
					&& relative_step(depth, other) > FLYING_PIXEL_SHARE
				{
					flying = true;
					break;
				}
			}

			keep[at] = !flying;
		}
	}

	keep
}

/// How far apart two depths are, as a share of the nearer one.
fn relative_step(one: f32, other: f32) -> f64 {
	let one = f64::from(one);
	let other = f64::from(other);

	(one - other).abs() / one.min(other)
}

/// The fine cell a point falls in, or `None` outside the grid.
fn fine_cell(x: f64, z: f64) -> Option<usize> {
	let column = math::floor((x + EXTENT / 2.0 - FINE_OFFSET) / FINE_CELL);
	let row = math::floor((z + EXTENT / 2.0 - FINE_OFFSET) / FINE_CELL);

	if column < 0 || row < 0 || column >= FINE_SIDE as i64 || row >= FINE_SIDE as i64 {
		return None;
	}

	Some(row as usize * FINE_SIDE + column as usize)
}

/// The middle of a fine cell, in the hunt's frame.
fn fine_centre(cell: usize) -> (f64, f64) {
	let column = cell % FINE_SIDE;
	let row = cell / FINE_SIDE;

	(
		-EXTENT / 2.0 + FINE_OFFSET + (column as f64 + 0.5) * FINE_CELL,
		-EXTENT / 2.0 + FINE_OFFSET + (row as f64 + 0.5) * FINE_CELL,
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::pose::ArPose;

	/// A one-row picture with the depths given.
	fn row(depths: &[f32]) -> DepthFrame {
		DepthFrame {
			width: depths.len(),
			height: 1,
			depths: depths.to_vec(),
			pose: ArPose::AHEAD,
			tan_half_width: 0.4,
			tan_half_height: 0.7,
		}
	}

	#[test]
	fn a_silhouette_is_flying_and_a_smooth_plane_is_not() {
		let step = flying_pixels(&row(&[1.0, 1.0, 1.2]));
		let plane = flying_pixels(&row(&[1.0, 1.02, 1.04]));

		assert_eq!(step, vec![true, false, false]);
		assert_eq!(plane, vec![true, true, true]);
	}

	#[test]
	fn near_levels_merge_and_the_floor_stays_zero() {
		let mut tops = vec![f32::NAN; FINE_CELLS];
		let mut level = vec![false; FINE_CELLS];
		let mut patch =
			|rows: core::ops::Range<usize>, columns: core::ops::Range<usize>, height| {
				for row in rows.clone() {
					for column in columns.clone() {
						let cell = row * FINE_SIDE + column;

						tops[cell] = height;
						level[cell] = true;
					}
				}
			};

		patch(10..30, 10..30, 0.02);
		patch(40..60, 10..20, 0.74);
		patch(70..90, 10..20, 0.44);
		patch(100..110, 10..20, 0.46);

		let levels = levels_from(&tops, &level);

		assert_eq!(levels[0], 0, "the floor is the first level");
		assert_eq!(levels.len(), 3, "the two chair heights are one level");
		assert!(
			levels
				.iter()
				.any(|&centimetres| (f64::from(centimetres) / 100.0 - 0.44).abs() <= 0.03)
		);
		assert!(
			levels
				.iter()
				.any(|&centimetres| (f64::from(centimetres) / 100.0 - 0.74).abs() <= 0.03)
		);
	}

	#[test]
	fn a_plane_is_level_and_a_step_is_not() {
		let tops = vec![0.02_f32; FINE_CELLS];
		let walls = vec![false; FINE_CELLS];
		let mut stepped = tops.clone();

		stepped[100 * FINE_SIDE + 100] = 0.5;

		let flat = level_cells(&tops, &walls);
		let step = level_cells(&stepped, &walls);

		assert!(flat[100 * FINE_SIDE + 100]);
		assert!(!step[100 * FINE_SIDE + 100], "a step is not a level top");
		assert!(step[50 * FINE_SIDE + 50], "the floor around it still is");
	}
}
