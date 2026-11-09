//! How the reaper's own arms move: drawn, breathing, kicking, reloading,
//! stabbing, swinging, parrying, flinching and throwing. The Flutter
//! `ViewmodelMotion`, as pure functions of time.
//!
//! It decides nothing: the hunt says what the arms are doing and since
//! when, and [`pose`] answers where everything is at that moment. Nothing
//! here keeps state, so a hit-stop holds a pose by holding the time, and a
//! bench can show any frame.
//!
//! Lengths are metres and angles radians, in the view's frame: x to the
//! right, y up, z away from the eye. Yaw turns the muzzle right, pitch up,
//! roll clockwise as the eye sees it.

use std::f32::consts::PI;
use std::f32::consts::TAU;

use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::BladeRules;
use reapers_rules::GunRules;
use reapers_rules::HuntRules;
use reapers_rules::WeaponKind;

use super::curves::Ease;
use super::curves::Keys;
use super::curves::eased;
use super::curves::end;
use super::curves::key;
use super::curves::knock;
use super::curves::sample;
use super::swings::shape;

/// The manner the weapon in hand swings in. A class weapon in the heavy
/// slot swings in its own; the pistol and the knife, pressed into a swing,
/// make a short one of theirs.
pub const fn manner(archetype: Archetype, heavy: WeaponKind) -> Swing {
	match archetype {
		Archetype::Gun | Archetype::Art => Swing::Shoot,
		Archetype::Blade => Swing::Stab,
		Archetype::Heavy => {
			match heavy {
				WeaponKind::Scythe | WeaponKind::ChainWhip => Swing::Sweep,
				WeaponKind::Spear | WeaponKind::LanternPole => Swing::Thrust,
				WeaponKind::Bell => Swing::Toll,
				WeaponKind::BalanceFlail => Swing::Lash,
				WeaponKind::Greatsword | WeaponKind::Axe => Swing::Chop,
			}
		}
	}
}

/// The manner of a swing. Each class weapon takes one ([`manner`]): the
/// scythe and the chain whip sweep, the spear and lantern pole thrust, the
/// bell tolls, the flail lashes, the greatsword and axe chop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swing {
	/// Wide and flat, right to left.
	Sweep,
	/// Drawn back, then driven straight in.
	Thrust,
	/// Raised high, then brought down overhead to ring.
	Toll,
	/// Cocked back, then whipped across and down with a snap.
	Lash,
	/// Raised up and to the right, then hacked down across.
	Chop,
	/// A pistol pressed into a swing: a short chop with the butt.
	Shoot,
	/// A knife pressed into a swing: a short slash.
	Stab,
}

/// What the arms are doing. Each plays from the moment it began and then
/// holds where it ends: a dive stays dived, a raised orb stays raised.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ViewAction {
	#[default]
	Idle,
	/// The weapon brought up from below.
	Draw,
	/// A shot: the muzzle kicks and recovers, and the drum turns.
	Fire,
	/// The trigger pulled on an empty drum.
	Empty,
	/// The pistol tips up and the left hand swaps the drum.
	Reload,
	/// A reload finished early by a well-timed press: the drum slammed home
	/// and the pistol spun.
	ActiveReload,
	/// A reload pressed too early: it rattles and will not seat.
	Jam,
	/// The knife lunges in and is held ready.
	DiveIn,
	/// One stab, from the dive. Each comes in from the other side.
	Stab { nth: u8 },
	/// The knife comes back out.
	DiveOut,
	/// A heavy swing in the weapon's own manner.
	Swing,
	/// The weapon brought up and across to turn a charge aside.
	Parry,
	/// The reaper is hit: the arm jolts.
	Struck,
	/// The left hand holds the orb up while the throw is timed.
	Raise,
	/// The left hand throws the orb underarm.
	Throw,
}

/// What the left hand carries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LeftHolds {
	#[default]
	Nothing,
	Drum,
	Orb,
}

