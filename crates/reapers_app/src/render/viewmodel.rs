//! What the reaper sees of itself: a skeletal right arm holding the chosen
//! weapon low in the bottom right of the view, and a left arm that comes up
//! to reload and to throw the orb. The Flutter `ViewmodelRig`, on
//! `leptoskit_3d`.
//!
//! It is one model, `viewmodel.glb`, in a scene of its own with a fixed
//! camera at the eye, drawn over the hunt. Nothing in it is animated by a
//! clip: every tick the rig turns [`super::arms::pose`] into part offsets,
//! which the host eases over the tick on its own display clock, so the
//! arms move at the display's rate while the game samples at thirty ticks
//! a second.
//!
//! Each moving part is one mesh with one material: at most six draws for
//! the busiest moment, a reload or a throw. `viewmodel_budget`'s tests hold
//! every action to that, and to six thousand triangles, by reading the
//! built model.

use leptoskit::scene3d::Backdrop;
use leptoskit::scene3d::Blend;
use leptoskit::scene3d::Light;
use leptoskit::scene3d::MaterialOverride;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use leptoskit::scene3d::client::Scene;
use reapers_rules::Archetype;

use super::arms::ArmPose;
use super::arms::DRUM_GONE;
use super::arms::LeftHolds;
use super::arms::Pose;
use super::camera::Point;
use super::juice::Shake;
use super::puppet::Puppet;
use super::turn::add;
use super::turn::axis_angle;
use super::turn::inverse;
use super::turn::multiply;
use super::turn::rotate;
use super::turn::scale;
use super::turn::sub;
use super::turn::view_turn;

/// The model.
pub const VIEWMODEL_MODEL: &str = "models/viewmodel.glb";

/// The view's own camera: about 55 degrees tall, at the eye.
pub const VIEWMODEL_FOV: f32 = 55.0 * std::f32::consts::PI / 180.0;

/// Soul light, in linear RGB: the colour of every glow the reaper makes.
pub const SOUL: [f32; 3] = [0.36, 1.0, 0.8];

/// Where the grip rests, low in the bottom right of a portrait view (x
/// right, y up, z away from the eye).
const REST: Point = [0.078, -0.2, 0.47];
/// The arm turned in toward the middle and raised, so the forearm runs
/// away to the bottom right corner, and canted inward.
const REST_TURN: [f32; 3] = [-0.3, 0.24, -0.12];
/// The hand bent at the wrist to bring the weapon back level.
const REST_WRIST: [f32; 3] = [0.08, -0.2, -0.06];

// Where the parts' pivots sit in the model, glTF's frame (+Z toward the
// muzzle, +Y up), from art/blender/viewmodel_parts. A test reads them back
// out of the model.
/// The elbow, from the grip.
pub const ELBOW: Point = [0.012, -0.035, -0.44];
/// The wrist, from the grip.
pub const WRIST: Point = [0.0, 0.0, -0.139_96];
/// The pistol's trigger, from the grip: what an active reload spins about.
pub const TRIGGER: Point = [0.0, 0.028, 0.02];
/// The pistol's drum, from its grip.
pub const DRUM: Point = [0.0, 0.058, 0.027];
/// The pistol's muzzle, from its grip.
pub const MUZZLE: Point = [0.0, 0.0752, 0.236];
/// The left palm, from the left arm's grip.
pub const PALM: Point = [-0.004, 0.0, 0.004];
/// The scythe's blade, from its grip, at full size.
const SCYTHE_HEAD: Point = [0.5, 1.45, 0.0];
/// Where the left palm meets the pistol's drum, from the drum.
const PALM_TO_DRUM: Point = [-0.012, -0.028, 0.0];

/// How a weapon sits in the fist: leaned forward so a long weapon's head
/// comes into view, turned about its haft, shrunk to read at arm's length,
/// and the arm moved and turned to suit it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hold {
	lean: f32,
	twist: f32,
	scale: f32,
	rest: Point,
	turn: [f32; 3],
	wrist: [f32; 3],
}

