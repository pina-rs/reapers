//! The locomotion stepper: walking, edges, crouches, leaps, landings.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use reapers_rules::HuskMotion;
use reapers_rules::HuskMotionState;
use reapers_rules::MotionPace;
use reapers_rules::RoomSpotKind;
use reapers_rules::SeededRandom;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use support::room;
use support::room::heights;

/// What one husk did, tick by tick, until it had nothing to do.
struct Run<'nav> {
	motion: HuskMotion<'nav>,
	states: Vec<HuskMotionState>,
	supports: Vec<f64>,
	positions: Vec<Vec3>,
	cells: Vec<usize>,
	speeds: Vec<f64>,
	fractions: Vec<f64>,
	ticks: i64,
	idle: bool,
}

impl<'nav> Run<'nav> {
	fn until_idle(motion: HuskMotion<'nav>) -> Self {
		let mut run = Self {
			motion,
			states: Vec::new(),
			supports: Vec::new(),
			positions: Vec::new(),
			cells: Vec::new(),
			speeds: Vec::new(),
			fractions: Vec::new(),
			ticks: 0,
			idle: false,
		};

		while !run.motion.is_idle() && run.ticks < 4000 {
			run.motion.tick();
			run.states.push(run.motion.state());
			run.supports.push(run.motion.support_height());
			run.positions.push(run.motion.position());
			run.cells.push(run.motion.cell());
			run.speeds.push(run.motion.speed());
			run.fractions.push(run.motion.state_fraction());
			run.ticks += 1;
		}

		run.idle = run.motion.is_idle();
		run
	}

	fn final_support(&self) -> f64 {
		self.motion.support_height()
	}

	fn final_level(&self) -> i8 {
		self.nav().level_of(self.motion.cell())
	}

	fn nav(&self) -> &reapers_rules::RoomNav {
		self.motion.nav
	}

	/// Whether `wanted` appears in `states`, in order.
	fn has_in_order(&self, wanted: &[HuskMotionState]) -> bool {
		let mut at = 0;

		for state in &self.states {
			if *state == wanted[at] {
				at += 1;

				if at == wanted.len() {
					return true;
				}
			}
		}

		false
	}
}

#[test]
#[allow(
	clippy::too_many_lines,
	reason = "one journey, walked once and held to on the way"
)]
fn an_ape_walks_to_the_desk_across_it_and_back_down() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let floor = room::height_of(heights::FLOOR);
	let desk = room::height_of(heights::DESK);
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(7),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("an ape stands on the floor");

	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);
	let up = Run::until_idle(motion);

	assert!(up.idle, "the ape never settled");
	assert!(up.fractions.iter().all(|value| (0.0..=1.0).contains(value)));
	assert!(
		up.has_in_order(&[
			HuskMotionState::Walk,
			HuskMotionState::Edge,
			HuskMotionState::Crouch,
			HuskMotionState::Air,
			HuskMotionState::Land,
			HuskMotionState::Recover,
			HuskMotionState::Walk,
		]),
		"the walk went {:?}",
		up.states
	);
	assert!((up.final_support() - desk).abs() <= 1e-9);
	assert!(nav.is_standable(up.motion.cell()));
	assert!(up.final_level() >= 0);
	assert_eq!(
		i64::from(nav.grid.levels_centimetres[up.final_level() as usize]),
		heights::DESK
	);
	assert!((up.motion.position().x - 0.3).abs() <= 0.03);
	assert!((up.motion.position().z - 2.8).abs() <= 0.03);

	// Off the desk, every footfall stands somewhere its body fits, at the
	// height of the level under it.
	for index in 0..up.states.len() {
		if up.states[index] == HuskMotionState::Air {
			continue;
		}

		let cell = up.cells[index];
		let level = nav.level_of(cell);
		let height = if level < 0 {
			continue;
		} else {
			(f64::from(nav.grid.levels_centimetres[level as usize])
				- f64::from(nav.grid.eye_height_centimetres))
				/ 100.0
		};

		assert!(
			nav.is_standable(cell),
			"tick {index} stands on cell {cell} mid-{:?}",
			up.states[index]
		);
		assert!((up.supports[index] - height).abs() <= 1e-9);
	}

	// Every support height is one of the room's levels, never something
	// between them.
	for support in &up.supports {
		let is_a_level = nav.grid.levels_centimetres.iter().any(|&level| {
			((f64::from(level) - f64::from(nav.grid.eye_height_centimetres)) / 100.0 - support)
				.abs()
				< 1e-9
		});

		assert!(is_a_level, "a support of {support} is not a level");
	}

	// Walking again along the desk, from where it landed to the far end.
	let mut motion = up.motion;
	motion.go_to(Vec3::new(-0.4, 0.0, 2.8), MotionPace::Walk);
	let across = Run::until_idle(motion);

	assert!(across.idle);
	assert!((across.final_support() - desk).abs() <= 1e-9);
	assert!(
		across
			.speeds
			.iter()
			.any(|speed| (*speed - 0.7).abs() < 1e-9),
		"the ape never reached its walking pace"
	);

	// At its gait it covers exactly a gait step a tick.
	let mut constant_steps = 0;

	for index in 1..across.positions.len() {
		if (across.speeds[index] - 0.7).abs() > 1e-9
			|| (across.speeds[index - 1] - 0.7).abs() > 1e-9
		{
			continue;
		}

		let away_x = across.positions[index].x - across.positions[index - 1].x;
		let away_z = across.positions[index].z - across.positions[index - 1].z;
		let step = (away_x * away_x + away_z * away_z).sqrt();

		assert!((step - 0.7 / 60.0).abs() <= 1e-9);
		constant_steps += 1;
	}

	assert!(constant_steps > 2);

	// And back down to the floor.
	let mut motion = across.motion;
	motion.go_to(Vec3::new(-0.5, 0.0, 1.2), MotionPace::Walk);
	let down = Run::until_idle(motion);

	assert!(down.idle);
	assert!((down.final_support() - floor).abs() <= 1e-9);
	assert!(down.has_in_order(&[
		HuskMotionState::Edge,
		HuskMotionState::Crouch,
		HuskMotionState::Air,
		HuskMotionState::Land,
		HuskMotionState::Recover,
	]));
	assert!(nav.is_standable(down.motion.cell()));
}

