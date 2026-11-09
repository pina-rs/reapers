//! A room drawn by hand, played in on a simulator or a desk
//! (`REAPERS_DRAWN_ROOM`): `reapers_world`'s kitchen with a stool at its
//! counter, read as if the platform saw it, and drawn where the camera's
//! picture would be.

use reapers_rules::Vec3;
use reapers_world::Block;
use reapers_world::DrawnRoom;
use reapers_world::RoomGridBuilder;

use crate::hunt::overlay::Lens;
use crate::hunt::overlay::Sketch;
use crate::theme::BONE;

/// A stool at the counter, 40 cm square and 45 cm high, off to the right
/// where the floor in front of it is outside the ring kept clear round the
/// player: a husk too small to leap onto the counter from the floor gets up
/// by way of it, as a penguin does.
const STOOL: Block = Block {
	left: 0.95,
	right: 1.35,
	near: 1.75,
	far: 2.15,
	height: 0.45,
};

/// The pictures the room's grid is read from: ten round the arc ahead,
/// from a standing eye, as a sigil traced over the floor takes them.
/// `reapers_world`'s own tests read their rooms from the same ten.
const GRID_YAWS: [f64; 5] = [-60.0, -30.0, 0.0, 30.0, 60.0];
const GRID_PITCHES: [f64; 2] = [-40.0, -15.0];

/// The room hunts are played in with `REAPERS_DRAWN_ROOM`.
pub fn room() -> Option<DrawnRoom> {
	crate::config::BuildFlags::current()
		.drawn_room
		.then(kitchen)
}

/// The drawn kitchen: a counter across the room ahead, and a stool at it.
pub fn kitchen() -> DrawnRoom {
	let mut room = DrawnRoom::kitchen();
	room.blocks.push(STOOL);
	room
}

/// Reads `room` into `grid` cell by cell, as a phone tracing a sigil in it
/// would.
pub fn read_grid(room: &DrawnRoom, grid: &mut RoomGridBuilder) {
	for pitch in GRID_PITCHES {
		for yaw in GRID_YAWS {
			grid.add_frame(&room.picture(yaw, pitch));
		}
	}
}

