//! The room as the game and the server both see it: a gravity-aligned
//! grid of 10 centimetre cells, centred on the player.

use serde::Deserialize;
use serde::Serialize;

use crate::error::FormatError;
use crate::math;
use crate::vec3::Vec3;

/// What a cell of a [`RoomGrid`] holds.
///
/// The kind is stored in the top three bits of the cell's byte, and the
/// bottom five hold either a level index ([`RoomCellKind::Walkable`] and
/// [`RoomCellKind::Hidden`]) or an obstacle top in 8 centimetre steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomCellKind {
	/// No picture ever saw this cell. A husk may cross it, since the room
	/// is not known to be blocked there, but nothing is known to be under
	/// it.
	Unknown,

	/// A level top a husk can stand on: a floor, a desk, a chair seat.
	Walkable,

	/// Something standing in the room. The payload is its top in 8 cm
	/// steps.
	Obstacle,

	/// Something taller than a husk can pass over. It blocks every leap.
	Wall,

	/// Floor that the scan knows is there but the player could not see,
	/// directly behind something. A husk may stand there to hide.
	Hidden,
}

impl RoomCellKind {
	/// The three-bit value stored in a cell's byte.
	const fn code(self) -> u8 {
		match self {
			Self::Unknown => 0,
			Self::Walkable => 1,
			Self::Obstacle => 2,
			Self::Wall => 3,
			Self::Hidden => 4,
		}
	}

	/// The kind whose code is `code`, or nothing when the code names no
	/// kind.
	const fn from_code(code: u8) -> Option<Self> {
		match code {
			0 => Some(Self::Unknown),
			1 => Some(Self::Walkable),
			2 => Some(Self::Obstacle),
			3 => Some(Self::Wall),
			4 => Some(Self::Hidden),
			_ => None,
		}
	}
}

/// One cell of a [`RoomGrid`]: its kind and its five-bit payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomCell {
	/// What the cell holds.
	pub kind: RoomCellKind,

	/// The level index for a walkable or hidden cell, or the top in 8 cm
	/// steps for an obstacle.
	pub payload: u8,
}

impl RoomCell {
	/// The most levels a grid can carry in five bits.
	pub const MAX_LEVEL: u8 = 31;
	/// The tallest obstacle the format can store, in centimetres.
	pub const MAX_OBSTACLE_CENTIMETRES: i64 = 248;
	/// A cell nothing saw.
	pub const UNKNOWN: Self = Self {
		kind: RoomCellKind::Unknown,
		payload: 0,
	};
	/// Something too tall to stand on or leap over.
	pub const WALL: Self = Self {
		kind: RoomCellKind::Wall,
		payload: 0,
	};

	/// A level top at `level` in the grid's level table.
	///
	/// The five-bit payload carries levels up to 31; anything else is a
	/// mistake on the caller's part.
	pub const fn walkable(level: u8) -> Self {
		assert!(level <= Self::MAX_LEVEL, "a level index must be in 0..31");
		Self {
			kind: RoomCellKind::Walkable,
			payload: level,
		}
	}

	/// Floor at `level` that is known but out of sight behind something.
	pub const fn hidden(level: u8) -> Self {
		assert!(level <= Self::MAX_LEVEL, "a level index must be in 0..31");
		Self {
			kind: RoomCellKind::Hidden,
			payload: level,
		}
	}

	/// Something standing `height_centimetres` above the floor, rounded to
	/// the nearest 8 centimetres the format can store.
	pub fn obstacle(height_centimetres: i64) -> Self {
		assert!(
			(0..=Self::MAX_OBSTACLE_CENTIMETRES).contains(&height_centimetres),
			"an obstacle's height must be in 0..248 centimetres"
		);
		Self {
			kind: RoomCellKind::Obstacle,
			payload: math::round(height_centimetres as f64 / 8.0) as u8,
		}
	}

	/// The cell whose byte is `byte`.
	pub const fn from_byte(byte: u8) -> Option<Self> {
		let Some(kind) = RoomCellKind::from_code(byte >> 5) else {
			return None;
		};

		Some(Self {
			kind,
			payload: byte & 0x1F,
		})
	}