/// The right arm, as offsets from where it rests.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArmPose {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	/// The whole arm, turned about its elbow.
	pub pitch: f32,
	pub yaw: f32,
	pub roll: f32,
	/// The hand bent at the wrist, on top of the arm's turn.
	pub wrist_pitch: f32,
	pub wrist_yaw: f32,
	pub wrist_roll: f32,
	/// The weapon alone spun forward in the fist, about its trigger.
	pub spin: f32,
}

/// The left arm: not an offset but a place in the view.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeftPose {
	pub at: [f32; 3],
	pub pitch: f32,
	pub yaw: f32,
	pub roll: f32,
	pub holds: LeftHolds,
	/// How far it has carried the spare drum toward the pistol's own: 0
	/// not at all, 1 there.
	pub to_drum: f32,
}

/// Everything in the first-person view at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
	pub arm: ArmPose,
	pub left: Option<LeftPose>,
	/// How far the pistol's drum has turned.
	pub drum_spin: f32,
	/// How far the pistol's drum has fallen out, in metres; past
	/// [`DRUM_GONE`] it is not drawn.
	pub drum_drop: f32,
	/// The muzzle flash's frame, while it shows.
	pub flash: Option<usize>,
	/// How bright a heavy swing's trail is, while it shows.
	pub trail: f32,
}

/// How far the drum falls before it is gone.
pub const DRUM_GONE: f32 = 0.119;

/// The hunt's ticks, sixty a second.
pub const TICKS_PER_SECOND: f32 = HuntRules::TICK_RATE as f32;

/// How long drawing a weapon takes: as long as the rules take to let it
/// fire.
pub const DRAW_SECONDS: f32 = Arsenal::SWITCH_TICKS as f32 / TICKS_PER_SECOND;
/// A shot's kick, and how fast it recovers: it peaks after a quarter of
/// the recovery and is gone by about 120 ms.
pub const KICK: f32 = 6.0 * PI / 180.0;
const KICK_RATE: f32 = 1.0 / 0.024;
/// A reload: the rules' 84 ticks.
pub const RELOAD_SECONDS: f32 = GunRules::RELOAD_TICKS as f32 / TICKS_PER_SECOND;
/// Where in a reload the new drum goes home.
const HOME: f32 = 0.62;
/// How long the spin of an active reload takes, and a jam's hold-up (the
/// rules' 30 ticks).
pub const FLOURISH_SECONDS: f32 = 0.3;
pub const JAM_SECONDS: f32 = GunRules::JAM_TICKS as f32 / TICKS_PER_SECOND;
/// Where a jam stops a reload.
const JAMMED_AT: f32 = 0.5;
/// The knife: the rules' 13 ticks in, a 25 cm stab in 90 ms, their 18
/// ticks out.
pub const DIVE_IN_SECONDS: f32 = BladeRules::IN_TICKS as f32 / TICKS_PER_SECOND;
pub const DIVE_OUT_SECONDS: f32 = BladeRules::OUT_TICKS as f32 / TICKS_PER_SECOND;
pub const STAB_REACH: f32 = 0.25;
pub const STAB_SECONDS: f32 = 0.09;
/// A heavy swing: a wind-up, then a 120° strike in 180 ms.
pub const WIND_UP: f32 = 0.12;
pub const STRIKE_SECONDS: f32 = 0.18;
const STRIKE: f32 = WIND_UP + STRIKE_SECONDS;
/// When a heavy swing lands: halfway through its strike.
pub const CONTACT: f32 = WIND_UP + STRIKE_SECONDS / 2.0;
pub const PARRY_SECONDS: f32 = 0.42;
pub const STRUCK_SECONDS: f32 = 0.45;
/// The throw lets go of the orb at 0.3 s.
pub const THROW_SECONDS: f32 = 0.62;
pub const THROW_RELEASE: f32 = 0.3;
const RAISE_SECONDS: f32 = 0.25;
/// The flash shows each of its three frames for one of the rig's ticks.
pub const FLASH_FRAME_SECONDS: f32 = 1.0 / 30.0;

