//! One encounter between a reaper and a husk, advanced a tick at a time.

use thiserror::Error;

use crate::arts::Art;
use crate::arts::ArtRules;
use crate::arts::ArtSkill;
use crate::hunt::arsenal::Archetype;
use crate::hunt::arsenal::Arsenal;
use crate::hunt::arsenal::BladeRules;
use crate::hunt::arsenal::GunRules;
use crate::hunt::arsenal::HeavyRules;
use crate::hunt::art_flight::ArtFlight;
use crate::hunt::capture::CaptureGrade;
use crate::hunt::capture::CaptureRules;
use crate::hunt::capture::OrbBlock;
use crate::hunt::capture::OrbStage;
use crate::hunt::capture::OrbView;
use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_event::EscapeCause;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_event::HuntPhase;
use crate::hunt::hunt_event::HuskRing;
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
use crate::nft_catalogue::Rarity;
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

mod walker;

use walker::Walker;

/// A plan no phone could have made, so no hunt is played in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("The plan is not one a phone could make")]
pub struct IllFormedPlan;

/// One encounter between a reaper and a husk, advanced a tick at a time.
///
/// The simulation owns every result: where the husk is, whether a shot hit
/// a seal, whether a reaper flinched and whether an orb held. It reads
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

	/// How the husk's rarity sets the odds of an orb.
	rarity_odds: f64,

	schedule: SeededRandom,

	/// Decides when the husk shrieks, and nothing else. Kept apart from
	/// `schedule`, which a reaper's own hits advance, so that everyone in
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

	/// The husk's body on the room's grid, for a husk that walks in a room
	/// the phone read cell by cell. Its place is copied into the fields
	/// above every tick, so the rest of the hunt reads them as it always
	/// has.
	walker: Option<Walker>,

	// Arsenal ------------------------------------------------------------
	slot: usize,
	drawn_at: i64,

	/// Rounds in the gun's magazine.
	rounds: i64,

	next_shot_tick: i64,
	reload_start: i64,
	reload_end: i64,
	jammed: bool,
	boosted: bool,

	/// Dives the blade has ready.
	dive_charges: i64,

	dive_recharge: i64,
	dive: Option<Dive>,

	/// The soonest a blade or heavy weapon can strike again in arm's reach,
	/// where no dive is needed.
	next_melee_tick: i64,

	// Arts ---------------------------------------------------------------
	/// Throws the reaper has ready.
	art_charges: i64,

	recharge_ticks: i64,
	next_throw_tick: i64,
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

	// Hits ---------------------------------------------------------------
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

	/// Until when the husk stands stunned: after a parry, a gust, or an orb
	/// it burst out of. Close and open, for the blade.
	stunned_until: i64,

	// Soul, the orb and the broken window --------------------------------
	/// Soul energy, from 0 to [`CaptureRules::SOUL_MAX`]. An orb costs some.
	soul: f64,

	orb_held_ticks: i64,
	orb: Option<Orb>,
	orbs_landed: i64,

	/// Orbs this husk has broken out of.
	failures: i64,

	broken_end_tick: i64,
	bonus_broken_ticks: i64,

	// Readable state -----------------------------------------------------
	tick: i64,
	phase: HuntPhase,
	ward: f64,

	/// Ward this reaper has taken off the husk, all told. In a rift, it is
	/// this reaper's share of the circle's work.
	dealt: f64,

	resolve: i64,
	awareness: f64,
	steady_ticks: i64,
	ambushed: bool,
	cadence: i64,

	/// Broken windows the husk will still let pass. The last is its escape.
	lapses_left: i64,

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

/// A dive under way: the reaper lunging in at the husk with the blade or
/// the heavy weapon.
#[derive(Clone, Copy, Debug)]
struct Dive {
	weapon: Archetype,

	/// The tick it began.
	start: i64,

	/// The tick the first stab, or the swing, lands.
	strike_at: i64,

	/// How long it lasts, from going in to coming back out.
	ticks: i64,

	stabs: i64,
	last_stab: i64,
}

impl Dive {
	fn new(weapon: Archetype, start: i64, strike_at: i64, ticks: i64) -> Self {
		Self {
			weapon,
			start,
			strike_at,
			ticks,
			stabs: 0,
			last_stab: -BladeRules::STAB_INTERVAL_TICKS,
		}
	}
}

/// An orb the reaper threw, and what became of it.
#[derive(Clone, Copy, Debug)]
struct Orb {
	thrown_at: i64,
	grade: CaptureGrade,
	on_target: bool,
	target: Vec3,

	/// Decided as it strikes the husk: the chance it holds, and the pulse it
	/// will break on, `None` when it holds.
	chance: Option<f64>,
	break_on: Option<i64>,

	/// What became of it, once it has.
	broke_on: Option<i64>,
	held: bool,
}

impl Orb {
	const fn stage_at(&self, ticks: i64) -> OrbStage {
		OrbStage::at(ticks, self.on_target, self.broke_on, self.held)
	}

	fn pulses_at(&self, ticks: i64) -> i64 {
		CaptureRules::pulses_at(ticks, self.broke_on)
	}
}

impl HuntSim {
	/// A lone hunt: the husk hides somewhere around the reaper, and its ward
	/// is theirs alone to break.
	///
	/// `ward` and `stats` are the husk's and the reaper's own, and `rarity`
	/// sets the odds of an orb. The hunt eases or stiffens them for
	/// `difficulty` itself, so that the phone and the server cannot
	/// disagree about what a difficulty means.
	pub fn new(
		seed: i64,
		vessel: Vessel,
		rarity: Rarity,
		ward: i64,
		stats: &HuntStats,
		plan: HuntPlan,
		difficulty: Difficulty,
	) -> Result<Self, IllFormedPlan> {
		Self::build(
			seed,
			vessel,
			CaptureRules::rarity_factor(rarity),
			f64::from(ward as i32) * difficulty.ward(),
			stats.at(difficulty),
			plan,
			difficulty,
			false,
			None,
		)
	}