/// The grave pistol, the bone knife and the scythe in the model.
const PISTOL: &str = "weapon_gravePistol";
const KNIFE: &str = "weapon_boneKnife";
const SCYTHE: &str = "weapon_scythe";

/// The part the right hand holds for a slot of `archetype`, if any. Every
/// class weapon is drawn as the scythe until its own is built. The art is
/// thrown, not held, so its hand is empty.
pub const fn weapon_part(archetype: Archetype) -> Option<&'static str> {
	match archetype {
		Archetype::Gun => Some(PISTOL),
		Archetype::Blade => Some(KNIFE),
		Archetype::Heavy => Some(SCYTHE),
		Archetype::Art => None,
	}
}

const fn hold(archetype: Archetype) -> Hold {
	match archetype {
		Archetype::Gun | Archetype::Art => {
			Hold {
				lean: 0.0,
				twist: 0.0,
				scale: 1.0,
				rest: [0.0; 3],
				turn: [0.0; 3],
				wrist: [0.0; 3],
			}
		}
		Archetype::Blade => {
			Hold {
				lean: 0.5,
				twist: 0.0,
				scale: 1.0,
				rest: [0.0, -0.005, 0.0],
				turn: [0.0; 3],
				wrist: [0.0, -0.35, 0.0],
			}
		}
		Archetype::Heavy => {
			Hold {
				lean: 0.95,
				twist: -0.4,
				scale: 0.42,
				rest: [0.01, -0.03, 0.0],
				turn: [0.0, -0.05, 0.0],
				wrist: [0.0; 3],
			}
		}
	}
}

/// A place and a turn (and a size), and what is under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
	pub at: Point,
	pub turn: [f32; 4],
	pub size: f32,
}

impl Frame {
	pub const IDENTITY: Self = Self {
		at: [0.0; 3],
		turn: [0.0, 0.0, 0.0, 1.0],
		size: 1.0,
	};

	pub const fn turned(turn: [f32; 4]) -> Self {
		Self {
			at: [0.0; 3],
			turn,
			size: 1.0,
		}
	}

	pub const fn at(at: Point) -> Self {
		Self {
			at,
			turn: [0.0, 0.0, 0.0, 1.0],
			size: 1.0,
		}
	}

	/// `local`, as seen from outside this frame.
	#[must_use]
	pub fn then(self, local: Self) -> Self {
		Self {
			at: self.point(local.at),
			turn: multiply(self.turn, local.turn),
			size: self.size * local.size,
		}
	}

	pub fn point(self, local: Point) -> Point {
		add(self.at, rotate(self.turn, scale(local, self.size)))
	}

	pub const fn transform(self) -> Transform {
		Transform {
			translation: self.at,
			rotation: self.turn,
			scale: [self.size; 3],
		}
	}
}

/// From the arms' frame (z away from the eye) to the view's (the eye looks
/// down -Z).
const fn view(point: Point) -> Point {
	[point[0], point[1], -point[2]]
}

/// The arm model faces the other way from the eye: its muzzle runs toward
/// +Z.
const FLIP: [f32; 4] = [0.0, 1.0, 0.0, 0.0];

/// Where everything is, in the view, for one pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
	/// `vm_right` and `vm_wrist_r`'s offsets.
	pub arm: Frame,
	pub wrist: [f32; 4],
	/// The weapon's grip, the pistol's muzzle and a heavy weapon's head,
	/// in the view.
	pub grip: Point,
	pub muzzle: Point,
	pub head: Point,
	/// Where the elbow is: what a swing turns about.
	pub elbow: Point,
	/// `vm_left`'s offset, while it shows.
	pub left: Option<Frame>,
}

