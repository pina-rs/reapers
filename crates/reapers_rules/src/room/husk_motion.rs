//! One husk's place in the room, moved one tick at a time.

use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::room::leap::Leap;
use crate::room::locomotion_profile::LocomotionProfile;
use crate::room::room_grid::RoomGrid;
use crate::room::room_nav::RoomNav;
use crate::room::room_nav::RoomSpot;
use crate::room::room_nav::RoomSpotKind;
use crate::room::room_nav::RoomStep;
use crate::seeded_random::SeededRandom;
use crate::vec3::Vec3;
use crate::vec3::radians;
use crate::vec3::wrap_angle;

/// The pace a husk keeps when it has somewhere to go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionPace {
	/// Its unhurried walk.
	Walk,

	/// Its flight speed.
	Run,
}

/// What a husk is doing with its body at one instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HuskMotionState {
	/// Standing still, looking about.
	Idle,

	/// Turning on the spot because the way it must go is behind it.
	Turn,

	/// Walking a path.
	Walk,

	/// Running a path.
	Run,

	/// Decelerating to a stop at the edge of a leap, facing the landing.
	Edge,

	/// Gathering itself before take-off.
	Crouch,

	/// In the air on a ballistic arc, or flying.
	Air,

	/// Touching down, with a dip scaled by the impact.
	Land,

	/// Standing up, scaled by how far it fell.
	Recover,
}

/// One husk's place in the room, moved one tick at a time.
///
/// The rules own the root: where it is, which way it faces, what it is
/// doing and how far through that it is. A view draws exactly this and
/// adds the legs, the squash and the gaze. Nothing a strike does is
/// decided here.
///
/// A walk accelerates at no more than [`HuskMotion::ACCELERATION`] and
/// turns at no more than [`HuskMotion::TURN_RATE`], so a path that turns
/// a corner is walked as a body walks it. A leap is the exact parabola
/// from [`Leap`], and the surface under the root changes in one step when
/// the body crosses an edge, never between ticks.
///
/// The only variation comes from `random`: how long a crouch gathers,
/// and how long a landing takes to stand up. The same seed and the same
/// commands make the same motion, bit for bit.
pub struct HuskMotion<'a> {
	/// The room this husk moves through.
	pub nav: &'a RoomNav,

	/// Where chance comes from: only the lengths of a crouch and a
	/// recovery read it.
	pub random: SeededRandom,

	position: Vec3,
	heading: f64,
	state: HuskMotionState,
	state_fraction: f64,
	support_height: f64,
	speed: f64,
	distance_travelled: f64,
	look_at: Option<Vec3>,
	cell: usize,

	steps: Option<Vec<RoomStep>>,
	step_index: usize,
	pace: MotionPace,
	target_speed: f64,

	pending_target: Option<Vec3>,
	pending_spot: Option<RoomSpot>,
	needs_path: bool,
	halt_requested: bool,
	face_target: Option<f64>,
	arrival_facing: Option<Vec3>,

	edge_entry_distance: f64,

	leap: Option<Leap>,
	landing_cell: i64,
	landing_region: i64,
	crouch_tick: i64,
	crouch_ticks: i64,
	air_tick: i64,

	land_tick: i64,
	land_ticks: i64,
	land_dip: f64,
	land_speed: f64,
	land_heading: f64,

	recover_tick: i64,
	recover_ticks: i64,
	fall_height: f64,

	flight_waypoints: Vec<Vec3>,
	flight_index: usize,
	perch_target: Option<RoomSpot>,
	perched: bool,
}

impl<'a> HuskMotion<'a> {
	/// How fast a walking body may change its speed, in metres a second
	/// squared.
	pub const ACCELERATION: f64 = 4.0;
	/// How close a walking body must come to a waypoint to have reached
	/// it.
	pub const ARRIVAL_RADIUS: f64 = 0.02;
	/// How far a walking body may turn in one second, in radians.
	pub const TURN_RATE: f64 = radians(240.0);

