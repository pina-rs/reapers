//! Compatibility pins for hunt logs recorded at the origin.

mod support;

use reapers_rules::Difficulty;
use reapers_rules::HuntBot;
use reapers_rules::HuntLog;
use reapers_rules::HuntPlan;
use reapers_rules::HuntSim;
use reapers_rules::Rarity;
use reapers_rules::Vessel;
use support::SimFor;
use support::room;

fn played(roomed: bool) -> HuntLog {
	let seed = 5;
	let vessel = Vessel::Ape;
	let plan = HuntPlan {
		room: roomed.then(room::furnished_room),
		..HuntPlan::new(0, 6000)
	};
	let stats = SimFor {
		seed,
		vessel,
		..SimFor::default()
	}
	.stats();
	let mut sim = HuntSim::new(
		seed,
		vessel,
		Rarity::Common,
		Rarity::Common.ward(),
		&stats,
		plan.clone(),
		Difficulty::Normal,
	)
	.unwrap();
	let mut bot = HuntBot::human(&sim);
	let mut inputs = Vec::new();
	while !sim.is_over() && inputs.len() < 20_000 {
		let input = bot.decide(&sim);
		sim.step(&input);
		inputs.push(input);
	}
	assert!(sim.outcome().is_some());
	HuntLog {
		inputs,
		plan,
		difficulty: Difficulty::Normal,
	}
}

fn digest(bytes: &[u8]) -> u64 {
	bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
		(hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
	})
}

#[test]
fn origin_logs_use_the_current_combat_versions() {
	for (roomed, version, expected) in [
		(false, 10, 0x1f83_52a2_a8ee_121e),
		(true, 11, 0x120e_d4e0_421d_bb5c),
	] {
		let mut bytes = played(roomed).encode();
		assert_eq!(bytes[0], version);
		// The combat header changes deliberately. Keep the older payload
		// pins intact so a new version cannot hide an accidental layout change.
		bytes[0] = if roomed { 8 } else { 7 };
		assert_eq!(digest(&bytes), expected);
	}
}

#[test]
fn version_twelve_round_trips_with_and_without_a_room() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	for roomed in [false, true] {
		let mut log = played(roomed);
		log.inputs[0] = HuntInput::new(Aim::AHEAD, 0).with_eye(Eye::new(400, -100, 200));
		let bytes = log.encode();
		assert_eq!(bytes[0], 12);
		if !roomed {
			assert_eq!(&bytes[9..13], &[0; 4]);
		}
		assert_eq!(HuntLog::decode(&bytes), Ok(log));
	}
}

#[test]
fn version_twelve_refuses_truncation_bad_room_lengths_and_unknown_versions() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLogError;
	let log = HuntLog::new(vec![
		HuntInput::new(Aim::AHEAD, 0).with_eye(Eye::new(1, 0, 0)),
	]);
	let bytes = log.encode();
	for end in 9..13 {
		assert_eq!(
			HuntLog::decode(&bytes[..end]),
			Err(HuntLogError::EndsMidPlan)
		);
	}
	for end in 14..bytes.len() {
		assert_eq!(
			HuntLog::decode(&bytes[..end]),
			Err(HuntLogError::EndsMidTick)
		);
	}
	let mut bad = bytes.clone();
	bad[9..13].copy_from_slice(&u32::MAX.to_le_bytes());
	assert_eq!(HuntLog::decode(&bad), Err(HuntLogError::EndsMidPlan));
	bad[0] = 13;
	assert_eq!(HuntLog::decode(&bad), Err(HuntLogError::UnsupportedVersion));
	let mut origin = bytes;
	let end = origin.len();
	origin[end - 6..].fill(0);
	assert_eq!(HuntLog::decode(&origin).unwrap().inputs[0].eye, Eye::ORIGIN);
}