/// Where the arms are for `pose`, with the weapon of a slot of `archetype`
/// in hand.
pub fn place(archetype: Archetype, pose: &Pose) -> Placed {
	let hold = hold(archetype);
	let arm_pose: &ArmPose = &pose.arm;
	let rest_turn = multiply(
		view_turn(
			REST_TURN[0] + hold.turn[0],
			REST_TURN[1] + hold.turn[1],
			REST_TURN[2] + hold.turn[2],
		),
		FLIP,
	);
	let elbow = rotate(rest_turn, ELBOW);
	let turn = view_turn(arm_pose.yaw, arm_pose.pitch, arm_pose.roll);
	let pivot = add(
		view(add(
			add(REST, hold.rest),
			[arm_pose.x, arm_pose.y, arm_pose.z],
		)),
		elbow,
	);
	let arm = Frame {
		at: sub(pivot, rotate(turn, elbow)),
		turn: multiply(turn, rest_turn),
		size: 1.0,
	};
	// The wrist turns in the view's sense, about the wrist.
	let wrist_view = view_turn(
		REST_WRIST[0] + hold.wrist[0] + arm_pose.wrist_yaw,
		REST_WRIST[1] + hold.wrist[1] + arm_pose.wrist_pitch,
		REST_WRIST[2] + hold.wrist[2] + arm_pose.wrist_roll,
	);
	let wrist = multiply(inverse(arm.turn), multiply(wrist_view, arm.turn));
	let hand = arm.then(Frame::at(WRIST)).then(Frame::turned(wrist));
	let socket = hand.then(Frame::at(scale(WRIST, -1.0)));
	let spun = socket
		.then(Frame::at(TRIGGER))
		.then(Frame::turned(axis_angle([1.0, 0.0, 0.0], arm_pose.spin)));
	let weapon_frame = spun.then(Frame::at(scale(TRIGGER, -1.0))).then(held(hold));
	let left = pose.left.map(|left| {
		let place = view(left.at);
		let to_drum = if left.to_drum > 0.0 && archetype == Archetype::Gun {
			let drum = weapon_frame.point(DRUM);
			let target = add(drum, PALM_TO_DRUM);
			add(place, scale(sub(target, place), left.to_drum))
		} else {
			place
		};
		Frame {
			at: to_drum,
			turn: multiply(view_turn(left.yaw, left.pitch, left.roll), FLIP),
			size: 1.0,
		}
	});
	Placed {
		arm,
		wrist,
		grip: socket.at,
		muzzle: weapon_frame.point(MUZZLE),
		head: weapon_frame.point(SCYTHE_HEAD),
		elbow: pivot,
		left,
	}
}

/// How the weapon sits in its socket.
fn held(hold: Hold) -> Frame {
	Frame {
		at: [0.0; 3],
		turn: multiply(
			axis_angle([1.0, 0.0, 0.0], hold.lean),
			axis_angle([0.0, 1.0, 0.0], hold.twist),
		),
		size: hold.scale,
	}
}

/// The muzzle flash's three frames.
const FLASHES: [&str; 3] = ["vm_flash_0", "vm_flash_1", "vm_flash_2"];

