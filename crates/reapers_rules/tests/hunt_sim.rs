//! The hunt, a tick at a time.

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

/// A module named after `check`, with one test in it for every vessel a lone
/// reaper hunts (every [`Vessel`] that [`Vessel::is_solo`]). Each test hands
/// its vessel to the function `check`.
macro_rules! solo_vessel_tests {
	($check:ident) => {
		mod $check {
			use super::*;

			// Every solo vessel has a test here.
			const _: () = {
				let mut solo = 0;
				let mut index = 0;

				while index < Vessel::ALL.len() {
					if Vessel::ALL[index].is_solo() {
						solo += 1;
					}

					index += 1;
				}

				assert!(solo == 5);
			};

			#[test]
			fn ape() {
				$check(Vessel::Ape);
			}

			#[test]
			fn bird() {
				$check(Vessel::Bird);
			}

			#[test]
			fn penguin() {
				$check(Vessel::Penguin);
			}

			#[test]
			fn llama() {
				$check(Vessel::Llama);
			}

			#[test]
			fn robot() {
				$check(Vessel::Robot);
			}
		}
	};
}

/// Steps `sim` with `input` until `phase` is reached or `limit` ticks pass.
fn run_until_within(
	sim: &mut HuntSim,
	phase: HuntPhase,
	mut input: impl FnMut(&HuntSim) -> HuntInput,
	limit: usize,
) -> Vec<HuntEvent> {
	let mut events = Vec::new();
	let mut ticks = 0;

	while ticks < limit && sim.phase() != phase {
		let next = input(sim);

		events.extend(sim.step(&next));
		ticks += 1;
	}

	assert_eq!(sim.phase(), phase, "never reached {phase}");

	events
}

/// [`run_until_within`] with the usual limit of 4000 ticks.
fn run_until(
	sim: &mut HuntSim,
	phase: HuntPhase,
	input: impl FnMut(&HuntSim) -> HuntInput,
) -> Vec<HuntEvent> {
	run_until_within(sim, phase, input, 4000)
}

/// Steps `sim` once with what `choose` makes of it.
fn step_with(sim: &mut HuntSim, choose: impl FnOnce(&HuntSim) -> HuntInput) -> Vec<HuntEvent> {
	let input = choose(sim);

	sim.step(&input)
}

/// Looks straight at the husk without moving.
fn steady_on_husk(sim: &HuntSim) -> HuntInput {
	HuntInput::still(sim.pose().direction(), 0.0)
}

/// Looks at the husk and taps the first open seal.
fn tap_open_seal(sim: &HuntSim) -> HuntInput {
	for index in 0..sim.vessel().seals().len() {
		if sim.seal_open(index) {
			return HuntInput::measured(
				sim.pose().direction(),
				0.0,
				&[sim.pose().seal_direction(index)],
				false,
			);
		}
	}

	steady_on_husk(sim)
}

/// The seals of `sim`'s husk that can be struck right now.
fn open_seals(sim: &HuntSim) -> Vec<usize> {
	(0..sim.vessel().seals().len())
		.filter(|&seal| sim.seal_open(seal))
		.collect()
}

/// The seals of `sim`'s husk on the side the reaper can see.
fn in_view(sim: &HuntSim) -> Vec<usize> {
	(0..sim.vessel().seals().len())
		.filter(|&seal| sim.pose().seal_faces_player(seal))
		.collect()
}

/// The one event `pick` finds in `events`.
fn single<T>(events: &[HuntEvent], pick: impl Fn(&HuntEvent) -> Option<T>) -> T {
	let mut found: Vec<T> = events.iter().filter_map(pick).collect();

	assert_eq!(found.len(), 1, "expected exactly one such event");

	found.remove(0)
}

fn ambush_of(event: &HuntEvent) -> Option<bool> {
	match event {
		HuntEvent::StalkEnded { ambush } => Some(*ambush),
		_ => None,
	}
}

fn forgiven_of(event: &HuntEvent) -> Option<bool> {
	match event {
		HuntEvent::Flinched { forgiven, .. } => Some(*forgiven),
		_ => None,
	}
}

/// A [`HuntEvent::StrikeLanded`], taken apart.
struct Strike {
	seal: usize,
	damage: f64,
	cadence: i64,
	empowered: bool,
	cleaved: Vec<usize>,
}

