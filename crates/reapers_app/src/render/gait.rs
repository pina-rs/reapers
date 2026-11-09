//! How a husk carries itself over the ground: legs that keep step with the
//! ground it covers, the squash of a crouch and a landing, the stretch of a
//! take-off, the tuck and reach of a leap, and where it looks.
//!
//! It reads the rules' [`HuskPose`] and nothing else, and decides nothing
//! about the hunt. [`super::husk::HuskRig`] puts what it works out on the
//! creature's own pivots, as part offsets the host eases between ticks.
//!
//! A stride is measured in ground, not time: the cycle moves on by the
//! distance the body covered over the length of one stride, so a planted
//! foot travels back exactly as fast as the body goes forward, and a body
//! that stops settles onto its feet where it stands.

use std::f64::consts::PI;
use std::f64::consts::TAU;

use reapers_rules::Footing;
use reapers_rules::HuskMotionState;
use reapers_rules::HuskPose;
use reapers_rules::Vec3;
use reapers_rules::Vessel;

/// What a limb is to the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimbKind {
	/// A front leg, or a biped's leg.
	Fore,
	/// A hind leg.
	Hind,
	/// An arm or a flipper, which swings against the legs.
	Arm,
}

/// One limb on its pivot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limb {
	/// The pivot's name in the model.
	pub part: &'static str,
	pub kind: LimbKind,
	/// Where in the stride it is when the stride is at its start: 0, or a
	/// half for the limbs that step between them.
	pub phase: f64,
	/// From the pivot to the sole, in the model's own units.
	pub length: f64,
	/// 1 for a limb on the creature's left (the model's +x), −1 on its
	/// right.
	pub side: f64,
}

/// How a creature is put together, as far as walking goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Build {
	pub limbs: &'static [Limb],
	/// The leg a stride is measured by, pivot to sole, in model units.
	pub leg: f64,
	/// The pivot the head turns on.
	pub head: &'static str,
	/// The pivot a long neck tips on, when it has one: the head then only
	/// turns.
	pub neck: Option<&'static str>,
	/// How far the body rolls from foot to foot, in radians: a penguin
	/// waddles.
	pub waddle: f64,
	/// Whether it stands on two legs, which come up in front of it in a
	/// leap, rather than four, which fold under it.
	pub upright: bool,
}

/// The most limbs any creature has.
pub const MAX_LIMBS: usize = 4;

const fn limb(part: &'static str, kind: LimbKind, phase: f64, length: f64, side: f64) -> Limb {
	Limb {
		part,
		kind,
		phase,
		length,
		side,
	}
}

/// An ape knuckle-walks on a diagonal trot: each forelimb steps with the
/// hind leg across from it.
const APE: Build = Build {
	limbs: &[
		limb("leg_fl", LimbKind::Fore, 0.0, 0.66, 1.0),
		limb("leg_br", LimbKind::Hind, 0.0, 0.42, -1.0),
		limb("leg_fr", LimbKind::Fore, 0.5, 0.66, -1.0),
		limb("leg_bl", LimbKind::Hind, 0.5, 0.42, 1.0),
	],
	leg: 0.42,
	head: "head",
	neck: None,
	waddle: 0.03,
	upright: false,
};

const LLAMA: Build = Build {
	limbs: &[
		limb("leg_fl", LimbKind::Fore, 0.0, 0.52, 1.0),
		limb("leg_br", LimbKind::Hind, 0.0, 0.52, -1.0),
		limb("leg_fr", LimbKind::Fore, 0.5, 0.52, -1.0),
		limb("leg_bl", LimbKind::Hind, 0.5, 0.52, 1.0),
	],
	leg: 0.52,
	head: "head",
	neck: Some("neck"),
	waddle: 0.02,
	upright: false,
};

/// A penguin's legs are stubs: it waddles, rolling onto each foot, and
/// its flippers swing against its feet.
const PENGUIN: Build = Build {
	limbs: &[
		limb("leg_l", LimbKind::Fore, 0.0, 0.15, 1.0),
		limb("leg_r", LimbKind::Fore, 0.5, 0.15, -1.0),
		limb("arm_l", LimbKind::Arm, 0.5, 0.26, 1.0),
		limb("arm_r", LimbKind::Arm, 0.0, 0.26, -1.0),
	],
	leg: 0.15,
	head: "head",
	neck: None,
	waddle: 0.13,
	upright: true,
};

