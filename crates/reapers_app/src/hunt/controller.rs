//! Runs one hunt: feeds the rules what the player does and keeps the record
//! the server will judge. The Flutter `HuntController`.
//!
//! The view calls [`HuntController::frame`] as often as it draws. The rules
//! advance at their own fixed rate, so the same play gives the same hunt
//! on a 30 Hz guest and a 120 Hz screen.

use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::Eye;
use reapers_rules::HuntBot;
use reapers_rules::HuntEvent;
use reapers_rules::HuntInput;
use reapers_rules::HuntLog;
use reapers_rules::HuntPhase;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::RiftSeat;
use reapers_rules::Vec3;
use reapers_world::ArPose;

/// The eye supplied by the current tracking source for one frame.
#[derive(Clone, Copy, Debug)]
pub enum FrameEye {
	Origin,
	World(Vec3),
	Unavailable,
}

/// Why the simulation is held, or whether it is playing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseStatus {
	Playing,
	Tracking,
	ReturnToStance,
	ResumeHeldOrb,
}

/// Physical and queued orb input, independent of the rules' held ring.
#[derive(Clone, Copy, Debug, Default)]
struct OrbInput {
	held: bool,
	pressed: bool,
	down: bool,
}

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

/// The most ticks a press of FIRE waits for a busy weapon
/// (`docs/gameplay.md`, "The fight").
pub const FIRE_BUFFER_TICKS: i64 = 8;

/// What a press of FIRE on the coming tick would meet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FireWait {
	/// The weapon takes a press on this tick, or its timing is the player's:
	/// a reload, a dive, an art with nothing left to throw.
	Now,
	/// The weapon in hand takes a press in this many ticks.
	Ticks(i64),
	/// The husk shrieks from this tick: a press now is a flinch.
	Shriek,
	/// Nothing takes a press: the husk is broken, caught or gone.
	Idle,
}

