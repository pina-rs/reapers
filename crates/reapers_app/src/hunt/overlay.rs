//! Everything drawn flat over the hunt that is anchored to the world: the
//! veil's motes and horizon where there is no camera, the sigil hung in the
//! room, the seals on the husk, the reap arc, the mark of each strike, and
//! where to look when the husk is out of view. The Flutter `VeilBackdrop`,
//! `SigilPainter` and `HuntHudPainter`, drawn as one `Vector`.
//!
//! It reads the rules and the pose and decides nothing. Points are placed
//! in the view's own units (the screen's layout size), rounded to a whole
//! unit, so that a still view draws exactly the same picture.

use std::f64::consts::FRAC_PI_2;
use std::f64::consts::PI;
use std::f64::consts::TAU;
use std::fmt::Write as _;
use std::sync::LazyLock;

use leptoskit::prelude::*;
use reapers_rules::HuntPhase;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::SeededRandom;
use reapers_rules::SigilTrace;
use reapers_rules::Vec3;
use reapers_rules::wrap_angle;
use reapers_world::ArPose;

use super::arena::Slash;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::WOUND;

/// The view: its size in layout units, how it is pointed, and its lens.
#[derive(Clone, Copy, Debug)]
pub struct Lens {
	pub width: f64,
	pub height: f64,
	pub pose: ArPose,
	pub fov: f64,
}

impl Lens {
	fn aspect(&self) -> f64 {
		self.width / self.height
	}

	/// Where `direction` falls on the screen, or `None` behind the player.
	pub fn place(&self, direction: Vec3) -> Option<(f64, f64)> {
		self.pose
			.project(direction, self.fov, self.aspect())
			.map(|(x, y)| {
				(
					f64::midpoint(x, 1.0) * self.width,
					(1.0 - y) / 2.0 * self.height,
				)
			})
	}

	fn centre(&self) -> (f64, f64) {
		(self.width / 2.0, self.height / 2.0)
	}
}

/// A flat drawing being put together.
#[derive(Default)]
pub struct Sketch {
	paths: Vec<VectorPath>,
}

