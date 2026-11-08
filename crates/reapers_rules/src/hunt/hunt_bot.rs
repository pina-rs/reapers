//! A simulated player.

use crate::arts::Art;
use crate::arts::ArtRules;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::HuntPhase;
use crate::hunt::hunt_input::Aim;
use crate::hunt::hunt_input::HuntInput;
use crate::hunt::hunt_input::HuntLog;
use crate::hunt::hunt_sim::HuntSim;
use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::husk_pose::Footing;
use crate::math;
use crate::vec3::Vec3;
use crate::vec3::radians;
use crate::vec3::wrap_angle;

/// How a [`HuntBot`] plays: how fast it turns, how quickly it sees, and
/// how often it misses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntBotStyle {
	/// How fast the bot turns toward a target, in radians a second.
	pub turn_speed: f64,

	/// How fast the bot sweeps the reap arc, in radians a second.
	pub reap_speed: f64,

	/// Ticks a seal must have been open before the bot taps it.
	pub reaction_ticks: i64,

	/// Every this-many taps lands wide of the husk. Zero means never.
	pub miss_every: i64,

	/// How far through a charge the bot lets the husk come before it
	/// swings, from 0 to 1. Above 1 it never swings, and takes the blow.
	pub charge_nerve: f64,
}

impl Default for HuntBotStyle {
	fn default() -> Self {
		Self::CAREFUL
	}
}

impl HuntBotStyle {
	/// A bot with no limits but its turning: it sees a seal the moment it
	/// opens and never taps wide.
	pub const CAREFUL: Self = Self {
		turn_speed: 1.2,
		reap_speed: 0.9,
		reaction_ticks: 0,
		miss_every: 0,
		charge_nerve: 0.4,
	};
	/// A bot with a person's limits: it needs a moment to notice a seal has
	/// opened, and now and then it taps wide.
	pub const HUMAN: Self = Self {
		reaction_ticks: 16,
		miss_every: 7,
		..Self::CAREFUL
	};
}

/// A simulated player: plays a hunt the way a careful person would.
///
/// The bot reads only what a player can see: where the husk is, which seals
/// are open, and what phase the hunt is in. It turns at a human pace, taps
/// open seals as fast as the weapon allows, holds still through a shriek and
/// sweeps the reap arc at a steady speed. A bot whose reaper knows an art
/// throws it when it is what the moment calls for.
///
/// It exists for three jobs: the rules' own tests and pacing checks, the
/// game's autoplay (which lets a hunt be watched without being played), and
/// filling a rift with company when too few reapers have gathered.
///
/// The bot does not hold the hunt it plays; each call is handed it.
#[derive(Clone, Debug)]
pub struct HuntBot {
	pub style: HuntBotStyle,

	open_since: Vec<i64>,
	taps: i64,

	/// Every input the bot has made, in order.
	pub recorded: Vec<HuntInput>,

	/// Every event its inputs brought about, in order, when it played with
	/// [`play`](Self::play).
	pub events: Vec<HuntEvent>,

	aim: Vec3,
	reap_target: f64,
}

