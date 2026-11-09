//! Reading drawn rooms into the room grid: the furniture comes out as the
//! floor, levels, obstacles and hidden floor a player would expect to walk
//! among.

use std::sync::OnceLock;
use std::time::Instant;

use reapers_rules::RoomCell;
use reapers_rules::RoomCellKind;
use reapers_rules::RoomGrid;
use reapers_rules::RoomNav;
use reapers_rules::RoomRegion;
use reapers_rules::Vessel;
use reapers_rules::degrees;
use reapers_rules::radians;
use reapers_world::Block;
use reapers_world::DrawnRoom;
use reapers_world::RoomGridBuilder;
use reapers_world::Shot;

/// The heights and pitches every room is read from: ten pictures around a
/// 120° arc, from a standing eye, as a sigil's survey would take them.
const YAWS: [f64; 5] = [-60.0, -30.0, 0.0, 30.0, 60.0];
const PITCHES: [f64; 2] = [-40.0, -15.0];

/// The desk of the rules' drawn room: 1.2 by 0.6 m at 0.75 m, 2.5 m ahead.
const DESK: Block = Block {
	left: -0.6,
	right: 0.6,
	near: 2.5,
	far: 3.1,
	height: 0.75,
};

/// A chair 0.4 by 0.4 m at 0.45 m, touching the desk's left end.
const CHAIR: Block = Block {
	left: -1.05,
	right: -0.65,
	near: 2.55,
	far: 2.95,
	height: 0.45,
};

/// A sofa seat 0.45 m high, 3.9 m wide and 0.8 m deep.
const SOFA_SEAT: Block = Block {
	left: -1.95,
	right: 1.95,
	near: 3.0,
	far: 3.8,
	height: 0.45,
};

/// The sofa's back: 0.88 m high and too thin to be a level of its own.
const SOFA_BACK: Block = Block {
	left: -1.95,
	right: 1.95,
	near: 3.8,
	far: 3.9,
	height: 0.88,
};

/// A kitchen counter 3 m wide and 0.6 m deep, 0.9 m high.
const COUNTER: Block = Block {
	left: -1.5,
	right: 1.5,
	near: 2.5,
	far: 3.1,
	height: 0.9,
};

fn desk_room() -> DrawnRoom {
	DrawnRoom::with(&[DESK])
}

fn desk_and_chair_room() -> DrawnRoom {
	DrawnRoom::with(&[DESK, CHAIR])
}

fn sofa_room() -> DrawnRoom {
	DrawnRoom {
		wall: 5.0,
		blocks: vec![SOFA_SEAT, SOFA_BACK],
		..DrawnRoom::default()
	}
}

fn counter_room() -> DrawnRoom {
	DrawnRoom::with(&[COUNTER])
}

/// Reads `room` from the ten pictures of a survey, each taken as `shot`
/// says.
fn read_with(room: &DrawnRoom, mut shot: impl FnMut(usize) -> Shot) -> RoomGrid {
	let mut builder = RoomGridBuilder::new();
	let mut picture = 0;

	for pitch in PITCHES {
		for yaw in YAWS {
			builder.add_frame(&room.picture_with(yaw, pitch, &shot(picture)));
			picture += 1;
		}
	}

	assert!(builder.frames() > 0, "the survey was charted");

	builder.finish()
}

/// Reads `room` from a perfect survey.
fn read(room: &DrawnRoom) -> RoomGrid {
	read_with(room, |_| Shot::default())
}

fn desk_grid() -> &'static RoomGrid {
	static GRID: OnceLock<RoomGrid> = OnceLock::new();

	GRID.get_or_init(|| read(&desk_room()))
}

fn desk_and_chair_grid() -> &'static RoomGrid {
	static GRID: OnceLock<RoomGrid> = OnceLock::new();

	GRID.get_or_init(|| read(&desk_and_chair_room()))
}

fn sofa_grid() -> &'static RoomGrid {
	static GRID: OnceLock<RoomGrid> = OnceLock::new();

	GRID.get_or_init(|| read(&sofa_room()))
}

fn counter_grid() -> &'static RoomGrid {
	static GRID: OnceLock<RoomGrid> = OnceLock::new();

	GRID.get_or_init(|| read(&counter_room()))
}

/// The navigation a husk of `vessel` gets in `grid`, at a typical 3 m and
/// never nearer than the v2 staging's 1.5 m.
fn nav_for(grid: &RoomGrid, vessel: Vessel) -> RoomNav {
	RoomNav::new(grid.clone(), vessel.locomotion(), 3.0, 1.5)
}