const ROBOT: Build = Build {
	limbs: &[
		limb("leg_l", LimbKind::Fore, 0.0, 0.33, 1.0),
		limb("leg_r", LimbKind::Fore, 0.5, 0.33, -1.0),
		limb("arm_l", LimbKind::Arm, 0.5, 0.4, 1.0),
		limb("arm_r", LimbKind::Arm, 0.0, 0.4, -1.0),
	],
	leg: 0.33,
	head: "head",
	neck: None,
	waddle: 0.04,
	upright: true,
};

/// A bird flies: it has a head to look with and no legs to walk on.
const BIRD: Build = Build {
	limbs: &[],
	leg: 0.2,
	head: "head",
	neck: None,
	waddle: 0.0,
	upright: false,
};

/// How `vessel` is put together, or nothing for a stack of relics.
pub const fn build(vessel: Vessel) -> Option<&'static Build> {
	match vessel {
		Vessel::Ape => Some(&APE),
		Vessel::Llama => Some(&LLAMA),
		Vessel::Penguin => Some(&PENGUIN),
		Vessel::Robot => Some(&ROBOT),
		Vessel::Bird => Some(&BIRD),
		Vessel::ReliquaryStack => None,
	}
}

/// How the body is held at one instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Carriage {
	/// The body's height against its rest: under 1 squashes it, over 1
	/// stretches it. Its width goes the other way, so its volume holds.
	pub squash: f64,
	/// How far the body tips forward, nose down, in radians.
	pub pitch: f64,
	/// How far it rolls onto its left side, in radians.
	pub roll: f64,
	/// How far it rises over its feet, as a share of its half height.
	pub bob: f64,
	/// Each limb of the [`Build`], in its order: its swing about the
	/// model's x (positive takes a hanging foot back) and its spread about
	/// the model's z (positive lifts it away from the body on its side).
	pub limbs: [(f64, f64); MAX_LIMBS],
	/// The head's turn to the creature's right and its tip down, in
	/// radians.
	pub look: (f64, f64),
}

impl Carriage {
	/// Standing still, at rest.
	pub const REST: Self = Self {
		squash: 1.0,
		pitch: 0.0,
		roll: 0.0,
		bob: 0.0,
		limbs: [(0.0, 0.0); MAX_LIMBS],
		look: (0.0, 0.0),
	};
}

impl Default for Carriage {
	fn default() -> Self {
		Self::REST
	}
}

/// How many strides a body takes for every leg length of ground, at its
/// slowest: a stride is this many leg lengths long.
const STRIDE_LEGS: f64 = 1.8;
/// The longest a planted foot reaches either side of the hip, as a share
/// of the leg: about 37°. A faster gait than this allows takes quicker
/// steps rather than longer ones.
const REACH: f64 = 0.6;
/// The quickest a stride cycles, in cycles a second. The rig is told where
/// it is thirty times a second, and a quicker cycle would flicker; a body
/// going faster lets its feet slip rather than blur.
const MOST_CADENCE: f64 = 4.0;
/// Slower than this, in metres a second, a body is standing.
const STILL_SPEED: f64 = 0.04;
/// How quickly a stride fades in and settles out, per second.
const SETTLE_RATE: f64 = 7.0;
/// How quickly the head follows what it looks at, per second.
const GAZE_RATE: f64 = 9.0;
/// The farthest the head turns, and tips up and down, in radians.
const GAZE_TURN: f64 = 1.1;
const GAZE_UP: f64 = 0.45;
const GAZE_DOWN: f64 = 0.85;
/// How far a perched husk looks down past the edge, in radians.
const PERCH_LOOK: f64 = 0.55;
/// A crouch squashes the body this far, and a landing between these two,
/// by how far it fell.
const CROUCH_SQUASH: f64 = 0.12;
const LAND_SQUASH_LEAST: f64 = 0.10;
const LAND_SQUASH_MOST: f64 = 0.15;
/// A fall this long, in metres, lands with the deepest squash.
const DEEP_FALL: f64 = 0.9;
/// A take-off stretches the body this far, easing back over the first
/// share of the flight.
const TAKE_OFF_STRETCH: f64 = 0.08;
const STRETCH_SHARE: f64 = 0.25;
/// The legs reach for the landing over the last share of the flight.
const REACH_SHARE: f64 = 0.3;
/// A move longer than this between two frames is the husk set down
/// somewhere new, not a step.
const TELEPORT: f64 = 0.5;