fn strikes(events: &[HuntEvent]) -> Vec<Strike> {
	events
		.iter()
		.filter_map(|event| {
			match event {
				HuntEvent::StrikeLanded {
					seal,
					damage,
					cadence,
					empowered,
					cleaved,
				} => {
					Some(Strike {
						seal: *seal,
						damage: *damage,
						cadence: *cadence,
						empowered: *empowered,
						cleaved: cleaved.clone(),
					})
				}
				_ => None,
			}
		})
		.collect()
}

fn single_strike(events: &[HuntEvent]) -> Strike {
	let mut found = strikes(events);

	assert_eq!(found.len(), 1, "expected exactly one strike");

	found.remove(0)
}

/// Plays `sim` to its end with a bot in `style`.
fn play_out(mut sim: HuntSim, style: HuntBotStyle) -> (HuntSim, HuntBot) {
	let mut bot = HuntBot::new(&sim, style);

	bot.play(&mut sim);

	(sim, bot)
}

mod stalk {
	use super::*;

	#[test]
	fn holding_a_steady_aim_on_the_husk_springs_an_ambush() {
		let mut sim = default_sim();
		let events = run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		assert!(single(&events, ambush_of));
		assert!(sim.ambushed());
		assert_eq!(sim.tick(), HuntRules::AMBUSH_TICKS);
		assert!(
			in_view(&sim).len() >= 2,
			"a husk never hides all but one of its seals"
		);
		assert_eq!(
			open_seals(&sim),
			in_view(&sim),
			"an ambush exposes every seal the reaper can see"
		);
	}

	#[test]
	fn turning_fast_alarms_the_husk_and_loses_the_ambush() {
		let mut sim = default_sim();
		let events = run_until(&mut sim, HuntPhase::Fight, |_| {
			HuntInput::new(Aim::AHEAD, 6000)
		});

		assert!(!single(&events, ambush_of));
		assert_eq!(sim.awareness(), 1.0);
	}

	#[test]
	fn turning_slowly_never_alarms_the_husk() {
		let mut sim = default_sim();

		for _ in 0..600 {
			sim.step(&HuntInput::new(Aim::AHEAD, 1200));
		}

		assert_eq!(sim.phase(), HuntPhase::Stalk);
		assert_eq!(sim.awareness(), 0.0);
	}

	#[test]
	fn a_husk_that_is_never_found_notices_on_its_own() {
		let mut sim = default_sim();

		run_until(&mut sim, HuntPhase::Fight, |_| idle());

		assert_eq!(sim.tick(), HuntRules::STALK_LIMIT_TICKS + 1);
		assert!(!sim.ambushed());
	}

	#[test]
	fn striking_the_husk_first_starts_the_fight_without_an_ambush() {
		let mut sim = default_sim();
		let events = step_with(&mut sim, |sim| {
			HuntInput::measured(Vec3::FORWARD, 0.0, &[sim.pose().direction()], false)
		});

		assert!(!single(&events, ambush_of));
		assert_eq!(count!(events, HuntEvent::StrikeGlanced { .. }), 1);
	}
}

mod strike {
	use super::*;

	/// A ferryman, so no strike in this group cleaves, brought to the fight.
	fn fighting() -> HuntSim {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Ferryman,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		sim
	}

	#[test]
	fn a_tap_on_an_open_seal_removes_ward_and_shuts_the_seal() {
		let mut sim = fighting();
		let events = step_with(&mut sim, tap_open_seal);
		let strike = single_strike(&events);

		assert!((sim.ward() - (Rarity::Common.ward() as f64 - strike.damage)).abs() <= 1e-9);
		assert!(!sim.seal_open(strike.seal));
		assert!(strike.empowered, "the ambush strike");
	}

	#[test]
	fn a_tap_away_from_the_husk_whiffs_and_breaks_the_cadence() {
		let mut sim = fighting();

		step_with(&mut sim, tap_open_seal);

		assert_eq!(sim.cadence(), 1);

		for _ in 0..sim.stats().swing_ticks {
			step_with(&mut sim, steady_on_husk);
		}

		let events = step_with(&mut sim, |sim| {
			HuntInput::measured(
				sim.pose().direction(),
				0.0,
				&[-sim.pose().direction()],
				false,
			)
		});

		assert_eq!(count!(events, HuntEvent::StrikeWhiffed), 1);
		assert_eq!(sim.cadence(), 0);
	}

