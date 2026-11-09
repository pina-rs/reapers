//! How a husk moves through the room its plan describes.

mod support;

use std::f64::consts::PI;

use reapers_rules::*;
use support::*;

/// One tick of a hunt: the pose before and after it, and the phase it was
/// played in.
struct Step {
	before: HuskPose,
	after: HuskPose,
	phase: HuntPhase,
}

/// Everything a hunt for a vessel did, tick by tick, played by the shared
/// bot inside a plan.
struct Hunt {
	steps: Vec<Step>,
	events: Vec<Vec<HuntEvent>>,
}

impl Hunt {
	fn new(vessel: Vessel) -> Self {
		Self::with(vessel, 3, HuntPlan::OPEN)
	}

	fn seeded(vessel: Vessel, seed: i64) -> Self {
		Self::with(vessel, seed, HuntPlan::OPEN)
	}

	fn with(vessel: Vessel, seed: i64, plan: HuntPlan) -> Self {
		let mut sim = HuntSim::new(
			seed,
			vessel,
			Rarity::Uncommon,
			Rarity::Uncommon.ward(),
			&borrowed(),
			plan,
			Difficulty::Normal,
		)
		.unwrap();
		let mut bot = HuntBot::human(&sim);
		let mut steps = Vec::new();
		let mut events = Vec::new();

		while !sim.is_over() && sim.tick() < MAX_TICKS {
			let before = sim.pose();
			let phase = sim.phase();

			events.push(bot.step_once(&mut sim));
			steps.push(Step {
				before,
				after: sim.pose(),
				phase,
			});
		}

		Self { steps, events }
	}
}

const MAX_TICKS: i64 = HuntLog::MAX_TICKS as i64;
const TICK_RATE: f64 = HuntRules::TICK_RATE as f64;

fn borrowed() -> HuntStats {
	HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap()
}

fn range(pose: &HuskPose) -> f64 {
	(pose.position.x * pose.position.x + pose.position.z * pose.position.z).sqrt()
}

fn creatures() -> impl Iterator<Item = Vessel> {
	Vessel::ALL.into_iter().filter(|vessel| vessel.is_solo())
}

fn log_in(plan: HuntPlan, inputs: Vec<HuntInput>) -> HuntLog {
	HuntLog {
		plan,
		..HuntLog::new(inputs)
	}
}

/// A test for each named vessel, each running `check` on its vessel.
macro_rules! for_vessels {
	($check:ident: $($name:ident => $vessel:ident),+ $(,)?) => {
		$(
			#[test]
			fn $name() {
				$check(Vessel::$vessel);
			}
		)+
	};
}

mod where_a_husk_can_be {
	use super::*;

	fn keeps_its_feet_on_the_floor(vessel: Vessel) {
		assert_eq!(vessel.gait(), Gait::Ground);

		let hunt = Hunt::new(vessel);
		let floor = -HuntRules::EYE_HEIGHT;

		for step in &hunt.steps {
			assert!((step.after.position.y - vessel.half_height() - floor).abs() <= 1e-9);
		}

		assert!(hunt.steps[0].after.pitch() < 0.0, "look down");
	}

	for_vessels!(keeps_its_feet_on_the_floor:
		ape_keeps_its_feet_on_the_floor => Ape,
		penguin_keeps_its_feet_on_the_floor => Penguin,
		llama_keeps_its_feet_on_the_floor => Llama,
		robot_keeps_its_feet_on_the_floor => Robot,
	);

	fn stays_in_its_band_of_air(vessel: Vessel) {
		assert_eq!(vessel.gait(), Gait::Air);

		let hunt = Hunt::new(vessel);

		for step in &hunt.steps {
			let y = step.after.position.y;

			assert!(y >= vessel.low_metres() - 0.06 && y <= vessel.high_metres() + 0.06);
		}
	}

	for_vessels!(stays_in_its_band_of_air:
		bird_stays_in_its_band_of_air => Bird,
	);

