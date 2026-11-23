//! One look at the room, and how one is made from the platform's hit tests.

use reapers_rules::Vec3;

use crate::math;
use crate::pose::ArPose;

/// One look at the room: how far away everything in a picture is, where the
/// phone was pointing when it was taken, and where it stood.
///
/// Depths are z-depths: how far each pixel is along the direction the
/// camera faced, not along its own ray. A pixel `a` across and `b` up from
/// the middle, in tangents of the view, is at the point
/// `depth * (forward + a * right + b * up)`. A depth that is not a
/// positive, finite number is a pixel nothing was seen at.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthFrame {
	pub width: usize,
	pub height: usize,

	/// How far away each pixel is, in metres along the direction the camera
	/// faced, row by row from the top.
	pub depths: Vec<f32>,

	/// Where the phone was pointing.
	pub pose: ArPose,

	/// Where the phone's eye was when the picture was taken, in the hunt's
	/// space: x to the player's right, y up, z ahead of the hunt's origin.
	///
	/// A picture is measured from its own eye. Charting adds this to every
	/// point, so pictures taken from different places while the player moved
	/// land in one room instead of each being charted around itself. Zero is
	/// the hunt's origin, where the player stood when the room was first
	/// read.
	pub origin: Vec3,

	/// How much the picture takes in: the tangent of half its view across
	/// and of half its view up and down.
	pub tan_half_width: f64,
	pub tan_half_height: f64,
}

impl DepthFrame {
	/// How wide one pixel is, in tangents of the view, in the pictures every
	/// count in [`RoomMap`](crate::RoomMap) and
	/// [`OcclusionMap`](crate::OcclusionMap) was tuned on: the Seeker's
	/// camera, 252 pixels across a view whose half has a tangent of 0.4122.
	pub const PIXEL: f64 = 2.0 * 0.4122 / 252.0;

	/// How far, in tangents of the view, pixel `column` is to the right of
	/// the middle of a picture `width` pixels wide, and pixel `row` is above
	/// the middle of one `height` tall, at their centres.
	pub fn tangents(&self, column: usize, row: usize) -> (f64, f64) {
		(
			((column as f64 + 0.5) / self.width as f64 * 2.0 - 1.0) * self.tan_half_width,
			(1.0 - (row as f64 + 0.5) / self.height as f64 * 2.0) * self.tan_half_height,
		)
	}

	/// The depth picture that the platform's answers to `grid`'s rays make.
	///
	/// `distances` holds one answer for each of the grid's rays, in the
	/// order of [`RayGrid::rays`]: how far along the ray, in metres, the
	/// platform's depth was hit, or `None` where it was not.
	///
	/// The platform answers along each ray, and a frame holds z-depths, so
	/// each answer is first divided by the length its ray has when its
	/// forward part is 1. A grid is much coarser than the pictures the
	/// room's counts were tuned on, so it is then filled out to that
	/// fineness ([`PIXEL`](Self::PIXEL)), one cell of four rays at a time,
	/// by the one rule that is exact for a flat surface: the reciprocal of
	/// a z-depth across a plane changes evenly with the tangents.
	///
	/// Only what lies between the rays is filled: the thin border outside the
	/// outermost ones is left empty. And only a cell that lies on one flat
	/// surface is filled. Anything else,
	/// such as the edge of a table against the floor beyond it, or the
	/// crease where the floor meets a wall, would be filled with a surface
	/// that is not there, so it is left empty instead, as is a cell with a
	/// corner that was not hit. A cell is flat when its four corners are
	/// (see [`RayGrid::FLATNESS`]), and when, each way, it slopes as the cell
	/// before or the cell after it does: a crease along the line between two
	/// rows passes the first test but not the second. So a grid needs three
	/// rays each way at the least.
	pub fn from_rays(grid: &RayGrid, distances: &[Option<f64>]) -> Self {
		Self::from_rays_at(grid, distances, Vec3::ZERO)
	}