/// Everything at `since` seconds into `action`, with a weapon in hand that
/// swings in `manner`, `clock` seconds into the hunt (for breathing).
pub fn pose(manner: Swing, action: ViewAction, since: f32, clock: f32) -> Pose {
	let mut pose = Pose::default();
	idle(&mut pose.arm, clock);
	match action {
		ViewAction::Idle => {}
		ViewAction::Draw => draw(&mut pose.arm, since),
		ViewAction::Fire => fire(&mut pose, since),
		ViewAction::Empty => {
			// The hammer falls on nothing: a twitch, no kick.
			pose.arm.wrist_pitch += knock(0.012, 30.0, since);
		}
		ViewAction::Reload => reload(&mut pose, since / RELOAD_SECONDS),
		ViewAction::ActiveReload => {
			let spun = (since / FLOURISH_SECONDS).clamp(0.0, 1.0);
			reload(&mut pose, HOME + spun * (1.0 - HOME));
			pose.arm.spin = Ease::InOut.apply(spun) * TAU;
		}
		ViewAction::Jam => {
			reload(&mut pose, JAMMED_AT);
			let stuck = (since / JAM_SECONDS).clamp(0.0, 1.0);
			let rattle = (stuck * PI * 14.0).sin() * (1.0 - stuck);
			pose.arm.wrist_roll += 0.09 * rattle;
			pose.arm.x += 0.004 * rattle;
		}
		ViewAction::DiveIn => dive(&mut pose.arm, sample(LUNGE, since / DIVE_IN_SECONDS)),
		ViewAction::Stab { nth } => {
			dive(&mut pose.arm, 1.0);
			stab(&mut pose.arm, since, nth);
		}
		ViewAction::DiveOut => {
			let out = Ease::InOut.apply(since / DIVE_OUT_SECONDS);
			dive(&mut pose.arm, 1.0 - out);
		}
		ViewAction::Swing => swing(&mut pose, manner, since),
		ViewAction::Parry => parry(&mut pose.arm, since),
		ViewAction::Struck => struck(&mut pose.arm, since),
		ViewAction::Raise => raised(&mut pose, since),
		ViewAction::Throw => throw(&mut pose, since),
	}
	pose
}

/// When `action` has finished moving and holds still, in seconds, with a
/// weapon in hand that swings in `manner`.
pub fn duration(manner: Swing, action: ViewAction) -> f32 {
	match action {
		ViewAction::Idle | ViewAction::Raise => 0.0,
		ViewAction::Draw => DRAW_SECONDS,
		ViewAction::Fire | ViewAction::Empty => 0.25,
		ViewAction::Reload => RELOAD_SECONDS,
		ViewAction::ActiveReload => FLOURISH_SECONDS,
		ViewAction::Jam => JAM_SECONDS,
		ViewAction::DiveIn => DIVE_IN_SECONDS,
		ViewAction::Stab { .. } => end(THRUST),
		ViewAction::DiveOut => DIVE_OUT_SECONDS,
		ViewAction::Swing => shape(manner).end,
		ViewAction::Parry => PARRY_SECONDS,
		ViewAction::Struck => STRUCK_SECONDS,
		ViewAction::Throw => THROW_SECONDS,
	}
}

fn idle(arm: &mut ArmPose, t: f32) {
	// Breathing: a slow rise and fall, and a wander slower still.
	arm.y += 0.0032 * (t * TAU * 0.23).sin();
	arm.x += 0.0018 * (t * TAU * 0.11 + 1.3).sin();
	arm.pitch += 0.008 * (t * TAU * 0.23 - 0.6).sin();
	arm.roll += 0.006 * (t * TAU * 0.13).sin();
}

/// The arm and hand trailing the view's turn: `sway` is a lagged turn,
/// in radians, to the right when positive.
pub fn sway(arm: &mut ArmPose, yaw: f32, pitch: f32) {
	arm.yaw += yaw * 0.4;
	arm.wrist_yaw += yaw * 0.6;
	arm.roll += yaw * 0.5;
	arm.pitch += pitch * 0.4;
	arm.wrist_pitch += pitch * 0.6;
}

