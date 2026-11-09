//! The crosshair's four weapons: the pistol, the knife, the class weapon.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]
#![allow(
	clippy::assert_is_empty,
	reason = "asserting an empty event list reads clearer than an equality"
)]

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

// what an event says, read out of the tick it happened in ---------------

/// The gun's press: rounds left, and whether the magazine was boosted.
fn fired(events: &[HuntEvent]) -> (i64, bool) {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::ShotFired {
					rounds_left,
					boosted,
				} => Some((*rounds_left, *boosted)),
				_ => None,
			}
		})
		.unwrap()
}

/// The hit: weapon, seal, damage, cadence, critical, empowered.
fn landed(events: &[HuntEvent]) -> (Archetype, Option<usize>, f64, i64, bool, bool) {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::HitLanded {
					weapon,
					seal,
					damage,
					cadence,
					critical,
					empowered,
					..
				} => Some((*weapon, *seal, *damage, *cadence, *critical, *empowered)),
				_ => None,
			}
		})
		.unwrap()
}

fn reload_perfect(events: &[HuntEvent]) -> bool {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::ReloadFinished { perfect } => Some(*perfect),
				_ => None,
			}
		})
		.unwrap()
}

fn drawn_slot(events: &[HuntEvent]) -> usize {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::WeaponDrawn(slot) => Some(*slot),
				_ => None,
			}
		})
		.unwrap()
}

fn stalk_ambush(events: &[HuntEvent]) -> bool {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::StalkEnded { ambush } => Some(*ambush),
				_ => None,
			}
		})
		.unwrap()
}

fn missed_weapon(events: &[HuntEvent]) -> Archetype {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::Missed(weapon) => Some(*weapon),
				_ => None,
			}
		})
		.unwrap()
}

fn dive_weapon(events: &[HuntEvent]) -> Archetype {
	events
		.iter()
		.find_map(|event| {
			match event {
				HuntEvent::DiveStarted(weapon) => Some(*weapon),
				_ => None,
			}
		})
		.unwrap()
}

// the fights the tests are fought in ------------------------------------

/// A fight a ferryman began with an ambush: every seal in view is open, and
/// nothing a hit does cleaves.
fn ambushed() -> HuntSim {
	let mut sim = sim_for(SimFor {
		reaper_class: ReaperClass::Ferryman,
		..SimFor::default()
	});

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));

	sim
}

/// A fight a ferryman began by startling the husk: no ambush, and no seal
/// open.
fn startled() -> HuntSim {
	let mut sim = sim_for(SimFor {
		reaper_class: ReaperClass::Ferryman,
		..SimFor::default()
	});

	run_until(
		&mut sim,
		HuntPhase::Fight,
		Some(|_| HuntInput::new(Aim::AHEAD, 6000)),
	);

	assert!(!sim.ambushed());

	sim
}

/// What a hit worth `share` of the reaper's blow does with nothing but the
/// cadence behind it.
fn plain_damage(sim: &HuntSim, share: f64, critical: bool) -> f64 {
	sim.stats().hit
		* share
		* (1.0 + sim.cadence().clamp(0, HuntRules::CADENCE_CAP) as f64 * HuntRules::CADENCE_STEP)
		* if critical {
			HuntRules::CRITICAL_STRIKE
		} else {
			1.0
		}
}

/// Fires the gun at the husk until it is empty, and presses once more to
/// start the reload.
fn start_reload(sim: &mut HuntSim) {
	run_till(
		sim,
		&HuntSim::reloading,
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);
	assert!(!sim.is_over(), "the gun never ran dry");
}

/// Holds `slot` until it is drawn.
fn draw(sim: &mut HuntSim, slot: usize) {
	for _ in 0..=Arsenal::SWITCH_TICKS {
		let input = HuntInput::measured(sim.pose().direction(), 0.0, false, slot, false);

		sim.step(&input);
	}

	assert_eq!(sim.selected_slot(), slot);
	assert_eq!(sim.draw_ticks_left(), 0);
}

/// Looks at the husk, and swings at a charge only at the last moment, so
/// that it is stopped within arm's reach.
fn parry_late(sim: &HuntSim) -> HuntInput {
	HuntInput::measured(
		sim.pose().direction(),
		if sim.charge_progress() > 0.9 {
			sim.stats().parry_speed + 0.5
		} else {
			0.0
		},
		false,
		sim.selected_slot(),
		false,
	)
}