	/// The depth picture that the platform's answers to `grid`'s rays make,
	/// taken from `origin`: where the phone's eye was, in the hunt's space.
	///
	/// See [`from_rays`](Self::from_rays) for how the answers become a
	/// picture. The origin is kept so that the picture charts where it was
	/// taken rather than where the hunt began.
	pub fn from_rays_at(grid: &RayGrid, distances: &[Option<f64>], origin: Vec3) -> Self {
		let hits: Vec<Option<RayHit>> = distances
			.iter()
			.map(|distance| {
				distance.map(|distance| {
					RayHit {
						distance,
						normal: None,
					}
				})
			})
			.collect();

		Self::from_ray_hits_at(grid, &hits, origin)
	}

	/// Like [`from_rays`](Self::from_rays), taking optional surface normals
	/// alongside each ray distance so sparse ray grids can verify flat quads
	/// in 2×2 cells and reconstruct single-row horizontal ledges (such as
	/// counters or tables).
	pub fn from_ray_hits(grid: &RayGrid, hits: &[Option<RayHit>]) -> Self {
		Self::from_ray_hits_at(grid, hits, Vec3::ZERO)
	}

	/// Like [`from_rays_at`](Self::from_rays_at), using surface normals when
	/// the platform provides them.
	pub fn from_ray_hits_at(grid: &RayGrid, hits: &[Option<RayHit>], origin: Vec3) -> Self {
		let width = (math::round(2.0 * grid.tan_half_width / Self::PIXEL).max(0) as usize)
			.max(grid.columns);
		let height =
			(math::round(2.0 * grid.tan_half_height / Self::PIXEL).max(0) as usize).max(grid.rows);
		let mut frame = Self {
			width,
			height,
			depths: vec![f32::NAN; width * height],
			pose: grid.pose,
			origin,
			tan_half_width: grid.tan_half_width,
			tan_half_height: grid.tan_half_height,
		};

		if grid.columns < 2 || grid.rows < 2 || hits.len() < grid.columns * grid.rows {
			return frame;
		}

		let has_normals = hits
			.iter()
			.take(grid.columns * grid.rows)
			.any(|hit| hit.as_ref().and_then(|h| h.normal).is_some());

		if !has_normals && (grid.columns < 3 || grid.rows < 3) {
			return frame;
		}

		let samples = RaySamples::of(grid, hits, has_normals);
		let col_scale = grid.columns as f64 / width as f64;
		let row_scale = grid.rows as f64 / height as f64;

		// Rasterize only flat 2×2 ray cells directly over their pixel bounds.
		for top in 0..grid.rows - 1 {
			let y0 = math::ceil((top as f64 + 0.5) / row_scale - 0.5).max(0);
			let y1 = math::floor((top as f64 + 1.5) / row_scale - 0.5).min(height as i64 - 1);
			if y1 < y0 {
				continue;
			}
			let (y0, y_end) = (y0 as usize, y1 as usize);

			for left in 0..grid.columns - 1 {
				if !samples.is_flat(left, top) {
					continue;
				}

				let x0 = math::ceil((left as f64 + 0.5) / col_scale - 0.5).max(0);
				let x1 = math::floor((left as f64 + 1.5) / col_scale - 0.5).min(width as i64 - 1);
				if x1 < x0 {
					continue;
				}
				let (x0, x_end) = (x0 as usize, x1 as usize);

				let r00 = samples.reciprocal(left, top);
				let r10 = samples.reciprocal(left + 1, top);
				let r01 = samples.reciprocal(left, top + 1);
				let r11 = samples.reciprocal(left + 1, top + 1);

				for row in y0..=y_end {
					let down = ((row as f64 + 0.5) * row_scale - 0.5 - top as f64).clamp(0.0, 1.0);
					let inv_down = 1.0 - down;
					let left_edge = r00 * inv_down + r01 * down;
					let right_edge = r10 * inv_down + r11 * down;
					let row_base = row * width;

					for column in x0..=x_end {
						let right =
							((column as f64 + 0.5) * col_scale - 0.5 - left as f64).clamp(0.0, 1.0);
						let reciprocal = left_edge * (1.0 - right) + right_edge * right;

						if reciprocal > 0.0 && reciprocal.is_finite() {
							frame.depths[row_base + column] = (1.0 / reciprocal) as f32;
						}
					}
				}
			}
		}

		// Single-row horizontal ledge reconstruction: when two adjacent rays
		// hit the same upward-facing level plane (e.g. a narrow kitchen counter
		// top on a sparse 12×16 ray grid), synthesize that plane over ±0.5
		// ray-rows where no flat quad already filled the pixels.
		if has_normals {
			Self::reconstruct_horizontal_ledges(&mut frame, grid, &samples, col_scale, row_scale);
		}

		frame
	}