	#[test]
	fn a_tap_during_a_swing_waits_for_it_instead_of_being_lost() {
		let mut sim = fighting();

		step_with(&mut sim, tap_open_seal);

		// Tap again a few ticks before the weapon is ready.
		for _ in 1..sim.stats().swing_ticks - 5 {
			step_with(&mut sim, steady_on_husk);
		}

		let early = step_with(&mut sim, tap_open_seal);

		assert_eq!(count!(early, HuntEvent::StrikeLanded { .. }), 0);

		let mut later = Vec::new();

		for _ in 0..6 {
			later.extend(step_with(&mut sim, steady_on_husk));
		}

		assert_eq!(count!(later, HuntEvent::StrikeLanded { .. }), 1);
	}

	#[test]
	fn a_tap_made_too_early_in_a_swing_is_dropped() {
		let mut sim = fighting();

		step_with(&mut sim, tap_open_seal);
		step_with(&mut sim, tap_open_seal);

		let mut later = Vec::new();

		for _ in 0..sim.stats().swing_ticks + 5 {
			later.extend(step_with(&mut sim, steady_on_husk));
		}

		assert_eq!(count!(later, HuntEvent::StrikeLanded { .. }), 0);
	}

	#[test]
	fn hits_in_cadence_grow_up_to_the_cap() {
		let mut sim = fighting();
		let mut damages = Vec::new();

		for tick in 0..2000 {
			if damages.len() >= 8 {
				break;
			}

			let events = if tick % sim.stats().swing_ticks == 0 {
				step_with(&mut sim, tap_open_seal)
			} else {
				step_with(&mut sim, steady_on_husk)
			};

			if sim.phase() != HuntPhase::Fight {
				break;
			}

			damages.extend(
				strikes(&events)
					.into_iter()
					.filter(|strike| !strike.empowered && strike.cleaved.is_empty())
					.map(|strike| strike.damage),
			);
		}

		assert!(damages.len() > 3);

		let cap = sim.stats().strike_damage
			* (1.0 + HuntRules::CADENCE_CAP as f64 * HuntRules::CADENCE_STEP);

		assert!(damages.iter().all(|&damage| damage <= cap + 1e-9));
		assert!(damages.last() > damages.first());
	}
}

mod shriek {
	use super::*;

	fn at_shriek(reaper_class: ReaperClass) -> HuntSim {
		let mut sim = sim_for(SimFor {
			reaper_class,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);
		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);

		sim
	}

	fn moving(sim: &HuntSim) -> HuntInput {
		HuntInput::still(sim.pose().direction(), 3.0)
	}

	#[test]
	fn the_warning_comes_first_and_lasts_its_full_length() {
		let mut sim = default_sim();

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		let start = sim.tick();
		let events = run_until(&mut sim, HuntPhase::Windup, steady_on_husk);

		assert_eq!(count!(events, HuntEvent::ShriekWindup), 1);
		assert!(sim.tick() - start >= HuntRules::SHRIEK_GAP_MIN_TICKS);

		let warned = sim.tick();

		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);