/// The way to a point just off the side of the husk's body: `beyond`
/// radians past its edge.
fn off_the_edge(sim: &HuntSim, beyond: f64) -> Vec3 {
	let pose = sim.pose();
	let mut inside = 0.0;
	let mut outside = 0.5;

	for _ in 0..40 {
		let middle = f64::midpoint(inside, outside);

		if pose.covers(
			reapers_rules::Vec3::ZERO,
			Vec3::from_angles(pose.yaw() + middle, pose.pitch()),
			0.0,
		) {
			inside = middle;
		} else {
			outside = middle;
		}
	}

	Vec3::from_angles(pose.yaw() + outside + beyond, pose.pitch())
}

/// Waits until the husk is fighting, out of arm's reach but within `rule`
/// metres of it.
fn within_dive_of(sim: &mut HuntSim, rule: f64) {
	run_till(
		sim,
		&|sim: &HuntSim| {
			sim.phase() == HuntPhase::Fight
				&& sim.pose().distance() > Arsenal::CLOSE_METRES
				&& sim.pose().distance() <= rule
		},
		Some(steady_on_husk),
	);
}

#[test]
fn a_shot_on_an_open_seal_is_critical_and_shuts_it() {
	let mut sim = ambushed();
	let seal = open_seals(&sim)[0];
	let events = sim.step(&fire_at(
		&sim,
		sim.pose().seal_direction(reapers_rules::Vec3::ZERO, seal),
		None,
	));
	let (weapon, hit_seal, damage, _cadence, critical, empowered) = landed(&events);

	assert_eq!(fired(&events).0, 5);
	assert_eq!(weapon, Archetype::Gun);
	assert_eq!(hit_seal, Some(seal));
	assert!(critical);
	assert!(empowered, "the ambush shot");
	assert!(
		(damage
			- sim.stats().hit
				* GunRules::SHOT_STRIKE
				* HuntRules::CRITICAL_STRIKE
				* HuntRules::AMBUSH_STRIKE)
			.abs()
			< 1e-9
	);
	assert!((sim.ward() - (sim.ward_max() - damage)).abs() < 1e-9);
	assert!(!sim.seal_open(seal));
}

#[test]
fn a_shot_near_an_open_seal_counts_as_on_it() {
	let mut sim = ambushed();
	let seal = sim
		.pose()
		.seal_direction(reapers_rules::Vec3::ZERO, open_seals(&sim)[0]);
	let near = Vec3::from_angles(seal.yaw(), seal.pitch() + sim.stats().reach_radians * 0.6);

	assert!(sim.on_open_seal(near));

	let events = sim.step(&fire_at(&sim, near, None));

	assert!(landed(&events).4, "critical");
}

#[test]
fn a_shot_at_the_body_is_half_a_critical_and_gives_the_stalk_away() {
	let mut sim = sim_for(SimFor {
		reaper_class: ReaperClass::Ferryman,
		..SimFor::default()
	});
	let events = sim.step(&fire_at(&sim, sim.pose().direction(), None));
	let (_weapon, seal, damage, _cadence, critical, _empowered) = landed(&events);

	assert!(!stalk_ambush(&events));
	assert_eq!(seal, None);
	assert!(!critical);
	assert!((damage - sim.stats().hit * GunRules::SHOT_STRIKE).abs() < 1e-9);
}

#[test]
fn a_shot_just_off_the_body_is_pulled_onto_it_and_further_off_misses() {
	let assist = sim_for(SimFor::default()).stats().assist_radians;
	let mut near = sim_for(SimFor::default());
	let mut far = sim_for(SimFor::default());

	assert_eq!(
		count!(
			near.step(&fire_at(&near, off_the_edge(&near, assist * 0.5), None)),
			HuntEvent::HitLanded { .. }
		),
		1
	);
	assert_eq!(
		missed_weapon(&far.step(&fire_at(&far, off_the_edge(&far, assist * 1.5), None))),
		Archetype::Gun
	);
	assert_eq!(far.phase(), HuntPhase::Fight, "a miss is heard too");
}

#[test]
fn the_pistol_fires_no_faster_than_its_interval() {
	let mut sim = ambushed();
	let mut shots = Vec::new();

	for _ in 0..GunRules::FIRE_INTERVAL_TICKS * 3 {
		let at = sim.tick();
		let events = sim.step(&fire_at(&sim, sim.pose().direction(), None));

		if count!(events, HuntEvent::ShotFired { .. }) > 0 {
			shots.push(at);
		}
	}

	assert_eq!(shots.len(), 3);
	assert_eq!(shots[1] - shots[0], GunRules::FIRE_INTERVAL_TICKS);
	assert_eq!(shots[2] - shots[1], GunRules::FIRE_INTERVAL_TICKS);
	assert!(sim.cooldown_ticks() < GunRules::FIRE_INTERVAL_TICKS);
}