	fn keeps_its_distance_until_it_charges(vessel: Vessel) {
		for seed in 1..=6 {
			let hunt = Hunt::seeded(vessel, seed);
			let mut charged = false;

			for (step, events) in hunt.steps.iter().zip(&hunt.events) {
				charged = charged
					|| events
						.iter()
						.any(|event| matches!(event, HuntEvent::ChargeBegan));

				// Once it has come at the reaper it is inside its usual
				// distance until it has run back out, but never nearer than
				// a charge brings it.
				let low = if charged {
					HuntRules::CHARGE_METRES
				} else {
					vessel.near_metres()
				};
				let at = range(&step.after);

				assert!(
					at >= low - 1e-6 && at <= vessel.far_metres() + 1e-6,
					"seed {seed}: {at} m"
				);
			}
		}
	}

	for_vessels!(keeps_its_distance_until_it_charges:
		ape_keeps_its_distance_until_it_charges => Ape,
		bird_keeps_its_distance_until_it_charges => Bird,
		penguin_keeps_its_distance_until_it_charges => Penguin,
		llama_keeps_its_distance_until_it_charges => Llama,
		robot_keeps_its_distance_until_it_charges => Robot,
	);

	fn never_leaves_the_part_of_the_room_it_was_given(vessel: Vessel) {
		let plan = HuntPlan::new(4000, 6000);

		for seed in 1..=6 {
			for step in Hunt::with(vessel, seed, plan.clone()).steps {
				assert!(
					plan.contains(step.after.yaw()),
					"seed {seed} at {}°",
					degrees(step.after.yaw())
				);
			}
		}
	}

	for_vessels!(never_leaves_the_part_of_the_room_it_was_given:
		ape_never_leaves_the_part_of_the_room_it_was_given => Ape,
		bird_never_leaves_the_part_of_the_room_it_was_given => Bird,
		penguin_never_leaves_the_part_of_the_room_it_was_given => Penguin,
		llama_never_leaves_the_part_of_the_room_it_was_given => Llama,
		robot_never_leaves_the_part_of_the_room_it_was_given => Robot,
	);

	#[test]
	fn a_plan_too_narrow_to_hunt_in_is_refused() {
		let narrow = HuntPlan::new(0, HuntPlan::MIN_HALF_WIDTH_CENTIDEGREES - 1);

		assert!(!narrow.is_well_formed());
		assert!(
			HuntSim::new(
				1,
				Vessel::Penguin,
				Rarity::Common,
				280,
				&borrowed(),
				narrow.clone(),
				Difficulty::Normal
			)
			.is_err()
		);
		assert!(
			HuntSim::replay(
				1,
				Vessel::Penguin,
				Rarity::Common,
				280,
				&borrowed(),
				&log_in(narrow, vec![idle()]),
			)
			.is_none(),
			"a record played in such a plan proves nothing"
		);
	}

	#[test]
	fn a_rifts_husk_stands_where_it_is_and_faces_the_circle() {
		let mut sim = HuntSim::rift(9, Vessel::ReliquaryStack, 1e9, &borrowed(), None);
		let start = sim.pose().position;

		for _ in 0..20 * HuntRules::TICK_RATE {
			sim.step(&idle());

			assert_eq!(sim.pose().position, start);
			assert!(sim.pose().faces(reapers_rules::Vec3::ZERO));
		}

		assert!(sim.pose().yaw().abs() < 1e-9);
	}
}

mod how_a_husk_moves {
	use super::*;

	#[test]
	fn it_moves_smoothly_never_more_than_a_step_in_one_tick() {
		for vessel in creatures() {
			let limit = vessel.bolt_speed() / TICK_RATE * 1.6;

			for step in Hunt::new(vessel).steps {
				assert!(
					(step.after.position - step.before.position).length() <= limit,
					"{vessel:?}"
				);
			}
		}
	}

	#[test]
	fn it_turns_its_body_a_little_at_a_time() {
		let limit = radians(HuntRules::TURN_DEGREES_PER_TICK) + 1e-9;

		for vessel in creatures() {
			for step in Hunt::new(vessel).steps {
				assert!(wrap_angle(step.after.heading - step.before.heading).abs() <= limit);
			}
		}
	}

