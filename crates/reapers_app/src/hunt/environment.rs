//! Weapon marks on surfaces the phone actually measured. These are visual
//! effects, not damage to the player's room or part of the combat replay.
//! A miss queues its original ray; XR answers it against planes and depth.
//! Marks stay in world space, so turning the phone does not drag them along.

use std::collections::VecDeque;

use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::Art;
use reapers_rules::ArtRules;
use reapers_rules::HuntEvent;
use reapers_rules::Vec3;
use reapers_rules::WeaponKind;

use super::overlay::Lens;
use super::overlay::Sketch;
use super::overlay::colour;
use crate::render::scene::Key;
use crate::render::scene::scene;
use crate::theme::Rgb;

const MAX_MARKS: usize = 64;
const MAX_PENDING: usize = 8;
const MARK_SECONDS: f64 = 45.0;
const SPARK_SECONDS: f64 = 0.4;

/// Which weapon left a mark. A bell's pressure ring is not an axe's cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
	Bullet,
	Knife,
	Heavy(WeaponKind),
	Art(Art),
}

impl Kind {
	fn reach(self) -> f64 {
		match self {
			Self::Bullet => 25.0,
			Self::Knife | Self::Heavy(_) => Arsenal::CLOSE_METRES,
			Self::Art(_) => ArtRules::FARTHEST_METRES,
		}
	}

	fn radius(self) -> f64 {
		match self {
			Self::Bullet => 0.025,
			Self::Knife => 0.09,
			Self::Heavy(WeaponKind::Spear) => 0.07,
			Self::Heavy(WeaponKind::Scythe | WeaponKind::Greatsword | WeaponKind::Axe) => 0.23,
			Self::Heavy(_) => 0.17,
			Self::Art(Art::Cinder | Art::Rime) => 0.28,
			Self::Art(Art::Gale) => 0.45,
		}
	}

	fn tint(self) -> Rgb {
		match self {
			Self::Art(Art::Cinder) => Rgb(255, 138, 55),
			Self::Art(Art::Rime) => Rgb(155, 225, 255),
			Self::Art(Art::Gale) | Self::Heavy(WeaponKind::Bell) => Rgb(198, 219, 208),
			_ => Rgb(204, 191, 162),
		}
	}
}

/// One accepted weapon use, captured when it happened rather than when an
/// asynchronous hit test finishes. All coordinates use the rules' axes.
#[derive(Clone, Copy, Debug)]
pub struct Request {
	pub kind: Kind,
	pub tick: i64,
	pub origin: Vec3,
	pub direction: Vec3,
	issued: f64,
}

/// A real surface hit. A missing hit never invents a wall in the distance.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
	pub at: Vec3,
	pub normal: Vec3,
	pub distance: f64,
}

#[derive(Clone, Copy, Debug)]
struct Mark {
	request: Request,
	surface: Surface,
	since: f64,
}

/// A bounded, per-arena collection of surface marks and outstanding rays.
pub struct Environment {
	key: Key,
	seconds: f64,
	pending: VecDeque<Request>,
	marks: VecDeque<Mark>,
}

impl Default for Environment {
	fn default() -> Self {
		Self::new(super::super::render::scene::HUNT)
	}
}

impl Environment {
	pub fn new(key: Key) -> Self {
		Self {
			key,
			seconds: 0.0,
			pending: VecDeque::new(),
			marks: VecDeque::new(),
		}
	}

	pub fn advance(&mut self, delta: f64) {
		self.seconds += delta.max(0.0);
		self.marks
			.retain(|mark| self.seconds - mark.since < MARK_SECONDS);
		self.pending
			.retain(|request| self.seconds - request.issued < 0.5);
	}