/// Every part the rig shows or hides, and whether it shows for `pose`, with
/// the weapon of a slot of `archetype` in hand. A hidden part hides
/// everything under it.
pub fn visibility(archetype: Archetype, pose: &Pose) -> [(&'static str, bool); 11] {
	let holds = pose.left.map_or(LeftHolds::Nothing, |left| left.holds);
	let held = weapon_part(archetype);
	[
		(PISTOL, held == Some(PISTOL)),
		(KNIFE, held == Some(KNIFE)),
		(SCYTHE, held == Some(SCYTHE)),
		("pistol_drum", pose.drum_drop < DRUM_GONE),
		(FLASHES[0], pose.flash == Some(0)),
		(FLASHES[1], pose.flash == Some(1)),
		(FLASHES[2], pose.flash == Some(2)),
		("vm_left", pose.left.is_some()),
		("viewmodel_soul_cylinder", holds == LeftHolds::Drum),
		("viewmodel_orb_held", holds == LeftHolds::Orb),
		("vm_trail", pose.trail > 0.01),
	]
}

/// The first-person arms in a scene of their own.
pub struct ViewmodelRig {
	puppet: Puppet,
}

impl ViewmodelRig {
	/// Brings the arms into `scene`, with its camera at the eye and the
	/// light they are seen in, over a transparent ground.
	pub fn spawn(scene: &Scene) -> Self {
		scene.push(Op::SetCamera {
			transform: Transform::IDENTITY,
			fov_y: VIEWMODEL_FOV,
			near: 0.02,
			far: 20.0,
		});
		scene.push(Op::SetLight(Light {
			direction: [-0.4, -1.0, -0.6],
			color: [0.85, 0.92, 1.0],
			intensity: 3.2,
			ambient: 0.7,
			shadow: 0.0,
		}));
		scene.push(Op::SetBackdrop(Backdrop::Transparent));
		let mut puppet = Puppet::spawn(scene, VIEWMODEL_MODEL, None);
		// The flash and the trail are light: added to what is behind them,
		// in the soul's colour.
		for part in FLASHES.into_iter().chain(["vm_trail"]) {
			scene.push(Op::SetBlend {
				node: puppet.node,
				part: Some(part.to_owned()),
				blend: Blend::Additive,
			});
			puppet.material(scene, part, glow(1.0));
		}
		Self { puppet }
	}

	/// Shows `pose`, with the weapon of a slot of `archetype` in hand, shaken
	/// by `shake`, eased over the next `seconds`.
	pub fn show(
		&mut self,
		scene: &Scene,
		archetype: Archetype,
		pose: &Pose,
		shake: Shake,
		seconds: f32,
	) {
		let placed = place(archetype, pose);
		let puppet = &mut self.puppet;
		for (part, visible) in visibility(archetype, pose) {
			puppet.visible(scene, part, visible);
		}
		// The shake turns the whole view about the eye, by at most a few
		// tenths of a degree.
		let angle =
			super::juice::pixels_to_angle(super::juice::MAX_SHAKE_PIXELS, 800.0, VIEWMODEL_FOV);
		let shaken = view_turn(shake.x * angle, shake.y * angle, shake.roll * angle);
		puppet.offset(
			scene,
			"viewmodel",
			Frame::turned(shaken).transform(),
			seconds,
		);
		puppet.offset(scene, "vm_right", placed.arm.transform(), seconds);
		puppet.offset(
			scene,
			"vm_wrist_r",
			Frame::turned(placed.wrist).transform(),
			seconds,
		);
		let spin = axis_angle([1.0, 0.0, 0.0], pose.arm.spin);
		puppet.offset(scene, "vm_spin", Frame::turned(spin).transform(), seconds);
		if let Some(weapon) = weapon_part(archetype) {
			puppet.offset(scene, weapon, held(hold(archetype)).transform(), seconds);
		}
		// The drum turns about its own axis and tumbles as it falls.
		let drum = Frame {
			at: [0.0, -pose.drum_drop, 0.0],
			turn: multiply(
				axis_angle([1.0, 0.0, 0.0], pose.drum_drop * 14.0),
				axis_angle([0.0, 0.0, 1.0], pose.drum_spin),
			),
			size: 1.0,
		};
		puppet.offset(scene, "pistol_cylinder", drum.transform(), seconds);
		if let Some(frame) = placed.left {
			puppet.offset(scene, "vm_left", frame.transform(), seconds);
		}
		self.trail(scene, &placed, pose.trail, seconds);
	}

	/// A heavy swing's trail: an arc about the elbow in the plane the head
	/// sweeps, its bright end at the head.
	fn trail(&mut self, scene: &Scene, placed: &Placed, brightness: f32, seconds: f32) {
		let puppet = &mut self.puppet;
		if brightness <= 0.01 {
			return;
		}
		let reach = sub(placed.head, placed.elbow);
		let radius = reach[0].hypot(reach[2]).max(0.05);
		// The arc's head is at 120 degrees round from its tail.
		let head_angle = (-reach[2]).atan2(reach[0]);
		let arc = Frame {
			at: [placed.elbow[0], placed.head[1], placed.elbow[2]],
			turn: axis_angle([0.0, 1.0, 0.0], head_angle - 120_f32.to_radians()),
			size: radius,
		};
		puppet.offset(scene, "vm_trail", arc.transform(), seconds);
		puppet.material(scene, "vm_trail", glow(brightness));
	}

	/// Takes the arms out of `scene`.
	pub fn remove(&self, scene: &Scene) {
		self.puppet.remove(scene);
	}
}

/// Soul light at `brightness`, from 0 to 1.
pub fn glow(brightness: f32) -> MaterialOverride {
	MaterialOverride {
		color: Some([SOUL[0], SOUL[1], SOUL[2], 1.0]),
		opacity: Some(brightness.clamp(0.0, 1.0)),
		..MaterialOverride::default()
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::WeaponKind;

	use super::*;
	use crate::render::arms::ViewAction;
	use crate::render::arms::manner;
	use crate::render::arms::pose;

	/// The slots whose weapon is held, with the scythe in the heavy one.
	const HELD: [Archetype; 3] = [Archetype::Gun, Archetype::Blade, Archetype::Heavy];

	/// `action`'s pose `since` seconds in, for a slot of `archetype`.
	fn posed(archetype: Archetype, action: ViewAction, since: f32) -> Pose {
		pose(manner(archetype, WeaponKind::Scythe), action, since, 0.0)
	}

	/// How high `point` is in the view, from -1 at its bottom edge to 1 at
	/// its top.
	fn height(point: Point) -> f32 {
		point[1] / (-point[2] * (VIEWMODEL_FOV / 2.0).tan())
	}

	#[test]
	fn the_reload_happens_where_it_is_seen() {
		// From the moment the old drum starts to fall until the new one is
		// home and the hand has let go, the pistol and the hand carrying the
		// spare drum are in the frame.
		for step in 25..=78 {
			let share = step as f32 / 100.0;
			let since = share * crate::render::arms::RELOAD_SECONDS;
			let placed = place(
				Archetype::Gun,
				&posed(Archetype::Gun, ViewAction::Reload, since),
			);
			let left = placed.left.expect("the left hand is up");
			for (part, point) in [
				("grip", placed.grip),
				("muzzle", placed.muzzle),
				("left hand", left.at),
			] {
				assert!(
					(-0.95..0.5).contains(&height(point)),
					"at {share} the {part} is at {}",
					height(point)
				);
			}
		}
	}

	#[test]
	fn the_scythe_is_over_the_middle_when_it_lands() {
		// A phone held upright, about 390 by 844 points.
		let aspect = 390.0 / 844.0;
		let landing = crate::render::arms::CONTACT;
		let pose = posed(Archetype::Heavy, ViewAction::Swing, landing);
		let head = place(Archetype::Heavy, &pose).head;
		let across = head[0] / (-head[2] * (VIEWMODEL_FOV / 2.0).tan() * aspect);
		assert!(across.abs() < 0.6, "the head is {across} across");
		assert!(height(head).abs() < 0.6, "the head is {} up", height(head));
		assert!(pose.trail > 0.9, "the trail is {} bright", pose.trail);
	}

	#[test]
	fn a_parry_brings_every_weapon_up_and_in() {
		// On an upright phone, about 390 by 844 points.
		let aspect = 390.0 / 844.0;
		let across = |point: Point| point[0] / (-point[2] * (VIEWMODEL_FOV / 2.0).tan() * aspect);
		for weapon in HELD {
			let rest = place(weapon, &posed(weapon, ViewAction::Idle, 0.0)).grip;
			for since in [0.1, 0.15, 0.2] {
				let held = place(weapon, &posed(weapon, ViewAction::Parry, since)).grip;
				assert!(
					across(held).abs() < 0.5 && height(held) > height(rest),
					"{weapon:?} parries at {:.2}, {:.2}, from {:.2}, {:.2} at rest",
					across(held),
					height(held),
					across(rest),
					height(rest)
				);
			}
		}
	}
}