/// A smooth step from 0 to 1.
fn smooth(t: f64) -> f64 {
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

/// Moves `value` towards `target` at `rate` a second, over `delta`.
fn approach(value: &mut f64, target: f64, rate: f64, delta: f64) {
	*value += (target - *value) * (1.0 - (-rate * delta).exp());
}

/// Where a foot is along the ground in its stride at `phase` (wrapped to
/// 0..1), from 1 (as far forward as it goes) to −1: planted and pushed back
/// at an even pace for the first half, then swung forward again.
pub fn foot_reach(phase: f64) -> f64 {
	let phase = phase.rem_euclid(1.0);
	if phase < 0.5 {
		1.0 - 4.0 * phase
	} else {
		-1.0 + 2.0 * smooth((phase - 0.5) * 2.0)
	}
}

/// One husk's walk, kept from frame to frame.
#[derive(Clone, Debug)]
pub struct Footwork {
	build: Option<&'static Build>,
	/// How far through its stride the body is, in strides.
	phase: f64,
	/// How much the body is walking, from 0 (standing) to 1.
	stride: f64,
	/// Where the body was last frame, and which way it faced.
	last: Option<(Vec3, f64)>,
	/// How fast it rose last frame, in metres a second.
	rising: f64,
	/// The highest the body rose in the leap under way, and how far the
	/// last leap fell.
	apex: f64,
	fall: f64,
	/// The head's turn and tip, as they follow what it looks at.
	look: (f64, f64),
}

impl Footwork {
	/// A body of `vessel`, standing.
	pub fn new(vessel: Vessel) -> Self {
		Self {
			build: build(vessel),
			phase: 0.0,
			stride: 0.0,
			last: None,
			rising: 0.0,
			apex: f64::NEG_INFINITY,
			fall: 0.0,
			look: (0.0, 0.0),
		}
	}

	/// How the creature is built, if it walks at all.
	pub const fn build(&self) -> Option<&'static Build> {
		self.build
	}

	/// How far through its stride the body is, in strides.
	pub const fn phase(&self) -> f64 {
		self.phase
	}

	/// How long one stride is, in metres, for a body at `pose` going at
	/// `speed` metres a second.
	pub fn stride_length(&self, pose: &HuskPose, speed: f64) -> f64 {
		let leg = self.build.map_or(0.0, |build| build.leg) * pose.scale;
		(STRIDE_LEGS * leg).max(speed / MOST_CADENCE)
	}

	/// Moves on by `delta` seconds with the husk at `pose`, and says how
	/// the body is held.
	pub fn step(&mut self, pose: &HuskPose, delta: f64) -> Carriage {
		let Some(build) = self.build else {
			return Carriage::REST;
		};
		let delta = delta.max(0.0);
		let (covered, rising) = self.travel(pose, delta);
		let speed = if delta > 0.0 { covered / delta } else { 0.0 };
		let state = pose.locomotion.map(|locomotion| locomotion.state);
		let airborne =
			matches!(state, Some(HuskMotionState::Air)) || pose.leaping && state.is_none();
		let stride_length = self.stride_length(pose, speed);
		if stride_length > 0.0 && !airborne {
			self.phase = (self.phase + covered / stride_length).rem_euclid(1.0);
		}
		approach(
			&mut self.stride,
			f64::from(u8::from(speed > STILL_SPEED && !airborne)),
			SETTLE_RATE,
			delta,
		);
		self.rising = rising;
		let mut carriage = self.walk(build, pose, stride_length);
		if let Some(locomotion) = pose.locomotion {
			self.act(
				build,
				pose,
				locomotion.state,
				locomotion.state_fraction,
				&mut carriage,
			);
		} else if pose.leaping {
			tuck(build, 1.0, 0.0, &mut carriage);
		}
		let target = Self::gaze(pose);
		approach(&mut self.look.0, target.0, GAZE_RATE, delta);
		approach(&mut self.look.1, target.1, GAZE_RATE, delta);
		carriage.look = self.look;
		carriage
	}

	/// How far the body covered over the ground since last frame, counting
	/// a turn on the spot as the steps it takes, and how fast it rose.
	fn travel(&mut self, pose: &HuskPose, delta: f64) -> (f64, f64) {
		let now = (pose.position, pose.heading);
		let Some((was, heading)) = self.last.replace(now) else {
			return (0.0, 0.0);
		};
		let moved = pose.position - was;
		let flat = moved.x.hypot(moved.z);
		if flat > TELEPORT {
			return (0.0, 0.0);
		}
		let turned = reapers_rules::wrap_angle(pose.heading - heading).abs() * pose.half_width();
		let rising = if delta > 0.0 { moved.y / delta } else { 0.0 };
		(flat + 0.5 * turned, rising)
	}

	/// The legs in step with the ground, the waddle and the bob of each
	/// step, faded in as the body walks.
	fn walk(&self, build: &Build, pose: &HuskPose, stride_length: f64) -> Carriage {
		let mut carriage = Carriage::REST;
		let stride = self.stride;
		if stride <= 1e-3 {
			return carriage;
		}
		for (slot, limb) in build.limbs.iter().take(MAX_LIMBS).enumerate() {
			let phase = self.phase + limb.phase;
			carriage.limbs[slot].0 = match limb.kind {
				LimbKind::Fore | LimbKind::Hind => {
					// A planted foot goes back half a stride while the body
					// goes forward half a stride.
					let leg = limb.length * pose.scale;
					let reach = (0.25 * stride_length).min(REACH * leg);
					let along = foot_reach(phase) * reach;
					-(along / leg).clamp(-1.0, 1.0).asin() * stride
				}
				LimbKind::Arm => -0.35 * (TAU * phase).cos() * stride,
			};
		}
		carriage.roll = build.waddle * (TAU * self.phase).sin() * stride;
		carriage.bob = 0.04 * (TAU * self.phase * 2.0).cos().mul_add(-0.5, 0.5) * stride;
		carriage
	}

	/// What each state of a walk on the grid adds: the lean at an edge, the
	/// gathering crouch, the stretch, tuck and reach of a leap, and the
	/// squash of a landing.
	fn act(
		&mut self,
		build: &Build,
		pose: &HuskPose,
		state: HuskMotionState,
		fraction: f64,
		carriage: &mut Carriage,
	) {
		let fraction = fraction.clamp(0.0, 1.0);
		if state == HuskMotionState::Air {
			self.apex = self.apex.max(pose.position.y);
		} else if self.apex.is_finite() {
			// Down: how far it fell decides how hard it lands.
			self.fall = (self.apex - pose.position.y).max(0.0);
			self.apex = f64::NEG_INFINITY;
		}
		match state {
			HuskMotionState::Edge => {
				// It slows to the brink and leans to look over.
				carriage.pitch += 0.12 * smooth(fraction);
			}
			HuskMotionState::Crouch => {
				let gather = smooth(fraction);
				carriage.squash *= 1.0 - CROUCH_SQUASH * gather;
				carriage.pitch += 0.1 * gather;
				brace(build, 0.25 * gather, carriage);
			}
			HuskMotionState::Air => {
				let stretch = 1.0 - smooth(fraction / STRETCH_SHARE);
				carriage.squash *= 1.0 + TAKE_OFF_STRETCH * stretch;
				// The nose follows the arc: up as it rises, down as it falls.
				carriage.pitch += (-self.rising / 3.0).atan().clamp(-0.4, 0.4);
				let folded = (PI * (fraction / (1.0 - REACH_SHARE)).min(1.0)).sin();
				let reaching = smooth((fraction - (1.0 - REACH_SHARE)) / REACH_SHARE);
				tuck(build, folded, reaching, carriage);
			}
			HuskMotionState::Land => {
				let depth = landing_squash(self.fall);
				let impact = if fraction < 0.25 {
					smooth(fraction / 0.25)
				} else {
					1.0 - smooth((fraction - 0.25) / 0.75)
				};
				carriage.squash *= 1.0 - depth * impact;
				brace(build, 0.3 * impact, carriage);
			}
			HuskMotionState::Recover => {
				let depth = landing_squash(self.fall);
				carriage.squash *= 1.0 - 0.3 * depth * (1.0 - smooth(fraction));
			}
			HuskMotionState::Idle
			| HuskMotionState::Turn
			| HuskMotionState::Walk
			| HuskMotionState::Run => {}
		}
	}

	/// Where the head would look, as its turn and tip: at where the feet
	/// come down next, and down over the edge of a perch.
	fn gaze(pose: &HuskPose) -> (f64, f64) {
		let target = pose
			.locomotion
			.and_then(|locomotion| locomotion.next_foothold);
		let Some(target) = target else {
			let perched = pose.footing == Footing::Ledge
				&& pose
					.locomotion
					.is_some_and(|locomotion| locomotion.state == HuskMotionState::Idle);
			return (0.0, if perched { PERCH_LOOK } else { 0.0 });
		};
		let eye = pose.position + Vec3::new(0.0, 0.6 * pose.half_height(), 0.0);
		let way = target - eye;
		let (sin, cos) = pose.heading.sin_cos();
		let ahead = way.x * sin + way.z * cos;
		let right = way.x * cos - way.z * sin;
		let flat = ahead.hypot(right);
		if flat < 0.5 * pose.half_width() {
			// Underfoot: look down at it, not round at it.
			return (0.0, GAZE_DOWN);
		}
		(
			right.atan2(ahead).clamp(-GAZE_TURN, GAZE_TURN),
			(-way.y).atan2(flat).clamp(-GAZE_UP, GAZE_DOWN),
		)
	}
}

