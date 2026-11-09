//! Birth gifts and their awakenings, in the hunt.

use reapers_rules::*;

/// How many of `events` match `pattern`.
macro_rules! count {
	($events:expr, $pattern:pat) => {
		$events
			.iter()
			.filter(|event| matches!(event, $pattern))
			.count()
	};
}

fn hunt_with(gift: ReaperGift, rank: i64, seed: i64) -> HuntSim {
	let reaper_class = gift.reaper_class();

	HuntSim::open(
		seed,
		Vessel::Penguin,
		Rarity::Common,
		1000,
		&HuntStats::derive(
			&HuntLoadout {
				soul: Some(ReaperSoul {
					gift,
					temperament: Temperament::Keen,
				}),
				soul_rank: rank,
				..HuntLoadout::new(
					reaper_class,
					1,
					Weapon::new(reaper_class.weapon(), Temper::Shrouded),
				)
			},
			1.0,
		)
		.unwrap(),
	)
}

/// [`hunt_with`] with the usual seed, 19.
fn hunt(gift: ReaperGift, rank: i64) -> HuntSim {
	hunt_with(gift, rank, 19)
}

fn still(sim: &HuntSim) -> HuntInput {
	HuntInput::still(sim.pose().direction(), 0.0)
}

/// Holds still on the husk until `reached`, and returns what happened.
fn until(sim: &mut HuntSim, reached: impl Fn(&HuntSim) -> bool) -> Vec<HuntEvent> {
	let mut events = Vec::new();

	while !reached(sim) && !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
		let input = still(sim);

		events.extend(sim.step(&input));
	}

	assert!(reached(sim), "encounter ended before the action");

	events
}

mod birth_gifts_change_the_fight {
	use super::*;

	/// Plays a hunt for a reaper born with `gift` and checks the gift
	/// stirred.
	fn can_be_activated_through_player_actions(gift: ReaperGift) {
		let mut sim = hunt(gift, 0);
		let mut bot = HuntBot::new(
			&sim,
			HuntBotStyle {
				miss_every: 7,
				..HuntBotStyle::CAREFUL
			},
		);
		let mut events = Vec::new();
		let mut shrieks = 0;

		while !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
			let decision = bot.decide(&sim);
			// Anchor needs lost resolve; Return Light needs a mistake after
			// its cadence promise. Other gifts face careful play and misses.
			let fail_charge = gift == ReaperGift::Anchor && sim.phase() == HuntPhase::Charge;
			let fail_shriek =
				gift == ReaperGift::ReturnLight && sim.phase() == HuntPhase::Shriek && shrieks > 0;
			let input = if fail_charge {
				still(&sim)
			} else if fail_shriek {
				HuntInput::new(decision.aim, 3000)
			} else {
				decision
			};
			let next = sim.step(&input);

			if next
				.iter()
				.any(|event| matches!(event, HuntEvent::ShriekBegan))
			{
				shrieks += 1;
			}

			events.extend(next);
		}