#[test]
fn the_support_height_jumps_exactly_at_the_edge() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let desk = room::height_of(heights::DESK);
	let floor = room::height_of(heights::FLOOR);
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(3),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("an ape stands on the floor");

	// Out onto the desk first.
	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);

	let mut reached_desk = false;

	for _ in 0..4000 {
		motion.tick();

		if motion.support_height() == desk && motion.state() == HuskMotionState::Walk {
			reached_desk = true;
			break;
		}
	}

	assert!(reached_desk);

	// Then ask back the way it came: the descent crosses the desk's
	// edge, and the support changes in one step, never between ticks.
	motion.go_to(Vec3::new(-0.5, 0.0, 1.2), MotionPace::Walk);

	let mut previous = motion.support_height();
	let mut switches = 0;

	for _ in 0..4000 {
		motion.tick();
		let support = motion.support_height();

		if support != previous {
			switches += 1;
			assert!(
				support == floor || support == desk,
				"the support became {support}, which is no level of this room"
			);
		}

		previous = support;

		if motion.is_idle() {
			break;
		}
	}

	assert_eq!(switches, 1, "the support changed once, at the edge");
	assert!((motion.support_height() - floor).abs() <= 1e-9);
}

/// A husk's motion out to the desk and onto it, for 400 ticks.
fn record_motion(nav: &reapers_rules::RoomNav) -> Vec<(Vec3, f64, f64, HuskMotionState)> {
	let mut motion = HuskMotion::new(
		nav,
		SeededRandom::new(11),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("an ape stands on the floor");

	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);

	let mut record = Vec::new();

	for _ in 0..400 {
		motion.tick();
		record.push((
			motion.position(),
			motion.heading(),
			motion.support_height(),
			motion.state(),
		));
	}

	record
}

#[test]
fn the_same_seed_gives_the_same_motion_bit_for_bit() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let one = record_motion(&nav);
	let two = record_motion(&nav);

	assert_eq!(one.len(), two.len());

	for (index, (first, second)) in one.iter().zip(two.iter()).enumerate() {
		assert_eq!(first, second, "tick {index} moved differently");
	}
}

#[test]
fn the_next_foothold_is_the_landing_of_a_leap() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let height = room::height_of(heights::DESK) + Vessel::Ape.locomotion().half_height();
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(4),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("an ape stands on the floor");

	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);

	let mut walked = 0;
	let mut flew = 0;

	for _ in 0..4000 {
		motion.tick();

		match motion.state() {
			HuskMotionState::Walk | HuskMotionState::Run => {
				let foothold = motion.next_foothold().expect("a walk is making for a cell");
				let away = (foothold.x - motion.position().x).abs()
					+ (foothold.z - motion.position().z).abs();

				assert!(away > 0.005, "a walk's foothold is ahead of the body");
				walked += 1;
			}
			HuskMotionState::Air => {
				let foothold = motion.next_foothold().expect("a leap lands somewhere");

				assert!((foothold.y - height).abs() <= 1e-9);
				flew += 1;
			}
			_ => {}
		}

		if motion.is_idle() {
			break;
		}
	}

	assert!(walked > 0 && flew > 0);
	assert!(
		motion.next_foothold().is_none(),
		"an arrived husk has no completion to show"
	);
}

