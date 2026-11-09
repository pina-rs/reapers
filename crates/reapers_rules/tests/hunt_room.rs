//! Husks that walk the room's grid: across a desk, up a chair, down on
//! their arc, never nearer than they keep, and the same on every replay.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use std::sync::OnceLock;

use reapers_rules::Difficulty;
use reapers_rules::Footing;
use reapers_rules::HuntBot;
use reapers_rules::HuntEvent;
use reapers_rules::HuntInput;
use reapers_rules::HuntLog;
use reapers_rules::HuntLogError;
use reapers_rules::HuntOutcome;
use reapers_rules::HuntPhase;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::HuskMotionState;
use reapers_rules::HuskPose;
use reapers_rules::LeapRules;
use reapers_rules::Rarity;
use reapers_rules::RoomGrid;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use support::SimFor;
use support::idle;
use support::room;
use support::room::heights;

/// The arc ahead a phone reads a room in.
fn room_plan(room: RoomGrid) -> HuntPlan {
	HuntPlan {
		room: Some(room),
		..HuntPlan::new(0, 6000)
	}
}

fn room_sim(vessel: Vessel, seed: i64, room: RoomGrid) -> HuntSim {
	let options = SimFor {
		seed,
		vessel,
		..SimFor::default()
	};

	HuntSim::new(
		seed,
		vessel,
		Rarity::Common,
		Rarity::Common.ward(),
		&options.stats(),
		room_plan(room),
		Difficulty::Normal,
	)
	.unwrap()
}

/// What a walking husk was doing on one tick.
#[derive(Clone, Debug)]
struct Frame {
	phase: HuntPhase,
	state: HuskMotionState,
	support: f64,
	position: Vec3,
	stunned: bool,
	events: Vec<HuntEvent>,
}

impl Frame {
	fn across(&self) -> f64 {
		(self.position.x * self.position.x + self.position.z * self.position.z).sqrt()
	}
}

/// Steps `sim` for up to `ticks`, playing `input`, and keeps every frame.
fn trace(sim: &mut HuntSim, ticks: i64, input: impl Fn(&HuntSim) -> HuntInput) -> Vec<Frame> {
	let mut frames = Vec::new();

	for _ in 0..ticks {
		if sim.is_over() {
			break;
		}

		let events = sim.step(&input(sim));
		let pose = sim.pose();
		let locomotion = pose.locomotion.expect("a walker on the grid");

		frames.push(Frame {
			phase: sim.phase(),
			state: locomotion.state,
			support: locomotion.support_height,
			position: pose.position,
			stunned: sim.stunned(),
			events,
		});
	}

	frames
}

/// Swings the phone through every charge, to turn it aside.
fn parrying(sim: &HuntSim) -> HuntInput {
	if sim.phase() == HuntPhase::Charge && sim.charge_progress() > 0.3 {
		HuntInput::measured(Vec3::new(0.0, 0.0, 1.0), 4.0, false, 0, false)
	} else {
		idle()
	}
}

fn rooms() -> Vec<RoomGrid> {
	vec![
		room::desk_room(),
		room::desk_and_chair_room(),
		room::sofa_room(),
		room::counter_room(),
		room::furnished_room(),
	]
}

/// Many hunts in many rooms, for every husk that walks, played once and
/// shared by the tests that read them.
fn every_trace() -> &'static [(Vessel, i64, Vec<Frame>)] {
	static TRACES: OnceLock<Vec<(Vessel, i64, Vec<Frame>)>> = OnceLock::new();

	TRACES.get_or_init(play_every_trace)
}

fn play_every_trace() -> Vec<(Vessel, i64, Vec<Frame>)> {
	let mut traces = Vec::new();

	for vessel in [Vessel::Ape, Vessel::Penguin, Vessel::Llama, Vessel::Robot] {
		for (index, room) in rooms().into_iter().enumerate() {
			for seed in 0..4 {
				let seed = seed + 10 * index as i64;
				let mut sim = room_sim(vessel, seed, room.clone());

				traces.push((vessel, seed, trace(&mut sim, 4000, |_| idle())));

				let mut sim = room_sim(vessel, seed, room.clone());

				traces.push((vessel, seed, trace(&mut sim, 4000, parrying)));
			}
		}
	}

	traces
}

fn height(centimetres: i64) -> f64 {
	room::height_of(centimetres)
}

