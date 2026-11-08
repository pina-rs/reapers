//! The pattern a reaper traces to draw a husk out of hiding.

use core::fmt;

use crate::hunt::difficulty::Difficulty;
use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;
use crate::vec3::Vec3;
use crate::vec3::degrees;
use crate::vec3::radians;
use crate::vessel::Gait;

/// How hard a husk's sigil is to trace.
///
/// Every rarity has one. The rarer the husk, the more lights its sigil has,
/// the wider they are spread, the smaller each one is, the longer each has
/// to be held, and the more often the trace doubles back across itself. The
/// rarest let a light go out again if the next is too long coming.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SigilShape {
	/// How many lights the sigil has. At least two: a light to go from and
	/// one to go to.
	pub lights: usize,

	/// How far apart round the reaper its two outermost lights hang.
	pub span_degrees: f64,

	/// How near the middle of the view has to come to a light to rest on it.
	pub radius_degrees: f64,

	/// How long the view has to rest on a light, steadily, for it to catch.
	pub hold_seconds: f64,

	/// How many times the trace from light to light crosses itself.
	pub crossings: usize,

	/// How long a caught light waits for the next to catch before it goes
	/// out again, or `None` when a caught light stays caught.
	pub fray_seconds: Option<f64>,
}

impl SigilShape {
	pub const fn new(
		lights: usize,
		span_degrees: f64,
		radius_degrees: f64,
		hold_seconds: f64,
		crossings: usize,
		fray_seconds: Option<f64>,
	) -> Self {
		Self {
			lights,
			span_degrees,
			radius_degrees,
			hold_seconds,
			crossings,
			fray_seconds,
		}
	}
}

/// One light of a sigil.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SigilLight {
	/// How far to the right of the sigil's middle it hangs, in radians.
	pub yaw: f64,

	/// How far above the horizon it hangs, in radians. Below is negative.
	pub pitch: f64,
}

impl SigilLight {
	/// Which way it is from the reaper, for a sigil whose middle is straight
	/// ahead.
	pub fn direction(&self) -> Vec3 {
		Vec3::from_angles(self.yaw, self.pitch)
	}
}

impl fmt::Display for SigilLight {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"SigilLight({}°, {}°)",
			math::round(degrees(self.yaw)),
			math::round(degrees(self.pitch)),
		)
	}
}

/// The pattern a reaper traces with the phone to draw a husk out of hiding.
///
/// A sigil is a handful of lights hung in the air around the reaper. They
/// catch one at a time, in order, each once the middle of the view has
/// rested on it steadily for a moment. When the last one catches, the husk
/// shows itself.
///
/// Tracing it is also how the room is read. The lights hang where the phone
/// has to look for this husk: over the floor for one that walks, at about
/// head height for one that flies, and across the whole arc it will keep
/// to. Resting on a light holds the phone still long enough for the camera
/// to take a picture there.
///
/// A sigil is drawn from the hunt's seed, so the same husk always has the
/// same one.
#[derive(Clone, Debug, PartialEq)]
pub struct Sigil {
	/// The lights, in the order they catch.
	pub lights: Vec<SigilLight>,

	/// How near the middle of the view has to come to a light to rest on
	/// it, in radians.
	pub radius: f64,

	/// How long the view has to rest on a light for it to catch.
	pub hold_seconds: f64,

	/// How long a caught light waits for the next before going out, or
	/// `None`.
	pub fray_seconds: Option<f64>,
}

impl Sigil {
	/// Tries this many sigils for one that crosses itself as its shape asks
	/// before settling for the nearest to it.
	const ATTEMPTS: usize = 2000;
	/// How close, as a share of a light's radius, the trace may pass to a
	/// light it does not stop at.
	const CLEARANCE_SHARE: f64 = 1.5;
	/// Mixed into the hunt's seed, so that the sigil does not follow the
	/// same rolls as everything else the seed decides.
	const SALT: i64 = 0x5161;