/// The room's edges, where they fall on the screen: the floor's outline at
/// the walls, a few lines across the floor, and every block's edges.
pub fn draw(sketch: &mut Sketch, lens: &Lens, room: &DrawnRoom) {
	let floor = -room.eye_height;
	let wall = room.wall;
	let line = |sketch: &mut Sketch, from: Vec3, to: Vec3, alpha: f64, width: f64| {
		// Split, so that a line that runs past the player is drawn only
		// where it is in front.
		let points: Vec<_> = (0..=16)
			.map(|step| {
				let t = f64::from(step) / 16.0;
				lens.place(from * (1.0 - t) + to * t)
			})
			.collect();
		sketch.line_through(&points, BONE, alpha, width);
	};
	let corners = [
		Vec3::new(-wall, floor, -wall),
		Vec3::new(wall, floor, -wall),
		Vec3::new(wall, floor, wall),
		Vec3::new(-wall, floor, wall),
	];
	for index in 0..4 {
		let (from, to) = (corners[index], corners[(index + 1) % 4]);
		line(sketch, from, to, 0.35, 1.2);
		let up = Vec3::new(0.0, 1.0 + room.eye_height, 0.0);
		line(sketch, from, from + up, 0.2, 1.0);
	}
	for step in 1..8 {
		let z = f64::from(step) * 0.6;
		line(
			sketch,
			Vec3::new(-wall, floor, z),
			Vec3::new(wall, floor, z),
			0.1,
			1.0,
		);
	}
	for block in &room.blocks {
		let top = floor + block.height;
		let box_corner = |x: f64, y: f64, z: f64| Vec3::new(x, y, z);
		let (l, r, n, f) = (block.left, block.right, block.near, block.far);
		for (y, alpha) in [(floor, 0.4), (top, 0.75)] {
			line(sketch, box_corner(l, y, n), box_corner(r, y, n), alpha, 1.6);
			line(sketch, box_corner(r, y, n), box_corner(r, y, f), alpha, 1.6);
			line(sketch, box_corner(r, y, f), box_corner(l, y, f), alpha, 1.6);
			line(sketch, box_corner(l, y, f), box_corner(l, y, n), alpha, 1.6);
		}
		for (x, z) in [(l, n), (r, n), (r, f), (l, f)] {
			line(
				sketch,
				box_corner(x, floor, z),
				box_corner(x, top, z),
				0.6,
				1.6,
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::f64::consts::PI;

	use reapers_rules::Difficulty;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntRules;
	use reapers_rules::HuntSim;
	use reapers_rules::HuntStats;
	use reapers_rules::HuskMotionState;
	use reapers_rules::HuskPose;
	use reapers_rules::RoomGrid;
	use reapers_rules::RoomNav;
	use reapers_rules::Vessel;
	use reapers_rules::market_listings;
	use reapers_rules::radians;

	use super::*;
	use crate::hunt::arena::hidden_by;

	fn grid() -> RoomGrid {
		let mut grid = RoomGridBuilder::new();
		read_grid(&kitchen(), &mut grid);
		grid.finish()
	}

	fn nav(vessel: Vessel) -> RoomNav {
		RoomNav::for_hunt(
			grid(),
			vessel.locomotion(),
			HuntRules::STAGING_FAR_METRES,
			HuntRules::NEAREST_METRES,
		)
	}

	/// The largest region a body can stand on whose top is `height` metres
	/// off the floor, give or take 5 cm, by its place among the regions.
	fn region_at(nav: &RoomNav, height: f64) -> usize {
		nav.regions
			.iter()
			.enumerate()
			.filter(|(_, region)| {
				region.is_standable() && (region.height_metres() - height).abs() < 0.05
			})
			.max_by_key(|(_, region)| region.standable_cells.len())
			.map_or_else(
				|| panic!("nothing to stand on {height} m up"),
				|(index, _)| index,
			)
	}

	fn leaps(nav: &RoomNav, from: usize, to: usize) -> bool {
		nav.links
			.iter()
			.any(|link| link.from_region == from && link.to_region == to)
	}

	/// A penguin of the size it hunts at, standing `top` metres off the
	/// kitchen's floor at `x`, `z`.
	fn penguin_on(x: f64, top: f64, z: f64) -> HuskPose {
		let vessel = Vessel::Penguin;
		let scale = vessel.locomotion().body_height / (2.0 * vessel.half_height());
		let floor = -kitchen().eye_height;
		HuskPose {
			scale,
			..HuskPose::new(
				vessel,
				Vec3::new(x, floor + top + vessel.half_height() * scale, z),
				PI,
			)
		}
	}

	#[test]
	fn the_kitchen_reads_into_a_floor_a_stool_and_a_counter() {
		let grid = grid();
		assert!(grid.is_well_formed());
		for height in [0_u8, 45, 92] {
			assert!(
				grid.levels_centimetres
					.iter()
					.any(|&level| level.abs_diff(height) <= 5),
				"no level at {height} cm in {:?}",
				grid.levels_centimetres,
			);
		}
	}

	#[test]
	fn a_penguin_gets_onto_the_counter_by_way_of_the_stool() {
		let penguin = nav(Vessel::Penguin);
		let (floor, stool, counter) = (
			region_at(&penguin, 0.0),
			region_at(&penguin, 0.45),
			region_at(&penguin, 0.92),
		);
		assert!(leaps(&penguin, floor, stool));
		assert!(leaps(&penguin, stool, counter));
		assert!(
			!leaps(&penguin, floor, counter),
			"a penguin cannot jump 92 cm"
		);
		assert!(penguin.reachable_from(floor)[counter]);
		// A llama jumps 90 cm, just short of the counter, and an ape a
		// metre, which clears it.
		let llama = nav(Vessel::Llama);
		assert!(!leaps(
			&llama,
			region_at(&llama, 0.0),
			region_at(&llama, 0.92)
		));
		let ape = nav(Vessel::Ape);
		assert!(leaps(&ape, region_at(&ape, 0.0), region_at(&ape, 0.92)));
	}

	#[test]
	fn a_top_does_not_hide_the_husk_on_it_and_hides_the_one_behind_it() {
		let room = kitchen();
		let survey = room.survey(Vessel::Penguin);
		let occlusion = survey.occlusion();
		assert!(!hidden_by(occlusion, &penguin_on(0.0, 0.92, 2.5)));
		assert!(!hidden_by(occlusion, &penguin_on(-1.2, 0.92, 2.6)));
		assert!(!hidden_by(occlusion, &penguin_on(0.8, 0.45, 2.0)));
		assert!(hidden_by(occlusion, &penguin_on(0.0, 0.0, 3.4)));
	}

	#[test]
	fn a_hunt_in_the_kitchen_walks_the_grid_it_was_read_into() {
		let nft = market_listings()
			.into_iter()
			.find(|nft| nft.collection.vessel() == Vessel::Penguin)
			.expect("a penguin on the market");
		let mut plan = kitchen()
			.survey(Vessel::Penguin)
			.plan_within(0.0, radians(80.0));
		plan.room = Some(grid());
		let mut sim = HuntSim::new(
			7,
			Vessel::Penguin,
			nft.rarity(),
			nft.rarity().ward(),
			&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
			plan,
			Difficulty::default(),
		)
		.expect("a hunt in the kitchen");
		let floor = -kitchen().eye_height;
		let mut tops = Vec::new();
		for _ in 0..60 * 40 {
			sim.step(&HuntInput::still(sim.pose().direction(), 0.0));
			let pose = sim.pose();
			let locomotion = pose.locomotion.expect("a walker's pose");
			if locomotion.state != HuskMotionState::Air {
				let top = ((locomotion.support_height - floor) * 100.0).round() as i64;
				if !tops.contains(&top) {
					tops.push(top);
				}
			}
		}
		assert_ne!(tops, Vec::<i64>::new());
		assert!(
			tops.iter()
				.all(|&top| [0, 45, 92].iter().any(|level| (top - level).abs() <= 5)),
			"stood at {tops:?} cm",
		);
	}
}
