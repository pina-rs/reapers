//! The Rust side of `reapers_shared/test/room_cover_test.dart`.

mod support;

use reapers_rules::*;
use support::*;

const MAX_TICKS: i64 = HuntLog::MAX_TICKS as i64;

fn borrowed() -> HuntStats {
	HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap()
}

fn sim_in(seed: i64, vessel: Vessel, plan: HuntPlan, rarity: Rarity) -> HuntSim {
	HuntSim::new(
		seed,
		vessel,
		rarity.ward(),
		&borrowed(),
		plan,
		Difficulty::Normal,
	)
	.unwrap()
}

/// An epic ape in the kitchen.
fn sim(seed: i64) -> HuntSim {
	sim_in(seed, Vessel::Ape, kitchen(), Rarity::Epic)
}

/// Plays `sim` with a careful bot, calling `look` after every tick.
fn play(sim: &mut HuntSim, mut look: impl FnMut(&HuntSim, &[HuntEvent])) {
	let mut bot = HuntBot::human(sim);

	while !sim.is_over() && sim.tick() < MAX_TICKS {
		let events = bot.step_once(sim);

		look(sim, &events);
	}
}

fn range(at: Vec3) -> f64 {
	(at.x * at.x + at.z * at.z).sqrt()
}

fn close(actual: f64, expected: f64) -> bool {
	(actual - expected).abs() <= 1e-9
}

mod what_the_plan_knows_of_cover {
	use super::*;

	#[test]
	fn something_is_hidden_only_when_the_counter_is_in_front_of_it() {
		let plan = kitchen();
		let eye = 1.45;

		// Behind the counter: low down hidden, high up seen over the top.
		assert!(plan.hides(Vec3::new(0.0, -eye + 0.4, 3.2)));
		assert!(!plan.hides(Vec3::new(0.0, -eye + 1.3, 3.2)));
		// In front of it, nothing is hidden however low.
		assert!(!plan.hides(Vec3::new(0.0, -eye + 0.05, 2.0)));
		// Where nothing stands, nothing is hidden.
		assert!(!plan.hides(Vec3::new(3.2, -eye + 0.05, 0.0)));
		// A plan that knows no cover hides nothing.
		assert!(!HuntPlan::OPEN.hides(Vec3::new(0.0, -eye, 3.2)));
	}

	#[test]
	fn a_plan_with_cover_travels_whole_with_the_record_of_its_hunt() {
		let plan = kitchen();
		let log = HuntLog {
			plan: plan.clone(),
			..HuntLog::new(vec![
				HuntInput {
					aim: Aim::new(120, -400),
					turn_speed: 30,
					taps: Vec::new(),
					focus: true,
				},
				HuntInput::new(Aim::new(-50, 10), 2000),
			])
		};
		let decoded = HuntLog::decode(&log.encode()).unwrap();

		assert_eq!(decoded.plan, plan);
		assert!(close(decoded.plan.ledge_at(36).unwrap(), 0.92));
		assert!(decoded.inputs[0].focus);
		assert!(!decoded.inputs[1].focus);
		assert_eq!(decoded.inputs[1].turn_speed, 2000);
	}

	#[test]
	fn cover_no_phone_could_have_measured_is_refused() {
		let good = kitchen();

		for (label, plan) in [
			(
				"cover without a reach to stand at",
				HuntPlan {
					cover_centimetres: good.cover_centimetres.clone(),
					hide_decimetres: good.hide_decimetres.clone(),
					ledges: good.ledges.clone(),
					..HuntPlan::new(0, 9000)
				},
			),
			(
				"a hiding place in front of its cover",
				HuntPlan {
					reach_decimetres: good.reach_decimetres.clone(),
					cover_centimetres: good.cover_centimetres.clone(),
					hide_decimetres: good
						.hide_decimetres
						.iter()
						.map(|&hide| if hide == 0 { 0 } else { 10 })
						.collect(),
					ledges: good.ledges.clone(),
					..HuntPlan::new(0, 9000)
				},
			),
			(
				"a hiding place behind nothing",
				HuntPlan {
					reach_decimetres: good.reach_decimetres.clone(),
					cover_centimetres: vec![0; HuntPlan::SLICES],
					hide_decimetres: good.hide_decimetres.clone(),
					ledges: good.ledges.clone(),
					..HuntPlan::new(0, 9000)
				},
			),
		] {
			assert!(!plan.is_well_formed(), "{label}");
			assert!(
				HuntSim::replay(
					1,
					Vessel::Ape,
					280,
					&borrowed(),
					&HuntLog {
						plan,
						..HuntLog::new(Vec::new())
					},
				)
				.is_none(),
				"{label}"
			);
		}
	}
}

