//! The Rust side of `reapers_shared/test/reaper_soul_test.dart`.

mod support;

use reapers_rules::*;

/// `reaper` with its experience and lost hunts changed, refused as Dart's
/// `ReaperState.copyWith` refuses a record no reaper could have.
fn with_xp_and_losses(reaper: &ReaperState, xp: i64, hunts_lost: i64) -> ReaperState {
	ReaperState::new(
		reaper.id().to_owned(),
		*reaper.genome(),
		xp,
		reaper.weapons().to_vec(),
		reaper.outfits().to_vec(),
		reaper.weapon(),
		reaper.outfit(),
		reaper.codex().to_vec(),
		hunts_lost,
		reaper.rifts_sealed(),
		*reaper.memory(),
		reaper.art(),
	)
	.unwrap()
}

mod birth_identity {
	use std::collections::HashSet;

	use super::*;

	#[test]
	fn r1_identity_round_trips_rare_decoration_never_changes_power() {
		for seed in 0..200 {
			let genome = ReaperGenome::shuffle(seed);
			let soul = ReaperSoul::from_genome(&genome);
			let restored = ReaperSoul::from_genome(&ReaperGenome::parse(&genome.code()).unwrap());

			assert_eq!(restored.gift, soul.gift);
			assert_eq!(restored.temperament, soul.temperament);
			assert_eq!(soul.gift.reaper_class(), genome.reaper_class);

			for omen in Omen::ALL {
				let decorated = ReaperSoul::from_genome(
					&genome
						.with(GenomeChanges {
							omen: Some(omen),
							palette: Some(2),
							aura: Some(Aura::Embers),
							..Default::default()
						})
						.unwrap(),
				);
				assert_eq!(decorated.gift, soul.gift);
				assert_eq!(decorated.temperament, soul.temperament);
			}
		}
	}

	#[test]
	fn all_72_birth_builds_occur_each_class_has_one_gift_per_path() {
		let builds: HashSet<(ReaperGift, Temperament)> = (0..4000)
			.map(|seed| {
				let soul = ReaperSoul::from_genome(&ReaperGenome::shuffle(seed));

				(soul.gift, soul.temperament)
			})
			.collect();
		assert_eq!(builds.len(), 72);

		for reaper_class in ReaperClass::ALL {
			let paths: HashSet<SoulPath> = ReaperGift::ALL
				.into_iter()
				.filter(|gift| gift.reaper_class() == reaper_class)
				.map(ReaperGift::path)
				.collect();
			assert_eq!(paths, SoulPath::ALL.into_iter().collect::<HashSet<_>>());
		}
	}

	#[test]
	fn each_disposition_benefits_one_affordance_and_pays_a_real_cost() {
		let baseline = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
		let with_temperament = |temperament: Temperament| {
			HuntStats::derive(
				&HuntLoadout {
					soul: Some(ReaperSoul {
						gift: ReaperGift::Furrow,
						temperament,
					}),
					..HuntLoadout::BORROWED
				},
				1.0,
			)
			.unwrap()
		};
		let patient = with_temperament(Temperament::Patient);
		assert!(patient.capture_ticks > baseline.capture_ticks);
		assert!(patient.swing_ticks > baseline.swing_ticks);
		let keen = with_temperament(Temperament::Keen);
		assert!(keen.reach_radians > baseline.reach_radians);
		assert!(keen.capture_ticks < baseline.capture_ticks);
		let stalwart = with_temperament(Temperament::Stalwart);
		assert!(stalwart.resolve > baseline.resolve);
		assert!(stalwart.strike_damage < baseline.strike_damage);
		let hungry = with_temperament(Temperament::Hungry);
		assert!(hungry.strike_damage > baseline.strike_damage);
		assert!(hungry.flinch_speed < baseline.flinch_speed);
	}
}

mod earned_memory {
	use serde_json::json;

	use super::*;

	const RICH: HuntOutcome = HuntOutcome::absorbed_with(
		ReapGrade::Perfect,
		true,
		2000,
		HuntDeeds {
			holds: 12,
			parries: 8,
			max_cadence: 40,
		},
	);

	#[test]
	fn a_successful_hunt_awards_at_most_one_mark_per_path() {
		let memory = SoulMemory::default().remember(&RICH);
		assert_eq!(
			serde_json::to_value(memory).unwrap(),
			json!({"unseen": 1, "stillness": 1, "rhythm": 1})
		);
		assert_eq!(memory.remember(&RICH).rhythm, 2);
		let restored: HuntOutcome =
			serde_json::from_value(serde_json::to_value(RICH).unwrap()).unwrap();
		assert_eq!(
			serde_json::to_value(restored.deeds).unwrap(),
			serde_json::to_value(RICH.deeds).unwrap()
		);
	}

	#[test]
	fn escapes_and_incomplete_cadence_cannot_farm_growth() {
		let escaped = HuntOutcome::escaped_by(
			EscapeCause::Time,
			true,
			5000,
			HuntDeeds {
				holds: 20,
				max_cadence: 50,
				..Default::default()
			},
		);
		let short = HuntOutcome::absorbed_with(
			ReapGrade::Clean,
			false,
			800,
			HuntDeeds {
				max_cadence: 4,
				..Default::default()
			},
		);
		assert_eq!(
			serde_json::to_value(SoulMemory::default().remember(&escaped)).unwrap(),
			serde_json::to_value(SoulMemory::default()).unwrap()
		);
		assert_eq!(SoulMemory::default().remember(&short).rhythm, 0);
	}

