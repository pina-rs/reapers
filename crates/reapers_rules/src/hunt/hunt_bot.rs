//! A simulated player.

use crate::arts::Art;
use crate::hunt::arsenal::Archetype;
use crate::hunt::arsenal::Arsenal;
use crate::hunt::arsenal::BladeRules;
use crate::hunt::arsenal::GunRules;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::HuntPhase;
use crate::hunt::hunt_input::HuntInput;
use crate::hunt::hunt_input::HuntLog;
use crate::hunt::hunt_sim::HuntSim;
use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::husk_pose::Footing;
use crate::math;
use crate::vec3::Vec3;
use crate::vec3::radians;
use crate::vec3::wrap_angle;

/// How a [`HuntBot`] plays: how fast it turns, how quickly it sees, how
/// often it misses, and how it handles the gun and the orb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntBotStyle {
	/// How fast the bot turns toward a target, in radians a second.
	pub turn_speed: f64,

	/// Ticks a seal must have been open before the bot aims at it.
	pub reaction_ticks: i64,

	/// Every this-many presses of FIRE go wide of the husk. Zero means
	/// never.
	pub miss_every: i64,

	/// How far through a charge the bot lets the husk come before it
	/// swings, from 0 to 1. Above 1 it never swings, and takes the blow.
	pub charge_nerve: f64,

	/// The tick of a reload at which the bot presses FIRE again to finish
	/// it early, or `None` for a bot that waits every reload out.
	pub active_reload_tick: Option<i64>,

	/// Every this-many active reloads, the bot presses too early and jams
	/// the gun. Zero means never.
	pub jam_every: i64,

	/// How long the bot holds ORB before letting go: where the shrinking
	/// ring is when it throws.
	pub orb_hold_ticks: i64,

	/// The share of its ward a husk must be at or under before the bot
	/// throws an orb at it while it still stands. Zero waits for the ward
	/// to break.
	pub orb_at_share: f64,

	/// Whether the bot takes the knife to a stunned husk in arm's reach.
	pub knife: bool,

	/// Whether the bot presses FIRE at all. One that does not only parries,
	/// holds still and throws orbs: a reaper in a rift who leaves the ward
	/// to the others.
	pub fires: bool,
}

impl Default for HuntBotStyle {
	fn default() -> Self {
		Self::CAREFUL
	}
}

impl HuntBotStyle {
	/// A bot with no limits but its turning: it sees a seal the moment it
	/// opens, never fires wide and finishes every reload in the active
	/// window.
	pub const CAREFUL: Self = Self {
		turn_speed: 1.2,
		reaction_ticks: 0,
		miss_every: 0,
		charge_nerve: 0.4,
		active_reload_tick: Some(52),
		jam_every: 0,
		orb_hold_ticks: 58,
		orb_at_share: 0.0,
		knife: true,
		fires: true,
	};
	/// A bot with a person's limits: it needs a moment to notice a seal has
	/// opened, now and then it fires wide, one active reload in three comes
	/// too early and jams, and it lets go of ORB with the ring in its
	/// middle third.
	pub const HUMAN: Self = Self {
		reaction_ticks: 16,
		miss_every: 7,
		jam_every: 3,
		orb_hold_ticks: 40,
		..Self::CAREFUL
	};
}

/// A simulated player: plays a hunt the way a careful person would.
///
/// The bot reads only what a player can see: where the husk is, which
/// seals are open, what the ring says and what its weapons show. It turns
/// at a human pace and fires the gun when the crosshair is on an open seal
/// or the body, reloads (actively, when it is told to), parries a charge,
/// then takes the knife to a stunned husk in arm's reach. It holds still
/// through a shriek, and once the husk is broken it holds ORB until the
/// ring is where it wants it and lets go. A bot whose reaper knows an art
/// throws it when it is what the moment calls for.
///
/// It exists for three jobs: the rules' own tests and pacing checks, the
/// game's autoplay (which lets a hunt be watched without being played),
/// and filling a rift with company when too few reapers have gathered.
///
/// The bot does not hold the hunt it plays; each call is handed it.
#[derive(Clone, Debug)]
pub struct HuntBot {
	pub style: HuntBotStyle,

