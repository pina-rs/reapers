//! Everything a husk is fought in: the scene it is drawn in, the phone's
//! pose, and, once a fight is engaged, the rules of that fight. The Flutter
//! `HuntArena`.
//!
//! A lone hunt and a reaper's seat in a rift are both fought here. The
//! arena does not know which it is showing.

use std::rc::Rc;

use leptoskit::scene3d::Op;
use reapers_rules::HuntEvent;
use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;
use reapers_rules::HuskPose;
use reapers_rules::Nft;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_world::ArPose;
use reapers_world::OcclusionMap;
use reapers_world::to_platform;

use super::controller::HuntController;
use crate::render::camera::look_at;
use crate::render::husk::HuskMood;
use crate::render::husk::HuskRig;
use crate::render::reaction::HuskPlight;
use crate::render::scene::GameScene;
use crate::render::scene::Key;
use crate::render::scene::forget;
use crate::render::scene::light_hunt;
use crate::render::scene::scene;

/// The vertical field of view the hunt is drawn with when there is no
/// camera picture to match: a browser, a simulator, a refused permission.
pub const HUNT_FOV: f64 = 62.0 * std::f64::consts::PI / 180.0;

/// How long a husk takes to come apart once the fight is over.
const DISSOLVE_SECONDS: f64 = 1.7;
/// How long an absorbed husk convulses whole before it starts to come
/// apart: long enough for its death throes to be seen.
const DEATH_THROES_SECONDS: f64 = 0.35;
/// How long a husk takes to come together when it first appears.
const GATHER_SECONDS: f64 = 1.1;
/// How long the mark of a blow takes to fade.
const WOUND_SECONDS: f64 = 0.9;
/// How far a husk whose ward has broken has already come apart.
const BROKEN_DISSOLVE: f64 = 0.2;
/// How long a slash stays on the screen.
const SLASH_SECONDS: f64 = 0.35;

/// A cut on the screen where a strike went: where, from 0 to 1 across and
/// down the view, at what angle, and how old it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slash {
	pub x: f64,
	pub y: f64,
	pub angle: f64,
	pub age: f64,
}

impl Slash {
	/// How much of the cut is left, from 1 (fresh) to 0 (gone).
	pub fn left(&self) -> f64 {
		(1.0 - self.age / SLASH_SECONDS).clamp(0.0, 1.0)
	}
}

/// One husk's arena.
pub struct Arena {
	pub scene: Rc<GameScene>,
	pub rig: HuskRig,
	pub controller: Option<HuntController>,
	/// Where the husk stands until a fight is engaged.
	rest: Option<HuskPose>,
	/// How far the husk has come apart, from 0 (whole) to 1 (gone).
	pub come_apart: f64,
	/// How fresh the last blow the reaper took is, from 1 (just struck)
	/// down to 0.
	pub wound: f64,
	gathered: f64,
	over_for: f64,
	pub slashes: Vec<Slash>,
	slash_count: u32,
	/// The room as it was seen, to hide the husk behind, where it was.
	pub occlusion: Option<OcclusionMap>,
	/// The vertical field of view the husk is drawn with.
	pub fov: f64,
	/// The view's width over its height.
	pub aspect: f64,
	/// The camera last sent, so a still view sends nothing.
	camera: Option<([i32; 6], i32)>,
	/// Whether the scene is drawn over the camera's picture, from the
	/// platform's own view of the room.
	camera_picture: bool,
}

impl Arena {
	/// An arena under `key` for a husk of `vessel` dressed as `nft`, in a
	/// fresh scene.
	pub fn new(key: Key, vessel: Vessel, nft: &Nft, aspect: f64, camera_picture: bool) -> Self {
		// A new hunt starts afresh: the last husk, and its scene, go.
		forget(key);
		let scene = scene(key);
		let ops = scene.ops();
		light_hunt(ops, camera_picture);
		let rig = HuskRig::spawn(ops, vessel, nft);
		scene.flush();
		Self {
			scene,
			rig,
			controller: None,
			rest: None,
			come_apart: 0.0,
			wound: 0.0,
			gathered: 1.0,
			over_for: 0.0,
			slashes: Vec::new(),
			slash_count: 0,
			occlusion: None,
			fov: HUNT_FOV,
			aspect,
			camera: None,
			camera_picture,
		}
	}

	/// Where the phone points: the fight's (which may play itself), or the
	/// player's `live` pose.
	pub fn pose(&self, live: ArPose) -> ArPose {
		self.controller
			.as_ref()
			.map_or(live, |controller| controller.pose(live))
	}