	/// A lone hunt in the open at normal difficulty, which is never refused.
	pub fn open(seed: i64, vessel: Vessel, rarity: Rarity, ward: i64, stats: &HuntStats) -> Self {
		Self::new(
			seed,
			vessel,
			rarity,
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
	/// hears the husk shriek at the same moments. Once the shared ward
	/// breaks there is one window, the room's, and an orb thrown in it
	/// costs nothing and always holds.
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
			1.0,
			ward,
			*stats,
			HuntPlan::OPEN,
			Difficulty::Normal,
			true,
			sighting,
		)
		.unwrap_or_else(|IllFormedPlan| unreachable!("the open plan is well formed"))
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "every one is a field of the hunt, taken once to build it"
	)]
	#[allow(clippy::too_many_lines)]
	fn build(
		seed: i64,
		vessel: Vessel,
		rarity_odds: f64,
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
			rarity_odds,
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
			walker: None,
			slot: Arsenal::GUN_SLOT,
			drawn_at: 0,
			rounds: GunRules::ROUNDS,
			next_shot_tick: 0,
			reload_start: -1,
			reload_end: 0,
			jammed: false,
			boosted: false,
			dive_charges: BladeRules::DIVE_CHARGES,
			dive_recharge: 0,
			dive: None,
			next_melee_tick: 0,
			art_charges: if stats.art.is_none() {
				0
			} else {
				ArtRules::STARTING_CHARGES
			},
			recharge_ticks: 0,
			next_throw_tick: 0,
			flights: Vec::new(),
			burn_until: 0,
			burn_per_tick: 0.0,
			frozen_until: 0,
			exposed_until: 0,
			open_until: vec![0; seals],
			shut_until: vec![0; seals],
			opened_order: vec![0; seals],
			opened_count: 0,
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
			stunned_until: 0,
			soul: 0.0,
			orb_held_ticks: 0,
			orb: None,
			orbs_landed: 0,
			failures: 0,
			broken_end_tick: 0,
			bonus_broken_ticks: 0,
			tick: 0,
			phase: HuntPhase::Stalk,
			ward,
			dealt: 0.0,
			resolve: stats.resolve,
			awareness: 0.0,
			steady_ticks: 0,
			ambushed: false,
			cadence: 0,
			lapses_left: if in_rift {
				1
			} else {
				HuntRules::LAPSES + difficulty.lapses()
			},
			outcome: None,
		};

		// Drawn from the path's stream before anything else, as the reap
		// arc's direction once was, so that a husk still moves as it always
		// has.
		let _ = sim.path.next_chance(0.5);
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
		sim.set_walker_down();

		Ok(sim)
	}

	/// Replays `log` from the start and returns how the hunt ended, or
	/// `None` when the log stops before the hunt does, or was played in a
	/// plan no phone could have made.
	pub fn replay(
		seed: i64,
		vessel: Vessel,
		rarity: Rarity,
		ward: i64,
		stats: &HuntStats,
		log: &HuntLog,
	) -> Option<HuntOutcome> {
		if !log.plan.is_well_formed() {
			return None;
		}

		let mut sim = Self::new(
			seed,
			vessel,
			rarity,
			ward,
			stats,
			log.plan.clone(),
			log.difficulty,
		)
		.ok()?;

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

	/// How the husk's rarity sets the odds of an orb.
	pub const fn rarity_odds(&self) -> f64 {
		self.rarity_odds
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

	/// Ward this reaper has taken off the husk, all told.
	pub const fn dealt(&self) -> f64 {
		self.dealt
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

	/// Hits in a row.
	pub const fn cadence(&self) -> i64 {
		self.cadence
	}

	/// Broken windows the husk will still let pass. The last is its escape.
	pub const fn lapses_left(&self) -> i64 {
		self.lapses_left
	}

	/// Orbs this husk has broken out of.
	pub const fn failures(&self) -> i64 {
		self.failures
	}

	/// Soul energy, from 0 to [`CaptureRules::SOUL_MAX`].
	pub const fn soul(&self) -> f64 {
		self.soul
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

	/// Ticks left to land another hit and keep the current cadence.
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
			math::round(HuntRules::FIGHT_LIMIT_TICKS as f64 * self.difficulty.fight_length())
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

	/// What the ring under the husk says.
	pub fn ring(&self) -> HuskRing {
		match self.phase {
			HuntPhase::Windup | HuntPhase::Shriek => HuskRing::DrawingBreath,
			HuntPhase::Charge => HuskRing::Charging,
			HuntPhase::Broken | HuntPhase::Capture => HuskRing::Weakened,
			HuntPhase::Fight if self.stunned() => HuskRing::Stunned,
			HuntPhase::Fight if !self.in_rift && self.weakened() => HuskRing::Weakened,
			_ => HuskRing::Open,
		}
	}

	/// Whether the husk's ward is down to what an orb may be thrown at.
	fn weakened(&self) -> bool {
		self.ward <= self.ward_max * CaptureRules::WEAKENED_SHARE
	}

	/// Sets what is left of the husk's ward, the way a test or a practice
	/// hunt does. The server never does this: a replay's ward is the
	/// rules' own.
	pub fn set_ward(&mut self, ward: f64) {
		self.ward = ward;
	}

	/// Sets the reaper's soul energy, the way a test does.
	pub fn set_soul(&mut self, soul: f64) {
		self.soul = soul;
	}

	/// Sets the dives the blade has ready, the way a test does.
	pub fn set_dive_charges(&mut self, charges: i64) {
		self.dive_charges = charges;
	}

	/// Sets the ward this reaper has taken off the husk, the way a test
	/// does.
	pub fn set_dealt(&mut self, dealt: f64) {
		self.dealt = dealt;
	}
}

impl HuntSim {
	/// Where the husk is right now.
	pub fn pose(&self) -> HuskPose {
		if let Some(walker) = &self.walker {
			return walker.pose(self.vessel);
		}

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
			scale: 1.0,
			locomotion: None,
		}
	}

	/// What the husk stands on or behind.
	pub const fn footing(&self) -> Footing {
		self.footing
	}

	/// Whether the husk is in the air between one footing and the next.
	/// A husk walking a room's grid counts from the moment it gathers
	/// itself until it has stood up again, and all the way down to open
	/// floor when it has to come down.
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
		if let Some(walker) = &self.walker {
			return walker.speed(self.move_ticks);
		}

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

	// The arsenal, as the view reads it ----------------------------------

	/// The slot in hand.
	pub const fn selected_slot(&self) -> usize {
		self.slot
	}

	/// Ticks until the weapon in hand is drawn and can be fired.
	pub fn draw_ticks_left(&self) -> i64 {
		(self.drawn_at - self.tick).max(0)
	}

	/// Whether the gun is reloading.
	pub const fn reloading(&self) -> bool {
		self.reload_start >= 0
	}

	/// How far through its reload the gun is, from 0 to 1, as a share of an
	/// unjammed reload. A jammed reload runs past 1.
	pub fn reload_progress(&self) -> f64 {
		if self.reloading() {
			(self.tick - self.reload_start) as f64 / GunRules::RELOAD_TICKS as f64
		} else {
			0.0
		}
	}

	/// Whether the reload under way has jammed.
	pub fn reload_jammed(&self) -> bool {
		self.reloading() && self.jammed
	}

	/// Whether FIRE now would finish the reload early.
	pub fn in_active_reload(&self) -> bool {
		if !self.reloading() || self.jammed {
			return false;
		}

		let into = self.tick - self.reload_start;

		(GunRules::ACTIVE_FROM_TICK..=GunRules::ACTIVE_UNTIL_TICK).contains(&into)
	}

	/// Whether the magazine in the gun was loaded by an active reload.
	pub const fn magazine_boosted(&self) -> bool {
		self.boosted
	}

	/// Rounds left in the gun's magazine.
	pub const fn rounds(&self) -> i64 {
		self.rounds
	}

	/// How far the next dive charge has come back, from 0 to 1. Zero when
	/// every charge is ready.
	pub fn dive_recharge(&self) -> f64 {
		if self.dive_charges >= BladeRules::DIVE_CHARGES {
			0.0
		} else {
			self.dive_recharge as f64 / BladeRules::CHARGE_TICKS as f64
		}
	}

	/// Dives the blade has ready.
	pub const fn dive_charges(&self) -> i64 {
		self.dive_charges
	}

	/// Whether the reaper is diving in at the husk.
	pub const fn diving(&self) -> bool {
		self.dive.is_some()
	}

	/// The weapon of the dive under way, or `None`.
	pub fn dive_weapon(&self) -> Option<Archetype> {
		self.dive.map(|dive| dive.weapon)
	}

	/// How far through the dive under way the reaper is, from 0 to 1.
	pub fn dive_progress(&self) -> f64 {
		match self.dive {
			Some(dive) => {
				math::clamp(
					(self.tick - dive.start) as f64 / dive.ticks as f64,
					0.0,
					1.0,
				)
			}
			None => 0.0,
		}
	}

	/// Ticks until the weapon in hand can act again: drawn, fired, swung or
	/// thrown. A reload counts for the gun.
	pub fn cooldown_ticks(&self) -> i64 {
		let ready = match Arsenal::archetype_of(self.slot) {
			Some(Archetype::Gun) => {
				if self.reloading() {
					self.reload_end
				} else {
					self.next_shot_tick
				}
			}
			Some(Archetype::Blade | Archetype::Heavy) => {
				match self.dive {
					Some(dive) => dive.start + dive.ticks,
					None => self.next_melee_tick,
				}
			}
			_ => self.next_throw_tick,
		};

		(ready.max(self.drawn_at) - self.tick).max(0)
	}

	/// Whether a shot, stab or swing along `aim` would strike an open seal.
	pub fn on_open_seal(&self, aim: Vec3) -> bool {
		self.nearest_open_seal(aim, &self.pose(), self.stats.reach_radians)
			.is_some()
	}

	/// Whether the weapon in hand can reach the husk along `aim`: whether a
	/// press now would do anything to it.
	pub fn in_reach(&self, aim: Vec3) -> bool {
		if !(self.fighting() || self.phase == HuntPhase::Stalk) {
			return false;
		}

		let current = self.pose();
		let close = current.distance() <= Arsenal::CLOSE_METRES;

		match Arsenal::archetype_of(self.slot) {
			Some(Archetype::Gun) => {
				self.on_open_seal(aim) || current.covers(aim, self.stats.assist_radians)
			}
			Some(Archetype::Blade) => {
				current.covers(aim, self.stats.blade_reach_radians)
					&& (close
						|| (current.distance() <= BladeRules::DIVE_METRES && self.dive_charges > 0))
			}
			Some(Archetype::Heavy) => {
				current.covers(aim, self.stats.heavy_reach_radians)
					&& current.distance() <= HeavyRules::DIVE_METRES
			}
			_ => self.stats.art.is_some() && self.art_charges > 0,
		}
	}

	// The orb, as the view reads it --------------------------------------

	/// Why an orb cannot be thrown right now, or `None` when it can.
	pub fn orb_block(&self) -> Option<OrbBlock> {
		if !self.fighting() && self.phase != HuntPhase::Broken {
			return Some(OrbBlock::NotFighting);
		}

		if self.orb_in_flight() {
			return Some(OrbBlock::InFlight);
		}

		if self.in_rift {
			return if self.phase == HuntPhase::Broken {
				None
			} else {
				Some(OrbBlock::NotWeakened)
			};
		}

		if self.phase != HuntPhase::Broken && !self.weakened() {
			return Some(OrbBlock::NotWeakened);
		}

		if self.soul < CaptureRules::COST {
			return Some(OrbBlock::LowSoul);
		}

		None
	}

	/// How far the ring has shrunk around the husk while ORB is held, from
	/// 0 to 1, or `None` while it is not held.
	pub fn orb_ring(&self) -> Option<f64> {
		if self.orb_held_ticks == 0 {
			None
		} else {
			Some(CaptureRules::ring_progress(self.orb_held_ticks))
		}
	}

	/// The grade a throw let go now would have, with the crosshair along
	/// `aim`. [`CaptureGrade::Plain`] while ORB is not held.
	pub fn orb_grade(&self, aim: Vec3) -> CaptureGrade {
		CaptureRules::grade_for(self.orb_held_ticks, self.pose().covers(aim, 0.0))
	}

	/// The last orb thrown, or `None` before the first.
	pub fn orb(&self) -> Option<OrbView> {
		let thrown = self.orb?;
		let ticks = self.tick - thrown.thrown_at;

		Some(OrbView {
			thrown_at: thrown.thrown_at,
			ticks,
			stage: thrown.stage_at(ticks),
			grade: thrown.grade,
			on_target: thrown.on_target,
			pulses: thrown.pulses_at(ticks),
			target: thrown.target,
			chance: thrown.chance,
			broke_free_on: thrown.broke_on,
		})
	}

	fn orb_in_flight(&self) -> bool {
		match self.orb {
			Some(orb) => self.tick - orb.thrown_at < CaptureRules::IMPACT_TICK,
			None => false,
		}
	}

	/// Ticks left before a broken husk recovers.
	pub fn broken_ticks_left(&self) -> i64 {
		if self.phase == HuntPhase::Broken {
			(self.broken_end_tick - self.tick).max(0)
		} else {
			0
		}
	}

	/// How long a broken husk stays down.
	pub const fn broken_ticks(&self) -> i64 {
		if self.in_rift {
			RiftRules::REAP_WINDOW_TICKS
		} else {
			self.stats.broken_ticks + self.bonus_broken_ticks
		}
	}
}

impl HuntSim {
	// Seals --------------------------------------------------------------

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
		self.room_hides(current.seal_position(index))
	}

	/// Whether something in the room stands between the eye and `point`:
	/// read from the grid for a husk that walks it, and from the plan's
	/// slices otherwise.
	fn room_hides(&self, point: Vec3) -> bool {
		match &self.walker {
			Some(walker) => walker.grid().hides(point),
			None => self.plan.hides(point),
		}
	}

	fn seal_count(&self) -> usize {
		self.vessel.seals().len()
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

	/// Advances the hunt by one tick and returns what happened in it.
	pub fn step(&mut self, input: &HuntInput) -> Vec<HuntEvent> {
		if self.is_over() {
			return Vec::new();
		}

		let mut events = Vec::new();
		let thrown = self.let_go(input);

		self.step_missed_orb(&mut events);

		match self.phase {
			HuntPhase::Stalk => self.step_stalk(input, &mut events),
			HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Shriek | HuntPhase::Charge => {
				self.step_fight(input, thrown, &mut events);
			}
			HuntPhase::Broken => self.step_broken(input, thrown, &mut events),
			HuntPhase::Capture => self.step_capture(input, &mut events),
			HuntPhase::Absorbed | HuntPhase::Escaped => {}
		}

		self.tick += 1;

		events
	}

	/// Follows ORB, and returns how long it was held when it is let go this
	/// tick, or `None`.
	fn let_go(&mut self, input: &HuntInput) -> Option<i64> {
		if input.orb_held {
			self.orb_held_ticks += 1;

			return None;
		}

		let held = self.orb_held_ticks;

		self.orb_held_ticks = 0;

		(held > 0).then_some(held)
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

		// A reaper who fires first, or dives in, has given up the ambush,
		// and the blow still counts.
		self.step_arsenal(input, events, true);

		if self.phase != HuntPhase::Stalk {
			return;
		}

		let current = self.pose();
		let speed = input.turn_radians_per_second();

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

	/// Starts the fight, without an ambush, if the husk has not yet noticed
	/// the reaper. A shot or a dive gives a reaper away.
	fn engage(&mut self, events: &mut Vec<HuntEvent>) {
		if self.phase == HuntPhase::Stalk {
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

	/// Whether the husk stands stunned: after a parry, a gust, or an orb it
	/// burst out of. Close and open, for the blade.
	pub fn stunned(&self) -> bool {
		self.fighting() && self.tick < self.stunned_until
	}

	/// Ticks left of the stun, or 0.
	pub fn stun_ticks_left(&self) -> i64 {
		if self.stunned() {
			self.stunned_until - self.tick
		} else {
			0
		}
	}

	fn step_fight(&mut self, input: &HuntInput, thrown: Option<i64>, events: &mut Vec<HuntEvent>) {
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

		self.step_shriek(input, thrown, events);

		if self.is_over() {
			return;
		}

		// Nothing is fired into a shriek: a press is a flinch. The weapons
		// keep reloading, recharging and coming back from a dive.
		if self.phase == HuntPhase::Shriek {
			self.step_arsenal(input, events, false);
			return;
		}

		self.step_charge(input, events);

		if self.is_over() {
			return;
		}

		self.step_seals(events);
		self.step_arsenal(input, events, true);

		if let Some(held) = thrown {
			self.throw_orb(input.aim.direction(), held, events);

			if self.phase == HuntPhase::Capture {
				return;
			}
		}

		if self.ward <= 0.0 {
			self.break_ward(events);
		}
	}

	fn step_shriek(&mut self, input: &HuntInput, thrown: Option<i64>, events: &mut Vec<HuntEvent>) {
		// A husk in mid-leap lands before it draws breath.
		if self.phase == HuntPhase::Fight && self.tick >= self.next_shriek_tick && !self.leaping {
			self.echo_cadence = self.cadence;
			self.phase = HuntPhase::Windup;
			self.phase_end_tick = self.tick + HuntRules::WINDUP_TICKS;
			events.push(HuntEvent::ShriekWindup);

			// A husk shrieks at the reaper's face, not from behind the
			// sofa: it comes out over the top as it draws breath.
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
			let tick = self.tick;
			self.open_until.fill(tick);
			events.push(HuntEvent::ShriekBegan);
			return;
		}

		if self.phase != HuntPhase::Shriek {
			return;
		}

		let past_grace = self.tick - self.shriek_start_tick >= HuntRules::SHRIEK_GRACE_TICKS;
		let moved = input.turn_radians_per_second() > self.stats.flinch_speed
			|| input.fire
			|| thrown.is_some();

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
		self.gain_soul(CaptureRules::HOLD_SOUL);
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
			self.bonus_broken_ticks += HuntRules::TOLL_TICKS;
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

	/// Whether the husk has stopped in its tracks: stunned, or knocked still
	/// for a moment by a heavy blow.
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
			// A walker with no way down from where it is does not charge
			// this time.
			if self.walker.is_some() {
				if !self.walk_down(false) {
					self.schedule_charge();
				}

				return;
			}

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
		if self.walker.is_some() {
			self.begin_dash();
			self.move_start = self.tick;
			self.move_ticks = self.charge_ticks();
			events.push(HuntEvent::ChargeBegan);
			return;
		}

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
	/// stands stunned where it got to, close, every seal in view open.
	fn parry(&mut self, events: &mut Vec<HuntEvent>) {
		self.end_charge();
		self.move_start = -1;
		self.stun(HuntRules::STAGGER_TICKS);
		events.push(HuntEvent::Parried);
		self.parries += 1;
		self.gain_soul(CaptureRules::PARRY_SOUL);
		self.expose_all_seals(events);

		if self.stats.gift == Some(ReaperGift::Reprisal) {
			self.chosen_blow_ready = true;
			self.gift_strike_ready = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Reprisal));
		}
	}

	/// Stuns the husk where it stands for `ticks`, and it runs after.
	fn stun(&mut self, ticks: i64) {
		self.staggered_until = self.tick + ticks;
		self.stunned_until = self.staggered_until;
		self.next_bolt_tick = self.staggered_until;
	}

	/// The charge got through. It costs the reaper resolve, twice over if it
	/// caught them mid-dive, and the husk is off again at once.
	fn strike_reaper(&mut self, events: &mut Vec<HuntEvent>) {
		if self.passage_ready {
			self.passage_ready = false;
			self.parry(events);
			// This is the gift's protection, not an input the player
			// performed, and it pays no soul.
			self.parries -= 1;
			self.soul = math::max(0.0, self.soul - CaptureRules::PARRY_SOUL);
			self.stun(SoulRules::PASSAGE_STAGGER_TICKS_PER_STRENGTH * self.stats.gift_strength());
			events.push(HuntEvent::GiftStirred(ReaperGift::Passage));
			return;
		}

		let caught_diving = self.diving();

		self.end_charge();
		self.end_dive(events);
		self.resolve = math::max(
			0.0,
			self.resolve as f64 - if caught_diving { 2.0 } else { 1.0 },
		) as i64;
		self.cadence = 0;
		self.hit_streak = 0;
		events.push(HuntEvent::Struck {
			resolve_left: self.resolve,
			diving: caught_diving,
		});

		if self.resolve <= 0 {
			self.escape(events, EscapeCause::Resolve);
			return;
		}

		self.bolt(events);
	}

	fn end_charge(&mut self) {
		self.end_dash();
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

	// The arsenal --------------------------------------------------------

	/// Draws, reloads, recharges and fires the reaper's weapons for one
	/// tick. A reaper who is not `firing` (in a shriek) presses nothing:
	/// whatever FIRE did, the shriek has taken as a flinch.
	fn step_arsenal(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>, firing: bool) {
		self.step_reload(events);
		self.step_dive_charges();
		self.step_art_recharge();

		let pressed = firing && input.fire;

		if self.dive.is_some() {
			self.step_dive(input.aim, events, pressed, firing);
			return;
		}

		self.select(usize::from(input.slot), events);

		if !pressed || self.tick < self.drawn_at {
			return;
		}

		match Arsenal::archetype_of(self.slot) {
			Some(Archetype::Gun) => self.pull_trigger(input.aim, events),
			Some(Archetype::Blade) => self.thrust(input.aim, events),
			Some(Archetype::Heavy) => self.heave(input.aim, events),
			_ => self.throw_art(input.aim, events),
		}
	}

	fn select(&mut self, slot: usize, events: &mut Vec<HuntEvent>) {
		if slot == self.slot || slot >= Arsenal::SLOTS {
			return;
		}

		self.slot = slot;
		self.drawn_at = self.tick + Arsenal::SWITCH_TICKS;
		events.push(HuntEvent::WeaponDrawn(slot));
	}

	// Gun ----------------------------------------------------------------

	fn pull_trigger(&mut self, aim: Aim, events: &mut Vec<HuntEvent>) {
		if self.reloading() {
			self.active_reload(events);
			return;
		}

		if self.rounds == 0 {
			self.reload_start = self.tick;
			self.reload_end = self.tick + GunRules::RELOAD_TICKS;
			self.jammed = false;
			events.push(HuntEvent::ReloadStarted);
			return;
		}

		if self.tick < self.next_shot_tick {
			return;
		}

		self.engage(events);
		self.rounds -= 1;
		self.next_shot_tick = self.tick + GunRules::FIRE_INTERVAL_TICKS;
		events.push(HuntEvent::ShotFired {
			rounds_left: self.rounds,
			boosted: self.boosted,
		});
		self.strike(
			Archetype::Gun,
			aim.direction(),
			GunRules::SHOT_STRIKE
				* if self.boosted {
					GunRules::BOOSTED_STRIKE
				} else {
					1.0
				},
			events,
		);
	}

	/// FIRE in the middle of a reload: in the window it finishes it at once,
	/// before the window it jams it, and after the window it does nothing.
	fn active_reload(&mut self, events: &mut Vec<HuntEvent>) {
		if self.jammed {
			return;
		}

		let into = self.tick - self.reload_start;

		if (GunRules::ACTIVE_FROM_TICK..=GunRules::ACTIVE_UNTIL_TICK).contains(&into) {
			self.finish_reload(events, true);
		} else if into < GunRules::ACTIVE_FROM_TICK {
			self.jammed = true;
			self.reload_end += GunRules::JAM_TICKS;
			events.push(HuntEvent::ReloadJammed);
		}
	}

	fn step_reload(&mut self, events: &mut Vec<HuntEvent>) {
		if self.reloading() && self.tick >= self.reload_end {
			self.finish_reload(events, false);
		}
	}

	fn finish_reload(&mut self, events: &mut Vec<HuntEvent>, perfect: bool) {
		self.rounds = GunRules::ROUNDS;
		self.boosted = perfect;
		self.reload_start = -1;
		self.jammed = false;
		events.push(HuntEvent::ReloadFinished { perfect });
	}

	// Blade and heavy ----------------------------------------------------

	fn step_dive_charges(&mut self) {
		if self.dive_charges >= BladeRules::DIVE_CHARGES {
			self.dive_recharge = 0;
			return;
		}

		self.dive_recharge += 1;

		if self.dive_recharge >= BladeRules::CHARGE_TICKS {
			self.dive_charges += 1;
			self.dive_recharge = 0;
		}
	}

	/// FIRE with the blade in hand: a stab in arm's reach, or a dive at a
	/// husk on the crosshair within reach of one.
	fn thrust(&mut self, aim: Aim, events: &mut Vec<HuntEvent>) {
		let current = self.pose();
		let ray = aim.direction();

		if current.distance() <= Arsenal::CLOSE_METRES {
			if self.tick < self.next_melee_tick {
				return;
			}

			self.engage(events);
			self.next_melee_tick = self.tick + BladeRules::STAB_INTERVAL_TICKS;
			self.strike(Archetype::Blade, ray, BladeRules::STAB_STRIKE, events);
			return;
		}

		let reachable = current.covers(ray, self.stats.blade_reach_radians)
			&& current.distance() <= BladeRules::DIVE_METRES
			&& self.dive_charges > 0;

		if !reachable {
			return;
		}

		self.engage(events);
		self.dive_charges -= 1;
		self.dive = Some(Dive::new(
			Archetype::Blade,
			self.tick,
			self.tick + BladeRules::IN_TICKS,
			BladeRules::IN_TICKS + BladeRules::WINDOW_TICKS + BladeRules::OUT_TICKS,
		));
		events.push(HuntEvent::DiveStarted(Archetype::Blade));
	}

	/// FIRE with the class weapon in hand: a swing in arm's reach, or a
	/// dive and a swing at a husk on the crosshair within reach of one.
	fn heave(&mut self, aim: Aim, events: &mut Vec<HuntEvent>) {
		let current = self.pose();
		let ray = aim.direction();

		if current.distance() <= Arsenal::CLOSE_METRES {
			if self.tick < self.next_melee_tick {
				return;
			}

			self.engage(events);
			self.next_melee_tick = self.tick + self.stats.swing_ticks;
			self.strike(Archetype::Heavy, ray, self.stats.heavy_strike, events);
			return;
		}

		let reachable = current.covers(ray, self.stats.heavy_reach_radians)
			&& current.distance() <= HeavyRules::DIVE_METRES;

		if !reachable {
			return;
		}

		self.engage(events);
		self.dive = Some(Dive::new(
			Archetype::Heavy,
			self.tick,
			self.tick + HeavyRules::IN_TICKS + self.stats.swing_ticks,
			HeavyRules::IN_TICKS + self.stats.swing_ticks + HeavyRules::OUT_TICKS,
		));
		events.push(HuntEvent::DiveStarted(Archetype::Heavy));
	}

	/// Plays a dive on: the blade stabs as it arrives and at each press in
	/// its window, the heavy weapon swings once, and then the reaper comes
	/// back out. A dive in a shriek lands nothing (`striking` is false).
	fn step_dive(&mut self, aim: Aim, events: &mut Vec<HuntEvent>, pressed: bool, striking: bool) {
		let Some(mut dive) = self.dive else {
			return;
		};
		let ray = aim.direction();

		match dive.weapon {
			Archetype::Blade => {
				let window_end = dive.strike_at + BladeRules::WINDOW_TICKS;
				let stab = self.tick == dive.strike_at
					|| (pressed
						&& self.tick > dive.strike_at
						&& self.tick < window_end
						&& dive.stabs < BladeRules::STABS_PER_DIVE
						&& self.tick - dive.last_stab >= BladeRules::STAB_INTERVAL_TICKS);

				if stab {
					dive.stabs += 1;
					dive.last_stab = self.tick;

					if striking {
						self.strike(Archetype::Blade, ray, BladeRules::STAB_STRIKE, events);
					}
				}
			}
			Archetype::Heavy => {
				if self.tick == dive.strike_at && striking {
					self.strike(Archetype::Heavy, ray, self.stats.heavy_strike, events);
				}
			}
			Archetype::Gun | Archetype::Art => {}
		}

		if self.tick >= dive.start + dive.ticks - 1 {
			self.end_dive(events);
		} else {
			self.dive = Some(dive);
		}
	}

	fn end_dive(&mut self, events: &mut Vec<HuntEvent>) {
		if self.dive.take().is_some() {
			events.push(HuntEvent::DiveEnded);
		}
	}

	// Hits ---------------------------------------------------------------

	/// A shot, stab or swing along `ray` doing `share` of a hit: on an open
	/// seal, on the body, or on nothing.
	fn strike(&mut self, weapon: Archetype, ray: Vec3, share: f64, events: &mut Vec<HuntEvent>) {
		let current = self.pose();
		let seal = self.nearest_open_seal(ray, &current, self.stats.reach_radians);

		if seal.is_none() {
			let pad = match weapon {
				Archetype::Gun => self.stats.assist_radians,
				Archetype::Blade => self.stats.blade_reach_radians,
				Archetype::Heavy => self.stats.heavy_reach_radians,
				Archetype::Art => 0.0,
			};
			// A blow at the part of a husk that is behind the counter hits
			// the counter.
			let blocked = self.room_hides(ray * current.distance());

			if blocked || !current.covers(ray, pad) {
				self.miss(weapon, events);
				return;
			}
		}

		self.land_hit(weapon, seal, share, events);
	}

	fn miss(&mut self, weapon: Archetype, events: &mut Vec<HuntEvent>) {
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

		events.push(HuntEvent::Missed(weapon));
	}

	/// A hit that landed: on open `seal`, or on the body when it is `None`.
	#[allow(clippy::too_many_lines)]
	fn land_hit(
		&mut self,
		weapon: Archetype,
		seal: Option<usize>,
		share: f64,
		events: &mut Vec<HuntEvent>,
	) {
		let critical = seal.is_some() || (weapon == Archetype::Blade && self.stunned());
		let cap = if self.stats.perk == ClassPerk::Chain {
			HuntRules::CHAIN_CADENCE_CAP
		} else {
			HuntRules::CADENCE_CAP
		};
		let mut scale = share * (1.0 + self.cadence.min(cap) as f64 * HuntRules::CADENCE_STEP);
		let mut empowered = false;
		let first_strike = self.ambush_strike_ready;
		let in_order = seal.is_some_and(|seal| self.seal_rank(seal) == Some(0));
		let aftermath = seal.is_some()
			&& self.gift_strike_ready
			&& self.stats.gift == Some(ReaperGift::Aftermath);

		if critical {
			scale *= HuntRules::CRITICAL_STRIKE;
		} else if weapon == Archetype::Heavy {
			scale *= HuntRules::CRITICAL_STRIKE * self.stats.body_share;
		}

		if self.ambush_strike_ready {
			self.ambush_strike_ready = false;
			scale *= HuntRules::AMBUSH_STRIKE;
			empowered = true;
		}

		if first_strike && self.stats.gift == Some(ReaperGift::FirstLight) {
			scale *= 1.0 + SoulRules::GIFT_STRIKE_PER_STRENGTH * self.stats.gift_strength() as f64;
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

		let verdict = self.stats.gift == Some(ReaperGift::Verdict) && in_order;
		let promised = seal.is_some()
			&& self.gift_strike_ready
			&& (verdict
				|| matches!(
					self.stats.gift,
					Some(ReaperGift::Keening | ReaperGift::Reprisal)
				));
		let execution = seal.is_some()
			&& self.stats.gift == Some(ReaperGift::LastWord)
			&& self.cadence >= SoulRules::EXECUTION_CADENCE
			&& self.ward <= self.ward_max * SoulRules::EXECUTION_WARD_SHARE;

		if promised || execution {
			scale *= 1.0 + SoulRules::GIFT_STRIKE_PER_STRENGTH * self.stats.gift_strength() as f64;
			empowered = true;

			if promised {
				self.gift_strike_ready = false;
			}

			if let Some(gift) = self.stats.gift {
				events.push(HuntEvent::GiftStirred(gift));
			}
		}

		if aftermath {
			self.gift_strike_ready = false;
			empowered = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Aftermath));
		}

		if seal.is_some() && self.frozen() {
			scale *= self.stats.art.map_or(1.0, |art| art.shatter());
			empowered = true;
			self.frozen_until = self.tick;
			events.push(HuntEvent::IceShattered);
		}

		let mut damage = self.stats.hit * scale;
		let mut cleaved = Vec::new();

		if let Some(seal) = seal {
			self.shut_seal(seal);

			if self.stats.perk == ClassPerk::Cleave {
				for other in 0..self.seal_count() {
					if !self.seal_open(other) {
						continue;
					}

					if aftermath {
						self.open_until[other] = self.open_until[other].max(
							self.tick
								+ HuntRules::EXPOSED_TICKS
								+ SoulRules::GATHERING_TICKS_PER_STRENGTH
									* self.stats.gift_strength(),
						);
					} else {
						self.shut_seal(other);
					}
					cleaved.push(other);
					let cleave_share = HuntRules::CLEAVE_SHARE
						+ if first_strike && self.stats.gift == Some(ReaperGift::Furrow) {
							SoulRules::FURROW_CLEAVE_PER_STRENGTH
								* self.stats.gift_strength() as f64
						} else {
							0.0
						};
					damage += self.stats.hit * scale * cleave_share;
				}
			}
		}

		if first_strike && self.stats.gift == Some(ReaperGift::Furrow) && !cleaved.is_empty() {
			empowered = true;
			events.push(HuntEvent::GiftStirred(ReaperGift::Furrow));
		}

		self.harm(damage);

		if weapon == Archetype::Heavy {
			self.knock();
		}

		self.cadence += 1;
		self.hit_streak += 1;
		self.max_hit_streak = self.max_hit_streak.max(self.hit_streak);
		self.last_hit_tick = self.tick;
		events.push(HuntEvent::HitLanded {
			weapon,
			seal,
			damage,
			cadence: self.cadence,
			critical,
			empowered,
			cleaved,
		});
		self.after_gift_strike(seal, in_order, events);
	}

	/// Takes `damage` off the ward, counts it as this reaper's, and turns it
	/// into soul energy.
	fn harm(&mut self, damage: f64) {
		self.ward = math::max(0.0, self.ward - damage);
		self.dealt += damage;
		self.gain_soul(CaptureRules::SOUL_PER_WARD * damage / self.ward_max);
	}

	fn gain_soul(&mut self, gained: f64) {
		self.soul = math::min(CaptureRules::SOUL_MAX, self.soul + gained);
	}

	/// Conditional powers keep their own counters, never a second
	/// simulation.
	fn after_gift_strike(
		&mut self,
		seal: Option<usize>,
		in_order: bool,
		events: &mut Vec<HuntEvent>,
	) {
		match self.stats.gift {
			Some(ReaperGift::Gathering) => {
				if let Some(seal) = seal
					&& self.cadence % SoulRules::GATHERING_HITS == 0
				{
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
				if seal.is_none() {
					return;
				}

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
				self.bonus_broken_ticks +=
					SoulRules::REFRAIN_TICKS_PER_STRENGTH * self.stats.gift_strength();
				self.refrain_tolls += 1;
				events.push(HuntEvent::GiftStirred(ReaperGift::Refrain));
			}
			_ => {}
		}
	}

	/// A heavy blow stops the husk in its tracks.
	fn knock(&mut self) {
		if self.phase != HuntPhase::Fight || self.leaping {
			return;
		}

		self.staggered_until = self.staggered_until.max(self.tick + self.stats.knock_ticks);
		self.halt();
	}

	// Arts ---------------------------------------------------------------

	fn step_art_recharge(&mut self) {
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
	}

	/// FIRE with the art in hand: it goes at the crosshair at once.
	fn throw_art(&mut self, aim: Aim, events: &mut Vec<HuntEvent>) {
		let Some(art) = self.stats.art else {
			return;
		};

		if self.art_charges == 0 || self.tick < self.next_throw_tick {
			return;
		}

		self.next_throw_tick = self.tick + ArtRules::THROW_INTERVAL_TICKS;
		self.throw(art, aim, events);
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
		if let Some(walker) = &self.walker {
			return ray * walker.grid().surface_along(ray, ArtRules::FARTHEST_METRES);
		}

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
				(current.position - at).length() <= current.half_width() + ArtRules::BURST_METRES
			}
			// Ice flies straight, and breaks on whatever the husk is behind.
			Art::Rime => flight.homing && !self.room_hides(current.position),
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

		let damage = self.stats.hit * art.strike();

		self.harm(damage);
		events.push(HuntEvent::ArtLanded {
			art: flight.art,
			at,
			hit: true,
			damage,
		});

		match flight.art {
			Art::Cinder => {
				self.burn_until = self.tick + art.burn_ticks();
				self.burn_per_tick = self.stats.hit * art.burn_strike() / art.burn_ticks() as f64;
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
			self.harm(self.burn_per_tick);
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
		} else if self.walker.is_some() {
			self.gust_walker(art.gust_stun_ticks());
		} else if self.leaping {
			self.staggered_until = self.move_start + self.move_ticks + art.gust_stun_ticks();
		} else if self.footing != Footing::Open {
			self.leave_cover(events, false);
			self.staggered_until = self.tick + self.move_ticks + art.gust_stun_ticks();
		} else {
			self.halt();
			self.staggered_until = self.tick + art.gust_stun_ticks();
		}

		self.stunned_until = self.staggered_until;
		self.next_bolt_tick = self.staggered_until;
		self.exposed_until = self.staggered_until + ArtRules::SHAKEN_TICKS;
		self.expose_all_seals(events);
	}

	// Break --------------------------------------------------------------

	fn break_ward(&mut self, events: &mut Vec<HuntEvent>) {
		self.phase = HuntPhase::Broken;
		self.ward = 0.0;
		self.settle(events);
		self.broken_end_tick = self.tick + self.broken_ticks();
		events.push(HuntEvent::Broken {
			window_ticks: self.broken_ticks(),
		});
	}

	/// Stops everything going on around the husk: it is going nowhere, and
	/// nothing more lands on it. A husk in mid-leap still lands.
	fn settle(&mut self, events: &mut Vec<HuntEvent>) {
		self.halt();
		self.flights.clear();
		self.burn_until = self.tick;
		self.frozen_until = self.tick;
		self.end_dive(events);
	}

	fn step_broken(&mut self, input: &HuntInput, thrown: Option<i64>, events: &mut Vec<HuntEvent>) {
		if self.leaping {
			self.continue_leap(events);
		}

		self.step_reload(events);
		self.step_dive_charges();
		self.step_art_recharge();
		self.select(usize::from(input.slot), events);

		if let Some(held) = thrown {
			self.throw_orb(input.aim.direction(), held, events);

			if self.phase == HuntPhase::Capture {
				return;
			}
		}

		if self.tick < self.broken_end_tick {
			return;
		}

		self.lapses_left -= 1;

		if self.lapses_left <= 0 {
			self.escape(events, EscapeCause::Lapsed);
			return;
		}

		self.phase = HuntPhase::Fight;
		self.ward = self.ward_max * HuntRules::RECOVER_SHARE;
		self.cadence = 0;
		// It pulls itself together and is off at once.
		self.next_bolt_tick = self.tick + 30;
		self.schedule_shriek();
		self.schedule_charge();
		events.push(HuntEvent::Recovered {
			lapses_left: self.lapses_left,
		});
	}

	// The orb ------------------------------------------------------------

	/// ORB was let go after `held` ticks, with the crosshair along `aim`.
	fn throw_orb(&mut self, aim: Vec3, held: i64, events: &mut Vec<HuntEvent>) {
		if self.orb_block().is_some() {
			return;
		}

		let current = self.pose();
		let on_target = current.covers(aim, self.stats.orb_aim_radians);
		let grade = CaptureRules::grade_for(held, current.covers(aim, 0.0));

		if !self.in_rift {
			self.soul -= CaptureRules::COST;
		}

		let target = if on_target {
			current.position
		} else {
			self.landing_along(aim)
		};

		self.orb = Some(Orb {
			thrown_at: self.tick,
			grade,
			on_target,
			target,
			chance: None,
			break_on: None,
			broke_on: None,
			held: false,
		});
		events.push(HuntEvent::OrbThrown { grade, on_target });

		if !on_target {
			return;
		}

		// The husk sees it coming and can do nothing more: a charge or a
		// shriek is called off, and it is held where it stands.
		self.phase = HuntPhase::Capture;
		self.settle(events);
	}

	/// Follows a missed orb down. It plays no part in the fight.
	fn step_missed_orb(&mut self, events: &mut Vec<HuntEvent>) {
		if let Some(orb) = self.orb
			&& !orb.on_target
			&& self.tick - orb.thrown_at == CaptureRules::IMPACT_TICK
		{
			events.push(HuntEvent::OrbMissed);
		}
	}

	/// One tick of a capture. The fight clock is paused while an orb holds
	/// the husk: it can do nothing, and a hunt must not time out during a
	/// capture.
	fn step_capture(&mut self, input: &HuntInput, events: &mut Vec<HuntEvent>) {
		if self.leaping {
			self.continue_leap(events);
		}

		self.step_reload(events);
		self.step_dive_charges();
		self.step_art_recharge();
		self.select(usize::from(input.slot), events);

		let Some(orb) = self.orb else {
			return;
		};
		let ticks = self.tick - orb.thrown_at;

		if ticks == CaptureRules::IMPACT_TICK {
			let chance = if self.in_rift {
				1.0
			} else {
				CaptureRules::chance(
					self.rarity_odds,
					self.ward / self.ward_max,
					orb.grade,
					self.stats.capture,
					self.failures,
				)
			};

			if let Some(orb) = self.orb.as_mut() {
				orb.chance = Some(chance);
			}
			events.push(HuntEvent::OrbImpact { chance });

			// In a rift a landed orb is the reaper's cut, and nothing is
			// rolled.
			if self.in_rift {
				self.captured(orb.grade, events);
				return;
			}

			let roll = CaptureRules::roll(self.seed, self.orbs_landed);
			self.orbs_landed += 1;

			if let Some(orb) = self.orb.as_mut() {
				orb.break_on = CaptureRules::break_pulse(roll, chance);
			}
		}

		if ticks == CaptureRules::DRAWN_TICK {
			events.push(HuntEvent::OrbAbsorbed);
		}

		if ticks == CaptureRules::LANDED_TICK {
			events.push(HuntEvent::OrbLanded);
		}

		for (index, &pulse_tick) in CaptureRules::PULSE_TICKS.iter().enumerate() {
			if ticks != pulse_tick {
				continue;
			}

			let pulse = index as i64 + 1;

			if self.orb.and_then(|orb| orb.break_on) == Some(pulse) {
				self.burst_out(pulse, events);
				return;
			}

			events.push(HuntEvent::OrbPulse(pulse));
		}

		if ticks >= CaptureRules::RESULT_TICK {
			let grade = self.orb.map_or(CaptureGrade::Plain, |orb| orb.grade);

			self.captured(grade, events);
		}
	}

	fn captured(&mut self, grade: CaptureGrade, events: &mut Vec<HuntEvent>) {
		if let Some(orb) = self.orb.as_mut() {
			orb.held = true;
		}
		self.phase = HuntPhase::Absorbed;
		self.outcome = Some(HuntOutcome::absorbed_with(
			grade,
			self.ambushed,
			self.tick + 1,
			self.deeds(),
		));
		events.push(HuntEvent::HuskCaptured(grade));
	}

	/// The orb cracked on `pulse`. The husk bursts out where it lay, with a
	/// little of its ward back, and stands stunned before the fight goes on.
	fn burst_out(&mut self, pulse: i64, events: &mut Vec<HuntEvent>) {
		if let Some(orb) = self.orb.as_mut() {
			orb.broke_on = Some(pulse);
		}
		self.failures += 1;
		self.phase = HuntPhase::Fight;
		self.ward = math::min(
			self.ward + self.ward_max * CaptureRules::FAILURE_WARD,
			self.ward_max * CaptureRules::WEAKENED_SHARE,
		);
		self.stun(CaptureRules::FAILURE_STUN_TICKS);
		// The fight starts over around it: the next shriek and charge are as
		// far off as ever, well past the stun.
		self.schedule_shriek();
		self.schedule_charge();

		events.push(HuntEvent::OrbBrokeFree(pulse));
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

	// Movement -----------------------------------------------------------

	/// Moves the husk on by one tick: along the move it is making, or into a
	/// new one when it is time to amble or to bolt.
	fn move_husk(&mut self, events: &mut Vec<HuntEvent>) {
		if self.vessel.gait() == Gait::Still {
			return;
		}

		if self.walker.is_some() {
			self.walk(events);
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
		if self.walker.is_some() {
			self.amble_walking();
			return;
		}

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

		if self.walker.is_some() {
			self.bolt_walking(events);
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
		if self.walker.is_some() {
			self.walk_down(may_stay);
			return;
		}

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

	/// Stops the husk where it stands. A leap cannot be stopped in the air.
	fn halt(&mut self) {
		if self.walker.is_some() {
			self.halt_walker();
			return;
		}

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