	/// Sets a husk down at `start`, or at the player's cell, facing
	/// `heading` radians to the right of straight ahead.
	///
	/// Returns nothing when there is nowhere near `start` the body can
	/// stand, which a room too narrow for this vessel causes.
	pub fn new(
		nav: &'a RoomNav,
		random: SeededRandom,
		start: Option<Vec3>,
		heading: f64,
	) -> Option<Self> {
		let place = start.unwrap_or_else(|| {
			nav.grid
				.centre_of(i64::from(nav.grid.player_x), i64::from(nav.grid.player_z))
		});
		let cell = nav.standable_cell_near(place, 20)?;
		let support_height = nav.support_height_of(cell)?;
		let within = nav.is_standable_at(
			nav.grid.cell_x_at(place.x, place.z),
			nav.grid.cell_z_at(place.x, place.z),
		);
		let here = if within {
			place
		} else {
			nav.grid.centre_of(nav.grid.x_of(cell), nav.grid.z_of(cell))
		};

		Some(Self {
			nav,
			random,
			position: Vec3::new(here.x, support_height + nav.profile.half_height(), here.z),
			heading: wrap_angle(heading),
			state: HuskMotionState::Idle,
			state_fraction: 0.0,
			support_height,
			speed: 0.0,
			distance_travelled: 0.0,
			look_at: None,
			cell,
			steps: None,
			step_index: 0,
			pace: MotionPace::Walk,
			target_speed: 0.0,
			pending_target: None,
			pending_spot: None,
			needs_path: false,
			halt_requested: false,
			face_target: None,
			arrival_facing: None,
			edge_entry_distance: 0.01,
			leap: None,
			landing_cell: -1,
			landing_region: -1,
			crouch_tick: 0,
			crouch_ticks: 0,
			air_tick: 0,
			land_tick: 0,
			land_ticks: 0,
			land_dip: 0.0,
			land_speed: 0.0,
			land_heading: 0.0,
			recover_tick: 0,
			recover_ticks: 0,
			fall_height: 0.0,
			flight_waypoints: Vec::new(),
			flight_index: 0,
			perch_target: None,
			perched: false,
		})
	}

	/// The body and its jumping numbers.
	pub fn profile(&self) -> &LocomotionProfile {
		&self.nav.profile
	}

	/// The middle of the body, in the hunt's frame.
	pub const fn position(&self) -> Vec3 {
		self.position
	}

	/// Which way the body faces, in radians to the right of straight
	/// ahead.
	pub const fn heading(&self) -> f64 {
		self.heading
	}

	/// What the body is doing.
	pub const fn state(&self) -> HuskMotionState {
		self.state
	}

	/// How far through the current state it is, from 0 to 1.
	pub const fn state_fraction(&self) -> f64 {
		self.state_fraction
	}

	/// How high the surface under the root is, in the hunt's frame. It
	/// changes in one step when the body crosses an edge.
	pub const fn support_height(&self) -> f64 {
		self.support_height
	}

	/// The cell the root is over.
	pub const fn cell(&self) -> usize {
		self.cell
	}

	/// How fast the body is travelling over the ground, in metres a
	/// second.
	pub const fn speed(&self) -> f64 {
		self.speed
	}

	/// How far the body has travelled, in metres, for lining a stride up
	/// with the ground it covers.
	pub const fn distance_travelled(&self) -> f64 {
		self.distance_travelled
	}

	/// The point the husk is looking at: the next waypoint, or the
	/// landing point while it gathers and leaps.
	pub const fn look_at(&self) -> Option<Vec3> {
		self.look_at
	}

	/// Where the feet will come down next: the landing of a leap that
	/// has begun, or the cell the walk is making for.
	///
	/// Nothing once the husk has arrived, so a view drawing a waiting
	/// husk has no stale foothold to reach for.
	pub fn next_foothold(&self) -> Option<Vec3> {
		match self.state {
			HuskMotionState::Edge => {
				self.current_step()
					.and_then(|step| step.leap)
					.map(|link| link.leap.to)
			}
			HuskMotionState::Crouch | HuskMotionState::Air => self.leap.map(|leap| leap.to),
			// A landing husk is already standing on the ground it
			// reaches, so it names the next step it will walk from.
			HuskMotionState::Land | HuskMotionState::Recover => {
				self.steps
					.as_ref()
					.and_then(|steps| steps.get(self.step_index + 1))
					.map(|step| step.point)
			}
			HuskMotionState::Idle
			| HuskMotionState::Turn
			| HuskMotionState::Walk
			| HuskMotionState::Run => self.current_step().map(|step| step.point),
		}
	}

	/// Whether the husk has a route to follow.
	pub fn has_route(&self) -> bool {
		self.step_index < self.steps.as_ref().map_or(0, Vec::len)
	}

	/// Whether the husk is standing still with nothing to do.
	pub fn is_idle(&self) -> bool {
		self.state == HuskMotionState::Idle && !self.needs_path && !self.has_route()
	}

	/// Sends the husk to `point`, walking there unless `pace` says
	/// otherwise.
	///
	/// A command given in the air, or while gathering, is remembered and
	/// taken up once the body is on the ground again.
	pub fn go_to(&mut self, point: Vec3, pace: MotionPace) {
		self.pending_target = Some(point);
		self.pending_spot = None;
		self.pace = pace;
		self.needs_path = true;
		self.halt_requested = false;
		self.arrival_facing = None;
	}

