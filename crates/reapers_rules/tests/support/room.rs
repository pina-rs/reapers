//! Rooms drawn by hand, cell by cell, for the room tests.
//!
//! The player stands at cell (40, 20) of an 80 by 60 grid: 2 m from the
//! near edge, with 4 m ahead. One cell is 10 cm, so a desk 2.5 m ahead
//! starts at row 45.
//!
//! Drawn from the research record's furniture: a 0.75 m desk, a 0.45 m
//! chair beside it, a 0.45 m sofa with an 0.88 m back, and a 0.90 m
//! counter.

use reapers_rules::Leap;
use reapers_rules::RoomCell;
use reapers_rules::RoomGrid;
use reapers_rules::RoomNav;
use reapers_rules::RoomRegion;
use reapers_rules::RoomSpot;
use reapers_rules::RoomSpotKind;
use reapers_rules::Vec3;
use reapers_rules::Vessel;

/// The heights the drawn rooms use, in centimetres above the floor.
pub mod heights {
	pub const FLOOR: i64 = 0;
	pub const CHAIR: i64 = 45;
	pub const DESK: i64 = 75;
	pub const COUNTER: i64 = 90;

	/// The sofa's back, rounded to the 8 cm an obstacle top can store.
	pub const SOFA_BACK: i64 = 88;
}

/// A room drawn by hand, one piece of furniture at a time.
pub struct DrawnRoom {
	width: u8,
	depth: u8,
	eye_height_centimetres: u16,
	grid_yaw: u8,
	levels: Vec<u8>,
	player_x: u8,
	player_z: u8,
	cells: Vec<u8>,
}

impl DrawnRoom {
	/// An empty floor, `levels` are its level heights in centimetres and
	/// begin at the floor.
	pub fn new(levels: &[i64]) -> Self {
		Self::sized(80, 60, levels)
	}

	/// An empty floor of a size of its own.
	pub fn sized(width: u8, depth: u8, levels: &[i64]) -> Self {
		assert!(
			!levels.is_empty() && levels[0] == 0,
			"levels begin at the floor"
		);

		Self {
			width,
			depth,
			eye_height_centimetres: 145,
			grid_yaw: 0,
			levels: levels.iter().map(|&level| level as u8).collect(),
			player_x: width / 2,
			player_z: depth / 3,
			cells: vec![RoomCell::walkable(0).byte(); usize::from(width) * usize::from(depth)],
		}
	}

	/// Turns the grid away from the hunt's own axes.
	pub fn with_yaw(mut self, grid_yaw: u8) -> Self {
		self.grid_yaw = grid_yaw;
		self
	}

	/// Puts the player's cell somewhere else.
	pub fn with_player(mut self, x: u8, z: u8) -> Self {
		self.player_x = x;
		self.player_z = z;
		self
	}

	/// Fills a rectangle with a level top, in centimetres above the
	/// floor.
	pub fn platform(&mut self, x: i64, z: i64, width: i64, depth: i64, height_centimetres: i64) {
		let level = self.level_of(height_centimetres);
		self.fill(x, z, width, depth, RoomCell::walkable(level).byte());
	}

	/// Fills a rectangle with floor the scan knows is there but cannot
	/// see.
	pub fn hidden(&mut self, x: i64, z: i64, width: i64, depth: i64, height_centimetres: i64) {
		let level = self.level_of(height_centimetres);
		self.fill(x, z, width, depth, RoomCell::hidden(level).byte());
	}

	/// Fills a rectangle with something standing in the room.
	pub fn obstacle(&mut self, x: i64, z: i64, width: i64, depth: i64, height_centimetres: i64) {
		self.fill(
			x,
			z,
			width,
			depth,
			RoomCell::obstacle(height_centimetres).byte(),
		);
	}

	/// Fills a rectangle with nothing known.
	pub fn unknown(&mut self, x: i64, z: i64, width: i64, depth: i64) {
		self.fill(x, z, width, depth, RoomCell::UNKNOWN.byte());
	}

	/// Fills a rectangle with something too tall to pass.
	pub fn wall(&mut self, x: i64, z: i64, width: i64, depth: i64) {
		self.fill(x, z, width, depth, RoomCell::WALL.byte());
	}

	/// The grid the room describes.
	pub fn build(&self) -> RoomGrid {
		RoomGrid::new(
			self.eye_height_centimetres,
			0,
			self.grid_yaw,
			self.width,
			self.depth,
			self.player_x,
			self.player_z,
			self.levels.clone(),
			self.cells.clone(),
		)
	}

	fn level_of(&self, height_centimetres: i64) -> u8 {
		self.levels
			.iter()
			.position(|&level| i64::from(level) == height_centimetres)
			.unwrap_or_else(|| panic!("{height_centimetres} cm is not one of the room's levels"))
			as u8
	}

	fn fill(&mut self, x: i64, z: i64, width: i64, depth: i64, byte: u8) {
		for dz in 0..depth {
			for dx in 0..width {
				let at_x = x + dx;
				let at_z = z + dz;

				assert!(
					self.contains(at_x, at_z),
					"a drawn cell falls outside the room"
				);

				self.cells[at_z as usize * usize::from(self.width) + at_x as usize] = byte;
			}
		}
	}

