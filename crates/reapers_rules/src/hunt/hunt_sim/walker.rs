//! A husk that walks the room's grid.
//!
//! Where the phone read the room cell by cell, a husk on foot leaves the
//! slices behind: it walks the floor, crosses a desk, gathers at an edge
//! and leaps, all through [`HuskMotion`]. The hunt still decides when it
//! moves and why, from the same streams of chance; this decides where it
//! goes and how its body gets there.

use std::sync::Arc;

use super::HuntSim;
use crate::arts::ArtRules;
use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_event::HuntEvent;
use crate::hunt::hunt_event::HuntPhase;
use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::husk_pose::Footing;
use crate::hunt::husk_pose::HuskLocomotion;
use crate::hunt::husk_pose::HuskPose;
use crate::math;
use crate::room::HuskMotion;
use crate::room::HuskMotionState;
use crate::room::MotionPace;
use crate::room::RoomCellKind;
use crate::room::RoomGrid;
use crate::room::RoomNav;
use crate::room::RoomSpot;
use crate::room::RoomSpotKind;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;
use crate::vec3::Vec3;
use crate::vec3::radians;
use crate::vec3::wrap_angle;
use crate::vessel::Gait;
use crate::vessel::Vessel;

/// A husk's body on the room's grid, and what the hunt has it doing.
#[derive(Clone, Debug)]
pub(super) struct Walker {
	motion: HuskMotion<'static, Arc<RoomNav>>,

	/// How big the body is against the vessel's own measurements.
	scale: f64,

	/// Whether it is on its way somewhere it chose. It rests when it gets
	/// there.
	walking: bool,

	/// Whether it is on its way down to open floor because it has to be:
	/// to charge, to shriek, or knocked off by wind. Nothing stops it
	/// until it is down, as nothing stops a leap.
	descending: bool,

	/// What the leap under way comes down on.
	landing: Footing,

	/// A stun wind dealt it in the air or on its way down, in ticks,
	/// served from the moment it is down.
	stun_on_landing: Option<i64>,

	/// A charge under way.
	dash: Option<Dash>,
}

/// A charge across the floor at the reaper.
#[derive(Clone, Copy, Debug)]
struct Dash {
	from: Vec3,
	to: Vec3,
	at: Vec3,
	heading: f64,
}

impl Walker {
	/// Where a walker's body draws its chance from: how long it gathers
	/// before a leap and how long it takes to stand after one.
	const MOTION_STREAM: i64 = 0xB0D1;

	/// A body for `vessel` in `room`, set down as near `start` as it fits
	/// and facing `heading`. Nothing for a vessel that does not walk, or a
	/// room with nowhere near `start` it can stand.
	pub(super) fn new(
		seed: i64,
		vessel: Vessel,
		room: &RoomGrid,
		difficulty: Difficulty,
		start: Vec3,
		heading: f64,
	) -> Option<Self> {
		if vessel.gait() != Gait::Ground {
			return None;
		}

		let mut profile = vessel.locomotion();

		profile.walk_speed *= difficulty.pace();
		profile.run_speed *= difficulty.pace();

		let scale = profile.body_height / (2.0 * vessel.half_height());
		let nav = Arc::new(RoomNav::for_hunt(
			room.clone(),
			profile,
			HuntRules::STAGING_FAR_METRES,
			HuntRules::NEAREST_METRES,
		));
		// It starts on the floor, as a husk that does not walk the grid
		// does, unless there is no floor near enough to stand on.
		let start = nav
			.standable_cell_near_where(start, 20, |cell| {
				nav.region_of(cell)
					.is_some_and(|region| region.height_centimetres == 0)
			})
			.map_or(start, |cell| {
				nav.grid.centre_of(nav.grid.x_of(cell), nav.grid.z_of(cell))
			});
		let random = SeededRandom::new(mix_seed(&[seed, Self::MOTION_STREAM]));
		let motion = HuskMotion::new(nav, random, Some(start), heading)?;

		Some(Self {
			motion,
			scale,
			walking: false,
			descending: false,
			landing: Footing::Open,
			stun_on_landing: None,
			dash: None,
		})
	}

	pub(super) fn grid(&self) -> &RoomGrid {
		&self.motion.nav.grid
	}

