//! Runs one hunt: feeds the rules what the player does and keeps the record
//! the server will judge. The Flutter `HuntController`.
//!
//! The view calls [`HuntController::frame`] as often as it draws. The rules
//! advance at their own fixed rate, so the same play gives the same hunt
//! on a 30 Hz guest and a 120 Hz screen.

use reapers_rules::HuntBot;
use reapers_rules::HuntEvent;
use reapers_rules::HuntInput;
use reapers_rules::HuntLog;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::RiftSeat;
use reapers_rules::Vec3;
use reapers_world::ArPose;

/// What is fought: a lone hunt, or a seat in a rift, which counts the ward
/// it takes off toward the circle's.
#[derive(Clone, Debug)]
pub enum Fight {
	Hunt(HuntSim),
	Rift(RiftSeat),
}

impl Fight {
	fn sim(&self) -> &HuntSim {
		match self {
			Self::Hunt(sim) => sim,
			Self::Rift(seat) => &seat.sim,
		}
	}

	fn step(&mut self, input: &HuntInput) -> Vec<HuntEvent> {
		match self {
			Self::Hunt(sim) => sim.step(input),
			Self::Rift(seat) => seat.step(input),
		}
	}
}

/// One fight, and its record.
#[derive(Clone, Debug)]
pub struct HuntController {
	fight: Fight,
	/// Every tick's input so far, in order.
	recorded: Vec<HuntInput>,
	pending_taps: Vec<Vec3>,
	/// Whether the player's thumb is on their art, gathering it to throw.
	pub focus: bool,
	unspent: f64,
	/// The simulated player, when the hunt plays itself: it has a person's
	/// limits, needing a moment to react, and it misses.
	bot: Option<HuntBot>,
	bot_turn_speed: f64,
}

impl HuntController {
	/// A stalled frame catches up at most this many ticks, so a hitch never
	/// turns into a burst the player could not have played.
	const MAX_TICKS_PER_FRAME: u32 = 6;
	/// How long one tick of the rules is, in seconds.
	pub const TICK_SECONDS: f64 = 1.0 / HuntRules::TICK_RATE as f64;

	/// The player fights `fight`.
	pub const fn new(fight: Fight) -> Self {
		Self {
			fight,
			recorded: Vec::new(),
			pending_taps: Vec::new(),
			focus: false,
			unspent: 0.0,
			bot: None,
			bot_turn_speed: 0.0,
		}
	}

	/// The rules' own simulated player fights `sim`, with a person's limits
	/// (`REAPERS_AUTOPLAY`): how a hunt is watched on a simulator.
	pub fn autoplay(fight: Fight) -> Self {
		let bot = HuntBot::human(fight.sim());
		Self {
			bot: Some(bot),
			..Self::new(fight)
		}
	}

	/// The fight's rules.
	pub fn sim(&self) -> &HuntSim {
		self.fight.sim()
	}

	/// The rift's seat, when this is one.
	pub const fn seat(&self) -> Option<&RiftSeat> {
		match &self.fight {
			Fight::Rift(seat) => Some(seat),
			Fight::Hunt(_) => None,
		}
	}

	/// The rift's seat, to bring its view of the ward up to date.
	pub const fn seat_mut(&mut self) -> Option<&mut RiftSeat> {
		match &mut self.fight {
			Fight::Rift(seat) => Some(seat),
			Fight::Hunt(_) => None,
		}
	}

	/// Whether a simulated player is fighting.
	pub const fn plays_itself(&self) -> bool {
		self.bot.is_some()
	}

	/// Where the phone points: the player's `live` pose, or where the
	/// simulated player looks.
	pub fn pose(&self, live: ArPose) -> ArPose {
		match &self.bot {
			Some(bot) => ArPose::looking(bot.aim(), self.bot_turn_speed),
			None => live,
		}
	}

	/// Registers a tap that points along `ray`. It is played on the next
	/// tick.
	pub fn tap(&mut self, ray: Vec3) {
		self.pending_taps.push(ray);
	}

	/// The record of this hunt, for the server to replay.
	pub fn log(&self) -> HuntLog {
		HuntLog {
			inputs: self.recorded.clone(),
			plan: self.sim().plan().clone(),
			difficulty: self.sim().difficulty(),
		}
	}

	/// How many ticks have been played.
	pub fn ticks(&self) -> usize {
		self.recorded.len()
	}

