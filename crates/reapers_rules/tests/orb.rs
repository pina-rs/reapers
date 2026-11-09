//! Throwing the orb, the capture it makes, and the broken window.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use reapers_rules::*;
use support::*;

/// How many of `events` match `pattern`.
macro_rules! count {
	($events:expr, $pattern:pat) => {
		$events
			.iter()
			.filter(|event| matches!(event, $pattern))
			.count()
	};
}

/// A hunt the bot has fought to a broken ward, with soul to spare.
fn broken_for(options: SimFor) -> HuntSim {
	let mut sim = options.sim();
	let mut bot = HuntBot::careful(&sim);

	while sim.phase() != HuntPhase::Broken {
		bot.step_once(&mut sim);
	}

	sim
}

fn broken() -> HuntSim {
	broken_for(SimFor::default())
}

fn broken_with_seed(seed: i64) -> HuntSim {
	broken_for(SimFor {
		seed,
		..SimFor::default()
	})
}

/// Holds ORB on the husk for `held` ticks and lets go, and returns what
/// happened as it was let go.
fn throw_orb(sim: &mut HuntSim, held: i64, at: Option<Vec3>) -> Vec<HuntEvent> {
	for _ in 0..held {
		let aim = at.unwrap_or_else(|| sim.pose().direction());
		let input = HuntInput::measured(aim, 0.0, false, sim.selected_slot(), true);

		sim.step(&input);
	}

	let aim = at.unwrap_or_else(|| sim.pose().direction());
	let input = HuntInput::measured(aim, 0.0, false, sim.selected_slot(), false);

	sim.step(&input)
}

/// What happened in each tick after a throw, by ticks since the release.
fn follow_orb(sim: &mut HuntSim, ticks: i64) -> Vec<(i64, Vec<HuntEvent>)> {
	let mut after = Vec::new();

	for after_tick in 1..=ticks {
		if sim.is_over() {
			break;
		}

		after.push((after_tick, sim.step(&steady_on_husk(sim))));
	}

	after
}

/// The chance the first orb at a broken husk has.
fn broken_chance(sim: &HuntSim, grade: CaptureGrade) -> f64 {
	CaptureRules::chance(sim.rarity_odds(), 0.0, grade, sim.stats().capture, 0)
}

#[test]
fn throwing_needs_the_husk_weakened_and_the_soul_to_pay_for_it() {
	let mut sim = sim_for(SimFor::default());

	assert_eq!(sim.orb_block(), Some(OrbBlock::NotFighting));

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));

	assert_eq!(sim.orb_block(), Some(OrbBlock::NotWeakened));

	sim.set_ward(sim.ward_max() * CaptureRules::WEAKENED_SHARE);
	sim.set_soul(CaptureRules::COST - 1.0);

	assert_eq!(sim.orb_block(), Some(OrbBlock::LowSoul));

	sim.set_soul(CaptureRules::COST);

	assert_eq!(sim.orb_block(), None);

	sim.set_ward(sim.ward_max() * (CaptureRules::WEAKENED_SHARE + 0.01));

	assert_eq!(sim.orb_block(), Some(OrbBlock::NotWeakened));
}

#[test]
fn a_broken_husk_can_always_be_thrown_at_if_the_soul_is_there() {
	let mut sim = broken();

	assert_eq!(sim.orb_block(), None);

	sim.set_soul(CaptureRules::COST - 1.0);

	assert_eq!(sim.orb_block(), Some(OrbBlock::LowSoul));
}

#[test]
fn the_ring_shrinks_while_orb_is_held_and_grades_the_throw() {
	let mut sim = broken();

	assert!(sim.orb_ring().is_none());
	assert_eq!(sim.orb_grade(sim.pose().direction()), CaptureGrade::Plain);

	for _ in 0..30 {
		let aim = sim.pose().direction();
		let input = HuntInput::measured(aim, 0.0, false, sim.selected_slot(), true);

		sim.step(&input);
	}

	assert_eq!(sim.orb_ring(), Some(CaptureRules::ring_progress(30)));
	assert_eq!(sim.orb_grade(sim.pose().direction()), CaptureGrade::Great);
	assert_eq!(sim.orb_grade(-sim.pose().direction()), CaptureGrade::Plain);
}