impl HuntBot {
	/// A bot that plays `sim` in `style`.
	pub fn new(sim: &HuntSim, style: HuntBotStyle) -> Self {
		Self {
			style,
			open_since: vec![-1; sim.vessel().seals().len()],
			taps: 0,
			recorded: Vec::new(),
			events: Vec::new(),
			aim: Vec3::FORWARD,
			reap_target: 0.0,
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
		match sim.phase() {
			HuntPhase::Stalk => self.turn_toward(sim.pose().direction(), self.style.turn_speed),
			HuntPhase::Shriek | HuntPhase::Absorbed | HuntPhase::Escaped => self.hold(),
			HuntPhase::Fight | HuntPhase::Windup => {
				if Self::wants_to_throw(sim) {
					self.cast(sim)
				} else {
					self.fight(sim)
				}
			}
			HuntPhase::Charge => {
				if Self::wants_to_throw(sim) && sim.charge_progress() < self.style.charge_nerve {
					self.cast(sim)
				} else {
					self.cut(sim)
				}
			}
			HuntPhase::Broken => self.reap(sim),
		}
	}

	fn fight(&mut self, sim: &HuntSim) -> HuntInput {
		let pose = sim.pose();
		let input = self.turn_toward(pose.direction(), self.style.turn_speed);
		let mut open = Vec::new();

		for index in 0..self.open_since.len() {
			if !sim.seal_open(index) {
				self.open_since[index] = -1;
				continue;
			}

			if self.open_since[index] < 0 {
				self.open_since[index] = sim.tick();
			}

			if sim.tick() - self.open_since[index] >= self.style.reaction_ticks {
				open.push(index);
			}
		}

		// Stop striking as the warning ends, so no tap lands inside the
		// shriek.
		let near_shriek = sim.phase() == HuntPhase::Windup && sim.shriek_progress() > 0.8;
		let swing_ticks = sim.stats().swing_ticks;

		if open.is_empty() || near_shriek || swing_ticks <= 0 || sim.tick() % swing_ticks != 0 {
			return input;
		}

		open.sort_by_key(|&seal| sim.seal_rank(seal).unwrap_or(0));

		self.taps += 1;

		let wide = self.style.miss_every > 0 && self.taps % self.style.miss_every == 0;
		let target = if wide {
			Vec3::from_angles(pose.yaw() + radians(35.0), pose.pitch())
		} else {
			pose.seal_direction(open[0])
		};

		HuntInput {
			taps: vec![Aim::from_direction(target)],
			..input
		}
	}

	/// Whether to reach for the art: a throw is ready, nothing is in the
	/// air, and the husk is where this art is for.
	fn wants_to_throw(sim: &HuntSim) -> bool {
		let Some(art) = sim.stats().art else {
			return false;
		};
		let ready = sim.art_charges() > 0
			&& sim.flights().is_empty()
			&& !sim.frozen()
			&& !(sim.phase() == HuntPhase::Windup && sim.shriek_progress() > 0.4);

		if !ready {
			return false;
		}

		let away = sim.footing() != Footing::Open && !sim.leaping();

		match art.art {
			// An ember is lobbed at a place, so it is thrown at a husk at
			// rest.
			Art::Cinder => !sim.burning() && sim.speed() == 0.0 && !sim.leaping(),
			Art::Rime => sim.phase() == HuntPhase::Charge || sim.art_charges() == art.max_charges(),
			Art::Gale => {
				away || sim.phase() == HuntPhase::Charge || sim.art_charges() == art.max_charges()
			}
		}
	}

	/// Holds the art with one thumb and steadies on the husk, tapping with
	/// the other as it would anyway, then flicks.
	fn cast(&mut self, sim: &HuntSim) -> HuntInput {
		if sim.art_gathered() >= 1.0 && self.aim.angle_to(sim.pose().direction()) < radians(4.0) {
			return HuntInput::measured(self.aim, ArtRules::THROW_SPEED * 1.25, &[], true);
		}

		let input = if sim.phase() == HuntPhase::Charge {
			self.turn_toward(sim.pose().direction(), self.style.turn_speed)
		} else {
			self.fight(sim)
		};

		HuntInput {
			focus: true,
			..input
		}
	}

	fn reap(&mut self, sim: &HuntSim) -> HuntInput {
		if !sim.reap_begun() {
			self.reap_target = 0.0;

			return self.turn_toward(sim.reap_arc_point(0.0), self.style.turn_speed);
		}

		self.reap_target = math::clamp(
			self.reap_target + self.style.reap_speed / HuntRules::TICK_RATE as f64 / 0.9,
			0.0,
			1.05,
		);

		self.turn_toward(sim.reap_arc_point(self.reap_target), self.style.reap_speed)
	}

	/// Meets a charge: keeps fighting until the husk is well on its way,
	/// then swings across it, hard enough to turn it aside.
	fn cut(&mut self, sim: &HuntSim) -> HuntInput {
		if sim.charge_progress() < self.style.charge_nerve {
			return self.fight(sim);
		}

		self.turn_toward(
			Vec3::from_angles(self.aim.yaw() + radians(40.0), self.aim.pitch()),
			sim.stats().parry_speed * 1.2,
		)
	}

	fn hold(&self) -> HuntInput {
		HuntInput::still(self.aim, 0.0)
	}

	/// Turns the aim toward `target` at no more than `speed`, and reports
	/// the speed actually turned at.
	fn turn_toward(&mut self, target: Vec3, speed: f64) -> HuntInput {
		let angle = self.aim.angle_to(target);
		let step = speed / HuntRules::TICK_RATE as f64;

		if angle <= step {
			let turned = angle * HuntRules::TICK_RATE as f64;

			self.aim = target.normalized();

			return HuntInput::still(self.aim, turned);
		}

		let share = step / angle;

		self.aim = Vec3::from_angles(
			self.aim.yaw() + wrap_angle(target.yaw() - self.aim.yaw()) * share,
			self.aim.pitch() + (target.pitch() - self.aim.pitch()) * share,
		);

		HuntInput::still(self.aim, speed)
	}
}