#[test]
fn the_pistol_holds_six_rounds_and_fire_on_empty_reloads_it() {
	let mut sim = ambushed();
	let events = run_till(
		&mut sim,
		&HuntSim::reloading,
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);
	let started = sim.tick() - 1;

	assert_eq!(
		count!(events, HuntEvent::ShotFired { .. }),
		GunRules::ROUNDS as usize
	);
	assert_eq!(count!(events, HuntEvent::ReloadStarted), 1);
	assert_eq!(sim.rounds(), 0);
	assert_eq!(sim.cooldown_ticks(), GunRules::RELOAD_TICKS - 1);

	let reload = run_till(
		&mut sim,
		&|sim: &HuntSim| !sim.reloading(),
		Some(steady_on_husk),
	);

	assert_eq!(sim.tick() - 1 - started, GunRules::RELOAD_TICKS);
	assert!(!reload_perfect(&reload));
	assert_eq!(sim.rounds(), GunRules::ROUNDS);
	assert!(!sim.magazine_boosted());
}

#[test]
fn fire_in_the_active_window_reloads_at_once_and_harder() {
	let mut sim = ambushed();

	start_reload(&mut sim);

	let started = sim.tick() - 1;

	run_till(&mut sim, &HuntSim::in_active_reload, Some(steady_on_husk));

	assert_eq!(sim.tick() - started, GunRules::ACTIVE_FROM_TICK);

	let finished = sim.step(&steady_on_husk(&sim));

	assert!(finished.is_empty(), "the window waits for a press");

	let reload = sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert!(reload_perfect(&reload));
	assert_eq!(sim.rounds(), GunRules::ROUNDS);
	assert!(sim.magazine_boosted());

	let critical = sim.on_open_seal(sim.pose().direction());
	let expected = plain_damage(
		&sim,
		GunRules::SHOT_STRIKE * GunRules::BOOSTED_STRIKE,
		critical,
	);
	let shot = sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert!(fired(&shot).1, "boosted");
	assert!((landed(&shot).2 - expected).abs() < 1e-9);
}

#[test]
fn the_active_window_closes_after_its_last_tick() {
	let mut sim = ambushed();

	start_reload(&mut sim);

	let started = sim.tick() - 1;

	run_till(
		&mut sim,
		&|sim: &HuntSim| sim.tick() - started > GunRules::ACTIVE_UNTIL_TICK,
		Some(steady_on_husk),
	);

	assert!(!sim.in_active_reload());
	assert!(
		sim.step(&fire_at(&sim, sim.pose().direction(), None))
			.is_empty()
	);
	assert!(sim.reloading());
}

#[test]
fn fire_before_the_window_jams_the_reload() {
	let mut sim = ambushed();

	start_reload(&mut sim);

	let started = sim.tick() - 1;
	let jammed = sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert_eq!(count!(jammed, HuntEvent::ReloadJammed), 1);
	assert!(sim.reload_jammed());

	let rest = run_till(
		&mut sim,
		&|sim: &HuntSim| !sim.reloading(),
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);

	assert_eq!(count!(rest, HuntEvent::ReloadJammed), 0);
	assert!(!reload_perfect(&rest));
	assert_eq!(
		sim.tick() - 1 - started,
		GunRules::RELOAD_TICKS + GunRules::JAM_TICKS
	);
}

#[test]
fn the_gun_reloads_with_another_weapon_in_hand() {
	let mut sim = ambushed();

	start_reload(&mut sim);
	draw(&mut sim, Arsenal::BLADE_SLOT);

	let events = step_for(&mut sim, GunRules::RELOAD_TICKS, Some(steady_on_husk));

	assert_eq!(count!(events, HuntEvent::ReloadFinished { .. }), 1);
	assert_eq!(sim.rounds(), GunRules::ROUNDS);
}

#[test]
fn another_slot_takes_a_moment_to_draw_and_fires_nothing_until_then() {
	let mut sim = ambushed();
	let events = sim.step(&fire_at(
		&sim,
		sim.pose().direction(),
		Some(Arsenal::BLADE_SLOT),
	));

	assert_eq!(drawn_slot(&events), Arsenal::BLADE_SLOT);
	assert_eq!(count!(events, HuntEvent::HitLanded { .. }), 0);
	assert_eq!(sim.draw_ticks_left(), Arsenal::SWITCH_TICKS - 1);
	assert_eq!(sim.cooldown_ticks(), Arsenal::SWITCH_TICKS - 1);
}