#[test]
fn eye_bounds_are_enforced_on_decode_and_on_all_replay_inputs() {
	use reapers_rules::Eye;
	use reapers_rules::HuntLogError;
	let log = played(false);
	let stats = SimFor {
		seed: 5,
		vessel: Vessel::Ape,
		..SimFor::default()
	}
	.stats();
	assert!(
		HuntSim::replay(
			5,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&stats,
			&log
		)
		.is_some()
	);
	let height = log.plan.eye_centimetres * 10;
	for eye in [
		Eye::new(1001, 0, 0),
		Eye::new(1000, 0, 1),
		Eye::new(0, (499 - height) as i16, 0),
		Eye::new(0, (2201 - height) as i16, 0),
		Eye::new(i16::MIN, 0, i16::MAX),
	] {
		let mut invalid = log.clone();
		// Even an input after the completed hunt must be validated.
		invalid.inputs.push(invalid.inputs[0].clone().with_eye(eye));
		assert_eq!(
			HuntLog::decode(&invalid.encode()).err(),
			Some(HuntLogError::ImpossibleEye)
		);
		assert_eq!(
			HuntSim::replay(
				5,
				Vessel::Ape,
				Rarity::Common,
				Rarity::Common.ward(),
				&stats,
				&invalid
			),
			None
		);
	}
	for eye in [
		Eye::new(1000, 0, 0),
		Eye::new(600, 0, 800),
		Eye::new(0, (500 - height) as i16, 0),
		Eye::new(0, (2200 - height) as i16, 0),
	] {
		assert!(eye.is_within(&log.plan));
		let mut valid = log.clone();
		valid.inputs[0].eye = eye;
		assert_eq!(HuntLog::decode(&valid.encode()), Ok(valid));
	}
}

#[test]
fn eye_rounding_rejects_nonfinite_and_unrepresentable_offsets() {
	use reapers_rules::Eye;
	use reapers_rules::Vec3;
	assert_eq!(
		Eye::rounded(Vec3::new(0.4004, -0.1006, 0.0)),
		Some(Eye::new(400, -101, 0))
	);
	for value in [
		f64::NAN,
		f64::INFINITY,
		f64::NEG_INFINITY,
		32.768,
		-32.769,
		1e300,
	] {
		assert_eq!(Eye::rounded(Vec3::new(value, 0.0, 0.0)), None);
	}
	assert_eq!(
		Eye::rounded(Vec3::new(32.767, -32.768, 0.0)),
		Some(Eye::new(i16::MAX, i16::MIN, 0))
	);
}

#[test]
fn longest_encoded_log_fits_the_rpc_request_limit() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	use reapers_rules::RoomCell;
	use reapers_rules::RoomGrid;
	for side in [100_u8, 255] {
		let cells = (0..usize::from(side).pow(2))
			.map(|i| {
				if i % 2 == 0 {
					RoomCell::UNKNOWN.byte()
				} else {
					RoomCell::WALL.byte()
				}
			})
			.collect();
		let room = RoomGrid::new(
			160,
			0,
			0,
			side,
			side,
			side / 2,
			side / 2,
			(0..31).collect(),
			cells,
		);
		assert!(room.is_well_formed());
		let plan = HuntPlan {
			eye_centimetres: 160,
			reach_decimetres: vec![60; HuntPlan::SLICES],
			cover_centimetres: vec![0; HuntPlan::SLICES],
			hide_decimetres: vec![0; HuntPlan::SLICES],
			ledges: vec![false; HuntPlan::SLICES],
			room: Some(room),
			..HuntPlan::OPEN
		};
		let log = HuntLog {
			plan,
			inputs: vec![
				HuntInput::new(Aim::AHEAD, 0).with_eye(Eye::new(1, 0, 0));
				HuntLog::MAX_TICKS
			],
			difficulty: Difficulty::Normal,
		};
		let bytes = log.encode();
		assert_eq!(HuntLog::decode(&bytes), Ok(log));
		// Standard base64 has exactly four JSON-safe characters per three bytes.
		let base64_len = bytes.len().div_ceil(3) * 4;
		let json =
			serde_json::json!({"sighting": i64::MAX, "log": "A".repeat(base64_len)}).to_string();
		// leptoskit_rpc's HTTP BODY_LIMIT at the pinned revision is 2 MiB.
		assert!(json.len() + 1024 < 2 * 1024 * 1024);
		println!(
			"{side}x{side}: {} bytes, {base64_len} base64 characters, {} JSON bytes",
			bytes.len(),
			json.len()
		);
	}
}