fn near(one: f64, other: f64) -> bool {
	(one - other).abs() < 1e-9
}

/// The frames of a leap from `from` down or up to `to`: the take-off at
/// the edge to the end of the recovery, with the frames before it.
fn find_leap(frames: &[Frame], from: f64, to: f64) -> Option<usize> {
	frames
		.windows(2)
		.position(|pair| near(pair[0].support, from) && near(pair[1].support, to))
		.map(|index| index + 1)
}

#[test]
fn an_ape_walks_across_the_desk_and_leaps_down_on_its_arc() {
	let desk = height(heights::DESK);
	let floor = height(heights::FLOOR);
	let (frames, landed) = (0..64)
		.find_map(|seed| {
			let frames = trace(
				&mut room_sim(Vessel::Ape, seed, room::desk_room()),
				6000,
				|_| idle(),
			);
			let landed = find_leap(&frames, desk, floor)?;

			// It walked the desk to the edge it leaps from.
			let walked = frames[..landed]
				.iter()
				.rev()
				.skip_while(|frame| frame.state != HuskMotionState::Edge)
				.skip_while(|frame| frame.state == HuskMotionState::Edge)
				.take_while(|frame| {
					matches!(
						frame.state,
						HuskMotionState::Walk | HuskMotionState::Run | HuskMotionState::Turn
					) && near(frame.support, desk)
				})
				.count();

			(walked >= 10).then_some((frames, landed))
		})
		.expect("an ape that walks a desk and leaps off it");

	// Edge, crouch, air, land and recover, in that order and nothing else.
	let take_off = frames[..landed]
		.iter()
		.rposition(|frame| frame.state == HuskMotionState::Edge)
		.unwrap();
	let mut order: Vec<HuskMotionState> = Vec::new();

	for frame in &frames[take_off..] {
		if order.last() != Some(&frame.state) {
			order.push(frame.state);
		}

		if frame.state == HuskMotionState::Recover {
			break;
		}
	}

	assert_eq!(
		order,
		[
			HuskMotionState::Edge,
			HuskMotionState::Crouch,
			HuskMotionState::Air,
			HuskMotionState::Land,
			HuskMotionState::Recover,
		]
	);

	// It said where it was leaping as it gathered itself.
	let crouch = take_off
		+ frames[take_off..]
			.iter()
			.position(|frame| frame.state == HuskMotionState::Crouch)
			.unwrap();

	assert!(
		frames[crouch]
			.events
			.contains(&HuntEvent::Leapt(Footing::Open))
	);

	// In the air, the body is on the leap's parabola: level across the
	// floor, falling by gravity.
	let air: Vec<Vec3> = frames[crouch..]
		.iter()
		.skip_while(|frame| frame.state != HuskMotionState::Air)
		.take_while(|frame| frame.state == HuskMotionState::Air)
		.map(|frame| frame.position)
		.collect();
	let fall = LeapRules::GRAVITY / (HuntRules::TICK_RATE * HuntRules::TICK_RATE) as f64;

	assert!(air.len() >= 8, "a leap down a desk takes some time");

	for three in air.windows(3) {
		let across = (three[2] - three[1]) - (three[1] - three[0]);

		assert!(across.x.abs() < 1e-9 && across.z.abs() < 1e-9);
		assert!((across.y + fall).abs() < 1e-9, "{across:?}");
	}

	assert!(near(frames[landed].support, floor));
}

#[test]
fn a_penguin_climbs_to_the_desk_by_the_chair() {
	let floor = height(heights::FLOOR);
	let chair = height(heights::CHAIR);
	let desk = height(heights::DESK);
	let mut climbed = false;

	for seed in 0..32 {
		let frames = trace(
			&mut room_sim(Vessel::Penguin, seed, room::desk_and_chair_room()),
			6000,
			|_| idle(),
		);

		// Never from the floor to the desk, or back, in one bound.
		assert!(find_leap(&frames, floor, desk).is_none());
		assert!(find_leap(&frames, desk, floor).is_none());

		let mut surfaces: Vec<f64> = Vec::new();

		for frame in &frames {
			if surfaces
				.last()
				.is_none_or(|&last| !near(last, frame.support))
			{
				surfaces.push(frame.support);
			}
		}

		if let Some(reached) = surfaces.iter().position(|&surface| near(surface, desk)) {
			assert!(near(surfaces[reached - 1], chair));
			assert!(
				surfaces[..reached]
					.iter()
					.any(|&surface| near(surface, floor))
			);
			climbed = true;
		}
	}

	assert!(climbed, "a penguin that gets up on the desk");
}

