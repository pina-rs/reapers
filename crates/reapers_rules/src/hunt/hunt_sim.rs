//! One encounter between a reaper and a husk, advanced a tick at a time.

use thiserror::Error;

use crate::arts::Art;
use crate::arts::ArtRules;
use crate::arts::ArtSkill;
use crate::hunt::art_flight::ArtFlight;
use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_event::EscapeCause;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::HuntPhase;
use crate::hunt::hunt_event::ReapGrade;
use crate::hunt::hunt_input::Aim;
use crate::hunt::hunt_input::HuntInput;
use crate::hunt::hunt_input::HuntLog;
use crate::hunt::hunt_plan::HuntPlan;
use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::hunt_stats::HuntStats;
use crate::hunt::husk_pose::Footing;
use crate::hunt::husk_pose::HuskPose;
use crate::math::PI;
use crate::math::{self};
use crate::reaper_class::ClassPerk;
use crate::reaper_soul::HuntDeeds;
use crate::reaper_soul::ReaperGift;
use crate::reaper_soul::SoulRules;
use crate::rift::rift_room::RiftSighting;
use crate::rift::rift_rules::RiftRules;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;
use crate::vec3::Vec3;
use crate::vec3::radians;
use crate::vec3::smoothstep;
use crate::vec3::wrap_angle;
use crate::vessel::Gait;
use crate::vessel::Vessel;

/// A plan no phone could have made, so no hunt is played in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("The plan is not one a phone could make")]
pub struct IllFormedPlan;

/// One encounter between a reaper and a husk, advanced a tick at a time.
///
/// The simulation owns every result: where the husk is, whether a tap struck
/// a seal, whether a reaper flinched and how clean the reap was. It reads
/// nothing but its seed and the [`HuntInput`] of each tick, so the same seed
/// and the same inputs always end the same way, on a phone or on the server.
#[derive(Clone, Debug)]
#[allow(
	clippy::struct_excessive_bools,
	reason = "the hunt's flags, kept as they replay"
)]
pub struct HuntSim {
	seed: i64,
	vessel: Vessel,
	stats: HuntStats,
	plan: HuntPlan,
	difficulty: Difficulty,
	in_rift: bool,
	ward_max: f64,

	schedule: SeededRandom,

	/// Decides when the husk shrieks, and nothing else. Kept apart from
	/// `schedule`, which a reaper's own strikes advance, so that everyone in
	/// a rift is told to hold still at the same moment.
	shrieks: SeededRandom,

	/// Decides where the husk goes, and nothing else.
	path: SeededRandom,

	/// Decides when the husk charges, and nothing else.
	charges: SeededRandom,

	/// Decides when the husk makes for cover or a ledge, and which. Kept
	/// apart from `path` so that a room with nothing in it plays exactly as
	/// one did before the room's furniture was known.
	cover: SeededRandom,

	// Movement -----------------------------------------------------------
	/// Where the husk is: its bearing round the player, how far away it is
	/// along the floor, and how high the middle of its body is above the
	/// player's eye.
	bearing: f64,
	range: f64,
	height: f64,

	/// The way its body faces.
	heading: f64,

	bob_phase: f64,

	/// The move under way: where it started, how far it goes, when it began
	/// and how long it takes. `move_start` is negative while the husk rests.
	from_bearing: f64,
	from_range: f64,
	from_height: f64,
	by_bearing: f64,
	by_range: f64,
	by_height: f64,
	move_start: i64,
	move_ticks: i64,

	next_amble_tick: i64,
	next_bolt_tick: i64,

	/// What the husk stands on or behind, and, while `leaping`, what it is
	/// leaping to and how high the leap carries it above the straight line.
	footing: Footing,
	to_footing: Footing,
	leaping: bool,
	arc: f64,

	/// The surface under the husk, in metres from the eye, where it set off
	/// from and where it is going: for the view to put its shadow on.
	from_ground: f64,
	to_ground: f64,

	// Arts ---------------------------------------------------------------
	art_charges: i64,
	recharge_ticks: i64,
	gather_ticks: i64,
	cast_aim: Option<Aim>,
	flights: Vec<ArtFlight>,
	burn_until: i64,
	burn_per_tick: f64,
	frozen_until: i64,

	/// Until when the husk keeps to the open: burning, or shaken by wind.
	exposed_until: i64,

	// Seals --------------------------------------------------------------
	open_until: Vec<i64>,
	shut_until: Vec<i64>,
	opened_order: Vec<i64>,
	opened_count: i64,

	// Strikes ------------------------------------------------------------
	next_strike_tick: i64,
	buffered_tap: Option<Aim>,
	buffered_at: i64,
	last_hit_tick: i64,
	ambush_strike_ready: bool,
	chosen_blow_ready: bool,
	gift_strike_ready: bool,
	passage_ready: bool,
	return_ready: bool,
	gift_uses: i64,
	ordered_hits: i64,
	refrain_tolls: i64,
	echo_cadence: i64,
	holds: i64,
	parries: i64,
	hit_streak: i64,
	max_hit_streak: i64,

	// Shriek -------------------------------------------------------------
	phase_end_tick: i64,
	shriek_start_tick: i64,
	next_shriek_tick: i64,
	flinched_this_shriek: bool,
	steadfast_spent: bool,
	fight_ticks: i64,

	// Charge -------------------------------------------------------------
	next_charge_tick: i64,
	charge_start_tick: i64,
	staggered_until: i64,

	// Reap ---------------------------------------------------------------
	reap_direction: f64,
	arc_yaw: f64,
	arc_pitch: f64,
	capture_end_tick: i64,
	bonus_capture_ticks: i64,
	deviation_sum: f64,
	reap_ticks: i64,
	fast_ticks: i64,

	// Readable state -----------------------------------------------------
	tick: i64,
	phase: HuntPhase,
	ward: f64,
	resolve: i64,
	awareness: f64,
	steady_ticks: i64,
	ambushed: bool,
	cadence: i64,
	reap_attempts_left: i64,
	reap_begun: bool,
	reap_progress: f64,
	outcome: Option<HuntOutcome>,
}

/// Somewhere the room lets a husk be that it has to leap to.
#[derive(Clone, Copy, Debug)]
struct Spot {
	bearing: f64,
	range: f64,
	footing: Footing,

	/// The height of the middle of the husk's body there, from the eye.
	height: f64,
}

impl HuntSim {
	/// A lone hunt: the husk hides somewhere around the reaper, and its ward
	/// is theirs alone to break.
	///
	/// `ward` and `stats` are the husk's and the reaper's own. The hunt eases
	/// or stiffens them for `difficulty` itself, so that the phone and the
	/// server cannot disagree about what a difficulty means.
	pub fn new(
		seed: i64,
		vessel: Vessel,
		ward: i64,
		stats: &HuntStats,
		plan: HuntPlan,
		difficulty: Difficulty,
	) -> Result<Self, IllFormedPlan> {
		Self::build(
			seed,
			vessel,
			ward as f64 * difficulty.ward(),
			stats.at(difficulty),
			plan,
			difficulty,
			false,
			None,
		)
	}

	/// A lone hunt in the open at normal difficulty, which is never refused.
	pub fn open(seed: i64, vessel: Vessel, ward: i64, stats: &HuntStats) -> Self {
		Self::new(
			seed,
			vessel,
			ward,
			stats,
			HuntPlan::OPEN,
			Difficulty::Normal,
		)
		.unwrap_or_else(|IllFormedPlan| unreachable!("the open plan is well formed"))
	}

	/// One reaper's place in a rift.
	///
	/// The husk stands dead ahead and the fight starts at once: there is
	/// nothing to stalk. Its `ward` belongs to the whole circle, so whoever
	/// runs the rift keeps this reaper's view of it current with
	/// [`share_ward`](Self::share_ward). Every reaper given the same `seed`
	/// hears the husk shriek at the same moments, and there is one reap
	/// window, not two.
	///
	/// `sighting` is where this reaper sees the husk: the way to the place
	/// on the earth the whole circle is looking at. Without one it stands
	/// dead ahead of them, at its usual distance.
	pub fn rift(
		seed: i64,
		vessel: Vessel,
		ward: f64,
		stats: &HuntStats,
		sighting: Option<RiftSighting>,
	) -> Self {
		Self::build(
			seed,
			vessel,
			ward,
			*stats,
			HuntPlan::OPEN,
			Difficulty::Normal,
			true,
			sighting,
		)
		.unwrap_or_else(|IllFormedPlan| unreachable!("the open plan is well formed"))
	}