#[test]
fn the_four_slots_are_the_pistol_the_knife_the_class_weapon_and_the_art() {
	let archetypes: Vec<_> = (0..Arsenal::SLOTS).map(Arsenal::archetype_of).collect();

	assert_eq!(
		archetypes,
		[
			Some(Archetype::Gun),
			Some(Archetype::Blade),
			Some(Archetype::Heavy),
			Some(Archetype::Art),
		]
	);
}

// the bone knife -------------------------------------------------------

#[test]
fn the_knife_dives_at_a_husk_within_reach_and_stabs_as_it_arrives() {
	let mut sim = ambushed();

	draw(&mut sim, Arsenal::BLADE_SLOT);
	within_dive_of(&mut sim, BladeRules::DIVE_METRES);

	let started = sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert_eq!(dive_weapon(&started), Archetype::Blade);
	assert_eq!(count!(started, HuntEvent::HitLanded { .. }), 0);
	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES - 1);

	// The first stab lands as the reaper arrives, unasked.
	let arrival = step_for(&mut sim, BladeRules::IN_TICKS, None);

	assert_eq!(landed(&arrival).0, Archetype::Blade);

	// Two more presses in the window stab twice; a third does nothing.
	let window = step_for(
		&mut sim,
		BladeRules::WINDOW_TICKS - 1,
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);

	assert_eq!(
		count!(window, HuntEvent::HitLanded { .. }),
		(BladeRules::STABS_PER_DIVE - 1) as usize
	);

	let out = step_for(&mut sim, BladeRules::OUT_TICKS, None);

	assert_eq!(count!(out, HuntEvent::DiveEnded), 1);
	assert!(!sim.diving());
}

#[test]
fn a_dive_spends_a_charge_and_it_comes_back_in_three_seconds() {
	let mut sim = ambushed();

	draw(&mut sim, Arsenal::BLADE_SLOT);
	within_dive_of(&mut sim, BladeRules::DIVE_METRES);
	sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES - 1);

	step_for(&mut sim, BladeRules::CHARGE_TICKS - 1, None);

	assert!(sim.dive_recharge() > 0.9);

	step_for(&mut sim, 1, None);

	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES);
	assert_eq!(sim.dive_recharge(), 0.0);
}

#[test]
fn with_no_charge_left_the_knife_cannot_dive() {
	let mut sim = ambushed();

	draw(&mut sim, Arsenal::BLADE_SLOT);
	within_dive_of(&mut sim, BladeRules::DIVE_METRES);
	sim.set_dive_charges(0);

	assert!(!sim.in_reach(sim.pose().direction()));
	assert!(
		sim.step(&fire_at(&sim, sim.pose().direction(), None))
			.is_empty()
	);
}

#[test]
fn the_knife_does_not_dive_at_a_husk_beyond_its_reach() {
	let mut sim = ambushed();

	draw(&mut sim, Arsenal::BLADE_SLOT);

	let away = Vec3::from_angles(sim.pose().yaw() + 1.0, sim.pose().pitch());

	assert!(!sim.in_reach(away));
	assert!(sim.step(&fire_at(&sim, away, None)).is_empty());
	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES);
}

#[test]
fn in_arms_reach_every_press_stabs_critically_and_spends_nothing() {
	let mut sim = startled();

	run_till(
		&mut sim,
		&|sim: &HuntSim| sim.stunned() && sim.pose().distance() <= Arsenal::CLOSE_METRES,
		Some(parry_late),
	);
	draw(&mut sim, Arsenal::BLADE_SLOT);

	assert!(sim.stunned());

	let body = body_away_from_seals(&sim).unwrap();
	let expected = plain_damage(&sim, BladeRules::STAB_STRIKE, true);
	let stab = sim.step(&fire_at(&sim, body, None));
	let (_weapon, seal, damage, _cadence, critical, _empowered) = landed(&stab);

	assert_eq!(count!(stab, HuntEvent::DiveStarted(_)), 0);
	assert_eq!(seal, None, "the body, not a seal");
	assert!(critical, "a stab on a stunned husk");
	assert!((damage - expected).abs() < 1e-9);
	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES);

	let again = step_for(
		&mut sim,
		BladeRules::STAB_INTERVAL_TICKS,
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);

	assert_eq!(count!(again, HuntEvent::HitLanded { .. }), 1);
}