fn draw(arm: &mut ArmPose, since: f32) {
	let rest = 1.0 - Ease::Out.apply(since / DRAW_SECONDS);
	// Brought up from below: the arm swings up about the elbow and the
	// hand rolls level as it comes.
	arm.y -= 0.06 * rest;
	arm.pitch -= 0.55 * rest;
	arm.wrist_pitch -= 0.3 * rest;
	arm.roll += 0.35 * rest;
}

fn fire(pose: &mut Pose, since: f32) {
	let kick = knock(KICK, KICK_RATE, since);
	let shove = knock(-0.03, 1.0 / 0.03, since);
	let arm = &mut pose.arm;
	// The hand kicks up at the wrist, the arm shoves back.
	arm.wrist_pitch += kick * 0.85;
	arm.pitch += kick * 0.15;
	arm.z += shove;
	arm.wrist_roll += kick * 0.25;
	// The drum turns a chamber, quickly but not at once. It has six, so a
	// turn of a sixth looks like none once it is done.
	pose.drum_spin = Ease::Out.apply(since / 0.08) * TAU / 6.0;
	if since < FLASH_FRAME_SECONDS * 3.0 {
		pose.flash = Some((since / FLASH_FRAME_SECONDS) as usize);
	}
}

const DIP: Keys = &[
	key(0.0, 0.0),
	eased(0.14, 1.0, Ease::Out),
	key(0.8, 1.0),
	key(1.0, 0.0),
];
/// The left hand's way up into view and back down, as a share of the
/// reload.
const LEFT_IN: Keys = &[
	key(0.14, 0.0),
	eased(0.36, 1.0, Ease::Out),
	key(0.7, 1.0),
	eased(0.88, 0.0, Ease::In),
];
/// How far it has carried the spare drum to the pistol's: close by while
/// the old drum falls, then home.
const TO_DRUM: Keys = &[
	key(0.3, 0.0),
	eased(0.46, 0.7, Ease::Out),
	key(0.56, 0.72),
	eased(HOME, 1.0, Ease::In),
	key(0.68, 1.0),
	key(0.8, 0.2),
];

/// How a reload brings the pistol into view: the forearm's lift about the
/// elbow, the muzzle's tip and the roll that shows the drum, in radians, and
/// how far in toward the middle the hand comes, in metres.
const RELOAD_LIFT: f32 = 0.22;
const RELOAD_TIP: f32 = 0.16;
const RELOAD_ROLL: f32 = 0.5;
const RELOAD_IN: f32 = 0.035;
/// How far the left hand rises from below the frame with the spare drum.
const LEFT_RISE: f32 = 0.25;

fn reload(pose: &mut Pose, share: f32) {
	if !(0.0..1.0).contains(&share) {
		return;
	}
	let dip = sample(DIP, share);
	let arm = &mut pose.arm;
	// The forearm comes up about the elbow and in toward the middle, the
	// muzzle tips down a little at the wrist, and the pistol rolls its drum
	// over toward the eye: the whole change happens where it is seen.
	arm.pitch += RELOAD_LIFT * dip;
	arm.wrist_pitch -= RELOAD_TIP * dip;
	arm.wrist_roll -= RELOAD_ROLL * dip;
	arm.wrist_yaw += 0.1 * dip;
	arm.x -= RELOAD_IN * dip;
	arm.y += 0.02 * dip;
	// The old drum drops out and is gone; the new one goes home.
	let home = share >= HOME;
	pose.drum_drop = if home || share < 0.18 {
		0.0
	} else {
		((share - 0.18) / 0.14).clamp(0.0, 1.0) * 0.12
	};
	if home {
		pose.drum_spin = Ease::Out.apply((share - HOME) / 0.1) * TAU;
	}
	let reach = sample(LEFT_IN, share);
	if reach > 0.001 {
		// Up from below the frame, left of the pistol, palm up with the
		// spare drum in it; then across to the pistol's drum.
		pose.left = Some(LeftPose {
			at: [-0.06 + 0.01 * reach, -0.42 + LEFT_RISE * reach, 0.52],
			pitch: 0.9 - 0.2 * reach,
			yaw: 0.55,
			roll: -0.5,
			holds: if home {
				LeftHolds::Nothing
			} else {
				LeftHolds::Drum
			},
			to_drum: sample(TO_DRUM, share),
		});
	}
}