	#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
	fn build(
		seed: i64,
		vessel: Vessel,
		ward: f64,
		stats: HuntStats,
		plan: HuntPlan,
		difficulty: Difficulty,
		in_rift: bool,
		sighting: Option<RiftSighting>,
	) -> Result<Self, IllFormedPlan> {
		if !plan.is_well_formed() {
			return Err(IllFormedPlan);
		}

		let seals = vessel.seals().len();
		let mut sim = Self {
			seed,
			vessel,
			stats,
			plan,
			difficulty,
			in_rift,
			ward_max: ward,
			schedule: SeededRandom::new(mix_seed(&[seed, 0x5EA1])),
			shrieks: SeededRandom::new(mix_seed(&[seed, 0x5C81])),
			path: SeededRandom::new(mix_seed(&[seed, 0x9A7B])),
			charges: SeededRandom::new(mix_seed(&[seed, 0xC4A6])),
			cover: SeededRandom::new(mix_seed(&[seed, 0xC07E])),
			bearing: 0.0,
			range: 0.0,
			height: 0.0,
			heading: 0.0,
			bob_phase: 0.0,
			from_bearing: 0.0,
			from_range: 0.0,
			from_height: 0.0,
			by_bearing: 0.0,
			by_range: 0.0,
			by_height: 0.0,
			move_start: -1,
			move_ticks: 1,
			next_amble_tick: 0,
			next_bolt_tick: -1,
			footing: Footing::Open,
			to_footing: Footing::Open,
			leaping: false,
			arc: 0.0,
			from_ground: 0.0,
			to_ground: 0.0,
			art_charges: if stats.art.is_none() {
				0
			} else {
				ArtRules::STARTING_CHARGES
			},
			recharge_ticks: 0,
			gather_ticks: 0,
			cast_aim: None,
			flights: Vec::new(),
			burn_until: 0,
			burn_per_tick: 0.0,
			frozen_until: 0,
			exposed_until: 0,
			open_until: vec![0; seals],
			shut_until: vec![0; seals],
			opened_order: vec![0; seals],
			opened_count: 0,
			next_strike_tick: 0,
			buffered_tap: None,
			buffered_at: 0,
			last_hit_tick: -HuntRules::CADENCE_WINDOW_TICKS - 1,
			ambush_strike_ready: false,
			chosen_blow_ready: false,
			gift_strike_ready: false,
			passage_ready: false,
			return_ready: false,
			gift_uses: 0,
			ordered_hits: 0,
			refrain_tolls: 0,
			echo_cadence: 0,
			holds: 0,
			parries: 0,
			hit_streak: 0,
			max_hit_streak: 0,
			phase_end_tick: 0,
			shriek_start_tick: 0,
			next_shriek_tick: 0,
			flinched_this_shriek: false,
			steadfast_spent: false,
			fight_ticks: 0,
			next_charge_tick: -1,
			charge_start_tick: 0,
			staggered_until: 0,
			reap_direction: 1.0,
			arc_yaw: 0.0,
			arc_pitch: 0.0,
			capture_end_tick: 0,
			bonus_capture_ticks: 0,
			deviation_sum: 0.0,
			reap_ticks: 0,
			fast_ticks: 0,
			tick: 0,
			phase: HuntPhase::Stalk,
			ward,
			resolve: stats.resolve,
			awareness: 0.0,
			steady_ticks: 0,
			ambushed: false,
			cadence: 0,
			reap_attempts_left: if in_rift {
				1
			} else {
				HuntRules::REAP_ATTEMPTS + difficulty.reaps()
			},
			reap_begun: false,
			reap_progress: 0.0,
			outcome: None,
		};

		sim.reap_direction = if sim.path.next_chance(0.5) { 1.0 } else { -1.0 };
		sim.bob_phase = sim.path.next_range(0.0, 2.0 * PI);

		// How far out it starts, as a share of the room it has wherever it
		// turns out to be.
		let outward = sim.path.next_range(0.0, 1.0);

		sim.height = sim.resting_height();
		sim.from_ground = sim.floor();
		sim.to_ground = sim.floor();
		sim.next_amble_tick = sim.rest_ticks();

		if in_rift {
			// A rift's husk stands where the circle sees it, or dead ahead,
			// and faces this reaper.
			sim.bearing = sighting.map_or(0.0, |sighting| sighting.bearing);
			sim.range = match sighting {
				Some(sighting) => sighting.metres,
				None => sim.range_at(sim.bearing, outward),
			};
			sim.heading = wrap_angle(sim.bearing + PI);
			sim.phase = HuntPhase::Fight;
			sim.schedule_shriek();

			return Ok(sim);
		}

		// With the whole circle to hide in, a husk starts somewhere the
		// reaper has to turn to find. Inside a plan it starts anywhere in
		// it.
		sim.bearing = if sim.plan.is_whole_circle() {
			let side = if sim.path.next_chance(0.5) { 1.0 } else { -1.0 };

			side * radians(sim.path.next_range(55.0, 180.0))
		} else {
			let offset = sim.path.next_range(-0.8, 0.8);

			sim.plan
				.clamp(sim.plan.centre() + offset * sim.plan.half_width())
		};
		sim.range = sim.range_at(sim.bearing, outward);
		sim.heading = sim.path.next_range(-PI, PI);

		Ok(sim)
	}

	/// Replays `log` from the start and returns how the hunt ended, or
	/// `None` when the log stops before the hunt does, or was played in a
	/// plan no phone could have made.
	pub fn replay(
		seed: i64,
		vessel: Vessel,
		ward: i64,
		stats: &HuntStats,
		log: &HuntLog,
	) -> Option<HuntOutcome> {
		let mut sim =
			Self::new(seed, vessel, ward, stats, log.plan.clone(), log.difficulty).ok()?;

		for input in &log.inputs {
			sim.step(input);

			if sim.is_over() {
				break;
			}
		}

		sim.outcome
	}

	pub const fn seed(&self) -> i64 {
		self.seed
	}

	pub const fn vessel(&self) -> Vessel {
		self.vessel
	}

	/// The numbers the hunt is played with, already eased or stiffened for
	/// its difficulty.
	pub const fn stats(&self) -> &HuntStats {
		&self.stats
	}

	/// Where the husk may go.
	pub const fn plan(&self) -> &HuntPlan {
		&self.plan
	}

	/// How forgiving the hunt is.
	pub const fn difficulty(&self) -> Difficulty {
		self.difficulty
	}

	/// Whether this is one reaper's place in a rift rather than a lone hunt.
	pub const fn in_rift(&self) -> bool {
		self.in_rift
	}

	/// The ward the husk started with.
	pub const fn ward_max(&self) -> f64 {
		self.ward_max
	}

	/// Ticks advanced so far.
	pub const fn tick(&self) -> i64 {
		self.tick
	}

	pub const fn phase(&self) -> HuntPhase {
		self.phase
	}

	/// What is left of the husk's ward.
	pub const fn ward(&self) -> f64 {
		self.ward
	}

	/// Flinches the reaper can still take.
	pub const fn resolve(&self) -> i64 {
		self.resolve
	}

	/// How alarmed an unaware husk is, from 0 to 1. At 1 it notices.
	pub const fn awareness(&self) -> f64 {
		self.awareness
	}

	/// Ticks the reaper has spent steady on the husk during the stalk.
	pub const fn steady_ticks(&self) -> i64 {
		self.steady_ticks
	}

	/// Whether the fight opened with an ambush.
	pub const fn ambushed(&self) -> bool {
		self.ambushed
	}

	/// Seal hits in a row.
	pub const fn cadence(&self) -> i64 {
		self.cadence
	}

	/// Unfinished reap windows the husk will still allow.
	pub const fn reap_attempts_left(&self) -> i64 {
		self.reap_attempts_left
	}

	/// Whether the aim has reached the start of the arc.
	pub const fn reap_begun(&self) -> bool {
		self.reap_begun
	}

	/// How far along the arc the cut has travelled, from 0 to 1.
	pub const fn reap_progress(&self) -> f64 {
		self.reap_progress
	}

	/// How the hunt ended, or `None` while it is still running.
	pub const fn outcome(&self) -> Option<HuntOutcome> {
		self.outcome
	}

	/// Throws the reaper has ready.
	pub const fn art_charges(&self) -> i64 {
		self.art_charges
	}

	pub const fn is_over(&self) -> bool {
		self.outcome.is_some()
	}

	/// What the reaper has done so far that a soul remembers.
	pub const fn deeds(&self) -> HuntDeeds {
		HuntDeeds {
			holds: self.holds,
			parries: self.parries,
			max_cadence: self.max_hit_streak,
		}
	}

	/// How close the reaper is to springing an ambush, from 0 to 1.
	pub fn steadiness(&self) -> f64 {
		math::clamp(
			self.steady_ticks as f64 / HuntRules::AMBUSH_TICKS as f64,
			0.0,
			1.0,
		)
	}

	/// Ticks of fighting left before the husk escapes.
	pub fn fight_ticks_left(&self) -> i64 {
		(self.fight_limit_ticks() - self.fight_ticks).max(0)
	}

	/// Ticks before another strike can land. The view reads the same
	/// recovery the rules enforce, including a swing that hit nothing.
	pub fn strike_ticks_left(&self) -> i64 {
		(self.next_strike_tick - self.tick).max(0)
	}

	/// Ticks left to land another seal hit and keep the current cadence.
	pub fn cadence_ticks_left(&self) -> i64 {
		if self.cadence == 0 {
			0
		} else {
			(HuntRules::CADENCE_WINDOW_TICKS - (self.tick - self.last_hit_tick)).max(0)
		}
	}

	fn fight_limit_ticks(&self) -> i64 {
		if self.in_rift {
			RiftRules::FIGHT_LIMIT_TICKS
		} else {
			math::round(HuntRules::FIGHT_LIMIT_TICKS as f64 * self.difficulty.patience())
		}
	}

	/// Sets the ward to what the whole circle has left of it.
	///
	/// Only a rift's ward is shared, and only while it is still being fought
	/// over: once it has broken for this reaper it stays broken. A lone
	/// hunt's ward is left alone.
	pub fn share_ward(&mut self, shared: f64) {
		if self.in_rift && self.fighting() {
			self.ward = math::max(0.0, shared);
		}
	}

	/// Ticks left in the current reap window.
	pub fn capture_ticks_left(&self) -> i64 {
		(self.capture_end_tick - self.tick).max(0)
	}

	/// The length of the current reap window.
	pub const fn capture_ticks(&self) -> i64 {
		self.stats.capture_ticks + self.bonus_capture_ticks
	}

	/// How far through the warning or the shriek the husk is, from 0 to 1.
	pub fn shriek_progress(&self) -> f64 {
		let length = match self.phase {
			HuntPhase::Windup => HuntRules::WINDUP_TICKS,
			HuntPhase::Shriek => HuntRules::SHRIEK_TICKS,
			_ => 0,
		};

		if length == 0 {
			return 0.0;
		}

		math::clamp(
			1.0 - (self.phase_end_tick - self.tick) as f64 / length as f64,
			0.0,
			1.0,
		)
	}

