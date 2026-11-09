//! Runs one hunt: feeds the rules what the player does and keeps the record
//! the server will judge. The Flutter `HuntController`.
//!
//! The view calls [`HuntController::frame`] as often as it draws. The rules
//! advance at their own fixed rate, so the same play gives the same hunt
//! on a 30 Hz guest and a 120 Hz screen.

use reapers_rules::Arsenal;
use reapers_rules::HuntBot;
use reapers_rules::HuntEvent;
use reapers_rules::HuntInput;
use reapers_rules::HuntLog;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::RiftSeat;
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
	/// Whether FIRE was pressed since the last tick: one press, one shot.
	fire_pending: bool,
	/// Whether ORB is held down.
	orb_held: bool,
	/// Whether ORB went down since the last tick: a press as quick as a
	/// frame is still held for one tick, so that its release is a throw.
	orb_pressed: bool,
	/// The weapon slot in hand.
	slot: usize,
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
			fire_pending: false,
			orb_held: false,
			orb_pressed: false,
			slot: Arsenal::GUN_SLOT,
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

	/// The slot in hand, from 0 to [`Arsenal::SLOTS`] - 1.
	pub const fn slot(&self) -> usize {
		self.slot
	}

	/// Whether the reaper carries what is in `slot` into this fight. In a
	/// rift it carries only the pistol and its art: the husk stands beyond
	/// any dive, so nothing else could reach it (`docs/gameplay.md`, "The
	/// fight"), and the rail offers nothing that cannot.
	pub const fn carries(&self, slot: usize) -> bool {
		match self.fight {
			Fight::Hunt(_) => slot < Arsenal::SLOTS,
			Fight::Rift(_) => slot == Arsenal::GUN_SLOT || slot == Arsenal::ART_SLOT,
		}
	}

	/// Selects a weapon slot. It is drawn on the next tick. A slot the
	/// reaper does not carry is not selected.
	pub fn select(&mut self, slot: usize) {
		if self.carries(slot) {
			self.slot = slot;
		}
	}

	/// Presses FIRE once. Holding it down does not fire again.
	pub fn press(&mut self) {
		self.fire_pending = true;
	}

	/// Holds ORB down, or lets it go. The tick it comes up is the throw.
	pub fn hold_orb(&mut self, held: bool) {
		self.orb_pressed |= held && !self.orb_held;
		self.orb_held = held;
	}

	/// Whether ORB is down.
	pub const fn orb_held(&self) -> bool {
		self.orb_held
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
	/// `pose`, and returns what happened, each with the rules' tick it
	/// happened on.
	pub fn frame(&mut self, delta: f64, pose: &ArPose) -> Vec<(i64, HuntEvent)> {
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
			let tick = self.sim().tick();
			events.extend(
				self.fight
					.step(&input)
					.into_iter()
					.map(|event| (tick, event)),
			);
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
	/// is turning, whether FIRE was pressed, the slot in hand and whether
	/// ORB is held.
	fn read_input(&mut self, pose: &ArPose) -> HuntInput {
		if let Some(bot) = self.bot.as_mut() {
			let mut input = bot.decide(self.fight.sim());
			self.bot_turn_speed = input.turn_radians_per_second();
			self.fire_pending = false;
			// The rules' player reaches for whatever the moment calls for;
			// it only has what this fight carries.
			if !self.carries(usize::from(input.slot)) {
				input.slot = Arsenal::GUN_SLOT as u8;
			}
			self.slot = usize::from(input.slot);
			return input;
		}
		let fire = std::mem::take(&mut self.fire_pending);
		let orb_held = std::mem::take(&mut self.orb_pressed) || self.orb_held;
		HuntInput::measured(pose.forward, pose.turn_speed, fire, self.slot, orb_held)
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
			nft.rarity(),
			nft.rarity().ward(),
			&stats,
			HuntPlan::OPEN,
			Difficulty::default(),
		)
		.unwrap()
	}

	#[test]
	fn an_orb_pressed_and_let_go_between_two_frames_is_held_for_a_tick() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.hold_orb(true);
		controller.hold_orb(false);
		controller.frame(2.0 * HuntController::TICK_SECONDS, &ArPose::AHEAD);
		let held: Vec<bool> = controller
			.log()
			.inputs
			.iter()
			.map(|input| input.orb_held)
			.collect();
		assert_eq!(held, [true, false], "held for one tick, then let go");
		// Held across frames, it is held on every tick, and once let go it
		// is not held again.
		controller.hold_orb(true);
		controller.frame(2.0 * HuntController::TICK_SECONDS, &ArPose::AHEAD);
		controller.hold_orb(false);
		controller.frame(HuntController::TICK_SECONDS, &ArPose::AHEAD);
		let held: Vec<bool> = controller
			.log()
			.inputs
			.iter()
			.map(|input| input.orb_held)
			.collect();
		assert_eq!(held, [true, false, true, true, false]);
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
	fn a_fire_press_is_played_once_on_the_next_tick() {
		let mut controller = HuntController::new(Fight::Hunt(sim(5)));
		controller.press();
		controller.select(Arsenal::BLADE_SLOT);
		controller.hold_orb(true);
		controller.frame(HuntController::TICK_SECONDS * 2.0, &ArPose::AHEAD);
		let log = controller.log();
		assert!(log.inputs[0].fire);
		assert_eq!(log.inputs[0].slot as usize, Arsenal::BLADE_SLOT);
		assert!(log.inputs[0].orb_held);
		assert!(!log.inputs[1].fire);
		assert!(log.inputs[1].orb_held, "the thumb is still down");
		controller.hold_orb(false);
		controller.frame(HuntController::TICK_SECONDS, &ArPose::AHEAD);
		assert!(!controller.log().inputs[2].orb_held);
	}

	/// A rift is fought with the pistol and the art alone: the player cannot
	/// draw the rest, and nor can a fight that plays itself.
	#[test]
	fn a_rift_carries_only_the_pistol_and_the_art() {
		let seat = || {
			reapers_rules::RiftSeat::new(
				5,
				reapers_rules::RiftSite::LondonBridge,
				1000.0,
				&HuntLoadout::BORROWED,
				None,
			)
			.unwrap()
		};
		let mut controller = HuntController::new(Fight::Rift(seat()));
		controller.select(Arsenal::BLADE_SLOT);
		assert_eq!(controller.slot(), Arsenal::GUN_SLOT);
		controller.select(Arsenal::HEAVY_SLOT);
		assert_eq!(controller.slot(), Arsenal::GUN_SLOT);
		controller.select(Arsenal::ART_SLOT);
		assert_eq!(controller.slot(), Arsenal::ART_SLOT);
		assert!(HuntController::new(Fight::Hunt(sim(1))).carries(Arsenal::BLADE_SLOT));

		let mut autoplay = HuntController::autoplay(Fight::Rift(seat()));
		let mut seconds = 0.0;
		while !autoplay.sim().is_over() && seconds < 60.0 {
			let pose = autoplay.pose(ArPose::AHEAD);
			autoplay.frame(1.0 / 30.0, &pose);
			seconds += 1.0 / 30.0;
		}
		assert!(
			autoplay
				.log()
				.inputs
				.iter()
				.all(|input| autoplay.carries(usize::from(input.slot)))
		);
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
				nft.rarity(),
				nft.rarity().ward(),
				&stats,
				&read,
			);
			assert_eq!(replayed, Some(outcome), "seed {seed}");
		}
	}
}