	/// The middle of the body.
	pub(super) fn position(&self) -> Vec3 {
		self.dash
			.map_or_else(|| self.motion.position(), |dash| dash.at)
	}

	pub(super) fn heading(&self) -> f64 {
		self.dash
			.map_or_else(|| self.motion.heading(), |dash| dash.heading)
	}

	/// Whether the body is gathering for a leap, in the air, or coming
	/// down from one.
	pub(super) fn is_leaping(&self) -> bool {
		self.dash.is_none()
			&& matches!(
				self.motion.state(),
				HuskMotionState::Crouch
					| HuskMotionState::Air
					| HuskMotionState::Land
					| HuskMotionState::Recover
			)
	}

	/// Whether it is in a move nothing the hunt does stops: a leap, or the
	/// way down it has to take.
	pub(super) fn committed(&self) -> bool {
		self.is_leaping() || self.descending
	}

	/// What the body stands on or behind: anything raised is a ledge, floor
	/// the eye cannot see is hidden, and the rest is open.
	pub(super) fn footing(&self) -> Footing {
		self.footing_of(self.motion.cell())
	}

	pub(super) const fn landing(&self) -> Footing {
		self.landing
	}

	fn footing_of(&self, cell: usize) -> Footing {
		let nav = &self.motion.nav;

		if nav
			.region_of(cell)
			.is_some_and(|region| region.height_centimetres > 0)
		{
			return Footing::Ledge;
		}

		let grid = &nav.grid;

		if grid.cell_at(grid.x_of(cell), grid.z_of(cell)).kind == RoomCellKind::Hidden {
			Footing::Hidden
		} else {
			Footing::Open
		}
	}

	/// How fast it is covering ground, in metres a second. A charge is
	/// told as its whole run over the `charge_ticks` it takes.
	pub(super) fn speed(&self, charge_ticks: i64) -> f64 {
		match self.dash {
			Some(dash) => {
				flat_length(dash.to - dash.from)
					/ (charge_ticks as f64 / HuntRules::TICK_RATE as f64)
			}
			None => self.motion.speed(),
		}
	}

	/// Where the husk is and what its body is doing, for the view.
	pub(super) fn pose(&self, vessel: Vessel) -> HuskPose {
		let leaping = self.is_leaping();
		let support_height = self.motion.support_height();
		let (state, state_fraction, next_foothold) = match self.dash {
			Some(dash) => (HuskMotionState::Run, 0.0, Some(dash.to)),
			None => {
				(
					self.motion.state(),
					self.motion.state_fraction(),
					self.motion.next_foothold(),
				)
			}
		};

		HuskPose {
			vessel,
			position: self.position(),
			heading: self.heading(),
			footing: if leaping {
				self.landing
			} else {
				self.footing()
			},
			leaping,
			ground: Some(support_height),
			scale: self.scale,
			locomotion: Some(HuskLocomotion {
				state,
				state_fraction,
				support_height,
				next_foothold,
			}),
		}
	}

	/// Moves the body on by a tick, and returns what a leap it has just
	/// gathered itself for comes down on.
	fn tick(&mut self) -> Option<Footing> {
		let before = self.motion.state();

		self.motion.tick();

		if self.motion.state() != HuskMotionState::Crouch || before == HuskMotionState::Crouch {
			return None;
		}

		let grid = self.grid();
		let landing = self.motion.next_foothold().map_or(Footing::Open, |to| {
			let (x, z) = (grid.cell_x_at(to.x, to.z), grid.cell_z_at(to.x, to.z));

			if grid.contains(x, z) {
				self.footing_of(grid.index_of(x, z))
			} else {
				Footing::Open
			}
		});

		self.landing = landing;

		Some(landing)
	}