#[test]
fn the_surface_underfoot_changes_only_in_a_leap() {
	for (vessel, seed, frames) in every_trace() {
		for pair in frames.windows(2) {
			if near(pair[0].support, pair[1].support) {
				continue;
			}

			assert!(
				matches!(pair[1].state, HuskMotionState::Air | HuskMotionState::Land),
				"{vessel} {seed}: {:?} to {:?}",
				pair[0],
				pair[1],
			);
		}
	}
}

#[test]
fn a_husk_comes_down_to_the_floor_before_it_charges() {
	let floor = height(heights::FLOOR);
	let mut charges = 0;
	let mut raised = 0;

	for (vessel, seed, frames) in every_trace() {
		for frame in frames {
			if frame.support > floor + 0.01 {
				raised += 1;
			}

			if frame.events.contains(&HuntEvent::ChargeBegan) {
				charges += 1;
				assert!(near(frame.support, floor), "{vessel} {seed}: {frame:?}");
			}
		}
	}

	assert!(charges > 0, "some husks charge");
	assert!(raised > 0, "some husks get up on the furniture");
}

#[test]
fn no_husk_comes_nearer_than_it_keeps_but_to_charge() {
	let mut charged = 0;

	for (vessel, seed, frames) in every_trace() {
		let mut charging = false;

		for frame in frames {
			if frame.phase == HuntPhase::Charge {
				charging = true;
				charged += 1;
			}

			// From the moment it charges until it is first back out, it
			// may be nearer: charging, stunned, or backing off.
			if charging
				&& frame.phase != HuntPhase::Charge
				&& frame.across() >= HuntRules::NEAREST_METRES
			{
				charging = false;
			}

			assert!(
				charging || frame.stunned || frame.across() >= HuntRules::NEAREST_METRES - 1e-9,
				"{vessel} {seed}: {:.3} m in {frame:?}",
				frame.across(),
			);
		}
	}

	assert!(charged > 0, "some husks charge");
}

/// A whole hunt in the furnished room, played by the human-paced bot:
/// what it was given, what every tick looked like, and how it ended.
fn played(seed: i64, vessel: Vessel) -> (HuntLog, Vec<HuskPose>, Option<HuntOutcome>) {
	let plan = room_plan(room::furnished_room());
	let mut sim = HuntSim::new(
		seed,
		vessel,
		Rarity::Common,
		Rarity::Common.ward(),
		&stats_for(seed, vessel),
		plan.clone(),
		Difficulty::Normal,
	)
	.unwrap();
	let mut bot = HuntBot::human(&sim);
	let mut inputs = Vec::new();
	let mut poses = Vec::new();

	while !sim.is_over() && inputs.len() < 20_000 {
		let input = bot.decide(&sim);

		sim.step(&input);
		inputs.push(input);
		poses.push(sim.pose());
	}

	let log = HuntLog {
		inputs,
		plan,
		difficulty: Difficulty::Normal,
	};

	(log, poses, sim.outcome())
}

fn stats_for(seed: i64, vessel: Vessel) -> HuntStats {
	SimFor {
		seed,
		vessel,
		..SimFor::default()
	}
	.stats()
}

#[test]
fn a_room_hunt_replays_from_its_log_to_the_last_bit() {
	let (log, poses, outcome) = played(5, Vessel::Ape);
	let bytes = log.encode();

	assert!(outcome.is_some(), "the bot finishes the hunt");
	assert_eq!(bytes[0], 11, "a log with a room uses current combat rules");

	let decoded = HuntLog::decode(&bytes).unwrap();

	assert_eq!(decoded, log);
	assert_eq!(
		HuntSim::replay(
			5,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&stats_for(5, Vessel::Ape),
			&decoded,
		),
		outcome,
	);

	let mut again = HuntSim::new(
		5,
		Vessel::Ape,
		Rarity::Common,
		Rarity::Common.ward(),
		&stats_for(5, Vessel::Ape),
		decoded.plan.clone(),
		decoded.difficulty,
	)
	.unwrap();

	for (input, pose) in decoded.inputs.iter().zip(&poses) {
		again.step(input);
		assert_eq!(&again.pose(), pose);
	}
}