		assert_eq!(sim.tick() - warned, HuntRules::WINDUP_TICKS);
	}

	#[test]
	fn holding_still_through_it_exposes_every_seal() {
		let mut sim = at_shriek(ReaperClass::Harvester);
		let events = run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		assert_eq!(count!(events, HuntEvent::HeldFirm), 1);
		assert_eq!(count!(events, HuntEvent::Flinched { .. }), 0);
		assert_eq!(sim.resolve(), sim.stats().resolve);
		assert!(sim.pose().faces_player(), "it turned to shriek");
		assert_eq!(open_seals(&sim), in_view(&sim));
		assert!(open_seals(&sim).len() >= 2);
	}

	#[test]
	fn moving_during_it_costs_one_resolve_once() {
		let mut sim = at_shriek(ReaperClass::Harvester);
		let events = run_until(&mut sim, HuntPhase::Fight, moving);

		assert_eq!(count!(events, HuntEvent::Flinched { .. }), 1);
		assert_eq!(count!(events, HuntEvent::HeldFirm), 0);
		assert_eq!(sim.resolve(), sim.stats().resolve - 1);
	}

	#[test]
	fn tapping_during_it_is_a_flinch_too() {
		let mut sim = at_shriek(ReaperClass::Harvester);

		for _ in 0..HuntRules::SHRIEK_GRACE_TICKS {
			step_with(&mut sim, steady_on_husk);
		}

		let events = step_with(&mut sim, |sim| {
			HuntInput::measured(
				sim.pose().direction(),
				0.0,
				&[sim.pose().direction()],
				false,
			)
		});

		assert_eq!(count!(events, HuntEvent::Flinched { .. }), 1);
		assert_eq!(count!(events, HuntEvent::StrikeLanded { .. }), 0);
	}

	#[test]
	fn movement_in_the_first_moments_is_forgiven() {
		let mut sim = at_shriek(ReaperClass::Harvester);
		let mut events = Vec::new();

		for _ in 0..HuntRules::SHRIEK_GRACE_TICKS - 1 {
			events.extend(step_with(&mut sim, moving));
		}

		events.extend(run_until(&mut sim, HuntPhase::Fight, steady_on_husk));

		assert_eq!(count!(events, HuntEvent::Flinched { .. }), 0);
		assert_eq!(count!(events, HuntEvent::HeldFirm), 1);
	}

	#[test]
	fn running_out_of_resolve_lets_the_husk_escape() {
		let mut sim = default_sim();

		run_until_within(
			&mut sim,
			HuntPhase::Escaped,
			|sim| {
				HuntInput::still(
					sim.pose().direction(),
					if sim.phase() == HuntPhase::Shriek {
						3.0
					} else {
						0.0
					},
				)
			},
			HuntLog::MAX_TICKS,
		);

		assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Resolve));
		assert_eq!(sim.resolve(), 0);
	}

	#[test]
	fn a_ferryman_is_forgiven_the_first_flinch_of_a_hunt() {
		let mut sim = at_shriek(ReaperClass::Ferryman);
		let first = run_until(&mut sim, HuntPhase::Fight, moving);

		assert!(single(&first, forgiven_of));
		assert_eq!(sim.resolve(), sim.stats().resolve);

		run_until(&mut sim, HuntPhase::Shriek, wary_on_husk);

		let second = run_until(&mut sim, HuntPhase::Fight, moving);

		assert!(!single(&second, forgiven_of));
		assert_eq!(sim.resolve(), sim.stats().resolve - 1);
	}
}

mod break_and_reap {
	use super::*;

	#[test]
	fn a_careful_player_absorbs_a_common_husk_inside_the_time_limit() {
		let mut sim = default_sim();
		let mut bot = HuntBot::careful(&sim);
		let outcome = bot.play(&mut sim);

		let outcome = outcome.expect("the hunt should end");

		assert!(outcome.absorbed());
		assert_ne!(outcome.grade(), Some(ReapGrade::Ragged));
		assert_eq!(count!(bot.events, HuntEvent::Broken), 1);
	}

	#[test]
	fn a_steady_sweep_grades_better_than_a_hurried_one() {
		let (steady, _) = play_out(
			default_sim(),
			HuntBotStyle {
				reap_speed: 0.8,
				..HuntBotStyle::CAREFUL
			},
		);
		let (hurried, _) = play_out(
			default_sim(),
			HuntBotStyle {
				reap_speed: 6.0,
				..HuntBotStyle::CAREFUL
			},
		);

		assert_eq!(steady.outcome().unwrap().grade(), Some(ReapGrade::Perfect));
		assert_eq!(hurried.outcome().unwrap().grade(), Some(ReapGrade::Ragged));
	}

