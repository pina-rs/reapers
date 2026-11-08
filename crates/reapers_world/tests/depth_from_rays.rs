//! Depth from the platform's AR, a ray at a time: the grid of rays, the
//! frame it makes, and the room read from such frames.

mod support;

use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_world::DepthFrame;
use reapers_world::DrawnRoom;
use reapers_world::RayGrid;
use reapers_world::RoomMap;
use reapers_world::RoomSurvey;
use support::Sweep;
use support::assert_close;
use support::assert_near;
use support::pictures;
use support::slice_at;
use support::sweep_with;

const EYE: f64 = 1.45;

/// The usual grid, through the drawn room's camera, held as the room's
/// pictures are.
fn grid(yaw: f64, pitch: f64) -> RayGrid {
	RayGrid::new(
		DrawnRoom::pose(yaw, pitch),
		DrawnRoom::TAN_HALF_WIDTH,
		DrawnRoom::TAN_HALF_HEIGHT,
	)
}

/// How many of `frame`'s pixels were filled in.
fn filled(frame: &DepthFrame) -> usize {
	frame
		.depths
		.iter()
		.filter(|depth| depth.is_finite())
		.count()
}

/// Asserts that every pixel `frame` filled in is where `truth`, a picture
/// of the same room through the same view, says it is, to within three
/// centimetres a metre. A crease so shallow that it passes for flat, such
/// as the floor running into a far wall, is filled in a little out.
#[track_caller]
fn assert_true_to(frame: &DepthFrame, truth: &DepthFrame) {
	assert_eq!((frame.width, frame.height), (truth.width, truth.height));

	for (index, (depth, true_depth)) in frame.depths.iter().zip(&truth.depths).enumerate() {
		if depth.is_finite() {
			assert!(
				(depth / true_depth - 1.0).abs() < 0.03,
				"pixel {index} is {depth}, where it is really {true_depth}"
			);
		}
	}
}

/// Asserts that no pixel `frame` filled in is nearer than `truth` says,
/// by more than three centimetres a metre: nothing is made up in front of
/// what is there. Something thinner than the gap between two rays, such as the
/// end of a counter seen edge on, can fall between them and be missed, and
/// what is behind it read instead; no grid of rays can help that.
#[track_caller]
fn assert_nothing_made_up(frame: &DepthFrame, truth: &DepthFrame) {
	assert_eq!((frame.width, frame.height), (truth.width, truth.height));

	for (index, (depth, true_depth)) in frame.depths.iter().zip(&truth.depths).enumerate() {
		if depth.is_finite() {
			assert!(
				*depth > true_depth * 0.97,
				"pixel {index} is {depth}, where it is really {true_depth}"
			);
		}
	}
}

/// A sweep made of the platform's rays.
fn rays() -> Sweep {
	Sweep {
		every: 10.0,
		rays: true,
		..Sweep::default()
	}
}

mod the_grid {
	use super::*;

	#[test]
	fn has_a_ray_for_every_cell_each_of_length_one() {
		let grid = grid(30.0, -20.0);

		assert_eq!(grid.rays().count(), RayGrid::COLUMNS * RayGrid::ROWS);

		for ray in grid.rays() {
			assert_close(ray.length(), 1.0, 1e-12);
		}
	}

	#[test]
	fn reads_from_the_top_left_as_a_picture_does() {
		let grid = grid(0.0, 0.0);
		let first = grid.ray(0, 0);
		let last = grid.ray(RayGrid::COLUMNS - 1, RayGrid::ROWS - 1);

		assert!(first.x < 0.0 && first.y > 0.0);
		assert!(last.x > 0.0 && last.y < 0.0);
		assert_near(first + last, Vec3::new(0.0, 0.0, 2.0 * first.z), 1e-12);
	}
}

mod the_frame_it_makes {
	use super::*;

	#[test]
	fn holds_depth_along_the_view_not_along_each_ray() {
		// A wall two metres ahead, square on. Every ray that meets it meets
		// it farther off than two metres, as none is straight ahead; but
		// every point of it is two metres along the view.
		let room = DrawnRoom::walled(2.0);
		let grid = grid(0.0, 0.0);
		let hits = room.hits(&grid);
		let frame = DepthFrame::from_rays(&grid, &hits);

		for (index, hit) in hits.iter().enumerate() {
			let (_, up) = grid.tangents(index % grid.columns, index / grid.columns);

			if up.abs() < 0.3 {
				assert!(hit.unwrap() > 2.0);
			}
		}

		// The middle of the view, well clear of the floor and the ceiling.
		let mut on_the_wall = 0;

		for row in 0..frame.height {
			for column in 0..frame.width {
				let (_, up) = frame.tangents(column, row);
				let depth = frame.depths[row * frame.width + column];

				if up.abs() < 0.3 && depth.is_finite() {
					assert_close(f64::from(depth), 2.0, 1e-4);
					on_the_wall += 1;
				}
			}
		}

		assert!(on_the_wall > frame.width * 100);
	}

