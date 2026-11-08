//! When a hunt is ready to begin.

mod support;

use reapers_rules::*;
use support::*;

#[test]
fn blade_recovery_and_cadence_reflect_the_actual_strike_windows() {
	let mut sim = default_sim();
	let steady = HuntInput::still(sim.pose().direction(), 0.0);

	assert_eq!(sim.strike_ticks_left(), 0);
	assert_eq!(sim.cadence_ticks_left(), 0);

	for _ in 0..HuntRules::AMBUSH_TICKS {
		sim.step(&steady);
	}

	let seal = (0..sim.vessel().seals().len())
		.find(|&index| sim.seal_open(index))
		.expect("an open seal");
	let tap = HuntInput::measured(
		sim.pose().direction(),
		0.0,
		&[sim.pose().seal_direction(seal)],
		false,
	);
	let hit = sim.step(&tap);

	assert_eq!(
		hit.iter()
			.filter(|event| matches!(event, HuntEvent::StrikeLanded { .. }))
			.count(),
		1
	);
	assert_eq!(sim.strike_ticks_left(), sim.stats().swing_ticks - 1);
	assert_eq!(
		sim.cadence_ticks_left(),
		HuntRules::CADENCE_WINDOW_TICKS - 1
	);

	for _ in 0..sim.stats().swing_ticks {
		sim.step(&steady);
	}

	assert_eq!(sim.strike_ticks_left(), 0);
	assert!(sim.cadence_ticks_left() > 0);

	for _ in 0..=HuntRules::CADENCE_WINDOW_TICKS {
		sim.step(&steady);
	}

	assert_eq!(sim.strike_ticks_left(), 0);
	assert_eq!(sim.cadence_ticks_left(), 0);
	assert_eq!(sim.cadence(), 0);

	let miss = sim.step(&HuntInput::measured(
		Vec3::FORWARD,
		0.0,
		&[Vec3::new(0.0, 1.0, 0.0)],
		false,
	));

	assert_eq!(
		miss.iter()
			.filter(|event| matches!(event, HuntEvent::StrikeWhiffed))
			.count(),
		1
	);
	assert_eq!(sim.strike_ticks_left(), sim.stats().swing_ticks - 1);
	assert_eq!(sim.cadence_ticks_left(), 0);
}