	#[test]
	fn a_window_left_unfinished_lets_the_husk_recover_then_escape() {
		let mut sim = default_sim();
		let mut bot = HuntBot::careful(&sim);

		while sim.phase() != HuntPhase::Broken {
			bot.step_once(&mut sim);
		}

		let recovered = run_until(&mut sim, HuntPhase::Fight, steady_on_husk);
		let attempts_left = single(&recovered, |event| {
			match event {
				HuntEvent::ReapRecovered { attempts_left } => Some(*attempts_left),
				_ => None,
			}
		});

		assert_eq!(attempts_left, 1);
		assert_eq!(
			sim.ward(),
			Rarity::Common.ward() as f64 * HuntRules::RECOVER_SHARE
		);

		while sim.phase() != HuntPhase::Broken {
			bot.step_once(&mut sim);
		}

		run_until(&mut sim, HuntPhase::Escaped, steady_on_husk);

		assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Reap));
	}

	#[test]
	fn the_cut_only_moves_forward() {
		let mut sim = default_sim();
		let mut bot = HuntBot::careful(&sim);

		while !sim.reap_begun() || sim.reap_progress() < 0.4 {
			bot.step_once(&mut sim);
		}

		let reached = sim.reap_progress();

		step_with(&mut sim, |sim| {
			HuntInput::still(sim.reap_arc_point(0.1), 0.5)
		});

		assert_eq!(sim.reap_progress(), reached);
	}

	#[test]
	fn a_fight_that_runs_too_long_ends_with_the_husk_escaping() {
		let mut sim = sim_for(SimFor {
			rarity: Rarity::Epic,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);
		run_until_within(
			&mut sim,
			HuntPhase::Escaped,
			wary_on_husk,
			HuntLog::MAX_TICKS,
		);

		assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Time));
	}
}

mod class_perks {
	use super::*;

	#[test]
	fn a_harvester_cleaves_through_neighbouring_seals() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		let mut cleaves = Vec::new();
		let mut bot = HuntBot::careful(&sim);

		while !sim.is_over() && sim.phase() != HuntPhase::Broken {
			cleaves.extend(strikes(&bot.step_once(&mut sim)));
		}

		assert!(cleaves.iter().any(|strike| !strike.cleaved.is_empty()));
	}

	#[test]
	fn a_weigher_striking_in_order_hits_twice_as_hard() {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Weigher,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		// The ambush has exposed every seal in view; the one that opened
		// first ranks 0 and the last ranks highest.
		let open = open_seals(&sim);
		let ranked = |rank: usize| {
			let found: Vec<usize> = open
				.iter()
				.copied()
				.filter(|&seal| sim.seal_rank(seal) == Some(rank))
				.collect();

			assert_eq!(found.len(), 1, "one seal ranks {rank}");

			found[0]
		};
		let oldest = ranked(0);
		let newest = ranked(open.len() - 1);

		assert!(open.len() >= 2);

		let tap = |sim: &HuntSim, seal: usize| {
			HuntInput::measured(
				sim.pose().direction(),
				0.0,
				&[sim.pose().seal_direction(seal)],
				false,
			)
		};

		let out_of_order = single_strike(&step_with(&mut sim, |sim| tap(sim, newest)));

		for _ in 0..sim.stats().swing_ticks {
			step_with(&mut sim, steady_on_husk);
		}

		let in_order = single_strike(&step_with(&mut sim, |sim| tap(sim, oldest)));

		// Take the ambush bonus off the first and the cadence off the second.
		let plain = out_of_order.damage / HuntRules::AMBUSH_STRIKE;
		let measured = in_order.damage / (1.0 + HuntRules::CADENCE_STEP);

		assert!((measured - plain * HuntRules::MEASURE_STRIKE).abs() <= 1e-9);
	}

	#[test]
	fn a_mourner_gains_reap_time_from_each_unflinching_hold() {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Mourner,
			..SimFor::default()
		});
		let before = sim.capture_ticks();

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);
		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);
		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		assert_eq!(sim.capture_ticks(), before + HuntRules::TOLL_TICKS);
	}

	#[test]
	fn a_chooser_strikes_three_times_as_hard_after_holding_firm() {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Chooser,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, |_| idle());
		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);
		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		let strike = single_strike(&step_with(&mut sim, tap_open_seal));

		assert!(
			(strike.damage - sim.stats().strike_damage * HuntRules::CHOSEN_BLOW_STRIKE).abs()
				<= 1e-9
		);
	}

	#[test]
	fn a_hollow_builds_cadence_past_the_usual_cap() {
		let (_, bot) = play_out(
			sim_for(SimFor {
				reaper_class: ReaperClass::Hollow,
				..SimFor::default()
			}),
			HuntBotStyle::CAREFUL,
		);
		let best = strikes(&bot.events)
			.iter()
			.map(|strike| strike.cadence)
			.max()
			.expect("some strike landed");

		assert!(best > HuntRules::CADENCE_CAP);
	}
}

