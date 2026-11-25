//! How the reaper's own arms move: drawn, breathing, kicking, reloading,
//! stabbing, swinging, parrying, flinching and throwing. The Flutter
//! `ViewmodelMotion`, as pure functions of time.
//!
//! It decides nothing: the hunt says what the arms are doing and since
//! when, and [`pose`] answers where everything is at that moment. Nothing
//! but [`Trailing`] keeps state, so a hit-stop holds a pose by holding the
//! time, and a bench can show any frame; [`Trailing`] then lays on what
//! lags behind the pose from frame to frame, on springs that move the same
//! at any frame rate.
//!
//! Lengths are metres and angles radians, in the view's frame: x to the
//! right, y up, z away from the eye. Yaw turns the muzzle right, pitch up,
//! roll clockwise as the eye sees it.

use std::f32::consts::PI;
use std::f32::consts::TAU;
use std::sync::LazyLock;

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
use super::spring::Spring;
use super::spring::inertia;
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
	/// The weapon lunges in and is held ready.
	DiveIn,
	/// One stab: from the dive's ready pose when `dived`, or from rest at a
	/// husk in arm's reach. Each comes in from the other side.
	Stab { nth: u8, dived: bool },
	/// The weapon comes back out.
	DiveOut,
	/// A heavy swing in the weapon's own manner: a wind-up, and the blow
	/// [`CONTACT`] into it. Begun `dived`, it is held in the dive until the
	/// blow, and comes back out as the swing follows through, as the rules'
	/// dive does.
	Swing { dived: bool },
	/// A heavy swing at a husk in arm's reach: the rules land it on the
	/// press, so it starts at its own blow and follows through, fast.
	Strike,
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
/// How far a shot kicks the muzzle up at its height, in radians: the art
/// bible's 7° (§11.4).
pub const KICK: f32 = 7.0 * PI / 180.0;
/// How far a shot shoves the arm back toward the eye at its height, in
/// metres: enough to bring the muzzle back about 10 pt on a phone held
/// upright.
pub const SHOVE: f32 = 0.016;
/// How the kick is shared between the hand, at the wrist, and the forearm,
/// about the elbow: the wrist snaps up with the shot and the forearm is
/// dragged up after it, so the hand whips.
const KICK_WRIST: f32 = 0.7;
/// The arm spring the kick rides (the art bible's `arm`, §13): stiffness and
/// damping on a unit mass. Damped at 0.85, it settles in about 180 ms.
const ARM_STIFFNESS: f32 = 683.0;
const ARM_DAMPING: f32 = 44.4;
/// When what is left of a kick is let go: a twentieth of it is left 200 ms
/// in, and that is faded out over the last 50 ms so nothing jumps.
pub const RECOIL_SECONDS: f32 = 0.25;
/// The drum turns a chamber over six frames.
pub const DRUM_TURN_SECONDS: f32 = 6.0 / TICKS_PER_SECOND;
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
/// The ticks from the start of a swing to its blow: a swing the rules land
/// on a tick begins this many ticks before it.
pub const CONTACT_TICKS: i64 = (CONTACT * TICKS_PER_SECOND + 0.5) as i64;
pub const PARRY_SECONDS: f32 = 0.42;
pub const STRUCK_SECONDS: f32 = 0.45;
/// The throw lets go of the orb at 0.3 s.
pub const THROW_SECONDS: f32 = 0.62;
pub const THROW_RELEASE: f32 = 0.3;
const RAISE_SECONDS: f32 = 0.25;

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
		ViewAction::DiveIn => {
			dive(
				&mut pose.arm,
				manner,
				sample(LUNGE, since / DIVE_IN_SECONDS),
			);
		}
		ViewAction::Stab { nth, dived } => {
			if dived {
				dive(&mut pose.arm, manner, 1.0);
			}
			stab(&mut pose.arm, since, nth);
		}
		ViewAction::DiveOut => {
			let out = Ease::InOut.apply(since / DIVE_OUT_SECONDS);
			dive(&mut pose.arm, manner, 1.0 - out);
		}
		ViewAction::Swing { dived } => {
			if dived {
				dive(&mut pose.arm, manner, swing_dive(since));
			}
			swing(&mut pose, manner, since);
		}
		ViewAction::Strike => swing(&mut pose, manner, CONTACT + since),
		ViewAction::Parry => parry(&mut pose.arm, since),
		ViewAction::Struck => struck(&mut pose.arm, since),
		ViewAction::Raise => raised(&mut pose, since),
		ViewAction::Throw => throw(&mut pose, since),
	}
	pose
}

