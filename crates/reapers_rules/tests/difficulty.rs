//! How difficulty changes a hunt.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use reapers_rules::*;
use support::*;

fn stats() -> HuntStats {
	HuntStats::derive(
		&HuntLoadout::new(
			ReaperClass::Harvester,
			1,
			Weapon::new(WeaponKind::Scythe, Temper::Shrouded),
		),
		1.0,
	)
	.unwrap()
}

fn hunt_with(difficulty: Difficulty, seed: i64, vessel: Vessel) -> HuntSim {
	HuntSim::new(
		seed,
		vessel,
		Rarity::Uncommon.ward(),
		&stats(),
		HuntPlan::OPEN,
		difficulty,
	)
	.unwrap()
}

/// [`hunt_with`] with the usual seed, 7, and an ape.
fn hunt(difficulty: Difficulty) -> HuntSim {
	hunt_with(difficulty, 7, Vessel::Ape)
}

/// Someone who has never played: slow to see a seal open, and wide of the
/// mark with every other tap.
fn beginner(sim: &HuntSim) -> HuntBot {
	HuntBot::new(
		sim,
		HuntBotStyle {
			turn_speed: 0.9,
			reaction_ticks: 40,
			miss_every: 2,
			..HuntBotStyle::CAREFUL
		},
	)
}

/// Whether a beginner brings down the husk of `sim`.
fn beginner_absorbs(mut sim: HuntSim) -> bool {
	beginner(&sim)
		.play(&mut sim)
		.is_some_and(|outcome| outcome.absorbed())
}

fn solo_vessels() -> impl Iterator<Item = Vessel> {
	Vessel::ALL.into_iter().filter(|vessel| vessel.is_solo())
}

mod what_a_difficulty_changes {
	use super::*;

	#[test]
	fn normal_is_the_game_as_written() {
		let sim = hunt(Difficulty::Normal);
		let stats = stats();

		assert_eq!(sim.ward_max(), Rarity::Uncommon.ward() as f64);
		assert_eq!(sim.stats().resolve, stats.resolve);
		assert_eq!(sim.stats().reach_radians, stats.reach_radians);
		assert_eq!(sim.stats().flinch_speed, stats.flinch_speed);
		assert_eq!(sim.stats().capture_ticks, stats.capture_ticks);
		assert_eq!(sim.reap_attempts_left(), HuntRules::REAP_ATTEMPTS);
	}

	#[test]
	fn easy_thins_the_ward_and_widens_every_window() {
		let sim = hunt(Difficulty::Easy);
		let stats = stats();

		assert!((sim.ward_max() - Rarity::Uncommon.ward() as f64 * 0.35).abs() <= 1e-9);
		assert_eq!(sim.stats().resolve, stats.resolve + 3);
		assert!(sim.stats().reach_radians > stats.reach_radians * 1.5);
		assert!(sim.stats().flinch_speed > stats.flinch_speed * 2.0);
		assert!(sim.stats().capture_ticks > stats.capture_ticks * 2);
		assert!(sim.stats().reap_tolerance_radians > stats.reap_tolerance_radians * 1.5);
		assert_eq!(sim.reap_attempts_left(), HuntRules::REAP_ATTEMPTS + 2);
		// What a strike does is the reaper's own, whatever the difficulty.
		assert_eq!(sim.stats().strike_damage, stats.strike_damage);
	}

	#[test]
	fn hard_stiffens_the_ward_and_narrows_the_windows() {
		let sim = hunt(Difficulty::Hard);
		let stats = stats();

		assert!(sim.ward_max() > Rarity::Uncommon.ward() as f64);
		assert!(sim.stats().reach_radians < stats.reach_radians);
		assert!(sim.stats().flinch_speed < stats.flinch_speed);
		assert_eq!(sim.stats().resolve, stats.resolve);
	}

	#[test]
	fn a_husk_gets_about_slower_on_easy_and_faster_on_hard() {
		let top_speed = |difficulty: Difficulty| {
			let mut sim = hunt(difficulty);
			let mut bot = HuntBot::human(&sim);
			let mut top: f64 = 0.0;

			while !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
				bot.step_once(&mut sim);
				top = top.max(sim.speed());
			}

			top
		};

		assert!(top_speed(Difficulty::Easy) < top_speed(Difficulty::Normal));
		assert!(top_speed(Difficulty::Hard) > top_speed(Difficulty::Normal));
	}
}

mod who_can_bring_a_husk_down {
	use super::*;

	#[test]
	fn a_beginner_takes_it_on_easy_on_every_seed_tried() {
		for seed in 1..=12 {
			for vessel in solo_vessels() {
				assert!(
					beginner_absorbs(hunt_with(Difficulty::Easy, seed, vessel)),
					"{vessel}, seed {seed}"
				);
			}
		}
	}

	#[test]
	fn the_same_beginner_mostly_loses_it_on_hard() {
		let mut absorbed = 0;
		let mut hunts = 0;

		for seed in 1..=12 {
			for vessel in solo_vessels() {
				hunts += 1;

				if beginner_absorbs(hunt_with(Difficulty::Hard, seed, vessel)) {
					absorbed += 1;
				}
			}
		}

		assert!(absorbed < hunts / 2);
	}