#[test]
fn letting_go_throws_it_and_spends_the_soul() {
	let mut sim = broken();
	let soul = sim.soul();
	let events = throw_orb(&mut sim, 50, None);
	let thrown = events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::OrbThrown { grade, on_target } => Some((*grade, *on_target)),
				_ => None,
			}
		})
		.unwrap();

	assert_eq!(thrown, (CaptureGrade::Excellent, true));
	assert_eq!(sim.soul(), soul - CaptureRules::COST);
	assert_eq!(sim.phase(), HuntPhase::Capture);
	assert_eq!(sim.orb_block(), Some(OrbBlock::NotFighting));
	assert_eq!(sim.ring(), HuskRing::Weakened);
}

#[test]
fn the_fight_clock_pauses_while_an_orb_holds_the_husk() {
	let mut sim = broken();
	let left = sim.fight_ticks_left();

	assert!(left > 0);

	throw_orb(&mut sim, 50, None);

	assert_eq!(sim.phase(), HuntPhase::Capture);

	// The broken window and the orb's flight are no part of the fight: a
	// hunt must not time out while the husk is held.
	follow_orb(&mut sim, CaptureRules::RESULT_TICK);

	assert_ne!(sim.phase(), HuntPhase::Capture, "the orb has resolved");
	assert_eq!(
		sim.fight_ticks_left(),
		left,
		"the clock does not run while the husk is held"
	);

	// And it does run in a fight, so the test above means something.
	let mut fighting = sim_for(SimFor::default());

	run_until(&mut fighting, HuntPhase::Fight, Some(steady_on_husk));

	let before = fighting.fight_ticks_left();

	for _ in 0..10 {
		let input = steady_on_husk(&fighting);

		fighting.step(&input);
	}

	assert_eq!(fighting.fight_ticks_left(), before - 10);
}

#[test]
fn a_hold_too_short_to_time_is_thrown_ungraded() {
	let mut sim = broken();
	let thrown = throw_orb(&mut sim, 3, None)
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::OrbThrown { grade, on_target } => Some((*grade, *on_target)),
				_ => None,
			}
		})
		.unwrap();

	assert_eq!(thrown, (CaptureGrade::Plain, true));
}

#[test]
fn an_orb_thrown_wide_is_lost_and_the_soul_with_it() {
	let mut sim = broken();
	let soul = sim.soul();
	let wide = Vec3::from_angles(sim.pose().yaw() + 0.6, sim.pose().pitch());
	let thrown = throw_orb(&mut sim, 50, Some(wide))
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::OrbThrown { grade, on_target } => Some((*grade, *on_target)),
				_ => None,
			}
		})
		.unwrap();

	assert_eq!(thrown, (CaptureGrade::Plain, false));
	assert_eq!(sim.soul(), soul - CaptureRules::COST);
	assert_eq!(sim.phase(), HuntPhase::Broken);
	assert_eq!(sim.orb_block(), Some(OrbBlock::InFlight));

	let after = follow_orb(&mut sim, CaptureRules::IMPACT_TICK);
	let impact = &after[CaptureRules::IMPACT_TICK as usize - 1].1;

	assert_eq!(count!(impact, HuntEvent::OrbMissed), 1);
	assert_eq!(sim.orb().unwrap().stage, OrbStage::Missed);
	assert_eq!(sim.orb_block(), None);
}

#[test]
fn an_orb_a_little_off_the_body_still_strikes_it() {
	let mut sim = broken();
	let pose = sim.pose();
	let mut edge = 0.0;

	while pose.covers(
		reapers_rules::Vec3::ZERO,
		Vec3::from_angles(pose.yaw() + edge, pose.pitch()),
		0.0,
	) {
		edge += 0.001;
	}

	let near = Vec3::from_angles(
		pose.yaw() + edge + sim.stats().orb_aim_radians * 0.8,
		pose.pitch(),
	);
	let thrown = throw_orb(&mut sim, 50, Some(near))
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::OrbThrown { grade, on_target } => Some((*grade, *on_target)),
				_ => None,
			}
		})
		.unwrap();

	assert!(thrown.1);
	assert_eq!(thrown.0, CaptureGrade::Plain, "off the husk itself");
}

#[test]
fn pressing_fire_or_letting_go_of_orb_in_a_shriek_is_a_flinch() {
	let mut sim = sim_for(SimFor::default());

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));
	run_until(&mut sim, HuntPhase::Shriek, Some(steady_on_husk));
	step_for(&mut sim, HuntRules::SHRIEK_GRACE_TICKS, None);

	let events = throw_orb(&mut sim, 2, None);

	assert_eq!(count!(events, HuntEvent::Flinched { .. }), 1);
	assert_eq!(count!(events, HuntEvent::OrbThrown { .. }), 0);
}