	#[test]
	fn once_it_is_under_way_it_faces_the_way_it_is_going() {
		for vessel in creatures() {
			let hunt = Hunt::new(vessel);
			let mut moving = 0;
			let mut facing = 0;

			for step in &hunt.steps {
				let travel = step.after.position - step.before.position;
				let along = Vec3::new(travel.x, 0.0, travel.z);

				if along.length() < 0.4 * vessel.walk_speed() / TICK_RATE {
					continue;
				}

				moving += 1;

				if along.normalized().dot(step.after.forward()) > 0.5 {
					facing += 1;
				}
			}

			assert!(moving > 60, "{vessel:?} moves");
			assert!(
				f64::from(facing) / f64::from(moving) > 0.7,
				"{vessel:?} mostly faces where it goes"
			);
		}
	}

	fn stalk_ambushes(events: &[HuntEvent]) -> Vec<bool> {
		events
			.iter()
			.filter_map(|event| {
				match event {
					HuntEvent::StalkEnded { ambush } => Some(*ambush),
					_ => None,
				}
			})
			.collect()
	}

	fn relocations(events: &[HuntEvent]) -> usize {
		events
			.iter()
			.filter(|event| matches!(event, HuntEvent::Relocating))
			.count()
	}

	#[test]
	fn a_husk_that_notices_the_reaper_bolts() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			..SimFor::default()
		});
		let mut events = Vec::new();

		// Spinning on the spot is what alarms it.
		while sim.phase() == HuntPhase::Stalk {
			events.extend(sim.step(&HuntInput::measured(Vec3::FORWARD, 4.0, false, 0, false)));
		}

		assert_eq!(stalk_ambushes(&events), [false]);
		assert_eq!(relocations(&events), 1);

		sim.step(&idle());

		assert!((sim.speed() - Vessel::Ape.bolt_speed()).abs() <= 0.4);
	}

	#[test]
	fn a_husk_that_is_ambushed_stays_where_it_is() {
		let mut sim = sim_for(SimFor {
			vessel: Vessel::Ape,
			..SimFor::default()
		});
		let mut events = Vec::new();

		while sim.phase() == HuntPhase::Stalk {
			let input = HuntInput::measured(sim.pose().direction(), 0.0, false, 0, false);

			events.extend(sim.step(&input));
		}

		assert_eq!(stalk_ambushes(&events), [true]);
		assert_eq!(relocations(&events), 0);
	}

	#[test]
	fn drawing_breath_it_stops_and_turns_on_the_reaper() {
		for vessel in creatures() {
			let hunt = Hunt::new(vessel);
			let mut shrieks = 0;

			for (step, events) in hunt.steps.iter().zip(&hunt.events) {
				if step.phase == HuntPhase::Shriek {
					assert!(
						(step.after.position - step.before.position).length() < 0.002,
						"{vessel:?} holds still to shriek"
					);
				}

				if events
					.iter()
					.any(|event| matches!(event, HuntEvent::ShriekBegan))
				{
					shrieks += 1;

					assert!(step.after.faces(reapers_rules::Vec3::ZERO), "{vessel:?}");
				}
			}

			assert!(shrieks > 0, "{vessel:?}");
		}
	}

	#[test]
	fn it_runs_off_again_some_time_after_each_shriek() {
		let hunt = Hunt::seeded(Vessel::Llama, 5);
		let all: Vec<_> = hunt.events.into_iter().flatten().collect();
		let of = |kind: &str| all.iter().filter(|event| event.kind() == kind).count();

		assert!(of("ShriekBegan") > 0);
		assert!(of("Relocating") >= of("HeldFirm"));
	}

	#[test]
	fn a_broken_husk_does_not_move() {
		for vessel in creatures() {
			for step in Hunt::new(vessel).steps {
				if step.phase == HuntPhase::Broken {
					// A husk in the air still bobs where it hangs.
					assert!((step.after.position - step.before.position).length() < 0.002);
				}
			}
		}
	}
}

mod seals_and_the_side_they_are_on {
	use super::*;