	#[test]
	fn is_as_fine_as_the_pictures_the_room_is_read_from() {
		let frame = DrawnRoom::default().ray_picture(0.0, -30.0);
		let picture = DrawnRoom::default().picture(0.0, -30.0);

		assert_eq!((frame.width, frame.height), (picture.width, picture.height));
	}

	#[test]
	fn fills_in_the_floor_exactly_and_most_of_the_picture() {
		let room = DrawnRoom::default();

		for pitch in [-50.0, -30.0] {
			let frame = room.ray_picture(10.0, pitch);

			assert_true_to(&frame, &room.picture(10.0, pitch));
			assert!(filled(&frame) * 10 > frame.depths.len() * 7, "at {pitch}°");
		}
	}

	#[test]
	fn never_guesses_across_an_edge_or_a_crease() {
		// The counter's edge against the floor beyond it, its front against
		// the floor in front, and the floor against the walls: none of it
		// is filled in with a surface that is not there.
		let room = DrawnRoom::kitchen();
		let mut yaw = -60.0;

		while yaw <= 60.0 {
			for pitch in [-40.0, -25.0, -10.0] {
				assert_nothing_made_up(&room.ray_picture(yaw, pitch), &room.picture(yaw, pitch));
			}

			yaw += 15.0;
		}
	}

	#[test]
	fn leaves_a_hole_where_a_ray_hit_nothing() {
		let room = DrawnRoom::kitchen();
		let grid = grid(0.0, -30.0);
		let whole = DepthFrame::from_rays(&grid, &room.hits(&grid));
		let mut hits = room.hits(&grid);

		// A patch in the middle of the view that the platform could not
		// read: something dark, or glass.
		for row in 24..36 {
			for column in 18..30 {
				hits[row * RayGrid::COLUMNS + column] = None;
			}
		}

		let holed = DepthFrame::from_rays(&grid, &hits);

		assert!(filled(&holed) < filled(&whole));
		assert!(filled(&holed) > 0);
		assert_nothing_made_up(&holed, &room.picture(0.0, -30.0));
	}

	#[test]
	fn is_empty_where_nothing_was_hit_and_teaches_nothing() {
		let grid = grid(0.0, -30.0);
		let frame = DepthFrame::from_rays(&grid, &vec![None; RayGrid::COLUMNS * RayGrid::ROWS]);
		let mut map = RoomMap::new(Vessel::Penguin);

		assert_eq!(filled(&frame), 0);
		assert!(!map.add(&frame));
		assert_eq!(map.floor_depth(), None);
	}

	#[test]
	fn is_empty_from_a_grid_too_small_to_tell_a_crease() {
		let grid = RayGrid {
			columns: 2,
			rows: 2,
			..grid(0.0, -50.0)
		};
		let frame = DepthFrame::from_rays(&grid, &DrawnRoom::default().hits(&grid));

		assert_eq!(filled(&frame), 0);
	}
}

mod the_room_read_from_them {
	use super::*;

	#[test]
	fn finds_the_floor_and_the_walls() {
		let mut map = RoomMap::new(Vessel::Penguin);

		sweep_with(&mut map, &DrawnRoom::default(), -60.0, 60.0, -30.0, rays());

		assert_close(map.floor_depth().unwrap(), EYE, 0.03);
		assert_close(map.reach(slice_at(0.0)).unwrap(), 4.0, 0.3);
		assert!(!map.tight(slice_at(0.0)));
	}

	#[test]
	fn finds_the_counter_its_ledge_and_room_behind_it() {
		let mut map = RoomMap::new(Vessel::Ape);

		sweep_with(&mut map, &DrawnRoom::kitchen(), -70.0, 70.0, -30.0, rays());

		let counter = map.furniture(slice_at(0.0)).unwrap();

		assert_close(counter.front, 2.2, 0.2);
		assert_close(counter.cover, 0.92, 0.08);
		assert!(counter.ledge);
		assert!(counter.hide.is_some());
	}

	#[test]
	fn hides_a_husk_behind_the_counter_and_not_on_the_floor() {
		let mut survey = RoomSurvey::new(Vessel::Ape);

		for frame in pictures(&DrawnRoom::kitchen(), -60.0, 60.0, -30.0, rays()) {
			survey.add(&frame);
		}

		let hides = |point: Vec3| {
			survey
				.occlusion()
				.hides(point.yaw(), point.pitch(), point.length())
		};

		assert!(hides(Vec3::new(0.0, -EYE + 0.3, 3.4)));
		assert!(!hides(Vec3::new(0.0, -EYE + 1.4, 3.4)));
		assert!(!hides(Vec3::new(0.0, -EYE + 0.05, 1.8)));
	}
}
