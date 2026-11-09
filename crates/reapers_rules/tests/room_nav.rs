//! The room navigation: standable cells, regions, links, spots, paths.

mod support;

use reapers_rules::RoomSpotKind;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use support::room;
use support::room::heights;

#[test]
fn a_vessels_locomotion_is_the_creature_tables() {
	let ape = Vessel::Ape.locomotion();

	assert_eq!(ape.body_height, 0.60);
	assert_eq!(ape.foot_radius, 0.18);
	assert_eq!(ape.jumps_up, 1.00);
	assert_eq!(ape.drops_down, 1.50);
	assert_eq!(ape.leaps_across, 1.2);
	assert_eq!(ape.walk_speed, Vessel::Ape.walk_speed());
	assert_eq!(ape.run_speed, Vessel::Ape.bolt_speed());
	assert!(!ape.flies());

	let penguin = Vessel::Penguin.locomotion();

	assert_eq!(penguin.jumps_up, 0.50);
	assert_eq!(penguin.drops_down, 0.90);
	assert_eq!(penguin.leaps_across, 0.6);
	assert!(!penguin.flies());

	let llama = Vessel::Llama.locomotion();

	assert_eq!(llama.jumps_up, 0.90);
	assert_eq!(llama.drops_down, 1.40);
	assert_eq!(llama.leaps_across, 1.1);

	let robot = Vessel::Robot.locomotion();

	assert_eq!(robot.jumps_up, 0.80);
	assert_eq!(robot.drops_down, 1.30);
	assert_eq!(robot.leaps_across, 0.9);

	let bird = Vessel::Bird.locomotion();

	assert_eq!(bird.body_height, 0.34);
	assert_eq!(bird.foot_radius, 0.12);
	assert!(bird.flies());
	assert_eq!(bird.half_height(), 0.17);
}

#[test]
fn an_ape_can_stand_on_the_desk_and_keeps_its_feet_on_it() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let desk = &room::regions_at(&nav, heights::DESK)[0];

	assert!(desk.is_standable());
	assert!(!desk.standable_cells.is_empty(), "the desk keeps its feet");

	let (left, right) = (-0.65, 0.55);
	let (front, back) = (2.45, 3.05);
	let radius = Vessel::Ape.locomotion().foot_radius;

	for &cell in &desk.standable_cells {
		let point = nav.grid.centre_of(nav.grid.x_of(cell), nav.grid.z_of(cell));
		let nearest_edge = (point.x - left)
			.min(right - point.x)
			.min(point.z - front)
			.min(back - point.z);

		assert!(
			nearest_edge + 1e-9 >= radius,
			"cell {cell} at {}, {} keeps only {nearest_edge} of its {} m foot",
			point.x,
			point.z,
			radius
		);
	}
}

#[test]
fn the_floor_and_desk_are_two_regions_and_the_ape_leaps_between_them() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);

	assert!(room::has_link(&nav, heights::FLOOR, heights::DESK));
	assert!(room::has_link(&nav, heights::DESK, heights::FLOOR));

	// Each link carries its solved arc.
	for link in &nav.links {
		assert!(link.ticks() > 0);
		assert!(link.gap_metres <= Vessel::Ape.locomotion().leaps_across + 1e-9);
	}
}

#[test]
fn a_penguin_cannot_reach_the_desk_directly_but_can_through_a_chair() {
	let nav = room::nav_for(room::desk_and_chair_room(), Vessel::Penguin);

	assert!(!room::has_link(&nav, heights::FLOOR, heights::DESK));
	assert!(room::has_link(&nav, heights::FLOOR, heights::CHAIR));
	assert!(room::has_link(&nav, heights::CHAIR, heights::DESK));

	let path = nav
		.path(Vec3::new(-0.5, 0.0, 1.2), Vec3::new(0.3, 0.0, 2.8))
		.expect("the penguin reaches the desk through the chair");

	let ups: Vec<i64> = path
		.leaps()
		.map(|link| nav.regions[link.to_region].height_centimetres)
		.collect();

	assert_eq!(ups, vec![heights::CHAIR, heights::DESK]);
}

#[test]
fn an_ape_goes_straight_up_without_a_chair() {
	let nav = room::nav_for(room::desk_and_chair_room(), Vessel::Ape);
	let path = nav
		.path(Vec3::new(-0.5, 0.0, 1.2), Vec3::new(0.3, 0.0, 2.8))
		.expect("the ape reaches the desk in one bound");

	let ups: Vec<i64> = path
		.leaps()
		.map(|link| nav.regions[link.to_region].height_centimetres)
		.collect();

	assert_eq!(ups.len(), 1);
	assert_eq!(ups[0], heights::DESK);
}