#[test]
fn a_penguin_goes_floor_chair_desk_in_two_leaps() {
	let nav = room::nav_for(room::desk_and_chair_room(), Vessel::Penguin);
	let desk = room::height_of(heights::DESK);
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(5),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("a penguin stands on the floor");

	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);

	let journey = Run::until_idle(motion);
	let mut air_runs = 0;

	for index in 0..journey.states.len() {
		if journey.states[index] == HuskMotionState::Air
			&& (index == 0 || journey.states[index - 1] != HuskMotionState::Air)
		{
			air_runs += 1;
		}
	}

	assert!(journey.idle);
	assert_eq!(air_runs, 2);
	assert!((journey.final_support() - desk).abs() <= 1e-9);
	assert!(nav.is_standable(journey.motion.cell()));
}

#[test]
fn a_bird_flies_round_a_column_and_perches_on_the_desk() {
	let grid = room::desk_with_column_room();
	let nav = room::nav_for(grid, Vessel::Bird);
	let desk = room::height_of(heights::DESK);
	let perch = *room::spots_of(&nav, RoomSpotKind::Perch)
		.iter()
		.find(|spot| spot.point.x.abs() < 0.05)
		.expect("the desk holds a perch");
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(3),
		Some(Vec3::new(perch.point.x, 0.0, 0.5)),
		0.0,
	)
	.expect("a bird takes the air");

	motion.go_to_spot(perch, MotionPace::Walk);

	let flight = Run::until_idle(motion);

	assert!(flight.idle);
	assert!(flight.states.contains(&HuskMotionState::Air));
	assert!(flight.states.contains(&HuskMotionState::Land));
	assert!(flight.states.contains(&HuskMotionState::Recover));
	assert!((flight.final_support() - desk).abs() <= 1e-9);
	assert!(
		(flight.motion.position().y - (desk + Vessel::Bird.locomotion().half_height())).abs()
			<= 1e-9
	);
	assert!(
		flight
			.positions
			.iter()
			.any(|point| (point.x - perch.point.x).abs() > 0.3),
		"the bird flew straight through the column"
	);

	for point in &flight.positions {
		let x = nav.grid.cell_x_at(point.x, point.z);
		let z = nav.grid.cell_z_at(point.x, point.z);

		if !nav.grid.contains(x, z) {
			continue;
		}

		assert_ne!(
			nav.grid.cell_at(x, z).kind,
			reapers_rules::RoomCellKind::Obstacle
		);
	}
}

#[test]
fn a_command_issued_in_the_air_is_taken_up_on_landing() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(9),
		Some(Vec3::new(-0.5, 0.0, 1.2)),
		0.0,
	)
	.expect("an ape stands on the floor");

	motion.go_to(Vec3::new(0.3, 0.0, 2.8), MotionPace::Walk);

	let mut saw_crouch = false;

	for _ in 0..4000 {
		motion.tick();

		if motion.state() == HuskMotionState::Crouch {
			saw_crouch = true;

			// A halt mid-gather is remembered until the husk lands.
			motion.go_to(Vec3::new(-0.5, 0.0, 1.2), MotionPace::Run);

			break;
		}
	}

	assert!(saw_crouch);

	let rest = Run::until_idle(motion);

	assert!(rest.idle);
	assert!(
		(rest.final_support() - room::height_of(heights::FLOOR)).abs() <= 1e-9,
		"the new order, given in the air, was not taken up on landing"
	);
	assert!((rest.motion.position().x - -0.5).abs() <= 0.03);
}

#[test]
fn a_turn_in_place_happens_when_the_way_is_behind() {
	let nav = room::nav_for(room::desk_room(), Vessel::Ape);
	let mut motion = HuskMotion::new(
		&nav,
		SeededRandom::new(2),
		Some(Vec3::new(0.0, 0.0, 1.5)),
		0.0,
	)
	.expect("an ape stands on the floor");

	motion.face(Vec3::new(0.0, 0.0, -1.0));

	let mut turned = false;
	let mut saw_turn_state = false;

	for _ in 0..600 {
		let before = motion.position();
		motion.tick();

		if motion.state() == HuskMotionState::Turn {
			saw_turn_state = true;
			assert_eq!(motion.position(), before, "a turn holds its place");
		}

		if (motion.heading() - std::f64::consts::PI).abs() < 0.01 {
			turned = true;
			break;
		}
	}

	assert!(saw_turn_state);
	assert!(turned);
}
