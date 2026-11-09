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