	/// Where the nearest leap down from the surface it stands on lands:
	/// the leap whose take-off is nearest the body, to anything lower.
	/// Nothing on the floor, or on a surface with no way down.
	fn nearest_way_down(&self) -> Option<Vec3> {
		let nav = &self.motion.nav;
		let here = nav.region_of(self.motion.cell())?;
		let at = self.motion.position();
		let mut nearest: Option<(f64, usize)> = None;

		// The links come in take-off order, and only a nearer one replaces
		// the one found, so a tie goes to the lower take-off cell.
		for link in &nav.links {
			let lower = nav
				.regions
				.get(link.to_region)
				.is_some_and(|region| region.height_centimetres < here.height_centimetres);

			if link.from_region != here.id || !lower {
				continue;
			}

			let take_off = nav
				.grid
				.centre_of(nav.grid.x_of(link.from_cell), nav.grid.z_of(link.from_cell));
			let distance = flat_length(take_off - at);

			if nearest.is_none_or(|(found, _)| distance < found) {
				nearest = Some((distance, link.to_cell));
			}
		}

		nearest.map(|(_, landing)| {
			nav.grid
				.centre_of(nav.grid.x_of(landing), nav.grid.z_of(landing))
		})
	}

	/// Whether the body has a way from where it stands to `target`.
	fn has_way_to(&self, target: Vec3) -> bool {
		self.motion
			.nav
			.path(self.motion.position(), target)
			.is_some_and(|path| !path.is_empty())
	}

	/// Sets off for the reaper, straight across the surface it stands on,
	/// to within [`HuntRules::CHARGE_METRES`]. It stops short of anything
	/// in the way rather than run through the sofa or up the desk.
	fn begin_dash(&mut self) {
		let from = self.motion.position();
		let away = flat_length(from);
		let goal = if away <= HuntRules::CHARGE_METRES {
			from
		} else {
			let near = HuntRules::CHARGE_METRES / away;

			Vec3::new(from.x * near, from.y, from.z * near)
		};
		let support = self.motion.support_height();
		let nav = &self.motion.nav;
		let steps = math::ceil_int(flat_length(goal - from) / (RoomGrid::CELL_METRES / 2.0)).max(1);
		let mut to = from;

		for step in 1..=steps {
			let point = from + (goal - from) * (step as f64 / steps as f64);
			let (x, z) = (
				nav.grid.cell_x_at(point.x, point.z),
				nav.grid.cell_z_at(point.x, point.z),
			);
			let level = nav.grid.contains(x, z).then(|| nav.grid.index_of(x, z));

			if !level
				.and_then(|cell| nav.support_height_of(cell))
				.is_some_and(|height| (height - support).abs() < 0.005)
			{
				break;
			}

			to = point;
		}

		self.motion.halt();
		self.walking = false;
		self.descending = false;
		self.dash = Some(Dash {
			from,
			to,
			at: from,
			heading: self.motion.heading(),
		});
	}

	/// Carries the charge on to `progress` of the way, the face turning to
	/// the reaper as it comes.
	fn advance_dash(&mut self, progress: f64) {
		let Some(dash) = &mut self.dash else {
			return;
		};
		let step = radians(HuntRules::TURN_DEGREES_PER_TICK);
		let toward = math::atan2(-dash.at.x, -dash.at.z);

		dash.at = dash.from + (dash.to - dash.from) * (progress * progress);
		dash.heading =
			wrap_angle(dash.heading + math::clamp(wrap_angle(toward - dash.heading), -step, step));
	}

	/// Ends a charge where it got to, the body set down there.
	fn end_dash(&mut self) {
		if let Some(dash) = self.dash.take() {
			self.motion.set_down(dash.at, dash.heading);
		}
	}
}

/// How far `point` is from the player across the floor, or how long a
/// move is across it.
fn flat_length(point: Vec3) -> f64 {
	math::sqrt(point.x * point.x + point.z * point.z)
}

impl HuntSim {
	/// A husk that walks, in a room the phone read as a grid, walks it from
	/// as near where it would have stood as its body fits.
	pub(super) fn set_walker_down(&mut self) {
		let Some(room) = &self.plan.room else {
			return;
		};
		let start = Vec3::new(
			math::sin(self.bearing) * self.range,
			0.0,
			math::cos(self.bearing) * self.range,
		);

		self.walker = Walker::new(
			self.seed,
			self.vessel,
			room,
			self.difficulty,
			start,
			self.heading,
		);
		self.sync_walker();
	}