	fn reconstruct_horizontal_ledges(
		frame: &mut Self,
		grid: &RayGrid,
		samples: &RaySamples,
		col_scale: f64,
		row_scale: f64,
	) {
		const LEVEL_NORMAL_Y: f64 = 0.90;
		const MAX_PLANE_HEIGHT_GAP: f64 = 0.03;
		const MAX_HORIZONTAL_STEP: f64 = 0.35;

		let width = frame.width;
		let height = frame.height;
		let pose = grid.pose;
		let inv_w = 2.0 / width as f64;
		let inv_h = 2.0 / height as f64;

		for r in 0..grid.rows {
			let y0 = math::ceil((r as f64) / row_scale - 0.5).max(0);
			let y1 = math::floor((r as f64 + 1.0) / row_scale - 0.5).min(height as i64 - 1);
			if y1 < y0 {
				continue;
			}
			let (y0, y_end) = (y0 as usize, y1 as usize);

			for c in 0..grid.columns - 1 {
				let (Some(n0), Some(n1)) = (samples.normal(c, r), samples.normal(c + 1, r)) else {
					continue;
				};
				if n0.y < LEVEL_NORMAL_Y || n1.y < LEVEL_NORMAL_Y {
					continue;
				}
				let (Some(p0), Some(p1)) = (samples.point(c, r), samples.point(c + 1, r)) else {
					continue;
				};
				if (p0.y - p1.y).abs() > MAX_PLANE_HEIGHT_GAP {
					continue;
				}
				let d0 = samples.distance(c, r);
				let d1 = samples.distance(c + 1, r);
				if (d0 - d1).abs() > MAX_HORIZONTAL_STEP {
					continue;
				}

				let plane_y = f64::midpoint(p0.y, p1.y);
				let min_d = d0.min(d1) * 0.75;
				let max_d = d0.max(d1) * 1.25;

				let x0 = math::ceil((c as f64 + 0.5) / col_scale - 0.5).max(0);
				let x1 = math::floor((c as f64 + 1.5) / col_scale - 0.5).min(width as i64 - 1);
				if x1 < x0 {
					continue;
				}
				let (x0, x_end) = (x0 as usize, x1 as usize);

				for row in y0..=y_end {
					let row_base = row * width;
					let up = (1.0 - (row as f64 + 0.5) * inv_h) * frame.tan_half_height;
					let base_y = pose.forward.y + pose.up.y * up;
					let up_sq_plus_one = 1.0 + up * up;

					for column in x0..=x_end {
						if frame.depths[row_base + column].is_finite() {
							continue;
						}
						let across = ((column as f64 + 0.5) * inv_w - 1.0) * frame.tan_half_width;
						let dir_y = base_y + pose.right.y * across;
						if dir_y.abs() < 1e-4 {
							continue;
						}
						let z_depth = plane_y / dir_y;
						if !z_depth.is_finite() || z_depth <= 0.0 {
							continue;
						}
						let stretch = math::sqrt(up_sq_plus_one + across * across);
						let ray_dist = z_depth * stretch;
						if ray_dist >= min_d && ray_dist <= max_d {
							frame.depths[row_base + column] = z_depth as f32;
						}
					}
				}
			}
		}
	}
}

/// One ray-cast hit returned by the platform: distance along the ray in metres
/// and an optional unit surface normal in the hunt's world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
	pub distance: f64,
	pub normal: Option<Vec3>,
}

/// The rays to ask the platform about, to make a [`DepthFrame`] from its
/// depth.
///
/// The game's Wasm guest never sees the platform's depth picture: it can
/// only cast a ray and be told how far along it the depth was hit. So a
/// look at the room is a grid of rays through the view, one through the
/// middle of each cell of a `columns` by `rows` grid laid across it, as
/// the pixels of a picture are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayGrid {
	/// Where the phone was pointing: the pose of the same platform frame the
	/// rays are cast in.
	pub pose: ArPose,

	/// The tangent of half the view across and up and down. The grid need
	/// not cover the camera's whole picture, only some of it.
	pub tan_half_width: f64,
	pub tan_half_height: f64,

	/// Rays across and down. Three each way at the least without normals
	/// (two each way with surface normals): see [`DepthFrame::from_ray_hits`].
	pub columns: usize,
	pub rows: usize,
}