#[test]
fn version_twelve_tick_has_the_specified_wire_bytes() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	let mut input = HuntInput::new(Aim::new(-1234, 2345), 4567).with_eye(Eye::new(400, -123, 789));
	input.fire = true;
	input.orb_held = true;
	input.slot = 2;
	let bytes = HuntLog::new(vec![input.clone()]).encode();
	assert_eq!(
		&bytes[13..],
		&[
			0x2e, 0xfb, 0x29, 0x09, 0xd7, 0x11, 3, 2, 0x90, 1, 0x85, 0xff, 0x15, 3
		]
	);
	assert_eq!(HuntLog::decode(&bytes).unwrap().inputs, vec![input]);
}

#[test]
fn all_constructors_default_to_origin_and_tick_limit_is_unchanged() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLogError;
	use reapers_rules::Vec3;
	let inputs = [
		HuntInput::default(),
		HuntInput::new(Aim::AHEAD, 0),
		HuntInput::measured(Vec3::new(0.0, 0.0, 1.0), 0.0, true, 0, true),
		HuntInput::still(Vec3::new(0.0, 0.0, 1.0), 0.0),
	];
	assert!(inputs.iter().all(|input| input.eye == Eye::ORIGIN));
	let log = HuntLog::new(vec![
		inputs[0].clone().with_eye(Eye::new(1, 0, 0));
		HuntLog::MAX_TICKS + 1
	]);
	assert_eq!(HuntLog::decode(&log.encode()), Err(HuntLogError::TooLong));
}

#[test]
fn a_grid_can_hide_a_husk_from_one_eye_and_reveal_it_from_the_side() {
	use reapers_rules::Vec3;
	let mut drawn = room::DrawnRoom::sized(100, 100, &[0]);
	drawn.obstacle(44, 58, 8, 6, 96);
	let grid = drawn.build();
	let target = Vec3::new(0.0, -1.15, 4.2);
	let side = Vec3::new(0.9, 0.0, 0.0);
	assert!(grid.hides_from(Vec3::ZERO, target));
	assert!(!grid.hides_from(side, target - side));
	assert!(grid.hides_from(-side, target + side));
	for x in -10..=10 {
		for z in -10..=10 {
			let ray = Vec3::new(f64::from(x) / 10.0, -0.2, f64::from(z) / 10.0);
			assert_eq!(grid.hides_from(Vec3::ZERO, ray), grid.hides(ray));
			assert_eq!(
				grid.surface_along_from(Vec3::ZERO, ray, 5.0).to_bits(),
				grid.surface_along(ray, 5.0).to_bits()
			);
		}
	}
}

#[test]
fn a_plan_wall_is_seen_from_either_side_and_origin_queries_keep_their_bits() {
	use reapers_rules::Vec3;
	let mut plan = HuntPlan {
		reach_decimetres: vec![30; HuntPlan::SLICES],
		cover_centimetres: vec![100; HuntPlan::SLICES],
		hide_decimetres: vec![0; HuntPlan::SLICES],
		ledges: vec![false; HuntPlan::SLICES],
		..HuntPlan::OPEN
	};
	let near = Vec3::new(0.5, -1.0, 1.0);
	let far = Vec3::new(0.5, -1.0, 4.0);
	assert!(plan.hides_from(near, far - near));
	assert!(plan.hides_from(far, near - far));
	assert!(!plan.hides_from(near, Vec3::new(0.0, 0.0, 0.5)));
	for x in -20..=20 {
		let p = Vec3::new(f64::from(x) / 10.0, -0.8, -0.0);
		assert_eq!(plan.hides_from(Vec3::ZERO, p), plan.hides(p));
	}
	plan.cover_centimetres.fill(0);
	assert!(!plan.hides_from(near, far - near));
}