	#[test]
	fn level_alone_cannot_awaken_a_gift_other_paths_cannot_substitute() {
		let rank = |level: i64, marks: i64| {
			SoulRules::rank(
				level,
				&SoulMemory {
					stillness: marks,
					..Default::default()
				},
				ReaperGift::Anchor,
			)
		};
		assert_eq!(rank(30, 0), 0);
		assert_eq!(rank(4, 30), 0);
		assert_eq!(rank(5, 2), 0);
		assert_eq!(rank(5, 3), 1);
		assert_eq!(rank(11, 30), 1);
		assert_eq!(rank(12, 12), 2);
		assert_eq!(rank(19, 30), 2);
		assert_eq!(rank(20, 30), 3);
		assert_eq!(
			SoulRules::rank(
				30,
				&SoulMemory {
					unseen: 100,
					rhythm: 100,
					..Default::default()
				},
				ReaperGift::Anchor,
			),
			0
		);
	}

	#[test]
	fn legacy_saves_keep_growth_without_inventing_remembered_deeds() {
		let veteran = with_xp_and_losses(
			&ReaperState::bound(ReaperState::OATH_ID, ReaperGenome::shuffle(13)),
			Progression::xp_for_level(12),
			3,
		);
		let mut json = serde_json::to_value(&veteran).unwrap();
		json.as_object_mut().unwrap().remove("memory");
		let restored: ReaperState = serde_json::from_value(json).unwrap();
		assert_eq!(restored.level(), 12);
		assert_eq!(restored.genome(), veteran.genome());
		assert_eq!(restored.hunts_lost(), 3);
		assert_eq!(restored.soul_rank(), 0);
	}

	#[test]
	fn invalid_stored_counters_are_refused_rather_than_clamped() {
		assert!(
			serde_json::from_value::<SoulMemory>(
				json!({"unseen": -1, "stillness": 0, "rhythm": 0})
			)
			.is_err()
		);
		assert!(
			serde_json::from_value::<HuntDeeds>(
				json!({"holds": -1, "parries": 0, "maxCadence": 0})
			)
			.is_err()
		);
	}
}

mod power_replay {
	use super::*;

	/// Plays a hunt as `gift` awakened to `rank` and checks that its log
	/// replays to the same result, and that its deeds are the ones played.
	fn replays_every_result_and_deed(gift: ReaperGift, rank: i64) {
		let stats = HuntStats::derive(
			&HuntLoadout {
				soul: Some(ReaperSoul {
					gift,
					temperament: Temperament::Keen,
				}),
				soul_rank: rank,
				..HuntLoadout::new(
					gift.reaper_class(),
					4,
					Weapon::new(gift.reaper_class().weapon(), Temper::Shrouded),
				)
			},
			1.0,
		)
		.unwrap();
		let mut sim = HuntSim::open(19, Vessel::Penguin, 400, &stats);
		let mut bot = HuntBot::new(
			&sim,
			HuntBotStyle {
				miss_every: 7,
				..HuntBotStyle::CAREFUL
			},
		);
		let outcome = bot.play(&mut sim);
		let log = HuntLog::decode(&HuntLog::new(bot.recorded.clone()).encode()).unwrap();
		let replay = HuntSim::replay(19, Vessel::Penguin, 400, &stats, &log);
		let outcome = outcome.expect("the hunt ends");
		assert_eq!(
			replay.map(|replay| serde_json::to_value(replay).unwrap()),
			Some(serde_json::to_value(outcome).unwrap())
		);
		assert_eq!(
			outcome.deeds.holds,
			bot.events
				.iter()
				.filter(|event| matches!(event, HuntEvent::HeldFirm))
				.count() as i64
		);
		// Passage's automatic protection must not count as a player parry.
		let protections = bot
			.events
			.iter()
			.filter(|event| matches!(event, HuntEvent::GiftStirred(ReaperGift::Passage)))
			.count();
		let parried = bot
			.events
			.iter()
			.filter(|event| matches!(event, HuntEvent::Parried))
			.count();
		assert_eq!(
			outcome.deeds.parries,
			(parried - usize::from(protections > 1)) as i64
		);
	}

	/// A test for each gift at awakening 0 and at the highest awakening,
	/// [`SoulRules::MAX_RANK`].
	macro_rules! power_replays {
		($($name:ident => $gift:ident),+ $(,)?) => {
			$(
				mod $name {
					use super::*;

					#[test]
					fn at_awakening_0_replays_every_result_and_deed() {
						replays_every_result_and_deed(ReaperGift::$gift, 0);
					}

					#[test]
					fn at_awakening_3_replays_every_result_and_deed() {
						assert_eq!(SoulRules::MAX_RANK, 3);
						replays_every_result_and_deed(ReaperGift::$gift, SoulRules::MAX_RANK);
					}
				}
			)+

			#[test]
			fn every_gift_is_replayed() {
				let named = [$(ReaperGift::$gift),+];

				assert_eq!(named.as_slice(), ReaperGift::ALL.as_slice());
			}
		};
	}

	power_replays! {
		furrow => Furrow,
		aftermath => Aftermath,
		gathering => Gathering,
		passage => Passage,
		anchor => Anchor,
		return_light => ReturnLight,
		foreknowledge => Foreknowledge,
		verdict => Verdict,
		equilibrium => Equilibrium,
		first_light => FirstLight,
		reprisal => Reprisal,
		last_word => LastWord,
		hunger => Hunger,
		echo => Echo,
		unbroken => Unbroken,
		hush => Hush,
		keening => Keening,
		refrain => Refrain,
	}

	#[test]
	fn previous_combat_logs_cannot_be_interpreted_under_new_powers() {
		let mut bytes = HuntLog::new(Vec::new()).encode();
		bytes[0] = 4;
		assert!(HuntLog::decode(&bytes).is_err());
	}
}
