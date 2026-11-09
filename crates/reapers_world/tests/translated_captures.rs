//! A player who moves while the room is read: each picture is charted from
//! the eye that took it, into one room, and the plan and the grid agree about
//! the floor they both stand on.

use reapers_rules::Difficulty;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::RoomGrid;
use reapers_rules::RoomNav;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::market_listings;
use reapers_rules::radians;
use reapers_world::Block;
use reapers_world::DepthFrame;
use reapers_world::DrawnRoom;
use reapers_world::RayGrid;
use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;
use reapers_world::Shot;

/// A desk 1.2 by 0.6 m at 0.75 m, 2.5 m ahead, in an otherwise empty room.
const DESK: Block = Block {
	left: -0.6,
	right: 0.6,
	near: 2.5,
	far: 3.1,
	height: 0.75,
};

/// The heights and pitches a sigil's survey reads.
const YAWS: [f64; 5] = [-60.0, -30.0, 0.0, 30.0, 60.0];
const PITCHES: [f64; 2] = [-40.0, -15.0];

fn desk_room() -> DrawnRoom {
	DrawnRoom::with(&[DESK])
}

/// Where a player who walks a little while tracing steps to, in turn: half a
/// pace right, a hand's width up, half a pace left and back.
const EYES: [Vec3; 4] = [
	Vec3::ZERO,
	Vec3::new(0.35, 0.0, 0.05),
	Vec3::new(-0.25, 0.08, -0.15),
	Vec3::new(0.1, -0.05, 0.2),
];

/// Reads `room` from the ten survey pictures, the first `moved` of them taken
/// from the eyes [`EYES`] gives, in turn.
fn read(room: &DrawnRoom, moved: bool) -> RoomGrid {
	let mut builder = RoomGridBuilder::new();
	let mut picture = 0;

	for pitch in PITCHES {
		for yaw in YAWS {
			let eye = if moved {
				EYES[picture % EYES.len()]
			} else {
				Vec3::ZERO
			};

			builder.add_frame(&room.picture_from(eye, yaw, pitch));
			picture += 1;
		}
	}

	assert!(builder.frames() > 0, "the survey was charted");
	builder.finish()
}

/// Reads `room` into a survey, pictures taken as [`read`] takes them.
fn survey(room: &DrawnRoom, moved: bool) -> RoomSurvey {
	let mut survey = RoomSurvey::new(Vessel::Ape);
	let mut picture = 0;

	for pitch in PITCHES {
		for yaw in YAWS {
			let eye = if moved {
				EYES[picture % EYES.len()]
			} else {
				Vec3::ZERO
			};

			survey.add(&room.picture_from(eye, yaw, pitch));
			picture += 1;
		}
	}

	survey
}

fn nav(grid: &RoomGrid) -> RoomNav {
	RoomNav::for_hunt(
		grid.clone(),
		Vessel::Ape.locomotion(),
		HuntRules::STAGING_FAR_METRES,
		HuntRules::NEAREST_METRES,
	)
}

/// The cell columns and rows the biggest surface at `metres` covers.
fn bounds(nav: &RoomNav, metres: f64) -> (i64, i64, i64, i64) {
	let region = nav
		.regions
		.iter()
		.filter(|region| region.is_standable() && (region.height_metres() - metres).abs() <= 0.05)
		.max_by_key(|region| region.standable_cells.len())
		.unwrap_or_else(|| panic!("nothing to stand on {metres} m up"));
	let mut bounds = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);

	for &cell in &region.cells {
		let (x, z) = (nav.grid.x_of(cell), nav.grid.z_of(cell));

		bounds.0 = bounds.0.min(x);
		bounds.1 = bounds.1.max(x);
		bounds.2 = bounds.2.min(z);
		bounds.3 = bounds.3.max(z);
	}

	bounds
}

fn hunt(plan: HuntPlan) -> HuntSim {
	let nft = market_listings()
		.into_iter()
		.find(|nft| nft.collection.vessel() == Vessel::Ape)
		.expect("an ape on the market");

	HuntSim::new(
		7,
		Vessel::Ape,
		nft.rarity(),
		nft.rarity().ward(),
		&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
		plan,
		Difficulty::default(),
	)
	.expect("a hunt in the room")
}