#[test]
fn a_surface_too_small_to_be_a_region_is_dropped() {
	let mut drawn = room::DrawnRoom::new(&[heights::FLOOR, heights::DESK]);
	drawn.platform(10, 10, 2, 2, heights::DESK);

	let nav = room::nav_for(drawn.build(), Vessel::Ape);

	assert!(
		room::regions_at(&nav, heights::DESK).is_empty(),
		"two cells of desk are no surface to stand on"
	);
}

#[test]
fn a_perch_is_an_edge_of_a_raised_surface_turned_to_the_player() {
	let nav = room::nav_for(room::desk_and_chair_room(), Vessel::Ape);
	let perches = room::spots_of(&nav, RoomSpotKind::Perch);

	assert!(!perches.is_empty(), "a raised edge is a perch");

	for spot in &perches {
		let region = nav.region_of(spot.cell).expect("a perch sits in a region");

		assert!(region.height_centimetres > 0);
		assert!(spot.facing.is_some());
		assert!(
			(spot.point.y - (region.height_metres() + room::floor_height())).abs() < 1e-9,
			"a perch stands on its surface"
		);
	}
}

#[test]
fn open_floor_is_floor_the_player_can_see_inside_the_husks_distances() {
	let nav = room::nav_for(room::sofa_room(), Vessel::Penguin);
	let open = room::spots_of(&nav, RoomSpotKind::Open);

	assert!(!open.is_empty(), "the floor ahead is in view");

	for spot in &open {
		let cell = nav
			.grid
			.cell_at(nav.grid.x_of(spot.cell), nav.grid.z_of(spot.cell));

		assert_eq!(cell.kind, reapers_rules::RoomCellKind::Walkable);
		assert_eq!(spot.level_index, 0);

		let distance = (spot.point.x * spot.point.x + spot.point.z * spot.point.z).sqrt();

		assert!(distance >= nav.near_metres - 1e-9);
		assert!(distance <= nav.range + 1e-9);
	}
}

#[test]
fn a_penguin_finds_hidden_places_to_peek_over_the_sofa() {
	let nav = room::nav_for(room::sofa_room(), Vessel::Penguin);
	let peeks = room::spots_of(&nav, RoomSpotKind::Peek);

	assert!(
		!peeks.is_empty(),
		"the hidden floor behind the sofa gives a peek"
	);

	for spot in &peeks {
		let cell = nav
			.grid
			.cell_at(nav.grid.x_of(spot.cell), nav.grid.z_of(spot.cell));

		assert_eq!(cell.kind, reapers_rules::RoomCellKind::Hidden);
		assert!(spot.facing.is_some());
	}
}

#[test]
fn a_deep_hidden_floor_gives_a_penguin_somewhere_to_hide() {
	// The sofa room, with the floor seen behind it twelve cells deep
	// instead of four.
	let mut room_drawn =
		room::DrawnRoom::sized(80, 90, &[heights::FLOOR, heights::CHAIR]).with_player(40, 20);

	room_drawn.platform(20, 50, 40, 8, heights::CHAIR);
	room_drawn.obstacle(20, 58, 40, 4, heights::SOFA_BACK);
	room_drawn.hidden(20, 62, 40, 12, heights::FLOOR);

	let nav = room::nav_for(room_drawn.build(), Vessel::Penguin);
	let hides = room::spots_of(&nav, RoomSpotKind::Hide);

	assert!(
		!hides.is_empty(),
		"a deep hidden floor gives a hiding place"
	);

	for spot in &hides {
		let cell = nav
			.grid
			.cell_at(nav.grid.x_of(spot.cell), nav.grid.z_of(spot.cell));

		assert_eq!(cell.kind, reapers_rules::RoomCellKind::Hidden);
		assert!(nav.is_standable(spot.cell));
	}
}

#[test]
fn nothing_paths_through_a_wall() {
	let nav = room::nav_for(room::walled_room(), Vessel::Penguin);

	assert!(
		nav.path(Vec3::new(0.0, 0.0, 1.2), Vec3::new(0.0, 0.0, 4.0))
			.is_none(),
		"no route crosses the wall"
	);
}

#[test]
fn a_path_walks_round_an_obstacle_never_through_it() {
	let nav = room::nav_for(room::desk_with_column_room(), Vessel::Ape);
	let path = nav
		.path(Vec3::new(-0.9, 0.0, 0.8), Vec3::new(0.3, 0.0, 2.8))
		.expect("the column does not close the room");

	for step in &path.steps {
		let (x, z) = (nav.grid.x_of(step.cell), nav.grid.z_of(step.cell));
		let kind = nav.grid.cell_at(x, z).kind;

		assert_ne!(kind, reapers_rules::RoomCellKind::Obstacle);
		assert_ne!(kind, reapers_rules::RoomCellKind::Wall);
		assert!(nav.is_standable(step.cell));
	}
}

