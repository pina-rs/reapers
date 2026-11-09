//! Everything a husk is fought in: the scene it is drawn in, the phone's
//! pose, and, once a fight is engaged, the rules of that fight. The Flutter
//! `HuntArena`.
//!
//! A lone hunt and a reaper's seat in a rift are both fought here. The
//! arena does not know which it is showing.

use std::rc::Rc;

use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use reapers_rules::HuntEvent;
use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;
use reapers_rules::HuskPose;
use reapers_rules::Nft;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::WeaponKind;
use reapers_world::ArPose;
use reapers_world::OcclusionMap;
use reapers_world::to_platform;

use super::arms::Arms;
use super::arms::HuskMarks;
use super::controller::HuntController;
use crate::render::camera::look_at;
use crate::render::husk::HuskMood;
use crate::render::husk::HuskRig;
use crate::render::juice::dive_zoom;
use crate::render::juice::zoomed_fov;
use crate::render::orb::soul_taken;
use crate::render::reaction::HuskPlight;
use crate::render::scene::GameScene;
use crate::render::scene::Key;
use crate::render::scene::forget;
use crate::render::scene::light_hunt;
use crate::render::scene::scene;
use crate::render::turn::multiply;
use crate::seams::ViewmodelProps;
use crate::seams::WorldView;
use crate::seams::dive;
use crate::seams::shake_turn;

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
/// How high above the ground under the husk an orb that has landed rests,
/// in metres: its own radius.
const ORB_REST: f64 = 0.126;
/// How long a soul let out of an orb takes to gather itself again.
const REFORM_SECONDS: f64 = 0.5;
/// How quickly the arms' sense of the view's turn follows it, per second.
const TURN_FOLLOW: f64 = 12.0;

/// One husk's arena.
pub struct Arena {
	/// The scene's name.
	key: Key,
	pub scene: Rc<GameScene>,
	pub rig: HuskRig,
	pub controller: Option<HuntController>,
	/// The reaper's own arms, once a fight is engaged.
	pub arms: Option<Arms>,
	/// How much of the husk an orb has drawn in, from 0 to 1.
	taken: f64,
	/// The view's turn, in radians a second to the right and up, and the
	/// way it faced last tick.
	turn_rate: [f64; 2],
	facing: Option<(f64, f64)>,
	/// The view's height in layout units, for the shake's few pixels.
	pub height: f64,
	/// Where the husk stands until a fight is engaged.
	rest: Option<HuskPose>,
	/// How far the husk has come apart, from 0 (whole) to 1 (gone).
	pub come_apart: f64,
	/// How fresh the last blow the reaper took is, from 1 (just struck)
	/// down to 0.
	pub wound: f64,
	gathered: f64,
	over_for: f64,
	/// The room as it was seen, to hide the husk behind, where it was.
	pub occlusion: Option<OcclusionMap>,
	/// The vertical field of view the husk is drawn with.
	pub fov: f64,
	/// The view's width over its height.
	pub aspect: f64,
	/// The camera last sent, so a still view sends nothing, and its turn.
	camera: Option<([i32; 10], i32)>,
	turned: [f32; 4],
	/// Where the room was charted from, in the scene's space, once a fight is
	/// drawn in the room's own frame rather than around wherever the phone is
	/// now. `None` until then, and the husk follows the phone.
	room_frame: Option<[f32; 3]>,
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
			key,
			scene,
			rig,
			controller: None,
			arms: None,
			taken: 0.0,
			turn_rate: [0.0; 2],
			facing: None,
			height: 800.0,
			rest: None,
			come_apart: 0.0,
			wound: 0.0,
			gathered: 1.0,
			over_for: 0.0,
			occlusion: None,
			fov: HUNT_FOV,
			aspect,
			camera: None,
			turned: [0.0, 0.0, 0.0, 1.0],
			room_frame: None,
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