/// An action the arms were in when another cut it short: what it was, and
/// how far it had played, in seconds, when the cut came.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cut {
	pub from: ViewAction,
	pub played: f32,
}

/// How long the arms blend out of whatever `action` cut short, in seconds:
/// about a tenth of a second, and always over before the action's blow, so
/// the weapon is exactly where the rules land it when they do. A strike
/// lands on the press, so it cuts straight to its blow.
pub fn blend_seconds(action: ViewAction) -> f32 {
	match action {
		ViewAction::Strike => 0.0,
		ViewAction::Stab { .. } => BLEND_SECONDS.min(STAB_SECONDS),
		ViewAction::Swing { .. } => BLEND_SECONDS.min(CONTACT),
		_ => BLEND_SECONDS,
	}
}

/// Whether `action` begins with a jolt: a shot's kick, a dry fire's
/// twitch, a blow taken. Its own sudden start is the point of it, so a blend
/// into it carries on the motion before without smoothing the jolt away.
const fn jolts(action: ViewAction) -> bool {
	matches!(
		action,
		ViewAction::Fire | ViewAction::Empty | ViewAction::Struck
	)
}

/// How long a blend out of a cut takes, at most.
const BLEND_SECONDS: f32 = 0.1;
/// The step the speeds at a cut are measured over, in seconds.
const BLEND_STEP: f32 = 1.0e-4;

/// Everything at `since` seconds into `action`, as [`pose`] has it, carried
/// on from the action `cut` short: at the cut the arms are where the action
/// before had them and moving as it moved them, and the difference dies
/// away over [`blend_seconds`]. An action that [`jolts`] keeps its own
/// sudden start on top of the motion carried on, and a shot cutting short a
/// shot adds its kick to what is left of the last one's. It is a function of
/// time alone, like [`pose`]: held by a hit-stop, sampled anywhere.
pub fn flowing(
	manner: Swing,
	action: ViewAction,
	since: f32,
	clock: f32,
	cut: Option<Cut>,
	lively: bool,
) -> Pose {
	let mut now = lively_pose(manner, action, since, clock, lively);
	let blend = blend_seconds(action);
	let Some(cut) = cut else {
		return now;
	};
	if action == ViewAction::Fire && cut.from == ViewAction::Fire {
		// A kick is an impulse on a spring: a shot fired as the last one
		// recovers adds its kick to what is left of that one, which goes on
		// exactly as it would have.
		recoil(&mut now.arm, cut.played + since);
		return now;
	}
	if since >= blend {
		return now;
	}
	let before = |at: f32| lively_pose(manner, cut.from, at, clock, lively);
	let after = |at: f32| lively_pose(manner, action, at, clock, lively);
	let back = cut.played.min(BLEND_STEP);
	let (was, was_back) = (before(cut.played), before(cut.played - back));
	let (start, start_on) = (after(0.0), after(BLEND_STEP));
	let carry = |was: f32, was_back: f32, start: f32, start_on: f32, wrap: bool| {
		let mut offset = was - start;
		if wrap {
			offset = (offset + PI).rem_euclid(TAU) - PI;
		}
		let speed_before = if back > 0.0 {
			(was - was_back) / back
		} else {
			0.0
		};
		let speed_after = if jolts(action) {
			0.0
		} else {
			(start_on - start) / BLEND_STEP
		};
		inertia(offset, speed_before - speed_after, blend, since)
	};
	let channels = arm_channels(&now.arm);
	let mut blended = channels;
	let [was, was_back, start, start_on] =
		[&was.arm, &was_back.arm, &start.arm, &start_on.arm].map(arm_channels);
	for (slot, value) in blended.iter_mut().enumerate() {
		*value += carry(
			was[slot],
			was_back[slot],
			start[slot],
			start_on[slot],
			slot == SPIN,
		);
	}
	set_arm_channels(&mut now.arm, blended);
	if let (Some(left), Some(was), Some(was_back), Some(start), Some(start_on)) = (
		now.left.as_mut(),
		before(cut.played).left,
		before(cut.played - back).left,
		after(0.0).left,
		after(BLEND_STEP).left,
	) {
		let [was, was_back, start, start_on] =
			[&was, &was_back, &start, &start_on].map(left_channels);
		let mut channels = left_channels(left);
		for (slot, value) in channels.iter_mut().enumerate() {
			*value += carry(
				was[slot],
				was_back[slot],
				start[slot],
				start_on[slot],
				false,
			);
		}
		left.at = [channels[0], channels[1], channels[2]];
		left.pitch = channels[3];
		left.yaw = channels[4];
		left.roll = channels[5];
	}
	now
}