/// The knife's lunge in, as a share of the dive.
const LUNGE: Keys = &[key(0.0, 0.0), eased(0.55, 1.12, Ease::Out), key(1.0, 1.0)];
/// One stab, from the ready pose: out in 90 ms and back in 120 ms.
const THRUST: Keys = &[
	key(0.0, 0.0),
	eased(STAB_SECONDS, 1.0, Ease::Out),
	key(STAB_SECONDS + 0.03, 0.96),
	key(STAB_SECONDS + 0.15, 0.0),
];

fn dive(arm: &mut ArmPose, ready: f32) {
	// Raised, nearer the middle, the point brought down onto the husk.
	arm.x -= 0.03 * ready;
	arm.y += 0.03 * ready;
	arm.z += 0.05 * ready;
	arm.pitch += 0.1 * ready;
	arm.wrist_pitch -= 0.3 * ready;
	arm.yaw -= 0.06 * ready;
	arm.wrist_yaw -= 0.08 * ready;
}

fn stab(arm: &mut ArmPose, since: f32, nth: u8) {
	let thrust = sample(THRUST, since);
	// Each stab comes in from the other side.
	let side = if nth % 2 == 1 { 1.0 } else { -1.0 };
	arm.z += STAB_REACH * thrust;
	arm.x -= (0.045 + 0.01 * side) * thrust;
	arm.y += 0.02 * thrust;
	arm.wrist_pitch -= 0.1 * thrust;
	arm.wrist_roll += 0.12 * side * thrust;
}

/// Shares a big turn of the weapon between the arm, about the elbow, and
/// the hand, at the wrist, the way a swing is thrown.
fn turn_arm_and_wrist(arm: &mut ArmPose, pitch: f32, yaw: f32, roll: f32) {
	const ARM: f32 = 0.45;
	arm.pitch += pitch * ARM;
	arm.wrist_pitch += pitch * (1.0 - ARM);
	arm.yaw += yaw * ARM;
	arm.wrist_yaw += yaw * (1.0 - ARM);
	arm.roll += roll * 0.6;
	arm.wrist_roll += roll * 0.4;
}

fn swing(pose: &mut Pose, manner: Swing, since: f32) {
	let shape = shape(manner);
	if since > shape.end {
		return;
	}
	turn_arm_and_wrist(
		&mut pose.arm,
		sample(shape.pitch, since),
		sample(shape.yaw, since),
		sample(shape.roll, since),
	);
	pose.arm.x += sample(shape.x, since);
	pose.arm.y += sample(shape.y, since);
	pose.arm.z += sample(shape.z, since);
	if shape.trail {
		// The trail lights as the strike begins, is brightest as it lands
		// and fades as it ends.
		pose.trail = sample(TRAIL, since);
	}
}

const TRAIL: Keys = &[
	key(WIND_UP - 0.02, 0.0),
	eased(CONTACT, 1.0, Ease::Out),
	key(STRIKE, 0.8),
	eased(STRIKE + 0.12, 0.0, Ease::Out),
];

// A parry brings the weapon up and across the middle of the view, turned
// flat to block, and holds it there: on an upright phone the view is narrow,
// so it moves in by centimetres, not a hand's breadth.
const PARRY_X: Keys = &[
	key(0.0, 0.0),
	eased(0.09, -0.045, Ease::Out),
	key(0.2, -0.05),
	key(0.42, 0.0),
];
const PARRY_ROLL: Keys = &[
	key(0.0, 0.0),
	eased(0.09, -0.8, Ease::Out),
	key(0.2, -0.85),
	key(0.42, 0.0),
];
const PARRY_YAW: Keys = &[
	key(0.0, 0.0),
	eased(0.09, -0.25, Ease::Out),
	key(0.2, -0.23),
	key(0.42, 0.0),
];
const PARRY_Y: Keys = &[
	key(0.0, 0.0),
	eased(0.09, 0.12, Ease::Out),
	key(0.2, 0.12),
	key(0.42, 0.0),
];