mod a_husk_in_a_furnished_room {
	use super::*;

	#[test]
	fn leaps_up_onto_the_counter_and_down_behind_it_like_a_cat() {
		let mut footings = Vec::new();
		let mut leaps = 0;

		for seed in 1..=12 {
			play(&mut sim(seed), |_, events| {
				leaps += events
					.iter()
					.filter(|event| matches!(event, HuntEvent::Leapt(_)))
					.count();
			});

			play(&mut sim(seed), |sim, _| {
				if !sim.leaping() && !footings.contains(&sim.footing()) {
					footings.push(sim.footing());
				}
			});
		}

		assert!(leaps > 10);

		for footing in Footing::ALL {
			assert!(footings.contains(&footing), "{footing}");
		}
	}

	#[test]
	fn stands_on_the_counter_top_and_behind_it_on_the_floor() {
		let floor = -HuntRules::EYE_HEIGHT;

		for seed in 1..=8 {
			play(&mut sim(seed), |sim, _| {
				if sim.leaping() || sim.phase() == HuntPhase::Charge {
					return;
				}

				let pose = sim.pose();
				let feet = pose.position.y - Vessel::Ape.half_height();

				match sim.footing() {
					Footing::Ledge => {
						assert!(close(feet, floor + 0.92));
						assert!(close(pose.ground.unwrap(), floor + 0.92));
						assert!(range(pose.position) > 2.2);
					}
					Footing::Hidden => {
						assert!(close(feet, floor));
						assert!(close(range(pose.position), 3.2));
					}
					Footing::Open => {
						assert!(close(feet, floor));
						assert!(!sim.plan().hides(pose.position));
					}
				}
			});
		}
	}

	#[test]
	fn moves_smoothly_through_every_leap_no_step_longer_than_a_stride() {
		for seed in 1..=8 {
			let mut sim = sim(seed);
			let mut before = sim.pose().position;

			play(&mut sim, |sim, _| {
				let after = sim.pose().position;

				assert!((after - before).length() < 0.2, "seed {seed}");
				before = after;
			});
		}
	}

	#[test]
	fn clears_the_counter_when_it_leaps_over_it() {
		let plan = kitchen();

		for seed in 1..=8 {
			play(&mut sim(seed), |sim, _| {
				let at = sim.pose().position;
				let away = range(at);
				let passing =
					sim.leaping() && (away - 2.2).abs() < 0.05 && plan.cover_at(at.yaw()) > 0.0;

				if passing {
					let feet = at.y - Vessel::Ape.half_height();

					assert!(feet > -HuntRules::EYE_HEIGHT + 0.92);
				}
			});
		}
	}

	#[test]
	fn its_seals_behind_the_counter_are_out_of_sight_and_out_of_reach() {
		let mut hidden_ticks = 0;

		for seed in 1..=12 {
			play(&mut sim(seed), |sim, _| {
				if sim.footing() != Footing::Hidden || sim.leaping() {
					return;
				}

				hidden_ticks += 1;

				let pose = sim.pose();

				for seal in 0..sim.vessel().seals().len() {
					if sim.plan().hides(pose.seal_position(seal)) {
						assert!(!sim.seal_open(seal));
					}
				}
			});
		}

		assert!(hidden_ticks > 0);
	}

	#[test]
	fn a_blow_at_the_part_behind_the_counter_hits_the_counter() {
		let mut sim = sim(5);

		while (sim.footing() != Footing::Hidden || sim.leaping())
			&& sim.tick() < MAX_TICKS
			&& !sim.is_over()
		{
			let input = HuntInput::measured(sim.pose().direction(), 0.0, &[], false);

			sim.step(&input);
		}

		assert_eq!(sim.footing(), Footing::Hidden, "it should hide in time");

		// Low on the body, below the top of the counter.
		let pose = sim.pose();
		let low = Vec3::new(
			pose.position.x,
			pose.position.y - Vessel::Ape.half_height() * 0.8,
			pose.position.z,
		)
		.normalized();

		assert!(sim.plan().hides(low * pose.distance()));

		let events = sim.step(&HuntInput {
			aim: Aim::from_direction(pose.direction()),
			turn_speed: 0,
			taps: vec![Aim::from_direction(low)],
			focus: false,
		});

		assert!(!events.iter().any(|event| {
			matches!(
				event,
				HuntEvent::StrikeGlanced { .. } | HuntEvent::StrikeLanded { .. }
			)
		}));
	}