	/// Moves a husk that walks the room's grid by a tick, as
	/// [`move_husk`](Self::move_husk) moves one that does not.
	pub(super) fn walk(&mut self, events: &mut Vec<HuntEvent>) {
		let Some(walker) = self.walker.as_mut() else {
			return;
		};

		if walker.dash.is_some() && self.phase != HuntPhase::Charge {
			walker.end_dash();
		}

		if !walker.committed() {
			// Frozen, it does not even turn.
			if self.tick < self.frozen_until {
				self.halt();
				return;
			}

			if self.phase == HuntPhase::Charge {
				self.advance_dash();
				return;
			}

			// Drawing breath, a husk stops where it is and turns on the
			// reaper. Stunned, it does the same. One still backing off
			// after a charge gets clear first: it never stands nearer
			// than it keeps.
			let backing_off =
				walker.walking && flat_length(walker.position()) < HuntRules::NEAREST_METRES;

			if (self.phase == HuntPhase::Windup
				|| self.phase == HuntPhase::Shriek
				|| self.tick < self.staggered_until)
				&& !backing_off
			{
				self.halt();

				if let Some(walker) = self.walker.as_mut() {
					let at = walker.position();

					walker.motion.face(Vec3::new(-at.x, 0.0, -at.z));
				}
			} else if !walker.walking {
				let bolting = self.phase == HuntPhase::Fight
					&& self.next_bolt_tick >= 0
					&& self.tick >= self.next_bolt_tick;

				if bolting {
					self.bolt(events);
				} else if self.tick >= self.next_amble_tick {
					self.amble(events);
				}
			}
		}

		self.step_walker(events);
	}

	/// Brings a leap under way down, whatever else is going on.
	pub(super) fn continue_leap(&mut self, events: &mut Vec<HuntEvent>) {
		if self.walker.is_some() {
			self.step_walker(events);
		} else {
			self.continue_move();
		}
	}

	/// Moves the walker's body on a tick and takes stock: a leap it gathers
	/// for is told, a stun wind dealt it in the air is served once it is
	/// down, and a husk that got where it was going rests.
	fn step_walker(&mut self, events: &mut Vec<HuntEvent>) {
		let Some(walker) = self.walker.as_mut() else {
			return;
		};

		if walker.dash.is_some() {
			return;
		}

		if let Some(landing) = walker.tick() {
			events.push(HuntEvent::Leapt(landing));
		}

		let down =
			walker.motion.is_idle() || (!walker.is_leaping() && walker.footing() == Footing::Open);

		if walker.descending && down {
			walker.descending = false;
		}

		let stun = if walker.committed() {
			None
		} else {
			walker.stun_on_landing.take()
		};
		let arrived = walker.walking && walker.motion.is_idle();

		if arrived {
			walker.walking = false;
		}

		self.sync_walker();

		if let Some(ticks) = stun
			&& self.fighting()
		{
			self.staggered_until = self.staggered_until.max(self.tick + ticks);
			self.stunned_until = self.staggered_until;
			self.next_bolt_tick = self.staggered_until;
			self.exposed_until = self
				.exposed_until
				.max(self.staggered_until + ArtRules::SHAKEN_TICKS);
		}

		if arrived {
			self.next_amble_tick = self.tick + self.rest_ticks();
		}
	}

	/// Copies where the walker is into the fields the rest of the hunt
	/// reads: its bearing, range and height, its heading, its footing, and
	/// whether it is in a move nothing stops.
	fn sync_walker(&mut self) {
		let Some(walker) = &self.walker else {
			return;
		};
		let at = walker.position();

		self.bearing = math::atan2(at.x, at.z);
		self.range = flat_length(at);
		self.height = at.y;
		self.heading = walker.heading();
		self.leaping = walker.committed();
		self.footing = walker.footing();
		self.to_footing = walker.landing();
	}

	/// Starts a charge across the floor.
	pub(super) fn begin_dash(&mut self) {
		if let Some(walker) = self.walker.as_mut() {
			walker.begin_dash();
		}

		self.sync_walker();
	}

	fn advance_dash(&mut self) {
		let progress = math::clamp(
			(self.tick - self.move_start) as f64 / self.move_ticks as f64,
			0.0,
			1.0,
		);

		if let Some(walker) = self.walker.as_mut() {
			walker.advance_dash(progress);
		}

		self.sync_walker();
	}