fn parry(arm: &mut ArmPose, since: f32) {
	turn_arm_and_wrist(arm, 0.0, sample(PARRY_YAW, since), 0.0);
	// Turned flat at the wrist alone: rolled about the elbow, low in the
	// corner, the hand would swing down out of the view.
	arm.wrist_roll += sample(PARRY_ROLL, since);
	arm.x += sample(PARRY_X, since);
	arm.y += sample(PARRY_Y, since);
}

fn struck(arm: &mut ArmPose, since: f32) {
	// Knocked down and back, and it shakes as it settles.
	let jolt = knock(1.0, 22.0, since) + 0.35 * (since * 60.0).sin() * (-since * 9.0).exp();
	arm.y -= 0.03 * jolt;
	arm.z -= 0.04 * jolt;
	arm.pitch -= 0.08 * jolt;
	arm.wrist_pitch -= 0.12 * jolt;
	arm.roll += 0.1 * jolt;
}

// The left hand's underarm throw: up from below, back down, then forward
// and up, letting go at 0.3 s.
const THROW_Y: Keys = &[
	key(0.0, -0.42),
	eased(0.14, -0.15, Ease::Out),
	key(0.22, -0.2),
	eased(THROW_RELEASE, -0.07, Ease::Out),
	key(0.44, -0.04),
	eased(THROW_SECONDS, -0.45, Ease::In),
];
const THROW_Z: Keys = &[
	key(0.0, 0.4),
	key(0.14, 0.42),
	key(0.22, 0.36),
	eased(THROW_RELEASE, 0.56, Ease::Out),
	key(0.44, 0.62),
	key(THROW_SECONDS, 0.5),
];
const THROW_PITCH: Keys = &[
	key(0.0, 0.5),
	key(0.14, 0.35),
	key(0.22, -0.2),
	eased(THROW_RELEASE, 0.7, Ease::Out),
	key(THROW_SECONDS, 1.0),
];
/// Where the raised orb is held while the throw is timed.
const RAISED: [f32; 3] = [-0.058, -0.15, 0.42];

fn throw(pose: &mut Pose, since: f32) {
	if since > THROW_SECONDS {
		return;
	}
	let share = since / THROW_SECONDS;
	pose.left = Some(LeftPose {
		at: [
			RAISED[0] + 0.03 * share,
			sample(THROW_Y, since),
			sample(THROW_Z, since),
		],
		pitch: sample(THROW_PITCH, since),
		yaw: 0.35,
		roll: -0.4,
		holds: if since < THROW_RELEASE {
			LeftHolds::Orb
		} else {
			LeftHolds::Nothing
		},
		to_drum: 0.0,
	});
	// The right arm draws back out of the way.
	pose.arm.y -= 0.04 * (PI * share).sin();
	pose.arm.x += 0.02 * (PI * share).sin();
}

fn raised(pose: &mut Pose, since: f32) {
	let rise = Ease::Out.apply(since / RAISE_SECONDS);
	let bob = (since * TAU * 0.7).sin();
	let low = THROW_Y[0].value;
	pose.left = Some(LeftPose {
		at: [
			RAISED[0],
			low + (RAISED[1] - low) * rise + 0.003 * bob,
			RAISED[2],
		],
		pitch: 0.35 + 0.02 * bob,
		yaw: 0.35,
		roll: -0.4,
		holds: LeftHolds::Orb,
		to_drum: 0.0,
	});
	pose.arm.y -= 0.015 * rise;
	pose.arm.x += 0.008 * rise;
}