#[test]
fn a_seal_shot_uses_the_eye_on_the_firing_tick() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntPhase;
	use reapers_rules::Vec3;
	let mut sim = SimFor::default().sim();
	support::run_until(&mut sim, HuntPhase::Fight, Some(support::steady_on_husk));
	let eye = Eye::new(900, 0, 0);
	let mut found = false;
	for _ in 0..300 {
		let idle = HuntInput::new(Aim::AHEAD, 0).with_eye(eye);
		sim.step(&idle);
		if let Some(index) = (0..sim.vessel().seals().len()).find(|&i| sim.seal_open(i)) {
			let ray = sim.pose().seal_direction(eye.point(), index);
			let rounded = Aim::from_direction(ray).direction();
			if !sim.on_open_seal(rounded) {
				continue;
			}
			let mut origin = sim.clone();
			let mut moved = sim.clone();
			let shot = HuntInput::measured(ray, 0.0, true, 0, false);
			let from_eye = moved.step(&shot.clone().with_eye(eye));
			let from_origin = origin.step(&shot.with_eye(Eye::ORIGIN));
			if from_eye
				.iter()
				.any(|e| matches!(e, HuntEvent::HitLanded { seal: Some(i), .. } if *i == index))
				&& !from_origin
					.iter()
					.any(|e| matches!(e, HuntEvent::HitLanded { seal: Some(i), .. } if *i == index))
			{
				assert!(sim.in_reach(rounded));
				found = true;
				break;
			}
		}
	}
	assert!(
		found,
		"a moved-eye seal hit must differ from the same origin shot"
	);
	assert_ne!(eye.point(), Vec3::ZERO);
}

#[test]
fn orb_preview_throw_and_missed_landing_use_the_recorded_eye() {
	use reapers_rules::Aim;
	use reapers_rules::CaptureGrade;
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntPhase;
	use reapers_rules::Vec3;
	let mut sim = SimFor::default().sim();
	support::run_until(&mut sim, HuntPhase::Fight, Some(support::steady_on_husk));
	sim.set_ward(0.0);
	sim.step(&HuntInput::new(Aim::AHEAD, 0));
	assert_eq!(sim.phase(), HuntPhase::Broken);
	sim.set_soul(100.0);
	let eye = Eye::new(800, 0, 0);
	for _ in 0..50 {
		let ray = sim.pose().direction_from(eye.point());
		sim.step(&HuntInput::measured(ray, 0.0, false, 0, true).with_eye(eye));
	}
	let aim = Aim::from_direction(sim.pose().direction_from(eye.point())).direction();
	let preview = sim.orb_grade(aim);
	assert_eq!(preview, CaptureGrade::Excellent);
	let mut missed = sim.clone();
	let events = sim.step(&HuntInput::measured(aim, 0.0, false, 0, false).with_eye(eye));
	assert!(events.iter().any(
		|event| matches!(event, HuntEvent::OrbThrown { grade, on_target: true } if *grade == preview)
	));
	let down = Vec3::new(0.0, -1.0, 0.0);
	let events = missed.step(&HuntInput::measured(down, 0.0, false, 0, false).with_eye(eye));
	assert!(events.iter().any(|event| {
		matches!(
			event,
			HuntEvent::OrbThrown {
				on_target: false,
				..
			}
		)
	}));
	let target = missed.orb().unwrap().target;
	assert!((target.x - 0.8).abs() < 1e-9);
	assert!((target.y + missed.plan().eye_height()).abs() < 1e-9);
}

#[test]
fn arts_leave_the_hand_below_the_recorded_eye() {
	use reapers_rules::Arsenal;
	use reapers_rules::Art;
	use reapers_rules::ArtRules;
	use reapers_rules::ArtSkill;
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntStats;
	use reapers_rules::ReaperClass;
	use reapers_rules::Temper;
	use reapers_rules::Vec3;
	use reapers_rules::Weapon;
	for art in [Art::Cinder, Art::Rime, Art::Gale] {
		let options = SimFor::default();
		let stats = HuntStats::derive(
			&HuntLoadout {
				art: Some(ArtSkill::new(art, 1).unwrap()),
				..HuntLoadout::new(
					ReaperClass::Harvester,
					6,
					Weapon::new(ReaperClass::Harvester.weapon(), Temper::Shrouded),
				)
			},
			1.0,
		)
		.unwrap();
		let mut sim = HuntSim::new(
			options.seed,
			options.vessel,
			options.rarity,
			options.rarity.ward(),
			&stats,
			HuntPlan::OPEN,
			Difficulty::Normal,
		)
		.unwrap();
		let eye = Eye::new(400, 100, -200);
		for _ in 0..40 {
			sim.step(
				&HuntInput::measured(
					sim.pose().direction_from(eye.point()),
					0.0,
					false,
					Arsenal::ART_SLOT,
					false,
				)
				.with_eye(eye),
			);
		}
		let events = sim.step(
			&HuntInput::measured(
				sim.pose().direction_from(eye.point()),
				0.0,
				true,
				Arsenal::ART_SLOT,
				false,
			)
			.with_eye(eye),
		);
		assert!(events.contains(&HuntEvent::ArtThrown(art)));
		assert_eq!(
			sim.flights()[0].from,
			eye.point() + Vec3::new(0.0, -ArtRules::HAND_DROP, 0.0)
		);
	}
}

