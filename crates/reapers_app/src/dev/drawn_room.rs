//! A room drawn by hand, played in on a simulator or a desk
//! (`REAPERS_DRAWN_ROOM`): `reapers_world`'s kitchen, read as if the
//! platform saw it, and drawn where the camera's picture would be.

use reapers_rules::Vec3;
use reapers_world::DrawnRoom;

use crate::hunt::overlay::Lens;
use crate::hunt::overlay::Sketch;
use crate::theme::BONE;

/// The room hunts are played in with `REAPERS_DRAWN_ROOM`.
pub fn room() -> Option<DrawnRoom> {
	crate::config::BuildFlags::current()
		.drawn_room
		.then(DrawnRoom::kitchen)
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