	/// The level index this cell stands at, or nothing when it holds no
	/// level.
	pub const fn level(self) -> Option<u8> {
		match self.kind {
			RoomCellKind::Walkable | RoomCellKind::Hidden => Some(self.payload),
			_ => None,
		}
	}

	/// How high above the floor an obstacle's top is, in centimetres.
	pub const fn top_centimetres(self) -> i64 {
		self.payload as i64 * 8
	}

	/// How this cell is stored: three bits of kind above five of payload.
	pub const fn byte(self) -> u8 {
		(self.kind.code() << 5) | (self.payload & 0x1F)
	}
}

/// The room as the game and the server both see it: a gravity-aligned grid
/// of 10 centimetre cells, centred on the player's pivot.
///
/// A cell holds one of five kinds. A walkable or hidden cell stands at one
/// of the levels in `levels_centimetres`, which are heights above the
/// floor and begin at the floor itself. An obstacle holds its top in 8 cm
/// steps, and a wall is too tall to cross.
///
/// `x` runs to the player's right and `z` straight ahead, in the hunt's
/// own frame, with the origin at the player's eye. The grid may be turned
/// to the room's own axes by `grid_yaw`; every conversion between a world
/// point and a cell goes through that turn, so a cell is a square in the
/// room even when it is a diamond in the hunt's frame.
///
/// Everything a husk reads from the grid is an integer or a quotient of
/// integers, so a hunt replays exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomGrid {
	/// How far the floor is below the player's eye, in centimetres.
	pub eye_height_centimetres: u16,

	/// How far in front of the player's turning axis the phone is held, in
	/// centimetres. The grid is centred on the player's pivot, not the
	/// lens.
	pub pivot_arm_centimetres: u8,

	/// How far the grid is turned from the hunt's own axes, in 360/256
	/// steps.
	pub grid_yaw: u8,

	/// How many cells across, from the player's left to right.
	pub width: u8,

	/// How many cells deep, from behind the player to ahead.
	pub depth: u8,

	/// The player's cell's column. It holds the world origin.
	pub player_x: u8,

	/// The player's cell's row.
	pub player_z: u8,

	/// Every level a cell can stand at, in centimetres above the floor.
	/// Index zero is the floor.
	pub levels_centimetres: Vec<u8>,

	/// Every cell, row-major with `x` fastest, as each cell's byte.
	pub cells: Vec<u8>,

	/// The turn's cosine, so a point does not pay for a sine twice.
	yaw_cos: f64,

	/// The turn's sine.
	yaw_sin: f64,
}

impl RoomGrid {
	/// One cell is 10 centimetres across.
	pub const CELL_CENTIMETRES: i64 = 10;
	/// One cell is a tenth of a metre across.
	pub const CELL_METRES: f64 = 0.1;
	/// The format this grid is written in.
	pub const VERSION: u8 = 1;
	/// How high a wall is taken to be, in centimetres. It is above
	/// anything a husk can jump, so every leap through it is refused.
	pub const WALL_CENTIMETRES: i64 = 255;

