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
//!    Add where the picture was taken, so pictures taken from different
//!    places chart one room.
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
use reapers_rules::Vec3;

use crate::depth_frame::DepthFrame;
use crate::math;
use crate::math::PI;
use crate::room_map::Cloud;
use crate::room_map::FloorTracker;
use crate::room_map::SELF_METRES;
use crate::room_map::STRIDE;
use crate::room_map::level_points;

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

/// Precomputed Gaussian weights for `sigma = LEVEL_BLUR / LEVEL_BIN = 2.0`
/// over offsets `0..=LEVEL_WINDOW` (5 cm either side).
const LEVEL_GAUSS: [f64; 6] = [
	1.0,
	0.882_496_902_584_595_5,
	0.606_530_659_712_633_4,
	0.324_652_467_358_349_74,
	0.135_335_283_236_612_7,
	0.043_936_933_623_407_42,
];

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
const MAX_LEVELS: usize = RoomCell::MAX_LEVELS;

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

/// Where one picture was taken, and how its metres are brought into the
/// hunt's frame. See [`RoomGridBuilder::placement`].
#[derive(Clone, Copy, Debug)]
struct PictureAt {
	/// Turns the picture's own metres into the room's, along the ground.
	ground: f64,

	/// How far the floor is below the picture's own eye, in its own metres.
	picture_floor: f64,

	/// Brings a height above the floor, as a share of the eye that took the
	/// picture, to a share of the hunt's own eye.
	heights: f64,

	/// Where the picture's eye was, in the hunt's frame.
	origin: Vec3,
}

/// A grid read from depth pictures, before it is finished.
///
/// Feed it every picture the survey takes with
/// [`add_frame`](Self::add_frame), then call [`finish`](Self::finish). It
/// uses the same [`FloorTracker`] rules as [`RoomMap`](crate::RoomMap) for the
/// floor and per-picture scale, so what it reads agrees with the plan a hunt
/// plays inside without allocating a duplicate 2 MiB slice table.
#[derive(Clone, Debug)]
pub struct RoomGridBuilder {
	/// The floor and the picture scale, shared with [`RoomMap`](crate::RoomMap).
	floor: FloorTracker,

	/// The count of points in each height bin of each fine cell.
	counts: Vec<u8>,

	/// Highest touched height bin + 1 per fine cell (0 when empty), so
	/// `tops_of` skips empty cells in O(1) without scanning 63 bins.
	max_bin: Vec<u8>,

	/// The lowest height above the floor at which a carving ray crossed each
	/// fine cell, or infinity where none did.
	clear: Vec<f32>,