	open_since: Vec<i64>,
	presses: i64,
	reloads: i64,
	orb_held: i64,
	art_wanted: bool,
	art_charges_seen: i64,

	/// Every input the bot has made, in order.
	pub recorded: Vec<HuntInput>,

	/// Every event its inputs brought about, in order, when it played with
	/// [`play`](Self::play).
	pub events: Vec<HuntEvent>,

	aim: Vec3,
}

impl HuntBot {
	/// How far into a reload the bot will still go for the art: there is
	/// just time to draw it, throw, and have the gun back for the active
	/// reload.
	const ART_RELOAD_TICKS: i64 = 4;

	/// A bot that plays `sim` in `style`.
	pub fn new(sim: &HuntSim, style: HuntBotStyle) -> Self {
		Self {
			style,
			open_since: vec![-1; sim.vessel().seals().len()],
			presses: 0,
			reloads: 0,
			orb_held: 0,
			art_wanted: false,
			art_charges_seen: 0,
			recorded: Vec::new(),
			events: Vec::new(),
			aim: Vec3::FORWARD,
		}
	}

	/// A careful bot for `sim`.
	pub fn careful(sim: &HuntSim) -> Self {
		Self::new(sim, HuntBotStyle::CAREFUL)
	}

	/// A bot with a person's limits for `sim`.
	pub fn human(sim: &HuntSim) -> Self {
		Self::new(sim, HuntBotStyle::HUMAN)
	}

	/// Plays until the hunt ends or [`HuntLog::MAX_TICKS`] pass.
	pub fn play(&mut self, sim: &mut HuntSim) -> Option<HuntOutcome> {
		self.play_for(sim, HuntLog::MAX_TICKS as i64)
	}

	/// Plays until the hunt ends or `max_ticks` pass.
	pub fn play_for(&mut self, sim: &mut HuntSim, max_ticks: i64) -> Option<HuntOutcome> {
		while !sim.is_over() && sim.tick() < max_ticks {
			let events = self.step_once(sim);

			self.events.extend(events);
		}

		sim.outcome()
	}

	/// Advances one tick and returns its events.
	pub fn step_once(&mut self, sim: &mut HuntSim) -> Vec<HuntEvent> {
		let input = self.decide(sim);
		let events = sim.step(&input);

		self.recorded.push(input);

		events
	}

	/// Where the bot is looking, after its last decision.
	pub const fn aim(&self) -> Vec3 {
		self.aim
	}

	/// What the bot does this tick. Whoever calls this must feed the result
	/// to the hunt, since the bot assumes each decision is acted on.
	pub fn decide(&mut self, sim: &HuntSim) -> HuntInput {
		self.track_seals(sim);

		match sim.phase() {
			HuntPhase::Stalk => {
				let speed = self.turn_toward(sim.pose().direction(), self.style.turn_speed);

				self.input(sim, speed, None, false, false, None)
			}
			HuntPhase::Shriek | HuntPhase::Capture | HuntPhase::Absorbed | HuntPhase::Escaped => {
				self.hold(sim)
			}
			HuntPhase::Fight | HuntPhase::Windup => self.fight(sim),
			HuntPhase::Charge => {
				if sim.charge_progress() < self.style.charge_nerve {
					self.fight(sim)
				} else {
					self.parry(sim)
				}
			}
			HuntPhase::Broken => self.broken(sim),
		}
	}

	fn track_seals(&mut self, sim: &HuntSim) {
		for index in 0..self.open_since.len() {
			if !sim.seal_open(index) {
				self.open_since[index] = -1;
			} else if self.open_since[index] < 0 {
				self.open_since[index] = sim.tick();
			}
		}
	}

	/// Where to put the crosshair: the oldest open seal it has noticed, or
	/// the middle of the body.
	fn target(&self, sim: &HuntSim) -> Vec3 {
		let mut noticed: Vec<usize> = (0..self.open_since.len())
			.filter(|&index| {
				self.open_since[index] >= 0
					&& sim.tick() - self.open_since[index] >= self.style.reaction_ticks
			})
			.collect();

		noticed.sort_by_key(|&seal| sim.seal_rank(seal).unwrap_or(0));

		if noticed.is_empty() {
			sim.pose().direction()
		} else {
			sim.pose().seal_direction(noticed[0])
		}
	}

