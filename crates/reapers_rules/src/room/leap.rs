//! The ballistic leap: a real jump under real gravity, solved exactly.

use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::room::room_grid::RoomGrid;
use crate::vec3::Vec3;

/// The fixed numbers of a ballistic leap.
pub struct LeapRules;

impl LeapRules {
	/// How much the apex is raised when the arc would hit something.
	pub const CLEARANCE_STEP: f64 = 0.05;
	/// How many times the solver tries before it gives up on a leap.
	pub const CLEARANCE_TRIES: i64 = 3;
	/// How much the apex rises for a leap that lands higher than it
	/// starts. A jump up goes steeply, the way an animal gathers itself.
	pub const CLIMB_CLEARANCE: f64 = 0.20;
	/// The apex above the higher end of a leap that lands lower than it
	/// starts.
	pub const DROP_CLEARANCE: f64 = 0.12;
	/// Real gravity, in metres a second squared. The husks are life-sized,
	/// so the player's intuition for a small animal's jump is the right
	/// one.
	pub const GRAVITY: f64 = 9.81;
	/// The apex above the ends of a leap between two places at the same
	/// height. A gap needs a hop to clear it.
	pub const LEVEL_CLEARANCE: f64 = 0.22;
	/// The arc is sampled no less often than every 10 centimetres.
	pub const SAMPLE_METRES: f64 = 0.10;
}

/// One ballistic leap between two surfaces, solved tick by tick.
///
/// A leap leaves at full speed, keeps its horizontal speed and lands at
/// full speed, so the body is never eased into or out of it. The path is
/// the analytic parabola, evaluated at whole ticks:
/// `y = y0 + vy t - g t² / 2`. Only +, −, ×, ÷ and √ are used, and IEEE
/// 754 makes √ correctly rounded, so the same leap replays identically on
/// every platform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leap {
	/// The middle of the body where the leap starts.
	pub from: Vec3,

	/// The middle of the body where the leap lands.
	pub to: Vec3,

	/// The whole number of ticks the leap takes.
	pub ticks: i64,

	/// The constant horizontal velocity, in metres a second.
	pub horizontal_velocity: Vec3,

	/// The initial upward velocity, in metres a second.
	pub vertical_velocity: f64,

	/// How far the apex rose above the higher end of the leap.
	pub clearance: f64,

	/// The highest the middle of the body reaches, in metres from the eye.
	pub apex_height: f64,
}

impl Leap {
	/// Solves a leap from `from` to `to`, both the middle of the body,
	/// that clears the room's surfaces.
	///
	/// The apex is raised by [`LeapRules::CLEARANCE_STEP`] and the leap
	/// solved again when the arc would pass through a surface plus the
	/// body's half height, up to [`LeapRules::CLEARANCE_TRIES`] times. A
	/// leap that still hits returns nothing, so a caller can refuse the
	/// link rather than send a husk through a desk.
	///
	/// `clearance` is the height of the apex above the higher end of the
	/// leap. When it is omitted the solver picks one from the direction of
	/// the jump: a climb gathers more than a drop, and a gap hops.
	pub fn solve(
		from: Vec3,
		to: Vec3,
		grid: &RoomGrid,
		half_height: f64,
		clearance: Option<f64>,
	) -> Option<Self> {
		let mut rise = clearance.unwrap_or(match to.y.total_cmp(&from.y) {
			core::cmp::Ordering::Greater => LeapRules::CLIMB_CLEARANCE,
			core::cmp::Ordering::Less => LeapRules::DROP_CLEARANCE,
			core::cmp::Ordering::Equal => LeapRules::LEVEL_CLEARANCE,
		});

		for _ in 0..LeapRules::CLEARANCE_TRIES {
			let leap = Self::solve_once(from, to, rise);

			if leap.clears(grid, half_height) {
				return Some(leap);
			}

			rise += LeapRules::CLEARANCE_STEP;
		}

		None
	}

	/// The leap's duration, in seconds.
	pub fn flight_seconds(&self) -> f64 {
		self.ticks as f64 / HuntRules::TICK_RATE as f64
	}

	/// How fast the body is falling when it lands, in metres a second. It
	/// is positive for a body moving down.
	pub fn impact_speed(&self) -> f64 {
		-(self.vertical_velocity - LeapRules::GRAVITY * self.flight_seconds())
	}

	/// Where the middle of the body is at whole tick `tick`.
	///
	/// Tick zero is `from` and tick `ticks` is `to`. A leap is analytic,
	/// so this is the exact place at that tick however far it is from the
	/// start.
	pub fn position_at(&self, tick: i64) -> Vec3 {
		self.position_at_seconds(tick as f64 / HuntRules::TICK_RATE as f64)
	}

	/// Where the middle of the body is at `seconds` into the leap.
	pub fn position_at_seconds(&self, seconds: f64) -> Vec3 {
		Vec3::new(
			self.from.x + self.horizontal_velocity.x * seconds,
			self.from.y + self.vertical_velocity * seconds
				- 0.5 * LeapRules::GRAVITY * seconds * seconds,
			self.from.z + self.horizontal_velocity.z * seconds,
		)
	}

	fn solve_once(from: Vec3, to: Vec3, clearance: f64) -> Self {
		let apex = math::max(from.y, to.y) + clearance;
		let rise = math::sqrt(2.0 * (apex - from.y) / LeapRules::GRAVITY);
		let fall = math::sqrt(2.0 * (apex - to.y) / LeapRules::GRAVITY);
		let ticks = math::ceil_int(math::max(1.0, (rise + fall) * HuntRules::TICK_RATE as f64));
		let flight = ticks as f64 / HuntRules::TICK_RATE as f64;
		let vertical = (to.y - from.y + 0.5 * LeapRules::GRAVITY * flight * flight) / flight;

		Self {
			from,
			to,
			ticks,
			horizontal_velocity: Vec3::new((to.x - from.x) / flight, 0.0, (to.z - from.z) / flight),
			vertical_velocity: vertical,
			clearance,
			apex_height: from.y + vertical * vertical / (2.0 * LeapRules::GRAVITY),
		}
	}

	/// Whether the arc stays above every surface it passes over.
	fn clears(&self, grid: &RoomGrid, half_height: f64) -> bool {
		let away_x = self.to.x - self.from.x;
		let away_z = self.to.z - self.from.z;
		let travel = math::sqrt(away_x * away_x + away_z * away_z);
		let steps = math::ceil_int(math::max(1.0, travel / LeapRules::SAMPLE_METRES));

		for step in 0..=steps {
			let place =
				self.position_at_seconds(self.flight_seconds() * step as f64 / steps as f64);
			let surface = (grid.top_centimetres_at_world(place.x, place.z)
				- i64::from(grid.eye_height_centimetres)) as f64
				/ 100.0;

			// Floating point leaves the landing a hair below the surface
			// it touches.
			if place.y < surface + half_height - 1e-9 {
				return false;
			}
		}

		true
	}
}