impl FireWait {
	/// What a press on the coming tick of `sim` meets, with `slot` in hand.
	/// It reads the rules and decides nothing for them: a press it lets
	/// through is judged by the rules on that tick.
	fn of(sim: &HuntSim, slot: usize) -> Self {
		let phase = sim.phase();
		// On the tick a warning ends the shriek begins, and a press on it
		// would be one fired into a shriek.
		let shrieks = phase == HuntPhase::Shriek
			|| (phase == HuntPhase::Windup && sim.shriek_progress() >= 1.0 && !sim.leaping());
		if shrieks {
			return Self::Shriek;
		}
		if !matches!(
			phase,
			HuntPhase::Stalk | HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Charge
		) {
			return Self::Idle;
		}
		if sim.diving() {
			return Self::Now;
		}
		if slot != sim.selected_slot() {
			// The press comes with a new weapon, drawn from this tick.
			return Self::Ticks(Arsenal::SWITCH_TICKS);
		}
		let ticks = match Arsenal::archetype_of(slot) {
			Some(Archetype::Gun) if sim.reloading() => 0,
			// An empty drum is reloaded on the press, once the gun is drawn.
			Some(Archetype::Gun) if sim.rounds() == 0 => sim.draw_ticks_left(),
			Some(Archetype::Art) | None if sim.art_charges() == 0 => 0,
			_ => sim.cooldown_ticks(),
		};
		if ticks > 0 {
			Self::Ticks(ticks)
		} else {
			Self::Now
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
	/// A press waiting for the weapon in hand, and the last tick it may be
	/// played on.
	fire_held: Option<i64>,
	orb: OrbInput,
	pause: PauseStatus,
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
			fire_held: None,
			orb: OrbInput {
				held: false,
				pressed: false,
				down: false,
			},
			pause: PauseStatus::Playing,
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

	/// Presses FIRE once. Holding it down does not fire again. A press the
	/// weapon in hand is too busy to take, but will take within
	/// [`FIRE_BUFFER_TICKS`], waits for it and is played on the first tick it
	/// would be taken; one still waiting when the husk begins to shriek is
	/// dropped, never played into it, and so is one waiting when the room is
	/// lost. Every other press is played on the next tick, as it always was,
	/// and the rules judge it there. A new press starts the wait again.
	///
	/// A press that waits is played with the aim of the tick it goes on, not
	/// the one it was made on: the shot goes where the phone points when it
	/// fires, so one made on a seal can miss if the phone swings off it. The
	/// wait is counted in the rules' ticks, not in real time, so across a
	/// frame the hunt drops time on, a press can go a little later than
	/// [`FIRE_BUFFER_TICKS`] of real time after it was made.
	pub fn press(&mut self) {
		if self.pause == PauseStatus::Playing {
			self.fire_pending = true;
		}
	}

	/// Holds ORB down, or lets it go. The tick it comes up is the throw.
	pub fn hold_orb(&mut self, held: bool) {
		let pressed = held && !self.orb.down;
		self.orb.down = held;
		if self.pause == PauseStatus::ResumeHeldOrb && pressed {
			self.pause = PauseStatus::Playing;
			self.orb.held = true;
			self.orb.pressed = true;
			return;
		}
		if self.pause != PauseStatus::Playing {
			return;
		}
		self.orb.pressed |= pressed;
		if pressed || !held {
			self.orb.held = held;
		}
	}

	/// Whether ORB is down.
	pub const fn orb_held(&self) -> bool {
		self.orb.held
	}

	/// Current gameplay pause, for the tracking and recovery guidance.
	pub const fn pause_status(&self) -> PauseStatus {
		self.pause
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
	pub fn frame(&mut self, delta: f64, pose: &ArPose, eye: FrameEye) -> Vec<(i64, HuntEvent)> {
		let eye = if self.bot.is_some() || matches!(self.fight, Fight::Rift(_)) {
			Ok(Eye::ORIGIN)
		} else {
			match eye {
				FrameEye::Origin => Ok(Eye::ORIGIN),
				FrameEye::Unavailable => Err(PauseStatus::Tracking),
				FrameEye::World(point) => {
					match Eye::rounded(point) {
						Some(eye) if eye.is_within(self.sim().plan()) => Ok(eye),
						Some(_) => Err(PauseStatus::ReturnToStance),
						None => Err(PauseStatus::Tracking),
					}
				}
			}
		};
		let eye = match eye {
			Ok(eye) => eye,
			Err(reason) => {
				self.pause = reason;
				self.unspent = 0.0;
				self.fire_pending = false;
				self.fire_held = None;
				self.orb.held = false;
				self.orb.pressed = false;
				return Vec::new();
			}
		};
		if self.pause != PauseStatus::Playing {
			if self.pause != PauseStatus::ResumeHeldOrb {
				self.pause = if self.sim().orb_ring().is_some() {
					PauseStatus::ResumeHeldOrb
				} else {
					PauseStatus::Playing
				};
			}
			self.unspent = 0.0;
			return Vec::new();
		}
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
			let input = self.read_input(pose).with_eye(eye);
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
			self.fire_held = None;
			// The rules' player reaches for whatever the moment calls for;
			// it only has what this fight carries.
			if !self.carries(usize::from(input.slot)) {
				input.slot = Arsenal::GUN_SLOT as u8;
			}
			self.slot = usize::from(input.slot);
			return input;
		}
		let fire = self.fire_this_tick();
		let orb_held = std::mem::take(&mut self.orb.pressed) || self.orb.held;
		HuntInput::measured(pose.forward, pose.turn_speed, fire, self.slot, orb_held)
	}

	/// Whether the coming tick's input presses FIRE: a press made since the
	/// last tick that the weapon in hand takes now, or one that has waited
	/// for it and is taken now. The record holds the tick a press is played
	/// on, not the one it was made on.
	fn fire_this_tick(&mut self) -> bool {
		let tick = self.sim().tick();
		let wait = FireWait::of(self.sim(), self.slot);
		if std::mem::take(&mut self.fire_pending) {
			self.fire_held = None;
			// Played as it always was unless the weapon will take it within
			// the buffer: in a shriek a press is the player's own, and a
			// flinch; one too long to wait for the rules hear now, and let go.
			if let FireWait::Ticks(ticks) = wait
				&& ticks <= FIRE_BUFFER_TICKS
			{
				self.fire_held = Some(tick + FIRE_BUFFER_TICKS);
				return false;
			}
			return true;
		}
		let Some(last) = self.fire_held else {
			return false;
		};
		match wait {
			FireWait::Now => {
				self.fire_held = None;
				true
			}
			FireWait::Ticks(ticks) if tick + ticks <= last => false,
			// Waited too long, or the husk shrieks, or nothing would take it.
			FireWait::Ticks(_) | FireWait::Shriek | FireWait::Idle => {
				self.fire_held = None;
				false
			}
		}
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
	fn unavailable_tracking_discards_time_and_fire() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.press();
		controller.frame(1.0, &ArPose::AHEAD, FrameEye::Unavailable);
		assert_eq!(controller.ticks(), 0);
		controller.press();
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert_eq!(controller.ticks(), 0);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert_eq!(controller.ticks(), 1);
		assert!(!controller.log().inputs[0].fire);
		controller.press();
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert!(controller.log().inputs[1].fire);
	}

	#[test]
	fn measured_eyes_are_recorded_per_tick_and_origin_modes_stay_at_origin() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		for point in [Vec3::new(0.4, 0.0, 0.0), Vec3::new(-0.2, 0.1, 0.3)] {
			controller.frame(
				2.0 * HuntController::TICK_SECONDS,
				&ArPose::AHEAD,
				FrameEye::World(point),
			);
		}
		let eyes: Vec<_> = controller
			.log()
			.inputs
			.iter()
			.map(|input| input.eye)
			.collect();
		assert_eq!(
			eyes,
			[
				Eye::new(400, 0, 0),
				Eye::new(400, 0, 0),
				Eye::new(-200, 100, 300),
				Eye::new(-200, 100, 300)
			]
		);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert_eq!(controller.log().inputs.last().unwrap().eye, Eye::ORIGIN);
		let mut bot = HuntController::autoplay(Fight::Hunt(sim(3)));
		bot.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Unavailable,
		);
		assert_eq!(bot.log().inputs[0].eye, Eye::ORIGIN);
		let seat = RiftSeat::new(
			5,
			reapers_rules::RiftSite::LondonBridge,
			1000.0,
			&HuntLoadout::BORROWED,
			None,
		)
		.unwrap();
		let mut rift = HuntController::new(Fight::Rift(seat));
		rift.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::World(Vec3::new(0.5, 0.0, 0.0)),
		);
		assert_eq!(rift.log().inputs[0].eye, Eye::ORIGIN);
	}

	#[test]
	fn stance_and_tracking_failures_pause_without_time_debt() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		let height = f64::from(controller.sim().plan().eye_centimetres) / 100.0;
		for point in [
			Vec3::new(0.6, 0.0, 0.8),
			Vec3::new(0.0, 0.5 - height, 0.0),
			Vec3::new(0.0, 2.2 - height, 0.0),
		] {
			controller.frame(
				HuntController::TICK_SECONDS,
				&ArPose::AHEAD,
				FrameEye::World(point),
			);
			assert_eq!(controller.pause_status(), PauseStatus::Playing);
		}
		for (point, reason) in [
			(Vec3::new(0.601, 0.0, 0.8), PauseStatus::ReturnToStance),
			(
				Vec3::new(0.0, 0.499 - height, 0.0),
				PauseStatus::ReturnToStance,
			),
			(
				Vec3::new(0.0, 2.201 - height, 0.0),
				PauseStatus::ReturnToStance,
			),
			(Vec3::new(f64::NAN, 0.0, 0.0), PauseStatus::Tracking),
			(Vec3::new(f64::INFINITY, 0.0, 0.0), PauseStatus::Tracking),
		] {
			let ticks = controller.ticks();
			controller.frame(20.0, &ArPose::AHEAD, FrameEye::World(point));
			assert_eq!(controller.pause_status(), reason);
			controller.frame(20.0, &ArPose::AHEAD, FrameEye::Origin);
			assert_eq!(controller.ticks(), ticks);
			controller.frame(
				HuntController::TICK_SECONDS,
				&ArPose::AHEAD,
				FrameEye::Origin,
			);
			assert_eq!(controller.ticks(), ticks + 1);
		}
	}

	#[test]
	fn queued_orb_is_discarded_on_pause() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.hold_orb(true);
		controller.hold_orb(false);
		controller.frame(1.0, &ArPose::AHEAD, FrameEye::Unavailable);
		controller.frame(1.0, &ArPose::AHEAD, FrameEye::Origin);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert!(!controller.log().inputs[0].orb_held);
		assert_eq!(controller.sim().orb_ring(), None);
	}

	#[test]
	fn an_unplayed_orb_held_through_tracking_loss_needs_a_fresh_press() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.hold_orb(true);
		controller.frame(1.0, &ArPose::AHEAD, FrameEye::Unavailable);

		for _ in 0..2 {
			controller.hold_orb(true);
			controller.frame(
				HuntController::TICK_SECONDS,
				&ArPose::AHEAD,
				FrameEye::Origin,
			);
		}

		assert!(!controller.log().inputs[0].orb_held);
		controller.hold_orb(false);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert!(controller.log().inputs.iter().all(|input| !input.orb_held));
		assert_eq!(controller.sim().orb_ring(), None);

		controller.hold_orb(true);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert!(controller.log().inputs.last().unwrap().orb_held);
	}

	#[test]
	fn a_held_orb_needs_a_fresh_press_after_tracking_recovers() {
		let seed = 3;
		let mut initial = sim(seed);
		let mut bot = HuntBot::careful(&initial);
		let mut prefix = Vec::new();
		while initial.phase() != reapers_rules::HuntPhase::Broken {
			assert!(prefix.len() < 10000 && !initial.is_over());
			let input = bot.decide(&initial);
			initial.step(&input);
			prefix.push(input);
		}
		let mut controller = HuntController::new(Fight::Hunt(initial));
		let aim = ArPose::looking(controller.sim().pose().direction(), 0.0);
		controller.hold_orb(true);
		controller.frame(HuntController::TICK_SECONDS, &aim, FrameEye::Origin);
		assert!(controller.sim().orb_ring().is_some());
		controller.frame(2.0, &aim, FrameEye::Unavailable);
		controller.hold_orb(false);
		controller.hold_orb(true);
		controller.press();
		controller.frame(2.0, &aim, FrameEye::Origin);
		assert_eq!(controller.pause_status(), PauseStatus::ResumeHeldOrb);
		controller.hold_orb(true);
		assert_eq!(controller.pause_status(), PauseStatus::ResumeHeldOrb);
		controller.hold_orb(false);
		controller.press();
		assert_eq!(controller.frame(2.0, &aim, FrameEye::Origin), []);
		assert_eq!(controller.ticks(), 1);
		controller.hold_orb(true);
		controller.frame(HuntController::TICK_SECONDS, &aim, FrameEye::Origin);
		assert_eq!(controller.pause_status(), PauseStatus::Playing);
		assert!(!controller.log().inputs[1].fire);
		controller.hold_orb(false);
		let events = controller.frame(HuntController::TICK_SECONDS, &aim, FrameEye::Origin);
		assert!(
			events
				.iter()
				.any(|(_, event)| matches!(event, HuntEvent::OrbThrown { .. }))
		);
		let mut log = controller.log();
		prefix.append(&mut log.inputs);
		let mut finished = controller.sim().clone();
		let mut bot = HuntBot::careful(&finished);
		while !finished.is_over() {
			assert!(prefix.len() < 21600);
			let input = bot.decide(&finished);
			finished.step(&input);
			prefix.push(input);
		}
		log.inputs = prefix;
		let nft = market_listings()[seed as usize % 8];
		let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
		assert_eq!(
			HuntSim::replay(
				seed,
				nft.collection.vessel(),
				nft.rarity(),
				nft.rarity().ward(),
				&stats,
				&log
			),
			finished.outcome()
		);
	}

	#[test]
	fn an_orb_pressed_and_let_go_between_two_frames_is_held_for_a_tick() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.hold_orb(true);
		controller.hold_orb(false);
		controller.frame(
			2.0 * HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
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
		controller.frame(
			2.0 * HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		controller.hold_orb(false);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
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
			fast.frame(1.0 / 120.0, &ArPose::AHEAD, FrameEye::Origin);
		}
		for _ in 0..30 {
			slow.frame(1.0 / 30.0, &ArPose::AHEAD, FrameEye::Origin);
		}
		assert_eq!(fast.ticks(), HuntRules::TICK_RATE as usize);
		assert_eq!(slow.ticks(), fast.ticks());
		assert_eq!(fast.log(), slow.log());
	}

	#[test]
	fn a_stalled_frame_catches_up_only_a_little() {
		let mut controller = HuntController::new(Fight::Hunt(sim(4)));
		controller.frame(5.0, &ArPose::AHEAD, FrameEye::Origin);
		assert_eq!(controller.ticks(), 6);
		controller.frame(f64::NAN, &ArPose::AHEAD, FrameEye::Origin);
		assert_eq!(controller.ticks(), 6);
	}

	#[test]
	fn a_fire_press_is_played_once_on_the_next_tick() {
		let mut controller = HuntController::new(Fight::Hunt(sim(5)));
		controller.press();
		controller.select(Arsenal::BLADE_SLOT);
		controller.hold_orb(true);
		controller.frame(
			HuntController::TICK_SECONDS * 2.0,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		let log = controller.log();
		assert!(log.inputs[0].fire);
		assert_eq!(log.inputs[0].slot as usize, Arsenal::BLADE_SLOT);
		assert!(log.inputs[0].orb_held);
		assert!(!log.inputs[1].fire);
		assert!(log.inputs[1].orb_held, "the thumb is still down");
		controller.hold_orb(false);
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		);
		assert!(!controller.log().inputs[2].orb_held);
	}

	/// Plays one tick of `controller`, aimed ahead, and answers what happened.
	fn tick(controller: &mut HuntController) -> Vec<(i64, HuntEvent)> {
		controller.frame(
			HuntController::TICK_SECONDS,
			&ArPose::AHEAD,
			FrameEye::Origin,
		)
	}

	/// The ticks the record presses FIRE on, from `from` on.
	fn fired_on(controller: &HuntController, from: usize) -> Vec<usize> {
		controller.log().inputs[from..]
			.iter()
			.enumerate()
			.filter(|(_, input)| input.fire)
			.map(|(index, _)| from + index)
			.collect()
	}

	fn shots(events: &[(i64, HuntEvent)]) -> Vec<i64> {
		events
			.iter()
			.filter(|(_, event)| matches!(event, HuntEvent::ShotFired { .. }))
			.map(|(tick, _)| *tick)
			.collect()
	}

	#[test]
	fn a_press_while_the_pistol_recovers_is_fired_the_tick_it_can_be() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.press();
		assert_eq!(shots(&tick(&mut controller)), [0]);
		for _ in 1..12 {
			tick(&mut controller);
		}
		// Six ticks before the pistol can fire again: the press waits for it,
		// and the record presses FIRE on the tick the shot goes.
		assert_eq!(controller.sim().cooldown_ticks(), 6);
		controller.press();
		let mut heard = Vec::new();
		for _ in 0..10 {
			heard.extend(tick(&mut controller));
		}
		assert_eq!(shots(&heard), [18]);
		assert_eq!(fired_on(&controller, 1), [18]);
		// Fourteen ticks away is longer than a press waits: it is played at
		// once, as it always was, the rules let it go, and it is never played
		// late.
		assert_eq!(controller.sim().cooldown_ticks(), 14);
		let pressed = controller.ticks();
		controller.press();
		let mut heard = Vec::new();
		for _ in 0..30 {
			heard.extend(tick(&mut controller));
		}
		assert_eq!(shots(&heard), Vec::<i64>::new());
		assert_eq!(fired_on(&controller, 19), [pressed]);
	}

	#[test]
	fn a_press_waits_at_most_eight_ticks() {
		for wait in [1, 8] {
			let mut controller = HuntController::new(Fight::Hunt(sim(3)));
			controller.press();
			tick(&mut controller);
			while controller.sim().cooldown_ticks() > wait {
				tick(&mut controller);
			}
			let pressed = controller.ticks();
			controller.press();
			for _ in 0..=FIRE_BUFFER_TICKS {
				tick(&mut controller);
			}
			assert_eq!(
				fired_on(&controller, pressed),
				[pressed + wait as usize],
				"waiting {wait}"
			);
		}
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		controller.press();
		tick(&mut controller);
		while controller.sim().cooldown_ticks() > FIRE_BUFFER_TICKS + 1 {
			tick(&mut controller);
		}
		let pressed = controller.ticks();
		controller.press();
		let mut heard = Vec::new();
		for _ in 0..20 {
			heard.extend(tick(&mut controller));
		}
		// Nine is one too many: played at once and let go.
		assert_eq!(fired_on(&controller, pressed), [pressed]);
		assert_eq!(shots(&heard), Vec::<i64>::new());
	}

	#[test]
	fn a_press_in_a_reload_is_played_at_once() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		while controller.sim().rounds() > 0 {
			controller.press();
			tick(&mut controller);
		}
		controller.press();
		let started = controller.ticks();
		assert!(
			tick(&mut controller)
				.iter()
				.any(|(_, event)| *event == HuntEvent::ReloadStarted)
		);
		tick(&mut controller);
		// Pressed too early in the reload, it jams: on the very tick.
		controller.press();
		let jammed = controller.ticks();
		assert!(controller.sim().reloading());
		assert!(
			tick(&mut controller)
				.iter()
				.any(|(at, event)| *at == jammed as i64 && *event == HuntEvent::ReloadJammed)
		);
		assert_eq!(fired_on(&controller, started), [started, jammed]);
	}

	#[test]
	fn a_press_in_a_dive_is_played_at_once() {
		let mut dived = None;
		for seed in [2, 4, 3] {
			let mut sim = sim(seed);
			let mut bot = HuntBot::human(&sim);
			while !sim.diving() && !sim.is_over() && sim.tick() < 20_000 {
				let input = bot.decide(&sim);
				sim.step(&input);
			}
			if sim.diving() {
				dived = Some(sim);
				break;
			}
		}
		let sim = dived.expect("the rules' player dives");
		let mut controller = HuntController::new(Fight::Hunt(sim));
		controller.press();
		tick(&mut controller);
		assert_eq!(fired_on(&controller, 0), [0]);
	}

	#[test]
	fn a_waiting_press_is_dropped_when_the_husk_shrieks() {
		let mut controller = HuntController::new(Fight::Hunt(sim(3)));
		// The first shot gives up the ambush, and the fight begins.
		controller.press();
		tick(&mut controller);
		let windup_left = |controller: &HuntController| {
			((1.0 - controller.sim().shriek_progress()) * HuntRules::WINDUP_TICKS as f64).round()
				as i64
		};
		while controller.sim().phase() != HuntPhase::Windup || windup_left(&controller) > 16 {
			assert!(controller.ticks() < 5_000, "it draws breath");
			tick(&mut controller);
		}
		assert_eq!(windup_left(&controller), 16);
		controller.press();
		tick(&mut controller);
		while windup_left(&controller) > 6 {
			tick(&mut controller);
		}
		// Eight ticks from a shot, which is as long as a press waits, but the
		// husk shrieks in six.
		assert_eq!(controller.sim().cooldown_ticks(), 8);
		let pressed = controller.ticks();
		controller.press();
		let mut heard = Vec::new();
		while controller.sim().phase() != HuntPhase::Fight {
			assert!(controller.ticks() < pressed + 200);
			heard.extend(tick(&mut controller));
		}
		assert!(
			heard
				.iter()
				.any(|(_, event)| *event == HuntEvent::ShriekBegan)
		);
		assert_eq!(shots(&heard), Vec::<i64>::new());
		assert!(
			!heard
				.iter()
				.any(|(_, event)| matches!(event, HuntEvent::Flinched { .. })),
			"{heard:?}"
		);
		assert_eq!(fired_on(&controller, pressed), Vec::<usize>::new());
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
			autoplay.frame(1.0 / 30.0, &pose, FrameEye::Origin);
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
				controller.frame(1.0 / 30.0, &pose, FrameEye::Origin);
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