	/// Only actual misses mark the room. A hit on the husk must not leave a
	/// bullet hole on the wall behind it. Reloads and cooldown presses do not
	/// enqueue anything. Arts use their landing direction, not the gaze now.
	/// `origin` anchors the hunt in platform space; `eye` is the player's
	/// current position inside that hunt, as the camera draws it.
	pub fn hear(
		&mut self,
		events: &[(i64, HuntEvent)],
		origin: Vec3,
		eye: Vec3,
		aim: Vec3,
		heavy: WeaponKind,
	) {
		for (tick, event) in events {
			let (kind, direction) = match event {
				HuntEvent::Missed(Archetype::Gun) => (Kind::Bullet, aim),
				HuntEvent::Missed(Archetype::Blade) => (Kind::Knife, aim),
				HuntEvent::Missed(Archetype::Heavy) => (Kind::Heavy(heavy), aim),
				HuntEvent::ArtLanded {
					art,
					at,
					hit: false,
					..
				} => (Kind::Art(*art), (*at - eye).normalized()),
				_ => continue,
			};
			if self.pending.len() == MAX_PENDING {
				self.pending.pop_front();
			}
			self.pending.push_back(Request {
				kind,
				tick: *tick,
				origin: origin + eye,
				direction,
				issued: self.seconds,
			});
		}
	}

	pub fn take_requests(&mut self) -> Vec<Request> {
		self.pending.drain(..).collect()
	}

	/// Keeps a hit only if it is finite, fresh, in front, and within this
	/// weapon's reach. A knife cannot cut a wall across the room.
	pub fn land(&mut self, request: Request, surface: Surface) {
		let finite = |point: Vec3| [point.x, point.y, point.z].into_iter().all(f64::is_finite);
		if !finite(surface.at)
			|| !finite(surface.normal)
			|| !surface.distance.is_finite()
			|| surface.normal.length() < 0.5
			|| surface.distance < 0.05
			|| surface.distance > request.kind.reach()
			|| self.seconds - request.issued > 0.5
			|| (surface.at - request.origin).dot(request.direction) <= 0.0
		{
			return;
		}
		if request.kind == Kind::Bullet {
			scene(self.key).set_shot_target(
				request.tick,
				reapers_world::to_platform(surface.at).map(|v| v as f32),
			);
		}
		if self.marks.len() == MAX_MARKS {
			self.marks.pop_front();
		}
		self.marks.push_back(Mark {
			request,
			surface,
			since: self.seconds,
		});
	}

	/// Projects surface-aligned scars and short sparks over the camera. Each
	/// point is projected afresh from the current eye, including translation.
	pub fn draw(&self, sketch: &mut Sketch, lens: &Lens, eye: Vec3) {
		for mark in &self.marks {
			mark.draw(sketch, lens, eye, self.seconds - mark.since);
		}
	}
}

impl Mark {
	fn basis(self) -> (Vec3, Vec3) {
		let normal = self.surface.normal.normalized();
		let axis = if normal.y.abs() < 0.9 {
			Vec3::UP
		} else {
			Vec3::new(1.0, 0.0, 0.0)
		};
		let across = normal.cross(axis).normalized();
		(across, normal.cross(across).normalized())
	}