	#[test]
	fn it_comes_out_from_behind_the_counter_to_shriek() {
		for seed in 1..=12 {
			play(&mut sim(seed), |sim, _| {
				if sim.phase() == HuntPhase::Shriek {
					assert_ne!(sim.footing(), Footing::Hidden, "seed {seed}");
				}
			});
		}
	}

	#[test]
	fn it_charges_only_from_open_floor() {
		for seed in 1..=12 {
			play(&mut sim(seed), |sim, events| {
				if events
					.iter()
					.any(|event| matches!(event, HuntEvent::ChargeBegan))
				{
					assert_eq!(sim.footing(), Footing::Open);
					assert!(!sim.leaping());
				}
			});
		}
	}

	#[test]
	fn a_penguin_cannot_jump_onto_a_counter_but_hides_behind_it() {
		let mut footings = Vec::new();

		for seed in 1..=12 {
			let mut sim = sim_in(seed, Vessel::Penguin, kitchen(), Rarity::Epic);

			play(&mut sim, |sim, _| footings.push(sim.footing()));
		}

		assert!(!footings.contains(&Footing::Ledge));
		assert!(footings.contains(&Footing::Hidden));
	}

	#[test]
	fn a_bird_comes_down_to_perch_on_the_counter_and_never_hides() {
		let mut footings = Vec::new();

		for seed in 1..=12 {
			let mut sim = sim_in(seed, Vessel::Bird, kitchen(), Rarity::Epic);

			play(&mut sim, |sim, _| {
				footings.push(sim.footing());

				if sim.footing() == Footing::Ledge && !sim.leaping() {
					assert!(close(
						sim.pose().position.y - Vessel::Bird.half_height(),
						-HuntRules::EYE_HEIGHT + 0.92,
					));
				}
			});
		}

		assert!(footings.contains(&Footing::Ledge));
		assert!(!footings.contains(&Footing::Hidden));
	}

	#[test]
	fn cover_too_low_to_hide_half_a_husk_is_not_hidden_behind() {
		for seed in 1..=8 {
			let mut sim = sim_in(seed, Vessel::Ape, kitchen_with(40, false), Rarity::Epic);

			play(&mut sim, |sim, _| assert_eq!(sim.footing(), Footing::Open));
		}
	}

	#[test]
	fn a_hunt_in_a_furnished_room_replays_to_the_same_end() {
		let mut sim = sim_in(9, Vessel::Ape, kitchen(), Rarity::Common);
		let mut bot = HuntBot::human(&sim);

		bot.play(&mut sim);

		let log = HuntLog::decode(
			&HuntLog {
				plan: sim.plan().clone(),
				..HuntLog::new(bot.recorded)
			}
			.encode(),
		)
		.unwrap();
		let replayed = HuntSim::replay(9, Vessel::Ape, Rarity::Common.ward(), &borrowed(), &log);

		assert_eq!(
			serde_json::to_value(replayed).unwrap(),
			serde_json::to_value(sim.outcome()).unwrap(),
		);
	}

	#[test]
	fn a_room_with_nothing_in_it_plays_as_it_did_before_cover() {
		let bare = kitchen();
		let plain = HuntPlan {
			reach_decimetres: bare.reach_decimetres.clone(),
			..HuntPlan::new(bare.centre_centidegrees, bare.half_width_centidegrees)
		};
		let empty = HuntPlan {
			reach_decimetres: bare.reach_decimetres.clone(),
			cover_centimetres: vec![0; HuntPlan::SLICES],
			hide_decimetres: vec![0; HuntPlan::SLICES],
			ledges: vec![false; HuntPlan::SLICES],
			..HuntPlan::new(bare.centre_centidegrees, bare.half_width_centidegrees)
		};
		let mut one = sim_in(4, Vessel::Ape, plain, Rarity::Epic);
		let mut other = sim_in(4, Vessel::Ape, empty, Rarity::Epic);

		HuntBot::human(&one).play(&mut one);
		HuntBot::human(&other).play(&mut other);

		assert_eq!(
			serde_json::to_value(other.outcome()).unwrap(),
			serde_json::to_value(one.outcome()).unwrap(),
		);
		assert_eq!(other.tick(), one.tick());
	}
}