	/// A grid of `width` by `depth` cells, row by row from the player's
	/// left to right and from behind the player to ahead, with `x`
	/// fastest.
	///
	/// `levels_centimetres` are heights above the floor and must begin at
	/// zero, and `cells` are in the same row-major order, one per cell.
	/// The cell count is checked, and everything deeper is left to
	/// [`RoomGrid::is_well_formed`], which [`RoomGrid::decode`] holds its
	/// bytes to.
	#[allow(
		clippy::too_many_arguments,
		reason = "a room is read out of its bytes in this order"
	)]
	pub fn new(
		eye_height_centimetres: u16,
		pivot_arm_centimetres: u8,
		grid_yaw: u8,
		width: u8,
		depth: u8,
		player_x: u8,
		player_z: u8,
		levels_centimetres: Vec<u8>,
		cells: Vec<u8>,
	) -> Self {
		assert!(
			cells.len() == usize::from(width) * usize::from(depth),
			"a room grid needs one cell per square"
		);

		let yaw = f64::from(grid_yaw) * 2.0 * math::PI / 256.0;

		Self {
			eye_height_centimetres,
			pivot_arm_centimetres,
			grid_yaw,
			width,
			depth,
			player_x,
			player_z,
			levels_centimetres,
			cells,
			yaw_cos: math::cos(yaw),
			yaw_sin: math::sin(yaw),
		}
	}

	/// How many cells the grid holds.
	pub fn cell_count(&self) -> usize {
		self.cells.len()
	}

	/// The grid's turn, in radians to the right of the hunt's own axes.
	pub fn yaw(&self) -> f64 {
		f64::from(self.grid_yaw) * 2.0 * math::PI / 256.0
	}

	/// Whether `x`, `z` names a cell of the grid.
	pub fn contains(&self, x: i64, z: i64) -> bool {
		x >= 0 && x < i64::from(self.width) && z >= 0 && z < i64::from(self.depth)
	}

	/// The cell index of `x`, `z`, row-major with `x` fastest. The pair
	/// must name a cell of the grid.
	pub fn index_of(&self, x: i64, z: i64) -> usize {
		debug_assert!(self.contains(x, z));

		z as usize * usize::from(self.width) + x as usize
	}

	/// The column of `index`.
	pub fn x_of(&self, index: usize) -> i64 {
		(index % usize::from(self.width)) as i64
	}

	/// The row of `index`.
	pub fn z_of(&self, index: usize) -> i64 {
		(index / usize::from(self.width)) as i64
	}

	/// The cell at `x`, `z`, or an unknown cell outside the grid.
	pub fn cell_at(&self, x: i64, z: i64) -> RoomCell {
		if !self.contains(x, z) {
			return RoomCell::UNKNOWN;
		}

		RoomCell::from_byte(self.cells[self.index_of(x, z)]).unwrap_or(RoomCell::UNKNOWN)
	}

	/// The stored byte at `x`, `z`, or zero outside the grid.
	pub fn byte_at(&self, x: i64, z: i64) -> u8 {
		if self.contains(x, z) {
			self.cells[self.index_of(x, z)]
		} else {
			0
		}
	}

	/// The level index under `x`, `z`, or -1 where it is not walkable.
	pub fn level_index_at(&self, x: i64, z: i64) -> i8 {
		let cell = self.cell_at(x, z);

		match cell.level() {
			Some(level) => level as i8,
			None => -1,
		}
	}

	/// The height above the floor of the top of `x`, `z`, in centimetres.
	///
	/// An unknown cell is taken to be floor, since nothing was seen to
	/// stand there. A wall, or a point outside the grid, is
	/// [`RoomGrid::WALL_CENTIMETRES`], which no leap clears.
	pub fn top_centimetres_at(&self, x: i64, z: i64) -> i64 {
		if !self.contains(x, z) {
			return Self::WALL_CENTIMETRES;
		}

		let cell = self.cell_at(x, z);

		match cell.kind {
			RoomCellKind::Walkable | RoomCellKind::Hidden => {
				i64::from(self.levels_centimetres[usize::from(cell.payload)])
			}
			RoomCellKind::Obstacle => cell.top_centimetres(),
			RoomCellKind::Unknown => 0,
			RoomCellKind::Wall => Self::WALL_CENTIMETRES,
		}
	}

	/// The height of the top of the cell under a world point, in
	/// centimetres above the floor.
	pub fn top_centimetres_at_world(&self, world_x: f64, world_z: f64) -> i64 {
		self.top_centimetres_at(
			self.cell_x_at(world_x, world_z),
			self.cell_z_at(world_x, world_z),
		)
	}

	/// The world height of the top of `x`, `z`, in metres from the
	/// player's eye.
	pub fn height_at(&self, x: i64, z: i64) -> f64 {
		(self.top_centimetres_at(x, z) - i64::from(self.eye_height_centimetres)) as f64 / 100.0
	}

	/// The cell column the world point falls in, nearest centre.
	pub fn cell_x_at(&self, world_x: f64, world_z: f64) -> i64 {
		let grid_x = world_x * self.yaw_cos - world_z * self.yaw_sin;

		i64::from(self.player_x) + math::floor_int(grid_x / Self::CELL_METRES + 0.5)
	}

	/// The cell row the world point falls in, nearest centre.
	pub fn cell_z_at(&self, world_x: f64, world_z: f64) -> i64 {
		let grid_z = world_x * self.yaw_sin + world_z * self.yaw_cos;

		i64::from(self.player_z) + math::floor_int(grid_z / Self::CELL_METRES + 0.5)
	}

	/// The middle of the top of `x`, `z`, in the hunt's frame.
	pub fn centre_of(&self, x: i64, z: i64) -> Vec3 {
		let grid_x = (x - i64::from(self.player_x)) as f64 * Self::CELL_METRES;
		let grid_z = (z - i64::from(self.player_z)) as f64 * Self::CELL_METRES;

		Vec3::new(
			grid_x * self.yaw_cos + grid_z * self.yaw_sin,
			self.height_at(x, z),
			-grid_x * self.yaw_sin + grid_z * self.yaw_cos,
		)
	}

	/// The grid direction from `from_x`, `from_z` to the player's cell, as
	/// a unit vector in grid space.
	pub fn toward_player(&self, from_x: i64, from_z: i64) -> Vec3 {
		let x = (i64::from(self.player_x) - from_x) as f64;
		let z = (i64::from(self.player_z) - from_z) as f64;
		let length = math::sqrt(x * x + z * z);

		if length == 0.0 {
			Vec3::FORWARD
		} else {
			Vec3::new(x / length, 0.0, z / length)
		}
	}

	/// The world direction of a unit vector given in grid space.
	pub fn world_direction(&self, grid_direction: Vec3) -> Vec3 {
		Vec3::new(
			grid_direction.x * self.yaw_cos + grid_direction.z * self.yaw_sin,
			grid_direction.y,
			-grid_direction.x * self.yaw_sin + grid_direction.z * self.yaw_cos,
		)
	}

	/// Whether something in the room stands between the player's eye and
	/// `point`, in the hunt's frame: whether the line of sight to it
	/// passes below the top of any cell it crosses.
	///
	/// The line is sampled a cell's width apart from the eye to the
	/// point, both ends included, so the cell under the point counts too.
	/// Beyond the grid the room is taken to be a wall.
	pub fn hides(&self, point: Vec3) -> bool {
		let away = math::sqrt(point.x * point.x + point.z * point.z);
		let steps = math::ceil_int(away / Self::CELL_METRES).max(1);

		for step in 0..=steps {
			let along = step as f64 / steps as f64;
			let top = self.top_metres_at_world(point.x * along, point.z * along);

			if top > point.y * along + 1e-9 {
				return true;
			}
		}

		false
	}

	/// How far out along `ray`, a unit direction from the player's eye,
	/// the first surface of the room stands, in metres: the floor, the top
	/// of something or the face of a wall. No farther than `farthest`.
	///
	/// The ray is sampled a cell's width apart, so where it lands is good
	/// to a cell.
	pub fn surface_along(&self, ray: Vec3, farthest: f64) -> f64 {
		let steps = math::ceil_int(farthest / Self::CELL_METRES).max(1);

		for step in 1..=steps {
			let metres = farthest * step as f64 / steps as f64;
			let point = ray * metres;

			if point.y <= self.top_metres_at_world(point.x, point.z) {
				return metres;
			}
		}

		farthest
	}

	/// The top of the cell under a world point, in metres from the eye.
	fn top_metres_at_world(&self, world_x: f64, world_z: f64) -> f64 {
		(self.top_centimetres_at_world(world_x, world_z) - i64::from(self.eye_height_centimetres))
			as f64
			/ 100.0
	}

	/// Whether this grid is one a phone or a test could have made.
	pub fn is_well_formed(&self) -> bool {
		if !(30..=300).contains(&self.eye_height_centimetres)
			|| self.width == 0
			|| self.depth == 0
			|| i64::from(self.player_x) >= i64::from(self.width)
			|| i64::from(self.player_z) >= i64::from(self.depth)
			|| self.levels_centimetres.is_empty()
			|| self.levels_centimetres.len() > usize::from(RoomCell::MAX_LEVEL)
			|| self.levels_centimetres.first() != Some(&0)
		{
			return false;
		}

		let mut previous = -1;

		for &level in &self.levels_centimetres {
			if i64::from(level) <= previous {
				return false;
			}

			previous = i64::from(level);
		}

		self.cells.iter().all(|&byte| {
			match RoomCell::from_byte(byte) {
				None => false,
				Some(cell) => {
					match cell.kind {
						RoomCellKind::Walkable | RoomCellKind::Hidden => {
							usize::from(cell.payload) < self.levels_centimetres.len()
						}
						_ => true,
					}
				}
			}
		})
	}

	/// Writes the grid as bytes: a header, the level table, then
	/// run-length encoded cells.
	///
	/// The encoding is a plain run length code. It uses no zlib and no
	/// other helper, so the same bytes are made on the phone, on the
	/// server and in a browser.
	pub fn encode(&self) -> Vec<u8> {
		let mut out = Vec::with_capacity(10 + self.levels_centimetres.len() + self.cells.len() * 3);

		out.push(Self::VERSION);
		out.extend_from_slice(&self.eye_height_centimetres.to_le_bytes());
		out.push(self.pivot_arm_centimetres);
		out.push(self.grid_yaw);
		out.push(self.width);
		out.push(self.depth);
		out.push(self.player_x);
		out.push(self.player_z);
		out.push(self.levels_centimetres.len() as u8);
		out.extend_from_slice(&self.levels_centimetres);

		let mut run_start = 0;

		for index in 1..=self.cells.len() {
			if index < self.cells.len() && self.cells[index] == self.cells[run_start] {
				continue;
			}

			let run = (index - run_start) as u16;

			out.extend_from_slice(&run.to_le_bytes());
			out.push(self.cells[run_start]);
			run_start = index;
		}

		out
	}

	/// Reads a grid written by [`RoomGrid::encode`].
	///
	/// Anything malformed is refused with a [`FormatError`]: a room never
	/// becomes a few plausible defaults.
	pub fn decode(bytes: &[u8]) -> Result<Self, FormatError> {
		if bytes.len() < 10 {
			return Err(FormatError::new("a room grid needs a ten-byte header"));
		}

		if bytes[0] != Self::VERSION {
			return Err(FormatError::with_input(
				"unknown room grid version",
				bytes[0].to_string(),
			));
		}

		let eye = u16::from_le_bytes([bytes[1], bytes[2]]);
		let width = bytes[5];
		let depth = bytes[6];
		let level_count = usize::from(bytes[9]);

		if level_count == 0 || level_count > usize::from(RoomCell::MAX_LEVEL) {
			return Err(FormatError::with_input(
				"a room grid carries this many levels",
				bytes[9].to_string(),
			));
		}

		let mut at = 10;

		if at + level_count > bytes.len() {
			return Err(FormatError::new("a room grid ends in its level table"));
		}

		let levels = bytes[at..at + level_count].to_vec();
		at += level_count;

		let cell_count = usize::from(width) * usize::from(depth);
		let mut cells = vec![0_u8; cell_count];
		let mut filled = 0;

		while filled < cell_count {
			if at + 3 > bytes.len() {
				return Err(FormatError::new("a room grid ends before its cells do"));
			}

			let run = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
			let cell = bytes[at + 2];
			at += 3;

			if run == 0 || usize::from(run) > cell_count - filled {
				return Err(FormatError::new("a room grid has a bad run length"));
			}

			for slot in &mut cells[filled..filled + usize::from(run)] {
				*slot = cell;
			}

			filled += usize::from(run);
		}

		if at != bytes.len() {
			return Err(FormatError::new("a room grid has bytes after its cells"));
		}

		let grid = Self::new(
			eye, bytes[3], bytes[4], width, depth, bytes[7], bytes[8], levels, cells,
		);

		if !grid.is_well_formed() {
			return Err(FormatError::new("a room grid is not well formed"));
		}

		Ok(grid)
	}
}

// The turn's cosine and sine are never NaN, so equality is an
// equivalence. They follow from `grid_yaw`, so the hash leaves them out.
impl Eq for RoomGrid {}

impl core::hash::Hash for RoomGrid {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		self.eye_height_centimetres.hash(state);
		self.pivot_arm_centimetres.hash(state);
		self.grid_yaw.hash(state);
		self.width.hash(state);
		self.depth.hash(state);
		self.player_x.hash(state);
		self.player_z.hash(state);
		self.levels_centimetres.hash(state);
		self.cells.hash(state);
	}
}

impl core::fmt::Display for RoomGrid {
	fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(
			formatter,
			"RoomGrid({}×{}, eye {} cm, {} levels)",
			self.width,
			self.depth,
			self.eye_height_centimetres,
			self.levels_centimetres.len()
		)
	}
}
