//! The room grid: its cells, its turn, and its bytes.

mod support;

use reapers_rules::FormatError;
use reapers_rules::RoomCell;
use reapers_rules::RoomCellKind;
use reapers_rules::RoomGrid;
use support::room;
use support::room::heights;

#[test]
fn every_drawn_room_encodes_and_decodes_exactly() {
	let rooms: Vec<(&str, RoomGrid)> = vec![
		("empty floor", room::empty_floor_room()),
		("desk", room::desk_room()),
		("desk and chair", room::desk_and_chair_room()),
		("sofa", room::sofa_room()),
		("counter", room::counter_room()),
	];

	for (name, grid) in rooms {
		let bytes = grid.encode();
		let decoded = RoomGrid::decode(&bytes).unwrap_or_else(|error| {
			panic!("{name} did not read back: {error}");
		});

		println!("{name}: {} bytes", bytes.len());

		assert_eq!(decoded, grid);
		assert!(grid.is_well_formed());
		assert!(decoded.is_well_formed());
	}
}

#[test]
fn an_empty_floor_compresses_to_a_few_bytes() {
	let room = room::empty_floor_room();

	assert!(room.encode().len() < 32);
}

#[test]
fn encoded_sizes_stay_small() {
	let rooms: Vec<(&str, RoomGrid)> = vec![
		("empty floor", room::empty_floor_room()),
		("desk", room::desk_room()),
		("desk and chair", room::desk_and_chair_room()),
		("sofa", room::sofa_room()),
		("counter", room::counter_room()),
	];

	for (name, grid) in rooms {
		let bytes = grid.encode();

		// A furnished room is a couple of hundred cells of interest in
		// a grid of thousands; the runs keep the wire small.
		assert!(
			bytes.len() < 2048,
			"{name} encoded to {} bytes",
			bytes.len()
		);
	}
}

#[test]
fn a_cell_stores_its_kind_above_its_payload() {
	assert_eq!(RoomCell::walkable(3).byte(), (1 << 5) | 3);
	assert_eq!(RoomCell::hidden(2).byte(), (4 << 5) | 2);
	assert_eq!(RoomCell::obstacle(85).byte(), (2 << 5) | 0b1011);
	assert_eq!(RoomCell::WALL.byte(), 3 << 5);
	assert_eq!(RoomCell::UNKNOWN.byte(), 0);

	assert_eq!(
		RoomCell::from_byte(RoomCell::obstacle(85).byte())
			.unwrap()
			.top_centimetres(),
		88
	);
	assert_eq!(
		RoomCell::from_byte(RoomCell::walkable(3).byte())
			.unwrap()
			.level(),
		Some(3)
	);
	assert_eq!(RoomCell::from_byte(0xE0), None);
}

#[test]
fn a_wall_is_higher_than_any_leap() {
	let mut room = room::DrawnRoom::new(&[heights::FLOOR]);
	room.wall(10, 10, 1, 1);

	let grid = room.build();

	assert_eq!(grid.top_centimetres_at(10, 10), RoomGrid::WALL_CENTIMETRES);
	assert_eq!(grid.level_index_at(10, 10), -1);
	assert_eq!(grid.cell_at(10, 10).kind, RoomCellKind::Wall);
}

#[test]
fn world_points_and_cells_agree_including_a_turned_grid() {
	let straight = room::desk_room();
	let turned = room::desk_drawn().with_yaw(64).build();

	for grid in [&straight, &turned] {
		for z in 0..i64::from(grid.depth) {
			for x in 0..i64::from(grid.width) {
				let centre = grid.centre_of(x, z);

				assert_eq!(grid.cell_x_at(centre.x, centre.z), x);
				assert_eq!(grid.cell_z_at(centre.x, centre.z), z);
			}
		}
	}

	// The turned grid's axes are 90 degrees round from the straight one:
	// one cell "ahead" in the grid lands a cell to the right.
	let ahead = turned.centre_of(i64::from(turned.player_x), i64::from(turned.player_z) + 1);

	assert!((ahead.x - 0.1).abs() < 1e-12);
	assert!(ahead.z.abs() < 1e-12);
}