	#[test]
	fn a_seal_on_the_far_side_cannot_be_seen() {
		// A penguin three metres ahead on the floor, first facing the reaper
		// and then with its back turned.
		let position = Vec3::new(0.0, 0.45 - HuntRules::EYE_HEIGHT, 3.0);
		let facing = HuskPose::new(Vessel::Penguin, position, PI);
		let turned = HuskPose::new(Vessel::Penguin, position, 0.0);
		let belly = 1;
		let back = 4;

		assert!(facing.faces(reapers_rules::Vec3::ZERO));
		assert!(facing.seal_faces(reapers_rules::Vec3::ZERO, belly));
		assert!(!facing.seal_faces(reapers_rules::Vec3::ZERO, back));
		assert!(!turned.faces(reapers_rules::Vec3::ZERO));
		assert!(!turned.seal_faces(reapers_rules::Vec3::ZERO, belly));
		assert!(turned.seal_faces(reapers_rules::Vec3::ZERO, back));
	}

	#[test]
	fn a_seal_on_its_left_is_on_the_reapers_right_when_it_faces_them() {
		let pose = HuskPose::new(Vessel::Penguin, Vec3::new(0.0, 0.0, 3.0), PI);
		let left_flipper = 2;

		assert!(Vessel::Penguin.seals()[left_flipper].offset.x > 0.0);
		assert!(pose.seal_position(left_flipper).x > 0.0);
	}

	#[test]
	fn whichever_way_a_husk_turns_at_least_two_seals_are_in_view() {
		for vessel in creatures() {
			for turn in 0..36 {
				for range in [vessel.near_metres(), vessel.far_metres()] {
					let height = if vessel.gait() == Gait::Ground {
						vessel.half_height() - HuntRules::EYE_HEIGHT
					} else {
						f64::midpoint(vessel.low_metres(), vessel.high_metres())
					};
					let pose = HuskPose::new(
						vessel,
						Vec3::new(0.0, height, range),
						radians(f64::from(turn * 10)),
					);
					let seen = (0..vessel.seals().len())
						.filter(|&seal| pose.seal_faces(reapers_rules::Vec3::ZERO, seal))
						.count();

					assert!(seen >= 2, "{vessel:?} turned {}° at {range} m", turn * 10);
				}
			}
		}
	}
}

mod plans {
	use super::*;

	#[test]
	fn an_arc_is_described_by_where_it_starts_and_ends() {
		let plan = HuntPlan::between(radians(-30.0), radians(90.0));

		assert_eq!(plan.centre_centidegrees, 3000);
		assert_eq!(plan.half_width_centidegrees, 6000);
		assert!(plan.contains(radians(0.0)));
		assert!(plan.contains(radians(89.0)));
		assert!(!plan.contains(radians(-31.0)));
		assert!(!plan.contains(radians(180.0)));
	}

	#[test]
	fn an_arc_can_run_across_straight_behind() {
		let plan = HuntPlan::between(radians(120.0), radians(-120.0));

		assert_eq!(plan.half_width_centidegrees, 6000);
		assert!(plan.contains(radians(180.0)));
		assert!(plan.contains(radians(-150.0)));
		assert!(!plan.contains(radians(0.0)));
	}

	#[test]
	fn a_bearing_outside_is_brought_to_the_nearest_edge() {
		let plan = HuntPlan::between(radians(-30.0), radians(90.0));
		let clamped = |bearing: f64| degrees(plan.clamp(radians(bearing)));

		assert!((clamped(100.0) - 90.0).abs() <= 1e-6);
		assert!((clamped(-100.0) - -30.0).abs() <= 1e-6);
		// Straight behind is nearer the far edge, the long way round.
		assert!((clamped(-170.0) - 90.0).abs() <= 1e-6);
		assert!((clamped(10.0) - 10.0).abs() <= 1e-6);
	}

	#[test]
	fn the_whole_circle_holds_everything() {
		assert!(HuntPlan::OPEN.is_well_formed());

		for turn in (-180..=180).step_by(15) {
			assert!(HuntPlan::OPEN.contains(radians(f64::from(turn))));
		}
	}

	#[test]
	fn a_plan_travels_with_the_record_of_its_hunt() {
		let plan = HuntPlan::new(-7250, 9100);
		let log = HuntLog::decode(&log_in(plan.clone(), vec![idle(), idle()]).encode()).unwrap();

		assert_eq!(log.plan, plan);
		assert_eq!(log.inputs.len(), 2);
	}