	/// Where the husk is right now.
	pub fn pose(&self) -> HuskPose {
		// A husk in the air never hangs quite still. One perched on a ledge
		// does.
		let bob = if self.vessel.gait() == Gait::Air
			&& self.footing == Footing::Open
			&& !self.leaping
		{
			0.05 * math::sin(0.9 * self.tick as f64 / HuntRules::TICK_RATE as f64 + self.bob_phase)
		} else {
			0.0
		};

		HuskPose {
			vessel: self.vessel,
			position: Vec3::new(
				math::sin(self.bearing) * self.range,
				self.height + bob,
				math::cos(self.bearing) * self.range,
			),
			heading: self.heading,
			footing: if self.leaping {
				self.to_footing
			} else {
				self.footing
			},
			leaping: self.leaping,
			ground: self.ground(),
		}
	}

	/// What the husk stands on or behind.
	pub const fn footing(&self) -> Footing {
		self.footing
	}

	/// Whether the husk is in the air between one footing and the next.
	pub const fn leaping(&self) -> bool {
		self.leaping
	}

	/// The floor, in metres from the eye.
	fn floor(&self) -> f64 {
		-self.plan.eye_height()
	}

	fn ground(&self) -> Option<f64> {
		let aloft = self.vessel.gait() != Gait::Ground;

		if !self.leaping {
			return if aloft && self.footing == Footing::Open {
				None
			} else {
				Some(self.from_ground)
			};
		}

		if aloft {
			return None;
		}

		let along = smoothstep((self.tick - self.move_start) as f64 / self.move_ticks as f64);

		Some(self.from_ground + (self.to_ground - self.from_ground) * along)
	}

	/// Arts in the air.
	pub fn flights(&self) -> &[ArtFlight] {
		&self.flights
	}

	/// How far the art in hand is gathered, from 0 to 1. At 1 a flick
	/// throws it.
	pub fn art_gathered(&self) -> f64 {
		if self.stats.art.is_none() {
			0.0
		} else {
			self.gather_ticks as f64 / ArtRules::GATHER_TICKS as f64
		}
	}

	/// Ticks until the next throw is ready, as a share of the wait.
	pub fn art_recharge(&self) -> f64 {
		match self.stats.art {
			Some(art) if self.art_charges < art.max_charges() => {
				self.recharge_ticks as f64 / art.recharge_ticks() as f64
			}
			_ => 0.0,
		}
	}

	/// Whether the husk is burning.
	pub fn burning(&self) -> bool {
		self.fighting() && self.tick < self.burn_until
	}

	/// Whether the husk is frozen where it stands.
	pub fn frozen(&self) -> bool {
		self.fighting() && self.tick < self.frozen_until
	}

	/// How fast the husk is covering ground right now, in metres a second.
	/// Zero while it rests.
	pub fn speed(&self) -> f64 {
		if self.move_start < 0 {
			return 0.0;
		}

		let mean_range = self.from_range + self.by_range / 2.0;
		let round = mean_range * self.by_bearing;
		let length = math::sqrt(
			round * round + self.by_range * self.by_range + self.by_height * self.by_height,
		);

		length / (self.move_ticks as f64 / HuntRules::TICK_RATE as f64)
	}

	/// Whether seal `index` can be struck right now.
	pub fn seal_open(&self, index: usize) -> bool {
		if !self.fighting()
			|| self
				.open_until
				.get(index)
				.is_none_or(|&until| self.tick >= until)
		{
			return false;
		}

		let current = self.pose();

		current.seal_faces_player(index) && !self.behind_cover(&current, index)
	}

	/// Whether seal `index` of a husk at `current` is behind something in
	/// the room, out of sight and out of reach.
	fn behind_cover(&self, current: &HuskPose, index: usize) -> bool {
		self.plan.hides(current.seal_position(index))
	}

	/// The order seal `index` opened in among the seals open now, starting
	/// at 0 for the oldest, or `None` when it is shut.
	///
	/// Only a reaper with the measure perk is shown this.
	pub fn seal_rank(&self, index: usize) -> Option<usize> {
		if !self.seal_open(index) {
			return None;
		}

		Some(
			(0..self.seal_count())
				.filter(|&other| {
					self.seal_open(other) && self.opened_order[other] < self.opened_order[index]
				})
				.count(),
		)
	}

	fn seal_count(&self) -> usize {
		self.vessel.seals().len()
	}

	/// The point a fraction `progress` of the way along the reap arc.
	///
	/// The arc bows upward in the middle, like the path of a swung blade.
	pub fn reap_arc_point(&self, progress: f64) -> Vec3 {
		Vec3::from_angles(
			self.arc_yaw + self.reap_direction * HuntRules::REAP_HALF_SPAN * (2.0 * progress - 1.0),
			self.arc_pitch_at(progress),
		)
	}

	/// Advances the hunt by one tick and returns what happened in it.
	pub fn step(&mut self, input: &HuntInput) -> Vec<HuntEvent> {
		if self.is_over() {
			return Vec::new();
		}

		let mut events = Vec::new();

		match self.phase {
			HuntPhase::Stalk => self.step_stalk(input, &mut events),
			HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Shriek | HuntPhase::Charge => {
				self.step_fight(input, &mut events);
			}
			HuntPhase::Broken => self.step_reap(input, &mut events),
			HuntPhase::Absorbed | HuntPhase::Escaped => {}
		}

		self.tick += 1;

		events
	}

	// Stalk --------------------------------------------------------------

	fn step_stalk(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		self.move_husk(events);

		// An art thrown at a husk that has not noticed the reaper goes
		// unseen until it lands. Then the husk knows, and the ambush is
		// given up.
		self.step_flights(events);

		if self.phase != HuntPhase::Stalk {
			return;
		}

		self.step_art(input, events);

		let current = self.pose();
		let speed = input.turn_radians_per_second();
		let struck = input
			.taps
			.iter()
			.any(|tap| current.covers(tap.direction(), self.stats.reach_radians));

		// A reaper who strikes first has given up the ambush, and the strike
		// still counts.
		if struck {
			self.begin_fight(events, false);
			self.step_strikes(input, events);
			return;
		}

		if speed > HuntRules::STARTLE_SPEED {
			self.awareness += (speed - HuntRules::STARTLE_SPEED) * HuntRules::STARTLE_GAIN
				/ HuntRules::TICK_RATE as f64;
		} else {
			self.awareness = math::max(
				0.0,
				self.awareness - HuntRules::STARTLE_DECAY / HuntRules::TICK_RATE as f64,
			);
		}

		let on_target =
			input.aim.direction().angle_to(current.direction()) <= HuntRules::AMBUSH_AIM;

		if on_target && speed <= HuntRules::STEADY_SPEED {
			self.steady_ticks += 1;
		} else {
			self.steady_ticks = (self.steady_ticks - 2).max(0);
		}

		if self.steady_ticks >= HuntRules::AMBUSH_TICKS {
			self.begin_fight(events, true);
			return;
		}

		if self.awareness >= 1.0 || self.tick >= HuntRules::STALK_LIMIT_TICKS {
			self.awareness = 1.0;
			self.begin_fight(events, false);
		}
	}

	fn begin_fight(&mut self, events: &mut Vec<HuntEvent>, ambush: bool) {
		self.phase = HuntPhase::Fight;
		self.ambushed = ambush;
		self.ambush_strike_ready = ambush;
		self.schedule_shriek();
		self.schedule_charge();
		events.push(HuntEvent::StalkEnded { ambush });

		if ambush {
			self.expose_all_seals(events);

			match self.stats.gift {
				Some(ReaperGift::Passage) => {
					self.passage_ready = true;
					events.push(HuntEvent::GiftStirred(ReaperGift::Passage));
				}
				Some(ReaperGift::Foreknowledge) => {
					for until in &mut self.open_until {
						*until += SoulRules::FOREKNOWLEDGE_TICKS_PER_STRENGTH
							* self.stats.gift_strength();
					}
					events.push(HuntEvent::GiftStirred(ReaperGift::Foreknowledge));
				}
				Some(ReaperGift::Hunger) => {
					self.cadence = self.stats.gift_strength();
					self.last_hit_tick = self.tick;
					events.push(HuntEvent::GiftStirred(ReaperGift::Hunger));
				}
				Some(ReaperGift::Hush) => {
					self.next_shriek_tick +=
						SoulRules::HUSH_TICKS_PER_STRENGTH * self.stats.gift_strength();
					events.push(HuntEvent::GiftStirred(ReaperGift::Hush));
				}
				_ => {}
			}

			return;
		}

		// It has seen the reaper, and it runs, unless it is in no state to.
		if !self.leaping && self.tick >= self.frozen_until && self.tick >= self.staggered_until {
			self.bolt(events);
		}
	}

	// Fight --------------------------------------------------------------

	fn fighting(&self) -> bool {
		matches!(
			self.phase,
			HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Shriek | HuntPhase::Charge
		)
	}

	fn step_fight(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		self.move_husk(events);
		self.fight_ticks += 1;

		if self.fight_ticks > self.fight_limit_ticks() {
			self.escape(events, EscapeCause::Time);
			return;
		}

		if self.tick - self.last_hit_tick > HuntRules::CADENCE_WINDOW_TICKS {
			self.cadence = 0;
			self.hit_streak = 0;
		}

		self.step_burn();
		self.step_flights(events);

		if self.ward <= 0.0 {
			self.break_ward(events);
			return;
		}

		self.step_shriek(input, events);

		if self.is_over() || self.phase == HuntPhase::Shriek {
			self.step_art(input, events);
			return;
		}

		self.step_charge(input, events);

		if self.is_over() {
			return;
		}

		self.step_art(input, events);
		self.step_seals(events);
		self.step_strikes(input, events);

		if self.ward <= 0.0 {
			self.break_ward(events);
		}
	}