#[test]
fn outside_the_grid_is_blocked_since_no_room_was_read_there() {
	let grid = room::empty_floor_room();

	assert_eq!(grid.cell_at(-1, 0).kind, RoomCellKind::Unknown);
	assert_eq!(grid.top_centimetres_at(-1, 0), RoomGrid::WALL_CENTIMETRES);
	assert_eq!(grid.level_index_at(-1, 0), -1);
	assert_eq!(grid.top_centimetres_at(0, -1), RoomGrid::WALL_CENTIMETRES);
}

#[test]
fn a_level_index_outside_the_table_is_not_well_formed() {
	let grid = RoomGrid::new(
		145,
		0,
		0,
		2,
		2,
		0,
		0,
		vec![0],
		vec![
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(4).byte(),
		],
	);

	assert!(!grid.is_well_formed());
}

#[test]
fn a_truncated_grid_is_refused() {
	let bytes = room::desk_room().encode();

	assert!(matches!(
		RoomGrid::decode(&bytes[..bytes.len() - 1]),
		Err(FormatError { .. })
	));
	assert!(matches!(
		RoomGrid::decode(&[1, 2, 3]),
		Err(FormatError { .. })
	));
}

#[test]
fn a_grid_whose_cells_end_early_is_refused() {
	let bytes = room::desk_room().encode();
	let short = &bytes[..bytes.len() - 3];

	assert!(matches!(RoomGrid::decode(short), Err(FormatError { .. })));
}

#[test]
fn a_grid_with_bytes_after_its_cells_is_refused() {
	let mut bytes = room::desk_room().encode();

	bytes.push(0);

	assert!(matches!(RoomGrid::decode(&bytes), Err(FormatError { .. })));
}

#[test]
fn a_grid_of_an_unknown_version_is_refused() {
	let mut bytes = room::desk_room().encode();

	bytes[0] = 99;

	assert!(matches!(RoomGrid::decode(&bytes), Err(FormatError { .. })));
}

#[test]
fn a_zero_run_length_is_refused() {
	let mut bytes = room::desk_room().encode();
	let at = bytes.len() - 3;

	bytes[at] = 0;
	bytes[at + 1] = 0;

	assert!(matches!(RoomGrid::decode(&bytes), Err(FormatError { .. })));
}

#[test]
fn a_level_table_cut_short_is_refused() {
	let bytes = room::desk_room().encode();

	assert!(matches!(
		RoomGrid::decode(&bytes[..10]),
		Err(FormatError { .. })
	));
}

#[test]
fn a_grid_with_an_unknown_cell_kind_is_refused() {
	let mut bytes = room::desk_room().encode();

	// The first cell run's byte: kind 5 is no kind at all.
	let at = 10 + usize::from(bytes[9]) + 2;
	bytes[at] = 5 << 5;

	assert!(matches!(RoomGrid::decode(&bytes), Err(FormatError { .. })));
}

#[test]
fn every_level_height_survives_the_byte_it_travels_in() {
	let mut room = room::DrawnRoom::new(&[0, 45, 75, 90, 145]);

	room.platform(34, 45, 12, 6, 75);
	room.platform(10, 10, 4, 4, 45);

	let decoded = RoomGrid::decode(&room.build().encode()).unwrap();

	assert_eq!(decoded.levels_centimetres, vec![0, 45, 75, 90, 145]);
	assert_eq!(decoded.top_centimetres_at(35, 46), 75);
	assert_eq!(decoded.top_centimetres_at(11, 11), 45);
}

#[test]
fn the_grid_reads_its_serde_form_back_the_same() {
	let grid = room::desk_and_chair_room();
	let bytes = serde_json::to_vec(&grid).expect("a grid serialises");
	let read: RoomGrid = serde_json::from_slice(&bytes).expect("a grid deserialises");

	assert_eq!(read, grid);
	assert_eq!(
		read.top_centimetres_at(30, 46),
		grid.top_centimetres_at(30, 46)
	);
}

#[test]
fn a_grid_is_formed_of_ten_centimetre_cells() {
	assert_eq!(RoomGrid::CELL_CENTIMETRES, 10);
	assert_eq!(RoomGrid::CELL_METRES, 0.1);
	assert_eq!(RoomGrid::VERSION, 1);
}