// the capture ----------------------------------------------------------

fn pulses_in(events: &[HuntEvent]) -> Vec<i64> {
	events
		.iter()
		.filter_map(|event| {
			match event {
				HuntEvent::OrbPulse(pulse) => Some(*pulse),
				_ => None,
			}
		})
		.collect()
}

fn impact_of(after: &[(i64, Vec<HuntEvent>)]) -> Option<f64> {
	after[CaptureRules::IMPACT_TICK as usize - 1]
		.1
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::OrbImpact { chance } => Some(*chance),
				_ => None,
			}
		})
}

#[test]
fn an_orb_that_holds_follows_the_timeline_to_the_end() {
	let chance = broken_chance(&broken(), CaptureGrade::Excellent);
	let mut sim = broken_with_seed(seed_breaking_on(None, chance));
	let thrown_at = sim.tick();

	throw_orb(&mut sim, 50, None);

	assert_eq!(sim.orb().unwrap().thrown_at, thrown_at + 50);

	let after = follow_orb(&mut sim, CaptureRules::RESULT_TICK);
	let at = |tick: i64| -> Vec<&'static str> {
		after
			.iter()
			.find(|(after_tick, _)| *after_tick == tick)
			.map_or_else(Vec::new, |(_, events)| {
				events.iter().map(HuntEvent::kind).collect()
			})
	};

	assert_eq!(at(CaptureRules::IMPACT_TICK), vec!["OrbImpact"]);
	assert!((impact_of(&after).unwrap() - chance).abs() < 1e-9);
	assert_eq!(at(CaptureRules::DRAWN_TICK), vec!["OrbAbsorbed"]);
	assert_eq!(at(CaptureRules::LANDED_TICK), vec!["OrbLanded", "OrbPulse"]);
	assert_eq!(at(CaptureRules::PULSE_TICKS[1]), vec!["OrbPulse"]);
	assert_eq!(at(CaptureRules::PULSE_TICKS[2]), vec!["OrbPulse"]);
	assert_eq!(at(CaptureRules::RESULT_TICK), vec!["HuskCaptured"]);

	let all_pulses: Vec<i64> = after
		.iter()
		.flat_map(|(_, events)| pulses_in(events))
		.collect();

	assert_eq!(all_pulses, vec![1, 2, 3]);
	assert!(sim.outcome().unwrap().absorbed());
	assert_eq!(
		sim.outcome().unwrap().grade(),
		Some(CaptureGrade::Excellent)
	);
	assert_eq!(sim.orb().unwrap().stage, OrbStage::Captured);
}

#[test]
fn an_orb_that_fails_breaks_on_the_pulse_the_roll_says_first() {
	let pulse = 1;
	let chance = broken_chance(&broken(), CaptureGrade::Excellent);
	let mut sim = broken_with_seed(seed_breaking_on(Some(pulse), chance));

	throw_orb(&mut sim, 50, None);

	let after = follow_orb(&mut sim, CaptureRules::RESULT_TICK);
	let burst = CaptureRules::PULSE_TICKS[(pulse - 1) as usize];
	let burst_events = &after[burst as usize - 1].1;

	assert_eq!(count!(burst_events, HuntEvent::OrbBrokeFree(_)), 1);
	assert_eq!(pulses_in(burst_events), Vec::<i64>::new());
	assert!(
		!after
			.iter()
			.any(|(_, events)| count!(events, HuntEvent::HuskCaptured(_)) > 0)
	);
	assert_eq!(sim.orb().unwrap().broke_free_on, Some(pulse));
}

#[test]
fn an_orb_that_fails_breaks_on_the_pulse_the_roll_says_second() {
	let pulse = 2;
	let chance = broken_chance(&broken(), CaptureGrade::Excellent);
	let mut sim = broken_with_seed(seed_breaking_on(Some(pulse), chance));

	throw_orb(&mut sim, 50, None);

	let after = follow_orb(&mut sim, CaptureRules::RESULT_TICK);
	let burst = CaptureRules::PULSE_TICKS[(pulse - 1) as usize];
	let burst_events = &after[burst as usize - 1].1;

	assert_eq!(count!(burst_events, HuntEvent::OrbBrokeFree(_)), 1);

	let all_pulses: Vec<i64> = after
		.iter()
		.flat_map(|(_, events)| pulses_in(events))
		.collect();

	assert_eq!(all_pulses, vec![1]);
	assert_eq!(sim.orb().unwrap().broke_free_on, Some(pulse));
}