// the class weapon -----------------------------------------------------

#[test]
fn the_class_weapon_dives_and_swings_once_knocking_the_husk_still() {
	let mut sim = ambushed();

	draw(&mut sim, Arsenal::HEAVY_SLOT);
	run_until_with_limit(&mut sim, HuntPhase::Fight, Some(steady_on_husk), 6000);
	run_till(
		&mut sim,
		&|sim: &HuntSim| {
			sim.pose().distance() > Arsenal::CLOSE_METRES
				&& sim.pose().distance() <= HeavyRules::DIVE_METRES
		},
		Some(steady_on_husk),
	);

	let started = sim.step(&fire_at(&sim, sim.pose().direction(), None));

	assert_eq!(dive_weapon(&started), Archetype::Heavy);
	assert_eq!(sim.dive_charges(), BladeRules::DIVE_CHARGES, "no charge");

	let swing_ticks = sim.stats().swing_ticks;
	let swing = step_for(
		&mut sim,
		HeavyRules::IN_TICKS + swing_ticks,
		Some(|sim: &HuntSim| fire_at(sim, sim.pose().direction(), None)),
	);

	assert_eq!(landed(&swing).0, Archetype::Heavy);
	assert!(sim.staggered());

	let out = step_for(&mut sim, HeavyRules::OUT_TICKS, None);

	assert_eq!(count!(out, HuntEvent::DiveEnded), 1);
	assert_eq!(count!(out, HuntEvent::HitLanded { .. }), 0);
}

#[test]
fn a_heavy_blow_to_the_body_does_its_weight_and_to_an_open_seal_twice_that() {
	let mut sim = startled();

	run_till(
		&mut sim,
		&|sim: &HuntSim| sim.stunned() && sim.pose().distance() <= Arsenal::CLOSE_METRES,
		Some(parry_late),
	);
	draw(&mut sim, Arsenal::HEAVY_SLOT);

	let body = body_away_from_seals(&sim).unwrap();
	let expected = plain_damage(&sim, sim.stats().heavy_strike, false)
		* HuntRules::CRITICAL_STRIKE
		* sim.stats().body_share;
	let (_weapon, seal, damage, _cadence, critical, _empowered) =
		landed(&sim.step(&fire_at(&sim, body, None)));

	assert_eq!(seal, None);
	assert!(!critical, "only the knife crits a stun");
	assert!((damage - expected).abs() < 1e-9);

	let swing_ticks = sim.stats().swing_ticks;
	step_for(&mut sim, swing_ticks - 1, None);
	let seal = open_seals(&sim)[0];
	let sealed_expected = plain_damage(&sim, sim.stats().heavy_strike, true);
	let on_seal = sim.step(&fire_at(
		&sim,
		sim.pose().seal_direction(reapers_rules::Vec3::ZERO, seal),
		None,
	));

	assert_eq!(landed(&on_seal).1, Some(seal));
	assert!((landed(&on_seal).2 - sealed_expected).abs() < 1e-9);
}

// soul energy ----------------------------------------------------------

#[test]
fn every_hit_turns_the_ward_it_takes_into_soul() {
	let mut sim = ambushed();
	let events = sim.step(&fire_at(&sim, sim.pose().direction(), None));
	let (_weapon, _seal, damage, _cadence, _critical, _empowered) = landed(&events);

	assert!((sim.soul() - CaptureRules::SOUL_PER_WARD * damage / sim.ward_max()).abs() < 1e-9);
}

#[test]
fn an_unflinching_hold_and_a_parry_each_add_a_little() {
	let mut sim = sim_for(SimFor::default());

	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));
	run_until(&mut sim, HuntPhase::Shriek, Some(steady_on_husk));
	run_until(&mut sim, HuntPhase::Fight, Some(steady_on_husk));

	assert_eq!(sim.soul(), CaptureRules::HOLD_SOUL);

	let parried = run_till(&mut sim, &HuntSim::stunned, Some(wary_on_husk));

	assert_eq!(count!(parried, HuntEvent::Parried), 1);
	assert!(sim.soul() >= CaptureRules::HOLD_SOUL + CaptureRules::PARRY_SOUL);
}

#[test]
fn breaking_a_ward_fills_the_soul_and_it_stops_at_a_hundred() {
	let mut sim = sim_for(SimFor::default());
	let mut bot = HuntBot::careful(&sim);

	while sim.phase() != HuntPhase::Broken {
		bot.step_once(&mut sim);
	}

	assert_eq!(sim.soul(), CaptureRules::SOUL_MAX);
}
