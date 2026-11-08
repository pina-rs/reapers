//! One rift, from the first reaper arriving to the last cut.

use thiserror::Error;

use crate::armory::Temper;
use crate::armory::Weapon;
use crate::geo::GeoPoint;
use crate::hunt::hunt_bot::HuntBot;
use crate::hunt::hunt_bot::HuntBotStyle;
use crate::hunt::hunt_event::EscapeCause;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntOutcome;
use crate::hunt::hunt_input::Aim;
use crate::hunt::hunt_input::HuntInput;
use crate::hunt::hunt_sim::HuntSim;
use crate::hunt::hunt_stats::HuntLoadout;
use crate::hunt::hunt_stats::HuntStats;
use crate::hunt::hunt_stats::InvalidLoadout;
use crate::math;
use crate::reaper_class::ReaperClass;
use crate::reaper_names::REAPER_GIVEN_NAMES;
use crate::rift::rift_frame::RiftFrame;
use crate::rift::rift_frame::RiftMember;
use crate::rift::rift_frame::RiftPhase;
use crate::rift::rift_frame::RiftReport;
use crate::rift::rift_rules::RiftRules;
use crate::rift::rift_site::RiftSite;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;
use crate::vec3::wrap_angle;

/// Where one reaper sees a rift's husk: which way it lies from them, and how
/// far off it is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiftSighting {
	/// The way to the husk in the reaper's own view, in radians to the right
	/// of where they first faced.
	pub bearing: f64,

	/// How far away the husk is drawn.
	pub metres: f64,
}

impl RiftSighting {
	/// Where a reaper `standing` sees a rift that stands at `anchor`, or
	/// `None` when they are too far away to be said to be there.
	///
	/// `north` is where north lies in the reaper's own view, as a bearing in
	/// the space their hunt is played in: it is what turns a direction on
	/// the earth into a direction the phone can be pointed.
	pub fn of(anchor: &GeoPoint, standing: &GeoPoint, north: f64) -> Option<Self> {
		let metres = standing.distance_to(anchor);

		if metres > RiftRules::SIGHT_METRES {
			return None;
		}

		Some(Self {
			bearing: wrap_angle(north + standing.bearing_to(anchor)),
			metres: math::clamp(
				metres,
				RiftRules::NEAREST_METRES,
				RiftRules::FARTHEST_METRES,
			),
		})
	}
}

/// A reaper arriving at a rift: who they are and what they fight with.
#[derive(Clone, Debug, PartialEq)]
pub struct RiftEntrant {
	pub name: String,

	pub loadout: HuntLoadout,
}

/// What sealing a rift paid one reaper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RiftReward {
	pub xp: i64,
	pub anima: i64,
}

/// Why a reaper could not take a seat in a rift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum RiftError {
	/// The circle is full, or the fight has begun.
	#[error("This rift is not taking reapers")]
	NotTakingReapers,

	/// The reaper could not have the loadout they came with.
	#[error(transparent)]
	InvalidLoadout(#[from] InvalidLoadout),
}

/// One reaper's own fight inside a rift.
///
/// Each reaper faces the husk themselves, with their own seals to strike and
/// their own arc to cut. What they share is the ward: every strike anyone
/// lands comes off the same total.
#[derive(Clone, Debug)]
pub struct RiftSeat {
	pub sim: HuntSim,

	/// Ward this seat has removed since the fight began.
	pub dealt: f64,
}

impl RiftSeat {
	/// `sighting` is where this reaper sees the husk. Without one it stands
	/// dead ahead of them.
	pub fn new(
		seed: i64,
		site: RiftSite,
		ward_max: f64,
		loadout: &HuntLoadout,
		sighting: Option<RiftSighting>,
	) -> Result<Self, InvalidLoadout> {
		Ok(Self {
			sim: HuntSim::rift(
				seed,
				site.vessel(),
				ward_max,
				&HuntStats::derive(loadout, 1.0)?,
				sighting,
			),
			dealt: 0.0,
		})
	}

	/// Advances this reaper's fight by one tick.
	pub fn step(&mut self, input: &HuntInput) -> Vec<HuntEvent> {
		let events = self.sim.step(input);

		self.tally(&events);

		events
	}

	/// Counts the ward `events` removed toward this seat's total.
	/// [`step`](Self::step) does this itself; it is for whoever advances
	/// `sim` some other way.
	pub fn tally(&mut self, events: &[HuntEvent]) {
		for event in events {
			if let HuntEvent::StrikeLanded { damage, .. } | HuntEvent::StrikeGlanced { damage } =
				event
			{
				self.dealt += damage;
			}
		}
	}

