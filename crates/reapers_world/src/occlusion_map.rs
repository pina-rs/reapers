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
/// read again. A picture taken from somewhere else is brought back to the
/// hunt's origin first, so a player who moved while tracing still hides a
/// husk behind the furniture they really stood in front of. The floor is
/// left out. A husk stands on the floor, and is never hidden by it.
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

	/// Takes in one picture whose floor is `floor` metres below its eye in
	/// its own metres, scaling every length by `scale` to agree with the
	/// floor the hunt stands on and moving it from where it was taken to the
	/// hunt's origin.
	pub fn add(&mut self, frame: &DepthFrame, floor: f64, scale: f64) {
		if !floor.is_finite() || floor <= 0.0 || !scale.is_finite() || scale <= 0.0 {
			return;
		}
		if frame.width == 0 || frame.height == 0 {
			return;
		}

		let pose = frame.pose;
		let origin = frame.origin;
		let width = frame.width;
		let height = frame.height;
		let inv_w = 2.0 / width as f64;
		let inv_h = 2.0 / height as f64;
		let across_step = inv_w * frame.tan_half_width;
		let across_start = (0.5 * inv_w - 1.0) * frame.tan_half_width;
		let step_x = (pose.right.x * across_step) as f32;
		let step_y = (pose.right.y * across_step) as f32;
		let step_z = (pose.right.z * across_step) as f32;
		let scale_f = scale as f32;
		let floor_f = floor as f32;
		let ox = origin.x as f32;
		let oy = origin.y as f32;
		let oz = origin.z as f32;
		let self_sq = (Self::SELF_METRES * Self::SELF_METRES) as f32;
		let rad_to_deg = 180.0_f32 / core::f32::consts::PI;
		let inv_cell = (1.0 / Self::CELL_DEGREES) as f32;
		let lowest_f = Self::LOWEST as f32;
		let highest_f = Self::HIGHEST as f32;
		let tan_low = math::tan(Self::LOWEST * (math::PI / 180.0)) as f32;
		let tan_high = math::tan(Self::HIGHEST * (math::PI / 180.0)) as f32;
		let floor_slack = Self::FLOOR_SLACK as f32;
		let floor_slack_per_m = Self::FLOOR_SLACK_PER_METRE as f32;
		let farthest_f = Self::FARTHEST as f32;
		let metres_buf = self.metres.as_mut_slice();

		for row in 0..height {
			let up = (1.0 - (row as f64 + 0.5) * inv_h) * frame.tan_half_height;
			let mut dir_x = (pose.forward.x + pose.up.x * up + pose.right.x * across_start) as f32;
			let mut dir_y = (pose.forward.y + pose.up.y * up + pose.right.y * across_start) as f32;
			let mut dir_z = (pose.forward.z + pose.up.z * up + pose.right.z * across_start) as f32;
			let row_depths = &frame.depths[row * width..(row + 1) * width];

			for &depth in row_depths {
				if depth > 0.0 && depth.is_finite() {
					let local_y = depth * dir_y;
					let scaled_d = depth * scale_f;
					let x = scaled_d * dir_x + ox;
					let y = local_y * scale_f + oy;
					let z = scaled_d * dir_z + oz;
					let away_sq = x * x + z * z;
					let dist_sq = away_sq + y * y;

					if dist_sq >= self_sq {
						let away = away_sq.sqrt();
						if y >= tan_low * away && y < tan_high * away {
							let pitch_deg = math::fast_atan2_f32(y, away) * rad_to_deg;
							if pitch_deg >= lowest_f && pitch_deg < highest_f {
								let yaw_deg = math::fast_atan2_f32(x, z) * rad_to_deg;
								let col = (((yaw_deg + 180.0) * inv_cell) as usize)
									.min(Self::COLUMNS - 1);
								let map_row = (((highest_f - pitch_deg) * inv_cell) as usize)
									.min(Self::ROWS - 1);
								let cell = map_row * Self::COLUMNS + col;

								let on_floor = ((floor_f + local_y) * scale_f).abs()
									<= floor_slack + floor_slack_per_m * away;

								if on_floor {
									if metres_buf[cell] == 0.0 {
										metres_buf[cell] = f32::INFINITY;
									}
								} else {
									let metres = dist_sq.sqrt().min(farthest_f);
									let known = metres_buf[cell];
									if known == 0.0 || metres < known {
										metres_buf[cell] = metres;
									}
								}
							}
						}
					}
				}

				dir_x += step_x;
				dir_y += step_y;
				dir_z += step_z;
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