/// Everything at `since` seconds into `action`, as [`pose`] has it, with a
/// heavy swing's [`follow_through`] when `lively`: not for a player who
/// asked for less motion.
pub fn lively_pose(
	manner: Swing,
	action: ViewAction,
	since: f32,
	clock: f32,
	lively: bool,
) -> Pose {
	let mut pose = pose(manner, action, since, clock);
	if lively {
		match action {
			ViewAction::Swing { .. } => follow_through(&mut pose.arm, manner, since),
			ViewAction::Strike => follow_through(&mut pose.arm, manner, CONTACT + since),
			_ => {}
		}
	}
	pose
}

/// How far a heavy swing carries on past its strike before it comes back,
/// as a share of the strike's own sweep.
const OVERSHOOT: f32 = 0.1;

/// What a lively heavy swing lays on its curves, `since` seconds from its
/// press: from the contact it is driven on faster, carries on past where
/// the strike ends, and comes back to rest with the swing. It is nothing at
/// the contact itself, so the weapon is where the rules land it, and nothing
/// once the swing is at rest. A sweep, a chop and a thrust follow through;
/// a lash already whips past, a toll rings, and the pistol's and knife's
/// short swings are over too quickly to.
fn follow_through(arm: &mut ArmPose, manner: Swing, since: f32) {
	if !matches!(manner, Swing::Sweep | Swing::Chop | Swing::Thrust) {
		return;
	}
	let shape = shape(manner);
	let on = (since - CONTACT) / (shape.end - CONTACT);
	if on <= 0.0 || on >= 1.0 {
		return;
	}
	// u·(1 − u)², scaled to peak at one a third of the way on: it leaves the
	// contact moving and arrives at rest flat.
	let carry = OVERSHOOT * 6.75 * on * (1.0 - on) * (1.0 - on);
	let sweep = |curve: Keys| (sample(curve, STRIKE) - sample(curve, WIND_UP)) * carry;
	turn_arm_and_wrist(arm, sweep(shape.pitch), sweep(shape.yaw), sweep(shape.roll));
	arm.x += sweep(shape.x);
	arm.y += sweep(shape.y);
	arm.z += sweep(shape.z);
}

/// The arm's spin in its fist, among [`arm_channels`]: an angle that wraps.
const SPIN: usize = 9;

/// The arm's every number, in one order.
const fn arm_channels(arm: &ArmPose) -> [f32; 10] {
	[
		arm.x,
		arm.y,
		arm.z,
		arm.pitch,
		arm.yaw,
		arm.roll,
		arm.wrist_pitch,
		arm.wrist_yaw,
		arm.wrist_roll,
		arm.spin,
	]
}

const fn set_arm_channels(arm: &mut ArmPose, channels: [f32; 10]) {
	[
		arm.x,
		arm.y,
		arm.z,
		arm.pitch,
		arm.yaw,
		arm.roll,
		arm.wrist_pitch,
		arm.wrist_yaw,
		arm.wrist_roll,
		arm.spin,
	] = channels;
}