	#[test]
	#[allow(clippy::float_cmp, reason = "the plan must travel exactly")]
	fn a_surveyed_plan_travels_whole() {
		let plan = HuntPlan {
			eye_centimetres: 97,
			reach_decimetres: (0..HuntPlan::SLICES)
				.map(|slice| 20 + slice as i32)
				.collect(),
			..HuntPlan::new(1250, 6000)
		};
		let log = HuntLog::decode(&log_in(plan.clone(), vec![idle()]).encode()).unwrap();

		assert_eq!(log.plan, plan);
		assert_eq!(log.plan.eye_height(), 0.97);
		assert_eq!(log.plan.reach_decimetres, plan.reach_decimetres);
		assert_eq!(log.inputs.len(), 1);
	}

	#[test]
	fn a_log_cut_off_inside_its_plan_is_refused() {
		let bytes = log_in(
			HuntPlan {
				reach_decimetres: vec![30; HuntPlan::SLICES],
				..HuntPlan::new(0, 9000)
			},
			Vec::new(),
		)
		.encode();

		assert!(HuntLog::decode(&bytes[..bytes.len() - 10]).is_err());
	}

	#[test]
	fn the_same_hunt_in_a_different_plan_is_a_different_hunt() {
		let replayed = |plan: HuntPlan| {
			let mut sim = HuntSim::open(
				4,
				Vessel::Llama,
				Rarity::Common,
				Rarity::Common.ward(),
				&borrowed(),
			);
			let mut bot = HuntBot::careful(&sim);

			bot.play(&mut sim);

			HuntSim::replay(
				4,
				Vessel::Llama,
				Rarity::Common,
				Rarity::Common.ward(),
				&borrowed(),
				&log_in(plan, bot.recorded),
			)
		};

		let elsewhere = HuntPlan::new(9000, 5000);

		assert_eq!(
			replayed(HuntPlan::OPEN).map(|outcome| outcome.absorbed()),
			Some(true)
		);
		assert!(
			!replayed(elsewhere).is_some_and(|outcome| outcome.absorbed()),
			"taps aimed at where the husk was in one room miss in another"
		);
	}
}

mod a_surveyed_room {
	use super::*;

	/// A room three metres deep ahead, with something standing 1.9 m away
	/// between 20 and 40 degrees to the right.
	fn room_with(eye_centimetres: i32) -> HuntPlan {
		HuntPlan {
			eye_centimetres,
			reach_decimetres: (0..HuntPlan::SLICES)
				.map(|slice| {
					let bearing = degrees(HuntPlan::bearing_of(slice));

					if bearing > 20.0 && bearing < 40.0 {
						19
					} else {
						30
					}
				})
				.collect(),
			..HuntPlan::new(0, 9000)
		}
	}

	fn room() -> HuntPlan {
		room_with(145)
	}

	#[test]
	fn the_way_opens_out_gradually_beside_what_stands_in_it() {
		let plan = room();

		// Over the thing itself, and well away from it.
		assert!((plan.reach(radians(27.5)) - 1.9).abs() <= 1e-9);
		assert!((plan.reach(radians(20.0)) - 1.9).abs() <= 1e-9);
		assert!((plan.reach(radians(-40.0)) - 3.0).abs() <= 1e-9);
		// Ten degrees to the side of it.
		assert!(
			(plan.reach(radians(10.0)) - (1.9 + HuntRules::REACH_EASE * radians(10.0))).abs()
				<= 1e-9
		);
		assert!(HuntPlan::OPEN.reach(0.0).is_infinite());
		assert!(HuntPlan::OPEN.reach(0.0) > 0.0);
	}

	fn never_goes_past_what_stands_in_its_way(vessel: Vessel) {
		let plan = room();

		for seed in 1..=6 {
			let hunt = Hunt::with(vessel, seed, plan.clone());

			for step in &hunt.steps {
				let at = range(&step.after);
				let yaw = step.after.yaw();

				// It keeps half its width clear of what stands in its way,
				// except where that would bring it nearer than it ever
				// comes.
				assert!(
					at <= HuntRules::CLOSEST_METRES.max(plan.reach(yaw) - vessel.half_width())
						+ 1e-6,
					"seed {seed} at {}°",
					degrees(yaw)
				);
				assert!(at >= HuntRules::CLOSEST_METRES - 1e-6);
			}
		}
	}