	/// Brings this seat's view of the ward up to date.
	///
	/// `ward` is the circle's ward as the rift last reported it, and
	/// `counted` is how much of this seat's own damage that figure already
	/// includes. Strikes the rift has not heard of yet are taken off as
	/// well, so a blow shows the moment it lands instead of when the news
	/// comes back.
	pub fn sync(&mut self, ward: f64, counted: f64) {
		if !self.sim.is_over() {
			self.sim.share_ward(ward - (self.dealt - counted));
		}
	}

	/// Plays `ticks` of standing still, to join a fight that has already
	/// started in step with everyone else in it.
	pub fn catch_up(&mut self, ticks: i64) {
		let idle = HuntInput::new(Aim::AHEAD, 0);

		for _ in 0..ticks {
			if self.sim.is_over() {
				break;
			}

			self.step(&idle);
		}
	}

	/// This seat's standing, to send to the rift.
	pub fn report(&self) -> RiftReport {
		RiftReport {
			ready: true,
			dealt: self.dealt,
			outcome: self.sim.outcome(),
			standing: None,
		}
	}
}

/// One member of the circle, as the room keeps them.
#[derive(Clone, Debug)]
struct Member {
	entrant: RiftEntrant,
	shade: bool,
	ready: bool,
	left: bool,
	dealt: f64,
	outcome: Option<HuntOutcome>,

	/// A shade's own fight, played by the room, and the bot that plays it.
	/// `None` for a person, whose fight is played on their phone.
	fight: Option<(RiftSeat, HuntBot)>,
}

impl Member {
	fn new(entrant: RiftEntrant, shade: bool) -> Self {
		Self {
			entrant,
			shade,
			ready: false,
			left: false,
			dealt: 0.0,
			outcome: None,
			fight: None,
		}
	}

	/// Still in the fight: here, and not cast out by flinching.
	fn standing(&self) -> bool {
		!self.left
			&& self.outcome.and_then(|outcome| outcome.escape()) != Some(EscapeCause::Resolve)
	}
}

/// One rift from the first reaper arriving to the last cut.
///
/// The room is the keeper of everything the circle shares: who is in it,
/// how much ward is left, and whether the rift was sealed. It reads no clock
/// and rolls no dice of its own: whoever hosts it, the game alone or the
/// server for many, calls [`advance`](Self::advance) as time passes and
/// [`report`](Self::report) as reapers act.
#[derive(Clone, Debug)]
pub struct RiftRoom {
	site: RiftSite,
	seed: i64,
	members: Vec<Member>,
	phase: RiftPhase,
	tick: i64,
	ward_max: f64,
	anchor: Option<GeoPoint>,
	gather_ticks: i64,
	countdown: Option<i64>,
	reap_ends_tick: i64,
}

impl RiftRoom {
	/// A reap is given this long past its window before the room gives up on
	/// reapers who have not said how theirs ended.
	const REPORT_GRACE_TICKS: i64 = 45;

	pub const fn new(site: RiftSite, seed: i64) -> Self {
		Self {
			site,
			seed,
			members: Vec::new(),
			phase: RiftPhase::Gathering,
			tick: 0,
			ward_max: 0.0,
			anchor: None,
			gather_ticks: 0,
			countdown: None,
			reap_ends_tick: 0,
		}
	}

	pub const fn site(&self) -> RiftSite {
		self.site
	}

	/// Decides the husk's timetable and who the shades are.
	pub const fn seed(&self) -> i64 {
		self.seed
	}

	pub const fn phase(&self) -> RiftPhase {
		self.phase
	}

	/// Ticks since the fight began.
	pub const fn tick(&self) -> i64 {
		self.tick
	}

	/// The ward the husk raised against this circle. Zero until the fight
	/// begins, since it depends on who came.
	pub const fn ward_max(&self) -> f64 {
		self.ward_max
	}

	/// Where on the earth the rift stands: decided by the first reaper to
	/// say where they are, and the same for everyone after. `None` until
	/// then, and for good if nobody in the circle can say.
	pub const fn anchor(&self) -> Option<GeoPoint> {
		self.anchor
	}

	/// Whether another reaper may still join.
	pub fn is_open(&self) -> bool {
		self.phase == RiftPhase::Gathering && self.present().count() < RiftRules::CIRCLE
	}

	/// Whether any reaper who joined is still here.
	pub fn is_abandoned(&self) -> bool {
		self.present().all(|member| member.shade)
	}

	/// What is left of the ward.
	pub fn ward(&self) -> f64 {
		let mut dealt = 0.0;

		for member in &self.members {
			dealt += member.dealt;
		}

		math::max(0.0, self.ward_max - dealt)
	}

	fn present(&self) -> impl Iterator<Item = &Member> {
		self.members.iter().filter(|member| !member.left)
	}

	/// Seats `entrant` in the circle and returns their seat.
	pub fn join(&mut self, entrant: RiftEntrant) -> Result<usize, RiftError> {
		if !self.is_open() {
			return Err(RiftError::NotTakingReapers);
		}

		HuntStats::derive(&entrant.loadout, 1.0)?;
		self.members.push(Member::new(entrant, false));

		Ok(self.members.len() - 1)
	}