#[test]
fn a_moving_eye_hunt_lands_on_the_same_bits_everywhere() {
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	let plan = HuntPlan {
		room: Some(room::furnished_room()),
		..HuntPlan::new(0, 6000)
	};
	let options = SimFor {
		seed: 5,
		vessel: Vessel::Ape,
		..SimFor::default()
	};
	let mut sim = HuntSim::new(
		5,
		Vessel::Ape,
		Rarity::Common,
		Rarity::Common.ward(),
		&options.stats(),
		plan,
		Difficulty::Normal,
	)
	.unwrap();
	let mut hash = 0xcbf2_9ce4_8422_2325_u64;
	let mut inputs = Vec::new();
	for tick in 0..600 {
		let eye = Eye::new(((tick % 80) as i16 - 40) * 10, 0, 0);
		let input = HuntInput::measured(
			sim.pose().direction_from(eye.point()),
			0.0,
			tick % 20 == 0,
			0,
			false,
		)
		.with_eye(eye);
		let events = sim.step(&input);
		inputs.push(input);
		for byte in sim
			.pose()
			.position
			.x
			.to_bits()
			.to_le_bytes()
			.into_iter()
			.chain(sim.pose().position.y.to_bits().to_le_bytes())
			.chain(sim.pose().position.z.to_bits().to_le_bytes())
			.chain(format!("{events:?}{:?}", sim.outcome()).bytes())
		{
			hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
		}
	}
	let log = HuntLog {
		inputs,
		plan: sim.plan().clone(),
		difficulty: Difficulty::Normal,
	};
	let bytes = log.encode();
	assert_eq!(bytes[0], 12);
	assert_eq!(HuntLog::decode(&bytes), Ok(log));
	assert_eq!(hash, 0x30a9_9835_023d_48ad);
}

#[test]
#[allow(
	clippy::too_many_lines,
	reason = "both weapons exercise the same range cases"
)]
fn melee_reach_and_dive_resources_follow_the_eye() {
	use reapers_rules::Archetype;
	use reapers_rules::Arsenal;
	use reapers_rules::BladeRules;
	use reapers_rules::Eye;
	use reapers_rules::HeavyRules;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::Vec3;
	for (slot, weapon, limit) in [
		(
			Arsenal::BLADE_SLOT,
			Archetype::Blade,
			BladeRules::DIVE_METRES,
		),
		(
			Arsenal::HEAVY_SLOT,
			Archetype::Heavy,
			HeavyRules::DIVE_METRES,
		),
	] {
		let mut saw_close = false;
		let mut saw_far = false;
		for seed in 0..12 {
			let mut sim = SimFor {
				seed,
				vessel: Vessel::Ape,
				..SimFor::default()
			}
			.sim();
			for _ in 0..30 {
				let input = HuntInput::measured(sim.pose().direction(), 0.0, false, slot, false);
				sim.step(&input);
			}
			for _ in 0..2000 {
				if sim.is_over() {
					break;
				}
				let pos = sim.pose().position;
				let horizontal = Vec3::new(pos.x, 0.0, pos.z).normalized();
				for toward in [true, false] {
					let eye = Eye::rounded(horizontal * if toward { 0.95 } else { -0.95 }).unwrap();
					let distance = sim.pose().distance_from(eye.point());
					let origin = sim.pose().distance();
					if !((toward
						&& origin > Arsenal::CLOSE_METRES
						&& distance < Arsenal::CLOSE_METRES - 0.1)
						|| (!toward && origin < limit && distance > limit + 0.1))
					{
						continue;
					}
					let input = HuntInput::measured(
						sim.pose().direction_from(eye.point()),
						0.0,
						true,
						slot,
						false,
					)
					.with_eye(eye);
					let mut candidate = sim.clone();
					candidate.step(
						&HuntInput::measured(
							sim.pose().direction_from(eye.point()),
							0.0,
							false,
							slot,
							false,
						)
						.with_eye(eye),
					);
					let aim = input.aim.direction();
					let preview = candidate.in_reach(aim);
					let charges = candidate.dive_charges();
					let events = candidate.step(&input);
					if toward
						&& origin > Arsenal::CLOSE_METRES
						&& distance < Arsenal::CLOSE_METRES - 0.1
						&& events.iter().any(
							|e| matches!(e, HuntEvent::HitLanded { weapon: w, .. } if *w == weapon),
						) {
						assert!(preview);
						assert_eq!(candidate.dive_charges(), charges);
						assert!(!events.contains(&HuntEvent::DiveStarted(weapon)));
						saw_close = true;
					}
					if !toward && origin < limit && distance > limit + 0.1 {
						assert!(!preview);
						assert!(!events.contains(&HuntEvent::DiveStarted(weapon)));
						assert_eq!(candidate.dive_charges(), charges);
						saw_far = true;
					}
				}
				if saw_close && saw_far {
					break;
				}
				sim.step(&HuntInput::measured(
					sim.pose().direction(),
					0.0,
					false,
					slot,
					false,
				));
			}
			if saw_close && saw_far {
				break;
			}
		}
		assert!(saw_close, "no close {weapon:?} case");
		assert!(saw_far, "no far {weapon:?} case");
	}
}