	/// Sends the husk to `spot`. A perch is landed on; any other is
	/// walked or flown to.
	pub fn go_to_spot(&mut self, spot: RoomSpot, pace: MotionPace) {
		self.pending_target = Some(spot.point);
		self.pending_spot = Some(spot);
		self.pace = pace;
		self.needs_path = true;
		self.halt_requested = false;
		self.arrival_facing = spot.facing;
	}

	/// Stops the husk where it is. A leap already begun cannot be
	/// stopped: the husk lands first.
	pub fn halt(&mut self) {
		self.pending_target = None;
		self.pending_spot = None;
		self.needs_path = false;
		self.halt_requested = true;
		self.steps = None;
		self.step_index = 0;
		self.flight_waypoints = Vec::new();
		self.arrival_facing = None;

		if self.speed == 0.0
			&& !matches!(
				self.state,
				HuskMotionState::Air
					| HuskMotionState::Crouch
					| HuskMotionState::Land
					| HuskMotionState::Recover
			) {
			self.state = HuskMotionState::Idle;
			self.state_fraction = 0.0;
			self.look_at = None;
		}
	}

	/// Turns the body to face `direction`, in place when it is standing
	/// still.
	pub fn face(&mut self, direction: Vec3) {
		if direction.x == 0.0 && direction.z == 0.0 {
			return;
		}

		self.face_target = Some(wrap_angle(math::atan2(direction.x, direction.z)));
		self.halt_requested = false;
	}

	/// Advances the body by one tick of 1/60 s.
	pub fn tick(&mut self) {
		if self.profile().flies() {
			self.advance_flight();

			return;
		}

		if self.profile().walk_speed <= 0.0 && self.profile().run_speed <= 0.0 {
			return;
		}

		match self.state {
			HuskMotionState::Air => self.advance_air(),
			HuskMotionState::Crouch => self.advance_crouch(),
			HuskMotionState::Land => self.advance_land(),
			HuskMotionState::Recover => self.advance_recover(),
			HuskMotionState::Idle
			| HuskMotionState::Turn
			| HuskMotionState::Walk
			| HuskMotionState::Run
			| HuskMotionState::Edge => self.advance_ground(),
		}
	}
}