	/// Stands the husk at `pose` until a fight is engaged.
	pub fn rest(&mut self, pose: HuskPose) {
		self.rest = Some(pose);
		self.rig.place(&pose);
	}

	/// Hands the arena the fight it is to show. A husk that was not in view
	/// gathers itself out of nothing where the fight begins. Where the room
	/// was seen, `occlusion` is how it looked.
	pub fn engage(&mut self, controller: HuntController, occlusion: Option<OcclusionMap>) {
		if self.rest.is_none()
			|| self
				.controller
				.as_ref()
				.is_some_and(|controller| controller.sim().is_over())
		{
			self.gathered = 0.0;
		}
		self.come_apart = 0.0;
		self.over_for = 0.0;
		self.wound = 0.0;
		self.slashes.clear();
		self.rest(controller.sim().pose());
		self.controller = Some(controller);
		self.occlusion = occlusion.filter(|map| map.pictures() > 0);
	}

	/// Whether a finished fight's husk has come apart completely.
	pub fn gone(&self) -> bool {
		self.controller
			.as_ref()
			.is_some_and(|controller| controller.sim().is_over())
			&& self.come_apart >= 1.0
	}

	/// Stands the player where the platform says the phone is, in its space:
	/// a platform that draws the scene over its camera picture draws it from
	/// there, not from the origin the scene's own camera stands at.
	pub fn anchor(&mut self, position: [f64; 3]) {
		if self.camera_picture {
			self.rig.set_origin(position.map(|value| value as f32));
		}
	}

	/// Points the scene's camera along `pose`, sending it only when it moved.
	pub fn aim(&mut self, pose: &ArPose) {
		let quantum = |value: f64| (value * 2000.0).round() as i32;
		let key = (
			[
				quantum(pose.forward.x),
				quantum(pose.forward.y),
				quantum(pose.forward.z),
				quantum(pose.up.x),
				quantum(pose.up.y),
				quantum(pose.up.z),
			],
			quantum(self.fov),
		);
		if self.camera == Some(key) {
			return;
		}
		self.camera = Some(key);
		let point = |direction: Vec3| to_platform(direction).map(|value| value as f32);
		self.scene.ops().push(Op::SetCamera {
			transform: look_at([0.0; 3], point(pose.forward), point(pose.up)),
			fov_y: self.fov as f32,
			near: 0.05,
			far: 120.0,
		});
	}

	/// While nothing is being fought: the husk drifts and the rules wait.
	pub fn idle(&mut self, delta: f64, live: ArPose) {
		self.aim(&live);
		self.rig.animate(
			self.scene.ops(),
			delta,
			HuskMood::default(),
			HuskPlight::default(),
		);
		self.scene.flush();
	}