/// The level index whose height is within 5 cm of `metres`.
fn level_near(grid: &RoomGrid, metres: f64) -> Option<u8> {
	grid.levels_centimetres
		.iter()
		.position(|&centimetres| (f64::from(centimetres) / 100.0 - metres).abs() <= 0.05)
		.map(|index| index as u8)
}

/// Every region of `nav` whose surface stands at `metres`, within 5 cm.
fn regions_at(nav: &RoomNav, metres: f64) -> Vec<&RoomRegion> {
	nav.regions
		.iter()
		.filter(|region| (region.height_metres() - metres).abs() <= 0.05)
		.collect()
}

/// The biggest region standing at `metres`, which is the furniture itself
/// rather than a stray cell.
fn largest_region_at(nav: &RoomNav, metres: f64) -> &RoomRegion {
	regions_at(nav, metres)
		.into_iter()
		.max_by_key(|region| region.cells.len())
		.unwrap_or_else(|| panic!("no region at {metres} m"))
}

/// The smallest and largest cell column and row of `cells`.
fn bounds(grid: &RoomGrid, cells: &[usize]) -> (i64, i64, i64, i64) {
	let mut bounds = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);

	for &cell in cells {
		let (x, z) = (grid.x_of(cell), grid.z_of(cell));

		bounds.0 = bounds.0.min(x);
		bounds.1 = bounds.1.max(x);
		bounds.2 = bounds.2.min(z);
		bounds.3 = bounds.3.max(z);
	}

	bounds
}

/// Whether `nav` offers a leap from a surface at `from` metres to one at
/// `to` metres, within 5 cm of each.
fn has_link(nav: &RoomNav, from: f64, to: f64) -> bool {
	nav.links.iter().any(|link| {
		let leaves = nav.regions[link.from_region].height_metres();
		let lands = nav.regions[link.to_region].height_metres();

		(leaves - from).abs() <= 0.05 && (lands - to).abs() <= 0.05
	})
}

/// Whether any leap touches a surface at `metres`, within 5 cm.
fn has_link_at(nav: &RoomNav, metres: f64) -> bool {
	nav.links.iter().any(|link| {
		(nav.regions[link.from_region].height_metres() - metres).abs() <= 0.05
			|| (nav.regions[link.to_region].height_metres() - metres).abs() <= 0.05
	})
}

#[test]
fn the_desk_room_reads_a_floor_a_desk_and_the_floor_behind_it() {
	let grid = desk_grid();

	assert!(grid.is_well_formed());
	assert_eq!(grid.eye_height_centimetres, 145);
	assert_eq!(grid.levels_centimetres, vec![0, 75]);

	// The desk is the only thing at 0.75 m, and its top is walkable.
	let nav = nav_for(grid, Vessel::Ape);
	let desk = largest_region_at(&nav, 0.75);

	assert!(desk.is_standable(), "the ape keeps its feet on the desk");
	assert!(
		(66..=80).contains(&desk.cells.len()),
		"the desk holds {} cells, not the drawn 72",
		desk.cells.len()
	);

	let (left, right, front, back) = bounds(grid, &desk.cells);

	assert!(
		(right - left + 1 - 12).abs() <= 2,
		"the desk is {} cells across, not 12",
		right - left + 1
	);
	assert!(
		(back - front + 1 - 6).abs() <= 2,
		"the desk is {} cells deep, not 6",
		back - front + 1
	);

	// The floor directly behind the desk was never seen: the desk's far
	// edge is 3.1 m out and the room's wall 4 m, and no sight line from the
	// eye passes over the desk before it reaches the floor.
	let mut hidden = 0;

	for z in 0..i64::from(grid.depth) {
		for x in 0..i64::from(grid.width) {
			let point = grid.centre_of(x, z);

			if point.x.abs() <= 0.55 && (3.15..3.95).contains(&point.z) {
				let cell = grid.cell_at(x, z);

				assert_eq!(
					cell.kind,
					RoomCellKind::Hidden,
					"the floor behind the desk at ({}, {}) is not hidden",
					point.x,
					point.z
				);
				assert_eq!(cell.level(), Some(0), "hidden floor is the floor");
				hidden += 1;
			}
		}
	}

	assert_eq!(hidden, 88, "the whole strip behind the desk is hidden");
}

#[test]
fn an_empty_builder_reads_an_unknown_room() {
	let grid = RoomGridBuilder::new().finish();

	assert!(grid.is_well_formed());
	assert_eq!(grid.eye_height_centimetres, 145);
	assert_eq!(grid.levels_centimetres, vec![0]);
	assert!(grid.cells.iter().all(|&byte| {
		RoomCell::from_byte(byte).is_some_and(|cell| cell.kind == RoomCellKind::Unknown)
	}));
}