impl RayGrid {
	/// Rays across and down a phone held upright: about a degree apart
	/// through the Seeker's camera, 3072 rays a look.
	///
	/// Fewer do for most of the room. In the drawn kitchen, 12 by 16 rays
	/// find the floor to a millimetre and the counter's front and height to
	/// a few centimetres. Without surface normals a flat surface needs three
	/// rows of rays to pass the slope-continuity check; supplying platform
	/// normals via [`DepthFrame::from_ray_hits`] lets 12 by 16 rays verify
	/// 2×2 quads and reconstruct narrow horizontal ledges directly.
	pub const COLUMNS: usize = 48;
	/// How far the corners of one cell of four rays may be from lying on one
	/// flat surface, and how far its slope may be from its neighbour's, as a
	/// share of the nearest corner's reciprocal depth: a little more than
	/// the platform's depth is out by on its own.
	pub const FLATNESS: f64 = 0.04;
	/// See [`COLUMNS`](Self::COLUMNS).
	pub const ROWS: usize = 64;

	/// The usual grid through a view `pose` points along.
	pub const fn new(pose: ArPose, tan_half_width: f64, tan_half_height: f64) -> Self {
		Self {
			pose,
			tan_half_width,
			tan_half_height,
			columns: Self::COLUMNS,
			rows: Self::ROWS,
		}
	}

	/// How far, in tangents of the view, the ray through `column` and `row`
	/// is to the right of and above the middle.
	pub fn tangents(&self, column: usize, row: usize) -> (f64, f64) {
		(
			((column as f64 + 0.5) / self.columns as f64 * 2.0 - 1.0) * self.tan_half_width,
			(1.0 - (row as f64 + 0.5) / self.rows as f64 * 2.0) * self.tan_half_height,
		)
	}

	/// The direction of the ray through `column` and `row`, of length 1, in
	/// the hunt's space. [`to_platform`](crate::to_platform) turns it into
	/// the direction to cast it along.
	pub fn ray(&self, column: usize, row: usize) -> Vec3 {
		let (across, up) = self.tangents(column, row);
		let pose = self.pose;

		(pose.forward + pose.right * across + pose.up * up).normalized()
	}

	/// Every ray of the grid, row by row from the top.
	pub fn rays(&self) -> impl Iterator<Item = Vec3> + '_ {
		(0..self.rows)
			.flat_map(move |row| (0..self.columns).map(move |column| self.ray(column, row)))
	}
}

/// Per-ray reciprocal z-depth, hit position, ray distance, and optional unit
/// surface normal laid out in grid order.
struct RaySamples {
	reciprocals: Vec<f64>,
	distances: Vec<f64>,
	points: Vec<Option<Vec3>>,
	normals: Vec<Option<Vec3>>,
	columns: usize,
	rows: usize,
}

impl RaySamples {
	/// Cosine of 10 degrees: normal agreement threshold for a planar 2×2 cell.
	const NORMAL_DOT_MIN: f64 = 0.9848;
	/// Maximum plane distance error (metres) for normal-verified 2×2 cells.
	const PLANE_TOLERANCE: f64 = 0.03;

