//! The hunt, a tick at a time.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]
#![allow(
	clippy::assert_is_empty,
	reason = "asserting an empty seal list reads clearer than an equality"
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
	HuntInput::measured(
		sim.pose().direction(),
		0.0,
		false,
		sim.selected_slot(),
		false,
	)
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
		.filter(|&seal| sim.pose().seal_faces(reapers_rules::Vec3::ZERO, seal))
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

/// A [`HuntEvent::HitLanded`], taken apart.
struct Hit {
	damage: f64,
	cadence: i64,
	critical: bool,
	empowered: bool,
	cleaved: Vec<usize>,
}

fn hits(events: &[HuntEvent]) -> Vec<Hit> {
	events
		.iter()
		.filter_map(|event| {
			match event {
				HuntEvent::HitLanded {
					weapon: _,
					seal: _,
					damage,
					cadence,
					critical,
					empowered,
					cleaved,
				} => {
					Some(Hit {
						damage: *damage,
						cadence: *cadence,
						critical: *critical,
						empowered: *empowered,
						cleaved: cleaved.clone(),
					})
				}
				_ => None,
			}
		})
		.collect()
}

fn single_hit(events: &[HuntEvent]) -> Hit {
	let mut found = hits(events);

	assert_eq!(found.len(), 1, "expected exactly one hit");

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
	fn firing_at_the_husk_first_starts_the_fight_without_an_ambush() {
		let mut sim = default_sim();
		let events = step_with(&mut sim, |sim| fire_at(sim, sim.pose().direction(), None));

		assert!(!single(&events, ambush_of));
		assert!(!single_hit(&events).critical);
	}
}

mod cadence {
	use super::*;

	/// A ferryman, so no hit in this group cleaves, brought to the fight.
	fn fighting() -> HuntSim {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Ferryman,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		sim
	}

	fn fire_open_seal(sim: &HuntSim) -> HuntInput {
		let open = open_seals(sim);

		fire_at(
			sim,
			if open.is_empty() {
				sim.pose().direction()
			} else {
				sim.pose()
					.seal_direction(reapers_rules::Vec3::ZERO, open[0])
			},
			None,
		)
	}

	#[test]
	fn a_miss_breaks_the_cadence() {
		let mut sim = fighting();

		step_with(&mut sim, fire_open_seal);

		assert_eq!(sim.cadence(), 1);
		assert_eq!(
			sim.cadence_ticks_left(),
			HuntRules::CADENCE_WINDOW_TICKS - 1
		);

		for _ in 0..GunRules::FIRE_INTERVAL_TICKS {
			step_with(&mut sim, steady_on_husk);
		}

		let events = step_with(&mut sim, |sim| fire_at(sim, -sim.pose().direction(), None));

		assert_eq!(count!(events, HuntEvent::Missed(_)), 1);
		assert_eq!(sim.cadence(), 0);
		assert_eq!(sim.cadence_ticks_left(), 0);
	}

	#[test]
	fn the_cadence_lapses_when_no_hit_lands_in_its_window() {
		let mut sim = fighting();

		step_with(&mut sim, fire_open_seal);

		for _ in 0..=HuntRules::CADENCE_WINDOW_TICKS {
			step_with(&mut sim, steady_on_husk);
		}

		assert_eq!(sim.cadence(), 0);
	}