#[test]
fn an_orb_that_fails_breaks_on_the_pulse_the_roll_says_third() {
	let pulse = 3;
	let chance = broken_chance(&broken(), CaptureGrade::Excellent);
	let mut sim = broken_with_seed(seed_breaking_on(Some(pulse), chance));

	throw_orb(&mut sim, 50, None);

	let after = follow_orb(&mut sim, CaptureRules::RESULT_TICK);
	let burst = CaptureRules::PULSE_TICKS[(pulse - 1) as usize];
	let burst_events = &after[burst as usize - 1].1;

	assert_eq!(count!(burst_events, HuntEvent::OrbBrokeFree(_)), 1);

	let all_pulses: Vec<i64> = after
		.iter()
		.flat_map(|(_, events)| pulses_in(events))
		.collect();

	assert_eq!(all_pulses, vec![1, 2]);
	assert_eq!(sim.orb().unwrap().broke_free_on, Some(pulse));
}

#[test]
fn a_husk_that_breaks_out_stands_stunned_a_little_warded() {
	let chance = broken_chance(&broken(), CaptureGrade::Excellent);
	let mut sim = broken_with_seed(seed_breaking_on(Some(1), chance));

	throw_orb(&mut sim, 50, None);
	follow_orb(&mut sim, CaptureRules::PULSE_TICKS[0]);

	assert_eq!(sim.phase(), HuntPhase::Fight);
	assert_eq!(sim.failures(), 1);
	assert!((sim.ward() - sim.ward_max() * CaptureRules::FAILURE_WARD).abs() < 1e-9);
	assert!(sim.stunned());
	assert_eq!(sim.stun_ticks_left(), CaptureRules::FAILURE_STUN_TICKS - 1);
	assert_eq!(sim.ring(), HuskRing::Stunned);
}

#[test]
fn breaking_out_never_leaves_a_husk_more_than_weakened() {
	let mut standing = sim_for(SimFor::default());

	run_until(&mut standing, HuntPhase::Fight, Some(steady_on_husk));

	let share = CaptureRules::WEAKENED_SHARE - 0.05;
	let chance = CaptureRules::chance(
		standing.rarity_odds(),
		share,
		CaptureGrade::Excellent,
		standing.stats().capture,
		0,
	);
	let mut sim = sim_for(SimFor {
		seed: seed_breaking_on(Some(1), chance),
		..SimFor::default()
	});

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));
	sim.set_ward(sim.ward_max() * share);
	sim.set_soul(CaptureRules::SOUL_MAX);
	throw_orb(&mut sim, 50, None);

	assert_eq!(sim.phase(), HuntPhase::Capture, "thrown at a standing husk");

	let after = follow_orb(&mut sim, CaptureRules::PULSE_TICKS[0]);

	assert!((impact_of(&after).unwrap() - chance).abs() < 1e-9);
	assert_eq!(sim.phase(), HuntPhase::Fight);
	assert!((sim.ward() - sim.ward_max() * CaptureRules::WEAKENED_SHARE).abs() < 1e-9);
}

#[test]
fn a_husk_held_by_an_orb_goes_nowhere() {
	let mut sim = broken();

	throw_orb(&mut sim, 50, None);

	let at = sim.pose().position;

	follow_orb(&mut sim, CaptureRules::LANDED_TICK);

	assert_eq!(sim.pose().position, at);
	assert_eq!(sim.phase(), HuntPhase::Capture);
}

// the broken window ----------------------------------------------------

#[test]
fn the_broken_window_lasts_as_long_as_the_class_rates_capture() {
	let mut sim = sim_for(SimFor::default());
	let mut bot = HuntBot::careful(&sim);
	let mut events = Vec::new();

	while sim.phase() != HuntPhase::Broken {
		events.extend(bot.step_once(&mut sim));
	}

	let expected =
		((3.5 + 0.5 * sim.stats().capture as f64) * HuntRules::TICK_RATE as f64).round() as i64;

	assert_eq!(sim.stats().broken_ticks, expected);
	assert_eq!(
		events.iter().find_map(|event| {
			match event {
				HuntEvent::Broken { window_ticks } => Some(*window_ticks),
				_ => None,
			}
		}),
		Some(sim.broken_ticks())
	);
	assert_eq!(sim.broken_ticks_left(), sim.broken_ticks() - 1);
}