/// The left hand's place and turn, in one order.
const fn left_channels(left: &LeftPose) -> [f32; 6] {
	[
		left.at[0], left.at[1], left.at[2], left.pitch, left.yaw, left.roll,
	]
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
		ViewAction::Swing { .. } => shape(manner).end,
		ViewAction::Strike => shape(manner).end - CONTACT,
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

/// How long the arms take to trail a turn of the view, in seconds, and how
/// far they trail at most, in radians.
const SWAY_LAG: f32 = 0.08;
const SWAY_MOST: f32 = 0.12;
/// How stiff the arms' trail behind the view is, as the frequency of its
/// spring: about as quick as [`SWAY_LAG`].
const SWAY_FREQUENCY: f64 = 4.0;
/// How stiff the hand is on the wrist, as the frequency of its spring: it
/// is left behind by a forearm swinging fast and catches up as it slows.
const WRIST_FREQUENCY: f64 = 9.0;
/// How much of what the forearm has left the hand behind the wrist gives,
/// and the most it bends for it, in radians.
const WRIST_GIVE: f32 = 0.25;
const WRIST_MOST: f32 = 0.12;

/// What trails the arm from frame to frame: the whole arm behind the view's
/// turn, and the hand behind a forearm that swings fast, bent back at the
/// wrist and catching up as the swing slows.
#[derive(Clone, Copy, Debug)]
pub struct Trailing {
	/// The trail behind the view's turn, to the right and up.
	sway: [Spring; 2],
	/// Where the hand would point, following the forearm's pitch, yaw and
	/// roll.
	wrist: [Spring; 3],
}

impl Default for Trailing {
	fn default() -> Self {
		Self {
			sway: [Spring::new(SWAY_FREQUENCY, 0.0); 2],
			wrist: [Spring::new(WRIST_FREQUENCY, 0.0); 3],
		}
	}
}

impl Trailing {
	/// Lays the trail on `pose` for a frame `seconds` long, over which the
	/// view turned at `turn_rate` radians a second (right, then up) and the
	/// arms' own clock ran `moved` seconds: a hit-stop holds the hand with
	/// the arm. For a player who asked for less motion (`lively` false) the
	/// hand stays square on the forearm.
	pub fn follow(
		&mut self,
		pose: &mut Pose,
		turn_rate: [f32; 2],
		seconds: f32,
		moved: f32,
		lively: bool,
	) {
		for (spring, rate) in self.sway.iter_mut().zip(turn_rate) {
			let trailing = (-rate * SWAY_LAG).clamp(-SWAY_MOST, SWAY_MOST);
			spring.step(f64::from(trailing), f64::from(seconds));
		}
		sway(
			&mut pose.arm,
			self.sway[0].value as f32,
			self.sway[1].value as f32,
		);
		let arm = &mut pose.arm;
		let forearm = [arm.pitch, arm.yaw, arm.roll];
		let wrist = [
			&mut arm.wrist_pitch,
			&mut arm.wrist_yaw,
			&mut arm.wrist_roll,
		];
		for ((spring, angle), bend) in self.wrist.iter_mut().zip(forearm).zip(wrist) {
			if !lively {
				spring.settle(f64::from(angle));
				continue;
			}
			let behind = spring.step(f64::from(angle), f64::from(moved)) as f32 - angle;
			*bend += (WRIST_GIVE * behind).clamp(-WRIST_MOST, WRIST_MOST);
		}
	}
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

/// A shot: the kick and the drum's turn. Its burst and smear are printed
/// flat over the view (`hunt::marks`), not drawn in light on the pistol.
fn fire(pose: &mut Pose, since: f32) {
	recoil(&mut pose.arm, since);
	// The drum turns a chamber, smoothly, over six frames. It has six, so a
	// turn of a sixth looks like none once it is done.
	pose.drum_spin = Ease::InOut.apply(since / DRUM_TURN_SECONDS) * TAU / 6.0;
}

/// A shot's kick on the arm, `since` seconds after it. The hand snaps the
/// muzzle up toward [`KICK`] and the arm back [`SHOVE`] on the shot's own
/// frame, under the burst and its one-frame smear, and the arm spring lets
/// them down; the forearm is dragged up after the hand, at its height about
/// 40 ms in, so the hand whips. A kick is linear in the spring, so two add:
/// a shot fired while one recovers lays its own on what is left of the last
/// ([`flowing`]).
fn recoil(arm: &mut ArmPose, since: f32) {
	if !(0.0..RECOIL_SECONDS).contains(&since) {
		return;
	}
	// What little is left is faded out over the last moments, so the kick
	// ends flat and the arm is exactly at rest once it is over.
	let fade = 1.0 - Ease::InOut.apply((since - (RECOIL_SECONDS - RECOIL_FADE)) / RECOIL_FADE);
	let wrist = released(since) * KICK_WRIST * fade;
	let forearm = dragged(since) * (1.0 - KICK_WRIST) * fade;
	let scale = KICK / kick_peak();
	arm.wrist_pitch += wrist * scale;
	arm.pitch += forearm * scale;
	arm.wrist_roll += wrist * scale * 0.25;
	arm.z -= SHOVE * released(since) * fade;
}

/// How long the end of a kick is faded out over, in seconds.
const RECOIL_FADE: f32 = 0.05;

/// The arm spring's decay rate and its ringing frequency, in radians a
/// second.
fn arm_spring() -> (f32, f32) {
	let decay = ARM_DAMPING / 2.0;
	(decay, (ARM_STIFFNESS - decay * decay).sqrt())
}

/// The arm spring thrown out to 1 at the shot and let go, `since` seconds
/// after: 0 until the shot, all of it the instant after, then down toward
/// rest, settling in about 180 ms.
fn released(since: f32) -> f32 {
	if since <= 0.0 {
		return 0.0;
	}
	let (decay, ring) = arm_spring();
	(-decay * since).exp() * ((ring * since).cos() + decay / ring * (ring * since).sin())
}

/// The arm spring struck once, `since` seconds after the shot, as the
/// forearm is dragged up after the hand: 0 until the shot, 1 at its height
/// about 40 ms in, then on toward nothing.
fn dragged(since: f32) -> f32 {
	if since <= 0.0 {
		return 0.0;
	}
	let (decay, ring) = arm_spring();
	let peak = (ring / decay).atan() / ring;
	let at = |time: f32| (-decay * time).exp() * (ring * time).sin();
	at(since) / at(peak)
}

/// The height of the kick shared between the snapping wrist and the dragged
/// forearm, as a share of one alone: what the muzzle's rise is scaled by to
/// peak at [`KICK`], about 20 ms in.
fn kick_peak() -> f32 {
	static PEAK: LazyLock<f32> = LazyLock::new(|| {
		(1..=120)
			.map(|step| {
				let time = step as f32 * 0.0005;
				released(time) * KICK_WRIST + dragged(time) * (1.0 - KICK_WRIST)
			})
			.fold(0.0, f32::max)
	});
	*PEAK
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

/// How far in a swing begun in a dive is held, `since` seconds into it:
/// all the way until its blow, then back out over the rules' dive out.
pub fn swing_dive(since: f32) -> f32 {
	1.0 - Ease::InOut.apply((since - CONTACT) / DIVE_OUT_SECONDS)
}

/// The arm lunged in, `ready` of the way, with a weapon that swings in
/// `manner`.
fn dive(arm: &mut ArmPose, manner: Swing, ready: f32) {
	match manner {
		Swing::Stab | Swing::Shoot => {
			// Raised, nearer the middle, the point brought down onto the husk.
			arm.x -= 0.03 * ready;
			arm.y += 0.03 * ready;
			arm.z += 0.05 * ready;
			arm.pitch += 0.1 * ready;
			arm.wrist_pitch -= 0.3 * ready;
			arm.yaw -= 0.06 * ready;
			arm.wrist_yaw -= 0.08 * ready;
		}
		Swing::Sweep | Swing::Thrust | Swing::Toll | Swing::Lash | Swing::Chop => {
			// A heavy weapon is carried in raised, ready to swing: turned
			// toward the middle it would be swung from the wrong side.
			arm.y += 0.025 * ready;
			arm.z += 0.05 * ready;
			arm.pitch += 0.08 * ready;
		}
	}
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

#[cfg(test)]
mod tests {
	use super::*;

	/// A sweep shown `rate` times a second with the view held still, with
	/// the hand's lag laid on, at each of the times in `at`.
	fn swept(rate: f32, lively: bool, at: &[f32]) -> Vec<(Pose, Pose)> {
		let mut trailing = Trailing::default();
		let mut shown = Vec::new();
		let frames = (1.2 * rate).round() as usize;
		for frame in 1..=frames {
			let t = frame as f32 / rate;
			let bare = pose(Swing::Sweep, ViewAction::Swing { dived: false }, t, 0.0);
			let mut trailed = bare;
			trailing.follow(&mut trailed, [0.0; 2], 1.0 / rate, 1.0 / rate, lively);
			if at.iter().any(|when| (when - t).abs() < 1e-5) {
				shown.push((bare, trailed));
			}
		}
		shown
	}

	#[test]
	fn the_hand_lags_a_fast_swing_the_same_at_any_frame_rate() {
		let at = [0.1, 0.2, 0.3, 0.5, 1.2];
		let sixty = swept(60.0, true, &at);
		assert_eq!(sixty.len(), at.len());
		for rate in [30.0, 120.0] {
			for ((_, one), (_, other)) in sixty.iter().zip(swept(rate, true, &at)) {
				for (a, b) in [
					(one.arm.wrist_pitch, other.arm.wrist_pitch),
					(one.arm.wrist_yaw, other.arm.wrist_yaw),
					(one.arm.wrist_roll, other.arm.wrist_roll),
				] {
					assert!((a - b).abs() < 0.01, "{rate} Hz: {a} against {b}");
				}
			}
		}
		// Bent back as the strike sweeps across, square again at rest.
		let (bare, trailed) = sixty[2];
		assert!(
			(trailed.arm.wrist_yaw - bare.arm.wrist_yaw).abs() > 0.03,
			"{} against {}",
			trailed.arm.wrist_yaw,
			bare.arm.wrist_yaw
		);
		assert!((trailed.arm.wrist_yaw - bare.arm.wrist_yaw).abs() <= WRIST_MOST);
		let (bare, trailed) = sixty[4];
		assert!((trailed.arm.wrist_yaw - bare.arm.wrist_yaw).abs() < 1e-3);
		// The rest of the arm is not touched.
		assert_eq!(trailed.arm.yaw, bare.arm.yaw);
		assert_eq!(trailed.arm.x, bare.arm.x);
	}

	#[test]
	fn a_player_who_asked_for_less_motion_keeps_the_hand_square() {
		for (bare, trailed) in swept(60.0, false, &[0.1, 0.2, 0.3, 0.5]) {
			assert_eq!(trailed, bare);
		}
	}

	const ACTIONS: [ViewAction; 17] = [
		ViewAction::Idle,
		ViewAction::Draw,
		ViewAction::Fire,
		ViewAction::Empty,
		ViewAction::Reload,
		ViewAction::ActiveReload,
		ViewAction::Jam,
		ViewAction::DiveIn,
		ViewAction::Stab {
			nth: 1,
			dived: true,
		},
		ViewAction::DiveOut,
		ViewAction::Swing { dived: false },
		ViewAction::Swing { dived: true },
		ViewAction::Strike,
		ViewAction::Parry,
		ViewAction::Struck,
		ViewAction::Raise,
		ViewAction::Throw,
	];

	#[test]
	fn a_cut_carries_on_from_where_the_arms_were_and_how_they_moved() {
		let step = 1e-4;
		for (manner, lively) in [
			(Swing::Sweep, true),
			(Swing::Sweep, false),
			(Swing::Chop, true),
			(Swing::Stab, true),
			(Swing::Shoot, true),
			(Swing::Toll, true),
		] {
			for from in ACTIONS {
				for to in ACTIONS {
					if blend_seconds(to) <= 0.0 {
						continue;
					}
					for played in [0.05, 0.13, 0.3] {
						let cut = Some(Cut { from, played });
						let at_cut = arm_channels(&flowing(manner, to, 0.0, 1.0, cut, lively).arm);
						let just_after =
							arm_channels(&flowing(manner, to, step, 1.0, cut, lively).arm);
						let at = |action, t| lively_pose(manner, action, t, 1.0, lively).arm;
						let was = arm_channels(&at(from, played));
						let was_back = arm_channels(&at(from, played - BLEND_STEP));
						for slot in 0..10 {
							let mut gap = at_cut[slot] - was[slot];
							if slot == SPIN {
								gap = (gap + PI).rem_euclid(TAU) - PI;
							}
							assert!(
								gap.abs() < 1e-4,
								"{manner:?} {from:?} to {to:?} at {played}: {slot} jumps {gap}"
							);
							let speed = (just_after[slot] - at_cut[slot]) / step;
							let mut speed_before = (was[slot] - was_back[slot]) / BLEND_STEP;
							if jolts(to) {
								// Its own jolt goes on top.
								let own = arm_channels(&at(to, step))[slot]
									- arm_channels(&at(to, 0.0))[slot];
								speed_before += own / step;
							}
							assert!(
								(speed - speed_before).abs() < 0.15 + 0.03 * speed_before.abs(),
								"{manner:?} {from:?} to {to:?} at {played}: {slot} moves at {speed}, was {speed_before}"
							);
						}
					}
				}
			}
		}
	}

	#[test]
	fn a_heavy_swing_follows_through_past_its_strike_and_settles() {
		let swing = ViewAction::Swing { dived: false };
		for manner in [Swing::Sweep, Swing::Chop, Swing::Thrust] {
			let shape = shape(manner);
			let lively = |t: f32| lively_pose(manner, swing, t, 0.0, true);
			let still = |t: f32| pose(manner, swing, t, 0.0);
			// Exactly where the rules land it at the contact, and at rest.
			assert_eq!(lively(CONTACT), still(CONTACT), "{manner:?}");
			assert_eq!(lively(shape.end), still(shape.end), "{manner:?}");
			assert_eq!(lively(WIND_UP), still(WIND_UP), "{manner:?}");
			let channel = |pose: Pose| {
				match manner {
					Swing::Sweep => pose.arm.yaw,
					Swing::Chop => pose.arm.pitch,
					_ => pose.arm.z,
				}
			};
			// Driven through the contact faster than before.
			let step = 1e-3;
			let lively_pace = (channel(lively(CONTACT + step)) - channel(lively(CONTACT))) / step;
			let still_pace = (channel(still(CONTACT + step)) - channel(still(CONTACT))) / step;
			assert!(lively_pace.abs() > still_pace.abs(), "{manner:?}");
			// Carried on past where the strike ends, the way it was going.
			let strike_end = channel(still(STRIKE));
			let way = (strike_end - channel(still(WIND_UP))).signum();
			let furthest = (0..=100)
				.map(|n| channel(lively(STRIKE + (shape.end - STRIKE) * n as f32 / 100.0)))
				.fold(
					strike_end,
					|far, at| if (at - far) * way > 0.0 { at } else { far },
				);
			assert!(
				(furthest - strike_end) * way > 0.01,
				"{manner:?}: {furthest} past {strike_end}"
			);
		}
		// A lash, a toll and the short swings are left as they are, and so is
		// every swing for a player who asked for less motion.
		for manner in [Swing::Lash, Swing::Toll, Swing::Shoot, Swing::Stab] {
			assert_eq!(
				lively_pose(manner, swing, 0.35, 0.0, true),
				pose(manner, swing, 0.35, 0.0)
			);
		}
		assert_eq!(
			lively_pose(Swing::Sweep, swing, 0.35, 0.0, false),
			pose(Swing::Sweep, swing, 0.35, 0.0)
		);
	}

	#[test]
	fn a_blend_is_over_before_the_blow_lands() {
		for from in ACTIONS {
			let cut = Some(Cut { from, played: 0.2 });
			let swing = ViewAction::Swing { dived: false };
			assert_eq!(
				flowing(Swing::Sweep, swing, CONTACT, 0.5, cut, true),
				pose(Swing::Sweep, swing, CONTACT, 0.5)
			);
			let stab = ViewAction::Stab {
				nth: 2,
				dived: true,
			};
			assert_eq!(
				flowing(Swing::Stab, stab, STAB_SECONDS, 0.5, cut, true),
				pose(Swing::Stab, stab, STAB_SECONDS, 0.5)
			);
			// A strike lands on the press: it cuts straight to its blow.
			assert_eq!(
				flowing(Swing::Sweep, ViewAction::Strike, 0.0, 0.5, cut, true),
				pose(Swing::Sweep, ViewAction::Strike, 0.0, 0.5)
			);
			// Past its blend, an action is untouched.
			assert_eq!(
				flowing(
					Swing::Shoot,
					ViewAction::Fire,
					BLEND_SECONDS,
					0.5,
					cut,
					true
				),
				pose(Swing::Shoot, ViewAction::Fire, BLEND_SECONDS, 0.5)
			);
		}
	}

	/// How far the muzzle is turned up by a shot `since` seconds in: the
	/// forearm's turn and the hand's, on top of where they rest.
	fn muzzle_rise(pose: &Pose) -> f32 {
		let rest = super::pose(Swing::Shoot, ViewAction::Idle, 0.0, 0.0).arm;
		pose.arm.pitch + pose.arm.wrist_pitch - rest.pitch - rest.wrist_pitch
	}

	#[test]
	fn a_shot_kicks_seven_degrees_on_the_arm_spring_and_settles() {
		let fired = |since: f32| pose(Swing::Shoot, ViewAction::Fire, since, 0.0);
		let rises: Vec<(f32, f32)> = (0..=300)
			.map(|step| {
				let since = step as f32 / 1000.0;
				(since, muzzle_rise(&fired(since)))
			})
			.collect();
		let (at, most) =
			rises.iter().copied().fold(
				(0.0, 0.0),
				|best, now| if now.1 > best.1 { now } else { best },
			);
		assert!(
			(most.to_degrees() - 7.0).abs() < 0.05,
			"it rises {}°",
			most.to_degrees()
		);
		// It snaps up with the shot: at its height on the shot's first frame,
		// under the burst and the smear, and on its way down from the next.
		assert!((0.0..0.03).contains(&at), "at its height {at} s in");
		let frame = |frames: f32| muzzle_rise(&fired(frames / TICKS_PER_SECOND)) / most;
		assert!(frame(1.0) > 0.95, "{} of it on f0", frame(1.0));
		assert!(frame(2.0) < frame(1.0) && frame(4.0) < 0.7, "falling by f3");
		// Settled in about 180 ms, as the spring's envelope has it (the
		// forearm, dragged after the hand, holds a little on a moment longer),
		// and nothing at all by the end.
		assert!(
			rises[180].1.abs() < 0.1 * most,
			"{}° left at 180 ms",
			rises[180].1.to_degrees()
		);
		assert!(
			rises[220].1.abs() < 0.02 * most,
			"{}° left at 220 ms",
			rises[220].1.to_degrees()
		);
		assert_eq!(muzzle_rise(&fired(RECOIL_SECONDS)), 0.0);
		assert_eq!(fired(RECOIL_SECONDS).arm.z, fired(1.0).arm.z);
		// The arm comes back toward the eye with the kick.
		assert!((fired(at).arm.z + SHOVE).abs() < 0.2 * SHOVE);
	}

	#[test]
	fn the_hand_whips_ahead_of_the_forearm() {
		let peak_of = |channel: fn(&ArmPose) -> f32| {
			(0..=200)
				.map(|step| step as f32 / 1000.0)
				.map(|since| {
					(
						since,
						channel(&pose(Swing::Shoot, ViewAction::Fire, since, 0.0).arm),
					)
				})
				.fold(
					(0.0, f32::MIN),
					|best, now| if now.1 > best.1 { now } else { best },
				)
				.0
		};
		let rest = pose(Swing::Shoot, ViewAction::Idle, 0.0, 0.0).arm;
		let wrist = peak_of(|arm| arm.wrist_pitch);
		let forearm = peak_of(|arm| arm.pitch);
		// The hand is at its height the instant the shot goes; the forearm,
		// dragged up after it, at the arm spring's own, about 40 ms on.
		let lead = forearm - wrist;
		assert!(
			(0.03..=0.045).contains(&lead),
			"the wrist leads by {lead} s ({wrist} against {forearm}; at rest {rest:?})"
		);
	}

	#[test]
	fn the_drum_turns_a_chamber_smoothly_over_six_frames() {
		let drum = |frames: f32| {
			pose(
				Swing::Shoot,
				ViewAction::Fire,
				frames / TICKS_PER_SECOND,
				0.0,
			)
			.drum_spin
		};
		assert_eq!(drum(0.0), 0.0);
		assert!((drum(3.0) - TAU / 12.0).abs() < 1e-5, "halfway at three");
		assert!((drum(6.0) - TAU / 6.0).abs() < 1e-5, "a sixth at six");
		assert_eq!(drum(9.0), drum(6.0));
		// Smooth: it starts and ends at rest, and never turns back.
		let steps: Vec<f32> = (0..=12)
			.map(|half| drum(half as f32 * 0.5))
			.collect::<Vec<_>>()
			.windows(2)
			.map(|pair| pair[1] - pair[0])
			.collect();
		assert!(steps.iter().all(|step| *step >= 0.0));
		assert!(steps[0] < steps[5] && steps[11] < steps[6], "{steps:?}");
	}

	#[test]
	fn a_shot_fired_in_the_last_ones_recovery_adds_to_it() {
		let fire = ViewAction::Fire;
		let lone = |since: f32| pose(Swing::Shoot, fire, since, 0.0);
		for played in [0.02, 0.04, 0.09] {
			let cut = Some(Cut { from: fire, played });
			let stacked = |since: f32| flowing(Swing::Shoot, fire, since, 0.0, cut, true);
			// Nothing is reset: the arms are where the last kick had them.
			assert_eq!(stacked(0.0).arm, lone(played).arm, "at {played}");
			// The two kicks add, all the way through.
			for since in [0.01, 0.03, 0.05, 0.1] {
				let both = muzzle_rise(&lone(since)) + muzzle_rise(&lone(played + since));
				assert!(
					(muzzle_rise(&stacked(since)) - both).abs() < 1e-6,
					"{played} then {since}"
				);
			}
			// The second kick lands on top of what is left of the first.
			let highest = |shown: &dyn Fn(f32) -> Pose| {
				(0..=120)
					.map(|step| muzzle_rise(&shown(step as f32 / 1000.0)))
					.fold(f32::MIN, f32::max)
			};
			assert!(highest(&stacked) > highest(&lone) + 0.01, "{played}");
		}
		// Once the last has settled there is nothing left of it to add.
		let late = Some(Cut {
			from: fire,
			played: RECOIL_SECONDS,
		});
		assert_eq!(
			flowing(Swing::Shoot, fire, 0.05, 0.0, late, true),
			lone(0.05)
		);
	}

	#[test]
	fn an_empty_drum_still_knocks() {
		let dry = pose(Swing::Shoot, ViewAction::Empty, 1.0 / 30.0, 0.0);
		let rest = pose(Swing::Shoot, ViewAction::Idle, 0.0, 0.0);
		let knocked = dry.arm.wrist_pitch - rest.arm.wrist_pitch;
		assert!(
			knocked > 0.005 && knocked < 0.02,
			"a small dry knock: {knocked}"
		);
		assert_eq!(dry.drum_spin, 0.0, "the drum does not turn");
	}

	#[test]
	fn a_hit_stop_holds_the_hand_with_the_arm() {
		let mut trailing = Trailing::default();
		let mut moving = pose(Swing::Sweep, ViewAction::Swing { dived: false }, 0.2, 0.0);
		trailing.follow(&mut moving, [0.0; 2], 1.0 / 60.0, 1.0 / 60.0, true);
		let held = pose(Swing::Sweep, ViewAction::Swing { dived: false }, 0.22, 0.0);
		let mut first = held;
		trailing.follow(&mut first, [0.0; 2], 1.0 / 60.0, 1.0 / 60.0, true);
		// The arms' clock stopped: the hand does not catch up through it.
		let mut second = held;
		trailing.follow(&mut second, [0.0; 2], 1.0 / 60.0, 0.0, true);
		assert_eq!(first.arm.wrist_yaw, second.arm.wrist_yaw);
	}
}
