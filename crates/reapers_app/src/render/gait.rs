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

use super::spring::Spring;
use super::spring::inertia;

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
/// The quickest a stride cycles, in cycles a second. A slow display tells
/// the rig where it is only thirty times a second, and a quicker cycle
/// would flicker there; a body going faster lets its feet slip rather than
/// blur.
const MOST_CADENCE: f64 = 4.0;
/// Slower than this, in metres a second, a body is standing.
const STILL_SPEED: f64 = 0.04;
/// How quickly a stride fades in and settles out, per second.
const SETTLE_RATE: f64 = 7.0;
/// How quickly the head follows what it looks at: the frequency of a
/// critically damped spring, so it turns, slows and settles on a thing with
/// no snap at the start of a turn and no wobble at its end.
const GAZE_FREQUENCY: f64 = 2.2;
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
/// Left where it is for longer than this, in seconds, by rules that move it
/// sixty times a second, a body is standing.
const STILL_AFTER: f64 = 0.05;
/// How the body springs back from a leap the rules give no warning of: the
/// spring's frequency, and the kick of a take-off (a stretch) and of the
/// deepest landing (a squash). Each peaks at about a sixth of a second at
/// [`TAKE_OFF_STRETCH`] and [`LAND_SQUASH_MOST`].
const BOUNCE_FREQUENCY: f64 = 3.0;
/// How long the body blends out of how it was held when its state changes
/// (a crouch into a leap, a leap into a landing), in seconds.
const CHANGE_BLEND: f64 = 0.12;
/// How many numbers a carriage holds besides its gaze, which follows on a
/// spring of its own.
const CHANNELS: usize = 4 + 2 * MAX_LIMBS;

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
///
/// The swing leaves the ground at the pace the foot was pushed back at and
/// comes down at the pace it will be pushed back at, so the foot never stops
/// dead: it follows through a little behind where it lifts, sweeps forward,
/// and reaches a little past where it plants before it settles back onto
/// it. A Hermite curve from −1 to 1 whose slopes at both ends match the
/// planted foot's.
///
/// For a player who asked for less motion (`lively` false) the swing does
/// not follow through: it eases from where the foot lifts to where it
/// plants and no further.
pub fn foot_reach(phase: f64, lively: bool) -> f64 {
	let phase = phase.rem_euclid(1.0);
	if phase < 0.5 {
		1.0 - 4.0 * phase
	} else if lively {
		let swung = (phase - 0.5) * 2.0;
		((-8.0 * swung + 12.0) * swung - 2.0) * swung - 1.0
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
	/// Where the rules last put the body, and which way it faced.
	last: Option<(Vec3, f64)>,
	/// How long the rules have left it where it is, in seconds: a display
	/// drawing faster than they move it sees it still between their ticks.
	unmoved: f64,
	/// How fast the body went and rose the last time the rules moved it, in
	/// metres a second.
	speed: f64,
	rising: f64,
	/// The highest the body rose in the leap under way, and how far the
	/// last leap fell.
	apex: f64,
	fall: f64,
	/// The head's turn and tip, as they follow what it looks at.
	look: [Spring; 2],
	/// The state of its walk on the grid, and whether it is in a leap: a
	/// change of either can change how it is held at once.
	act: Option<(Option<HuskMotionState>, bool)>,
	/// The carriage last shown, and how fast each of its numbers moved.
	shown: Option<([f64; CHANNELS], [f64; CHANNELS])>,
	/// What a change left to blend out of: how far the carriage before it
	/// was from the new one, how fast that was changing, and how long ago
	/// the change came, in seconds.
	change: Option<([f64; CHANNELS], [f64; CHANNELS], f64)>,
	/// The stretch of a take-off and the squash of a landing for a leap
	/// off the room's grid, as a share of its height, on a spring.
	bounce: Spring,
	/// Whether what is only for liveliness moves: off for a player who
	/// asked for less motion.
	lively: bool,
	/// Whether a leap off the grid may spring the body about its feet: off
	/// while its seals must stay where the rules put them.
	bouncy: bool,
}

impl Footwork {
	/// A body of `vessel`, standing.
	pub fn new(vessel: Vessel) -> Self {
		Self {
			build: build(vessel),
			phase: 0.0,
			stride: 0.0,
			last: None,
			unmoved: 0.0,
			speed: 0.0,
			rising: 0.0,
			apex: f64::NEG_INFINITY,
			fall: 0.0,
			look: [Spring::new(GAZE_FREQUENCY, 0.0); 2],
			act: None,
			shown: None,
			change: None,
			bounce: Spring::new(BOUNCE_FREQUENCY, 0.0),
			lively: true,
			bouncy: true,
		}
	}

	/// Lets what is only for liveliness move, or holds it still for a
	/// player who asked for less motion.
	pub fn set_lively(&mut self, lively: bool) {
		self.lively = lively;
	}

	/// Lets a leap off the room's grid stretch and squash the body, or
	/// holds it at the size the rules give it: a husk whose ward holds
	/// carries its seals where the rules put them, and a squash about its
	/// feet would carry them off.
	pub fn set_bouncy(&mut self, bouncy: bool) {
		self.bouncy = bouncy;
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
		let covered = self.travel(pose, delta);
		let speed = self.speed;
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
		let mut carriage = self.walk(build, pose, stride_length);
		if let Some(locomotion) = pose.locomotion {
			self.act(
				build,
				pose,
				locomotion.state,
				locomotion.state_fraction,
				&mut carriage,
			);
		} else {
			if pose.leaping {
				tuck(build, 1.0, 0.0, &mut carriage);
			}
			self.spring_off(pose, delta, &mut carriage);
		}
		self.blend(state, pose.leaping, delta, &mut carriage);
		let target = Self::gaze(pose);
		carriage.look = (
			self.look[0].step(target.0, delta),
			self.look[1].step(target.1, delta),
		);
		carriage
	}

	/// Off the room's grid the rules leap a husk with no crouch before it:
	/// the body stretches as it takes off and squashes as it lands, by how
	/// far it fell, on a spring around where the rules hold it.
	fn spring_off(&mut self, pose: &HuskPose, delta: f64, carriage: &mut Carriage) {
		let was_leaping = self.act.is_some_and(|(_, leaping)| leaping);
		if pose.leaping {
			self.apex = self.apex.max(pose.position.y);
		}
		if !self.lively || !self.bouncy {
			self.bounce.settle(0.0);
			if !pose.leaping {
				self.apex = f64::NEG_INFINITY;
			}
			return;
		}
		carriage.squash *= 1.0 + self.bounce.step(0.0, delta);
		// The take-off or the landing is seen as this frame ends: its kick
		// starts from here, whatever the frame rate. A kick of speed v rises
		// to v/(ωe) on a critically damped spring.
		let peak = |share: f64| share * TAU * BOUNCE_FREQUENCY * std::f64::consts::E;
		if self.act.is_some() {
			if pose.leaping && !was_leaping {
				self.bounce.velocity += peak(TAKE_OFF_STRETCH);
			} else if !pose.leaping && was_leaping && self.apex.is_finite() {
				self.fall = (self.apex - pose.position.y).max(0.0);
				self.bounce.velocity -= peak(landing_squash(self.fall));
			}
		}
		if !pose.leaping {
			self.apex = f64::NEG_INFINITY;
		}
	}

	/// Carries `carriage` on from the one shown before when the body's state
	/// changed, so a crouch, a take-off or a landing that changes how it is
	/// held at once is not a cut: the difference, and the speed it was
	/// changing at, die away over [`CHANGE_BLEND`].
	fn blend(
		&mut self,
		state: Option<HuskMotionState>,
		leaping: bool,
		delta: f64,
		carriage: &mut Carriage,
	) {
		let act = (state, leaping);
		let raw = channels(carriage);
		if self.act.is_some_and(|before| before != act)
			&& let Some((shown, speed)) = self.shown
		{
			let offset = std::array::from_fn(|slot| shown[slot] - raw[slot]);
			self.change = Some((offset, speed, 0.0));
		}
		self.act = Some(act);
		let mut blended = raw;
		if let Some((offset, speed, since)) = &mut self.change {
			*since += delta;
			if *since >= CHANGE_BLEND {
				self.change = None;
			} else {
				for (slot, value) in blended.iter_mut().enumerate() {
					*value += f64::from(inertia(
						offset[slot] as f32,
						speed[slot] as f32,
						CHANGE_BLEND as f32,
						*since as f32,
					));
				}
			}
		}
		let speed = match self.shown {
			Some((shown, _)) if delta > 0.0 => {
				std::array::from_fn(|slot| (blended[slot] - shown[slot]) / delta)
			}
			Some((_, speed)) => speed,
			None => [0.0; CHANNELS],
		};
		self.shown = Some((blended, speed));
		set_channels(carriage, blended);
	}

	/// How far the body covered over the ground since last frame, counting
	/// a turn on the spot as the steps it takes, and how fast it went and
	/// rose over the time the rules took to move it. Between the rules' ticks
	/// the body is not standing still, only not yet moved again: its speed is
	/// held until it has been left where it is for longer than a tick or two.
	fn travel(&mut self, pose: &HuskPose, delta: f64) -> f64 {
		let now = (pose.position, pose.heading);
		let Some((was, heading)) = self.last.replace(now) else {
			return 0.0;
		};
		if now == (was, heading) {
			self.unmoved += delta;
			if self.unmoved > STILL_AFTER {
				self.speed = 0.0;
				self.rising = 0.0;
			}
			return 0.0;
		}
		let span = self.unmoved + delta;
		self.unmoved = 0.0;
		let moved = pose.position - was;
		let flat = moved.x.hypot(moved.z);
		if flat > TELEPORT {
			self.speed = 0.0;
			self.rising = 0.0;
			return 0.0;
		}
		let turned = reapers_rules::wrap_angle(pose.heading - heading).abs() * pose.half_width();
		let covered = flat + 0.5 * turned;
		if span > 0.0 {
			self.speed = covered / span;
			self.rising = moved.y / span;
		}
		covered
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
					let along = foot_reach(phase, self.lively) * reach;
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

/// A carriage's numbers besides its gaze, in one order.
fn channels(carriage: &Carriage) -> [f64; CHANNELS] {
	let mut all = [0.0; CHANNELS];
	all[..4].copy_from_slice(&[carriage.squash, carriage.pitch, carriage.roll, carriage.bob]);
	for (slot, (swing, spread)) in carriage.limbs.iter().enumerate() {
		all[4 + 2 * slot] = *swing;
		all[5 + 2 * slot] = *spread;
	}
	all
}

fn set_channels(carriage: &mut Carriage, all: [f64; CHANNELS]) {
	[carriage.squash, carriage.pitch, carriage.roll, carriage.bob] =
		[all[0], all[1], all[2], all[3]];
	for (slot, limb) in carriage.limbs.iter_mut().enumerate() {
		*limb = (all[4 + 2 * slot], all[5 + 2 * slot]);
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
		for lively in [true, false] {
			assert!((foot_reach(0.0, lively) - 1.0).abs() < 1e-12);
			assert!(foot_reach(0.25, lively).abs() < 1e-12);
			assert!((foot_reach(0.5, lively) + 1.0).abs() < 1e-12);
			assert!((foot_reach(0.75, lively)).abs() < 1e-12);
			assert!((foot_reach(1.0, lively) - 1.0).abs() < 1e-12);
		}
	}

	#[test]
	fn a_foot_swings_on_at_the_pace_it_was_planted() {
		let step = 1e-6;
		let pace = |phase: f64| {
			(foot_reach(phase + step, true) - foot_reach(phase - step, true)) / (2.0 * step)
		};
		// No stop where it lifts or where it plants: the planted pace, −4
		// strides' reach a stride, carries through both.
		for at in [0.5, 1.0] {
			let before = (foot_reach(at - step, true) - foot_reach(at - 2.0 * step, true)) / step;
			let after = (foot_reach(at + 2.0 * step, true) - foot_reach(at + step, true)) / step;
			assert!(
				(before - after).abs() < 1e-3,
				"at {at}: {before} then {after}"
			);
			assert!((before + 4.0).abs() < 1e-3);
		}
		// It follows through a little past each end, and no further.
		let swing: Vec<f64> = (0..=1000)
			.map(|n| foot_reach(0.5 + 0.0005 * f64::from(n), true))
			.collect();
		let lowest = swing.iter().copied().fold(f64::INFINITY, f64::min);
		let highest = swing.iter().copied().fold(f64::NEG_INFINITY, f64::max);
		assert!(lowest < -1.05 && lowest > -1.1, "{lowest}");
		assert!(highest > 1.05 && highest < 1.1, "{highest}");
		assert!(pace(0.75) > 4.0, "it sweeps forward fast in the middle");
	}

	#[test]
	fn for_less_motion_a_foot_swings_no_further_than_it_plants() {
		let swing: Vec<f64> = (0..=1000)
			.map(|n| foot_reach(0.5 + 0.0005 * f64::from(n), false))
			.collect();
		assert!(swing.iter().all(|reach| (-1.0..=1.0).contains(reach)));
		// And a body walked for a player who asked for less motion swings
		// its legs on that stride, not the one that follows through.
		let walked = |lively: bool| {
			let mut footwork = Footwork::new(Vessel::Penguin);
			footwork.set_lively(lively);
			let mut z = 2.0;
			let mut swings = Vec::new();
			walk(
				&mut footwork,
				Vessel::Penguin.walk_speed(),
				2.0,
				&mut z,
				|_, carriage, _| swings.push(carriage.limbs[0].0),
			);
			swings
		};
		assert_ne!(walked(true), walked(false));
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

	/// The carriage of a penguin held `fraction` through `state`, once it has
	/// blended into it, after a leap that fell `fall` metres.
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
		let held = penguin(Vec3::new(0.0, -1.0, 2.0), state, fraction);
		for _ in 0..10 {
			footwork.step(&held, FRAME);
		}
		footwork.step(&held, FRAME)
	}

	#[test]
	fn a_leap_off_the_grid_stretches_on_take_off_and_squashes_on_landing() {
		let off_grid = |height: f64, leaping: bool| {
			HuskPose {
				leaping,
				..HuskPose::new(Vessel::Penguin, Vec3::new(0.0, height, 2.0), 0.0)
			}
		};
		let leap = |lively: bool, rate: f64| {
			let mut footwork = Footwork::new(Vessel::Penguin);
			footwork.set_lively(lively);
			let mut squashes = Vec::new();
			let frames = (1.5 * rate) as usize;
			for frame in 0..frames {
				let t = frame as f64 / rate;
				// Up for half a second, rising half a metre and falling a
				// metre, then down.
				let leaping = (0.2..0.7).contains(&t);
				let height = if leaping {
					let share = (t - 0.2) / 0.5;
					-1.0 + 2.0 * share * (1.0 - share) - 0.5 * share
				} else if t < 0.2 {
					-1.0
				} else {
					-1.5
				};
				squashes.push(footwork.step(&off_grid(height, leaping), 1.0 / rate).squash);
			}
			squashes
		};
		let sixty = leap(true, 60.0);
		let most = sixty.iter().copied().fold(0.0_f64, f64::max);
		let least = sixty.iter().copied().fold(2.0_f64, f64::min);
		assert!(most > 1.0 + 0.6 * TAKE_OFF_STRETCH, "it stretches: {most}");
		assert!(
			least < 1.0 - 0.6 * LAND_SQUASH_LEAST,
			"it squashes: {least}"
		);
		assert!((sixty.last().unwrap() - 1.0).abs() < 1e-3, "and settles");
		// The same at thirty frames a second, but for the frame a take-off
		// or a landing is first seen in, which a slower display sees later.
		let thirty = leap(true, 30.0);
		for (frame, squash) in thirty.iter().enumerate() {
			let t = frame as f64 / 30.0;
			if [0.2, 0.7].iter().any(|at| (t - at).abs() < 0.04) {
				continue;
			}
			assert!(
				(squash - sixty[2 * frame]).abs() < 0.012,
				"{frame}: {squash} {}",
				sixty[2 * frame]
			);
		}
		let fewest = thirty.iter().copied().fold(2.0_f64, f64::min);
		assert!((fewest - least).abs() < 0.015, "{fewest} {least}");
		// Held still for a player who asked for less motion.
		assert!(
			leap(false, 60.0)
				.iter()
				.all(|squash| (*squash - 1.0).abs() < 1e-12)
		);
	}

	#[test]
	fn a_change_of_state_carries_on_how_the_body_was_held() {
		// Reaching for the ground at the end of a leap, then landing: the
		// legs are not snapped from their reach to their brace.
		let mut footwork = Footwork::new(Vessel::Penguin);
		let at = Vec3::new(0.0, -1.0, 2.0);
		let mut reaching = Carriage::REST;
		for fraction in [0.9, 0.95, 0.99] {
			reaching = footwork.step(&penguin(at, HuskMotionState::Air, fraction), FRAME);
		}
		let landed = footwork.step(&penguin(at, HuskMotionState::Land, 0.0), FRAME);
		let mut fresh = Footwork::new(Vessel::Penguin);
		let raw = fresh.step(&penguin(at, HuskMotionState::Land, 0.0), FRAME);
		let jump = (raw.limbs[0].0 - reaching.limbs[0].0).abs();
		assert!(jump > 0.15, "a cut to blend: {jump}");
		// A frame on, it has gone on the way it was going, not jumped.
		let moved = (landed.limbs[0].0 - reaching.limbs[0].0).abs();
		assert!(moved < 0.5 * jump, "{moved} of {jump}");
		// Once the blend is over, the landing is held as it would be anyway.
		let mut later = landed;
		for _ in 0..4 {
			later = footwork.step(&penguin(at, HuskMotionState::Land, 0.0), FRAME);
		}
		for _ in 0..4 {
			fresh.step(&penguin(at, HuskMotionState::Land, 0.0), FRAME);
		}
		let settled = fresh.step(&penguin(at, HuskMotionState::Land, 0.0), FRAME);
		assert!((later.limbs[0].0 - settled.limbs[0].0).abs() < 1e-9);
		assert!((later.squash - settled.squash).abs() < 1e-9);
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

	#[test]
	fn the_head_eases_into_a_turn_and_settles_without_a_wobble() {
		let pose = HuskPose {
			locomotion: Some(HuskLocomotion {
				state: HuskMotionState::Idle,
				state_fraction: 0.0,
				support_height: -0.6,
				next_foothold: Some(Vec3::new(1.5, -0.3, 3.3)),
			}),
			..HuskPose::new(Vessel::Ape, Vec3::new(0.0, -0.3, 2.5), 0.0)
		};
		let turned = |rate: f64| {
			let mut footwork = Footwork::new(Vessel::Ape);
			(1..=(rate as usize))
				.map(|_| footwork.step(&pose, 1.0 / rate).look.0)
				.collect::<Vec<_>>()
		};
		let sixty = turned(60.0);
		let target = *sixty.last().unwrap();
		assert!(target > 0.5, "{target}");
		// It starts slowly, never passes what it looks at, and is there by
		// the end of a second.
		assert!(sixty[0] < 0.05 * target, "{} of {target}", sixty[0]);
		assert!(sixty.windows(2).all(|pair| pair[1] >= pair[0]));
		assert!(sixty.iter().all(|turn| *turn <= target + 1e-9));
		assert!((Footwork::gaze(&pose).0 - target).abs() < 0.01 * target);
		// The same at thirty and a hundred and twenty frames a second.
		for (rate, every) in [(30.0, 2), (120.0, 1)] {
			let other = turned(rate);
			for (at, turn) in sixty.iter().enumerate().skip(1).step_by(2) {
				let other = if rate < 60.0 {
					other[at / every]
				} else {
					other[2 * at + 1]
				};
				assert!(
					(turn - other).abs() < 1e-9,
					"{rate} Hz at {at}: {turn} against {other}"
				);
			}
		}
	}
}