fn colour(rgb: Rgb, alpha: f64) -> Color {
	Color::rgba(
		rgb.0,
		rgb.1,
		rgb.2,
		(alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
	)
}

fn unit(value: f64) -> f64 {
	value.round()
}

impl Sketch {
	fn add(&mut self, data: &str, fill: Option<Color>, stroke: Option<(Color, f64)>) {
		let Ok(data) = PathData::parse(data) else {
			return;
		};
		let mut path = VectorPath::new(data);
		if let Some(fill) = fill {
			path = path.fill(fill);
		}
		if let Some((paint, width)) = stroke
			&& let Ok(stroke) = Stroke::new(paint, width as f32)
		{
			path = path.stroke(stroke.round());
		}
		self.paths.push(path);
	}

	fn circle_data(x: f64, y: f64, radius: f64) -> String {
		let (x, y, radius) = (unit(x), unit(y), radius.max(0.5));
		format!(
			"M{} {} a{radius} {radius} 0 1 0 {} 0 a{radius} {radius} 0 1 0 {} 0 ",
			x - radius,
			y,
			2.0 * radius,
			-2.0 * radius
		)
	}

	fn dot(&mut self, at: (f64, f64), radius: f64, paint: Color) {
		self.add(&Self::circle_data(at.0, at.1, radius), Some(paint), None);
	}

	fn ring(&mut self, at: (f64, f64), radius: f64, paint: Color, width: f64) {
		self.add(
			&Self::circle_data(at.0, at.1, radius),
			None,
			Some((paint, width)),
		);
	}

	/// An arc of `sweep` radians from `start`, clockwise on the screen, with
	/// 0 at three o'clock.
	fn arc(
		&mut self,
		at: (f64, f64),
		radius: f64,
		start: f64,
		sweep: f64,
		paint: Color,
		width: f64,
	) {
		if sweep.abs() < 1e-3 {
			return;
		}
		if sweep.abs() >= TAU - 1e-3 {
			self.ring(at, radius, paint, width);
			return;
		}
		let end = start + sweep;
		let point = |angle: f64| {
			(
				unit(at.0 + radius * angle.cos()),
				unit(at.1 + radius * angle.sin()),
			)
		};
		let (from, to) = (point(start), point(end));
		let large = i32::from(sweep.abs() > PI);
		let clockwise = i32::from(sweep > 0.0);
		self.add(
			&format!(
				"M{} {} A{radius} {radius} 0 {large} {clockwise} {} {}",
				from.0, from.1, to.0, to.1
			),
			None,
			Some((paint, width)),
		);
	}

	fn line(&mut self, from: (f64, f64), to: (f64, f64), paint: Color, width: f64) {
		self.add(
			&format!(
				"M{} {} L{} {}",
				unit(from.0),
				unit(from.1),
				unit(to.0),
				unit(to.1)
			),
			None,
			Some((paint, width)),
		);
	}

	/// A line through `points`, broken wherever one is behind the player.
	fn polyline(&mut self, points: &[Option<(f64, f64)>], paint: Color, width: f64) {
		let mut data = String::new();
		let mut drawing = false;
		for point in points {
			match point {
				Some((x, y)) => {
					let _ = write!(
						data,
						"{}{} {} ",
						if drawing { "L" } else { "M" },
						unit(*x),
						unit(*y)
					);
					drawing = true;
				}
				None => drawing = false,
			}
		}
		if !data.is_empty() {
			self.add(&data, None, Some((paint, width)));
		}
	}

	/// The drawing, filling its parent.
	pub fn into_view(self, width: f64, height: f64) -> Element {
		let Ok(view_box) = ViewBox::new(0.0, 0.0, width as f32, height as f32) else {
			return View();
		};
		Vector(view_box)
			.scaling(VectorScaling::Stretch)
			.paths(self.paths)
			.class("absolute inset-0 w-full h-full")
			.aria_hidden(true)
	}

	pub fn is_empty(&self) -> bool {
		self.paths.is_empty()
	}
}

/// The veil's motes: directions scattered once over the whole sphere, with
/// how large and how bright each is.
static MOTES: LazyLock<Vec<(Vec3, f64, f64)>> = LazyLock::new(|| {
	let mut random = SeededRandom::new(0x7E11);
	(0..140)
		.map(|_| {
			let yaw = random.next_range(-PI, PI);
			let pitch = random.next_range(-0.85, 0.95).asin();
			(
				Vec3::from_angles(yaw, pitch),
				random.next_range(0.6, 2.4),
				random.next_range(0.15, 0.8),
			)
		})
		.collect()
});

/// The spirit realm with no world behind it: drifting motes and a faint
/// horizon, for when there is no camera to look through. The motes are
/// fixed in the hunt's space, so they slide across the screen as the view
/// turns: that is what tells the player they are turning at all.
pub fn veil(sketch: &mut Sketch, lens: &Lens) {
	// A band of cold light where the ground would meet the sky.
	let level: Vec<_> = (0..=24)
		.map(|step| {
			let yaw = lens.pose.forward.yaw() + (f64::from(step) / 24.0 - 0.5) * PI;
			lens.place(Vec3::from_angles(yaw, 0.0))
		})
		.collect();
	sketch.polyline(&level, colour(Rgb(0x6C, 0x84, 0xA0), 0.16), 26.0);
	sketch.polyline(&level, colour(Rgb(0x9F, 0xB8, 0xD0), 0.22), 1.0);
	// Motes in three brightnesses, each drawn as one path.
	for (low, high) in [(0.0, 0.35), (0.35, 0.6), (0.6, 1.0)] {
		let mut data = String::new();
		for (direction, size, glow) in MOTES.iter() {
			if *glow < low || *glow >= high {
				continue;
			}
			let Some((x, y)) = lens.place(*direction) else {
				continue;
			};
			if x < -10.0 || y < -10.0 || x > lens.width + 10.0 || y > lens.height + 10.0 {
				continue;
			}
			data.push_str(&Sketch::circle_data(x, y, *size));
		}
		if !data.is_empty() {
			sketch.add(
				&data,
				Some(colour(BONE, f64::midpoint(low, high) * 0.5)),
				None,
			);
		}
	}
}

/// How brightly the next light of a sigil shows at `seconds`: it breathes,
/// slowly, so that the eye finds it.
fn breath(seconds: f64) -> f64 {
	0.75 + 0.25 * (seconds * 2.4).sin()
}

/// The sigil hung in the room: its lights where they are around the player,
/// the thread drawn through those that have caught, the ring at the middle
/// that rests on them and fills as one catches, and which way to turn when
/// the next light is out of view.
pub fn sigil(sketch: &mut Sketch, lens: &Lens, trace: &SigilTrace, light: Rgb) {
	let count = trace.sigil.lights.len();
	let caught = trace.caught();
	let places: Vec<_> = (0..count)
		.map(|index| lens.place(trace.direction_of(index)))
		.collect();
	// The thread through what has caught, bent with the room: each stretch
	// is split so that it follows the sphere round the player.
	let mut thread = Vec::new();
	for index in 0..caught {
		let from = trace.direction_of(index);
		let to = if index + 1 < caught {
			trace.direction_of(index + 1)
		} else {
			lens.pose.forward
		};
		for step in 0..=8 {
			let t = f64::from(step) / 8.0;
			let between = (from * (1.0 - t) + to * t).normalized();
			thread.push(lens.place(between));
		}
		thread.push(None);
	}
	let fraying = trace.fraying();
	sketch.polyline(&thread, colour(light, 0.85 * (1.0 - 0.6 * fraying)), 2.0);
	for (index, place) in places.iter().enumerate() {
		let Some(at) = *place else {
			continue;
		};
		match index.cmp(&caught) {
			std::cmp::Ordering::Less => sketch.dot(at, 5.0, colour(light, 0.95)),
			std::cmp::Ordering::Equal => {
				let glow = breath(trace.seconds());
				sketch.dot(at, 22.0, colour(light, 0.12 * glow));
				sketch.ring(at, 14.0, colour(light, glow), 2.0);
				sketch.dot(at, 4.0, colour(BONE, glow));
			}
			std::cmp::Ordering::Greater => sketch.ring(at, 8.0, colour(light, 0.35), 1.0),
		}
	}
	// The ring at the middle of the screen: what rests on a light.
	let centre = lens.centre();
	sketch.ring(centre, 26.0, colour(BONE, 0.35), 1.0);
	sketch.arc(
		centre,
		26.0,
		-FRAC_PI_2,
		TAU * trace.fill(),
		colour(light, 1.0),
		2.6,
	);
	sketch.dot(centre, 1.8, colour(BONE, 0.9));
	if caught < count && places[caught].is_none_or(|(x, y)| off_screen(lens, x, y)) {
		edge_pointer(
			sketch,
			lens,
			trace.direction_of(caught),
			light,
			breath(trace.seconds()),
		);
	}
}

fn off_screen(lens: &Lens, x: f64, y: f64) -> bool {
	x < 0.0 || y < 0.0 || x > lens.width || y > lens.height
}

/// An arrow at the edge of the screen toward `target`, out of view.
fn edge_pointer(sketch: &mut Sketch, lens: &Lens, target: Vec3, light: Rgb, pulse: f64) {
	let aim = lens.pose.forward;
	let angle = (-(target.pitch() - aim.pitch())).atan2(wrap_angle(target.yaw() - aim.yaw()));
	let (cx, cy) = lens.centre();
	let (rx, ry) = (lens.width * 0.42, lens.height * 0.38);
	let tip = (cx + rx * angle.cos(), cy + ry * angle.sin());
	let back = |side: f64| {
		let spread = angle + PI + side * 0.45;
		(tip.0 + 18.0 * spread.cos(), tip.1 + 18.0 * spread.sin())
	};
	sketch.polyline(
		&[Some(back(1.0)), Some(tip), Some(back(-1.0))],
		colour(light, pulse),
		3.0,
	);
}

/// The radius of a seal on screen: at least 44 units across, however far
/// away the husk is.
const SEAL_RADIUS: f64 = 26.0;

/// Everything over a fight: the stalk's reticle, the seals, the shriek's
/// edge, the reap arc, the bruise of a blow and the slashes. `show_order`
/// numbers open seals in the order they opened (the Weigher's perk).
pub fn fight(
	sketch: &mut Sketch,
	lens: &Lens,
	sim: &HuntSim,
	light: Rgb,
	wound: f64,
	slashes: &[Slash],
	show_order: bool,
) {
	let seconds = sim.tick() as f64 / HuntRules::TICK_RATE as f64;
	let husk = sim.pose().direction();
	let pulse = 0.55 + 0.45 * (sim.tick() as f64 / 9.0).sin();
	let pointer = |sketch: &mut Sketch| {
		if lens.place(husk).is_none_or(|(x, y)| off_screen(lens, x, y)) {
			edge_pointer(sketch, lens, husk, light, pulse);
		}
	};
	match sim.phase() {
		HuntPhase::Stalk => {
			pointer(sketch);
			// The ring that closes as the reaper steadies on an unaware husk,
			// and frays as the husk grows alarmed.
			let steady = sim.steadiness();
			let alarm = sim.awareness();
			let eased = 1.0 - (1.0 - steady).powi(2);
			let radius = 40.0 - 18.0 * eased;
			let tint = BONE.lerp(WOUND, alarm);
			let (cx, cy) = lens.centre();
			let at = (cx + alarm * 3.0 * (sim.tick() as f64 * 1.7).sin(), cy);
			sketch.ring(at, radius, colour(tint, 0.35), 1.0);
			sketch.arc(at, radius, -FRAC_PI_2, TAU * steady, colour(tint, 1.0), 2.4);
			sketch.dot((cx, cy), 1.6, colour(tint, 1.0));
		}
		HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Charge => {
			pointer(sketch);
			seals(sketch, lens, sim, light, seconds, show_order);
			if sim.phase() == HuntPhase::Charge {
				// A ring that closes on the husk as it comes.
				if let Some(at) = lens.place(husk) {
					let left = 1.0 - sim.charge_progress();
					sketch.ring(
						at,
						30.0 + 140.0 * left,
						colour(WOUND, 0.4 + 0.4 * pulse),
						2.0,
					);
				}
			}
		}
		HuntPhase::Shriek => {
			pointer(sketch);
			// A still ring that shows how much of the hold is left.
			let centre = lens.centre();
			sketch.ring(centre, 54.0, colour(BONE, 0.25), 1.0);
			sketch.arc(
				centre,
				54.0,
				-FRAC_PI_2,
				TAU * (1.0 - sim.shriek_progress()),
				colour(light, 0.95),
				3.0,
			);
		}
		HuntPhase::Broken => reap_arc(sketch, lens, sim, light, seconds),
		HuntPhase::Absorbed | HuntPhase::Escaped => {}
	}
	if matches!(sim.phase(), HuntPhase::Windup | HuntPhase::Shriek) {
		// The world goes dark around the husk: a frame that closes in from
		// the edges. It swells and holds; it never flashes, and it is never
		// red.
		let depth = if sim.phase() == HuntPhase::Shriek {
			1.0
		} else {
			sim.shriek_progress()
		};
		edge(
			sketch,
			lens,
			Rgb(0x02, 0x02, 0x03),
			0.55 * depth,
			40.0 + 30.0 * depth,
		);
	}
	if wound > 0.0 {
		// A blow the reaper took: the edges bruise, and the bruise fades.
		edge(sketch, lens, WOUND, 0.45 * wound, 26.0);
	}
	for slash in slashes {
		let (x, y) = (slash.x * lens.width, slash.y * lens.height);
		let reach = 46.0;
		let (dx, dy) = (slash.angle.cos() * reach, slash.angle.sin() * reach);
		sketch.line(
			(x - dx, y - dy),
			(x + dx, y + dy),
			colour(BONE, 0.85 * slash.left()),
			2.0 + 2.0 * slash.left(),
		);
	}
}

/// A frame of `width` units round the screen.
fn edge(sketch: &mut Sketch, lens: &Lens, tint: Rgb, alpha: f64, width: f64) {
	let (w, h) = (lens.width, lens.height);
	sketch.add(
		&format!(
			"M0 0 H{w} V{h} H0 Z M{width} {width} V{} H{} V{width} Z",
			unit(h - width),
			unit(w - width)
		),
		Some(colour(tint, alpha)),
		None,
	);
}

fn seals(
	sketch: &mut Sketch,
	lens: &Lens,
	sim: &HuntSim,
	light: Rgb,
	seconds: f64,
	show_order: bool,
) {
	let pose = sim.pose();
	for index in 0..sim.vessel().seals().len() {
		if !sim.seal_open(index) {
			continue;
		}
		let Some(at) = lens.place(pose.seal_direction(index)) else {
			continue;
		};
		let rank = sim.seal_rank(index).unwrap_or(0);
		let first = show_order && rank == 0;
		let beat = 0.5 + 0.5 * (seconds * 5.0 + index as f64 * 1.7).sin();
		let ring = colour(light, 0.75 + 0.25 * beat);
		sketch.dot(at, SEAL_RADIUS * 1.5, colour(light, 0.14));
		sketch.ring(
			at,
			SEAL_RADIUS - 3.0 * beat,
			ring,
			if first { 2.6 } else { 1.8 },
		);
		sketch.dot(at, 3.2, colour(BONE, 1.0));
		// Four ticks turning round the ring: a seal reads as a mechanism
		// that is open, not as a button.
		for tick in 0..4 {
			let angle = seconds * 1.4 + index as f64 + f64::from(tick) * FRAC_PI_2;
			let (dx, dy) = (angle.cos(), angle.sin());
			sketch.line(
				(
					at.0 + dx * (SEAL_RADIUS + 3.0),
					at.1 + dy * (SEAL_RADIUS + 3.0),
				),
				(
					at.0 + dx * (SEAL_RADIUS + 9.0),
					at.1 + dy * (SEAL_RADIUS + 9.0),
				),
				ring,
				1.8,
			);
		}
		if show_order {
			// The order, as pips under the ring: one for the first to strike.
			for pip in 0..=rank.min(4) {
				sketch.dot(
					(
						at.0 - 8.0 * rank.min(4) as f64 / 2.0 + 8.0 * pip as f64,
						at.1 + SEAL_RADIUS + 16.0,
					),
					2.4,
					colour(BONE, 0.9),
				);
			}
		}
	}
}

fn reap_arc(sketch: &mut Sketch, lens: &Lens, sim: &HuntSim, light: Rgb, seconds: f64) {
	const SAMPLES: u32 = 28;
	let points: Vec<_> = (0..=SAMPLES)
		.map(|sample| lens.place(sim.reap_arc_point(f64::from(sample) / f64::from(SAMPLES))))
		.collect();
	sketch.polyline(&points, colour(BONE, 0.4), 2.0);
	if sim.reap_begun() {
		let reached = (sim.reap_progress() * f64::from(SAMPLES)).floor() as usize;
		sketch.polyline(
			&points[..=reached.min(SAMPLES as usize)],
			colour(light, 1.0),
			5.0,
		);
	}
	match points[0] {
		Some(start) if !sim.reap_begun() => {
			let beat = 0.5 + 0.5 * (seconds * 6.0).sin();
			sketch.ring(start, 14.0 + 8.0 * beat, colour(light, 1.0 - beat), 2.0);
			sketch.dot(start, 7.0, colour(light, 1.0));
		}
		None => edge_pointer(sketch, lens, sim.reap_arc_point(0.0), light, 0.9),
		Some(_) => {}
	}
	// Where the reaper is aiming: the tip of the blade.
	let centre = lens.centre();
	sketch.ring(centre, 11.0, colour(BONE, 1.0), 2.0);
	sketch.dot(centre, 2.0, colour(BONE, 1.0));
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::Gait;
	use reapers_rules::Rarity;
	use reapers_rules::Sigil;

	use super::*;

	fn lens() -> Lens {
		Lens {
			width: 390.0,
			height: 844.0,
			pose: ArPose::AHEAD,
			fov: super::super::arena::HUNT_FOV,
		}
	}

	#[test]
	fn straight_ahead_is_the_middle_of_the_screen_and_behind_is_nowhere() {
		let lens = lens();
		let (x, y) = lens.place(Vec3::FORWARD).unwrap();
		assert!((x - 195.0).abs() < 1e-9 && (y - 422.0).abs() < 1e-9);
		assert_eq!(lens.place(Vec3::new(0.0, 0.0, -1.0)), None);
		let (right, _) = lens.place(Vec3::from_angles(0.2, 0.0)).unwrap();
		assert!(right > 195.0, "the hunt's yaw turns right");
		let (_, up) = lens.place(Vec3::from_angles(0.0, 0.2)).unwrap();
		assert!(up < 422.0, "and its pitch looks up");
	}

	#[test]
	fn a_still_view_draws_the_same_picture() {
		let trace = SigilTrace::new(
			Sigil::draw(
				5,
				&Rarity::Rare.sigil(),
				Gait::Ground,
				Difficulty::default(),
			),
			0.0,
		);
		let draw = || {
			let mut sketch = Sketch::default();
			veil(&mut sketch, &lens());
			sigil(&mut sketch, &lens(), &trace, BONE);
			sketch.paths
		};
		assert_eq!(draw(), draw());
		assert_ne!(draw().len(), 0);
	}
}