#[test]
fn left_alone_the_husk_recovers_and_then_escapes() {
	let mut sim = broken();
	let recovered = run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));

	assert_eq!(
		recovered.iter().find_map(|event| {
			match event {
				HuntEvent::Recovered { lapses_left } => Some(*lapses_left),
				_ => None,
			}
		}),
		Some(1)
	);
	assert_eq!(sim.ward(), sim.ward_max() * HuntRules::RECOVER_SHARE);

	// A bot that never lets go of ORB cannot throw a second one, so the
	// next window passes and the husk escapes.
	let mut bot = HuntBot::new(
		&sim,
		HuntBotStyle {
			orb_hold_ticks: 9999,
			..HuntBotStyle::CAREFUL
		},
	);

	while sim.phase() != HuntPhase::Broken {
		bot.step_once(&mut sim);
	}

	run_until(&mut sim, HuntPhase::Escaped, Some(steady_on_husk));

	assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Lapsed));
}

#[test]
fn an_easy_hunt_lets_the_husk_get_up_twice_more() {
	assert_eq!(
		sim_for(SimFor {
			difficulty: Difficulty::Easy,
			..SimFor::default()
		})
		.lapses_left(),
		4
	);
	assert_eq!(sim_for(SimFor::default()).lapses_left(), 2);
	assert_eq!(
		sim_for(SimFor {
			difficulty: Difficulty::Hard,
			..SimFor::default()
		})
		.lapses_left(),
		2
	);
}

// in a rift ------------------------------------------------------------

#[test]
fn an_orb_at_the_broken_ward_is_free_and_always_holds() {
	let vessel = Vessel::ALL
		.iter()
		.copied()
		.find(|vessel| vessel.is_solo())
		.unwrap();
	let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
	let mut sim = HuntSim::rift(5, vessel, 1000.0, &stats, None);

	assert_eq!(sim.orb_block(), Some(OrbBlock::NotWeakened));

	sim.share_ward(0.0);

	let step = sim.step(&steady_on_husk(&sim));

	assert_eq!(count!(step, HuntEvent::Broken { .. }), 1);
	assert!(sim.soul() < CaptureRules::COST);
	assert_eq!(sim.orb_block(), None);
	assert_eq!(sim.broken_ticks(), RiftRules::REAP_WINDOW_TICKS);

	let soul = sim.soul();

	throw_orb(&mut sim, 30, None);

	let after = follow_orb(&mut sim, CaptureRules::IMPACT_TICK);
	let impact = &after[CaptureRules::IMPACT_TICK as usize - 1].1;

	assert_eq!(
		impact.iter().find_map(|event| {
			match event {
				HuntEvent::OrbImpact { chance } => Some(*chance),
				_ => None,
			}
		}),
		Some(1.0)
	);
	assert_eq!(
		impact.iter().find_map(|event| {
			match event {
				HuntEvent::HuskCaptured(grade) => Some(*grade),
				_ => None,
			}
		}),
		Some(CaptureGrade::Great)
	);
	assert_eq!(
		sim.outcome().unwrap().grade().map(CaptureGrade::cut),
		Some(ReapGrade::Clean)
	);
	assert_eq!(sim.soul(), soul);
}

// the ring -------------------------------------------------------------

#[test]
fn the_ring_says_what_the_husk_is_doing() {
	let mut sim = sim_for(SimFor::default());

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));

	assert_eq!(sim.ring(), HuskRing::Open);

	run_until(&mut sim, HuntPhase::Windup, Some(steady_on_husk));

	assert_eq!(sim.ring(), HuskRing::DrawingBreath);

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));
	run_until_with_limit(&mut sim, HuntPhase::Charge, Some(steady_on_husk), 6000);

	assert_eq!(sim.ring(), HuskRing::Charging);

	run_till(&mut sim, &HuntSim::stunned, Some(wary_on_husk));

	assert_eq!(sim.ring(), HuskRing::Stunned);

	run_till(
		&mut sim,
		&|sim: &HuntSim| !sim.stunned(),
		Some(steady_on_husk),
	);
	sim.set_ward(sim.ward_max() * CaptureRules::WEAKENED_SHARE);

	assert_eq!(sim.ring(), HuskRing::Weakened);
}