	fn step_shriek(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		// A husk in mid-leap lands before it draws breath.
		if self.phase == HuntPhase::Fight && self.tick >= self.next_shriek_tick && !self.leaping {
			self.echo_cadence = self.cadence;
			self.phase = HuntPhase::Windup;
			self.phase_end_tick = self.tick + HuntRules::WINDUP_TICKS;
			events.push(HuntEvent::ShriekWindup);

			// A husk shrieks at the reaper's face, not from behind the sofa:
			// it comes out over the top as it draws breath.
			if self.footing == Footing::Hidden && !self.leaping {
				self.leave_cover(events, false);
			}

			return;
		}

		if self.phase == HuntPhase::Windup && self.tick >= self.phase_end_tick && !self.leaping {
			self.phase = HuntPhase::Shriek;
			self.shriek_start_tick = self.tick;
			self.phase_end_tick = self.tick + HuntRules::SHRIEK_TICKS;
			self.flinched_this_shriek = false;
			self.buffered_tap = None;
			let tick = self.tick;
			self.open_until.fill(tick);
			events.push(HuntEvent::ShriekBegan);
			return;
		}

		if self.phase != HuntPhase::Shriek {
			return;
		}

		let past_grace = self.tick - self.shriek_start_tick >= HuntRules::SHRIEK_GRACE_TICKS;
		let moved =
			input.turn_radians_per_second() > self.stats.flinch_speed || !input.taps.is_empty();

		if past_grace && moved && !self.flinched_this_shriek {
			self.flinch(events);
		}

		if self.is_over() || self.tick < self.phase_end_tick {
			return;
		}

		self.phase = HuntPhase::Fight;
		self.schedule_shriek();
		// Having just been told to hold still, the reaper is given a moment
		// before being told to swing.
		if self.next_charge_tick >= 0 {
			self.next_charge_tick = self
				.next_charge_tick
				.max(self.tick + HuntRules::CHARGE_CLEAR_TICKS);
		}
		self.next_bolt_tick = self.tick
			+ HuntRules::BOLT_DELAY_MIN_TICKS
			+ self
				.path
				.next_int(HuntRules::BOLT_DELAY_MAX_TICKS - HuntRules::BOLT_DELAY_MIN_TICKS + 1);

		if self.flinched_this_shriek {
			return;
		}

		events.push(HuntEvent::HeldFirm);
		self.holds += 1;
		self.expose_all_seals(events);

		// Stillness steadies the hand: an unflinching hold readies a throw.
		if let Some(art) = self.stats.art
			&& self.art_charges < art.max_charges()
		{
			self.art_charges += 1;
		}

		if self.stats.perk == ClassPerk::ChosenBlow {
			self.chosen_blow_ready = true;
		}

		if self.stats.perk == ClassPerk::Toll {
			self.bonus_capture_ticks += HuntRules::TOLL_TICKS;
		}

		match self.stats.gift {
			Some(ReaperGift::Aftermath | ReaperGift::Verdict | ReaperGift::Keening) => {
				self.gift_strike_ready = true;
			}
			Some(ReaperGift::Anchor) => {
				if self.gift_uses < self.stats.gift_strength() && self.resolve < self.stats.resolve
				{
					self.resolve += 1;
					self.gift_uses += 1;
					events.push(HuntEvent::GiftStirred(ReaperGift::Anchor));
				}
			}
			Some(ReaperGift::Echo) => {
				self.cadence = HuntRules::CHAIN_CADENCE_CAP
					.min(self.echo_cadence + self.stats.gift_strength());
				self.last_hit_tick = self.tick;
				events.push(HuntEvent::GiftStirred(ReaperGift::Echo));
			}
			_ => {}
		}
	}

	fn flinch(&mut self, events: &mut Vec<HuntEvent>) {
		self.flinched_this_shriek = true;
		self.cadence = 0;
		self.hit_streak = 0;

		let steadfast = self.stats.perk == ClassPerk::Steadfast && !self.steadfast_spent;
		let returned = !steadfast && self.return_ready;
		let forgiven = steadfast || returned;

		if steadfast {
			self.steadfast_spent = true;
		} else if returned {
			self.return_ready = false;
			events.push(HuntEvent::GiftStirred(ReaperGift::ReturnLight));
		} else {
			self.resolve -= 1;
		}

		events.push(HuntEvent::Flinched {
			resolve_left: self.resolve,
			forgiven,
		});

		if self.resolve <= 0 {
			self.escape(events, EscapeCause::Resolve);
		}
	}

	fn schedule_shriek(&mut self) {
		let gap = HuntRules::SHRIEK_GAP_MAX_TICKS - HuntRules::SHRIEK_GAP_MIN_TICKS;

		self.next_shriek_tick =
			self.tick + HuntRules::SHRIEK_GAP_MIN_TICKS + self.shrieks.next_int(gap + 1);
	}

	// Charge -------------------------------------------------------------

	/// How far through its charge the husk is, from 0 to 1. Zero when it is
	/// not charging.
	pub fn charge_progress(&self) -> f64 {
		if self.phase == HuntPhase::Charge {
			math::clamp(
				(self.tick - self.charge_start_tick) as f64 / self.charge_ticks() as f64,
				0.0,
				1.0,
			)
		} else {
			0.0
		}
	}

	/// Whether the husk stands stunned after a charge was turned aside.
	pub fn staggered(&self) -> bool {
		self.fighting() && self.tick < self.staggered_until
	}

	fn charge_ticks(&self) -> i64 {
		math::round(HuntRules::CHARGE_TICKS as f64 * self.difficulty.tell())
	}

	/// Whether this husk charges at all: one that is rooted to the spot does
	/// not, and neither does a rift's, which is not any one reaper's to
	/// face.
	fn may_charge(&self) -> bool {
		!self.in_rift && self.vessel.gait() != Gait::Still
	}

	fn schedule_charge(&mut self) {
		if !self.may_charge() {
			return;
		}

		let gap = HuntRules::CHARGE_GAP_MAX_TICKS - HuntRules::CHARGE_GAP_MIN_TICKS;

		self.next_charge_tick =
			self.tick + HuntRules::CHARGE_GAP_MIN_TICKS + self.charges.next_int(gap + 1);
	}

	fn step_charge(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		if self.phase == HuntPhase::Charge {
			let swinging = self.tick - self.charge_start_tick >= HuntRules::CHARGE_GRACE_TICKS
				&& input.turn_radians_per_second() >= self.stats.parry_speed;

			if swinging {
				self.parry(events);
			} else if self.tick >= self.phase_end_tick {
				self.strike_reaper(events);
			}

			return;
		}

		let due = self.phase == HuntPhase::Fight
			&& self.next_charge_tick >= 0
			&& self.tick >= self.next_charge_tick
			&& self.tick >= self.staggered_until
			&& self.tick >= self.frozen_until;

		if !due || self.leaping {
			return;
		}

		// It has to be out in the open to run at anyone: it comes down or
		// out first, and the charge waits for it to land.
		if self.footing != Footing::Open {
			self.leave_cover(events, false);
			return;
		}

		// With a shriek about to start, the charge waits until it is over.
		if self.next_shriek_tick - self.tick < self.charge_ticks() + HuntRules::CHARGE_CLEAR_TICKS {
			return;
		}

		self.phase = HuntPhase::Charge;
		self.charge_start_tick = self.tick;
		self.phase_end_tick = self.tick + self.charge_ticks();
		self.next_bolt_tick = -1;

		// Straight at the reaper, to within arm's reach. One in the air
		// comes down to eye level as it comes.
		self.from_bearing = self.bearing;
		self.from_range = self.range;
		self.from_height = self.height;
		self.by_bearing = 0.0;
		self.by_range = math::min(0.0, HuntRules::CHARGE_METRES - self.range);
		self.by_height = if self.vessel.gait() == Gait::Air {
			-self.height
		} else {
			0.0
		};
		self.move_start = self.tick;
		self.move_ticks = self.charge_ticks();
		events.push(HuntEvent::ChargeBegan);
	}