	/// Hands the arena the fight it is to show, fought with `heavy` in the
	/// heavy slot. A husk that was not in view gathers itself out of nothing
	/// where the fight begins. Where the room was seen, `occlusion` is how it
	/// looked.
	pub fn engage(
		&mut self,
		controller: HuntController,
		occlusion: Option<OcclusionMap>,
		heavy: WeaponKind,
	) {
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
		self.rest(controller.sim().pose());
		self.arms = Some(Arms::new(controller.sim(), heavy));
		self.taken = 0.0;
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
	/// there, not from the origin the scene's own camera stands at. Once the
	/// room has been charted, the fight stays in the chart's frame and does
	/// not follow the phone.
	pub fn anchor(&mut self, position: [f64; 3]) {
		if self.camera_picture && self.room_frame.is_none() {
			self.rig.set_origin(position.map(|value| value as f32));
		}
	}

	/// Draws the fight in the frame the room was charted in: `origin` is where
	/// the phone's eye was when the first accepted picture was taken, in the
	/// scene's space. The husk then stands where the plan puts it in the room,
	/// however the player moves, and the room's occlusion is measured from the
	/// same place. A scene the game draws itself stands at the hunt's origin,
	/// so only a camera's picture needs the phone's own.
	pub fn ground_in_room(&mut self, origin: [f64; 3]) {
		let origin = if self.camera_picture {
			origin.map(|value| value as f32)
		} else {
			[0.0; 3]
		};
		self.room_frame = Some(origin);
		self.rig.set_origin(origin);
	}

	/// Points the scene's camera along `pose`, sending it only when it moved.
	/// Where the arena draws its own picture, the dive lens narrows it and
	/// a blow shakes it, by a few pixels at most; a camera's picture of the
	/// room is never moved.
	pub fn aim(&mut self, pose: &ArPose) {
		let point = |direction: Vec3| to_platform(direction).map(|value| value as f32);
		let mut camera = look_at([0.0; 3], point(pose.forward), point(pose.up));
		let fov = self.view_fov();
		if let Some(shake) = self.shake(fov) {
			camera.rotation = multiply(camera.rotation, shake);
		}
		self.turned = camera.rotation;
		let quantum = |value: f64| (value * 2000.0).round() as i32;
		let mut key = [0; 10];
		for (slot, value) in key.iter_mut().zip(
			[pose.forward, pose.up]
				.iter()
				.flat_map(|v| [v.x, v.y, v.z])
				.chain(camera.rotation.iter().map(|&v| f64::from(v)).take(4)),
		) {
			*slot = quantum(value);
		}
		let key = (key, quantum(fov));
		if self.camera == Some(key) {
			return;
		}
		self.camera = Some(key);
		self.scene.ops().push(Op::SetCamera {
			transform: camera,
			fov_y: fov as f32,
			near: 0.05,
			far: 120.0,
		});
	}

	/// Whether the arena draws its own picture of the world, so the dive
	/// lens and the shake may move it: not over a camera's picture, which
	/// cannot zoom or shake with it, and not for a player who asks for less
	/// motion.
	fn moves_its_picture(&self) -> bool {
		!self.camera_picture && !crate::platform::reduced_motion()
	}

	/// The vertical field of view the world is drawn with now: narrowed by
	/// the dive lens as the reaper lunges in.
	pub fn view_fov(&self) -> f64 {
		if !self.moves_its_picture() {
			return self.fov;
		}
		let (Some(arms), Some(controller)) = (&self.arms, &self.controller) else {
			return self.fov;
		};
		let sim = controller.sim();
		let props = arms.props(sim, None, [0.0; 2]);
		let lens = dive(props.action, props.action_seconds(sim.tick() as f64));
		let zoom = dive_zoom(lens, sim.pose().distance() as f32);
		f64::from(zoomed_fov(self.fov as f32, zoom))
	}

	/// The turn a blow's shake gives the world's camera now, if any.
	fn shake(&self, fov: f64) -> Option<[f32; 4]> {
		if !self.moves_its_picture() {
			return None;
		}
		let arms = self.arms.as_ref()?;
		let tick = self.controller.as_ref()?.sim().tick() as f64;
		let juice = arms.juice();
		(juice.trauma_at(tick) > 0.0)
			.then(|| shake_turn(juice, tick, fov as f32, self.height as f32))
	}

	/// The reaper's own arms and the light of their blows, for the
	/// viewmodel seam, once a fight is engaged.
	pub fn viewmodel(&self) -> Option<ViewmodelProps> {
		let controller = self.controller.as_ref()?;
		let arms = self.arms.as_ref()?;
		let sim = controller.sim();
		let marks = self.marks();
		let husk = sim.pose();
		let floor = husk.ground.unwrap_or(-sim.plan().eye_height());
		let world = WorldView {
			scene: self.key,
			camera: Transform {
				translation: self.rig.origin(),
				rotation: self.turned,
				scale: [1.0; 3],
			},
			fov_y: self.view_fov() as f32,
			aspect: self.aspect as f32,
			husk: marks.scene_point(husk.position),
			ground: marks.scene_point(Vec3::new(
				husk.position.x,
				floor + ORB_REST,
				husk.position.z,
			)),
		};
		let turn = self.turn_rate.map(|rate| rate as f32);
		Some(arms.props(sim, Some(world), turn))
	}

	fn marks(&self) -> HuskMarks {
		HuskMarks {
			origin: self.rig.origin(),
			light: self.rig.light,
		}
	}

	/// Follows how fast the view turns, for the arms to trail it.
	fn follow_turn(&mut self, delta: f64, pose: &ArPose) {
		let facing = (pose.forward.yaw(), pose.forward.pitch());
		if let Some((yaw, pitch)) = self.facing
			&& delta > 0.0
		{
			let turned = [
				reapers_rules::wrap_angle(facing.0 - yaw) / delta,
				(facing.1 - pitch) / delta,
			];
			let follow = 1.0 - (-TURN_FOLLOW * delta).exp();
			for (rate, now) in self.turn_rate.iter_mut().zip(turned) {
				*rate += (now - *rate) * follow;
			}
		}
		self.facing = Some(facing);
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
	/// `live`, and returns what happened, each on the rules' tick it happened
	/// on. Before a fight is engaged this only idles.
	pub fn advance(&mut self, delta: f64, live: ArPose) -> Vec<(i64, HuntEvent)> {
		let Some(controller) = self.controller.as_mut() else {
			self.idle(delta, live);
			return Vec::new();
		};
		let pose = controller.pose(live);
		let events = controller.frame(delta, &pose);
		let marks = self.marks();
		if let (Some(arms), Some(controller)) = (self.arms.as_mut(), self.controller.as_ref()) {
			let sim = controller.sim();
			for (tick, event) in &events {
				arms.hear(sim, *tick, event, &marks);
			}
			arms.follow(sim);
		}
		self.follow_turn(delta, &pose);
		for (_, event) in &events {
			match event {
				HuntEvent::HitLanded {
					empowered,
					seal,
					cleaved,
					critical,
					..
				} => {
					self.rig
						.flash(if *empowered || *critical { 1.0 } else { 0.7 });
					self.rig.hit(
						*seal,
						if *empowered || *critical { 1.0 } else { 0.75 },
						*empowered || *critical || !cleaved.is_empty(),
					);
				}
				HuntEvent::Struck { .. } => self.wound = 1.0,
				HuntEvent::Parried
				| HuntEvent::Broken { .. }
				| HuntEvent::HuskCaptured(_)
				| HuntEvent::OrbBrokeFree(_)
				| HuntEvent::IceShattered => {
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
		let hidden = self
			.occlusion
			.as_ref()
			.is_some_and(|room| hidden_by(room, &husk));
		// A soul an orb draws in goes into it; one that breaks out gathers
		// itself again, never at once.
		let taken = sim
			.orb()
			.map_or(0.0, |orb| soul_taken(&orb, sim.tick() as f64));
		self.taken = taken.max(self.taken - delta / REFORM_SECONDS);
		let dissolve = self.come_apart.max(1.0 - self.gathered).max(self.taken);
		self.rig
			.dissolve(if hidden { dissolve.max(0.92) } else { dissolve });
		self.rig.animate(self.scene.ops(), delta, mood, plight);
		self.aim(&pose);
		self.scene.flush();
		events
	}
}

/// Whether the room, where it was seen, hides `husk`: whether anything
/// stands nearer than the husk in its direction. The occlusion map is a
/// quarter-degree picture of the room; the husk is hidden whole or not at
/// all, by whether its middle is behind. A top it stands on is under its
/// middle, so it never hides it.
pub fn hidden_by(room: &OcclusionMap, husk: &HuskPose) -> bool {
	room.hides(
		husk.yaw(),
		husk.pitch(),
		husk.distance() - husk.half_width(),
	)
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
			broken: matches!(sim.phase(), HuntPhase::Broken | HuntPhase::Capture),
			reap: 0.0,
			dying: sim.phase() == HuntPhase::Absorbed,
			frozen: sim.frozen(),
		},
	)
}

#[cfg(test)]
mod tests {
	use reapers_rules::Vessel;
	use reapers_rules::market_listings;

	use super::*;
	use crate::render::scene::HUNT;

	fn penguin() -> Nft {
		market_listings()
			.into_iter()
			.find(|nft| nft.collection.vessel() == Vessel::Penguin)
			.expect("a penguin on the market")
	}

	#[test]
	fn a_charted_fight_is_drawn_in_the_room_frame() {
		let nft = penguin();
		let mut arena = Arena::new(HUNT, nft.collection.vessel(), &nft, 0.5, true);

		arena.ground_in_room([1.0, 1.5, -2.0]);

		assert_eq!(arena.rig.origin(), [1.0, 1.5, -2.0]);

		// The player walking after the room was read does not drag it along.
		arena.anchor([9.0, 9.0, 9.0]);

		assert_eq!(arena.rig.origin(), [1.0, 1.5, -2.0]);
	}

	#[test]
	fn a_scene_the_game_draws_keeps_its_own_origin() {
		let nft = penguin();
		let mut arena = Arena::new(HUNT, nft.collection.vessel(), &nft, 0.5, false);

		arena.ground_in_room([1.0, 1.5, -2.0]);

		assert_eq!(
			arena.rig.origin(),
			[0.0; 3],
			"the game's own camera stands at the hunt's origin"
		);
	}
}