	/// The sigil of a husk with `shape` that gets about by `gait`, for the
	/// hunt with `seed`, as forgiving as `difficulty` makes it.
	///
	/// The lights stand in a row of evenly spaced columns, one to a column,
	/// each at one of three heights and never at the height of the one
	/// beside it. The order they catch in is what makes a sigil hard: in a
	/// simple one it runs straight along the row, and in a hard one it
	/// doubles back and crosses itself. The trace never passes over a light
	/// it does not stop at, so the order can always be read from the shape.
	pub fn draw(seed: i64, shape: &SigilShape, gait: Gait, difficulty: Difficulty) -> Self {
		let mut random = SeededRandom::new(mix_seed(&[seed, Self::SALT]));
		let band = match gait {
			Gait::Ground => HuntRules::SIGIL_GROUND_BAND,
			Gait::Air | Gait::Still => HuntRules::SIGIL_AIR_BAND,
		};
		let clearance = radians(shape.radius_degrees) * Self::CLEARANCE_SHARE;
		let mut nearest: Option<Vec<SigilLight>> = None;
		let mut nearest_miss = shape.lights * shape.lights;

		// Heights and order are drawn together until they make a sigil that
		// crosses itself as often as its shape asks. A straight row is taken
		// at once.
		let mut attempt = 0;

		while attempt < Self::ATTEMPTS && nearest_miss > 0 {
			attempt += 1;

			let lights = Self::attempt(shape, band, &mut random);

			if Self::passes_over(&lights, clearance) {
				continue;
			}

			let miss = Self::count_crossings(&lights).abs_diff(shape.crossings);

			if miss < nearest_miss {
				nearest = Some(lights);
				nearest_miss = miss;
			}
		}

		Self {
			lights: nearest.unwrap_or_else(|| Self::row(shape, band, &mut random)),
			radius: radians(shape.radius_degrees) * difficulty.reach(),
			hold_seconds: shape.hold_seconds,
			fray_seconds: shape.fray_seconds.map(|fray| fray * difficulty.patience()),
		}
	}

	/// Half the width of the arc tracing this sigil reads, in radians: out
	/// to its farthest light and [`HuntRules::SIGIL_MARGIN`] beyond, which
	/// the camera takes in to either side of where it points.
	pub fn half_arc(&self) -> f64 {
		self.lights
			.iter()
			.map(|light| light.yaw.abs())
			.fold(0.0, math::max)
			+ HuntRules::SIGIL_MARGIN
	}

	/// How many times the trace from light to light crosses itself.
	pub fn crossings(&self) -> usize {
		Self::count_crossings(&self.lights)
	}

	/// One try at a sigil with `shape` whose lights hang within `band`: the
	/// heights drawn afresh, and the order too unless the sigil is a
	/// straight row.
	fn attempt(shape: &SigilShape, band: (f64, f64), random: &mut SeededRandom) -> Vec<SigilLight> {
		if shape.crossings == 0 {
			return Self::row(shape, band, random);
		}

		let mut places = Self::places(shape, band, random);

		for index in (1..places.len()).rev() {
			let other = random.next_index(index + 1);

			places.swap(index, other);
		}

		places
	}

	/// A sigil that runs straight along its row, from one end or the other.
	fn row(shape: &SigilShape, band: (f64, f64), random: &mut SeededRandom) -> Vec<SigilLight> {
		let mut places = Self::places(shape, band, random);

		if !random.next_chance(0.5) {
			places.reverse();
		}

		places
	}

	/// The lights of a sigil with `shape`, from left to right: evenly spaced
	/// across its span, each at one of three heights within `band` and never
	/// at the height of the one beside it.
	fn places(shape: &SigilShape, band: (f64, f64), random: &mut SeededRandom) -> Vec<SigilLight> {
		let (low, high) = band;
		let span = radians(shape.span_degrees);
		let count = shape.lights;
		let mut heights = vec![random.next_int(3)];

		while heights.len() < count {
			let last = heights[heights.len() - 1];

			heights.push((last + 1 + random.next_int(2)) % 3);
		}

		(0..count)
			.map(|column| {
				SigilLight {
					yaw: -span / 2.0 + span * column as f64 / (count as f64 - 1.0),
					pitch: low + (high - low) * heights[column] as f64 / 2.0,
				}
			})
			.collect()
	}

	/// Whether any stretch of the trace through `lights` passes within
	/// `clearance` of a light that is not one of its ends.
	fn passes_over(lights: &[SigilLight], clearance: f64) -> bool {
		lights.windows(2).enumerate().any(|(step, pair)| {
			lights.iter().enumerate().any(|(index, light)| {
				index != step
					&& index != step + 1
					&& Self::distance_to_stretch(light, &pair[0], &pair[1]) < clearance
			})
		})
	}

	/// How far `light` is, in radians, from the stretch of the trace between
	/// `from` and `to`: the shortest way round from one to the other, which
	/// is the straight line between them that the view draws.
	fn distance_to_stretch(light: &SigilLight, from: &SigilLight, to: &SigilLight) -> f64 {
		let (point, start, end) = (light.direction(), from.direction(), to.direction());
		// The pole of the great circle through both ends.
		let pole = start.cross(end).normalized();
		let off = point.dot(pole);
		let foot = (point - pole * off).normalized();
		let within = start.angle_to(foot) + foot.angle_to(end) <= start.angle_to(end) + 1e-9;

		if within {
			return math::asin(math::clamp(off.abs(), 0.0, 1.0));
		}

		math::min(point.angle_to(start), point.angle_to(end))
	}

	/// How many pairs of stretches of the trace through `lights` cross.
	/// Stretches that meet end to end do not count.
	fn count_crossings(lights: &[SigilLight]) -> usize {
		let mut count = 0;

		for first in 0..lights.len().saturating_sub(1) {
			for second in first + 2..lights.len().saturating_sub(1) {
				if Self::cross(
					&lights[first],
					&lights[first + 1],
					&lights[second],
					&lights[second + 1],
				) {
					count += 1;
				}
			}
		}

		count
	}