	/// Records that the reaper in `seat` has gone. What they dealt still
	/// counts, but they no longer stand in the circle. Once the rift is over
	/// there is nothing left to walk away from, and leaving changes nothing.
	/// A seat nobody was given is ignored.
	pub fn leave(&mut self, seat: usize) {
		if self.phase.is_over() {
			return;
		}

		let Some(member) = self.members.get_mut(seat) else {
			return;
		};

		member.left = true;

		if self.phase == RiftPhase::Gathering {
			self.countdown = None;
		}
	}

	/// Takes in what the reaper in `seat` says of their own fight. A seat
	/// nobody was given is ignored.
	pub fn report(&mut self, seat: usize, report: &RiftReport) {
		let phase = self.phase;
		let site = self.site;
		let Some(member) = self.members.get_mut(seat) else {
			return;
		};

		if member.left || phase.is_over() {
			return;
		}

		member.ready = member.ready || report.ready;

		if phase == RiftPhase::Gathering {
			// The rift takes its place while the circle gathers, and keeps
			// it: moving it mid-fight would turn every reaper's phone away
			// from it.
			if let Some(standing) = &report.standing
				&& self.anchor.is_none()
			{
				self.anchor = Some(site.anchor_for(standing));
			}

			return;
		}

		member.dealt = math::max(member.dealt, report.dealt);
		member.outcome = member.outcome.or(report.outcome);
	}

	/// Moves the rift on by `ticks` ticks.
	pub fn advance(&mut self, ticks: i64) {
		for _ in 0..ticks {
			if self.phase.is_over() {
				break;
			}

			match self.phase {
				RiftPhase::Gathering => self.gather(),
				RiftPhase::Fight => self.fight(),
				RiftPhase::Reap => self.reap(),
				RiftPhase::Sealed | RiftPhase::Failed => {}
			}
		}
	}

	/// The rift as the reaper in `seat` sees it right now, or `None` for a
	/// seat nobody was given.
	pub fn frame_for(&self, seat: usize) -> Option<RiftFrame> {
		let you = self.members.get(seat)?;
		let reward = self.reward_for(seat);
		let shown = if you.left {
			-1
		} else {
			self.members[..seat]
				.iter()
				.filter(|member| !member.left)
				.count() as i64
		};

		Some(RiftFrame {
			site: self.site,
			seed: self.seed,
			phase: self.phase,
			seat: shown,
			tick: self.tick,
			ticks_left: match self.phase {
				RiftPhase::Gathering => {
					self.countdown
						.unwrap_or(RiftRules::GATHER_LIMIT_TICKS - self.gather_ticks)
				}
				RiftPhase::Fight => RiftRules::FIGHT_LIMIT_TICKS - self.tick,
				RiftPhase::Reap => (self.reap_ends_tick - self.tick).max(0),
				RiftPhase::Sealed | RiftPhase::Failed => 0,
			},
			counting_down: self.countdown.is_some(),
			ward: self.ward(),
			ward_max: self.ward_max,
			members: self
				.present()
				.map(|member| {
					RiftMember {
						name: member.entrant.name.clone(),
						reaper_class: member.entrant.loadout.reaper_class,
						level: member.entrant.loadout.level,
						shade: member.shade,
						ready: member.ready,
						standing: member.standing(),
						dealt: member.dealt,
						finished: member.outcome.is_some(),
						cut: member.outcome.and_then(|outcome| outcome.grade()),
					}
				})
				.collect(),
			xp: reward.xp,
			anima: reward.anima,
			anchor: self.anchor,
		})
	}

	/// What the rift paid the reaper in `seat`. Nothing unless it was sealed
	/// and they stayed to the end.
	pub fn reward_for(&self, seat: usize) -> RiftReward {
		let Some(member) = self.members.get(seat) else {
			return RiftReward::default();
		};

		if self.phase != RiftPhase::Sealed || member.left {
			return RiftReward::default();
		}

		let fair_share = self.ward_max / RiftRules::CIRCLE as f64;
		let pull = RiftRules::IDLE_SHARE
			+ (1.0 - RiftRules::IDLE_SHARE) * math::min(1.0, member.dealt / fair_share);
		let part = match member.outcome.and_then(|outcome| outcome.grade()) {
			Some(grade) => grade.reward(),
			None if !member.standing() => RiftRules::CAST_OUT_SHARE,
			None => RiftRules::UNCUT_SHARE,
		};

		RiftReward {
			xp: math::round(RiftRules::XP as f64 * pull * part),
			anima: math::round(RiftRules::ANIMA as f64 * pull * part),
		}
	}

	// Gathering ----------------------------------------------------------