#[test]
fn the_desk_and_chair_room_reads_three_levels() {
	let grid = desk_and_chair_grid();

	assert!(grid.is_well_formed());
	assert_eq!(grid.eye_height_centimetres, 145);
	assert_eq!(grid.levels_centimetres.len(), 3);
	assert!(level_near(grid, 0.0).is_some());
	assert!(level_near(grid, 0.45).is_some());
	assert!(level_near(grid, 0.75).is_some());

	let nav = nav_for(grid, Vessel::Penguin);
	let chair = largest_region_at(&nav, 0.45);
	let desk = largest_region_at(&nav, 0.75);

	assert!(
		(12..=28).contains(&chair.cells.len()),
		"the chair holds {} cells, not the drawn 16",
		chair.cells.len()
	);
	assert!(chair.is_standable(), "the penguin fits on the chair");
	assert!(desk.is_standable());

	let (left, right, front, back) = bounds(grid, &chair.cells);

	assert!((right - left + 1 - 4).abs() <= 2, "the chair's width");
	assert!((back - front + 1 - 4).abs() <= 2, "the chair's depth");
}

#[test]
fn only_the_penguin_uses_the_chair() {
	let grid = desk_and_chair_grid();

	// The penguin reaches the desk through the chair: up 0.45 m, then
	// across and up again, never more than its 0.5 m jump.
	let penguin = nav_for(grid, Vessel::Penguin);

	assert!(has_link(&penguin, 0.0, 0.45), "floor to chair");
	assert!(has_link(&penguin, 0.45, 0.75), "chair to desk");
	assert!(has_link(&penguin, 0.45, 0.0), "chair to floor");
	assert!(has_link(&penguin, 0.75, 0.45), "desk to chair");
	assert!(!has_link(&penguin, 0.0, 0.75), "no floor to desk in one go");

	// The ape jumps the whole 0.75 m, and its feet are too wide for the
	// chair's middle.
	let ape = nav_for(grid, Vessel::Ape);

	assert!(has_link(&ape, 0.0, 0.75), "floor to desk");
	assert!(has_link(&ape, 0.75, 0.0), "desk to floor");
	assert!(!has_link_at(&ape, 0.45), "the ape never touches the chair");
}

#[test]
fn the_sofa_room_reads_its_seat_and_its_back() {
	let grid = sofa_grid();

	assert!(grid.is_well_formed());
	assert!(level_near(grid, 0.45).is_some());

	let nav = nav_for(grid, Vessel::Penguin);
	let seat = largest_region_at(&nav, 0.45);

	assert!(seat.is_standable(), "the penguin fits on the sofa's seat");
	assert!(
		(296..=320).contains(&seat.cells.len()),
		"the seat holds {} cells, not the drawn 312",
		seat.cells.len()
	);

	let (left, right, front, back) = bounds(grid, &seat.cells);

	assert!((right - left + 1 - 39).abs() <= 2, "the seat's width");
	assert!((back - front + 1 - 8).abs() <= 2, "the seat's depth");

	// The back is 0.88 m high and one 10 cm cell deep, so it never becomes
	// a level; it is an obstacle over the seat's width.
	let mut back_cells = 0;

	for z in 0..i64::from(grid.depth) {
		for x in 0..i64::from(grid.width) {
			let point = grid.centre_of(x, z);

			if point.x.abs() <= 1.85 && (3.75..3.85).contains(&point.z) {
				let cell = grid.cell_at(x, z);

				assert_ne!(
					cell.kind,
					RoomCellKind::Walkable,
					"the sofa's back is not a surface to stand on"
				);

				if cell.kind == RoomCellKind::Obstacle || cell.kind == RoomCellKind::Wall {
					assert!(
						(80..=96).contains(&grid.top_centimetres_at(x, z)),
						"the back's top is {} cm, not about 88",
						grid.top_centimetres_at(x, z)
					);
					back_cells += 1;
				}
			}
		}
	}

	assert!(
		back_cells >= 34,
		"only {back_cells} cells of the sofa's back are read"
	);
}

#[test]
fn the_counter_room_reads_a_ninety_centimetre_top() {
	let grid = counter_grid();

	assert!(grid.is_well_formed());
	assert!(level_near(grid, 0.9).is_some());

	let ape = nav_for(grid, Vessel::Ape);
	let counter = largest_region_at(&ape, 0.91);

	assert!(
		counter.is_standable(),
		"the ape keeps its feet on the counter"
	);
	assert!(
		(168..=188).contains(&counter.cells.len()),
		"the counter holds {} cells, not the drawn 180",
		counter.cells.len()
	);

	let (left, right, front, back) = bounds(grid, &counter.cells);

	assert!((right - left + 1 - 30).abs() <= 2, "the counter's width");
	assert!((back - front + 1 - 6).abs() <= 2, "the counter's depth");

	// A 0.9 m top is one bound up for an ape, and beyond a penguin.
	assert!(has_link(&ape, 0.0, 0.9), "the ape clears the counter");
	assert!(has_link(&ape, 0.9, 0.0), "and drops back down");

	let penguin = nav_for(grid, Vessel::Penguin);

	assert!(!has_link(&penguin, 0.0, 0.9), "the penguin cannot clear it");
}