	fn cross(a: &SigilLight, b: &SigilLight, c: &SigilLight, d: &SigilLight) -> bool {
		let turn = |from: &SigilLight, to: &SigilLight, point: &SigilLight| {
			(to.yaw - from.yaw) * (point.pitch - from.pitch)
				- (to.pitch - from.pitch) * (point.yaw - from.yaw)
		};

		turn(a, b, c) * turn(a, b, d) < 0.0 && turn(c, d, a) * turn(c, d, b) < 0.0
	}
}

/// Something that happened while a sigil was traced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SigilEvent {
	/// The light at this index caught.
	LightCaught(usize),

	/// The light at this index went out: the next was too long coming.
	LightLost(usize),

	/// The last light caught. The husk shows itself.
	SigilClosed,
}

/// A sigil as it is traced: which of its lights have caught, and how near
/// the next one is to catching.
///
/// It reads no clock. Each frame it is told where the phone points, how
/// fast it is turning and how long the frame was.
#[derive(Clone, Debug, PartialEq)]
pub struct SigilTrace {
	pub sigil: Sigil,

	/// The bearing the sigil hangs around, in radians: where the reaper
	/// faced when it appeared.
	pub centre: f64,

	caught: usize,
	fill: f64,
	slack: f64,
	seconds: f64,
}

impl SigilTrace {
	pub const fn new(sigil: Sigil, centre: f64) -> Self {
		Self {
			sigil,
			centre,
			caught: 0,
			fill: 0.0,
			slack: 0.0,
			seconds: 0.0,
		}
	}

	/// How long the sigil has been traced, in seconds.
	pub const fn seconds(&self) -> f64 {
		self.seconds
	}

	/// How many lights have caught.
	pub const fn caught(&self) -> usize {
		self.caught
	}

	/// How near the next light is to catching, from 0 to 1.
	pub const fn fill(&self) -> f64 {
		self.fill
	}

	/// Whether every light has caught.
	pub fn is_closed(&self) -> bool {
		self.caught == self.sigil.lights.len()
	}

	/// How near the last light caught is to going out, from 0 to 1. Always 0
	/// for a sigil whose lights stay caught.
	pub fn fraying(&self) -> f64 {
		match self.sigil.fray_seconds {
			Some(seconds) if self.caught > 0 && !self.is_closed() => self.slack / seconds,
			_ => 0.0,
		}
	}

	/// Which way the light at `index` is from the reaper.
	pub fn direction_of(&self, index: usize) -> Vec3 {
		let light = self.sigil.lights.get(index).copied().unwrap_or(SigilLight {
			yaw: 0.0,
			pitch: 0.0,
		});

		Vec3::from_angles(self.centre + light.yaw, light.pitch)
	}

	/// Whether a view along `aim` rests on the next light.
	pub fn rests_on(&self, aim: Vec3) -> bool {
		!self.is_closed() && aim.angle_to(self.direction_of(self.caught)) <= self.sigil.radius
	}

	/// Takes in where the phone pointed, `aim`, and how fast it was turning,
	/// for the last `delta_seconds`, and answers what came of it.
	///
	/// The next light fills while the view rests on it and turns slower than
	/// [`HuntRules::SIGIL_STEADY_SPEED`]. Resting on it while turning faster
	/// holds it where it is, and looking away drains it.
	pub fn advance(&mut self, aim: Vec3, turn_speed: f64, delta_seconds: f64) -> Vec<SigilEvent> {
		if self.is_closed() {
			return Vec::new();
		}

		self.seconds += delta_seconds;

		let resting = self.rests_on(aim);
		let step = delta_seconds / self.sigil.hold_seconds;

		if resting && turn_speed <= HuntRules::SIGIL_STEADY_SPEED {
			self.fill += step;
		} else if !resting {
			self.fill = math::max(0.0, self.fill - step * HuntRules::SIGIL_DRAIN);
		}

		if self.fill >= 1.0 {
			let index = self.caught;

			self.caught += 1;
			self.fill = 0.0;
			self.slack = 0.0;

			let mut events = vec![SigilEvent::LightCaught(index)];

			if self.is_closed() {
				events.push(SigilEvent::SigilClosed);
			}

			return events;
		}

		// A light waits while the next is being held, and frays only while
		// the reaper is looking for it.
		let Some(fray) = self.sigil.fray_seconds else {
			return Vec::new();
		};

		if self.caught == 0 || resting {
			return Vec::new();
		}

		self.slack += delta_seconds;

		if self.slack < fray {
			return Vec::new();
		}

		self.caught -= 1;
		self.slack = 0.0;
		self.fill = 0.0;

		vec![SigilEvent::LightLost(self.caught)]
	}
}