mod every_vessel {
	use super::*;

	/// Whether a careful player usually takes a common `vessel`.
	fn can_be_absorbed_by_a_careful_player(vessel: Vessel) {
		assert!(vessel.is_solo());

		let absorbed = (1..=12)
			.filter(|&seed| {
				let (sim, _) = play_out(
					sim_for(SimFor {
						seed,
						vessel,
						..SimFor::default()
					}),
					HuntBotStyle::CAREFUL,
				);

				sim.outcome().is_some_and(|outcome| outcome.absorbed())
			})
			.count();

		assert!(
			absorbed >= 10,
			"{vessel}: a level-one reaper should usually take a common husk"
		);
	}

	solo_vessel_tests!(can_be_absorbed_by_a_careful_player);

	#[test]
	fn seals_sit_inside_the_body_they_belong_to() {
		for vessel in Vessel::ALL {
			for seal in vessel.seals() {
				assert!(seal.offset.x.abs() <= vessel.half_width() * 1.1);
				assert!(seal.offset.y.abs() <= vessel.half_height() * 1.1);
			}
		}
	}
}

mod pacing {
	use super::*;

	/// Seconds of fighting, from the end of the stalk to the reap.
	fn fight_seconds(sim: &HuntSim, bot: &HuntBot) -> i64 {
		assert!(
			bot.events
				.iter()
				.any(|event| matches!(event, HuntEvent::StalkEnded { .. }))
		);

		(sim.outcome().unwrap().ticks - HuntRules::AMBUSH_TICKS) / HuntRules::TICK_RATE
	}

	fn human(options: SimFor) -> (HuntSim, HuntBot) {
		play_out(sim_for(options), HuntBotStyle::HUMAN)
	}

	fn human_hunts(rarity: Rarity, level: i64, temper: Temper) -> Vec<HuntOutcome> {
		let mut outcomes = Vec::new();

		for reaper_class in ReaperClass::ALL {
			for seed in 1..=4 {
				let (sim, _) = human(SimFor {
					seed,
					rarity,
					reaper_class,
					level,
					temper,
					..SimFor::default()
				});

				outcomes.push(sim.outcome().unwrap());
			}
		}

		outcomes
	}

	fn absorbed(outcomes: &[HuntOutcome]) -> usize {
		outcomes.iter().filter(|outcome| outcome.absorbed()).count()
	}

	#[test]
	fn a_common_husk_takes_a_level_one_reaper_about_half_a_minute() {
		for reaper_class in ReaperClass::ALL {
			let (sim, bot) = human(SimFor {
				reaper_class,
				..SimFor::default()
			});

			assert!(sim.outcome().unwrap().absorbed(), "{reaper_class:?}");
			assert!(
				(15..=60).contains(&fight_seconds(&sim, &bot)),
				"{reaper_class:?}"
			);
		}
	}

	#[test]
	fn every_class_can_take_an_uncommon_husk_at_level_one() {
		let outcomes = human_hunts(Rarity::Uncommon, 1, Temper::Shrouded);

		assert!(absorbed(&outcomes) >= outcomes.len() * 3 / 4);
	}

	#[test]
	fn an_epic_husk_is_too_much_for_any_level_one_reaper() {
		assert_eq!(absorbed(&human_hunts(Rarity::Epic, 1, Temper::Shrouded)), 0);
	}

	#[test]
	fn the_same_epic_husk_falls_to_a_levelled_better_armed_reaper() {
		let outcomes = human_hunts(Rarity::Epic, 10, Temper::Scorched);

		assert_eq!(absorbed(&outcomes), outcomes.len());
	}