	fn gather(&mut self) {
		self.gather_ticks += 1;

		if let Some(countdown) = self.countdown {
			self.countdown = Some(countdown - 1);

			if countdown <= 1 {
				self.begin();
			}

			return;
		}

		let anyone = self.present().next().is_some();

		if anyone && self.present().all(|member| member.ready) {
			self.countdown = Some(RiftRules::COUNTDOWN_TICKS);
			return;
		}

		// Nobody waits forever on a reaper who walked away with the app
		// open.
		if anyone && self.gather_ticks >= RiftRules::GATHER_LIMIT_TICKS {
			self.countdown = Some(RiftRules::COUNTDOWN_TICKS);
		}
	}

	fn begin(&mut self) {
		let reapers = self.present().count();

		if reapers == 0 {
			self.phase = RiftPhase::Failed;
			return;
		}

		let levels: i64 = self
			.present()
			.map(|member| member.entrant.loadout.level)
			.sum();
		let shade_level =
			1_i64.max(math::round(levels as f64 / reapers as f64) - RiftRules::SHADE_LEVEL_GAP);

		for empty in reapers..RiftRules::CIRCLE {
			let mut shade = Member::new(self.shade(empty, shade_level), true);

			shade.ready = true;
			self.members.push(shade);
		}

		let circle: Vec<HuntStats> = self
			.present()
			.filter_map(|member| HuntStats::derive(&member.entrant.loadout, 1.0).ok())
			.collect();

		self.ward_max = RiftRules::ward_for(&circle);

		let (seed, site, ward_max) = (self.seed, self.site, self.ward_max);

		for member in self
			.members
			.iter_mut()
			.filter(|member| !member.left && member.shade)
		{
			if let Ok(seat) = RiftSeat::new(seed, site, ward_max, &member.entrant.loadout, None) {
				// A shade is slower to see a seal and wider of the mark than
				// a careful person, so company is always worth more than
				// shades.
				let bot = HuntBot::new(
					&seat.sim,
					HuntBotStyle {
						reaction_ticks: 34,
						miss_every: 3,
						..HuntBotStyle::CAREFUL
					},
				);

				member.fight = Some((seat, bot));
			}
		}

		self.countdown = None;
		self.phase = RiftPhase::Fight;
		self.tick = 0;
	}

	/// The shade that fills seat `index` when nobody else does.
	fn shade(&self, index: usize, level: i64) -> RiftEntrant {
		let mut random = SeededRandom::new(mix_seed(&[self.seed, 0x5AADE, index as i64]));
		let reaper_class = random.pick(&ReaperClass::ALL);
		let temper = Temper::best_for(level);

		RiftEntrant {
			name: random.pick(&REAPER_GIVEN_NAMES).to_owned(),
			loadout: HuntLoadout::new(
				reaper_class,
				level,
				Weapon::new(reaper_class.weapon(), temper),
			),
		}
	}

	// Fight and reap -----------------------------------------------------

	fn fight(&mut self) {
		self.tick += 1;
		self.step_shades();

		if self.ward() <= 0.0 {
			self.phase = RiftPhase::Reap;
			self.reap_ends_tick = self.tick + RiftRules::REAP_WINDOW_TICKS;
			return;
		}

		let out_of_time = self.tick >= RiftRules::FIGHT_LIMIT_TICKS;
		let nobody_left = !self.present().any(Member::standing);

		if out_of_time || nobody_left {
			self.phase = RiftPhase::Failed;
		}
	}

	fn reap(&mut self) {
		self.tick += 1;
		self.step_shades();

		let standing = self.present().filter(|member| member.standing()).count();
		let everyone_done = self
			.present()
			.filter(|member| member.standing())
			.all(|member| member.outcome.is_some());

		if !everyone_done && self.tick < self.reap_ends_tick + Self::REPORT_GRACE_TICKS {
			return;
		}

		let cuts: Vec<&Member> = self
			.present()
			.filter(|member| member.outcome.and_then(|outcome| outcome.grade()).is_some())
			.collect();
		let enough = cuts.len() >= RiftFrame::cuts_needed_for(standing);
		// Shades can break a ward, but only the living can shut a rift: at
		// least one of the cuts has to be a person's.
		let by_the_living = cuts.iter().any(|member| !member.shade);

		self.phase = if enough && by_the_living {
			RiftPhase::Sealed
		} else {
			RiftPhase::Failed
		};
	}

	fn step_shades(&mut self) {
		let shared = self.ward();

		for member in &mut self.members {
			let Some((seat, bot)) = &mut member.fight else {
				continue;
			};

			if seat.sim.is_over() {
				continue;
			}

			seat.sync(shared, seat.dealt);
			let input = bot.decide(&seat.sim);
			seat.step(&input);
			member.dealt = seat.dealt;
			member.outcome = seat.sim.outcome();
		}
	}
}