/// How deep a landing from `fall` metres squashes the body.
fn landing_squash(fall: f64) -> f64 {
	LAND_SQUASH_LEAST + (LAND_SQUASH_MOST - LAND_SQUASH_LEAST) * (fall / DEEP_FALL).clamp(0.0, 1.0)
}

/// Legs set wide to push off or take a landing: fore legs forward, hind
/// legs back, arms out.
fn brace(build: &Build, amount: f64, carriage: &mut Carriage) {
	for (slot, limb) in build.limbs.iter().take(MAX_LIMBS).enumerate() {
		match limb.kind {
			LimbKind::Fore => carriage.limbs[slot].0 -= amount,
			LimbKind::Hind => carriage.limbs[slot].0 += amount,
			LimbKind::Arm => carriage.limbs[slot].1 += limb.side * amount * 1.5,
		}
	}
}

/// The legs in the air: folded under the body by `folded`, then reaching
/// for the landing by `reaching`; arms thrown out for balance.
fn tuck(build: &Build, folded: f64, reaching: f64, carriage: &mut Carriage) {
	let held = 1.0 - reaching;
	for (slot, limb) in build.limbs.iter().take(MAX_LIMBS).enumerate() {
		let (swing, spread) = match limb.kind {
			// A biped's legs come up in front of it, then straighten.
			LimbKind::Fore if build.upright => (-0.8 * folded * held - 0.2 * reaching, 0.0),
			LimbKind::Fore => (0.7 * folded * held - 0.6 * reaching, 0.0),
			LimbKind::Hind => (-0.7 * folded * held + 0.3 * reaching, 0.0),
			LimbKind::Arm => {
				(
					-0.3 * folded,
					limb.side * (0.9 * folded * held + 0.5 * reaching),
				)
			}
		};
		carriage.limbs[slot] = (swing, spread);
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::HuskLocomotion;

	use super::*;

	const FRAME: f64 = 1.0 / 30.0;

	/// A penguin of the room's size at `position`, facing straight ahead,
	/// doing `state`.
	fn penguin(position: Vec3, state: HuskMotionState, fraction: f64) -> HuskPose {
		HuskPose {
			scale: 0.53,
			ground: Some(-1.45),
			locomotion: Some(HuskLocomotion {
				state,
				state_fraction: fraction,
				support_height: -1.45,
				next_foothold: None,
			}),
			..HuskPose::new(Vessel::Penguin, position, 0.0)
		}
	}

	/// Walks a penguin straight ahead at `speed` for `seconds`, from `z`,
	/// handing each frame's carriage and the distance walked to `frame`.
	fn walk(
		footwork: &mut Footwork,
		speed: f64,
		seconds: f64,
		z: &mut f64,
		mut frame: impl FnMut(&Footwork, &Carriage, f64),
	) {
		let frames = (seconds / FRAME).round() as usize;
		for _ in 0..frames {
			*z += speed * FRAME;
			let pose = penguin(Vec3::new(0.0, -1.2, *z), HuskMotionState::Walk, 0.0);
			let carriage = footwork.step(&pose, FRAME);
			frame(footwork, &carriage, *z);
		}
	}

	#[test]
	fn a_stride_pushes_back_evenly_and_swings_forward() {
		assert!((foot_reach(0.0) - 1.0).abs() < 1e-12);
		assert!(foot_reach(0.25).abs() < 1e-12);
		assert!((foot_reach(0.5) + 1.0).abs() < 1e-12);
		assert!((foot_reach(0.75)).abs() < 1e-12);
		assert!((foot_reach(1.0) - 1.0).abs() < 1e-12);
	}

	#[test]
	fn a_planted_foot_does_not_skate() {
		let mut footwork = Footwork::new(Vessel::Penguin);
		let mut z = 2.0;
		let speed = Vessel::Penguin.walk_speed();
		// Long enough for the walk to have faded all the way in.
		walk(&mut footwork, speed, 3.0, &mut z, |_, _, _| {});
		let leg = PENGUIN.limbs[0].length * 0.53;
		let mut planted: Vec<f64> = Vec::new();
		let mut slips = Vec::new();
		walk(
			&mut footwork,
			speed,
			2.0,
			&mut z,
			|footwork, carriage, z| {
				let phase = (footwork.phase() + PENGUIN.limbs[0].phase).rem_euclid(1.0);
				// Where the foot is along the ground: the hip's place, and how
				// far forward of it the swing puts the foot.
				let foot = z - leg * carriage.limbs[0].0.sin();
				if (0.03..0.47).contains(&phase) {
					planted.push(foot);
				} else if !planted.is_empty() {
					let (low, high) = planted
						.iter()
						.fold((f64::MAX, f64::MIN), |(low, high), &at| {
							(low.min(at), high.max(at))
						});
					slips.push(high - low);
					planted.clear();
				}
			},
		);
		assert!(!slips.is_empty(), "the penguin took steps");
		for slip in slips {
			assert!(slip < 0.004, "a planted foot slid {slip} m");
		}
	}

	#[test]
	fn a_body_that_stops_settles_onto_its_feet() {
		let mut footwork = Footwork::new(Vessel::Ape);
		let mut z = 2.0;
		walk(&mut footwork, 0.7, 2.0, &mut z, |_, _, _| {});
		let mut last = Carriage::REST;
		walk(&mut footwork, 0.0, 1.5, &mut z, |_, carriage, _| {
			last = *carriage;
		});
		for (swing, spread) in last.limbs {
			assert!(
				swing.abs() < 0.01 && spread.abs() < 0.01,
				"{swing} {spread}"
			);
		}
		assert!(last.roll.abs() < 0.01);
	}

	#[test]
	fn a_quick_body_takes_quicker_steps_not_impossible_ones() {
		let footwork = Footwork::new(Vessel::Penguin);
		let pose = penguin(Vec3::new(0.0, -1.2, 2.0), HuskMotionState::Run, 0.0);
		let bolt = Vessel::Penguin.bolt_speed();
		let stride = footwork.stride_length(&pose, bolt);
		assert!(bolt / stride <= MOST_CADENCE + 1e-9);
		assert!(footwork.stride_length(&pose, 0.1) > 0.0);
	}

	/// The carriage of a penguin `fraction` through `state`, after a leap
	/// that fell `fall` metres.
	fn holding(state: HuskMotionState, fraction: f64, fall: f64) -> Carriage {
		let mut footwork = Footwork::new(Vessel::Penguin);
		// Up in the air, then down, so it knows how far it fell.
		footwork.step(
			&penguin(Vec3::new(0.0, -1.0 + fall, 2.0), HuskMotionState::Air, 0.5),
			FRAME,
		);
		footwork.step(
			&penguin(Vec3::new(0.0, -1.0, 2.0), HuskMotionState::Land, 0.0),
			FRAME,
		);
		footwork.step(&penguin(Vec3::new(0.0, -1.0, 2.0), state, fraction), FRAME)
	}

	#[test]
	fn a_crouch_and_a_landing_squash_and_a_take_off_stretches() {
		let crouched = holding(HuskMotionState::Crouch, 1.0, 0.0).squash;
		assert!((0.85..=0.9).contains(&crouched), "{crouched}");
		let soft = holding(HuskMotionState::Land, 0.25, 0.1).squash;
		let hard = holding(HuskMotionState::Land, 0.25, 1.2).squash;
		assert!((0.85..=0.9).contains(&soft) && (0.85..=0.9).contains(&hard));
		assert!(hard < soft, "a longer fall lands harder");
		let off = holding(HuskMotionState::Air, 0.0, 0.0).squash;
		assert!((1.05..=1.1).contains(&off), "{off}");
		let apex = holding(HuskMotionState::Air, 0.5, 0.0).squash;
		assert!((apex - 1.0).abs() < 1e-9);
		let stood = holding(HuskMotionState::Recover, 1.0, 1.2).squash;
		assert!((stood - 1.0).abs() < 1e-9);
	}

	#[test]
	fn a_leap_tucks_at_its_height_and_reaches_for_the_landing() {
		let mut footwork = Footwork::new(Vessel::Ape);
		let ape = |fraction| {
			HuskPose {
				locomotion: Some(HuskLocomotion {
					state: HuskMotionState::Air,
					state_fraction: fraction,
					support_height: -1.45,
					next_foothold: None,
				}),
				..HuskPose::new(Vessel::Ape, Vec3::new(0.0, -0.8, 2.5), 0.0)
			}
		};
		let tucked = footwork.step(&ape(0.35), FRAME);
		let reaching = footwork.step(&ape(0.98), FRAME);
		// The fore legs fold back under the body, then reach forward.
		assert!(tucked.limbs[0].0 > 0.4, "{:?}", tucked.limbs);
		assert!(reaching.limbs[0].0 < -0.4, "{:?}", reaching.limbs);
		// The hind legs fold forward under it.
		assert!(tucked.limbs[1].0 < -0.4, "{:?}", tucked.limbs);
	}

	#[test]
	fn the_head_looks_where_the_feet_go_next_and_down_from_a_perch() {
		let look = |foothold: Option<Vec3>, footing: Footing| {
			let mut footwork = Footwork::new(Vessel::Ape);
			let pose = HuskPose {
				footing,
				locomotion: Some(HuskLocomotion {
					state: HuskMotionState::Idle,
					state_fraction: 0.0,
					support_height: -0.6,
					next_foothold: foothold,
				}),
				..HuskPose::new(Vessel::Ape, Vec3::new(0.0, -0.3, 2.5), 0.0)
			};
			let mut carriage = Carriage::REST;
			for _ in 0..60 {
				carriage = footwork.step(&pose, FRAME);
			}
			carriage.look
		};
		// Ahead of it, down and to its right: the floor beyond a desk.
		let (right, down) = look(Some(Vec3::new(0.8, -1.4, 3.3)), Footing::Ledge);
		assert!(right > 0.3 && down > 0.3, "{right} {down}");
		let (left, _) = look(Some(Vec3::new(-0.8, -0.3, 3.3)), Footing::Open);
		assert!(left < -0.3, "{left}");
		let (_, perched) = look(None, Footing::Ledge);
		assert!(perched > 0.4, "{perched}");
		assert_eq!(look(None, Footing::Open), (0.0, 0.0));
	}
}