	fn of(grid: &RayGrid, hits: &[Option<RayHit>], has_normals: bool) -> Self {
		let count = grid.columns * grid.rows;
		let mut reciprocals = Vec::with_capacity(count);
		let mut distances = Vec::with_capacity(if has_normals { count } else { 0 });
		let mut points = Vec::with_capacity(if has_normals { count } else { 0 });
		let mut normals = Vec::with_capacity(if has_normals { count } else { 0 });
		let pose = grid.pose;

		for (index, hit) in hits.iter().take(count).enumerate() {
			let row = index / grid.columns;
			let column = index % grid.columns;
			let (across, up) = grid.tangents(column, row);
			let stretch = math::sqrt(1.0 + across * across + up * up);

			match hit {
				Some(hit) if hit.distance > 0.0 && hit.distance.is_finite() => {
					let z_depth = hit.distance / stretch;
					reciprocals.push(1.0 / z_depth);
					if has_normals {
						let direction = pose.forward + pose.right * across + pose.up * up;
						distances.push(hit.distance);
						points.push(Some(direction * z_depth));
						let unit_normal = hit.normal.and_then(|n| {
							let len = n.length();
							(len.is_finite() && len > 1e-6).then(|| n / len)
						});
						normals.push(unit_normal);
					}
				}
				_ => {
					reciprocals.push(f64::NAN);
					if has_normals {
						distances.push(f64::NAN);
						points.push(None);
						normals.push(None);
					}
				}
			}
		}

		Self {
			reciprocals,
			distances,
			points,
			normals,
			columns: grid.columns,
			rows: grid.rows,
		}
	}

	fn reciprocal(&self, column: usize, row: usize) -> f64 {
		if column < self.columns && row < self.rows {
			self.reciprocals[row * self.columns + column]
		} else {
			f64::NAN
		}
	}

	fn distance(&self, column: usize, row: usize) -> f64 {
		self.distances[row * self.columns + column]
	}

	fn point(&self, column: usize, row: usize) -> Option<Vec3> {
		self.points
			.get(row * self.columns + column)
			.copied()
			.flatten()
	}

	fn normal(&self, column: usize, row: usize) -> Option<Vec3> {
		self.normals
			.get(row * self.columns + column)
			.copied()
			.flatten()
	}

	/// Whether the cell from `column` and `row` to the next of each lies on
	/// one flat surface. Uses surface normals when all four corners carry
	/// them; otherwise falls back to the 3×3 reciprocal-slope continuity test.
	fn is_flat(&self, column: usize, row: usize) -> bool {
		if let (Some(n0), Some(n1), Some(n2), Some(n3)) = (
			self.normal(column, row),
			self.normal(column + 1, row),
			self.normal(column, row + 1),
			self.normal(column + 1, row + 1),
		) && let (Some(p0), Some(p1), Some(p2), Some(p3)) = (
			self.point(column, row),
			self.point(column + 1, row),
			self.point(column, row + 1),
			self.point(column + 1, row + 1),
		) {
			let aligned = n0.dot(n1) >= Self::NORMAL_DOT_MIN
				&& n0.dot(n2) >= Self::NORMAL_DOT_MIN
				&& n0.dot(n3) >= Self::NORMAL_DOT_MIN
				&& n1.dot(n2) >= Self::NORMAL_DOT_MIN
				&& n1.dot(n3) >= Self::NORMAL_DOT_MIN
				&& n2.dot(n3) >= Self::NORMAL_DOT_MIN;
			if aligned {
				let avg = (n0 + n1 + n2 + n3).normalized();
				let coplanar = [p1 - p0, p2 - p0, p3 - p0]
					.iter()
					.all(|delta| delta.dot(avg).abs() <= Self::PLANE_TOLERANCE);
				if coplanar {
					return true;
				}
			}
			// Normals were present and explicitly disagreed (crease or edge).
			return false;
		}

		let corners = [
			self.reciprocal(column, row),
			self.reciprocal(column + 1, row),
			self.reciprocal(column, row + 1),
			self.reciprocal(column + 1, row + 1),
		];
		let leeway = RayGrid::FLATNESS * corners.iter().copied().fold(0.0, f64::max);
		let close = |a: f64, b: f64| (a - b).abs() <= leeway;
		let even = |step: &dyn Fn(usize) -> f64, at: usize| {
			let before = at.checked_sub(1).map_or(f64::NAN, step);

			close(step(at), before) || close(step(at), step(at + 1))
		};

		close(corners[0] + corners[3], corners[1] + corners[2])
			&& [row, row + 1].iter().all(|line| {
				even(
					&|at| self.reciprocal(at + 1, *line) - self.reciprocal(at, *line),
					column,
				)
			})
			&& [column, column + 1].iter().all(|line| {
				even(
					&|at| self.reciprocal(*line, at + 1) - self.reciprocal(*line, at),
					row,
				)
			})
	}
}