	/// Advances the hunt by `delta` seconds of real time, with the phone at
	/// `pose`, and returns what happened.
	pub fn frame(&mut self, delta: f64, pose: &ArPose) -> Vec<HuntEvent> {
		if self.sim().is_over() || !delta.is_finite() || delta <= 0.0 {
			return Vec::new();
		}
		let mut events = Vec::new();
		self.unspent += delta;
		let mut ticks = 0;
		while ticks < Self::MAX_TICKS_PER_FRAME
			&& self.unspent >= Self::TICK_SECONDS
			&& !self.sim().is_over()
		{
			let input = self.read_input(pose);
			self.unspent -= Self::TICK_SECONDS;
			events.extend(self.fight.step(&input));
			self.recorded.push(input);
			ticks += 1;
		}
		// Time the display could not keep up with is dropped, not owed.
		if self.unspent > Self::TICK_SECONDS {
			self.unspent = 0.0;
		}
		events
	}

	/// What the player did this tick: where the phone points, how fast it
	/// is turning, and any taps since the last tick.
	fn read_input(&mut self, pose: &ArPose) -> HuntInput {
		if let Some(bot) = self.bot.as_mut() {
			let input = bot.decide(self.fight.sim());
			self.bot_turn_speed = input.turn_radians_per_second();
			self.pending_taps.clear();
			return input;
		}
		let taps = std::mem::take(&mut self.pending_taps);
		HuntInput::measured(pose.forward, pose.turn_speed, &taps, self.focus)
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntPlan;
	use reapers_rules::HuntStats;
	use reapers_rules::market_listings;

	use super::*;

	fn sim(seed: i64) -> HuntSim {
		let nft = market_listings()[seed as usize % 8];
		let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
		HuntSim::new(
			seed,
			nft.collection.vessel(),
			nft.rarity().ward(),
			&stats,
			HuntPlan::OPEN,
			Difficulty::default(),
		)
		.unwrap()
	}

	#[test]
	fn the_rules_advance_at_their_own_rate_whatever_the_frames() {
		let mut fast = HuntController::new(Fight::Hunt(sim(3)));
		let mut slow = HuntController::new(Fight::Hunt(sim(3)));
		for _ in 0..120 {
			fast.frame(1.0 / 120.0, &ArPose::AHEAD);
		}
		for _ in 0..30 {
			slow.frame(1.0 / 30.0, &ArPose::AHEAD);
		}
		assert_eq!(fast.ticks(), HuntRules::TICK_RATE as usize);
		assert_eq!(slow.ticks(), fast.ticks());
		assert_eq!(fast.log(), slow.log());
	}

	#[test]
	fn a_stalled_frame_catches_up_only_a_little() {
		let mut controller = HuntController::new(Fight::Hunt(sim(4)));
		controller.frame(5.0, &ArPose::AHEAD);
		assert_eq!(controller.ticks(), 6);
		controller.frame(f64::NAN, &ArPose::AHEAD);
		assert_eq!(controller.ticks(), 6);
	}

	#[test]
	fn taps_are_played_once_on_the_next_tick() {
		let mut controller = HuntController::new(Fight::Hunt(sim(5)));
		controller.tap(Vec3::FORWARD);
		controller.frame(HuntController::TICK_SECONDS * 2.0, &ArPose::AHEAD);
		let log = controller.log();
		assert_eq!(log.inputs[0].taps.len(), 1);
		assert_eq!(log.inputs[1].taps.len(), 0);
	}

	/// What the view hands `on_finished` is a log the keeper's replay
	/// accepts, and it comes to the outcome the device saw.
	#[test]
	fn an_autoplayed_hunt_records_a_log_the_server_replays_to_the_same_end() {
		for seed in 0..6 {
			let mut controller = HuntController::autoplay(Fight::Hunt(sim(seed)));
			let mut seconds = 0.0;
			while !controller.sim().is_over() && seconds < 200.0 {
				let pose = controller.pose(ArPose::AHEAD);
				controller.frame(1.0 / 30.0, &pose);
				seconds += 1.0 / 30.0;
			}
			let outcome = controller.sim().outcome().expect("the hunt ends");
			let log = controller.log();
			let bytes = log.encode();
			let read = HuntLog::decode(&bytes).unwrap();
			let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
			let nft = market_listings()[seed as usize % 8];
			let replayed = HuntSim::replay(
				seed,
				nft.collection.vessel(),
				nft.rarity().ward(),
				&stats,
				&read,
			);
			assert_eq!(replayed, Some(outcome), "seed {seed}");
		}
	}
}
