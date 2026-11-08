//! Reaper genomes: drawing them, writing them, and reading them back.

mod support;

use reapers_rules::*;

mod reaper_stats {
	use super::*;

	#[test]
	fn every_class_totals_the_same_so_no_shuffle_is_simply_better() {
		for reaper_class in ReaperClass::ALL {
			assert_eq!(
				reaper_class.stats().total(),
				ReaperStats::CLASS_TOTAL,
				"{}",
				reaper_class.name()
			);
		}
	}

	#[test]
	fn every_class_has_epithets_and_a_weapon_profile() {
		for reaper_class in ReaperClass::ALL {
			assert_ne!(reaper_epithets(reaper_class).len(), 0);

			// Every weapon kind has a profile: the lookup is total.
			let profile = weapon_profile(reaper_class.weapon());
			assert!(profile.strike > 0.0);
		}
	}
}

mod reaper_genome_shuffle {
	use std::collections::HashSet;

	use super::*;

	#[test]
	fn the_same_seed_always_draws_the_same_reaper() {
		assert_eq!(
			ReaperGenome::shuffle(42).code(),
			ReaperGenome::shuffle(42).code()
		);
	}

	#[test]
	fn different_seeds_draw_different_reapers() {
		let codes: HashSet<String> = (0..500)
			.map(|seed| ReaperGenome::shuffle(seed).code())
			.collect();

		assert!(codes.len() > 495, "{}", codes.len());
	}

	#[test]
	fn every_class_and_every_omen_turns_up() {
		let genomes: Vec<ReaperGenome> = (0..4000).map(ReaperGenome::shuffle).collect();

		assert_eq!(
			genomes
				.iter()
				.map(|genome| genome.reaper_class)
				.collect::<HashSet<_>>(),
			ReaperClass::ALL.into_iter().collect::<HashSet<_>>()
		);
		assert_eq!(
			genomes
				.iter()
				.map(|genome| genome.omen)
				.collect::<HashSet<_>>(),
			Omen::ALL.into_iter().collect::<HashSet<_>>()
		);
	}

	#[test]
	fn an_omen_is_rare() {
		let marked = (0..4000)
			.filter(|&seed| ReaperGenome::shuffle(seed).omen != Omen::None)
			.count();
		let share = marked as f64 / 4000.0;

		assert!((share - 0.2).abs() <= 0.04, "{share}");
	}
}

mod reaper_genome_code {
	use super::*;

	/// `code` with the first of its fields after the version that `matches`
	/// replaced by `replacement`: the first field, between two dashes, that
	/// matches.
	fn replace_field(code: &str, matches: impl Fn(&str) -> bool, replacement: &str) -> String {
		let mut fields: Vec<&str> = code.split('-').collect();
		let index = (1..fields.len())
			.find(|&index| matches(fields[index]))
			.unwrap();

		fields[index] = replacement;
		fields.join("-")
	}

	/// Whether `field` is `prefix` followed by one or more digits.
	fn numbered(field: &str, prefix: &str) -> bool {
		field.strip_prefix(prefix).is_some_and(|digits| {
			!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
		})
	}

	#[test]
	fn round_trips_exactly() {
		for seed in 0..300 {
			let genome = ReaperGenome::shuffle(seed);
			let parsed = ReaperGenome::parse(&genome.code()).unwrap();

			assert_eq!(parsed, genome);
			assert_eq!(parsed.name(), genome.name());
			assert_eq!(parsed.stature, genome.stature);
		}
	}

	#[test]
	fn the_borrowed_reaper_has_a_valid_code() {
		assert_eq!(
			ReaperGenome::parse(&ReaperGenome::BORROWED.code()).unwrap(),
			ReaperGenome::BORROWED
		);
	}

	#[test]
	fn rejects_a_malformed_code() {
		let valid = ReaperGenome::shuffle(1).code();
		let class_codes = ["har", "fer", "wei", "cho", "hol", "mou"];

		for broken in [
			String::new(),
			"nonsense".to_owned(),
			valid.replacen("R1", "R9", 1),
			replace_field(&valid, |field| numbered(field, "h"), "h99"),
			replace_field(&valid, |field| numbered(field, "s"), "s5000"),
			replace_field(&valid, |field| class_codes.contains(&field), "xyz"),
			format!("{valid}-extra"),
		] {
			assert!(ReaperGenome::parse(&broken).is_err(), "{broken}");
		}
	}
}

mod seeded_random {
	use super::*;

	#[test]
	fn has_a_fixed_sequence_that_must_never_change() {
		let mut random = SeededRandom::new(1);

		// These values are what minted reapers and recorded hunts were drawn
		// from. Changing the generator changes every one of them.
		assert_eq!(
			(0..4).map(|_| random.next_u32()).collect::<Vec<_>>(),
			[2_693_262_067, 11_749_833, 2_265_367_787, 4_213_581_821]
		);
	}

	#[test]
	fn stays_inside_its_bounds() {
		let mut random = SeededRandom::new(99);

		for _ in 0..2000 {
			let double = random.next_double();
			assert!((0.0..=0.999_999_999_9).contains(&double), "{double}");

			let int = random.next_int(7);
			assert!((0..=6).contains(&int), "{int}");
		}
	}

	#[test]
	fn mix_seed_separates_nearby_inputs() {
		assert_ne!(mix_seed(&[1, 2]), mix_seed(&[2, 1]));
		assert_ne!(hash_text("degenApes:1"), hash_text("degenApes:2"));
	}
}

mod progression {
	use super::*;

	#[test]
	fn levels_need_more_experience_each_time_up_to_the_cap() {
		for level in 1..Progression::LEVEL_CAP - 1 {
			assert!(
				Progression::xp_to_next(level + 1) > Progression::xp_to_next(level),
				"{level}"
			);
		}

		assert_eq!(Progression::level_for_xp(0), 1);
		assert_eq!(Progression::level_for_xp(Progression::xp_to_next(1) - 1), 1);
		assert_eq!(Progression::level_for_xp(Progression::xp_to_next(1)), 2);
		assert_eq!(Progression::level_for_xp(1 << 40), Progression::LEVEL_CAP);
		assert!((Progression::level_progress(1 << 40) - 1.0).abs() < f64::EPSILON);
	}

	#[test]
	fn xp_for_level_is_the_inverse_of_level_for_xp() {
		for level in 1..=Progression::LEVEL_CAP {
			assert_eq!(
				Progression::level_for_xp(Progression::xp_for_level(level)),
				level
			);
		}
	}
}