impl HuskMotion<'_> {
	/// The room this husk moves through, as its grid.
	pub(crate) fn grid(&self) -> &RoomGrid {
		&self.nav.grid
	}

	/// Follows the route, or stands and turns where it is.
	fn advance_ground(&mut self) {
		if self.needs_path {
			self.start_route();
		}

		if self.state == HuskMotionState::Edge {
			self.advance_edge();

			return;
		}

		let Some(step) = self.current_step() else {
			self.stand_and_face();

			return;
		};
		let distance = flat_distance(self.position, step.point);

		if step.leap.is_some() {
			let stopping =
				self.speed * self.speed / (2.0 * Self::ACCELERATION) + Self::ARRIVAL_RADIUS;

			if distance <= stopping {
				self.begin_edge(&step, distance);

				return;
			}
		} else if distance <= Self::ARRIVAL_RADIUS {
			self.step_index += 1;

			if !self.has_route() {
				self.finish_route();
			}

			return;
		}

		let desired = math::atan2(
			step.point.x - self.position.x,
			step.point.z - self.position.z,
		);
		let error = wrap_angle(desired - self.heading);
		let turn_step = Self::TURN_RATE / HuntRules::TICK_RATE as f64;
		let must_turn = error.abs() > radians(60.0);
		let turn_only = self.speed < 0.02 && error.abs() > radians(5.0);

		self.heading = wrap_angle(self.heading + math::clamp(error, -turn_step, turn_step));

		let wanted = if must_turn || turn_only {
			0.0
		} else {
			self.target_speed
		};

		self.speed = move_towards(
			self.speed,
			wanted,
			Self::ACCELERATION / HuntRules::TICK_RATE as f64,
		);

		if self.speed > 0.0 {
			let travel = math::min(self.speed / HuntRules::TICK_RATE as f64, distance);

			if travel >= distance {
				let level_index = step.level_index;
				self.settle_on(step.point, level_index);
				self.distance_travelled += distance;

				if step.leap.is_some() {
					self.begin_edge(&step, 0.0);
				} else {
					self.step_index += 1;

					if !self.has_route() {
						self.finish_route();
					}
				}

				return;
			}

			let next = Vec3::new(
				self.position.x + math::sin(self.heading) * travel,
				self.position.y,
				self.position.z + math::cos(self.heading) * travel,
			);

			self.position = next;
			self.distance_travelled += travel;
			self.settle_on(next, step.level_index);
		}

		// The pace is set from the profile's own numbers, so equality is
		// exact: the state runs only when the target is the run speed.
		#[allow(clippy::float_cmp, reason = "both sides are the same constant, copied")]
		let run = self.target_speed == self.profile().run_speed;
		self.look_at = Some(step.point);
		self.state = if must_turn || turn_only {
			HuskMotionState::Turn
		} else if run {
			HuskMotionState::Run
		} else {
			HuskMotionState::Walk
		};
		self.state_fraction = if must_turn || turn_only {
			math::clamp(1.0 - error.abs() / math::PI, 0.0, 1.0)
		} else {
			0.0
		};
	}

	/// The step the husk is making for, when there is one ahead.
	fn current_step(&self) -> Option<RoomStep> {
		self.steps
			.as_ref()
			.and_then(|steps| steps.get(self.step_index))
			.copied()
	}

	/// Asks the room for a route to what was asked for.
	fn start_route(&mut self) {
		self.needs_path = false;
		let target = self.pending_target.take();
		let spot = self.pending_spot.take();

		let Some(target) = target else {
			return;
		};

		if self.profile().flies() {
			self.plan_flight(target, spot);

			return;
		}

		let Some(path) = self
			.nav
			.path(self.position, target)
			.filter(|path| !path.is_empty())
		else {
			self.steps = None;
			self.step_index = 0;
			self.speed = 0.0;
			self.state = HuskMotionState::Idle;
			self.state_fraction = 0.0;
			self.look_at = None;

			return;
		};

		self.steps = Some(path.steps);
		self.step_index = 0;
		self.target_speed = if self.pace == MotionPace::Run {
			math::max(self.profile().run_speed, self.profile().walk_speed)
		} else {
			self.profile().walk_speed
		};
		self.state = HuskMotionState::Walk;
		self.state_fraction = 0.0;
	}

	/// Begins easing to a stop at the take-off point, facing the landing.
	fn begin_edge(&mut self, step: &RoomStep, distance: f64) {
		if self.halt_requested {
			self.stop_at(step);

			return;
		}

		self.state = HuskMotionState::Edge;
		self.edge_entry_distance = math::max(distance, 0.01);
		self.state_fraction = math::clamp(1.0 - distance / self.edge_entry_distance, 0.0, 1.0);
		self.look_at = step.leap.map(|link| link.leap.to);
	}

	/// Decelerates to the take-off point and turns to face the leap.
	fn advance_edge(&mut self) {
		let Some(step) = self.current_step() else {
			self.stand_and_face();

			return;
		};
		let Some(link) = step.leap else {
			self.state = HuskMotionState::Walk;

			return;
		};

		if self.halt_requested {
			self.stop_at(&step);

			return;
		}

		let leap = link.leap;
		let distance = flat_distance(self.position, step.point);
		let desired = math::atan2(link.direction().x, link.direction().z);
		let error = wrap_angle(desired - self.heading);
		let turn_step = Self::TURN_RATE / HuntRules::TICK_RATE as f64;

		self.heading = wrap_angle(self.heading + math::clamp(error, -turn_step, turn_step));
		self.speed = math::min(
			self.speed,
			math::sqrt(2.0 * Self::ACCELERATION * math::max(distance, 0.0)),
		);

		if distance > 0.005 {
			let travel = math::min(self.speed / HuntRules::TICK_RATE as f64, distance);
			let next = Vec3::new(
				self.position.x + (step.point.x - self.position.x) / distance * travel,
				self.position.y,
				self.position.z + (step.point.z - self.position.z) / distance * travel,
			);

			self.position = next;
			self.distance_travelled += travel;
			self.settle_on(next, step.level_index);
		} else {
			self.settle_on(step.point, step.level_index);
			self.speed = 0.0;
		}

		let remaining = flat_distance(self.position, step.point);

		self.state_fraction = math::clamp(1.0 - remaining / self.edge_entry_distance, 0.0, 1.0);
		self.look_at = Some(leap.to);

		if self.speed <= 0.02 && remaining <= 0.01 && error.abs() <= radians(8.0) {
			self.speed = 0.0;
			self.landing_cell = link.to_cell as i64;
			self.landing_region = self
				.nav
				.region_of(link.to_cell)
				.map_or(-1, |region| region.id as i64);
			self.begin_crouch(leap);
		}
	}

	/// Gathers itself before take-off, for a time that scales with the
	/// height of the jump.
	fn begin_crouch(&mut self, leap: Leap) {
		self.state = HuskMotionState::Crouch;
		self.leap = Some(leap);
		self.crouch_tick = 0;

		let rise = math::max(0.0, leap.to.y - leap.from.y);
		let climb = if self.profile().jumps_up <= 0.0 {
			0.0
		} else {
			math::clamp(rise / self.profile().jumps_up, 0.0, 1.0)
		};

		self.crouch_ticks =
			(10 + math::round(10.0 * climb) + self.random.next_int(3)).clamp(10, 20);
		self.position = leap.from;
		self.support_height = leap.from.y - self.profile().half_height();
		self.cell = self.cell_at(self.position);
		self.look_at = Some(leap.to);
		self.state_fraction = 0.0;
	}

	/// Sinks, holds, and lets go.
	fn advance_crouch(&mut self) {
		let Some(leap) = self.leap else {
			self.state = HuskMotionState::Idle;

			return;
		};

		self.crouch_tick += 1;
		let progress = math::clamp(self.crouch_tick as f64 / self.crouch_ticks as f64, 0.0, 1.0);
		let dip = 0.15 * self.profile().half_height() * 4.0 * progress * (1.0 - progress);

		self.position = Vec3::new(leap.from.x, leap.from.y - dip, leap.from.z);
		self.state_fraction = progress;
		self.look_at = Some(leap.to);

		if self.crouch_tick >= self.crouch_ticks {
			self.position = leap.from;
			self.state = HuskMotionState::Air;
			self.air_tick = 0;
			self.state_fraction = 0.0;
		}
	}

	/// Flies the solved arc, one exact place a tick.
	fn advance_air(&mut self) {
		let Some(leap) = self.leap else {
			self.state = HuskMotionState::Idle;

			return;
		};

		self.air_tick += 1;

		if self.air_tick >= leap.ticks {
			self.position = leap.to;
			self.support_height = self
				.nav
				.support_height_of(self.landing_cell.max(0) as usize)
				.unwrap_or(leap.to.y - self.profile().half_height());
			self.cell = if self.landing_cell >= 0 {
				self.landing_cell as usize
			} else {
				self.cell_at(self.position)
			};
			self.fall_height = math::max(0.0, leap.apex_height - leap.to.y);
			let impact = math::clamp(leap.impact_speed(), 0.0, 20.0);

			self.land_dip = math::min(0.18, 0.03 + impact * 0.04);
			self.land_ticks =
				(6 + math::round(6.0 * math::clamp(impact / 3.0, 0.0, 1.0))).clamp(6, 12);
			self.land_speed = math::sqrt(
				leap.horizontal_velocity.x * leap.horizontal_velocity.x
					+ leap.horizontal_velocity.z * leap.horizontal_velocity.z,
			);
			self.land_heading = math::atan2(leap.horizontal_velocity.x, leap.horizontal_velocity.z);
			self.land_tick = 0;
			self.state = HuskMotionState::Land;
			self.state_fraction = 0.0;
			self.look_at = Some(leap.to);
			self.leap = None;

			return;
		}

		let before = self.position;

		self.position = leap.position_at(self.air_tick);
		self.distance_travelled += flat_distance(before, self.position);
		self.state_fraction = self.air_tick as f64 / leap.ticks as f64;
		self.look_at = Some(leap.to);

		let under = self.cell_at(self.position);

		if self
			.nav
			.region_of(under)
			.is_some_and(|region| region.id as i64 == self.landing_region)
		{
			self.support_height = self
				.nav
				.support_height_of(under)
				.unwrap_or(self.support_height);
			self.cell = under;
		}
	}

	/// Touches down, carrying a little of its speed, with a dip scaled by
	/// the impact.
	fn advance_land(&mut self) {
		self.land_tick += 1;
		let progress = math::clamp(self.land_tick as f64 / self.land_ticks as f64, 0.0, 1.0);

		self.state_fraction = progress;

		if self.land_speed > 0.0 {
			let carry = self.land_speed * (1.0 - progress) / HuntRules::TICK_RATE as f64;
			let next_x = self.position.x + math::sin(self.land_heading) * carry;
			let next_z = self.position.z + math::cos(self.land_heading) * carry;
			let next_cell = self.cell_at(Vec3::new(next_x, 0.0, next_z));

			if self
				.nav
				.region_of(next_cell)
				.is_some_and(|region| region.id as i64 == self.landing_region)
			{
				self.position = Vec3::new(next_x, self.position.y, next_z);
				self.cell = next_cell;
				self.distance_travelled += carry;
			}
		}

		let dip = self.land_dip * (1.0 - progress);

		self.position = Vec3::new(
			self.position.x,
			self.support_height + self.profile().half_height() - dip,
			self.position.z,
		);

		if self.land_tick >= self.land_ticks {
			self.position = Vec3::new(
				self.position.x,
				self.support_height + self.profile().half_height(),
				self.position.z,
			);
			self.recover_tick = 0;
			// Land and recover together are the 18 to 36 ticks the
			// research record allows: a deeper fall takes longer to shake
			// off, and the landing above takes 6 to 12 of them.
			self.recover_ticks = (12
				+ math::round(12.0 * math::clamp(self.fall_height / 1.5, 0.0, 1.0))
				+ self.random.next_int(4))
			.clamp(12, 24);
			self.state = HuskMotionState::Recover;
			self.state_fraction = 0.0;
		}
	}

	/// Stands up again, and takes up what is left of the route.
	fn advance_recover(&mut self) {
		self.recover_tick += 1;
		self.state_fraction = math::clamp(
			self.recover_tick as f64 / self.recover_ticks as f64,
			0.0,
			1.0,
		);
		self.position = Vec3::new(
			self.position.x,
			self.support_height + self.profile().half_height(),
			self.position.z,
		);

		if self.recover_tick < self.recover_ticks {
			return;
		}

		self.leap = None;
		self.landing_region = -1;
		self.speed = 0.0;

		let landing = self.step_index + 1;

		if let Some(steps) = &self.steps {
			// The step after the take-off is the landing. When it is also
			// the take-off of the next leap, stay on it so the next tick
			// gathers for that leap; otherwise walk on past it.
			self.step_index = if steps.get(landing).is_some_and(|step| step.leap.is_some()) {
				landing
			} else {
				landing + 1
			};

			if self.has_route() {
				self.state = HuskMotionState::Idle;
				self.state_fraction = 0.0;
			} else {
				self.finish_route();
			}
		} else {
			self.finish_route();
		}
	}

	/// Lets the route go, and arrives.
	fn finish_route(&mut self) {
		self.steps = None;
		self.step_index = 0;
		self.speed = 0.0;
		self.state = HuskMotionState::Idle;
		self.state_fraction = 0.0;
		self.look_at = None;

		if let Some(facing) = self.arrival_facing.take() {
			self.face_target = Some(wrap_angle(math::atan2(facing.x, facing.z)));
		}
	}

	/// Slows to a stand where it is.
	fn stand_and_face(&mut self) {
		self.speed = move_towards(
			self.speed,
			0.0,
			Self::ACCELERATION / HuntRules::TICK_RATE as f64,
		);

		if self.speed > 0.0 {
			let travel = self.speed / HuntRules::TICK_RATE as f64;
			let next = Vec3::new(
				self.position.x + math::sin(self.heading) * travel,
				self.position.y,
				self.position.z + math::cos(self.heading) * travel,
			);

			self.position = next;
			self.distance_travelled += travel;
			let level = self.nav.level_of(self.cell_at(next));
			self.settle_on(next, level);
		}

		self.position = Vec3::new(
			self.position.x,
			self.support_height + self.profile().half_height(),
			self.position.z,
		);
		self.state = HuskMotionState::Idle;
		self.state_fraction = 0.0;
		self.look_at = None;

		let Some(face) = self.face_target else {
			return;
		};

		let error = wrap_angle(face - self.heading);

		if error.abs() <= radians(1.0) {
			self.face_target = None;

			return;
		}

		self.heading = wrap_angle(
			self.heading
				+ math::clamp(
					error,
					-Self::TURN_RATE / HuntRules::TICK_RATE as f64,
					Self::TURN_RATE / HuntRules::TICK_RATE as f64,
				),
		);
		self.state = HuskMotionState::Turn;
		self.state_fraction = math::clamp(1.0 - error.abs() / math::PI, 0.0, 1.0);
	}

	/// Stops dead at `step`, asked or arrived.
	fn stop_at(&mut self, step: &RoomStep) {
		self.speed = 0.0;
		self.settle_on(step.point, step.level_index);
		self.steps = None;
		self.step_index = 0;
		self.state = HuskMotionState::Idle;
		self.state_fraction = 0.0;
	}

	/// Puts the root on the surface of `level_index` at `point`, in one
	/// step.
	fn settle_on(&mut self, point: Vec3, level_index: i8) {
		let levels = &self.grid().levels_centimetres;

		if level_index >= 0 && (level_index as usize) < levels.len() {
			self.support_height = (i64::from(levels[level_index as usize])
				- i64::from(self.grid().eye_height_centimetres))
				as f64
				/ 100.0;
		}

		self.position = Vec3::new(
			point.x,
			self.support_height + self.profile().half_height(),
			point.z,
		);
		self.cell = self.cell_at(self.position);
	}

	/// The cell a point falls in, or the cell it was over when the point
	/// is outside the grid.
	fn cell_at(&self, point: Vec3) -> usize {
		let x = self.grid().cell_x_at(point.x, point.z);
		let z = self.grid().cell_z_at(point.x, point.z);

		if self.grid().contains(x, z) {
			self.grid().index_of(x, z)
		} else {
			self.cell
		}
	}
}

