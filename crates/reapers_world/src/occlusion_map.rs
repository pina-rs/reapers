//! The room as it looks from where the player stands.

use reapers_rules::degrees;
use reapers_rules::wrap_angle;

use crate::depth_frame::DepthFrame;
use crate::math;

/// The room as it looks from where the player stands: how far away the
/// nearest thing standing in it is, in every direction the camera saw.
///
/// It is what lets a husk go behind the counter and be hidden by it. The
/// phone only ever turns where the player stands, so a picture taken while
/// the sigil was traced still lines up with the view from the same spot
/// during the fight, whichever way the phone points then: nothing has to be
/// read again. The floor is left out. A husk stands on the floor, and is
/// never hidden by it.
///
/// Like [`RoomMap`](crate::RoomMap) this is arithmetic and nothing else, so
/// that it can be tested against rooms drawn by hand.
#[derive(Clone, Debug)]
pub struct OcclusionMap {
	/// For each cell: how far away the nearest thing standing on the floor
	/// is, in metres; infinity where only floor was seen; zero where nothing
	/// was seen.
	metres: Vec<f32>,
	pictures: usize,
}

impl Default for OcclusionMap {
	fn default() -> Self {
		Self::new()
	}
}

impl OcclusionMap {
	/// How wide one cell of the map is, in degrees, each way.
	pub const CELL_DEGREES: f64 = 0.25;
	pub const COLUMNS: usize = (360.0 / Self::CELL_DEGREES) as usize;
	/// The farthest a cell records, in metres.
	pub const FARTHEST: f64 = 40.0;
	/// How far off the floor a point may be and still be floor, as in
	/// [`RoomMap`](crate::RoomMap).
	const FLOOR_SLACK: f64 = 0.07;
	const FLOOR_SLACK_PER_METRE: f64 = 0.045;
	/// The map runs all the way round, and from
	/// [`LOWEST`](Self::LOWEST) to [`HIGHEST`](Self::HIGHEST) degrees above
	/// the horizon: from the floor under a sitting player's feet to the top
	/// of a flier's air.
	pub const HIGHEST: f64 = 48.0;
	pub const LOWEST: f64 = -72.0;
	pub const ROWS: usize = ((Self::HIGHEST - Self::LOWEST) / Self::CELL_DEGREES) as usize;
	/// Nearer than this, in metres, is the player's own feet and hands.
	const SELF_METRES: f64 = 0.6;

	/// A map that has seen nothing.
	pub fn new() -> Self {
		Self {
			metres: vec![0.0; Self::COLUMNS * Self::ROWS],
			pictures: 0,
		}
	}

	/// Pictures taken in so far.
	pub const fn pictures(&self) -> usize {
		self.pictures
	}

	/// Takes in one picture whose floor is `floor` metres below the eye in
	/// its own metres, scaling every length by `scale` to agree with the
	/// floor the hunt stands on.
	pub fn add(&mut self, frame: &DepthFrame, floor: f64, scale: f64) {
		let pose = frame.pose;

		for row in 0..frame.height {
			for column in 0..frame.width {
				let (across, up) = frame.tangents(column, row);
				let depth = f64::from(frame.depths[row * frame.width + column]);

				if depth.is_nan() || depth <= 0.0 || depth.is_infinite() {
					continue;
				}

				let x = depth * (pose.forward.x + pose.right.x * across + pose.up.x * up);
				let y = depth * (pose.forward.y + pose.right.y * across + pose.up.y * up);
				let z = depth * (pose.forward.z + pose.right.z * across + pose.up.z * up);
				let away = math::sqrt(x * x + z * z);
				let distance = math::sqrt(away * away + y * y);

				if distance < Self::SELF_METRES {
					continue;
				}

				let Some(cell) = Self::cell_of(math::atan2(x, z), math::atan2(y, away)) else {
					continue;
				};
				let on_floor =
					(floor + y).abs() <= Self::FLOOR_SLACK + Self::FLOOR_SLACK_PER_METRE * away;

				if on_floor {
					if self.metres[cell] == 0.0 {
						self.metres[cell] = f32::INFINITY;
					}

					continue;
				}

				let metres = (distance * scale).min(Self::FARTHEST) as f32;
				let known = self.metres[cell];

				if known == 0.0 || metres < known {
					self.metres[cell] = metres;
				}
			}
		}

		self.pictures += 1;
	}

	/// How far away the nearest thing standing in the room is along `yaw`
	/// and `pitch`, in metres: infinite where only floor was seen there, and
	/// `None` where nothing was.
	pub fn distance_at(&self, yaw: f64, pitch: f64) -> Option<f64> {
		let metres = Self::cell_of(yaw, pitch).map_or(0.0, |cell| self.metres[cell]);

		(metres != 0.0).then_some(f64::from(metres))
	}

	/// Whether a point `metres` away along `yaw` and `pitch` is behind
	/// something the camera saw.
	pub fn hides(&self, yaw: f64, pitch: f64, metres: f64) -> bool {
		self.distance_at(yaw, pitch)
			.is_some_and(|nearest| nearest < metres)
	}

	/// The map as the pixels of an image [`COLUMNS`](Self::COLUMNS) wide and
	/// [`ROWS`](Self::ROWS) tall, four bytes a pixel, for a shader to read:
	/// the distance in centimetres in red (high byte) and green (low byte),
	/// and in blue whether anything standing was seen there (255), only
	/// floor (128), or nothing (0). Alpha is always 255.
	pub fn to_pixels(&self) -> Vec<u8> {
		let mut pixels = vec![0; self.metres.len() * 4];

		for (metres, pixel) in self.metres.iter().zip(pixels.as_chunks_mut::<4>().0) {
			pixel[3] = 255;

			if *metres == 0.0 {
				continue;
			}

			let centimetres = if metres.is_finite() {
				math::round(f64::from(*metres) * 100.0).clamp(0, 0xFFFF) as u16
			} else {
				0xFFFF
			};
			let [high, low] = centimetres.to_be_bytes();

			pixel[0] = high;
			pixel[1] = low;
			pixel[2] = if metres.is_finite() { 255 } else { 128 };
		}

		pixels
	}

	/// The cell that holds the direction `yaw`, `pitch`, or `None` when it
	/// is above or below what the map covers. Row 0 is the highest.
	fn cell_of(yaw: f64, pitch: f64) -> Option<usize> {
		let up = degrees(pitch);

		if !(Self::LOWEST..Self::HIGHEST).contains(&up) {
			return None;
		}

		let column = math::floor((degrees(wrap_angle(yaw)) + 180.0) / Self::CELL_DEGREES)
			.clamp(0, Self::COLUMNS as i64 - 1) as usize;
		let row = math::floor((Self::HIGHEST - up) / Self::CELL_DEGREES)
			.clamp(0, Self::ROWS as i64 - 1) as usize;

		Some(row * Self::COLUMNS + column)
	}
}