	fn fight(&mut self, sim: &HuntSim) -> HuntInput {
		let target = self.target(sim);
		let speed = self.turn_toward(target, self.style.turn_speed);
		let on_target = self.aim.angle_to(target) < radians(0.5);

		// Nothing is pressed as the warning ends, so nothing lands in the
		// shriek.
		let calm = sim.phase() != HuntPhase::Windup || sim.shriek_progress() < 0.75;
		let orb = self.orb_hand(sim, calm && sim.phase() == HuntPhase::Fight);
		let slot = self.choose_slot(sim);

		if sim.selected_slot() != slot || !calm || !self.style.fires {
			return self.input(sim, speed, Some(slot), false, orb, None);
		}

		let fire = match Arsenal::archetype_of(slot) {
			Some(Archetype::Gun) => self.gun_press(sim, on_target),
			Some(Archetype::Blade | Archetype::Heavy) => {
				if sim.diving() {
					sim.tick() % BladeRules::STAB_INTERVAL_TICKS == 0
				} else {
					on_target && sim.cooldown_ticks() == 0
				}
			}
			_ => on_target && sim.cooldown_ticks() == 0,
		};

		if !fire {
			return self.input(sim, speed, Some(slot), false, orb, None);
		}

		self.presses += 1;

		let wide = self.style.miss_every > 0 && self.presses % self.style.miss_every == 0;
		let aim = if wide {
			Some(Vec3::from_angles(
				self.aim.yaw() + radians(12.0),
				self.aim.pitch(),
			))
		} else {
			None
		};

		self.input(sim, speed, Some(slot), true, orb, aim)
	}

	/// Whether to press FIRE with the gun in hand: to shoot, to start a
	/// reload, or to finish one early.
	fn gun_press(&mut self, sim: &HuntSim, on_target: bool) -> bool {
		if sim.reloading() {
			let Some(active_tick) = self.style.active_reload_tick else {
				return false;
			};

			if sim.reload_jammed() {
				return false;
			}

			let into = math::round(sim.reload_progress() * GunRules::RELOAD_TICKS as f64);
			let jams = self.style.jam_every > 0 && self.reloads % self.style.jam_every == 0;

			return into == if jams { active_tick / 2 } else { active_tick };
		}

		if sim.rounds() == 0 {
			self.reloads += 1;

			return true;
		}

		on_target && sim.cooldown_ticks() == 0
	}

	/// The slot the moment calls for.
	fn choose_slot(&mut self, sim: &HuntSim) -> usize {
		if sim.diving() {
			return sim.selected_slot();
		}

		if self.wants_art(sim) {
			return Arsenal::ART_SLOT;
		}

		let distance = sim.pose().distance();
		let close = distance <= Arsenal::CLOSE_METRES;
		let reach = close || (distance <= BladeRules::DIVE_METRES && sim.dive_charges() > 0);

		// Only worth it if the first stab lands before the stun wears off.
		let arrive = if sim.selected_slot() == Arsenal::BLADE_SLOT {
			0
		} else {
			Arsenal::SWITCH_TICKS
		} + if close { 0 } else { BladeRules::IN_TICKS };

		if self.style.knife && sim.stunned() && reach && sim.stun_ticks_left() > arrive + 4 {
			return Arsenal::BLADE_SLOT;
		}

		Arsenal::GUN_SLOT
	}