#[test]
fn a_moving_eye_is_followed_on_each_shot() {
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	let mut sim = SimFor::default().sim();
	for tick in 0..120 {
		let eye = Eye::new(if tick % 40 < 20 { 400 } else { -400 }, 0, 0);
		let ray = sim.pose().direction_from(eye.point());
		let fire = tick % 20 == 0;
		let events = sim.step(&HuntInput::measured(ray, 0.0, fire, 0, false).with_eye(eye));
		assert_eq!(sim.eye(), eye);
		if fire {
			assert!(
				events
					.iter()
					.any(|event| matches!(event, HuntEvent::HitLanded { .. })),
				"tick {tick}: {events:?}"
			);
		}
	}
}

#[test]
fn the_simulation_uses_grid_visibility_for_shots_from_either_side() {
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	let plan = HuntPlan {
		room: Some(room::desk_room()),
		..HuntPlan::new(0, 6000)
	};
	let options = SimFor {
		seed: 3,
		vessel: Vessel::Ape,
		..SimFor::default()
	};
	let sim = HuntSim::new(
		3,
		Vessel::Ape,
		Rarity::Common,
		Rarity::Common.ward(),
		&options.stats(),
		plan,
		Difficulty::Normal,
	)
	.unwrap();
	let side = Eye::new(900, 0, 0);
	let other = Eye::new(-900, 0, 0);
	let mut visible = sim.clone();
	let mut hidden = sim;
	visible.step(
		&HuntInput::measured(
			visible.pose().direction_from(side.point()),
			0.0,
			false,
			0,
			false,
		)
		.with_eye(side),
	);
	hidden.step(
		&HuntInput::measured(
			hidden.pose().direction_from(other.point()),
			0.0,
			false,
			0,
			false,
		)
		.with_eye(other),
	);
	assert!(!visible.husk_hidden());
	assert!(hidden.husk_hidden());
	let clear_ray = visible.pose().direction_from(side.point());
	let blocked_ray = hidden.pose().direction_from(other.point());
	assert!(visible.in_reach(clear_ray));
	assert!(!hidden.in_reach(blocked_ray));
	let clear = visible.step(&HuntInput::measured(clear_ray, 0.0, true, 0, false).with_eye(side));
	let blocked =
		hidden.step(&HuntInput::measured(blocked_ray, 0.0, true, 0, false).with_eye(other));
	assert!(
		clear
			.iter()
			.any(|e| matches!(e, HuntEvent::HitLanded { .. }))
	);
	assert!(blocked.iter().any(|e| matches!(e, HuntEvent::Missed(_))));
	visible.step(&HuntInput::measured(clear_ray, 0.0, false, 0, false).with_eye(side));
	hidden.step(&HuntInput::measured(blocked_ray, 0.0, false, 0, false).with_eye(other));
	assert!(visible.seal_open(0));
	assert!(!hidden.seal_open(0));
}