/// How far apart two places are, across the floor.
fn flat_distance(from: Vec3, to: Vec3) -> f64 {
	let away_x = to.x - from.x;
	let away_z = to.z - from.z;

	math::sqrt(away_x * away_x + away_z * away_z)
}

/// `value` moved at most `step` towards `target`.
fn move_towards(value: f64, target: f64, step: f64) -> f64 {
	if value < target {
		math::min(value + step, target)
	} else {
		math::max(value - step, target)
	}
}

impl HuskMotion<'_> {
	/// A flier holds the air instead of walking: it drifts to its place,
	/// round anything in the way, and lands on a perch it was given.
	fn advance_flight(&mut self) {
		if self.needs_path
			&& !matches!(self.state, HuskMotionState::Land | HuskMotionState::Recover)
		{
			self.start_route();
		}

		match self.state {
			HuskMotionState::Air => self.advance_flight_move(),
			HuskMotionState::Land => self.advance_land(),
			HuskMotionState::Recover => self.advance_recover(),
			_ => self.advance_flight_idle(),
		}
	}

	/// Plans a flight to `target`: up to the cruise height, round
	/// anything in the way, then down to it.
	fn plan_flight(&mut self, target: Vec3, spot: Option<RoomSpot>) {
		let profile = *self.profile();
		// The rules take every float through +, −, ×, ÷ and `math`, so
		// the middle of the band is written as a plain average.
		#[allow(
			clippy::manual_midpoint,
			reason = "the sum halved, as the rules write their arithmetic"
		)]
		let cruise = (profile.air_low_metres + profile.air_high_metres) / 2.0;
		let perch = spot.filter(|spot| spot.kind == RoomSpotKind::Perch);
		let end = match perch {
			Some(perch) => {
				Vec3::new(
					perch.point.x,
					perch.point.y + profile.half_height(),
					perch.point.z,
				)
			}
			None => Vec3::new(target.x, cruise, target.z),
		};

		self.perch_target = perch;
		self.perched = false;
		self.steps = None;
		self.step_index = 0;
		self.flight_index = 0;

		let mut waypoints: Vec<Vec3> = Vec::new();

		if self.position.y < cruise - 0.01 {
			waypoints.push(Vec3::new(self.position.x, cruise, self.position.z));
		}

		let last = waypoints.last().copied().unwrap_or(self.position);

		if !self.flight_clear(last, end)
			&& let Some(detour) = self.detour(last, end)
		{
			waypoints.push(detour);
		}

		waypoints.push(end);
		self.flight_waypoints = waypoints;
		self.target_speed = math::max(profile.run_speed, profile.walk_speed);
		self.speed = self.target_speed;
		self.state = HuskMotionState::Air;
		self.state_fraction = 0.0;
		self.look_at = self.flight_waypoints.first().copied();
	}

	/// Flies towards the next waypoint of the flight.
	fn advance_flight_move(&mut self) {
		if self.flight_index >= self.flight_waypoints.len() {
			self.arrive_in_flight();

			return;
		}

		let target = self.flight_waypoints[self.flight_index];
		let away_x = target.x - self.position.x;
		let away_y = target.y - self.position.y;
		let away_z = target.z - self.position.z;
		let distance = math::sqrt(away_x * away_x + away_y * away_y + away_z * away_z);

		if distance <= 0.01 {
			self.position = target;
			self.flight_index += 1;

			if self.flight_index >= self.flight_waypoints.len() {
				self.arrive_in_flight();
			}

			return;
		}

		let travel = math::min(self.speed / HuntRules::TICK_RATE as f64, distance);
		let desired = math::atan2(away_x, away_z);
		let error = wrap_angle(desired - self.heading);
		let turn_step = Self::TURN_RATE / HuntRules::TICK_RATE as f64;

		self.heading = wrap_angle(self.heading + math::clamp(error, -turn_step, turn_step));
		self.distance_travelled += travel;

		if travel >= distance {
			self.position = target;
			self.flight_index += 1;

			if self.flight_index >= self.flight_waypoints.len() {
				self.arrive_in_flight();
			}

			return;
		}

		self.position = Vec3::new(
			self.position.x + away_x / distance * travel,
			self.position.y + away_y / distance * travel,
			self.position.z + away_z / distance * travel,
		);
		self.cell = self.cell_at(self.position);
		self.support_height = self
			.grid()
			.height_at(self.grid().x_of(self.cell), self.grid().z_of(self.cell));
		self.state_fraction = (self.flight_index as f64 + travel / distance)
			/ math::max(1.0, self.flight_waypoints.len() as f64);
		self.look_at = Some(target);
	}

	/// Settles on a perch, or hangs in the air where it was sent.
	fn arrive_in_flight(&mut self) {
		if let Some(perch) = self.perch_target.take() {
			let cell = self
				.nav
				.standable_cell_near(perch.point, 20)
				.unwrap_or_else(|| self.cell_at(perch.point));

			self.cell = cell;
			self.support_height = self.nav.support_height_of(cell).unwrap_or(perch.point.y);
			self.position = Vec3::new(
				perch.point.x,
				self.support_height + self.profile().half_height(),
				perch.point.z,
			);
			self.land_tick = 0;
			self.land_ticks = 8;
			self.land_dip = 0.02;
			self.land_speed = 0.0;
			self.land_heading = self.heading;
			self.fall_height = 0.0;
			self.state = HuskMotionState::Land;
			self.state_fraction = 0.0;
			self.look_at = None;
			self.perched = true;

			return;
		}

		self.speed = 0.0;
		self.state = HuskMotionState::Idle;
		self.state_fraction = 0.0;
		self.look_at = None;
		self.perched = false;
	}

	/// Hangs, or turns where it hangs.
	fn advance_flight_idle(&mut self) {
		self.speed = 0.0;

		if self.perched {
			self.position = Vec3::new(
				self.position.x,
				self.support_height + self.profile().half_height(),
				self.position.z,
			);
		} else {
			let low = self.profile().air_low_metres;
			let high = self.profile().air_high_metres;

			self.position = Vec3::new(
				self.position.x,
				math::clamp(self.position.y, low, math::max(low, high)),
				self.position.z,
			);
		}

		self.cell = self.cell_at(self.position);
		self.support_height = self
			.grid()
			.height_at(self.grid().x_of(self.cell), self.grid().z_of(self.cell));
		self.state = HuskMotionState::Idle;
		self.state_fraction = 0.0;
		self.look_at = None;

		let Some(face) = self.face_target else {
			return;
		};

		let error = wrap_angle(face - self.heading);

		if error.abs() <= radians(1.0) {
			self.face_target = None;

			return;
		}

		self.heading = wrap_angle(
			self.heading
				+ math::clamp(
					error,
					-Self::TURN_RATE / HuntRules::TICK_RATE as f64,
					Self::TURN_RATE / HuntRules::TICK_RATE as f64,
				),
		);
		self.state = HuskMotionState::Turn;
		self.state_fraction = math::clamp(1.0 - error.abs() / math::PI, 0.0, 1.0);
	}

	/// Whether a straight flight between two places stays clear of the
	/// room.
	fn flight_clear(&self, from: Vec3, to: Vec3) -> bool {
		let away_x = to.x - from.x;
		let away_z = to.z - from.z;
		let distance = math::sqrt(away_x * away_x + away_z * away_z);
		let steps = math::ceil_int(distance / RoomGrid::CELL_METRES).max(1);

		for step in 0..=steps {
			let along = step as f64 / steps as f64;
			let x = from.x + away_x * along;
			let z = from.z + away_z * along;
			let y = from.y + (to.y - from.y) * along;
			let cell_x = self.grid().cell_x_at(x, z);
			let cell_z = self.grid().cell_z_at(x, z);

			if !self.grid().contains(cell_x, cell_z) {
				return false;
			}

			if self.grid().height_at(cell_x, cell_z) > y - self.profile().half_height() + 1e-9 {
				return false;
			}
		}

		true
	}

	/// A place off to the side of the straight course that both halves of
	/// the flight clear, nearer first.
	fn detour(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
		let away_x = to.x - from.x;
		let away_z = to.z - from.z;
		let distance = math::sqrt(away_x * away_x + away_z * away_z);

		if distance == 0.0 {
			return None;
		}

		let side_x = -away_z / distance;
		let side_z = away_x / distance;

		for offset in [0.5, -0.5, 1.0, -1.0, 1.5, -1.5, 2.0, -2.0] {
			#[allow(
				clippy::manual_midpoint,
				reason = "the sum halved, as the rules write their arithmetic"
			)]
			let point = Vec3::new(
				(from.x + to.x) / 2.0 + side_x * offset,
				(from.y + to.y) / 2.0,
				(from.z + to.z) / 2.0 + side_z * offset,
			);

			if self.flight_clear(from, point) && self.flight_clear(point, to) {
				return Some(point);
			}
		}

		None
	}
}
