//! The numbers of a capture: the chance, the throw and the roll.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

use reapers_rules::CaptureGrade;
use reapers_rules::CaptureRules;
use reapers_rules::Rarity;
use reapers_rules::ReapGrade;
use reapers_rules::SeededRandom;
use reapers_rules::mix_seed;

#[test]
fn the_chance_matches_the_formula_for_worked_values() {
	// 0.4 × (3 − 1.2) / 3 × 1.3 × (0.85 + 0.25)
	assert!(
		(CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Rare),
			0.6,
			CaptureGrade::Great,
			5,
			0,
		) - 0.3432)
			.abs()
			< 1e-9
	);
	// 0.6 × (3 − 0.6) / 3 × 1.5 × (0.85 + 0.1)
	assert!(
		(CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Uncommon),
			0.3,
			CaptureGrade::Excellent,
			2,
			0,
		) - 0.684)
			.abs()
			< 1e-9
	);
	// 0.25 × 1 × 1 × (0.85 + 0.05)
	assert!(
		(CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Epic),
			0.0,
			CaptureGrade::Plain,
			1,
			0,
		) - 0.225)
			.abs()
			< 1e-9
	);
}

#[test]
fn a_broken_husk_is_three_times_as_easy_to_hold_as_a_whole_one() {
	let at = |share| {
		CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Epic),
			share,
			CaptureGrade::Plain,
			3,
			0,
		)
	};

	assert!((at(0.0) / at(1.0) - 3.0).abs() < 1e-9);
	assert_eq!(at(1.5), at(1.0), "the share is clamped");
}

#[test]
fn every_orb_a_husk_broke_out_of_makes_the_next_likelier_to_a_cap() {
	let after = |failures| {
		CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Epic),
			0.0,
			CaptureGrade::Plain,
			3,
			failures,
		)
	};

	assert!((after(2) - (0.25 + 2.0 * CaptureRules::PITY_PER_FAILURE)).abs() < 1e-9);
	assert!((after(5) - (0.25 + CaptureRules::PITY_CAP)).abs() < 1e-9);
}

#[test]
fn the_chance_is_never_certain() {
	assert_eq!(
		CaptureRules::chance(
			CaptureRules::rarity_factor(Rarity::Common),
			0.0,
			CaptureGrade::Excellent,
			5,
			3,
		),
		CaptureRules::CHANCE_CAP
	);
}

#[test]
fn rarer_husks_are_harder_to_hold() {
	let factors = [
		CaptureRules::rarity_factor(Rarity::Common),
		CaptureRules::rarity_factor(Rarity::Uncommon),
		CaptureRules::rarity_factor(Rarity::Rare),
		CaptureRules::rarity_factor(Rarity::Epic),
	];

	assert_eq!(factors, [0.8, 0.6, 0.4, 0.25]);
}

#[test]
fn the_throw_is_graded_by_where_the_ring_is_when_orb_is_let_go() {
	let at = |held| CaptureRules::grade_for(held, true);

	assert_eq!(
		at(CaptureRules::SHORTEST_HOLD_TICKS - 1),
		CaptureGrade::Plain
	);
	assert_eq!(at(CaptureRules::SHORTEST_HOLD_TICKS), CaptureGrade::Plain);
	assert_eq!(
		at(CaptureRules::RING_TICKS / 3 - 1),
		CaptureGrade::Plain,
		"the outer third grades no better than no hold"
	);
	assert_eq!(at(CaptureRules::RING_TICKS / 3), CaptureGrade::Great);
	assert_eq!(
		at(CaptureRules::RING_TICKS * 2 / 3),
		CaptureGrade::Excellent
	);
	assert_eq!(at(CaptureRules::RING_TICKS - 1), CaptureGrade::Excellent);
	assert_eq!(
		at(CaptureRules::RING_TICKS),
		CaptureGrade::Plain,
		"the ring starts again"
	);
}

#[test]
fn off_the_husk_is_never_graded() {
	assert_eq!(
		CaptureRules::grade_for(CaptureRules::RING_TICKS - 1, false),
		CaptureGrade::Plain
	);
}

#[test]
fn the_ring_shrinks_from_the_edge_to_the_husk_then_starts_again() {
	assert_eq!(CaptureRules::ring_progress(0), 0.0);
	assert_eq!(
		CaptureRules::ring_progress(CaptureRules::RING_TICKS / 2),
		0.5
	);
	assert_eq!(CaptureRules::ring_progress(CaptureRules::RING_TICKS), 0.0);
}

#[test]
fn a_better_throw_pays_more_and_cuts_deeper_in_a_rift() {
	let rewards = [
		CaptureGrade::Plain.reward(),
		CaptureGrade::Great.reward(),
		CaptureGrade::Excellent.reward(),
	];

	assert_eq!(rewards, [1.0, 1.25, 1.5]);

	let cuts = [
		CaptureGrade::Plain.cut(),
		CaptureGrade::Great.cut(),
		CaptureGrade::Excellent.cut(),
	];

	assert_eq!(
		cuts,
		[ReapGrade::Ragged, ReapGrade::Clean, ReapGrade::Perfect]
	);
}

#[test]
fn a_grade_saved_before_the_orb_reads_as_the_one_that_pays_the_same() {
	assert_eq!(
		CaptureGrade::from_saved("ragged"),
		Some(CaptureGrade::Plain)
	);
	assert_eq!(CaptureGrade::from_saved("clean"), Some(CaptureGrade::Great));
	assert_eq!(
		CaptureGrade::from_saved("perfect"),
		Some(CaptureGrade::Excellent)
	);
	assert_eq!(CaptureGrade::from_saved("plain"), Some(CaptureGrade::Plain));
	assert_eq!(CaptureGrade::from_saved("great"), Some(CaptureGrade::Great));
	assert_eq!(
		CaptureGrade::from_saved("excellent"),
		Some(CaptureGrade::Excellent)
	);
}

#[test]
fn the_roll_is_the_same_for_the_same_hunt_and_attempt() {
	assert_eq!(CaptureRules::roll(42, 0), CaptureRules::roll(42, 0));
	assert_eq!(
		CaptureRules::roll(42, 0),
		SeededRandom::new(mix_seed(&[42, 0x0B5, 0])).next_double()
	);
	assert_ne!(CaptureRules::roll(42, 1), CaptureRules::roll(42, 0));
	assert_ne!(CaptureRules::roll(43, 0), CaptureRules::roll(42, 0));
}

#[test]
fn the_orb_holds_about_as_often_as_the_chance_says() {
	let chance = 0.5;
	let mut held = 0;

	for seed in 0..4000 {
		if CaptureRules::break_pulse(CaptureRules::roll(seed, 0), chance).is_none() {
			held += 1;
		}
	}

	assert!(((f64::from(held) / 4000.0) - chance).abs() < 0.03);
}

#[test]
fn an_orb_breaks_later_the_nearer_it_came() {
	let chance = 0.7;

	assert_eq!(CaptureRules::break_pulse(0.69, chance), None);
	assert_eq!(CaptureRules::break_pulse(0.7, chance), Some(3));
	assert_eq!(CaptureRules::break_pulse(0.75, chance), Some(3));
	assert_eq!(CaptureRules::break_pulse(0.85, chance), Some(2));
	assert_eq!(CaptureRules::break_pulse(0.95, chance), Some(1));
	assert_eq!(CaptureRules::break_pulse(0.999, chance), Some(1));
}