	#[test]
	fn no_class_is_far_stronger_than_another() {
		let seconds: Vec<(ReaperClass, i64)> = ReaperClass::ALL
			.into_iter()
			.map(|reaper_class| {
				let total = (1..=6)
					.map(|seed| {
						let (sim, bot) = human(SimFor {
							seed,
							reaper_class,
							level: 3,
							..SimFor::default()
						});

						fight_seconds(&sim, &bot)
					})
					.sum();

				(reaper_class, total)
			})
			.collect();
		let fastest = seconds.iter().map(|&(_, total)| total).min().unwrap();
		let slowest = seconds.iter().map(|&(_, total)| total).max().unwrap();

		assert!(slowest as f64 / (fastest as f64) < 2.2, "{seconds:?}");
	}
}

mod determinism_and_replay {
	use super::*;

	fn careful(options: SimFor) -> (HuntSim, HuntBot) {
		play_out(sim_for(options), HuntBotStyle::CAREFUL)
	}

	#[test]
	fn the_same_seed_and_inputs_give_the_same_events() {
		let (first_sim, first) = careful(SimFor {
			seed: 31,
			..SimFor::default()
		});
		let mut second = HuntSim::open(
			31,
			first_sim.vessel(),
			Rarity::Common.ward(),
			first_sim.stats(),
		);
		let mut replayed = Vec::new();

		for input in &first.recorded {
			replayed.extend(second.step(input));
		}

		assert_eq!(
			replayed.iter().map(HuntEvent::kind).collect::<Vec<_>>(),
			first.events.iter().map(HuntEvent::kind).collect::<Vec<_>>()
		);
		assert_eq!(
			second.outcome().unwrap().grade(),
			first_sim.outcome().unwrap().grade()
		);
		assert_eq!(
			second.outcome().unwrap().ticks,
			first_sim.outcome().unwrap().ticks
		);
	}

	#[test]
	fn a_log_survives_being_packed_into_bytes() {
		let (_, bot) = careful(SimFor {
			seed: 12,
			..SimFor::default()
		});
		let decoded = HuntLog::decode(&HuntLog::new(bot.recorded.clone()).encode()).unwrap();

		assert_eq!(decoded.inputs.len(), bot.recorded.len());

		for (decoded, recorded) in decoded.inputs.iter().zip(&bot.recorded) {
			assert_eq!(decoded.aim, recorded.aim);
			assert_eq!(decoded.turn_speed, recorded.turn_speed);
			assert_eq!(decoded.taps, recorded.taps);
		}
	}

	#[test]
	fn a_packed_log_replays_to_the_outcome_the_phone_saw() {
		for seed in 1..=20 {
			let (sim, bot) = careful(SimFor {
				seed,
				..SimFor::default()
			});
			let outcome = sim.outcome().unwrap();
			let replayed = HuntSim::replay(
				seed,
				Vessel::Penguin,
				Rarity::Common.ward(),
				&SimFor::default().stats(),
				&HuntLog::decode(&HuntLog::new(bot.recorded.clone()).encode()).unwrap(),
			)
			.unwrap();

			assert_eq!(replayed.absorbed(), outcome.absorbed(), "{seed}");
			assert_eq!(replayed.grade(), outcome.grade(), "{seed}");
			assert_eq!(replayed.ticks, outcome.ticks, "{seed}");
		}
	}

	#[test]
	fn a_log_that_stops_early_proves_nothing() {
		let (_, bot) = careful(SimFor::default());
		let cut = HuntLog::new(bot.recorded[..bot.recorded.len() / 2].to_vec());

		assert_eq!(
			HuntSim::replay(
				7,
				Vessel::Penguin,
				Rarity::Common.ward(),
				&SimFor::default().stats(),
				&cut,
			),
			None
		);
	}

	#[test]
	fn a_malformed_log_is_rejected() {
		let mut sim = default_sim();

		sim.step(&idle());

		let bytes = HuntLog::new(HuntBot::careful(&sim).recorded).encode();

		assert!(HuntLog::decode(&bytes[..0]).is_err());
		assert!(
			HuntLog::decode(&bytes[..3]).is_err(),
			"the plan is cut short"
		);
		assert!(
			HuntLog::decode(&HuntLog::new(vec![idle()]).encode()[..12]).is_err(),
			"a tick is cut short"
		);
		assert!(
			HuntLog::decode(&[1, 0, 0, 0, 0]).is_err(),
			"a log from before plans existed"
		);
	}
}
