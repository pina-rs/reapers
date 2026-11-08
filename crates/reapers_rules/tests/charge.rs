//! The Rust side of `reapers_shared/test/charge_test.dart`.

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
/// reaper hunts, as Dart's loop over `Vessel.values.where((vessel) =>
/// vessel.isSolo)` makes. Each test hands its vessel to the function
/// `check`.
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

fn range(pose: &HuskPose) -> f64 {
	(pose.position.x * pose.position.x + pose.position.z * pose.position.z).sqrt()
}

/// Steps `sim` with `input` until `stop` holds, and returns what happened.
fn until(
	sim: &mut HuntSim,
	stop: impl Fn(&HuntSim) -> bool,
	mut input: impl FnMut(&HuntSim) -> HuntInput,
) -> Vec<HuntEvent> {
	let mut events = Vec::new();

	while !stop(sim) && !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
		let next = input(sim);

		events.extend(sim.step(&next));
	}

	events
}

fn still(sim: &HuntSim) -> HuntInput {
	HuntInput::still(sim.pose().direction(), 0.0)
}

/// A hunt for an epic husk, which nothing here will break, brought to the
/// moment the husk starts its first charge.
fn charging(vessel: Vessel, seed: i64) -> HuntSim {
	let mut sim = sim_for(SimFor {
		seed,
		vessel,
		rarity: Rarity::Epic,
		..SimFor::default()
	});

	until(
		&mut sim,
		|sim| sim.phase() == HuntPhase::Charge,
		wary_on_husk,
	);

	assert_eq!(
		sim.phase(),
		HuntPhase::Charge,
		"it should charge in the end"
	);

	sim
}

/// [`charging`] with Dart's defaults: an ape, seed 4.
fn charging_ape() -> HuntSim {
	charging(Vessel::Ape, 4)
}

fn borrowed_stats() -> HuntStats {
	HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap()
}

mod a_charge {
	use super::*;