#[test]
fn noise_and_flying_pixels_leave_the_desk_level_clean() {
	// Every picture reads the whole room 7% long or short in turn, and the
	// desk's silhouette is smeared one pixel into what lies behind it.
	let grid = read_with(&desk_room(), |picture| {
		Shot {
			stretch: 1.0 + if picture % 2 == 0 { 0.07 } else { -0.07 },
			flying: 1,
			seed: picture as i64 + 1,
			..Shot::default()
		}
	});

	assert!(grid.is_well_formed());
	assert_eq!(grid.eye_height_centimetres, 145, "the floors average out");
	assert!(level_near(&grid, 0.75).is_some(), "the desk still snaps");

	// Directly past the desk's edge, nothing stands between the floor and
	// the desk's top: the smeared pixels did not grow a ramp.
	for z in 0..i64::from(grid.depth) {
		for x in 0..i64::from(grid.width) {
			let point = grid.centre_of(x, z);

			if point.x.abs() <= 0.8 && (3.15..3.9).contains(&point.z) {
				let top = grid.top_centimetres_at(x, z);

				assert!(
					!(10..65).contains(&top),
					"a ramp at ({}, {}) stands {top} cm high",
					point.x,
					point.z
				);
			}
		}
	}

	// And the desk itself is still a desk.
	let nav = nav_for(&grid, Vessel::Ape);
	let desk = largest_region_at(&nav, 0.75);
	let (left, right, front, back) = bounds(&nav.grid, &desk.cells);

	assert!((right - left + 1 - 12).abs() <= 2, "the desk's width");
	assert!((back - front + 1 - 6).abs() <= 2, "the desk's depth");
	assert!(has_link(&nav, 0.0, 0.75));
}

#[test]
fn a_turned_room_turns_the_grid_to_its_axes() {
	let room = DrawnRoom {
		room_yaw: radians(20.0),
		..desk_room()
	};
	let grid = read(&room);

	assert!(grid.is_well_formed());
	assert!(
		(degrees(grid.yaw()) - 20.0).abs() <= 1.5,
		"the grid turned {:?}°, not about 20",
		degrees(grid.yaw())
	);

	let nav = nav_for(&grid, Vessel::Ape);
	let desk = largest_region_at(&nav, 0.75);
	let (left, right, front, back) = bounds(&nav.grid, &desk.cells);

	assert!(desk.is_standable());
	assert!(
		(right - left + 1 - 12).abs() <= 1,
		"the desk is {} cells across its own axes, not 12",
		right - left + 1
	);
	assert!(
		(back - front + 1 - 6).abs() <= 1,
		"the desk is {} cells deep its own axes, not 6",
		back - front + 1
	);
	assert!(has_link(&nav, 0.0, 0.75));
}

#[test]
fn each_room_encodes_in_under_two_kilobytes() {
	for (name, grid) in [
		("desk", desk_grid()),
		("desk and chair", desk_and_chair_grid()),
		("sofa", sofa_grid()),
		("counter", counter_grid()),
	] {
		let bytes = grid.encode();

		println!("{name}: {} bytes", bytes.len());
		assert!(
			bytes.len() < 2048,
			"{name} encodes to {} bytes",
			bytes.len()
		);
		assert_eq!(
			RoomGrid::decode(&bytes).expect("the room reads back"),
			*grid,
			"{name} keeps its bytes"
		);
	}
}

#[test]
fn timing_on_this_mac() {
	let room = desk_and_chair_room();
	let mut frames = Vec::new();
	let mut picture = 0;

	for pitch in PITCHES {
		for yaw in YAWS {
			frames.push(room.picture_with(
				yaw,
				pitch,
				&Shot {
					seed: picture + 1,
					..Shot::default()
				},
			));
			picture += 1;
		}
	}

	let mut builder = RoomGridBuilder::new();
	let start = Instant::now();

	for frame in &frames {
		builder.add_frame(frame);
	}

	let added = start.elapsed();
	let start = Instant::now();
	let grid = builder.finish();
	let finished = start.elapsed();

	println!(
		"room grid: {} pictures, {:?} per add_frame, {:?} for finish, {} cells",
		frames.len(),
		added / frames.len() as u32,
		finished,
		grid.cell_count()
	);

	assert!(grid.is_well_formed());
}