#[test]
fn pictures_taken_from_different_places_chart_the_same_room() {
	let room = desk_room();
	let still = read(&room, false);
	let moved = read(&room, true);

	assert_eq!(still.eye_height_centimetres, 145);
	assert_eq!(moved.eye_height_centimetres, 145);

	let (still, moved) = (nav(&still), nav(&moved));

	assert!(
		still.regions.iter().any(|region| {
			region.is_standable() && (region.height_metres() - 0.75).abs() <= 0.05
		})
	);
	assert!(
		moved.regions.iter().any(|region| {
			region.is_standable() && (region.height_metres() - 0.75).abs() <= 0.05
		})
	);

	let (left, right, front, back) = bounds(&still, 0.75);
	let (moved_left, moved_right, moved_front, moved_back) = bounds(&moved, 0.75);

	assert!(
		(moved_left - left).abs() <= 1
			&& (moved_right - right).abs() <= 1
			&& (moved_front - front).abs() <= 1
			&& (moved_back - back).abs() <= 1,
		"the desk moved from ({left}..{right}, {front}..{back}) to \
		 ({moved_left}..{moved_right}, {moved_front}..{moved_back})"
	);
}

#[test]
fn a_survey_taken_while_walking_hides_the_same_furniture() {
	let room = desk_room();
	let behind = |survey: &RoomSurvey| {
		let room = survey.occlusion();
		let high = Vec3::new(0.0, -1.45 + 0.3, 3.4);
		let low = Vec3::new(0.0, -1.45 + 0.05, 1.8);

		vec![
			room.hides(high.yaw(), high.pitch(), high.length()),
			room.hides(low.yaw(), low.pitch(), low.length()),
		]
	};

	assert_eq!(
		behind(&survey(&room, false)),
		behind(&survey(&room, true)),
		"a picture taken elsewhere moved the desk's cover"
	);
}

#[test]
fn converted_rays_keep_the_eye_they_were_cast_from() {
	let room = desk_room();
	let eye = Vec3::new(-0.3, 0.06, 0.1);
	let mut builder = RoomGridBuilder::new();

	for yaw in [-40.0, -20.0, 0.0, 20.0, 40.0] {
		let pose = DrawnRoom::pose(yaw, -30.0);
		let grid = RayGrid::new(pose, DrawnRoom::TAN_HALF_WIDTH, DrawnRoom::TAN_HALF_HEIGHT);
		let hits = room.hits_from(eye, &grid);
		let frame = DepthFrame::from_rays_at(&grid, &hits, eye);

		assert_eq!(frame.origin, eye, "the ray picture forgot its eye");
		builder.add_frame(&frame);
	}

	assert!(builder.frames() > 0);
	let grid = builder.finish();

	assert_eq!(
		grid.eye_height_centimetres, 145,
		"the floor moved with the eye"
	);
}

#[test]
fn the_plan_and_the_grid_take_the_same_floor() {
	let room = desk_room();
	let mut survey = RoomSurvey::new(Vessel::Ape);
	let mut builder = RoomGridBuilder::new();
	let mut picture: i64 = 0;

	for pitch in PITCHES {
		for yaw in YAWS {
			// Every picture reads the whole room a little long or short, in
			// turn: an even count, so a median and a mean disagree.
			let frame = room.picture_from_with(
				Vec3::ZERO,
				yaw,
				pitch,
				&Shot {
					stretch: if picture % 2 == 0 { 1.07 } else { 0.93 },
					seed: picture + 1,
					..Shot::default()
				},
			);

			survey.add(&frame);
			builder.add_frame(&frame);
			picture += 1;
		}
	}

	let plan = survey.plan_within(0.0, radians(80.0));
	let grid = builder.finish();

	assert_eq!(
		plan.eye_centimetres,
		i32::from(grid.eye_height_centimetres),
		"the plan and the grid found different floors"
	);

	let plan = HuntPlan {
		room: Some(grid),
		..plan
	};

	assert!(plan.is_well_formed());
	assert!(hunt(plan.clone()).plan().knows_room());
}

#[test]
fn a_measured_floor_stays_exact_when_the_player_moves() {
	let room = desk_room();
	let mut survey = RoomSurvey::new(Vessel::Ape);
	let mut builder = RoomGridBuilder::new();
	let mut picture = 0;

	survey.measure_floor(1.45);
	builder.measure_floor(1.45);

	for pitch in PITCHES {
		for yaw in YAWS {
			let eye = Vec3::new(
				EYES[picture % EYES.len()].x,
				0.0,
				EYES[picture % EYES.len()].z,
			);
			let frame = room.picture_from(eye, yaw, pitch);

			survey.add(&frame);
			builder.add_frame(&frame);
			picture += 1;
		}
	}

	let plan = survey.plan_within(0.0, radians(80.0));
	let grid = builder.finish();

	assert_eq!(plan.eye_centimetres, 145);
	assert_eq!(grid.eye_height_centimetres, 145);
}