	#[test]
	fn comes_in_the_fight_never_in_the_stalk() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			rarity: Rarity::Epic,
			..SimFor::default()
		});
		let stalk = until(
			&mut sim,
			|sim| sim.phase() != HuntPhase::Stalk,
			wary_on_husk,
		);

		assert_eq!(count!(stalk, HuntEvent::ChargeBegan), 0);

		let fight = until(
			&mut sim,
			|sim| sim.phase() == HuntPhase::Charge,
			wary_on_husk,
		);

		assert_eq!(count!(fight, HuntEvent::ChargeBegan), 1);
	}

	/// Brings the husk to within arm's reach, facing you.
	fn brings_it_to_within_arms_reach_facing_you(vessel: Vessel) {
		let mut sim = charging(vessel, 4);
		let before = range(&sim.pose());

		until(&mut sim, |sim| sim.charge_progress() > 0.97, still);

		assert!(before > HuntRules::CHARGE_METRES);
		assert!((range(&sim.pose()) - HuntRules::CHARGE_METRES).abs() <= 0.15);
		assert!(sim.pose().faces_player());

		if vessel.gait() == Gait::Air {
			assert!(sim.pose().position.y.abs() < 0.15, "eye level");
		}
	}

	solo_vessel_tests!(brings_it_to_within_arms_reach_facing_you);

	#[test]
	fn that_is_not_met_costs_resolve_and_the_husk_runs_off() {
		let mut sim = charging_ape();
		let resolve = sim.resolve();
		let events = until(&mut sim, |sim| sim.phase() != HuntPhase::Charge, still);
		let struck: Vec<i64> = events
			.iter()
			.filter_map(|event| {
				match event {
					HuntEvent::Struck { resolve_left } => Some(*resolve_left),
					_ => None,
				}
			})
			.collect();

		assert_eq!(struck, vec![resolve - 1]);
		assert_eq!(sim.resolve(), resolve - 1);
		assert_eq!(count!(events, HuntEvent::Relocating), 1);
		assert!(!sim.staggered());
	}

	#[test]
	fn is_turned_aside_by_a_swing_of_the_phone() {
		let mut sim = charging_ape();
		let resolve = sim.resolve();
		let events = until(
			&mut sim,
			|sim| sim.phase() != HuntPhase::Charge,
			wary_on_husk,
		);

		assert_eq!(count!(events, HuntEvent::Parried), 1);
		assert_eq!(count!(events, HuntEvent::Struck { .. }), 0);
		assert_eq!(sim.resolve(), resolve);
	}

	#[test]
	fn turned_aside_leaves_the_husk_stunned_with_its_seals_open() {
		let mut sim = charging_ape();

		until(
			&mut sim,
			|sim| sim.phase() != HuntPhase::Charge,
			wary_on_husk,
		);

		let place = sim.pose().position;
		let open = (0..sim.vessel().seals().len())
			.filter(|&seal| sim.seal_open(seal))
			.count();

		assert!(sim.staggered());
		assert!(open >= 2);

		for _ in 0..HuntRules::STAGGER_TICKS - 2 {
			let input = still(&sim);

			sim.step(&input);
		}

		assert!((sim.pose().position - place).length() < 1e-9);

		// Then it is off.
		let after = until(
			&mut sim,
			|sim| !sim.staggered() && sim.speed() > 0.0,
			wary_on_husk,
		);

		assert_eq!(count!(after, HuntEvent::Relocating), 1);
	}

	#[test]
	fn a_swing_in_its_first_moments_does_not_count() {
		let mut sim = charging_ape();

		for _ in 0..HuntRules::CHARGE_GRACE_TICKS - 1 {
			let input = HuntInput::still(sim.pose().direction(), 5.0);
			let events = sim.step(&input);

			assert_eq!(count!(events, HuntEvent::Parried), 0);
		}

		assert_eq!(sim.phase(), HuntPhase::Charge);
	}

	#[test]
	fn a_slow_turn_is_not_a_swing() {
		let mut sim = charging_ape();
		let events = until(
			&mut sim,
			|sim| sim.phase() != HuntPhase::Charge,
			|sim| HuntInput::still(sim.pose().direction(), sim.stats().parry_speed - 0.1),
		);

		assert_eq!(count!(events, HuntEvent::Parried), 0);
		assert_eq!(count!(events, HuntEvent::Struck { .. }), 1);
	}

	#[test]
	fn struck_with_no_resolve_left_the_reaper_loses_the_husk() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			rarity: Rarity::Epic,
			..SimFor::default()
		});

		until(&mut sim, HuntSim::is_over, still);

		// Standing still is right for a shriek and wrong for a charge, so
		// the charges are what wear this reaper down.
		assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Resolve));
		assert_eq!(sim.resolve(), 0);
	}

	#[test]
	fn never_comes_with_a_shriek_they_ask_opposite_things() {
		for seed in 1..=8 {
			let mut sim = sim_for(SimFor {
				seed,
				vessel: Vessel::Llama,
				rarity: Rarity::Epic,
				..SimFor::default()
			});
			let mut last_shriek_end = -1000;
			let mut last_charge_end = -1000;
			let mut previous = sim.phase();

			while !sim.is_over() {
				let input = wary_on_husk(&sim);
				let events = sim.step(&input);

				if events
					.iter()
					.any(|event| matches!(event, HuntEvent::ChargeBegan))
				{
					assert!(
						sim.tick() - last_shriek_end >= HuntRules::CHARGE_CLEAR_TICKS,
						"seed {seed}: a charge straight after a shriek"
					);
				}

				if events
					.iter()
					.any(|event| matches!(event, HuntEvent::ShriekWindup))
				{
					assert!(
						sim.tick() - last_charge_end >= HuntRules::CHARGE_CLEAR_TICKS,
						"seed {seed}: a shriek straight after a charge"
					);
				}

				if previous == HuntPhase::Shriek && sim.phase() != HuntPhase::Shriek {
					last_shriek_end = sim.tick();
				}

				if previous == HuntPhase::Charge && sim.phase() != HuntPhase::Charge {
					last_charge_end = sim.tick();
				}

				previous = sim.phase();
			}
		}
	}

	#[test]
	fn a_rifts_husk_does_not_charge() {
		let mut sim = HuntSim::rift(
			3,
			Vessel::ReliquaryStack,
			100_000.0,
			&borrowed_stats(),
			None,
		);
		let events = until(&mut sim, HuntSim::is_over, still);

		assert_eq!(count!(events, HuntEvent::ChargeBegan), 0);
	}

	#[test]
	fn gives_more_warning_on_easy_and_asks_for_less_of_a_swing() {
		let ticks_of_warning = |difficulty: Difficulty| {
			let mut sim = HuntSim::new(
				4,
				Vessel::Ape,
				Rarity::Epic.ward(),
				&borrowed_stats(),
				HuntPlan::OPEN,
				difficulty,
			)
			.unwrap();

			until(
				&mut sim,
				|sim| sim.phase() == HuntPhase::Charge,
				wary_on_husk,
			);

			let start = sim.tick();

			until(&mut sim, |sim| sim.phase() != HuntPhase::Charge, still);

			sim.tick() - start
		};

		assert!(ticks_of_warning(Difficulty::Easy) > ticks_of_warning(Difficulty::Normal) + 30);

		let stats = borrowed_stats();

		assert!(stats.at(Difficulty::Easy).parry_speed < stats.parry_speed);
		assert!(stats.at(Difficulty::Hard).parry_speed > stats.parry_speed);
	}

	#[test]
	fn the_bot_meets_a_charge_the_way_a_careful_person_would() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			rarity: Rarity::Rare,
			level: 6,
			..SimFor::default()
		});
		let mut bot = HuntBot::human(&sim);

		bot.play(&mut sim);

		assert!(count!(bot.events, HuntEvent::ChargeBegan) > 0);
		assert_eq!(count!(bot.events, HuntEvent::Struck { .. }), 0);
		assert!(count!(bot.events, HuntEvent::Parried) > 0);
	}
}