#[test]
fn a_lean_into_dive_range_starts_the_dive_and_spends_a_blade_charge() {
	use reapers_rules::Archetype;
	use reapers_rules::Arsenal;
	use reapers_rules::BladeRules;
	use reapers_rules::Eye;
	use reapers_rules::HeavyRules;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::Vec3;
	for (slot, weapon, limit) in [
		(
			Arsenal::BLADE_SLOT,
			Archetype::Blade,
			BladeRules::DIVE_METRES,
		),
		(
			Arsenal::HEAVY_SLOT,
			Archetype::Heavy,
			HeavyRules::DIVE_METRES,
		),
	] {
		let mut found = false;
		for seed in 0..100 {
			let mut sim = SimFor {
				seed,
				vessel: Vessel::Ape,
				..SimFor::default()
			}
			.sim();
			for _ in 0..30 {
				sim.step(&HuntInput::measured(
					sim.pose().direction(),
					0.0,
					false,
					slot,
					false,
				));
			}
			let pos = sim.pose().position;
			let eye = Eye::rounded(Vec3::new(pos.x, 0.0, pos.z).normalized() * 0.95).unwrap();
			let origin = sim.pose().distance();
			let moved = sim.pose().distance_from(eye.point());
			if origin <= limit + 0.1 || moved >= limit - 0.1 || moved <= Arsenal::CLOSE_METRES + 0.1
			{
				continue;
			}
			let ray = sim.pose().direction_from(eye.point());
			sim.step(&HuntInput::measured(ray, 0.0, false, slot, false).with_eye(eye));
			let input = HuntInput::measured(
				sim.pose().direction_from(eye.point()),
				0.0,
				true,
				slot,
				false,
			)
			.with_eye(eye);
			assert!(sim.in_reach(input.aim.direction()));
			let charges = sim.dive_charges();
			let events = sim.step(&input);
			if events.contains(&HuntEvent::DiveStarted(weapon)) {
				assert_eq!(
					sim.dive_charges(),
					charges - i64::from(weapon == Archetype::Blade)
				);
				found = true;
				break;
			}
		}
		assert!(found, "a lean starts a {weapon:?} dive");
	}
}

#[test]
fn ice_keeps_the_launch_eye_when_the_caster_moves_during_flight() {
	use reapers_rules::Arsenal;
	use reapers_rules::Art;
	use reapers_rules::ArtSkill;
	use reapers_rules::Eye;
	use reapers_rules::HuntEvent;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntStats;
	use reapers_rules::ReaperClass;
	use reapers_rules::Temper;
	use reapers_rules::Weapon;

	let stats = HuntStats::derive(
		&HuntLoadout {
			art: Some(ArtSkill::new(Art::Rime, 1).unwrap()),
			..HuntLoadout::new(
				ReaperClass::Harvester,
				6,
				Weapon::new(ReaperClass::Harvester.weapon(), Temper::Shrouded),
			)
		},
		1.0,
	)
	.unwrap();
	let plan = HuntPlan {
		room: Some(room::desk_room()),
		..HuntPlan::new(0, 6000)
	};
	for (launch, moved, expected) in [
		(Eye::new(900, 0, 0), Eye::new(-900, 0, 0), true),
		(Eye::new(-900, 0, 0), Eye::new(900, 0, 0), false),
	] {
		let mut sim = HuntSim::new(
			3,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&stats,
			plan.clone(),
			Difficulty::Normal,
		)
		.unwrap();
		let input = |sim: &HuntSim, eye: Eye, fire| {
			HuntInput::measured(
				sim.pose().direction_from(eye.point()),
				0.0,
				fire,
				Arsenal::ART_SLOT,
				false,
			)
			.with_eye(eye)
		};
		for _ in 0..40 {
			let command = input(&sim, launch, false);
			sim.step(&command);
		}
		let command = input(&sim, launch, true);
		assert!(
			sim.step(&command)
				.contains(&HuntEvent::ArtThrown(Art::Rime))
		);
		let mut landed = None;
		for _ in 0..120 {
			let command = input(&sim, moved, false);
			for event in sim.step(&command) {
				if let HuntEvent::ArtLanded {
					art: Art::Rime,
					hit,
					..
				} = event
				{
					landed = Some(hit);
				}
			}
			if landed.is_some() {
				break;
			}
		}
		assert_eq!(
			landed,
			Some(expected),
			"launch: {launch:?}, moved: {moved:?}"
		);
	}
}