	/// How much vertical structure faces each degree of one quarter turn.
	yaw_weights: [f64; YAW_BINS],
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
			floor: FloorTracker::new(),
			counts: vec![0; FINE_HEIGHT_COUNTS],
			max_bin: vec![0; FINE_CELLS],
			clear: vec![f32::INFINITY; FINE_CELLS],
			yaw_weights: [0.0; YAW_BINS],
		}
	}

	/// Takes the floor as measured by the platform: see
	/// [`RoomMap::measure_floor`](crate::RoomMap::measure_floor). Every picture
	/// is then charted against it, in real metres, instead of against its own
	/// floor.
	pub fn measure_floor(&mut self, eye_height: f64) -> bool {
		self.floor.measure_floor(eye_height)
	}

	/// Pictures charted so far.
	pub fn frames(&self) -> usize {
		self.floor.frames()
	}

	/// How far the floor is below the hunt's origin, in metres, as the
	/// pictures so far have it: the same estimate the hunt's plan uses.
	pub fn floor_depth(&self) -> Option<f64> {
		self.floor.floor_depth()
	}

	/// Takes in one picture. Answers whether it was charted: a picture with
	/// no floor of its own, or one that disagrees with the floor the others
	/// found, teaches the grid nothing.
	///
	/// The picture's own eye is taken into account: a picture taken from
	/// elsewhere is charted where it was taken, in the room's own frame.
	pub fn add_frame(&mut self, frame: &DepthFrame) -> bool {
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;
		let cloud = Cloud::of(frame);
		let level = level_points(&cloud, columns, rows);
		self.add_prepared(frame, &cloud, &level, false)
	}

	/// Takes estimated depth, finding its own floor before calibrating it
	/// against a platform measurement, as the room survey does.
	pub fn add_estimated(&mut self, frame: &DepthFrame) -> bool {
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;
		let cloud = Cloud::of(frame);
		let level = level_points(&cloud, columns, rows);
		self.add_prepared(frame, &cloud, &level, true)
	}

	pub(crate) fn add_prepared(
		&mut self,
		frame: &DepthFrame,
		cloud: &Cloud,
		level: &[bool],
		estimated: bool,
	) -> bool {
		if self
			.floor
			.evaluate(
				cloud,
				level,
				frame.origin,
				reapers_rules::Gait::Ground,
				estimated,
			)
			.is_none()
		{
			return false;
		}
		self.chart_frame_with_cloud(frame, cloud)
	}

	fn chart_frame_with_cloud(&mut self, frame: &DepthFrame, cloud: &Cloud) -> bool {
		let Some(picture_floor) = self.floor.last_picture_floor() else {
			return false;
		};
		let origin = frame.origin;
		let at = self.placement(picture_floor, origin);
		let columns = frame.width / STRIDE;
		let rows = frame.height / STRIDE;

		self.chart(frame, cloud, columns, rows, at);
		self.carve(frame, at);

		true
	}

	/// Where one picture was taken and how its metres are brought into the
	/// hunt's frame: the floor is `picture_floor` below its own eye, in its
	/// own metres, and the eye itself was at `origin`.
	fn placement(&self, picture_floor: f64, origin: Vec3) -> PictureAt {
		let floor = self.room_floor();

		PictureAt {
			ground: floor.map_or(1.0, |floor| (floor + origin.y) / picture_floor),
			picture_floor,
			heights: floor.map_or(1.0, |floor| (floor + origin.y) / floor),
			origin,
		}
	}

	/// How far below the hunt's origin the room's floor is, in metres.
	fn room_floor(&self) -> Option<f64> {
		self.floor.floor_depth()
	}

	/// Charts one picture's non-flying points and vertical structure directly
	/// from the shared `Cloud`, checking 4-neighbour flying pixels only at
	/// strided coordinates and computing vertical normals inline with zero
	/// intermediate allocations.
	fn chart(
		&mut self,
		frame: &DepthFrame,
		cloud: &Cloud,
		columns: usize,
		rows: usize,
		at: PictureAt,
	) {
		if !at.ground.is_finite()
			|| at.ground <= 0.0
			|| !at.picture_floor.is_finite()
			|| at.picture_floor <= 0.0
			|| !at.heights.is_finite()
			|| at.heights <= 0.0
		{
			return;
		}
		let count = columns * rows;
		if count == 0 {
			return;
		}

		let (scale, origin) = (at.ground, at.origin);
		let self_sq = SELF_METRES * SELF_METRES;
		let height_scale = NOMINAL_FLOOR * at.heights / at.picture_floor;
		let max_height = HEIGHT_BIN * HEIGHT_BINS as f64;
		let inv_height_bin = 1.0 / HEIGHT_BIN;
		let horiz_share_sq = YAW_HORIZONTAL_SHARE * YAW_HORIZONTAL_SHARE;
		let yaw_scale = YAW_BINS as f64 / (PI / 2.0);

		// Compute which strided pixels pass the flying-pixel filter once so
		// neighbour normals reject flying pixels identically to `Cloud::of_filtered`.
		let mut keep = vec![false; count];
		for row in 0..rows {
			let sr = row * STRIDE;
			let base = row * columns;
			for column in 0..columns {
				let sc = column * STRIDE;
				keep[base + column] = is_not_flying(frame, sc, sr);
			}
		}

		let xs = cloud.xs.as_slice();
		let ys = cloud.ys.as_slice();
		let zs = cloud.zs.as_slice();
		let has_normals = columns >= 2 && rows >= 2;

		for row in 0..rows {
			let row_base = row * columns;
			let down_offset: isize = if row + 1 < rows {
				columns as isize
			} else {
				-(columns as isize)
			};

			for column in 0..columns {
				let index = row_base + column;
				if !keep[index] {
					continue;
				}

				let cx = f64::from(xs[index]);
				let cy = f64::from(ys[index]);
				let cz = f64::from(zs[index]);

				if has_normals {
					let right = if column + 1 < columns {
						index + 1
					} else {
						index - 1
					};
					let down = (index as isize + down_offset) as usize;
					if keep[right] && keep[down] {
						let ax = f64::from(xs[right]) - cx;
						let ay = f64::from(ys[right]) - cy;
						let az = f64::from(zs[right]) - cz;
						let bx = f64::from(xs[down]) - cx;
						let by = f64::from(ys[down]) - cy;
						let bz = f64::from(zs[down]) - cz;
						let nx = ay * bz - az * by;
						let ny = az * bx - ax * bz;
						let nz = ax * by - ay * bx;
						let horiz_sq = nx * nx + nz * nz;
						let len_sq = horiz_sq + ny * ny;

						if len_sq > 0.0 && horiz_sq >= horiz_share_sq * len_sq {
							let horizontal = math::sqrt(horiz_sq / len_sq);
							let folded = math::fast_atan2(nx, nz).rem_euclid(PI / 2.0);
							let bin = (folded * yaw_scale) as usize;
							self.yaw_weights[bin.min(YAW_BINS - 1)] += horizontal;
						}
					}
				}

				if cy > 0.0 || !cy.is_finite() {
					continue;
				}
				let x = cx * scale + origin.x;
				let z = cz * scale + origin.z;
				let away_sq = x * x + z * z;
				if !(away_sq >= self_sq && away_sq.is_finite()) {
					continue;
				}

				let height = (cy + at.picture_floor) * height_scale;
				if !height.is_finite() || height < -FLOOR_SLACK || height >= max_height {
					continue;
				}

				let Some(cell) = fine_cell(x, z) else {
					continue;
				};
				let bin = ((height.max(0.0) * inv_height_bin) as usize).min(HEIGHT_BINS - 1);
				let slot = cell * HEIGHT_BINS + bin;
				self.counts[slot] = self.counts[slot].saturating_add(1);
				let bin_tag = (bin + 1) as u8;
				if bin_tag > self.max_bin[cell] {
					self.max_bin[cell] = bin_tag;
				}
			}
		}
	}

	/// Records, for every fine cell a ray crossed before it hit something,
	/// the lowest height above the floor at which it crossed. A point well
	/// above that flew, since the ray passed beneath it.
	fn carve(&mut self, frame: &DepthFrame, at: PictureAt) {
		if !at.ground.is_finite()
			|| at.ground <= 0.0
			|| !at.picture_floor.is_finite()
			|| at.picture_floor <= 0.0
			|| !at.heights.is_finite()
			|| at.heights <= 0.0
			|| frame.width == 0
			|| frame.height == 0
		{
			return;
		}

		let pose = frame.pose;
		let inv_w = 2.0 / frame.width as f64;
		let inv_h = 2.0 / frame.height as f64;
		let across_step = (CARVE_STRIDE as f64) * inv_w * frame.tan_half_width;
		let across_start = (0.5 * inv_w - 1.0) * frame.tan_half_width;
		let step_dx = pose.right.x * across_step;
		let step_dy = pose.right.y * across_step;
		let step_dz = pose.right.z * across_step;
		let height_base = NOMINAL_FLOOR * at.heights;
		let height_y_factor = height_base / (at.ground * at.picture_floor);
		let clear = self.clear.as_mut_slice();

		for row in (0..frame.height).step_by(CARVE_STRIDE) {
			let up = (1.0 - (row as f64 + 0.5) * inv_h) * frame.tan_half_height;
			let mut dir_x = pose.forward.x + pose.up.x * up + pose.right.x * across_start;
			let mut dir_y = pose.forward.y + pose.up.y * up + pose.right.y * across_start;
			let mut dir_z = pose.forward.z + pose.up.z * up + pose.right.z * across_start;
			let row_base = row * frame.width;

			for column in (0..frame.width).step_by(CARVE_STRIDE) {
				let depth = f64::from(frame.depths[row_base + column]);

				if depth > 0.0 && depth.is_finite() && dir_y < 0.0 {
					let stretch = math::length(dir_x, dir_y, dir_z);
					let inv_stretch = 1.0 / stretch;
					let ray_x = dir_x * inv_stretch;
					let ray_y = dir_y * inv_stretch;
					let ray_z = dir_z * inv_stretch;
					let hit_world = depth * stretch * at.ground;
					let end_world = hit_world - CARVE_MARGIN;

					if SELF_METRES < end_world {
						const OFFSET: f64 = EXTENT / 2.0 - FINE_OFFSET;
						const INV_CELL: f64 = 1.0 / FINE_CELL;
						const FINE_SIDE_F: f64 = FINE_SIDE as f64;
						let hit_cell = fine_cell(
							ray_x * hit_world + at.origin.x,
							ray_z * hit_world + at.origin.z,
						);
						let mut along_world = SELF_METRES;
						let mut gx = (ray_x * along_world + at.origin.x + OFFSET) * INV_CELL;
						let mut gz = (ray_z * along_world + at.origin.z + OFFSET) * INV_CELL;
						let mut raw_height = height_base + ray_y * along_world * height_y_factor;
						let dgx = ray_x * CARVE_STEP * INV_CELL;
						let dgz = ray_z * CARVE_STEP * INV_CELL;
						let d_height = ray_y * CARVE_STEP * height_y_factor;

						while along_world < end_world {
							if !(0.0..FINE_SIDE_F).contains(&gx)
								|| !(0.0..FINE_SIDE_F).contains(&gz)
							{
								if gx < -1.0
									|| gx > FINE_SIDE_F + 1.0
									|| gz < -1.0
									|| gz > FINE_SIDE_F + 1.0
								{
									break;
								}
							} else {
								let cell = (gz as usize) * FINE_SIDE + (gx as usize);
								if Some(cell) == hit_cell {
									break;
								}
								let height = raw_height.max(0.0) as f32;
								if height < clear[cell] {
									clear[cell] = height;
								}
							}

							along_world += CARVE_STEP;
							gx += dgx;
							gz += dgz;
							raw_height += d_height;
						}
					}
				}

				dir_x += step_dx;
				dir_y += step_dy;
				dir_z += step_dz;
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
		let (tops, walls) = tops_of(
			&self.counts,
			&self.max_bin,
			&self.clear,
			wall_centimetres,
			scale,
		);
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
	max_bin: &[u8],
	clear: &[f32],
	wall_centimetres: f64,
	scale: f64,
) -> (Vec<f32>, Vec<bool>) {
	let mut tops = vec![f32::NAN; FINE_CELLS];
	let mut walls = vec![false; FINE_CELLS];
	let bin_step = HEIGHT_BIN * scale;
	let wall_metres = wall_centimetres / 100.0;

	for cell in 0..FINE_CELLS {
		let top_tag = usize::from(max_bin[cell]);
		if top_tag == 0 {
			continue;
		}
		let clear_limit = f64::from(clear[cell]) * scale + CARVE_SLACK;
		let base = cell * HEIGHT_BINS;

		for bin in (0..top_tag).rev() {
			if counts[base + bin] < MIN_POINTS_OF_TOP {
				continue;
			}

			let height = (bin as f64 + 0.5) * bin_step;

			// A ray crossed the cell beneath this height, so nothing was
			// there and the points are a silhouette the model smeared.
			if height > clear_limit {
				continue;
			}

			tops[cell] = height as f32;
			walls[cell] = height >= wall_metres;
			break;
		}
	}

	(tops, walls)
}

/// Which fine cells lie on a surface level enough to stand on.
fn level_cells(tops: &[f32], walls: &[bool]) -> Vec<bool> {
	let mut level = vec![false; FINE_CELLS];

	for row in 1..FINE_SIDE - 1 {
		let row_base = row * FINE_SIDE;
		for column in 1..FINE_SIDE - 1 {
			let cell = row_base + column;

			if walls[cell] || tops[cell].is_nan() {
				continue;
			}

			level[cell] = is_level_patch(tops, walls, row_base + column);
		}
	}

	level
}

/// Whether the 3 by 3 patch of tops centred on `center` is complete, free of
/// walls, no steeper than `PLANE_SLOPE`, and flat within `PLANE_RESIDUAL`.
#[inline]
fn is_level_patch(tops: &[f32], walls: &[bool], center: usize) -> bool {
	const INV_SQUARE_SUM: f64 = 1.0 / (6.0 * FINE_CELL * FINE_CELL);
	const SLOPE_SQ_MAX: f64 = PLANE_SLOPE * PLANE_SLOPE;

	let r0 = center - FINE_SIDE;
	let r1 = center;
	let r2 = center + FINE_SIDE;

	if walls[r0 - 1]
		|| walls[r0]
		|| walls[r0 + 1]
		|| walls[r1 - 1]
		|| walls[r1 + 1]
		|| walls[r2 - 1]
		|| walls[r2]
		|| walls[r2 + 1]
	{
		return false;
	}

	let v00 = f64::from(tops[r0 - 1]);
	let v01 = f64::from(tops[r0]);
	let v02 = f64::from(tops[r0 + 1]);
	let v10 = f64::from(tops[r1 - 1]);
	let v11 = f64::from(tops[r1]);
	let v12 = f64::from(tops[r1 + 1]);
	let v20 = f64::from(tops[r2 - 1]);
	let v21 = f64::from(tops[r2]);
	let v22 = f64::from(tops[r2 + 1]);

	let sum = v00 + v01 + v02 + v10 + v11 + v12 + v20 + v21 + v22;
	if !sum.is_finite() {
		return false;
	}

	// Because sum(across) == 0 and sum(down) == 0 on a symmetric 3×3 grid,
	// `sum(across * (v - mean)) == FINE_CELL * ((v02+v12+v22) - (v00+v10+v20))`.
	let dx = ((v02 + v12 + v22) - (v00 + v10 + v20)) * FINE_CELL;
	let dz = ((v20 + v21 + v22) - (v00 + v01 + v02)) * FINE_CELL;
	let slope_across = dx * INV_SQUARE_SUM;
	let slope_down = dz * INV_SQUARE_SUM;

	if slope_across * slope_across + slope_down * slope_down > SLOPE_SQ_MAX {
		return false;
	}

	let mean = sum / 9.0;
	let sa = slope_across * FINE_CELL;
	let sd = slope_down * FINE_CELL;

	(v00 - (mean - sa - sd)).abs() <= PLANE_RESIDUAL
		&& (v01 - (mean - sd)).abs() <= PLANE_RESIDUAL
		&& (v02 - (mean + sa - sd)).abs() <= PLANE_RESIDUAL
		&& (v10 - (mean - sa)).abs() <= PLANE_RESIDUAL
		&& (v11 - mean).abs() <= PLANE_RESIDUAL
		&& (v12 - (mean + sa)).abs() <= PLANE_RESIDUAL
		&& (v20 - (mean - sa + sd)).abs() <= PLANE_RESIDUAL
		&& (v21 - (mean + sd)).abs() <= PLANE_RESIDUAL
		&& (v22 - (mean + sa + sd)).abs() <= PLANE_RESIDUAL
}

/// The few heights the room's level tops stand at, in centimetres above the
/// floor, beginning with the floor itself.
fn levels_from(tops: &[f32], level: &[bool]) -> Vec<u8> {
	let mut histogram = [0.0_f64; LEVEL_BINS];
	let cell_area = FINE_CELL * FINE_CELL;
	let inv_bin = 1.0 / LEVEL_BIN;

	for (&top, &is_level) in tops.iter().zip(level) {
		if !is_level || top.is_nan() {
			continue;
		}

		let bin = math::round(f64::from(top) * inv_bin).clamp(0, LEVEL_BINS as i64 - 1) as usize;
		histogram[bin] += cell_area;
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
fn smooth_levels(histogram: &[f64; LEVEL_BINS]) -> [f64; LEVEL_BINS] {
	let mut out = [0.0_f64; LEVEL_BINS];
	let radius = LEVEL_WINDOW as usize;

	for (bin, slot) in out.iter_mut().enumerate() {
		let low = bin.saturating_sub(radius);
		let high = (bin + radius).min(LEVEL_BINS - 1);
		let mut sum = 0.0;
		for (at, &count) in histogram.iter().enumerate().take(high + 1).skip(low) {
			sum += count * LEVEL_GAUSS[bin.abs_diff(at)];
		}
		*slot = sum;
	}

	out
}

/// Every peak of the smoothed histogram that covers at least
/// [`LEVEL_PEAK_AREA`], as its mean height and the area of its cells.
fn peaks_of(histogram: &[f64; LEVEL_BINS], smoothed: &[f64; LEVEL_BINS]) -> Vec<(f64, f64)> {
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

		for (at, &count) in histogram.iter().enumerate().take(high + 1).skip(low) {
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
	let mut level_metres = [0.0_f64; MAX_LEVELS];
	let level_count = levels.len().min(MAX_LEVELS);
	for (dst, &cm) in level_metres.iter_mut().zip(levels.iter()).take(level_count) {
		*dst = f64::from(cm) / 100.0;
	}
	let level_slice = &level_metres[..level_count];
	let origin_offset = -EXTENT / 2.0 + FINE_OFFSET + 0.5 * FINE_CELL;

	for row in 0..FINE_SIDE {
		let z = origin_offset + row as f64 * FINE_CELL;
		let z_sq = z * z;
		let row_base = row * FINE_SIDE;

		for column in 0..FINE_SIDE {
			let cell = row_base + column;
			let top = tops[cell];
			if top.is_nan() {
				continue;
			}
			if walls[cell] {
				kinds[cell] = FineKind::Wall;
				continue;
			}

			let height = f64::from(top);
			let x = origin_offset + column as f64 * FINE_CELL;
			let tolerance = LEVEL_SNAP + LEVEL_SNAP_PER_METRE * math::sqrt(x * x + z_sq);
			let mut best_level = 0_u8;
			let mut best_dist = f64::INFINITY;

			for (index, &lvl_m) in level_slice.iter().enumerate() {
				let distance = (height - lvl_m).abs();
				if distance < best_dist {
					best_dist = distance;
					best_level = index as u8;
				}
			}

			kinds[cell] = if best_dist <= tolerance {
				FineKind::Walkable(best_level)
			} else {
				FineKind::Obstacle(top)
			};
		}
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

	let mut smoothed = [0.0_f64; YAW_BINS];
	let mut peak = 0_usize;
	let mut peak_val = f64::NEG_INFINITY;

	for (bin, slot) in smoothed.iter_mut().enumerate() {
		let mut sum = 0.0;
		for offset in -YAW_SMOOTH_DEGREES..=YAW_SMOOTH_DEGREES {
			let at = (bin as i64 + offset).rem_euclid(YAW_BINS as i64) as usize;
			sum += weights[at] * (3 - offset.abs()) as f64;
		}
		*slot = sum;
		if sum > peak_val {
			peak_val = sum;
			peak = bin;
		}
	}

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
	level_count: usize,
	unknown: Vec<u16>,
	obstacle: Vec<u16>,
	wall: Vec<u16>,
	/// The highest obstacle top voted for each cell.
	top: Vec<f32>,
}

impl CoarseVotes {
	fn new(level_count: usize) -> Self {
		let cells = GRID_SIDE * GRID_SIDE;
		let level_count = level_count.max(1);

		Self {
			walk: vec![0; cells * level_count],
			level_count,
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

		for (cell, out) in cells.iter_mut().enumerate() {
			let base = cell * self.level_count;
			let mut best_level = 0usize;
			let mut best_walk = 0u16;
			let mut total_walk = 0u16;

			for level in 0..levels.len() {
				let votes = self.walk[base + level];
				total_walk = total_walk.saturating_add(votes);
				if votes >= best_walk {
					best_level = level;
					best_walk = votes;
				}
			}

			let unknown = self.unknown[cell];
			let obstacle = self.obstacle[cell];
			let wall = self.wall[cell];
			let known = total_walk.saturating_add(obstacle).saturating_add(wall);

			if wall > 0 && wall >= total_walk && wall >= obstacle && wall >= unknown {
				*out = RoomCell::WALL.byte();
			} else if unknown > known {
				// Nothing was seen in most of it, so nothing is known.
			} else if total_walk > 0 && total_walk >= obstacle {
				*out = RoomCell::walkable(best_level as u8).byte();
			} else if obstacle > 0 {
				let centimetres = math::round(f64::from(self.top[cell]) * 100.0)
					.clamp(1, RoomCell::MAX_OBSTACLE_CENTIMETRES);

				*out = RoomCell::obstacle(centimetres).byte();
			}
		}

		cells
	}
}

/// Votes every fine cell into the 10 cm grid, turned by `yaw`.
fn downsample(kinds: &[FineKind], levels: &[u8], yaw: u8) -> Vec<u8> {
	let mut votes = CoarseVotes::new(levels.len());
	let angle = f64::from(yaw) * 2.0 * PI / 256.0;
	let sine = math::sin(angle);
	let cosine = math::cos(angle);
	let inv_grid = 1.0 / GRID_CELL;
	let half_extent = EXTENT / 2.0;
	let origin_offset = -half_extent + FINE_OFFSET + 0.5 * FINE_CELL;
	let step_gx = FINE_CELL * cosine * inv_grid;
	let step_gz = FINE_CELL * sine * inv_grid;
	let grid_side_i = GRID_SIDE as i64;

	for row in 0..FINE_SIDE {
		let z = origin_offset + row as f64 * FINE_CELL;
		let mut gx = (origin_offset * cosine - z * sine + half_extent) * inv_grid;
		let mut gz = (origin_offset * sine + z * cosine + half_extent) * inv_grid;
		let row_kinds = &kinds[row * FINE_SIDE..(row + 1) * FINE_SIDE];

		for &kind in row_kinds {
			let column = math::fast_floor(gx);
			let coarse_row = math::fast_floor(gz);
			gx += step_gx;
			gz += step_gz;

			if column < 0 || coarse_row < 0 || column >= grid_side_i || coarse_row >= grid_side_i {
				continue;
			}

			let coarse = coarse_row as usize * GRID_SIDE + column as usize;

			match kind {
				FineKind::Unknown => votes.unknown[coarse] += 1,
				FineKind::Walkable(level) => {
					votes.walk[coarse * votes.level_count + level as usize] += 1;
				}
				FineKind::Obstacle(top) => {
					votes.obstacle[coarse] += 1;
					votes.top[coarse] = votes.top[coarse].max(top);
				}
				FineKind::Wall => votes.wall[coarse] += 1,
			}
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
/// An obstacle as tall as or taller than the eye shadows the floor all the
/// way to the edge of the grid.
fn hide(cells: &mut [u8], levels: &[u8], eye_height: f64) {
	let hidden = RoomCell::hidden(0).byte();
	let half_extent = EXTENT / 2.0;
	let inv_grid = 1.0 / GRID_CELL;
	let center = half_extent * inv_grid;
	let inv_eye = 1.0 / eye_height;
	let grid_side_i = GRID_SIDE as i64;
	let angle_step = 2.0 * PI / HIDDEN_RAYS as f64;

	for ray in 0..HIDDEN_RAYS {
		let angle = ray as f64 * angle_step;
		let sine = math::sin(angle);
		let cosine = math::cos(angle);
		let mut shadow_end = 0.0;
		let mut last = usize::MAX;
		let mut along = HIDDEN_STEP;
		let step_x = cosine * HIDDEN_STEP * inv_grid;
		let step_z = sine * HIDDEN_STEP * inv_grid;
		let mut gx = center + step_x;
		let mut gz = center + step_z;

		while along <= half_extent {
			let column = math::fast_floor(gx);
			let row = math::fast_floor(gz);

			if column < 0 || row < 0 || column >= grid_side_i || row >= grid_side_i {
				break;
			}

			let cell = row as usize * GRID_SIDE + column as usize;

			if cell != last {
				last = cell;

				let Some(value) = RoomCell::from_byte(cells[cell]) else {
					along += HIDDEN_STEP;
					gx += step_x;
					gz += step_z;
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

						if top >= eye_height {
							shadow_end = EXTENT;
						} else if top > 0.0 {
							shadow_end =
								shadow_end.max((along + GRID_CELL / 2.0) / (1.0 - top * inv_eye));
						}
					}
				}
			}

			along += HIDDEN_STEP;
			gx += step_x;
			gz += step_z;
		}
	}
}

/// Whether a single pixel `(column, row)` is valid and not flying across a
/// silhouette relative to its 4-neighbours.
#[inline]
fn is_not_flying(frame: &DepthFrame, column: usize, row: usize) -> bool {
	let width = frame.width;
	let depths = frame.depths.as_slice();
	let at = row * width + column;
	let depth = depths[at];
	if !(depth > 0.0 && depth.is_finite()) {
		return false;
	}
	let depth_f = f64::from(depth);
	let exceeds = |other: f32| -> bool {
		let other_f = f64::from(other);
		other > 0.0
			&& other.is_finite()
			&& (depth_f - other_f).abs() > FLYING_PIXEL_SHARE * depth_f.min(other_f)
	};
	if column > 0 && exceeds(depths[at - 1]) {
		return false;
	}
	if column + 1 < width && exceeds(depths[at + 1]) {
		return false;
	}
	if row > 0 && exceeds(depths[at - width]) {
		return false;
	}
	if row + 1 < frame.height && exceeds(depths[at + width]) {
		return false;
	}
	true
}

/// Which pixels are not flying: a pixel whose depth differs from a
/// 4-neighbour by more than [`FLYING_PIXEL_SHARE`] is across a silhouette.
#[cfg(test)]
fn flying_pixels(frame: &DepthFrame) -> Vec<bool> {
	let mut keep = vec![false; frame.width * frame.height];

	for row in 0..frame.height {
		for column in 0..frame.width {
			keep[row * frame.width + column] = is_not_flying(frame, column, row);
		}
	}

	keep
}

/// The fine cell a point falls in, or `None` outside the grid.
#[inline]
fn fine_cell(x: f64, z: f64) -> Option<usize> {
	const OFFSET: f64 = EXTENT / 2.0 - FINE_OFFSET;
	const INV_CELL: f64 = 1.0 / FINE_CELL;
	let column = math::fast_floor((x + OFFSET) * INV_CELL);
	let row = math::fast_floor((z + OFFSET) * INV_CELL);

	if column < 0 || row < 0 || column >= FINE_SIDE as i64 || row >= FINE_SIDE as i64 {
		return None;
	}

	Some(row as usize * FINE_SIDE + column as usize)
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
			origin: Vec3::ZERO,
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