#[test]
fn a_log_without_a_room_is_written_as_it_always_was() {
	let (log, ..) = played(5, Vessel::Ape);
	let without = HuntLog {
		plan: HuntPlan {
			room: None,
			..log.plan.clone()
		},
		..log.clone()
	};
	let bytes = without.encode();
	let with = log.encode();
	let room = log.plan.room.as_ref().unwrap().encode();

	assert_eq!(bytes[0], 10);
	assert_eq!(HuntLog::decode(&bytes).unwrap(), without);

	// The room is the only difference: its length and its bytes, after the
	// plan and before the first tick.
	assert_eq!(with.len(), bytes.len() + 4 + room.len());
	assert_eq!(with[1..9], bytes[1..9]);
	assert_eq!(with[9..13], (room.len() as u32).to_le_bytes());
	assert_eq!(with[13..13 + room.len()], room[..]);
	assert_eq!(with[13 + room.len()..], bytes[9..]);
}

#[test]
fn a_broken_room_is_refused() {
	let (log, ..) = played(5, Vessel::Ape);
	let bytes = log.encode();
	let room = log.plan.room.as_ref().unwrap().encode();

	// Cut off inside the room.
	assert_eq!(
		HuntLog::decode(&bytes[..13 + room.len() / 2]),
		Err(HuntLogError::EndsMidPlan),
	);
	assert_eq!(
		HuntLog::decode(&bytes[..11]),
		Err(HuntLogError::EndsMidPlan)
	);

	// A length that runs past the end.
	let mut long = bytes.clone();

	long[9..13].copy_from_slice(&u32::MAX.to_le_bytes());
	assert_eq!(HuntLog::decode(&long), Err(HuntLogError::EndsMidPlan));

	// A grid no phone writes.
	let mut garbled = bytes.clone();

	garbled[13] = 0xFF;
	assert_eq!(HuntLog::decode(&garbled), Err(HuntLogError::BrokenRoom));

	// A room whose floor is not where the plan says the eye is above.
	let mut disagreeing = log.clone();

	disagreeing.plan.eye_centimetres += 10;
	assert!(!disagreeing.plan.is_well_formed());
	assert_eq!(
		HuntSim::replay(
			5,
			Vessel::Ape,
			Rarity::Common,
			Rarity::Common.ward(),
			&stats_for(5, Vessel::Ape),
			&disagreeing,
		),
		None,
	);
}

/// FNV-1a over what the view is told of every tick.
fn digest(poses: &[HuskPose]) -> u64 {
	let mut hash = 0xcbf2_9ce4_8422_2325_u64;
	let mut eat = |value: u64| {
		for byte in value.to_le_bytes() {
			hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
		}
	};

	for pose in poses {
		let locomotion = pose.locomotion.unwrap();

		eat(pose.position.x.to_bits());
		eat(pose.position.y.to_bits());
		eat(pose.position.z.to_bits());
		eat(pose.heading.to_bits());
		eat(locomotion.support_height.to_bits());
		eat(locomotion.state_fraction.to_bits());
		eat(locomotion.state as u64);
	}

	hash
}

/// The server replays natively and the phone plays as Wasm. A hunt in a
/// room lands on these bits on both: run this test for `wasm32-wasip1`
/// to see it.
#[test]
fn a_room_hunt_lands_on_the_same_bits_everywhere() {
	let (_, poses, _) = played(5, Vessel::Ape);

	assert_eq!(digest(&poses), ROOM_HUNT_DIGEST, "{:#018x}", digest(&poses));
}

const ROOM_HUNT_DIGEST: u64 = 0xed81_5ec4_4900_af2b;

#[test]
fn a_bird_flies_as_it_always_has_in_a_room() {
	for seed in 0..4 {
		let stats = stats_for(seed, Vessel::Bird);
		let start = |plan: HuntPlan| {
			HuntSim::new(
				seed,
				Vessel::Bird,
				Rarity::Common,
				Rarity::Common.ward(),
				&stats,
				plan,
				Difficulty::Normal,
			)
			.unwrap()
		};
		let mut open = start(HuntPlan::new(0, 6000));
		let mut roomed = start(room_plan(room::furnished_room()));

		for _ in 0..3000 {
			assert_eq!(open.step(&idle()), roomed.step(&idle()));
			assert_eq!(open.pose(), roomed.pose());
		}
	}
}