	/// The reaper swung in time. The husk is knocked out of its run and
	/// stands stunned where it got to, every seal in view open.
	fn parry(&mut self, events: &mut Vec<HuntEvent>) {
		self.end_charge();
		self.move_start = -1;
		self.staggered_until = self.tick + HuntRules::STAGGER_TICKS;
		self.next_bolt_tick = self.staggered_until;
		events.push(HuntEvent::Parried);
		self.parries += 1;
		self.expose_all_seals(events);

		if self.stats.gift == Some(ReaperGift::Reprisal) {
			self.chosen_blow_ready = true;
			self.gift_strike_ready = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Reprisal));
		}
	}

	/// The charge got through. It costs the reaper resolve, and the husk is
	/// off again at once.
	fn strike_reaper(&mut self, events: &mut Vec<HuntEvent>) {
		if self.passage_ready {
			self.passage_ready = false;
			self.parry(events);
			// This is the gift's protection, not an input the player
			// performed.
			self.parries -= 1;
			self.staggered_until = self.tick
				+ SoulRules::PASSAGE_STAGGER_TICKS_PER_STRENGTH * self.stats.gift_strength();
			self.next_bolt_tick = self.staggered_until;
			events.push(HuntEvent::GiftStirred(ReaperGift::Passage));
			return;
		}

		self.end_charge();
		self.resolve -= 1;
		self.cadence = 0;
		self.hit_streak = 0;
		events.push(HuntEvent::Struck {
			resolve_left: self.resolve,
		});

		if self.resolve <= 0 {
			self.escape(events, EscapeCause::Resolve);
			return;
		}

		self.bolt(events);
	}

	fn end_charge(&mut self) {
		self.phase = HuntPhase::Fight;
		self.schedule_charge();
		// A shriek does not come straight on top of a charge.
		self.next_shriek_tick = self
			.next_shriek_tick
			.max(self.tick + HuntRules::CHARGE_CLEAR_TICKS);
	}

	fn step_seals(&mut self, events: &mut Vec<HuntEvent>) {
		let current = self.pose();
		let mut open = (0..self.seal_count())
			.filter(|&index| self.seal_open(index))
			.count();

		while open < HuntRules::OPEN_SEALS.min(self.seal_count()) {
			// A seal only opens where the reaper can see it, so a turning
			// body lights its seals as they come round.
			let ready: Vec<usize> = (0..self.seal_count())
				.filter(|&index| {
					self.tick >= self.open_until[index]
						&& self.tick >= self.shut_until[index]
						&& current.seal_faces_player(index)
						&& !self.behind_cover(&current, index)
				})
				.collect();

			if ready.is_empty() {
				return;
			}

			let seal = self.schedule.pick(&ready);
			let ticks = 150 + self.schedule.next_int(110);

			self.open_seal(seal, ticks, events);
			open += 1;
		}
	}

	fn open_seal(&mut self, index: usize, ticks: i64, events: &mut Vec<HuntEvent>) {
		let was_shut = self.tick >= self.open_until[index];

		self.open_until[index] = self.tick + ticks;

		if !was_shut {
			return;
		}

		self.opened_order[index] = self.opened_count;
		self.opened_count += 1;
		events.push(HuntEvent::SealOpened(index));
	}

	fn expose_all_seals(&mut self, events: &mut Vec<HuntEvent>) {
		for index in 0..self.seal_count() {
			self.open_seal(index, HuntRules::EXPOSED_TICKS, events);
		}
	}

	fn step_strikes(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		let buffered = self.buffered_tap;

		if buffered.is_some() && self.tick - self.buffered_at > HuntRules::BUFFER_TICKS {
			self.buffered_tap = None;
		}

		if let Some(buffered) = buffered
			&& self.buffered_tap.is_some()
			&& self.tick >= self.next_strike_tick
		{
			self.buffered_tap = None;
			self.resolve_strike(buffered, events);
		}

		for &tap in &input.taps {
			if self.tick >= self.next_strike_tick {
				self.resolve_strike(tap, events);
				continue;
			}

			// One slot: a later tap replaces an earlier one that is still
			// waiting.
			self.buffered_tap = Some(tap);
			self.buffered_at = self.tick;
		}
	}

	#[allow(clippy::too_many_lines)]
	fn resolve_strike(&mut self, tap: Aim, events: &mut Vec<HuntEvent>) {
		let ray = tap.direction();
		let current = self.pose();

		self.next_strike_tick = self.tick + self.stats.swing_ticks;

		let Some(seal) = self.nearest_open_seal(ray, &current, self.stats.reach_radians) else {
			self.resolve_miss(ray, &current, events);
			return;
		};

		let gift = self.stats.gift;
		let strength = self.stats.gift_strength() as f64;
		let cap = if self.stats.perk == ClassPerk::Chain {
			HuntRules::CHAIN_CADENCE_CAP
		} else {
			HuntRules::CADENCE_CAP
		};
		let mut scale = 1.0 + self.cadence.min(cap) as f64 * HuntRules::CADENCE_STEP;
		let mut empowered = false;
		let first_strike = self.ambush_strike_ready;
		let in_order = self.seal_rank(seal) == Some(0);
		let aftermath = self.gift_strike_ready && gift == Some(ReaperGift::Aftermath);

		if self.ambush_strike_ready {
			self.ambush_strike_ready = false;
			scale *= HuntRules::AMBUSH_STRIKE;
			empowered = true;
		}

		if first_strike && gift == Some(ReaperGift::FirstLight) {
			scale *= 1.0 + SoulRules::GIFT_STRIKE_PER_STRENGTH * strength;
			events.push(HuntEvent::GiftStirred(ReaperGift::FirstLight));
		}

		if self.chosen_blow_ready {
			self.chosen_blow_ready = false;
			scale *= HuntRules::CHOSEN_BLOW_STRIKE;
			empowered = true;
		}

		if self.stats.perk == ClassPerk::Measure && in_order {
			scale *= HuntRules::MEASURE_STRIKE;
			empowered = true;
		}

		let verdict = gift == Some(ReaperGift::Verdict) && in_order;
		let promised = self.gift_strike_ready
			&& (verdict || matches!(gift, Some(ReaperGift::Keening | ReaperGift::Reprisal)));
		let execution = gift == Some(ReaperGift::LastWord)
			&& self.cadence >= SoulRules::EXECUTION_CADENCE
			&& self.ward <= self.ward_max * SoulRules::EXECUTION_WARD_SHARE;

		if (promised || execution)
			&& let Some(gift) = gift
		{
			scale *= 1.0 + SoulRules::GIFT_STRIKE_PER_STRENGTH * strength;
			empowered = true;
			if promised {
				self.gift_strike_ready = false;
			}
			events.push(HuntEvent::GiftStirred(gift));
		}

		if aftermath {
			self.gift_strike_ready = false;
			empowered = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Aftermath));
		}

		if self.frozen() {
			scale *= self.stats.art.map_or(1.0, |art| art.shatter());
			empowered = true;
			self.frozen_until = self.tick;
			events.push(HuntEvent::IceShattered);
		}

		let mut damage = self.stats.strike_damage * scale;
		let mut cleaved = Vec::new();

		self.shut_seal(seal);

		if self.stats.perk == ClassPerk::Cleave {
			let reach = self.stats.reach_radians * HuntRules::CLEAVE_REACH;

			for other in 0..self.seal_count() {
				let caught =
					self.seal_open(other) && ray.angle_to(current.seal_direction(other)) <= reach;

				if !caught {
					continue;
				}

				if aftermath {
					self.open_until[other] = self.open_until[other].max(
						self.tick
							+ HuntRules::EXPOSED_TICKS
							+ SoulRules::GATHERING_TICKS_PER_STRENGTH * self.stats.gift_strength(),
					);
				} else {
					self.shut_seal(other);
				}
				cleaved.push(other);
				let share = HuntRules::CLEAVE_SHARE
					+ if first_strike && gift == Some(ReaperGift::Furrow) {
						SoulRules::FURROW_CLEAVE_PER_STRENGTH * strength
					} else {
						0.0
					};
				damage += self.stats.strike_damage * scale * share;
			}
		}

		if first_strike && gift == Some(ReaperGift::Furrow) && !cleaved.is_empty() {
			empowered = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Furrow));
		}

		self.ward = math::max(0.0, self.ward - damage);
		self.knock();
		self.cadence += 1;
		self.hit_streak += 1;
		self.max_hit_streak = self.max_hit_streak.max(self.hit_streak);
		self.last_hit_tick = self.tick;
		events.push(HuntEvent::StrikeLanded {
			seal,
			damage,
			cadence: self.cadence,
			empowered,
			cleaved,
		});
		self.after_gift_strike(seal, in_order, events);
	}

	/// Conditional powers keep their own counters, never a second
	/// simulation.
	fn after_gift_strike(&mut self, seal: usize, in_order: bool, events: &mut Vec<HuntEvent>) {
		match self.stats.gift {
			Some(ReaperGift::Gathering) => {
				if self.cadence % SoulRules::GATHERING_HITS == 0 {
					self.shut_until[seal] = self.tick;
					self.open_seal(
						seal,
						HuntRules::SEAL_COOLDOWN_TICKS
							+ SoulRules::GATHERING_TICKS_PER_STRENGTH * self.stats.gift_strength(),
						events,
					);
					events.push(HuntEvent::GiftStirred(ReaperGift::Gathering));
				}
			}
			Some(ReaperGift::ReturnLight) => {
				if self.cadence >= SoulRules::RETURN_CADENCE
					&& !self.return_ready
					&& self.gift_uses < self.stats.gift_strength()
				{
					self.return_ready = true;
					self.gift_uses += 1;
					events.push(HuntEvent::GiftStirred(ReaperGift::ReturnLight));
				}
			}
			Some(ReaperGift::Equilibrium) => {
				self.ordered_hits = if in_order { self.ordered_hits + 1 } else { 0 };
				if self.ordered_hits == SoulRules::EQUILIBRIUM_HITS {
					self.ordered_hits = 0;
					for other in 0..self.seal_count() {
						self.shut_until[other] = self.tick;
						let ticks = (self.open_until[other] - self.tick).max(
							HuntRules::SEAL_COOLDOWN_TICKS
								+ SoulRules::EQUILIBRIUM_TICKS_PER_STRENGTH
									* self.stats.gift_strength(),
						);
						self.open_seal(other, ticks, events);
					}
					events.push(HuntEvent::GiftStirred(ReaperGift::Equilibrium));
				}
			}
			Some(ReaperGift::Refrain)
				if self.cadence % SoulRules::REFRAIN_HITS == 0
					&& self.refrain_tolls < SoulRules::REFRAIN_MAX_TOLLS =>
			{
				self.bonus_capture_ticks +=
					SoulRules::REFRAIN_TICKS_PER_STRENGTH * self.stats.gift_strength();
				self.refrain_tolls += 1;
				events.push(HuntEvent::GiftStirred(ReaperGift::Refrain));
			}
			_ => {}
		}
	}

	/// A heavy blade stops the husk in its tracks.
	fn knock(&mut self) {
		if self.stats.knock_ticks == 0 || self.phase != HuntPhase::Fight || self.leaping {
			return;
		}

		self.staggered_until = self.staggered_until.max(self.tick + self.stats.knock_ticks);
		self.halt();
	}

	fn resolve_miss(&mut self, ray: Vec3, current: &HuskPose, events: &mut Vec<HuntEvent>) {
		// A blow at the part of a husk that is behind the counter hits the
		// counter.
		let blocked = self.plan.hides(ray * current.distance());

		if blocked || !current.covers(ray, self.stats.reach_radians * 0.5) {
			self.hit_streak = 0;
			if self.stats.gift == Some(ReaperGift::Unbroken)
				&& self.cadence > 0
				&& self.gift_uses < self.stats.gift_strength()
			{
				self.gift_uses += 1;
				events.push(HuntEvent::GiftStirred(ReaperGift::Unbroken));
			} else {
				self.cadence = 0;
			}
			events.push(HuntEvent::StrikeWhiffed);
			return;
		}

		let damage = self.stats.strike_damage * self.stats.glance_share;

		self.ward = math::max(0.0, self.ward - damage);
		events.push(HuntEvent::StrikeGlanced { damage });
	}

	fn nearest_open_seal(&self, ray: Vec3, current: &HuskPose, reach: f64) -> Option<usize> {
		let mut nearest = None;
		let mut nearest_angle = reach;

		for index in 0..self.seal_count() {
			if !self.seal_open(index) {
				continue;
			}

			let angle = ray.angle_to(current.seal_direction(index));

			if angle <= nearest_angle {
				nearest = Some(index);
				nearest_angle = angle;
			}
		}

		nearest
	}

	fn shut_seal(&mut self, index: usize) {
		self.open_until[index] = self.tick;
		self.shut_until[index] = self.tick + HuntRules::SEAL_COOLDOWN_TICKS;
	}

	// Break and reap -----------------------------------------------------

	fn break_ward(&mut self, events: &mut Vec<HuntEvent>) {
		// A husk broken in mid-leap still lands, and hangs open where it
		// does.
		let landing = if self.leaping {
			self.destination()
		} else {
			self.pose().position
		};

		self.phase = HuntPhase::Broken;
		self.ward = 0.0;
		self.halt();
		self.buffered_tap = None;
		self.flights.clear();
		self.burn_until = self.tick;
		self.frozen_until = self.tick;
		self.arc_yaw = landing.yaw();
		self.arc_pitch = landing.pitch();
		self.capture_end_tick = self.tick + self.capture_ticks();
		self.reap_begun = false;
		self.reap_progress = 0.0;
		self.deviation_sum = 0.0;
		self.reap_ticks = 0;
		self.fast_ticks = 0;
		events.push(HuntEvent::Broken);
	}

	fn step_reap(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		let aim = input.aim.direction();

		if self.leaping {
			self.continue_move();
		}

		if !self.reap_begun && aim.angle_to(self.reap_arc_point(0.0)) <= HuntRules::REAP_START_AIM {
			self.reap_begun = true;
			events.push(HuntEvent::ReapBegan);
		}

		if self.reap_begun {
			self.track_cut(aim, input.turn_radians_per_second());
		}

		if self.reap_progress >= 1.0 {
			self.absorb(events);
			return;
		}

		if self.tick < self.capture_end_tick {
			return;
		}

		self.reap_attempts_left -= 1;

		if self.reap_attempts_left <= 0 {
			self.escape(events, EscapeCause::Reap);
			return;
		}

		self.phase = HuntPhase::Fight;
		self.ward = self.ward_max * HuntRules::RECOVER_SHARE;
		self.cadence = 0;
		// It pulls itself together and is off at once.
		self.next_bolt_tick = self.tick + 30;
		self.schedule_shriek();
		self.schedule_charge();
		events.push(HuntEvent::ReapRecovered {
			attempts_left: self.reap_attempts_left,
		});
	}

	fn track_cut(&mut self, aim: Vec3, speed: f64) {
		let along = wrap_angle(aim.yaw() - self.arc_yaw) * self.reap_direction
			/ (2.0 * HuntRules::REAP_HALF_SPAN)
			+ 0.5;
		let clamped = math::clamp(along, 0.0, 1.0);

		// The cut only ever moves forward: drifting back loses nothing.
		self.reap_progress = math::max(self.reap_progress, clamped);
		self.deviation_sum += (aim.pitch() - self.arc_pitch_at(clamped)).abs();
		self.reap_ticks += 1;

		if speed > HuntRules::REAP_SPEED_LIMIT {
			self.fast_ticks += 1;
		}
	}

	fn arc_pitch_at(&self, progress: f64) -> f64 {
		self.arc_pitch + radians(5.0) * 4.0 * progress * (1.0 - progress) - radians(2.5)
	}

	fn absorb(&mut self, events: &mut Vec<HuntEvent>) {
		let deviation = self.deviation_sum / self.reap_ticks as f64;
		let fast_share = self.fast_ticks as f64 / self.reap_ticks as f64;
		let tolerance = self.stats.reap_tolerance_radians;

		let grade = if deviation > tolerance || fast_share > 0.2 {
			ReapGrade::Ragged
		} else if deviation <= tolerance * HuntRules::PERFECT_SHARE && fast_share <= 0.05 {
			ReapGrade::Perfect
		} else {
			ReapGrade::Clean
		};

		self.phase = HuntPhase::Absorbed;
		self.outcome = Some(HuntOutcome::absorbed_with(
			grade,
			self.ambushed,
			self.tick + 1,
			self.deeds(),
		));
		events.push(HuntEvent::HuskAbsorbed(grade));
	}

	fn escape(&mut self, events: &mut Vec<HuntEvent>, cause: EscapeCause) {
		self.phase = HuntPhase::Escaped;
		self.outcome = Some(HuntOutcome::escaped_by(
			cause,
			self.ambushed,
			self.tick + 1,
			self.deeds(),
		));
		events.push(HuntEvent::HuskEscaped(cause));
	}

	// Arts ---------------------------------------------------------------

	/// Gathers, throws and readies the reaper's art.
	///
	/// A thumb held on the art gathers it while the phone is steady, and the
	/// art goes wherever the phone was last steady on. Once it is gathered,
	/// a flick of the phone throws it. Letting go before that loses nothing.
	fn step_art(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		let Some(art) = self.stats.art else {
			return;
		};

		if self.art_charges < art.max_charges() {
			self.recharge_ticks += 1;

			if self.recharge_ticks >= art.recharge_ticks() {
				self.art_charges += 1;
				self.recharge_ticks = 0;
			}
		} else {
			self.recharge_ticks = 0;
		}

		if !input.focus || self.art_charges == 0 {
			self.gather_ticks = 0;
			self.cast_aim = None;
			return;
		}

		let speed = input.turn_radians_per_second();
		let gathered = self.gather_ticks >= ArtRules::GATHER_TICKS;

		// Nothing is thrown into a shriek: the flick is a flinch, and the
		// art stays gathered in the hand.
		if gathered
			&& let Some(aim) = self.cast_aim
			&& speed >= ArtRules::THROW_SPEED
		{
			if self.phase == HuntPhase::Shriek {
				return;
			}

			self.throw(art, aim, events);
			return;
		}

		if speed <= HuntRules::STEADY_SPEED {
			self.gather_ticks = ArtRules::GATHER_TICKS.min(self.gather_ticks + 1);
			self.cast_aim = Some(input.aim);
		}
	}

	fn throw(&mut self, art: ArtSkill, aim: Aim, events: &mut Vec<HuntEvent>) {
		let ray = aim.direction();
		let current = self.pose();
		let at_husk = match art.art {
			Art::Gale => ray.angle_to(current.direction()) <= ArtRules::GUST_ANGLE,
			Art::Cinder | Art::Rime => current.covers(ray, self.stats.reach_radians),
		};
		let from = Vec3::new(0.0, -ArtRules::HAND_DROP, 0.0);
		let target = if at_husk {
			current.position
		} else {
			self.landing_along(ray)
		};

		self.art_charges -= 1;
		self.gather_ticks = 0;
		self.cast_aim = None;
		self.flights.push(ArtFlight {
			art: art.art,
			from,
			target,
			homing: at_husk && art.art.homing(),
			thrown_at: self.tick,
			ticks: ArtRules::SHORTEST_FLIGHT_TICKS.max(math::round(
				(target - from).length() / art.art.speed() * HuntRules::TICK_RATE as f64,
			)),
		});
		events.push(HuntEvent::ArtThrown(art.art));
	}

	/// Where an art thrown along `ray` at nothing comes down: on the floor,
	/// or against the front of whatever stands in the way, or at the end of
	/// its throw. Thrown over the top of something, it comes down behind it.
	fn landing_along(&self, ray: Vec3) -> Vec3 {
		let mut metres = ArtRules::FARTHEST_METRES;

		if ray.y < 0.0 {
			metres = math::min(metres, -self.plan.eye_height() / ray.y);
		}

		let across = math::sqrt(ray.x * ray.x + ray.z * ray.z);
		let cover = self.plan.cover_at(ray.yaw());

		if cover > 0.0 && across > 0.0 {
			let front = self.plan.front_at(HuntPlan::slice_of(ray.yaw())) / across;

			if front < metres && ray.y * front < cover - self.plan.eye_height() {
				metres = front;
			}
		}

		ray * metres
	}

	fn step_flights(&mut self, events: &mut Vec<HuntEvent>) {
		if self.flights.is_empty() {
			return;
		}

		let tick = self.tick;
		let (landing, flying): (Vec<ArtFlight>, Vec<ArtFlight>) = self
			.flights
			.iter()
			.partition(|flight| tick >= flight.lands_at());

		self.flights = flying;

		for flight in landing {
			self.art_lands(&flight, events);
		}
	}

	fn art_lands(&mut self, flight: &ArtFlight, events: &mut Vec<HuntEvent>) {
		let Some(art) = self.stats.art else {
			return;
		};
		let current = self.pose();
		let at = if flight.homing {
			current.position
		} else {
			flight.target
		};
		let hit = match flight.art {
			// Fire bursts where it comes down, whatever is in front of it: it
			// was lobbed over the top.
			Art::Cinder => {
				(current.position - at).length()
					<= self.vessel.half_width() + ArtRules::BURST_METRES
			}
			// Ice flies straight, and breaks on whatever the husk is behind.
			Art::Rime => flight.homing && !self.plan.hides(current.position),
			// Wind goes round things.
			Art::Gale => flight.homing,
		};

		if !hit {
			events.push(HuntEvent::ArtLanded {
				art: flight.art,
				at,
				hit: false,
				damage: 0.0,
			});

			if self.phase == HuntPhase::Stalk {
				self.begin_fight(events, false);
			}

			return;
		}

		let damage = self.stats.strike_damage * art.strike();

		self.ward = math::max(0.0, self.ward - damage);
		events.push(HuntEvent::ArtLanded {
			art: flight.art,
			at,
			hit: true,
			damage,
		});

		match flight.art {
			Art::Cinder => {
				self.burn_until = self.tick + art.burn_ticks();
				self.burn_per_tick =
					self.stats.strike_damage * art.burn_strike() / art.burn_ticks() as f64;
				self.frozen_until = self.tick;
				self.exposed_until = self.exposed_until.max(self.burn_until);

				// It runs, burning, and not into cover.
				if self.phase == HuntPhase::Fight && self.tick >= self.staggered_until {
					self.next_bolt_tick = self.tick + ArtRules::BURN_BOLT_TICKS;
				}
			}
			Art::Rime => self.freeze(art, events),
			Art::Gale => self.gust(art, events),
		}

		if self.phase == HuntPhase::Stalk {
			self.begin_fight(events, false);
		}
	}

	fn step_burn(&mut self) {
		if self.tick < self.burn_until {
			self.ward = math::max(0.0, self.ward - self.burn_per_tick);
		}
	}

	/// Ice holds the husk where it stands, every seal in view open. A charge
	/// stops dead.
	fn freeze(&mut self, art: ArtSkill, events: &mut Vec<HuntEvent>) {
		if self.phase == HuntPhase::Charge {
			self.end_charge();
			self.move_start = -1;
		}

		self.frozen_until = self.tick + art.freeze_ticks();
		self.burn_until = self.tick;
		self.halt();

		for index in 0..self.seal_count() {
			self.open_seal(index, art.freeze_ticks(), events);
		}
	}

	/// Wind knocks the husk off its ledge or out from behind its cover,
	/// turns a charge aside, and leaves it stunned, every seal in view open.
	fn gust(&mut self, art: ArtSkill, events: &mut Vec<HuntEvent>) {
		if self.phase == HuntPhase::Charge {
			self.end_charge();
			self.move_start = -1;
			self.staggered_until = self.tick + art.gust_stun_ticks();
		} else if self.leaping {
			self.staggered_until = self.move_start + self.move_ticks + art.gust_stun_ticks();
		} else if self.footing != Footing::Open {
			self.leave_cover(events, false);
			self.staggered_until = self.tick + self.move_ticks + art.gust_stun_ticks();
		} else {
			self.halt();
			self.staggered_until = self.tick + art.gust_stun_ticks();
		}

		self.next_bolt_tick = self.staggered_until;
		self.exposed_until = self.staggered_until + ArtRules::SHAKEN_TICKS;
		self.expose_all_seals(events);
	}

	// Movement -----------------------------------------------------------

	/// Moves the husk on by one tick: along the move it is making, or into a
	/// new one when it is time to amble or to bolt.
	fn move_husk(&mut self, events: &mut Vec<HuntEvent>) {
		if self.vessel.gait() == Gait::Still {
			return;
		}

		// A leap, once begun, comes down, whatever else is happening.
		if self.leaping {
			self.continue_move();
			return;
		}

		// Frozen, it does not even turn.
		if self.tick < self.frozen_until {
			self.halt();
			return;
		}

		// Drawing breath, a husk stops where it is and turns on the reaper.
		// Stunned, it does the same.
		if self.phase == HuntPhase::Windup
			|| self.phase == HuntPhase::Shriek
			|| self.tick < self.staggered_until
		{
			self.halt();
			self.turn_toward(self.bearing + PI);
			return;
		}

		// Charging, it comes straight on with its face to the reaper.
		if self.phase == HuntPhase::Charge {
			let progress = math::clamp(
				(self.tick - self.move_start) as f64 / self.move_ticks as f64,
				0.0,
				1.0,
			);

			self.range = self.from_range + self.by_range * progress * progress;
			self.height = self.from_height + self.by_height * progress;
			self.turn_toward(self.bearing + PI);
			return;
		}

		if self.move_start >= 0 {
			self.continue_move();
			return;
		}

		let bolting = self.phase == HuntPhase::Fight
			&& self.next_bolt_tick >= 0
			&& self.tick >= self.next_bolt_tick;

		if bolting {
			self.bolt(events);
		} else if self.tick >= self.next_amble_tick {
			self.amble(events);
		}
	}

	fn continue_move(&mut self) {
		let progress = (self.tick - self.move_start) as f64 / self.move_ticks as f64;

		if progress >= 1.0 {
			self.bearing = wrap_angle(self.from_bearing + self.by_bearing);
			self.range = if self.leaping {
				self.from_range + self.by_range
			} else {
				math::min(self.from_range + self.by_range, self.far_at(self.bearing))
			};
			self.height = self.from_height + self.by_height;
			self.move_start = -1;

			if self.leaping {
				self.land();
			}

			self.next_amble_tick = self.tick + self.rest_ticks();
			return;
		}

		let eased = smoothstep(progress);
		let along = self.from_range + self.by_range * eased;

		self.bearing = wrap_angle(self.from_bearing + self.by_bearing * eased);
		// Whatever stands between where it set off and where it is going, it
		// keeps to this side of: it goes round the front of the sofa, not
		// through it. A leap goes over the top instead.
		self.range = if self.leaping {
			along
		} else {
			math::min(along, self.far_at(self.bearing))
		};
		self.height =
			self.from_height + self.by_height * eased + self.arc * 4.0 * eased * (1.0 - eased);

		// The body faces the way it is going: along the floor, the sum of
		// going round the player and going toward or away from them.
		let round = self.range * self.by_bearing;

		self.turn_toward(math::atan2(
			self.by_range * math::sin(self.bearing) + round * math::cos(self.bearing),
			self.by_range * math::cos(self.bearing) - round * math::sin(self.bearing),
		));
	}

	/// A short, unhurried move to somewhere nearby. Now and then, where the
	/// room has somewhere better to be, it leaps up onto a ledge or behind
	/// something instead, the way a cat would.
	fn amble(&mut self, events: &mut Vec<HuntEvent>) {
		if self.footing != Footing::Open {
			self.leave_cover(events, true);
			return;
		}

		let spots = self.spots(self.bearing, radians(HuntRules::COVER_SEARCH_DEGREES), 0.0);

		if !spots.is_empty()
			&& self.tick >= self.exposed_until
			&& self.cover.next_chance(HuntRules::AMBLE_COVER_CHANCE)
		{
			let spot = self.cover.pick(&spots);

			self.leap_to(spot, events);
			return;
		}

		let by = radians(
			self.path
				.next_range(-HuntRules::AMBLE_DEGREES, HuntRules::AMBLE_DEGREES),
		);
		let to_bearing = self.plan.clamp(self.bearing + by);
		let outward = self.path.next_range(0.0, 1.0);
		let to_range = self.range_at(to_bearing, outward);

		self.set_off(
			to_bearing,
			to_range,
			self.vessel.walk_speed(),
			Footing::Open,
			None,
			events,
		);
	}

	/// A dash to somewhere well away from where the reaper is aiming: into
	/// cover or up out of reach when the room offers it, unless it is
	/// burning or the wind has shaken it, when it wants nothing but open
	/// floor.
	fn bolt(&mut self, events: &mut Vec<HuntEvent>) {
		self.next_bolt_tick = -1;

		if self.vessel.gait() == Gait::Still {
			return;
		}

		if self.tick >= self.exposed_until {
			let spots = self.spots(
				self.bearing,
				radians(HuntRules::BOLT_MAX_DEGREES),
				radians(HuntRules::BOLT_MIN_DEGREES / 2.0),
			);

			if !spots.is_empty() && self.cover.next_chance(HuntRules::BOLT_COVER_CHANCE) {
				let spot = self.cover.pick(&spots);

				self.leap_to(spot, events);
				events.push(HuntEvent::Relocating);
				return;
			}
		}

		let side = if self.path.next_chance(0.5) {
			1.0
		} else {
			-1.0
		};
		let by = radians(
			self.path
				.next_range(HuntRules::BOLT_MIN_DEGREES, HuntRules::BOLT_MAX_DEGREES),
		);
		// If one way runs out of room, it goes the other.
		let one_way = self.plan.clamp(self.bearing + side * by);
		let other_way = self.plan.clamp(self.bearing - side * by);
		let to_bearing = if wrap_angle(one_way - self.bearing).abs()
			>= wrap_angle(other_way - self.bearing).abs()
		{
			one_way
		} else {
			other_way
		};
		// It runs for the far half of the room it has there.
		let outward = self.path.next_range(0.5, 1.0);
		let to_range = self.range_at(to_bearing, outward);

		self.set_off(
			to_bearing,
			to_range,
			self.vessel.bolt_speed(),
			Footing::Open,
			None,
			events,
		);
		events.push(HuntEvent::Relocating);
	}

	/// Leaps down from a ledge, or out from behind cover, onto the open
	/// floor in front of it. A husk that `may_stay` is only ready to move,
	/// and may settle down where it is a while longer.
	fn leave_cover(&mut self, events: &mut Vec<HuntEvent>, may_stay: bool) {
		if may_stay && self.cover.next_chance(HuntRules::STAY_COVER_CHANCE) {
			self.next_amble_tick = self.tick + self.rest_ticks();
			return;
		}

		let to_bearing = self
			.plan
			.clamp(self.bearing + radians(self.cover.next_range(-20.0, 20.0)));
		let outward = self.cover.next_range(0.2, 0.8);
		let to_range = self.range_at(to_bearing, outward);
		let to_height = if self.vessel.gait() == Gait::Ground {
			None
		} else {
			Some(
				self.cover
					.next_range(self.vessel.low_metres(), self.vessel.high_metres()),
			)
		};

		self.set_off(
			to_bearing,
			to_range,
			self.vessel.bolt_speed() * HuntRules::LEAP_PACE,
			Footing::Open,
			to_height,
			events,
		);
	}

	fn leap_to(&mut self, spot: Spot, events: &mut Vec<HuntEvent>) {
		self.set_off(
			spot.bearing,
			spot.range,
			self.vessel.bolt_speed() * HuntRules::LEAP_PACE,
			spot.footing,
			Some(spot.height),
			events,
		);
	}

	/// Sets off for `to_bearing` and `to_range` at `speed` metres a second,
	/// ending at `to_height`, or at the height it rests at when that is not
	/// given. A move onto, off or over anything in the room is a leap.
	fn set_off(
		&mut self,
		to_bearing: f64,
		to_range: f64,
		speed: f64,
		toward: Footing,
		to_height: Option<f64>,
		events: &mut Vec<HuntEvent>,
	) {
		self.from_bearing = self.bearing;
		self.from_range = self.range;
		self.from_height = self.height;
		self.by_bearing = wrap_angle(to_bearing - self.bearing);
		self.by_range = to_range - self.range;
		let to_height = match to_height {
			Some(height) => height,
			None => self.resting_height(),
		};
		self.by_height = to_height - self.height;

		let mean_range = self.range + self.by_range / 2.0;
		let round = mean_range * self.by_bearing;
		let length = math::sqrt(
			round * round + self.by_range * self.by_range + self.by_height * self.by_height,
		);

		self.move_ticks = 12_i64.max(math::round(
			length / (speed * self.difficulty.pace()) * HuntRules::TICK_RATE as f64,
		));
		self.move_start = self.tick;

		if self.footing == Footing::Open && toward == Footing::Open {
			return;
		}

		self.leaping = true;
		self.to_footing = toward;
		self.to_ground = if toward == Footing::Ledge {
			self.height + self.by_height - self.vessel.half_height()
		} else {
			self.floor()
		};
		self.arc = self.leap_arc(to_bearing);
		self.move_ticks = HuntRules::LEAP_MIN_TICKS.max(self.move_ticks);
		events.push(HuntEvent::Leapt(toward));
	}

	/// How far above the straight line a leap rises at its height: a hop,
	/// and enough more that its feet clear the front edge of the counter or
	/// the table it leaps onto, off or over, where it passes it.
	fn leap_arc(&self, to_bearing: f64) -> f64 {
		let arc = HuntRules::HOP_METRES + 0.45 * self.by_height.abs();
		let edge = if self.footing == Footing::Open {
			to_bearing
		} else {
			self.bearing
		};
		let front = self.plan.front_at(HuntPlan::slice_of(edge));
		let from = self.range;
		let to = self.range + self.by_range;

		if (from - front) * (to - front) >= 0.0 {
			return arc;
		}

		// How far through the leap it passes the edge, kept off the very
		// ends, where no arc could lift it in time.
		let passing = math::clamp((front - from) / (to - from), 0.2, 0.8);
		let line = self.height + self.by_height * passing;
		let clear = self.floor()
			+ self.plan.cover_at(edge)
			+ HuntRules::LEAP_CLEARANCE
			+ self.vessel.half_height();

		math::max(arc, (clear - line) / (4.0 * passing * (1.0 - passing)))
	}

	fn land(&mut self) {
		self.leaping = false;
		self.footing = self.to_footing;
		self.from_ground = self.to_ground;
		self.arc = 0.0;
	}

	/// Where the move under way ends.
	fn destination(&self) -> Vec3 {
		let bearing = self.from_bearing + self.by_bearing;
		let range = self.from_range + self.by_range;

		Vec3::new(
			math::sin(bearing) * range,
			self.from_height + self.by_height,
			math::cos(bearing) * range,
		)
	}

	/// Stops the husk where it stands. A leap cannot be stopped in the air.
	fn halt(&mut self) {
		if self.move_start < 0 || self.leaping {
			return;
		}

		self.move_start = -1;
		self.next_amble_tick = self.tick + self.rest_ticks();
	}

	fn turn_toward(&mut self, heading: f64) {
		let step = radians(HuntRules::TURN_DEGREES_PER_TICK);

		self.heading =
			wrap_angle(self.heading + math::clamp(wrap_angle(heading - self.heading), -step, step));
	}

	fn rest_ticks(&mut self) -> i64 {
		HuntRules::REST_MIN_TICKS
			+ self
				.path
				.next_int(HuntRules::REST_MAX_TICKS - HuntRules::REST_MIN_TICKS + 1)
	}

	/// Places within `within` radians of the bearing `around`, and no nearer
	/// to it than `beyond`, where the room lets the husk leap up onto a
	/// ledge or hide behind something. A place counts only where the ledge
	/// or the cover is as wide as the husk.
	fn spots(&self, around: f64, within: f64, beyond: f64) -> Vec<Spot> {
		if !self.plan.knows_cover() || self.vessel.gait() == Gait::Still {
			return Vec::new();
		}

		let mut spots = Vec::new();
		let farthest = self.vessel.far_metres() + HuntRules::HIDE_BEYOND;

		for slice in 0..HuntPlan::SLICES {
			let bearing = HuntPlan::bearing_of(slice);
			let away = wrap_angle(bearing - around).abs();

			if away > within || away < beyond || !self.plan.contains(bearing) {
				continue;
			}

			let front = self.plan.front_at(slice);

			if self.vessel.gait() == Gait::Ground
				&& let Some(hide) = self.plan.hide_at(slice)
				&& hide >= HuntRules::CLOSEST_METRES
				&& hide <= farthest
				&& self.spans(slice, hide, |other| self.hides_body(other, hide))
			{
				spots.push(Spot {
					bearing,
					range: hide,
					footing: Footing::Hidden,
					height: self.floor() + self.vessel.half_height(),
				});
			}

			let on_ledge = front + self.vessel.half_width() * HuntRules::LEDGE_INSET;

			if let Some(ledge) = self.plan.ledge_at(slice)
				&& ledge >= HuntRules::LOWEST_LEDGE
				&& ledge <= self.vessel.leap_metres()
				&& front >= HuntRules::CLOSEST_METRES
				&& on_ledge <= farthest
				&& self.spans(slice, on_ledge, |other| self.same_ledge(other, slice))
			{
				spots.push(Spot {
					bearing,
					range: on_ledge,
					footing: Footing::Ledge,
					height: self.floor() + ledge + self.vessel.half_height(),
				});
			}
		}

		spots
	}

	/// Whether every slice a husk `range` metres out along slice `slice`
	/// would stand across passes `test`.
	fn spans(&self, slice: usize, range: f64, test: impl Fn(usize) -> bool) -> bool {
		let half = math::atan2(self.vessel.half_width(), range);
		let reach = math::ceil_int(half / HuntPlan::SLICE_WIDTH);
		let slices = HuntPlan::SLICES as i64;

		(-reach..=reach).all(|step| test((slice as i64 + step).rem_euclid(slices) as usize))
	}

	/// Whether the cover along slice `other` stands in front of a husk
	/// hiding `range` metres out, and is tall enough to hide at least half
	/// of it.
	fn hides_body(&self, other: usize, range: f64) -> bool {
		let cover = self
			.plan
			.cover_centimetres
			.get(other)
			.map_or(0.0, |&cover| f64::from(cover));

		cover / 100.0 >= self.vessel.half_height()
			&& self.plan.front_at(other) < range - self.vessel.half_width() / 2.0
	}

	/// Whether slice `other` has the same ledge as slice `slice`: as high,
	/// to within a hand, and as far away, to within a stride.
	fn same_ledge(&self, other: usize, slice: usize) -> bool {
		let (Some(ledge), Some(height)) = (self.plan.ledge_at(other), self.plan.ledge_at(slice))
		else {
			return false;
		};
		let reach = |index: usize| self.plan.reach_decimetres.get(index).copied().unwrap_or(0);

		(ledge - height).abs() <= 0.1 && (reach(other) - reach(slice)).abs() <= 4
	}

	/// The farthest the husk may be along `bearing`: as far as it likes to
	/// keep, or, where something stands in the way, just this side of it.
	/// In a corner too tight even for that, it stands as close as it ever
	/// does.
	fn far_at(&self, bearing: f64) -> f64 {
		math::max(
			HuntRules::CLOSEST_METRES,
			math::min(
				self.vessel.far_metres(),
				self.plan.reach(bearing) - self.vessel.half_width(),
			),
		)
	}

	/// The nearest it may be along `bearing`: as near as it likes to come,
	/// or right up against whatever leaves it no more room than that.
	fn near_at(&self, bearing: f64) -> f64 {
		math::min(self.vessel.near_metres(), self.far_at(bearing))
	}

	/// A place along `bearing`, `outward` of the way from the nearest it may
	/// be there to the farthest.
	fn range_at(&self, bearing: f64, outward: f64) -> f64 {
		let near = self.near_at(bearing);

		near + outward * (self.far_at(bearing) - near)
	}

	/// How high the middle of the body is while it rests: standing on the
	/// floor, or somewhere in the band of air it keeps to.
	fn resting_height(&mut self) -> f64 {
		match self.vessel.gait() {
			Gait::Ground => self.vessel.half_height() - self.plan.eye_height(),
			Gait::Air | Gait::Still => {
				self.path
					.next_range(self.vessel.low_metres(), self.vessel.high_metres())
			}
		}
	}
}