	/// Advances the fight and everything drawn from it, with the player at
	/// `live`, and returns what happened. Before a fight is engaged this
	/// only idles.
	pub fn advance(&mut self, delta: f64, live: ArPose) -> Vec<HuntEvent> {
		let Some(controller) = self.controller.as_mut() else {
			self.idle(delta, live);
			return Vec::new();
		};
		let pose = controller.pose(live);
		let events = controller.frame(delta, &pose);
		let plays_itself = controller.plays_itself();
		for event in &events {
			match event {
				HuntEvent::StrikeLanded {
					empowered,
					seal,
					cleaved,
					..
				} => {
					self.rig.flash(if *empowered { 1.0 } else { 0.7 });
					self.rig.hit(
						Some(*seal),
						if *empowered { 1.0 } else { 0.75 },
						*empowered || !cleaved.is_empty(),
					);
					if plays_itself {
						let direction = self
							.controller
							.as_ref()
							.map(|controller| controller.sim().pose().seal_direction(*seal));
						if let Some(direction) = direction {
							self.mark_strike(&pose, direction);
						}
					}
				}
				HuntEvent::StrikeGlanced { .. } => {
					self.rig.flash(0.3);
					self.rig.hit(None, 0.3, false);
				}
				HuntEvent::Struck { .. } => self.wound = 1.0,
				HuntEvent::Parried | HuntEvent::Broken | HuntEvent::IceShattered => {
					self.rig.flash(1.0);
					self.rig.hit(None, 1.0, true);
				}
				HuntEvent::ArtLanded { hit: true, .. } => {
					self.rig.flash(0.8);
					self.rig.hit(None, 0.8, false);
				}
				_ => {}
			}
		}
		let Some(controller) = self.controller.as_ref() else {
			return events;
		};
		let sim = controller.sim();
		if sim.is_over() {
			self.over_for += delta;
			// An absorbed husk was broken first, and is already that far
			// gone.
			let absorbed = sim.phase() == HuntPhase::Absorbed;
			let throes = if absorbed { DEATH_THROES_SECONDS } else { 0.0 };
			self.come_apart = ((self.over_for - throes) / DISSOLVE_SECONDS)
				.clamp(if absorbed { BROKEN_DISSOLVE } else { 0.0 }, 1.0);
		} else {
			self.over_for = 0.0;
			self.come_apart = if sim.phase() == HuntPhase::Broken {
				BROKEN_DISSOLVE
			} else {
				0.0
			};
		}
		self.gathered = (self.gathered + delta / GATHER_SECONDS).min(1.0);
		self.wound = (self.wound - delta / WOUND_SECONDS).max(0.0);
		let (mood, plight) = mood_of(sim);
		let husk = sim.pose();
		self.rig.place(&husk);
		// Where the room was seen, whatever stands nearer than the husk in
		// its direction hides it. The occlusion map is a quarter-degree
		// picture of the room; the husk is hidden whole or not at all, by
		// whether its middle is behind.
		let hidden = self.occlusion.as_ref().is_some_and(|room| {
			room.hides(
				husk.yaw(),
				husk.pitch(),
				husk.distance() - husk.vessel.half_width(),
			)
		});
		let dissolve = self.come_apart.max(1.0 - self.gathered);
		self.rig
			.dissolve(if hidden { dissolve.max(0.92) } else { dissolve });
		self.rig.animate(self.scene.ops(), delta, mood, plight);
		self.aim(&pose);
		for slash in &mut self.slashes {
			slash.age += delta;
		}
		self.slashes.retain(|slash| slash.left() > 0.0);
		self.scene.flush();
		events
	}

	/// A tap at `x` and `y`, from 0 to 1 across and down the view: a strike
	/// along the ray through that point.
	pub fn strike(&mut self, x: f64, y: f64, live: ArPose) {
		let fov = self.fov;
		let aspect = self.aspect;
		let Some(controller) = self.controller.as_mut() else {
			return;
		};
		let striking = matches!(
			controller.sim().phase(),
			HuntPhase::Stalk
				| HuntPhase::Fight
				| HuntPhase::Windup
				| HuntPhase::Shriek
				| HuntPhase::Charge
		);
		if !striking {
			return;
		}
		let pose = controller.pose(live);
		controller.tap(pose.ray_through(x * 2.0 - 1.0, 1.0 - y * 2.0, fov, aspect));
		self.add_slash(x, y);
	}

	/// Leaves a slash where `direction` falls on the screen.
	fn mark_strike(&mut self, pose: &ArPose, direction: Vec3) {
		if let Some((x, y)) = pose.project(direction, self.fov, self.aspect) {
			self.add_slash(f64::midpoint(x, 1.0), (1.0 - y) / 2.0);
		}
	}

	fn add_slash(&mut self, x: f64, y: f64) {
		self.slash_count += 1;
		self.slashes.push(Slash {
			x,
			y,
			// Cuts alternate direction, like a blade drawn back and forth.
			angle: if self.slash_count.is_multiple_of(2) {
				-0.6
			} else {
				0.6
			} + x * 0.8,
			age: 0.0,
		});
	}
}

/// What the rules say a husk is doing and suffering, for the rig to act.
pub fn mood_of(sim: &HuntSim) -> (HuskMood, HuskPlight) {
	let fury = match sim.phase() {
		HuntPhase::Windup => sim.shriek_progress(),
		HuntPhase::Shriek | HuntPhase::Charge => 1.0,
		_ => 0.0,
	};
	(
		HuskMood {
			speed: sim.speed(),
			// Its mouth opens as it draws breath and stays open for the
			// shriek.
			gape: fury,
			frozen: sim.frozen(),
			burning: sim.burning(),
		},
		HuskPlight {
			ward: if sim.ward_max() <= 0.0 {
				0.0
			} else {
				sim.ward() / sim.ward_max()
			},
			fighting: sim.phase() != HuntPhase::Stalk,
			fury,
			staggered: sim.staggered(),
			broken: sim.phase() == HuntPhase::Broken,
			reap: if sim.phase() == HuntPhase::Broken && sim.reap_begun() {
				sim.reap_progress()
			} else {
				0.0
			},
			dying: sim.phase() == HuntPhase::Absorbed,
			frozen: sim.frozen(),
		},
	)
}