	#[test]
	fn hits_in_cadence_grow_up_to_the_cap() {
		let mut sim = fighting();
		let mut damages = Vec::new();

		for _ in 0..2000 {
			if damages.len() >= 8 {
				break;
			}

			let events = if sim.reloading() {
				step_with(&mut sim, steady_on_husk)
			} else {
				step_with(&mut sim, |sim| fire_at(sim, sim.pose().direction(), None))
			};

			if sim.phase() != HuntPhase::Fight {
				break;
			}

			damages.extend(
				hits(&events)
					.into_iter()
					.filter(|hit| !hit.empowered)
					.map(|hit| {
						hit.damage
							/ GunRules::SHOT_STRIKE
							/ if hit.critical {
								HuntRules::CRITICAL_STRIKE
							} else {
								1.0
							}
					}),
			);
		}

		assert!(damages.len() > 3);

		let cap = sim.stats().hit * (1.0 + HuntRules::CADENCE_CAP as f64 * HuntRules::CADENCE_STEP);

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
		assert!(
			sim.pose().faces(reapers_rules::Vec3::ZERO),
			"it turned to shriek"
		);
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

		let events = step_with(&mut sim, |sim| fire_at(sim, sim.pose().direction(), None));

		assert_eq!(count!(events, HuntEvent::Flinched { .. }), 1);
		assert_eq!(count!(events, HuntEvent::ShotFired { .. }), 0);
		assert_eq!(count!(events, HuntEvent::HitLanded { .. }), 0);
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

mod break_and_capture {
	use super::*;

	#[test]
	fn a_careful_player_absorbs_a_common_husk_inside_the_time_limit() {
		let mut sim = default_sim();
		let mut bot = HuntBot::careful(&sim);
		let outcome = bot.play(&mut sim);

		let outcome = outcome.expect("the hunt should end");

		assert!(outcome.absorbed());
		assert_ne!(outcome.grade(), Some(CaptureGrade::Plain));
		assert_eq!(count!(bot.events, HuntEvent::Broken { .. }), 1);
		assert!(count!(bot.events, HuntEvent::OrbThrown { .. }) > 0);
	}

	#[test]
	fn a_well_timed_throw_grades_better_than_a_hurried_one() {
		let (timed, _) = play_out(
			default_sim(),
			HuntBotStyle {
				orb_hold_ticks: 58,
				..HuntBotStyle::CAREFUL
			},
		);
		let (hurried, _) = play_out(
			default_sim(),
			HuntBotStyle {
				orb_hold_ticks: 8,
				..HuntBotStyle::CAREFUL
			},
		);

		assert_eq!(
			timed.outcome().unwrap().grade(),
			Some(CaptureGrade::Excellent)
		);
		assert_eq!(
			hurried.outcome().unwrap().grade(),
			Some(CaptureGrade::Plain),
			"the outer third grades no better than no hold"
		);
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
	fn a_harvester_cleaves_through_other_open_seals() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		let open = open_seals(&sim);
		let hit = single_hit(&step_with(&mut sim, |sim| {
			fire_at(
				sim,
				sim.pose()
					.seal_direction(reapers_rules::Vec3::ZERO, open[0]),
				None,
			)
		}));

		assert!(open.len() >= 2);
		assert_eq!(hit.cleaved, open[1..].to_vec());
		assert!(open_seals(&sim).is_empty());

		let alone = sim.stats().hit
			* GunRules::SHOT_STRIKE
			* HuntRules::CRITICAL_STRIKE
			* HuntRules::AMBUSH_STRIKE;

		assert!(
			(hit.damage - alone * (1.0 + HuntRules::CLEAVE_SHARE * hit.cleaved.len() as f64)).abs()
				<= 1e-9
		);
	}

	#[test]
	fn a_weigher_striking_in_order_hits_harder() {
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

		let out_of_order = single_hit(&step_with(&mut sim, |sim| {
			fire_at(
				sim,
				sim.pose().seal_direction(reapers_rules::Vec3::ZERO, newest),
				None,
			)
		}));

		for _ in 0..GunRules::FIRE_INTERVAL_TICKS {
			step_with(&mut sim, steady_on_husk);
		}

		let in_order = single_hit(&step_with(&mut sim, |sim| {
			fire_at(
				sim,
				sim.pose().seal_direction(reapers_rules::Vec3::ZERO, oldest),
				None,
			)
		}));

		// Take the ambush bonus off the first and the cadence off the
		// second.
		let plain = out_of_order.damage / HuntRules::AMBUSH_STRIKE;
		let measured = in_order.damage / (1.0 + HuntRules::CADENCE_STEP);

		assert!((measured - plain * HuntRules::MEASURE_STRIKE).abs() <= 1e-9);
	}

	#[test]
	fn a_mourner_keeps_a_broken_husk_down_longer_for_each_hold() {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Mourner,
			..SimFor::default()
		});
		let before = sim.broken_ticks();

		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);
		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);
		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		assert_eq!(sim.broken_ticks(), before + HuntRules::TOLL_TICKS);
	}

	#[test]
	fn a_chooser_hits_three_times_as_hard_after_holding_firm() {
		let mut sim = sim_for(SimFor {
			reaper_class: ReaperClass::Chooser,
			..SimFor::default()
		});

		run_until(&mut sim, HuntPhase::Fight, |_| idle());
		run_until(&mut sim, HuntPhase::Shriek, steady_on_husk);
		run_until(&mut sim, HuntPhase::Fight, steady_on_husk);

		let hit = single_hit(&step_with(&mut sim, |sim| {
			let open = open_seals(sim);

			fire_at(
				sim,
				if open.is_empty() {
					sim.pose().direction()
				} else {
					sim.pose()
						.seal_direction(reapers_rules::Vec3::ZERO, open[0])
				},
				None,
			)
		}));

		assert!(hit.critical);
		assert!(
			(hit.damage
				- sim.stats().hit
					* GunRules::SHOT_STRIKE
					* HuntRules::CRITICAL_STRIKE
					* HuntRules::CHOSEN_BLOW_STRIKE)
				.abs()
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
		let best = hits(&bot.events)
			.iter()
			.map(|hit| hit.cadence)
			.max()
			.expect("some hit landed");

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
			Rarity::Common,
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
			assert_eq!(decoded.fire, recorded.fire);
			assert_eq!(decoded.slot, recorded.slot);
			assert_eq!(decoded.orb_held, recorded.orb_held);
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
				Rarity::Common,
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
				Rarity::Common,
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

	#[test]
	fn a_tick_with_an_unknown_button_or_slot_is_refused() {
		let mut tick = HuntLog::new(vec![idle()]).encode();

		*tick.last_mut().unwrap() = Arsenal::SLOTS as u8;

		assert_eq!(
			HuntLog::decode(&tick),
			Err(HuntLogError::UnknownInput),
			"no such slot"
		);

		let mut tick = HuntLog::new(vec![idle()]).encode();
		let button = tick.len() - 2;

		tick[button] = 4;

		assert_eq!(
			HuntLog::decode(&tick),
			Err(HuntLogError::UnknownInput),
			"no such button"
		);
	}

	#[test]
	fn a_log_from_before_the_crosshair_is_refused() {
		let mut bytes = HuntLog::new(vec![idle()]).encode();

		bytes[0] = 6;

		assert_eq!(
			HuntLog::decode(&bytes),
			Err(HuntLogError::UnsupportedVersion)
		);
	}

	#[test]
	fn a_whole_hunt_survives_the_trip_to_the_server() {
		for reaper_class in ReaperClass::ALL {
			let (sim, bot) = play_out(
				sim_for(SimFor {
					seed: 21,
					reaper_class,
					..SimFor::default()
				}),
				HuntBotStyle::HUMAN,
			);
			let replayed = HuntSim::replay(
				21,
				sim.vessel(),
				Rarity::Common,
				Rarity::Common.ward(),
				&sim.stats().clone(),
				&HuntLog::decode(&HuntLog::new(bot.recorded.clone()).encode()).unwrap(),
			)
			.unwrap();

			assert_eq!(
				serde_json::to_value(replayed).unwrap(),
				serde_json::to_value(sim.outcome().unwrap()).unwrap(),
				"{reaper_class:?}",
			);
			assert!(
				count!(bot.events, HuntEvent::OrbThrown { .. }) > 0,
				"{reaper_class:?}",
			);
		}
	}
}
