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
use reapers_rules::HuskRing;
use reapers_rules::SeededRandom;
use reapers_rules::SigilTrace;
use reapers_rules::Vec3;
use reapers_rules::wrap_angle;
use reapers_world::ArPose;

use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::WOUND;

/// The view: its size in layout units, how it is pointed, its lens, and
/// where its eye is.
#[derive(Clone, Copy, Debug)]
pub struct Lens {
	pub width: f64,
	pub height: f64,
	pub pose: ArPose,
	pub fov: f64,
	/// Where the eye is in the hunt's frame, when the platform draws the
	/// scene over its camera picture from wherever the phone really is: the
	/// room was charted around the first picture's eye, and the player can
	/// have moved since. What is anchored to the world — a husk, its seals,
	/// the room's grid — is drawn from here, so it sits where the camera
	/// shows it. Zero, and immaterial, in a scene the game draws itself.
	pub eye: Vec3,
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

	/// Where the point `point`, anchored to the hunt's world, falls on the
	/// screen as the eye sees it, or `None` behind it. The camera the
	/// platform draws with stands at the eye, not at the hunt's origin, so
	/// this — and not [`place`](Self::place) — is where a husk, a seal or a
	/// line of the room's grid is drawn once the player has moved.
	pub fn place_point(&self, point: Vec3) -> Option<(f64, f64)> {
		self.place(point - self.eye)
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

pub(crate) fn colour(rgb: Rgb, alpha: f64) -> Color {
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

	pub(crate) fn dot(&mut self, at: (f64, f64), radius: f64, paint: Color) {
		self.add(&Self::circle_data(at.0, at.1, radius), Some(paint), None);
	}

	/// A dot at each of `centres`, all drawn as one path.
	pub(crate) fn dots(
		&mut self,
		centres: impl IntoIterator<Item = (f64, f64)>,
		radius: f64,
		paint: Color,
	) {
		let data: String = centres
			.into_iter()
			.map(|at| Self::circle_data(at.0, at.1, radius))
			.collect();
		if !data.is_empty() {
			self.add(&data, Some(paint), None);
		}
	}

	pub(crate) fn ring(&mut self, at: (f64, f64), radius: f64, paint: Color, width: f64) {
		self.add(
			&Self::circle_data(at.0, at.1, radius),
			None,
			Some((paint, width)),
		);
	}

	/// An arc of `sweep` radians from `start`, clockwise on the screen, with
	/// 0 at three o'clock.
	pub(crate) fn arc(
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

	pub(crate) fn line(&mut self, from: (f64, f64), to: (f64, f64), paint: Color, width: f64) {
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

	/// A line through `points` in `tint` at `alpha`, broken wherever one is
	/// behind the player.
	pub fn line_through(
		&mut self,
		points: &[Option<(f64, f64)>],
		tint: Rgb,
		alpha: f64,
		width: f64,
	) {
		self.polyline(points, colour(tint, alpha), width);
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

/// The frame a hunt or a rift is drawn in: the veil's ink under it, unless
/// the platform draws its camera's picture behind the scene, which the
/// room must show through.
pub fn frame_class(camera: bool) -> &'static str {
	if camera {
		"relative w-full h-screen overflow-hidden"
	} else {
		"relative w-full h-screen overflow-hidden bg-[#07080b]"
	}
}

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

/// The ink under everything drawn over a camera's picture, so a light mark
/// holds over a bright room.
const SHADOW: Rgb = Rgb(0x02, 0x02, 0x03);

/// How near straight behind a light may be before the edge arrow settles
/// on one side: within this, the shortest turn either way is so nearly the
/// same that the arrow always says right, rather than flickering across
/// ±180° when the phone shakes.
const BEHIND_SETTLE: f64 = 5.0 * PI / 180.0;

/// Where the next light is from the middle of the view, and what the reaper
/// should do about it. This is the one reading the arrow and the words
/// share, so the drawn light, the reticle and the guidance agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NextLight {
	/// The ring rests on it and the view is still: it is filling.
	Hold,
	/// The ring rests on it, but the phone is turning too fast for it to
	/// fill.
	Slow,
	/// It is on screen, away from the ring.
	Aim,
	/// It is off the left edge.
	Left,
	/// It is off the right edge.
	Right,
	/// It is above the view.
	Up,
	/// It is below the view.
	Down,
}

impl NextLight {
	/// What to do about the next light of `trace`, as `lens` sees the room.
	pub fn of(lens: &Lens, trace: &SigilTrace) -> Self {
		if trace.is_closed() {
			return Self::Hold;
		}

		let target = trace.direction_of(trace.caught());

		if trace.rests_on(lens.pose.forward) {
			return if lens.pose.turn_speed <= HuntRules::SIGIL_STEADY_SPEED {
				Self::Hold
			} else {
				Self::Slow
			};
		}

		if lens
			.place(target)
			.is_some_and(|(x, y)| !off_screen(lens, x, y))
		{
			return Self::Aim;
		}

		let bearing = bearing(lens, target);
		let (across, up) = (bearing.cos(), bearing.sin());

		if across.abs() >= up.abs() {
			if across >= 0.0 {
				Self::Right
			} else {
				Self::Left
			}
		} else if up >= 0.0 {
			Self::Down
		} else {
			Self::Up
		}
	}
}

/// Which way on the screen `target` lies from the middle of the view, in
/// radians clockwise from the right edge: 0 is right, π/2 down, −π/2 up and
/// π or −π left.
///
/// It is read through the view's own right and up, so a rolled phone turns
/// the arrow with its picture. A light behind the player has no place on
/// the screen: its arrow names the shorter way round, and exactly behind
/// settles on the right (see [`BEHIND_SETTLE`]) rather than choosing by the
/// sign of a vanishing number.
pub(crate) fn bearing(lens: &Lens, target: Vec3) -> f64 {
	let aim = lens.pose;
	let target = target.normalized();
	let (across, up, ahead) = (
		target.dot(aim.right),
		target.dot(aim.up),
		target.dot(aim.forward),
	);

	if ahead > 0.0 {
		let half_height = (lens.fov / 2.0).tan();

		return (-up / ahead / half_height).atan2(across / ahead / (half_height * lens.aspect()));
	}

	let turn = wrap_angle(target.yaw() - aim.forward.yaw());

	if turn.abs() >= PI - BEHIND_SETTLE {
		return 0.0;
	}

	if across >= 0.0 { 0.0 } else { PI }
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
	sketch.polyline(&thread, colour(SHADOW, 0.5), 4.0);
	sketch.polyline(&thread, colour(light, 0.85 * (1.0 - 0.6 * fraying)), 2.0);
	for (index, place) in places.iter().enumerate() {
		let Some(at) = *place else {
			continue;
		};
		match index.cmp(&caught) {
			std::cmp::Ordering::Less => {
				sketch.dot(at, 6.5, colour(SHADOW, 0.55));
				sketch.dot(at, 5.0, colour(light, 0.95));
			}
			std::cmp::Ordering::Equal => {
				let glow = breath(trace.seconds());
				sketch.dot(at, 22.0, colour(light, 0.12 * glow));
				sketch.ring(at, 15.0, colour(SHADOW, 0.6), 4.0);
				sketch.ring(at, 14.0, colour(light, glow), 2.0);
				sketch.dot(at, 5.0, colour(SHADOW, 0.75));
				sketch.dot(at, 4.0, colour(BONE, glow));
			}
			std::cmp::Ordering::Greater => {
				sketch.ring(at, 9.0, colour(SHADOW, 0.45), 3.0);
				sketch.ring(at, 8.0, colour(light, 0.35), 1.0);
			}
		}
	}
	// The ring at the middle of the screen: what rests on a light. It has a
	// dark edge under it so it reads over a bright room, and its fill is an
	// arc rather than a colour alone.
	let centre = lens.centre();
	sketch.ring(centre, 26.0, colour(SHADOW, 0.55), 3.4);
	sketch.ring(centre, 26.0, colour(BONE, 0.35), 1.0);
	sketch.arc(
		centre,
		27.0,
		-FRAC_PI_2,
		TAU * trace.fill(),
		colour(SHADOW, 0.6),
		4.2,
	);
	sketch.arc(
		centre,
		26.0,
		-FRAC_PI_2,
		TAU * trace.fill(),
		colour(light, 1.0),
		2.6,
	);
	sketch.dot(centre, 2.6, colour(SHADOW, 0.7));
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

/// An arrow at the edge of the screen toward `target`, out of view. Its
/// angle comes from the camera's own axes (see [`bearing`]), so it follows
/// the picture when the phone is rolled, and a light behind the player
/// always names a side.
fn edge_pointer(sketch: &mut Sketch, lens: &Lens, target: Vec3, light: Rgb, pulse: f64) {
	let angle = bearing(lens, target);
	let (cx, cy) = lens.centre();
	let (rx, ry) = (lens.width * 0.42, lens.height * 0.38);
	let tip = (cx + rx * angle.cos(), cy + ry * angle.sin());
	let back = |side: f64| {
		let spread = angle + PI + side * 0.45;
		(tip.0 + 18.0 * spread.cos(), tip.1 + 18.0 * spread.sin())
	};
	let points = [Some(back(1.0)), Some(tip), Some(back(-1.0))];

	sketch.polyline(&points, colour(SHADOW, 0.8 * pulse), 6.0);
	sketch.polyline(&points, colour(light, pulse), 3.0);
}

/// The radius of a seal on screen: at least 44 units across, however far
/// away the husk is.
const SEAL_RADIUS: f64 = 26.0;

/// Everything over a fight: the ring under the husk, the crosshair, the
/// seals, the shriek's edge, the orb's ring and the bruise of a blow. The
/// orb itself is in the world, with the reaper's arms. `show_order` numbers
/// open seals in the order they opened (the Weigher's perk).
pub fn fight(
	sketch: &mut Sketch,
	lens: &Lens,
	sim: &HuntSim,
	light: Rgb,
	wound: f64,
	show_order: bool,
) {
	let seconds = sim.tick() as f64 / HuntRules::TICK_RATE as f64;
	let pose = sim.pose();
	// How the husk is seen from where the camera is, not from the hunt's
	// origin: the marks sit on what the player is shown.
	let way = pose.position - lens.eye;
	let seen = || lens.place_point(pose.position);
	let pulse = 0.55 + 0.45 * (sim.tick() as f64 / 9.0).sin();
	let pointer = |sketch: &mut Sketch| {
		if seen().is_none_or(|(x, y)| off_screen(lens, x, y)) {
			edge_pointer(sketch, lens, way, light, pulse);
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
			husk_ring(sketch, lens, sim, seconds);
			seals(sketch, lens, sim, light, seconds, show_order);
			orb_ring(sketch, lens, sim);
			crosshair(sketch, lens, sim, light);
			if sim.phase() == HuntPhase::Charge {
				// A ring that closes on the husk as it comes.
				if let Some(at) = seen() {
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
			husk_ring(sketch, lens, sim, seconds);
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
		HuntPhase::Broken => {
			pointer(sketch);
			husk_ring(sketch, lens, sim, seconds);
			orb_ring(sketch, lens, sim);
			crosshair(sketch, lens, sim, light);
		}
		HuntPhase::Capture => husk_ring(sketch, lens, sim, seconds),
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
		let Some(at) = lens.place_point(pose.seal_position(index)) else {
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

/// The gold of a husk weak enough for an orb, and of an excellent throw.
const ORB_GOLD: Rgb = Rgb(0xE9, 0xC2, 0x5C);

/// The colour of the ring under the husk. These are signals, the same for
/// every reaper, not the reaper's own light: white is open, amber is a
/// shriek coming, red is a charge, blue is stunned, gold is weak enough for
/// an orb.
fn ring_colour(ring: HuskRing) -> Rgb {
	match ring {
		HuskRing::Open => BONE,
		HuskRing::DrawingBreath => Rgb(0xE3, 0xA2, 0x3B),
		HuskRing::Charging => WOUND,
		HuskRing::Stunned => Rgb(0x6F, 0xA9, 0xE6),
		HuskRing::Weakened => ORB_GOLD,
	}
}

/// The colour of an orb's ring and of an orb thrown, at `grade`.
fn orb_grade_colour(grade: reapers_rules::CaptureGrade) -> Rgb {
	match grade {
		reapers_rules::CaptureGrade::Plain => crate::theme::ASH,
		reapers_rules::CaptureGrade::Great => BONE.lerp(ORB_GOLD, 0.5),
		reapers_rules::CaptureGrade::Excellent => ORB_GOLD,
	}
}

/// The ring under a husk that says what it is about to do.
fn husk_ring(sketch: &mut Sketch, lens: &Lens, sim: &HuntSim, seconds: f64) {
	let pose = sim.pose();
	let half_width = pose.half_width();
	let half_height = pose.half_height();
	// Two points of the room the ring is drawn round: under the middle of
	// the body, and a half a body out to one side, placed as the eye that
	// draws the scene sees them.
	let below = pose.position - Vec3::new(0.0, half_height * 0.85, 0.0);
	let across = below + Vec3::new(half_width * 1.3, 0.0, 0.0);
	let (Some(at), Some(edge)) = (lens.place_point(below), lens.place_point(across)) else {
		return;
	};
	let radius = f64::max(28.0, (edge.0 - at.0).hypot(edge.1 - at.1));
	let ring = sim.ring();
	let beat = 0.5 + 0.5 * (seconds * if ring == HuskRing::Open { 3.0 } else { 8.0 }).sin();
	let paint = colour(ring_colour(ring), 0.55 + 0.4 * beat);

	sketch.ring(
		at,
		radius + 6.0,
		colour(ring_colour(ring), 0.12 + 0.12 * beat),
		8.0,
	);
	sketch.ring(
		at,
		radius,
		paint,
		if ring == HuskRing::Open { 1.4 } else { 2.6 },
	);
}

/// What a press now would do, as the crosshair shows it. Each state has a
/// shape of its own, not only a colour, and the controls name the two that
/// need words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sight {
	/// Not on the husk: a press does nothing to it.
	Off,
	/// On the husk, but too far for the blade or the class weapon.
	TooFar,
	/// A press now would strike the husk.
	On,
	/// A press now would strike an open seal, for twice the harm.
	Seal,
}

impl Sight {
	/// What the crosshair shows for `sim` along `aim`.
	pub fn of(sim: &HuntSim, aim: Vec3) -> Self {
		if sim.on_open_seal(aim) {
			return Self::Seal;
		}
		if sim.in_reach(aim) {
			return Self::On;
		}
		let stats = sim.stats();
		let reach = match reapers_rules::Arsenal::archetype_of(sim.selected_slot()) {
			Some(reapers_rules::Archetype::Blade) => stats.blade_reach_radians,
			Some(reapers_rules::Archetype::Heavy) => stats.heavy_reach_radians,
			_ => return Self::Off,
		};
		if sim.pose().covers(sim.eye().point(), aim, reach) {
			Self::TooFar
		} else {
			Self::Off
		}
	}
}

/// The crosshair at the middle of the screen, where every weapon and every
/// orb goes: four open arms when a press would miss, arms closed in when it
/// would strike, a ring round them on an open seal, and a cross when the
/// husk is too far for the weapon in hand.
fn crosshair(sketch: &mut Sketch, lens: &Lens, sim: &HuntSim, light: Rgb) {
	let sight = Sight::of(sim, lens.pose.forward);
	let (cx, cy) = lens.centre();
	let (tint, alpha, width, gap) = match sight {
		Sight::Off => (BONE, 0.75, 1.6, 7.0),
		Sight::TooFar => (crate::theme::ASH, 0.9, 1.8, 7.0),
		Sight::On => (BONE, 1.0, 2.0, 4.0),
		Sight::Seal => (light, 1.0, 2.4, 4.0),
	};
	let paint = colour(tint, alpha);
	// A dark edge under every stroke, so the crosshair reads over a bright
	// room as well as a dark one.
	let shadow = colour(Rgb(0x02, 0x02, 0x03), 0.55);
	let arm = 9.0;
	let stroke = |sketch: &mut Sketch, from: (f64, f64), to: (f64, f64)| {
		sketch.line(from, to, shadow, width + 2.0);
		sketch.line(from, to, paint, width);
	};
	if sight == Sight::TooFar {
		for side in 0..4 {
			let angle = FRAC_PI_2 / 2.0 + f64::from(side) * FRAC_PI_2;
			let (dx, dy) = (angle.cos(), angle.sin());
			stroke(sketch, (cx, cy), (cx + dx * arm, cy + dy * arm));
		}
		return;
	}
	for side in 0..4 {
		let angle = f64::from(side) * FRAC_PI_2;
		let (dx, dy) = (angle.cos(), angle.sin());
		stroke(
			sketch,
			(cx + dx * gap, cy + dy * gap),
			(cx + dx * (gap + arm), cy + dy * (gap + arm)),
		);
	}
	sketch.dot((cx, cy), 1.6, paint);
	if sight == Sight::Seal {
		sketch.ring((cx, cy), gap + arm + 5.0, colour(light, 0.9), 1.6);
	}
}

/// While ORB is held: a ring that shrinks from the edge of the screen onto
/// the husk. Its thirds are drawn as faint guides, because where it is when
/// ORB is let go grades the throw.
fn orb_ring(sketch: &mut Sketch, lens: &Lens, sim: &HuntSim) {
	let Some(progress) = sim.orb_ring() else {
		return;
	};
	let pose = sim.pose();
	let husk = lens
		.place_point(pose.position)
		.unwrap_or_else(|| lens.centre());
	let (cx, cy) = husk;
	let husk_radius = f64::max(
		24.0,
		pose.half_height() / (pose.position - lens.eye).length() / (lens.fov / 2.0).tan()
			* lens.height
			/ 2.0,
	);
	let outer = f64::min(lens.width, lens.height) * 0.48;
	let radius_at = |share: f64| outer - (outer - husk_radius) * share;
	let guide = colour(BONE, 0.18);

	for third in [1.0 / 3.0, 2.0 / 3.0] {
		sketch.ring((cx, cy), radius_at(third), guide, 1.0);
	}
	sketch.ring((cx, cy), husk_radius, colour(BONE, 0.32), 1.0);
	let grade = sim.orb_grade(lens.pose.forward);
	sketch.ring(
		(cx, cy),
		radius_at(progress),
		colour(orb_grade_colour(grade), 1.0),
		2.0 + 2.0 * grade.index() as f64,
	);
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::Gait;
	use reapers_rules::Rarity;
	use reapers_rules::Sigil;
	use reapers_rules::SigilLight;
	use reapers_rules::radians;

	use super::*;

	fn lens() -> Lens {
		Lens {
			width: 390.0,
			height: 844.0,
			pose: ArPose::AHEAD,
			fov: super::super::arena::HUNT_FOV,
			eye: Vec3::ZERO,
		}
	}

	/// A lens looking where `pose` says, at the hunt's default lens.
	fn looking(pose: ArPose) -> Lens {
		Lens { pose, ..lens() }
	}

	/// A trace whose only light hangs along `direction`.
	fn trace_at(direction: Vec3) -> SigilTrace {
		let centre = direction.yaw();
		SigilTrace::new(
			Sigil {
				lights: vec![SigilLight {
					yaw: 0.0,
					pitch: direction.pitch(),
				}],
				radius: radians(6.0),
				hold_seconds: 0.6,
				fray_seconds: None,
			},
			centre,
		)
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
	fn a_moved_eye_sees_the_world_where_the_camera_shows_it() {
		// The player charted the room where they first stood, then moved
		// half a pace before the fight. The camera draws from where they
		// are, so the world's marks are placed from there — and pointing
		// the phone at a husk puts it under the crosshair, which the
		// direction from the origin (the rule the marks used to follow)
		// would miss by the width of the player's step on screen.
		let eye = Vec3::new(0.35, 0.05, -0.30);
		let husk = Vec3::new(0.0, -1.2, 2.4);
		let lens = Lens {
			pose: ArPose::looking((husk - eye).normalized(), 0.0),
			eye,
			..lens()
		};
		let (x, y) = lens.place_point(husk).unwrap();

		assert!((x - 195.0).abs() <= 1e-6 && (y - 422.0).abs() <= 1e-6);

		let (old_x, old_y) = lens.place(husk.normalized()).unwrap();
		let drift = (old_x - 195.0).hypot(old_y - 422.0);

		assert!(
			drift > 40.0,
			"the origin's direction is {drift} units from the middle, not far from it"
		);
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

	#[test]
	fn the_edge_arrow_is_read_through_the_camera_not_the_world() {
		let lens = lens();
		let right = bearing(&lens, Vec3::from_angles(0.5, 0.0));
		assert!(
			right.abs() < 1e-9,
			"a light to the right is at the right edge"
		);
		assert!(bearing(&lens, Vec3::from_angles(-0.5, 0.0)) < -3.0);
		let up = bearing(&lens, Vec3::from_angles(0.0, 0.6));
		assert!((up + FRAC_PI_2).abs() < 1e-9, "{up}");
		let down = bearing(&lens, Vec3::from_angles(0.0, -0.6));
		assert!((down - FRAC_PI_2).abs() < 1e-9, "{down}");

		// The phone rolled a quarter turn: the world's up is now the
		// screen's sideways, so a light along the view's own up is up.
		let rolled = looking(ArPose {
			forward: Vec3::FORWARD,
			up: Vec3::new(1.0, 0.0, 0.0),
			right: Vec3::new(0.0, -1.0, 0.0),
			turn_speed: 0.0,
		});
		let along_up = rolled.pose.forward * 0.4 + rolled.pose.up * 0.9;
		let angle = bearing(&rolled, along_up);
		assert!((angle + FRAC_PI_2).abs() < 1e-9, "{angle}");
		assert_eq!(NextLight::of(&rolled, &trace_at(along_up)), NextLight::Up);
	}

	#[test]
	fn a_light_out_of_view_names_a_side_and_behind_settles_on_one() {
		let lens = lens();
		let next = |direction| NextLight::of(&lens, &trace_at(direction));

		assert_eq!(next(Vec3::from_angles(0.5, 0.0)), NextLight::Right);
		assert_eq!(next(Vec3::from_angles(-0.5, 0.0)), NextLight::Left);
		assert_eq!(next(Vec3::from_angles(0.0, 0.6)), NextLight::Up);
		assert_eq!(next(Vec3::from_angles(0.0, -0.6)), NextLight::Down);

		// Behind: the side is the shorter turn. Exactly behind is settled
		// on the right, and so is the whole cone either side of ±180°, so
		// the arrow does not flicker when the phone shakes at the seam.
		assert_eq!(next(Vec3::from_angles(2.5, 0.0)), NextLight::Right);
		assert_eq!(next(Vec3::from_angles(-2.5, 0.0)), NextLight::Left);
		assert_eq!(next(Vec3::from_angles(PI, 0.0)), NextLight::Right);
		assert_eq!(next(Vec3::from_angles(-PI, 0.0)), NextLight::Right);
		assert_eq!(next(Vec3::from_angles(PI - 0.02, 0.0)), NextLight::Right);
		assert_eq!(next(Vec3::from_angles(-PI + 0.02, 0.0)), NextLight::Right);
	}

	#[test]
	fn the_light_the_ring_rests_on_is_held_and_fills() {
		let trace = SigilTrace::new(
			Sigil::draw(
				5,
				&Rarity::Common.sigil(),
				Gait::Ground,
				Difficulty::default(),
			),
			0.4,
		);
		let target = trace.direction_of(0);
		let at = looking(ArPose::looking(target, 0.0));
		let (x, y) = at.place(target).unwrap();
		assert!((x - at.width / 2.0).abs() < 1e-9);
		assert!((y - at.height / 2.0).abs() < 1e-9);
		assert_eq!(NextLight::of(&at, &trace), NextLight::Hold);

		// Turning too fast at the same light does not fill it, and the guide
		// says so rather than saying to hold.
		let hurried = looking(ArPose::looking(target, HuntRules::SIGIL_STEADY_SPEED * 2.0));
		assert_eq!(NextLight::of(&hurried, &trace), NextLight::Slow);
	}

	#[test]
	fn the_projection_edge_and_the_arrow_agree() {
		let lens = lens();
		let half_width = (lens.fov / 2.0).tan() * lens.aspect();
		let yaw = half_width.atan();
		let (x, _) = lens.place(Vec3::from_angles(yaw, 0.0)).unwrap();
		assert!((x - lens.width).abs() < 1e-6, "{x}");
		assert!(bearing(&lens, Vec3::from_angles(yaw, 0.0)).abs() < 1e-9);
	}
}