#[test]
fn a_path_is_the_same_every_time_and_from_a_freshly_built_room() {
	let drawn = room::desk_and_chair_drawn();
	let first = room::nav_for(drawn.build(), Vessel::Penguin);
	let second = room::nav_for(drawn.build(), Vessel::Penguin);
	let from = Vec3::new(-0.5, 0.0, 1.2);
	let to = Vec3::new(0.3, 0.0, 2.8);

	let one = first.path(from, to).expect("a route");
	let other = first.path(from, to).expect("a route");
	let fresh = second.path(from, to).expect("a route");

	let cells: Vec<usize> = one.steps.iter().map(|step| step.cell).collect();
	let other_cells: Vec<usize> = other.steps.iter().map(|step| step.cell).collect();
	let fresh_cells: Vec<usize> = fresh.steps.iter().map(|step| step.cell).collect();

	assert_eq!(cells, other_cells);
	assert_eq!(cells, fresh_cells);

	for index in 0..one.steps.len() {
		assert_eq!(one.steps[index].point, other.steps[index].point);
		assert_eq!(one.steps[index].point, fresh.steps[index].point);
	}

	let links: Vec<(usize, usize)> = first
		.links
		.iter()
		.map(|link| (link.from_cell, link.to_cell))
		.collect();
	let other_links: Vec<(usize, usize)> = second
		.links
		.iter()
		.map(|link| (link.from_cell, link.to_cell))
		.collect();

	assert_eq!(links, other_links);
}

#[test]
fn a_turned_grid_finds_the_same_desk() {
	let grid = room::desk_drawn().with_yaw(64).build();
	let nav = room::nav_for(grid, Vessel::Ape);
	let path = nav
		.path(nav.grid.centre_of(35, 32), nav.grid.centre_of(43, 48))
		.expect("the turned room still holds the desk");

	assert!(path.leaps().next().is_some());
}

#[test]
fn a_point_over_a_desk_edge_is_snapped_to_a_standable_cell() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let edge = nav.grid.centre_of(34, 47);
	let cell = room::standable_near(&nav, edge);

	assert!(nav.is_standable(cell));
	assert_eq!(
		nav.region_of(cell).expect("a region").height_centimetres,
		heights::DESK
	);
}

#[test]
fn every_vessel_finds_the_floor_and_the_desk_in_the_furnished_room() {
	// The chair seat is 0.4 m across: only the light-footed can stand in
	// the middle of it with their feet clear of every edge.
	let chair_fits = |vessel: Vessel| vessel.locomotion().foot_radius <= 0.15;

	for vessel in [Vessel::Ape, Vessel::Penguin, Vessel::Llama, Vessel::Robot] {
		let nav = room::nav_for(room::desk_and_chair_room(), vessel);
		let heights: Vec<i64> = nav
			.regions
			.iter()
			.filter(|region| region.is_standable())
			.map(|region| region.height_centimetres)
			.collect();

		assert!(heights.contains(&heights::FLOOR), "{vessel} lost the floor");
		assert!(heights.contains(&heights::DESK), "{vessel} lost the desk");

		let chair = heights.contains(&heights::CHAIR);

		assert_eq!(
			chair,
			chair_fits(vessel),
			"{vessel} {} the chair",
			if chair {
				"stood on"
			} else {
				"could not stand on"
			}
		);
	}

	// A flier's room still knows the surfaces it perches on.
	let bird = room::nav_for(room::desk_and_chair_room(), Vessel::Bird);

	assert!(room::regions_at(&bird, heights::DESK)[0].is_standable());
}

#[test]
fn erosion_keeps_a_penguins_feet_off_the_chairs_edge() {
	let nav = room::nav_for(room::desk_and_chair_room(), Vessel::Penguin);
	let chair = &room::regions_at(&nav, heights::CHAIR)[0];
	let radius = Vessel::Penguin.locomotion().foot_radius;

	// The chair's cells span x -1.05 to -0.65 and z 2.55 to 2.95.
	let (left, right) = (-1.05, -0.65);
	let (front, back) = (2.55, 2.95);

	assert!(
		!chair.standable_cells.is_empty(),
		"the penguin fits on the chair"
	);

	for &cell in &chair.standable_cells {
		let point = nav.grid.centre_of(nav.grid.x_of(cell), nav.grid.z_of(cell));
		let nearest_edge = (point.x - left)
			.min(right - point.x)
			.min(point.z - front)
			.min(back - point.z);

		assert!(
			nearest_edge + 1e-9 >= radius,
			"cell at {}, {} keeps only {nearest_edge} m of its {} m foot",
			point.x,
			point.z,
			radius
		);
	}
}

#[test]
fn a_counter_takes_an_ape_but_no_penguin() {
	let ape = room::nav_for(room::counter_room(), Vessel::Ape);

	assert!(room::has_link(&ape, heights::FLOOR, heights::COUNTER));

	let penguin = room::nav_for(room::counter_room(), Vessel::Penguin);

	assert!(!room::has_link(&penguin, heights::FLOOR, heights::COUNTER));
}