	fn contains(&self, x: i64, z: i64) -> bool {
		x >= 0 && x < i64::from(self.width) && z >= 0 && z < i64::from(self.depth)
	}
}

/// An empty floor.
pub fn empty_floor_room() -> RoomGrid {
	DrawnRoom::new(&[heights::FLOOR]).build()
}

/// A desk with a 1.2 by 0.6 m top at 0.75 m, its front 2.5 m ahead.
///
/// The cells are x 34 to 45 and z 45 to 50. The player's cell is (40, 20).
pub fn desk_drawn() -> DrawnRoom {
	let mut room = DrawnRoom::new(&[heights::FLOOR, heights::DESK]);

	room.platform(34, 45, 12, 6, heights::DESK);
	room
}

pub fn desk_room() -> RoomGrid {
	desk_drawn().build()
}

/// The desk, with a chair seat 0.4 by 0.4 m at 0.45 m beside its left
/// end.
///
/// The chair touches the desk along x = 33/34, so a penguin can go
/// floor, chair, desk without ever jumping higher than its 0.5 m.
pub fn desk_and_chair_drawn() -> DrawnRoom {
	let mut room = DrawnRoom::new(&[heights::FLOOR, heights::CHAIR, heights::DESK]);

	room.platform(34, 45, 12, 6, heights::DESK);
	room.platform(30, 46, 4, 4, heights::CHAIR);
	room
}

pub fn desk_and_chair_room() -> RoomGrid {
	desk_and_chair_drawn().build()
}

/// A sofa 3.6 m ahead: a 0.45 m seat, an 0.88 m back, and floor seen
/// behind it, marked hidden where the back hides it.
pub fn sofa_room() -> RoomGrid {
	let mut room = DrawnRoom::sized(80, 80, &[heights::FLOOR, heights::CHAIR]).with_player(40, 20);

	room.platform(20, 50, 40, 8, heights::CHAIR);
	room.obstacle(20, 58, 40, 4, heights::SOFA_BACK);
	room.hidden(20, 62, 40, 4, heights::FLOOR);
	room.build()
}

/// A kitchen counter with a 0.9 m top, 2.5 m ahead.
pub fn counter_room() -> RoomGrid {
	let mut room = DrawnRoom::new(&[heights::FLOOR, heights::COUNTER]);

	room.platform(20, 46, 40, 6, heights::COUNTER);
	room.build()
}

/// The desk room with a 2.24 m column between the player and the desk,
/// to make a bird go round it.
pub fn desk_with_column_room() -> RoomGrid {
	let mut room = desk_drawn();

	room.obstacle(38, 33, 4, 4, 224);
	room.build()
}

/// A room cut in two by a wall straight across, 3 m ahead.
pub fn walled_room() -> RoomGrid {
	let mut room = DrawnRoom::new(&[heights::FLOOR]);

	room.wall(0, 50, 80, 1);
	room.build()
}

/// A room navigation for `vessel` in `grid`, at a typical 3 m, never
/// nearer the player than the v2 staging's 1.5 m.
pub fn nav_for(grid: RoomGrid, vessel: Vessel) -> RoomNav {
	RoomNav::new(grid, vessel.locomotion(), 3.0, 1.5)
}

/// Whether the room offers a leap from one surface height to another.
pub fn has_link(nav: &RoomNav, from_height: i64, to_height: i64) -> bool {
	nav.links.iter().any(|link| {
		nav.regions[link.from_region].height_centimetres == from_height
			&& nav.regions[link.to_region].height_centimetres == to_height
	})
}

/// The regions that stand at `height_centimetres`.
pub fn regions_at(nav: &RoomNav, height_centimetres: i64) -> Vec<&RoomRegion> {
	nav.regions
		.iter()
		.filter(|region| region.height_centimetres == height_centimetres)
		.collect()
}

/// How far the floor is below the player's eye, in metres.
pub fn floor_height() -> f64 {
	-f64::from(reapers_rules::HuntRules::EYE_CENTIMETRES) / 100.0
}

/// The height of `height_centimetres` above the floor, in the hunt's
/// frame.
pub fn height_of(height_centimetres: i64) -> f64 {
	floor_height() + height_centimetres as f64 / 100.0
}

/// A spot list filtered to one kind, for the tests' eyes.
pub fn spots_of(nav: &RoomNav, kind: RoomSpotKind) -> Vec<RoomSpot> {
	nav.spots_of(kind).into_iter().copied().collect()
}

/// The standable cell nearest a point, as the navigation snaps to it.
pub fn standable_near(nav: &RoomNav, point: Vec3) -> usize {
	nav.standable_cell_near(point, 20)
		.expect("a standable cell nearby")
}

/// A leap between two body centres over the drawn desk room, or nothing
/// when the room refuses it. The heights are above the floor.
pub fn leap_over_desk_room(from: (f64, f64), to: (f64, f64)) -> Option<Leap> {
	let ape = Vessel::Ape.locomotion();

	Leap::solve(
		Vec3::new(0.0, floor_height() + ape.half_height() + from.0, from.1),
		Vec3::new(0.0, floor_height() + ape.half_height() + to.0, to.1),
		&desk_room(),
		ape.half_height(),
		None,
	)
}