	/// Ends a charge where it got to.
	pub(super) fn end_dash(&mut self) {
		if let Some(walker) = self.walker.as_mut() {
			walker.end_dash();
		}

		self.sync_walker();
	}

	/// Stops a walker where it stands: in its tracks, unless it is in the
	/// air, when it lands first and goes no farther.
	pub(super) fn halt_walker(&mut self) {
		let Some(walker) = self.walker.as_mut() else {
			return;
		};

		walker.end_dash();

		let rest = walker.walking && !walker.is_leaping();

		walker.motion.halt();
		walker.descending = false;

		if rest {
			walker.walking = false;
		}

		self.sync_walker();

		if rest {
			self.next_amble_tick = self.tick + self.rest_ticks();
		}
	}

	/// Wind that catches a walker: it comes down off whatever it is on, and
	/// stands stunned for `ticks` once it is down.
	pub(super) fn gust_walker(&mut self, ticks: i64) {
		if !self.leaping && self.footing != Footing::Open {
			self.walk_down(false);
		}

		match self.walker.as_mut() {
			Some(walker) if walker.committed() => walker.stun_on_landing = Some(ticks),
			_ => self.halt(),
		}

		self.staggered_until = self.tick + ticks;
	}
}

impl HuntSim {
	/// An unhurried walk to open floor nearby. Now and then, where the room
	/// has somewhere better to be, it makes for a perch or a hiding place
	/// instead, the way a cat would.
	pub(super) fn amble_walking(&mut self) {
		if self.footing != Footing::Open {
			self.walk_down(true);
			return;
		}

		let spots = self.cover_spots(self.bearing, radians(HuntRules::COVER_SEARCH_DEGREES), 0.0);

		if !spots.is_empty()
			&& self.tick >= self.exposed_until
			&& self.cover.next_chance(HuntRules::AMBLE_COVER_CHANCE)
		{
			let spot = self.cover.pick(&spots);

			self.walk_to_spot(spot, MotionPace::Walk);
			return;
		}

		let by = radians(
			self.path
				.next_range(-HuntRules::AMBLE_DEGREES, HuntRules::AMBLE_DEGREES),
		);
		let to_bearing = self.plan.clamp(self.bearing + by);
		let outward = self.path.next_range(0.0, 1.0);

		self.walk_out(to_bearing, outward, MotionPace::Walk);
	}

