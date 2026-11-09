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

		if grid.columns < 2 || grid.rows < 2 || distances.len() < grid.columns * grid.rows {
			return frame;
		}

		let inverse = Reciprocals::of(grid, distances);
		let flat: Vec<bool> = (0..grid.rows - 1)
			.flat_map(|row| (0..grid.columns - 1).map(move |column| (column, row)))
			.map(|(column, row)| inverse.is_flat(column, row))
			.collect();

		// Where a pixel falls among the grid's rays, with ray `i` at `i`, or
		// `None` outside the outermost rays: what lies there is not known.
		let among = |along: f64, rays: usize| {
			(0.0..=(rays - 1) as f64).contains(&along).then(|| {
				let before = (math::floor(along) as usize).min(rays - 2);

				(before, along - before as f64)
			})
		};

		for row in 0..height {
			let (_, up) = frame.tangents(0, row);
			let Some((top, down)) = among(
				(1.0 - up / grid.tan_half_height) * (grid.rows as f64 / 2.0) - 0.5,
				grid.rows,
			) else {
				continue;
			};

			for column in 0..width {
				let (across, _) = frame.tangents(column, row);
				let Some((left, right)) = among(
					(across / grid.tan_half_width + 1.0) * (grid.columns as f64 / 2.0) - 0.5,
					grid.columns,
				) else {
					continue;
				};

				if !flat[top * (grid.columns - 1) + left] {
					continue;
				}

				let reciprocal = (inverse.at(left, top) * (1.0 - right)
					+ inverse.at(left + 1, top) * right)
					* (1.0 - down)
					+ (inverse.at(left, top + 1) * (1.0 - right)
						+ inverse.at(left + 1, top + 1) * right)
						* down;

				if reciprocal > 0.0 {
					frame.depths[row * width + column] = (1.0 / reciprocal) as f32;
				}
			}
		}

		frame
	}
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

	/// Rays across and down. Three each way at the least: see
	/// [`DepthFrame::from_rays`].
	pub columns: usize,
	pub rows: usize,
}

impl RayGrid {
	/// Rays across and down a phone held upright: about a degree apart
	/// through the Seeker's camera, 3072 rays a look.
	///
	/// Fewer do for most of the room. In the drawn kitchen, 12 by 16 rays
	/// find the floor to a millimetre and the counter's front and height to
	/// a few centimetres. What they miss is the counter's top. It is under
	/// three degrees tall from standing height, 2.2 m away, and a surface is
	/// only filled in where three rows of rays land on it (see
	/// [`DepthFrame::from_rays`]), so its top is never read as a ledge
	/// until there are 48 by 64. A grid may also be laid over only part of
	/// the view, such as its lower half, where the floor and what stands on
	/// it are.
	pub const COLUMNS: usize = 48;
	/// How far the corners of one cell of four rays may be from lying on one
	/// flat surface, and how far its slope may be from its neighbour's, as a
	/// share of the nearest corner's reciprocal depth: a little more than
	/// the platform's depth is out by on its own. A crease shallow enough to
	/// pass, such as the floor running into a far wall, is filled in a
	/// little nearer than it is: in the drawn rooms, never more than 2.5%.
	/// Half this, and the drawn kitchen's counter top is no longer read as a
	/// ledge.
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

/// The reciprocal of each ray's z-depth, laid out as the grid is, or NaN
/// where the ray hit nothing.
struct Reciprocals {
	values: Vec<f64>,
	columns: usize,
	rows: usize,
}

impl Reciprocals {
	fn of(grid: &RayGrid, distances: &[Option<f64>]) -> Self {
		let values = (0..grid.rows)
			.flat_map(|row| (0..grid.columns).map(move |column| (column, row)))
			.zip(distances)
			.map(|((column, row), distance)| {
				let (across, up) = grid.tangents(column, row);
				let stretch = math::sqrt(1.0 + across * across + up * up);

				match distance {
					Some(metres) if *metres > 0.0 && metres.is_finite() => stretch / metres,
					_ => f64::NAN,
				}
			})
			.collect();

		Self {
			values,
			columns: grid.columns,
			rows: grid.rows,
		}
	}

	/// The value at `column` and `row`, or NaN off the grid.
	fn at(&self, column: usize, row: usize) -> f64 {
		if column < self.columns && row < self.rows {
			self.values[row * self.columns + column]
		} else {
			f64::NAN
		}
	}

	/// Whether the cell from `column` and `row` to the next of each lies on
	/// one flat surface. A NaN anywhere it is needed fails every test.
	fn is_flat(&self, column: usize, row: usize) -> bool {
		let corners = [
			self.at(column, row),
			self.at(column + 1, row),
			self.at(column, row + 1),
			self.at(column + 1, row + 1),
		];
		let leeway = RayGrid::FLATNESS * corners.iter().copied().fold(0.0, f64::max);
		let close = |a: f64, b: f64| (a - b).abs() <= leeway;
		// Whether the step from `a` to `b` is the step before it or the one
		// after it along the same line, where `step(i)` is the step from
		// point `i` of the line to point `i + 1`.
		let even = |step: &dyn Fn(usize) -> f64, at: usize| {
			let before = at.checked_sub(1).map_or(f64::NAN, step);

			close(step(at), before) || close(step(at), step(at + 1))
		};

		close(corners[0] + corners[3], corners[1] + corners[2])
			&& [row, row + 1]
				.iter()
				.all(|line| even(&|at| self.at(at + 1, *line) - self.at(at, *line), column))
			&& [column, column + 1]
				.iter()
				.all(|line| even(&|at| self.at(*line, at + 1) - self.at(*line, at), row))
	}
}