	#[test]
	fn a_hunt_runs_longer_on_easy_before_the_husk_gives_up_waiting() {
		// Nobody strikes, so the fight runs to its limit.
		let ticks_to_escape = |difficulty: Difficulty| {
			let mut sim = hunt(difficulty);

			while !sim.is_over() {
				let input = wary_on_husk(&sim);

				sim.step(&input);
			}

			assert_eq!(sim.outcome().unwrap().escape(), Some(EscapeCause::Time));

			sim.tick()
		};

		assert!(
			ticks_to_escape(Difficulty::Easy)
				> ticks_to_escape(Difficulty::Normal) + 60 * HuntRules::TICK_RATE
		);
	}
}

mod the_record_of_a_hunt {
	use super::*;

	#[test]
	fn carries_the_difficulty_it_was_played_at() {
		for difficulty in Difficulty::ALL {
			let log = HuntLog::decode(
				&HuntLog {
					difficulty,
					..HuntLog::new(vec![idle()])
				}
				.encode(),
			)
			.unwrap();

			assert_eq!(log.difficulty, difficulty);
			assert_eq!(log.inputs.len(), 1);
		}
	}

	#[test]
	fn with_a_difficulty_nobody_has_heard_of_is_refused() {
		let mut bytes = HuntLog::new(vec![idle()]).encode();

		bytes[7] = Difficulty::ALL.len() as u8;

		assert_eq!(HuntLog::decode(&bytes), Err(HuntLogError::NoSuchDifficulty));
	}

	#[test]
	fn replays_to_the_same_end_at_the_difficulty_it_was_played_at() {
		let mut sim = hunt_with(Difficulty::Easy, 3, Vessel::Ape);
		let mut bot = beginner(&sim);

		bot.play(&mut sim);

		let log = HuntLog {
			difficulty: Difficulty::Easy,
			..HuntLog::new(bot.recorded.clone())
		};
		let replayed =
			|log: &HuntLog| HuntSim::replay(3, Vessel::Ape, Rarity::Uncommon.ward(), &stats(), log);

		assert!(sim.outcome().unwrap().absorbed());
		assert_eq!(replayed(&log).map(|outcome| outcome.absorbed()), Some(true));
		assert_eq!(
			replayed(&log).map(|outcome| outcome.ticks),
			Some(sim.outcome().unwrap().ticks)
		);
		// The same play against the husk at its full strength does not hold.
		assert!(
			!replayed(&HuntLog::new(bot.recorded.clone()))
				.is_some_and(|outcome| outcome.absorbed())
		);
	}
}

mod a_keeper_that_pays_for_real {
	use super::*;

	const HUSK: Nft = Nft::new(NftCollection::DegenApes, 1);

	fn now() -> Timestamp {
		Timestamp::utc(2026, 10, 2, 12, 0, 0)
	}

	fn later() -> Timestamp {
		now().plus_seconds(60)
	}

	fn released() -> PlayerState {
		PlayerState::initial()
			.take_oath(11)
			.unwrap()
			.acquire(HUSK)
			.unwrap()
			.release(HUSK, 5, now())
			.unwrap()
	}

	fn played(state: &PlayerState, difficulty: Difficulty) -> HuntLog {
		let [sighting] = state.sightings() else {
			panic!("one sighting");
		};
		let mut sim = HuntSim::new(
			sighting.seed,
			HUSK.collection.vessel(),
			HUSK.rarity().ward(),
			&HuntStats::derive(&state.loadout(), 1.0).unwrap(),
			HuntPlan::OPEN,
			difficulty,
		)
		.unwrap();
		let mut bot = HuntBot::careful(&sim);

		bot.play(&mut sim);

		HuntLog {
			difficulty,
			..HuntLog::new(bot.recorded)
		}
	}

	fn only_normal() -> Judging {
		Judging {
			difficulties: vec![Difficulty::Normal],
			..Judging::default()
		}
	}

	#[test]
	fn takes_a_hunt_played_on_easy_while_it_is_only_being_tested() {
		let state = released();
		let settlement = state
			.judge_hunt(
				1,
				&played(&state, Difficulty::Easy),
				later(),
				1,
				&Judging::default(),
			)
			.unwrap();

		assert!(settlement.outcome.absorbed());
	}

	#[test]
	fn refuses_one_once_only_the_game_itself_is_allowed() {
		let state = released();
		let refused = state.judge_hunt(
			1,
			&played(&state, Difficulty::Easy),
			later(),
			1,
			&only_normal(),
		);

		assert_eq!(refused.unwrap_err().refusal, Refusal::HuntUnproven);
		assert!(
			state
				.judge_hunt(
					1,
					&played(&state, Difficulty::Normal),
					later(),
					1,
					&only_normal(),
				)
				.unwrap()
				.outcome
				.absorbed()
		);
	}
}