	/// Whether to reach for the art: a throw is ready, nothing is in the
	/// air, and the husk is where this art is for. Drawing it and drawing
	/// the gun again costs shots, so the bot reaches for it as the gun
	/// starts to reload, which goes on in the other hand, unless the husk
	/// has gone where only wind can reach it. Once it has decided, it keeps
	/// to it until the throw is gone.
	fn wants_art(&mut self, sim: &HuntSim) -> bool {
		let Some(art) = sim.stats().art else {
			return false;
		};

		if self.art_wanted {
			let thrown = sim.art_charges() < self.art_charges_seen;

			if thrown || sim.art_charges() == 0 {
				self.art_wanted = false;
			}

			return self.art_wanted;
		}

		let ready = sim.art_charges() > 0
			&& sim.flights().is_empty()
			&& !sim.frozen()
			&& sim.phase() == HuntPhase::Fight;

		if !ready {
			return false;
		}

		let away = sim.footing() != Footing::Open && !sim.leaping();
		let spare = sim.reloading()
			&& sim.reload_progress() * GunRules::RELOAD_TICKS as f64
				<= Self::ART_RELOAD_TICKS as f64;

		if !spare && (art.art != Art::Gale || !away) {
			return false;
		}

		self.art_wanted = match art.art {
			// An ember is lobbed at a place, so it is thrown at a husk at
			// rest.
			Art::Cinder => !sim.burning() && sim.speed() == 0.0 && !sim.leaping(),
			Art::Rime => sim.art_charges() == art.max_charges(),
			Art::Gale => away || sim.art_charges() == art.max_charges(),
		};
		self.art_charges_seen = sim.art_charges();

		self.art_wanted
	}

	/// Meets a charge: swings across it, hard enough to turn it aside.
	fn parry(&mut self, sim: &HuntSim) -> HuntInput {
		let speed = self.turn_toward(
			Vec3::from_angles(self.aim.yaw() + radians(40.0), self.aim.pitch()),
			sim.stats().parry_speed * 1.2,
		);

		self.input(sim, speed, Some(sim.selected_slot()), false, false, None)
	}

	/// The husk is down: hold ORB on it and let go with the ring where the
	/// bot wants it.
	fn broken(&mut self, sim: &HuntSim) -> HuntInput {
		let speed = self.turn_toward(sim.pose().direction(), self.style.turn_speed);
		let orb = self.orb_hand(sim, true);

		self.input(sim, speed, Some(sim.selected_slot()), false, orb, None)
	}

	/// Whether ORB is down this tick. The bot holds it while an orb can be
	/// thrown and it wants one, and lets go after `orb_hold_ticks`, or at
	/// once when the moment stops being `calm`.
	fn orb_hand(&mut self, sim: &HuntSim, calm: bool) -> bool {
		let wanted = sim.orb_block().is_none()
			&& (sim.phase() == HuntPhase::Broken
				|| (self.style.orb_at_share > 0.0
					&& sim.ward() <= sim.ward_max() * self.style.orb_at_share));

		if self.orb_held > 0 && (self.orb_held >= self.style.orb_hold_ticks || !calm) {
			self.orb_held = 0;

			return false;
		}

		if self.orb_held == 0 && (!wanted || !calm) {
			return false;
		}

		self.orb_held += 1;

		true
	}

	fn hold(&mut self, sim: &HuntSim) -> HuntInput {
		self.orb_held = 0;

		HuntInput::measured(self.aim, 0.0, false, sim.selected_slot(), false)
	}

	fn input(
		&self,
		sim: &HuntSim,
		speed: f64,
		slot: Option<usize>,
		fire: bool,
		orb_held: bool,
		aim: Option<Vec3>,
	) -> HuntInput {
		HuntInput::measured(
			aim.unwrap_or(self.aim),
			speed,
			fire,
			slot.unwrap_or(sim.selected_slot()),
			orb_held,
		)
	}

	/// Turns the aim toward `target` at no more than `speed`, and returns
	/// the speed actually turned at.
	fn turn_toward(&mut self, target: Vec3, speed: f64) -> f64 {
		let angle = self.aim.angle_to(target);
		let step = speed / HuntRules::TICK_RATE as f64;

		if angle <= step {
			self.aim = target.normalized();

			return angle * HuntRules::TICK_RATE as f64;
		}

		let share = step / angle;

		self.aim = Vec3::from_angles(
			self.aim.yaw() + wrap_angle(target.yaw() - self.aim.yaw()) * share,
			self.aim.pitch() + (target.pitch() - self.aim.pitch()) * share,
		);

		speed
	}
}