	fn point(self, lens: &Lens, eye: Vec3, x: f64, y: f64) -> Option<(f64, f64)> {
		let (across, up) = self.basis();
		lens.place(self.surface.at + across * x + up * y - eye)
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "a surface stroke combines its projection and paint"
	)]
	fn stroke(
		self,
		sketch: &mut Sketch,
		lens: &Lens,
		eye: Vec3,
		points: &[(f64, f64)],
		alpha: f64,
		width: f64,
		tint: Rgb,
	) {
		let points: Vec<_> = points
			.iter()
			.map(|&(x, y)| self.point(lens, eye, x, y))
			.collect();
		sketch.line_through(&points, tint, alpha, width);
	}

	fn draw(self, sketch: &mut Sketch, lens: &Lens, eye: Vec3, age: f64) {
		let Some(centre) = lens.place(self.surface.at - eye) else {
			return;
		};
		if centre.0 < -lens.width
			|| centre.0 > lens.width * 2.0
			|| centre.1 < -lens.height
			|| centre.1 > lens.height * 2.0
		{
			return;
		}
		let kind = self.request.kind;
		let radius = kind.radius();
		let alpha = ((MARK_SECONDS - age) / 5.0).clamp(0.0, 1.0);
		let tint = kind.tint();
		match kind {
			Kind::Art(Art::Cinder) => {
				self.patch(
					sketch,
					lens,
					eye,
					radius,
					colour(Rgb(18, 12, 10), alpha * 0.8),
				);
				self.ring(
					sketch,
					lens,
					eye,
					radius * 0.8,
					alpha * (-age / 3.0).exp(),
					tint,
				);
				self.cracks(sketch, lens, eye, radius, alpha * 0.5, Rgb(45, 31, 24));
			}
			Kind::Art(Art::Rime) => {
				self.patch(sketch, lens, eye, radius, colour(tint, alpha * 0.24));
				self.cracks(sketch, lens, eye, radius, alpha, Rgb(227, 248, 255));
				self.ring(sketch, lens, eye, radius, alpha * 0.4, tint);
			}
			Kind::Knife
			| Kind::Heavy(WeaponKind::Scythe | WeaponKind::Greatsword | WeaponKind::Axe) => {
				let line = [
					(-radius, -radius * 0.4),
					(-radius * 0.2, 0.0),
					(radius, radius * 0.3),
				];
				self.stroke(sketch, lens, eye, &line, alpha * 0.9, 5.0, Rgb(8, 8, 10));
				self.stroke(sketch, lens, eye, &line, alpha, 1.5, tint);
			}
			Kind::Art(Art::Gale) | Kind::Heavy(WeaponKind::Bell) => {
				if age < 0.8 {
					self.ring(
						sketch,
						lens,
						eye,
						radius * (0.3 + age),
						(1.0 - age / 0.8) * 0.7,
						tint,
					);
				}
			}
			_ => {
				self.patch(
					sketch,
					lens,
					eye,
					radius * 0.65,
					colour(Rgb(8, 9, 11), alpha * 0.9),
				);
				self.ring(sketch, lens, eye, radius, alpha * 0.85, tint);
				self.cracks(sketch, lens, eye, radius, alpha, tint);
				if matches!(kind, Kind::Bullet | Kind::Heavy(WeaponKind::Spear)) {
					sketch.dot(centre, 2.0, colour(Rgb(5, 5, 6), alpha));
				}
			}
		}
		self.sparks(sketch, lens, eye, age, tint);
	}

	fn patch(
		self,
		sketch: &mut Sketch,
		lens: &Lens,
		eye: Vec3,
		radius: f64,
		paint: leptoskit::prelude::Color,
	) {
		let points: Option<Vec<_>> = (0..12)
			.map(|i| {
				let angle = f64::from(i) * std::f64::consts::TAU / 12.0;
				let edge = if i % 3 == 0 { radius * 0.85 } else { radius };
				self.point(lens, eye, edge * angle.cos(), edge * angle.sin())
			})
			.collect();
		if let Some(points) = points {
			sketch.patch(&points, paint);
		}
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "a surface ring combines its projection and paint"
	)]
	fn ring(self, sketch: &mut Sketch, lens: &Lens, eye: Vec3, radius: f64, alpha: f64, tint: Rgb) {
		let points: Vec<_> = (0..=16)
			.map(|i| {
				let angle = f64::from(i) / 16.0 * std::f64::consts::TAU;
				let chip = if i % 3 == 0 { 1.2 } else { 1.0 };
				(radius * angle.cos() * chip, radius * angle.sin() * chip)
			})
			.collect();
		self.stroke(sketch, lens, eye, &points, alpha, 2.5, tint);
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "surface cracks combine their projection and paint"
	)]
	fn cracks(
		self,
		sketch: &mut Sketch,
		lens: &Lens,
		eye: Vec3,
		radius: f64,
		alpha: f64,
		tint: Rgb,
	) {
		for i in 0..5 {
			let angle =
				f64::from(i) * std::f64::consts::TAU / 5.0 + (self.request.tick % 13) as f64;
			let (x, y) = (angle.cos(), angle.sin());
			self.stroke(
				sketch,
				lens,
				eye,
				&[
					(x * radius * 0.2, y * radius * 0.2),
					(x * radius, y * radius * 0.8),
					(x * radius * 1.6, y * radius * 1.4),
				],
				alpha * 0.7,
				1.2,
				tint,
			);
		}
	}

	fn sparks(self, sketch: &mut Sketch, lens: &Lens, eye: Vec3, age: f64, tint: Rgb) {
		if age >= SPARK_SECONDS {
			return;
		}
		let normal = self.surface.normal.normalized();
		let (across, up) = self.basis();
		let reach = self.request.kind.radius().max(0.08);
		for i in 0..7 {
			let angle = f64::from(i) * 2.4;
			let path = normal * 0.25 + across * (angle.cos() * reach) + up * (angle.sin() * reach);
			let a = self.surface.at + path * (age / SPARK_SECONDS) - Vec3::UP * (age * age * 0.5);
			let b = a - path * 0.12;
			if let (Some(a), Some(b)) = (lens.place(a - eye), lens.place(b - eye)) {
				sketch.line(a, b, colour(tint, 1.0 - age / SPARK_SECONDS), 1.5);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use reapers_world::ArPose;

	use super::*;

	fn miss(kind: Archetype, distance: f64) -> (Environment, Request, Surface) {
		let mut environment = Environment::default();
		environment.hear(
			&[(1, HuntEvent::Missed(kind))],
			Vec3::ZERO,
			Vec3::ZERO,
			Vec3::FORWARD,
			WeaponKind::Axe,
		);
		let request = environment.take_requests()[0];
		let surface = Surface {
			at: Vec3::FORWARD * distance,
			normal: -Vec3::FORWARD,
			distance,
		};
		(environment, request, surface)
	}

	#[test]
	fn rays_start_at_the_moved_eye_and_arts_point_at_their_landing() {
		let mut room = Environment::default();
		let origin = Vec3::new(3.0, 0.2, -2.0);
		let eye = Vec3::new(0.4, 0.1, 0.3);
		let at = Vec3::new(0.0, -0.5, 2.0);
		room.hear(
			&[
				(1, HuntEvent::Missed(Archetype::Gun)),
				(
					2,
					HuntEvent::ArtLanded {
						art: Art::Cinder,
						at,
						hit: false,
						damage: 0.0,
					},
				),
			],
			origin,
			eye,
			Vec3::FORWARD,
			WeaponKind::Axe,
		);
		let rays = room.take_requests();
		assert_eq!(rays[0].origin, origin + eye);
		assert_eq!(rays[0].direction, Vec3::FORWARD);
		assert_eq!(rays[1].origin, origin + eye);
		assert_eq!(rays[1].direction, (at - eye).normalized());
	}

	#[test]
	fn a_missed_shot_marks_a_measured_wall_but_a_knife_cannot_reach_it() {
		let (mut gun, request, wall) = miss(Archetype::Gun, 3.0);
		gun.land(request, wall);
		assert_eq!(gun.marks.len(), 1);
		let (mut knife, request, wall) = miss(Archetype::Blade, 3.0);
		knife.land(request, wall);
		assert!(knife.marks.is_empty());
	}

	#[test]
	fn marks_follow_the_surface_instead_of_the_crosshair() {
		let (mut room, request, wall) = miss(Archetype::Gun, 2.0);
		room.land(request, wall);
		let mark = room.marks[0];
		let lens = Lens {
			width: 390.0,
			height: 844.0,
			pose: ArPose::AHEAD,
			fov: 1.0,
			eye: Vec3::ZERO,
		};
		let centre = mark.point(&lens, Vec3::ZERO, 0.0, 0.0).unwrap();
		assert!((centre.0 - 195.0).abs() < 0.01);
		let moved = mark
			.point(&lens, Vec3::new(0.5, 0.0, 0.0), 0.0, 0.0)
			.unwrap();
		assert!(
			moved.0 < centre.0,
			"walking right leaves the mark on the wall to the left"
		);
	}

	#[test]
	fn no_hit_no_mark_and_old_or_invalid_results_are_discarded() {
		let (mut room, request, mut wall) = miss(Archetype::Gun, 2.0);
		wall.at.x = f64::NAN;
		room.land(request, wall);
		assert!(room.marks.is_empty());
		let (mut room, request, wall) = miss(Archetype::Gun, 2.0);
		room.advance(0.6);
		room.land(request, wall);
		assert!(room.marks.is_empty());
	}

	#[test]
	fn impact_storage_stays_bounded_and_expires() {
		let (mut room, request, wall) = miss(Archetype::Gun, 2.0);
		for _ in 0..1000 {
			room.land(request, wall);
		}
		assert_eq!(room.marks.len(), MAX_MARKS);
		room.advance(MARK_SECONDS + 1.0);
		assert!(room.marks.is_empty());
	}
}