	for_vessels!(never_goes_past_what_stands_in_its_way:
		ape_never_goes_past_what_stands_in_its_way => Ape,
		bird_never_goes_past_what_stands_in_its_way => Bird,
		penguin_never_goes_past_what_stands_in_its_way => Penguin,
		llama_never_goes_past_what_stands_in_its_way => Llama,
		robot_never_goes_past_what_stands_in_its_way => Robot,
	);

	fn still_moves_smoothly_round_it(vessel: Vessel) {
		let hunt = Hunt::with(vessel, 2, room());
		// Skirting something, it draws in toward the player as well as
		// going round, so it covers up to twice the ground it otherwise
		// would. It never jumps.
		let stride = vessel.bolt_speed() / TICK_RATE * 1.6 * 2.0;

		for step in &hunt.steps {
			assert!((step.after.position - step.before.position).length() < stride);
		}
	}

	for_vessels!(still_moves_smoothly_round_it:
		ape_still_moves_smoothly_round_it => Ape,
		bird_still_moves_smoothly_round_it => Bird,
		penguin_still_moves_smoothly_round_it => Penguin,
		llama_still_moves_smoothly_round_it => Llama,
		robot_still_moves_smoothly_round_it => Robot,
	);

	#[test]
	fn a_walker_stands_on_the_floor_the_phone_measured() {
		let hunt = Hunt::with(Vessel::Penguin, 3, room_with(98));

		for step in &hunt.steps {
			assert!((step.after.position.y - Vessel::Penguin.half_height() - -0.98).abs() <= 1e-9);
		}
	}

	#[test]
	fn in_a_room_with_no_room_the_husk_stands_as_close_as_it_ever_does() {
		let cramped = HuntPlan {
			reach_decimetres: vec![9; HuntPlan::SLICES],
			..HuntPlan::new(0, 9000)
		};
		let mut sim = HuntSim::new(
			1,
			Vessel::Penguin,
			Rarity::Common,
			280,
			&borrowed(),
			cramped,
			Difficulty::Normal,
		)
		.unwrap();

		for _ in 0..600 {
			sim.step(&idle());

			assert!((range(&sim.pose()) - HuntRules::CLOSEST_METRES).abs() <= 1e-9);
		}
	}

	#[test]
	fn a_plan_no_phone_could_have_made_is_refused() {
		let plan = |eye: i32, reach: Vec<i32>| {
			HuntPlan {
				eye_centimetres: eye,
				reach_decimetres: reach,
				..HuntPlan::new(0, 9000)
			}
		};

		assert!(plan(145, Vec::new()).is_well_formed());
		assert!(!plan(20, Vec::new()).is_well_formed());
		assert!(!plan(300, Vec::new()).is_well_formed());
		assert!(!plan(145, vec![30, 30, 30]).is_well_formed());
		assert!(!plan(145, vec![300; HuntPlan::SLICES]).is_well_formed());
	}

	#[test]
	fn a_hunt_in_a_surveyed_room_replays_to_the_same_end() {
		let plan = room_with(120);
		let mut sim = HuntSim::new(
			9,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&borrowed(),
			plan.clone(),
			Difficulty::Normal,
		)
		.unwrap();
		let mut bot = HuntBot::careful(&sim);

		bot.play(&mut sim);

		let log = HuntLog::decode(&log_in(plan, bot.recorded).encode()).unwrap();
		let replayed = HuntSim::replay(
			9,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&borrowed(),
			&log,
		);
		let outcome = sim.outcome().unwrap();

		assert_eq!(
			replayed.map(|replayed| replayed.absorbed()),
			Some(outcome.absorbed())
		);
		assert_eq!(
			replayed.map(|replayed| replayed.grade()),
			Some(outcome.grade())
		);
		assert_eq!(replayed.map(|replayed| replayed.ticks), Some(outcome.ticks));
	}
}