	/// A run to somewhere well away from where the reaper is aiming: to a
	/// perch or a hiding place when the room offers one, unless it is
	/// burning or the wind has shaken it, when it wants nothing but open
	/// floor.
	pub(super) fn bolt_walking(&mut self, events: &mut Vec<HuntEvent>) {
		if self.tick >= self.exposed_until {
			let spots = self.cover_spots(
				self.bearing,
				radians(HuntRules::BOLT_MAX_DEGREES),
				radians(HuntRules::BOLT_MIN_DEGREES / 2.0),
			);

			if !spots.is_empty() && self.cover.next_chance(HuntRules::BOLT_COVER_CHANCE) {
				let spot = self.cover.pick(&spots);

				self.walk_to_spot(spot, MotionPace::Run);
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
		// It runs for the far half of the floor it keeps to.
		let outward = self.path.next_range(0.5, 1.0);

		self.walk_out(to_bearing, outward, MotionPace::Run);
		events.push(HuntEvent::Relocating);
	}

	/// Comes down off a ledge, or out from behind cover, onto open floor in
	/// front of it. A husk that `may_stay` is only ready to move, and may
	/// settle down where it is a while longer, or comes down anywhere in
	/// front. One that may not takes the nearest way down, a surface at a
	/// time, and is on its way down until it is down.
	///
	/// Returns whether it stays or has a way down. A husk with no way down
	/// stays where it is, and rests before it tries again.
	pub(super) fn walk_down(&mut self, may_stay: bool) -> bool {
		if may_stay && self.cover.next_chance(HuntRules::STAY_COVER_CHANCE) {
			self.next_amble_tick = self.tick + self.rest_ticks();
			return true;
		}

		let to_bearing = self
			.plan
			.clamp(self.bearing + radians(self.cover.next_range(-20.0, 20.0)));
		let outward = self.cover.next_range(0.2, 0.8);
		let nearest = if may_stay {
			None
		} else {
			self.walker.as_ref().and_then(Walker::nearest_way_down)
		};
		let target = nearest.unwrap_or_else(|| self.open_floor_near(to_bearing, outward));
		let Some(walker) = self.walker.as_mut() else {
			return false;
		};

		if !walker.has_way_to(target) {
			self.next_amble_tick = self.tick + self.rest_ticks();
			return false;
		}

		walker.motion.go_to(target, MotionPace::Run);
		walker.walking = true;
		walker.descending = !may_stay;
		self.sync_walker();

		true
	}

	fn walk_to_spot(&mut self, spot: RoomSpot, pace: MotionPace) {
		if let Some(walker) = self.walker.as_mut() {
			walker.motion.go_to_spot(spot, pace);
			walker.walking = true;
		}
	}

	/// Walks, or runs, to the open floor nearest `outward` of the way
	/// across the band it keeps to along `to_bearing`.
	fn walk_out(&mut self, to_bearing: f64, outward: f64, pace: MotionPace) {
		let target = self.open_floor_near(to_bearing, outward);

		if let Some(walker) = self.walker.as_mut() {
			walker.motion.go_to(target, pace);
			walker.walking = true;
		}
	}

	/// The open floor nearest `outward` of the way from
	/// [`HuntRules::STAGING_NEAR_METRES`] to
	/// [`HuntRules::STAGING_FAR_METRES`] along `bearing`: floor the eye
	/// can see, no nearer than [`HuntRules::NEAREST_METRES`]. Where the
	/// room has none, the place itself, for the body to get as near as it
	/// can.
	fn open_floor_near(&self, bearing: f64, outward: f64) -> Vec3 {
		let metres = HuntRules::STAGING_NEAR_METRES
			+ outward * (HuntRules::STAGING_FAR_METRES - HuntRules::STAGING_NEAR_METRES);
		let aim = Vec3::new(
			math::sin(bearing) * metres,
			0.0,
			math::cos(bearing) * metres,
		);
		let Some(walker) = &self.walker else {
			return aim;
		};
		let mut nearest = aim;
		let mut nearest_distance = f64::INFINITY;

		// Spots come in cell order, and only a nearer one replaces the
		// one found, so a tie goes to the lower cell.
		for spot in &walker.motion.nav.spots {
			if spot.kind != RoomSpotKind::Open {
				continue;
			}

			let distance = flat_length(spot.point - aim);

			if distance < nearest_distance {
				nearest_distance = distance;
				nearest = spot.point;
			}
		}

		nearest
	}

	/// Perches, peeks and hiding places within `within` radians of the
	/// bearing `around`, no nearer to it than `beyond`, inside the plan, no
	/// farther than the husk ever hides, and somewhere its body can get to
	/// from where it stands.
	fn cover_spots(&self, around: f64, within: f64, beyond: f64) -> Vec<RoomSpot> {
		let Some(walker) = &self.walker else {
			return Vec::new();
		};
		let nav = &walker.motion.nav;
		let Some(here) = nav.region_of(walker.motion.cell()) else {
			return Vec::new();
		};
		let reachable = nav.reachable_from(here.id);
		let farthest = HuntRules::STAGING_FAR_METRES + HuntRules::HIDE_BEYOND;
		let lowest = math::round(HuntRules::LOWEST_LEDGE * 100.0);

		nav.spots
			.iter()
			.filter(|spot| {
				let bearing = math::atan2(spot.point.x, spot.point.z);
				let away = wrap_angle(bearing - around).abs();
				let region = nav.region_of(spot.cell);

				spot.kind != RoomSpotKind::Open
					&& spot.cell != walker.motion.cell()
					&& away <= within
					&& away >= beyond
					&& self.plan.contains(bearing)
					&& flat_length(spot.point) <= farthest
					&& region.is_some_and(|region| {
						reachable.get(region.id).copied().unwrap_or(false)
							&& (spot.kind != RoomSpotKind::Perch
								|| region.height_centimetres >= lowest)
					})
			})
			.copied()
			.collect()
	}
}