		assert!(
			events
				.iter()
				.any(|event| matches!(event, HuntEvent::GiftStirred(stirred) if *stirred == gift)),
			"{gift}: a documented birth power must be reachable in combat"
		);
	}

	/// One test for every gift.
	mod can_be_activated_through_player_actions {
		use super::*;

		// Every gift has a test here.
		const _: () = assert!(ReaperGift::ALL.len() == 18);

		macro_rules! gift_tests {
			($($name:ident: $gift:ident),+ $(,)?) => {
				$(
					#[test]
					fn $name() {
						can_be_activated_through_player_actions(ReaperGift::$gift);
					}
				)+
			};
		}

		gift_tests!(
			furrow: Furrow,
			aftermath: Aftermath,
			gathering: Gathering,
			passage: Passage,
			anchor: Anchor,
			return_light: ReturnLight,
			foreknowledge: Foreknowledge,
			verdict: Verdict,
			equilibrium: Equilibrium,
			first_light: FirstLight,
			reprisal: Reprisal,
			last_word: LastWord,
			hunger: Hunger,
			echo: Echo,
			unbroken: Unbroken,
			hush: Hush,
			keening: Keening,
			refrain: Refrain,
		);
	}

	#[test]
	fn hunger_starts_cadence_but_never_fabricates_five_landed_hits() {
		let mut sim = hunt(ReaperGift::Hunger, 3);

		until(&mut sim, |sim| sim.phase() == HuntPhase::Fight);

		assert_eq!(sim.cadence(), 4);
		assert_eq!(sim.deeds().max_cadence, 0);

		let input = still(&sim);

		sim.step(&input);

		assert_eq!(sim.cadence(), 4);
	}

	#[test]
	fn passage_protects_once_then_an_unmet_charge_costs_resolve() {
		let mut sim = hunt(ReaperGift::Passage, 0);

		until(&mut sim, |sim| sim.phase() == HuntPhase::Charge);

		let resolve = sim.resolve();
		let protected = until(&mut sim, |sim| sim.phase() != HuntPhase::Charge);

		assert_eq!(count!(protected, HuntEvent::Parried), 1);
		assert_eq!(sim.resolve(), resolve);
		assert_eq!(sim.deeds().parries, 0);

		until(&mut sim, |sim| sim.phase() == HuntPhase::Charge);

		let struck = until(&mut sim, |sim| sim.phase() != HuntPhase::Charge);

		assert_eq!(count!(struck, HuntEvent::Struck { .. }), 1);
		assert_eq!(sim.resolve(), resolve - 1);
	}

	#[test]
	fn awakened_passage_lengthens_protection_without_adding_uses() {
		let stagger = |rank: i64| {
			let mut sim = hunt(ReaperGift::Passage, rank);

			until(&mut sim, |sim| sim.phase() == HuntPhase::Charge);
			until(&mut sim, |sim| sim.phase() != HuntPhase::Charge);

			let start = sim.tick();

			until(&mut sim, |sim| !sim.staggered());

			sim.tick() - start
		};

		assert!(stagger(3) > stagger(0));
	}

	#[test]
	fn hush_delays_the_first_warning_further_as_it_awakens() {
		let first_warning = |rank: i64| {
			let mut sim = hunt(ReaperGift::Hush, rank);

			until(&mut sim, |sim| sim.phase() == HuntPhase::Windup);

			sim.tick()
		};

		assert!(first_warning(3) > first_warning(0));
	}

	#[test]
	fn refrain_keeps_a_broken_husk_down_longer_while_keeping_its_six_toll_cap() {
		let window = |rank: i64| {
			let mut sim = hunt(ReaperGift::Refrain, rank);
			let mut bot = HuntBot::careful(&sim);

			bot.play(&mut sim);

			let holds = count!(bot.events, HuntEvent::HeldFirm) as i64;
			let tolls = count!(bot.events, HuntEvent::GiftStirred(ReaperGift::Refrain)) as i64;

			assert!((1..=SoulRules::REFRAIN_MAX_TOLLS).contains(&tolls));

			sim.broken_ticks() - holds * HuntRules::TOLL_TICKS
		};

		assert!(window(3) > window(0));
	}
}

mod untrusted_combat_logs {
	use super::*;

	#[test]
	fn impossible_aims_are_refused() {
		for input in [
			HuntInput::new(Aim::new(18001, 0), 0),
			HuntInput::new(Aim::new(0, -9001), 0),
		] {
			assert!(HuntLog::decode(&HuntLog::new(vec![input]).encode()).is_err());
		}
	}

	#[test]
	fn a_bounded_log_is_accepted_its_first_excess_tick_is_refused() {
		let idle = HuntInput::new(Aim::AHEAD, 0);
		let limit = HuntLog::new(vec![idle; HuntLog::MAX_TICKS]).encode();

		assert_eq!(
			HuntLog::decode(&limit).unwrap().inputs.len(),
			HuntLog::MAX_TICKS
		);

		let excess = [limit, vec![0; 7]].concat();

		assert!(HuntLog::decode(&excess).is_err());
	}
}
