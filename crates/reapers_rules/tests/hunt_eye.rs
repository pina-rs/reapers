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
fn logs_from_the_origin_keep_their_version_seven_and_eight_bytes() {
	for (roomed, version, expected) in [
		(false, 7, 0x1f83_52a2_a8ee_121e),
		(true, 8, 0x120e_d4e0_421d_bb5c),
	] {
		let bytes = played(roomed).encode();
		assert_eq!(bytes[0], version);
		assert_eq!(digest(&bytes), expected);
	}
}

#[test]
fn version_nine_round_trips_with_and_without_a_room() {
	use reapers_rules::Aim;
	use reapers_rules::Eye;
	use reapers_rules::HuntInput;
	for roomed in [false, true] {
		let mut log = played(roomed);
		log.inputs[0] = HuntInput::new(Aim::AHEAD, 0).with_eye(Eye::new(400, -100, 200));
		let bytes = log.encode();
		assert_eq!(bytes[0], 9);
		if !roomed {
			assert_eq!(&bytes[9..13], &[0; 4]);
		}
		assert_eq!(HuntLog::decode(&bytes), Ok(log));
	}
}

#[test]
fn version_nine_refuses_truncation_bad_room_lengths_and_unknown_versions() {
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
	bad[0] = 10;
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
fn version_nine_tick_has_the_specified_wire_bytes() {
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
