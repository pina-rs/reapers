//! The ballistic leap: exact, clear of the room, and refused when not.

mod support;

use reapers_rules::LeapRules;
use support::room;
use support::room::heights;

#[test]
fn a_leap_lands_exactly_on_its_target_on_the_last_tick() {
	let leap = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");
	let landed = leap.position_at(leap.ticks);

	assert!((landed.x - leap.to.x).abs() <= 1e-9);
	assert!((landed.y - leap.to.y).abs() <= 1e-9);
	assert!((landed.z - leap.to.z).abs() <= 1e-9);
}

#[test]
fn floor_to_a_desk_and_back_take_whole_ticks_near_the_record() {
	// The research record's table: a 0.75 m desk up in about 38 ticks,
	// down in about 35, give or take a tick.
	let up = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");
	let down = room::leap_over_desk_room((0.75, 2.7), (0.0, 2.0)).expect("a leap");

	assert!((35..=41).contains(&up.ticks), "up took {} ticks", up.ticks);
	assert!(
		(32..=38).contains(&down.ticks),
		"down took {} ticks",
		down.ticks
	);
}

#[test]
fn the_counter_and_a_gap_take_whole_ticks_near_the_record() {
	// The rest of the research record's table: a 0.90 m counter in about
	// 40 ticks, and a 0.6 m gap on one level in about 27.
	let ape = reapers_rules::Vessel::Ape.locomotion();
	let floor = room::floor_height();
	let counter = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(0.0, floor + ape.half_height(), 1.5),
		reapers_rules::Vec3::new(0.0, floor + ape.half_height() + 0.90, 2.5),
		&room::counter_room(),
		ape.half_height(),
		None,
	)
	.expect("the ape takes the counter");
	let gap = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(0.0, floor + ape.half_height(), 1.0),
		reapers_rules::Vec3::new(0.0, floor + ape.half_height(), 1.6),
		&room::empty_floor_room(),
		ape.half_height(),
		Some(0.25),
	)
	.expect("a hop clears the gap");

	assert!(
		(38..=42).contains(&counter.ticks),
		"the counter took {} ticks",
		counter.ticks
	);
	assert!(
		(25..=29).contains(&gap.ticks),
		"the gap took {} ticks",
		gap.ticks
	);
}

#[test]
fn a_penguins_two_hops_take_whole_ticks_near_the_record() {
	let penguin = reapers_rules::Vessel::Penguin.locomotion();
	let floor = room::floor_height();

	// Floor to a 0.45 m chair, then the 0.30 m step to the desk.
	let to_chair = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(-0.5, floor + penguin.half_height(), 1.2),
		reapers_rules::Vec3::new(-0.6, floor + penguin.half_height() + 0.45, 2.3),
		&room::desk_and_chair_room(),
		penguin.half_height(),
		None,
	)
	.expect("the penguin reaches the chair");
	let to_desk = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(-0.6, floor + penguin.half_height() + 0.45, 2.3),
		reapers_rules::Vec3::new(0.2, floor + penguin.half_height() + 0.75, 2.8),
		&room::desk_and_chair_room(),
		penguin.half_height(),
		None,
	)
	.expect("the penguin reaches the desk");

	assert!(
		(31..=37).contains(&to_chair.ticks),
		"chair took {} ticks",
		to_chair.ticks
	);
	assert!(
		(29..=35).contains(&to_desk.ticks),
		"desk took {} ticks",
		to_desk.ticks
	);
}

#[test]
fn the_arc_clears_the_desk_edge_by_at_least_twelve_centimetres() {
	let leap = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");
	let desk_front = 2.45;
	let fraction = (desk_front - leap.from.z) / (leap.to.z - leap.from.z);
	let at_edge = leap.position_at_seconds(leap.flight_seconds() * fraction);
	let ape = reapers_rules::Vessel::Ape.locomotion();
	let desk_top = room::height_of(heights::DESK);
	let feet = at_edge.y - ape.half_height();

	assert!(feet - desk_top >= 0.12);
}

#[test]
fn a_leap_that_cannot_clear_a_column_is_refused() {
	let mut drawn = room::DrawnRoom::new(&[heights::FLOOR]);
	drawn.obstacle(39, 34, 2, 2, 248);

	let leap = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(0.0, room::floor_height() + 0.30, 1.0),
		reapers_rules::Vec3::new(0.0, room::floor_height() + 0.30, 2.0),
		&drawn.build(),
		0.30,
		None,
	);

	assert!(leap.is_none());
}

#[test]
fn the_apex_is_raised_when_a_low_obstacle_would_be_hit() {
	let mut drawn = room::DrawnRoom::new(&[heights::FLOOR]);
	drawn.obstacle(39, 34, 2, 2, 24);

	let leap = reapers_rules::Leap::solve(
		reapers_rules::Vec3::new(0.0, room::floor_height() + 0.30, 1.0),
		reapers_rules::Vec3::new(0.0, room::floor_height() + 0.30, 2.0),
		&drawn.build(),
		0.30,
		None,
	)
	.expect("a low obstacle only raises the apex");

	assert!(leap.clearance > LeapRules::LEVEL_CLEARANCE);
}

#[test]
fn the_same_leap_replays_bit_for_bit() {
	let first = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");
	let second = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");

	assert_eq!(first, second);

	for tick in 0..=first.ticks {
		assert_eq!(first.position_at(tick), second.position_at(tick));
	}
}

#[test]
fn a_leap_leaves_at_full_speed_and_lands_at_full_speed() {
	let leap = room::leap_over_desk_room((0.0, 2.0), (0.75, 2.7)).expect("a leap");

	// One tick in, the body has already covered nearly a full tick of
	// horizontal travel; the arc never eases in.
	let step = leap.position_at(1);
	let expected = leap.horizontal_velocity.z / reapers_rules::HuntRules::TICK_RATE as f64;

	assert!((step.z - leap.from.z - expected).abs() <= 1e-12);

	// The horizontal speed never changes: every tick covers the same
	// ground, first to last.
	let x_step = leap.horizontal_velocity.x / reapers_rules::HuntRules::TICK_RATE as f64;

	for tick in 1..=leap.ticks {
		let here = leap.position_at(tick);
		let before = leap.position_at(tick - 1);

		assert!((here.x - before.x - x_step).abs() <= 1e-12);
		assert!((here.z - before.z - expected).abs() <= 1e-12);
	}

	// It leaves going up and lands coming down: no easing at either end.
	assert!(leap.vertical_velocity > 0.0);
	assert!(leap.impact_speed() > 0.0);
}
