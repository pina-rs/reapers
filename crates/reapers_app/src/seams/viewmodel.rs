//! The reaper's own arms and weapons in first person, and the light their
//! blows and the capture orb throw into the world.
//!
//! The hunt tells [`viewmodel_view`] which slot's weapon is in hand, what
//! the arms are doing, what the blows have done to the player ([`Juice`])
//! and the orb as its rules see it ([`OrbView`]), all in the hunt's own
//! ticks. The weapon and the orb are the rules' own types.
//!
//! The seam shows the arms once for every frame the display draws, one
//! frame ahead: each pose is sampled where the hunt will be when the frame
//! is shown and eased into over it, so it is reached when it is due. The
//! hunt's tick it shows runs on with the display between the rules' ticks,
//! never back, and never more than one of them ahead.
//!
//! The rules' events say when an action begins: `WeaponDrawn` is
//! [`ViewAction::Draw`]; `ShotFired` is [`ViewAction::Fire`];
//! `ReloadStarted` and `ReloadJammed` are [`ViewAction::Reload`] and
//! [`ViewAction::Jam`]; `ReloadFinished` when active is
//! [`ViewAction::ActiveReload`]; `DiveStarted` is [`ViewAction::DiveIn`],
//! each of the blade's strikes a [`ViewAction::Stab`], and the dive's way
//! back out [`ViewAction::DiveOut`]; a heavy dive's swing is
//! [`ViewAction::Swing`], begun [`CONTACT_TICKS`](arms::CONTACT_TICKS)
//! before the rules land it, and a heavy strike in arm's reach, which the
//! rules land on the press, is [`ViewAction::Strike`]; a stab in arm's reach
//! is a [`ViewAction::Stab`] that was not dived;
//! `Parried` and `Struck` are [`ViewAction::Parry`] and
//! [`ViewAction::Struck`]; ORB held is [`ViewAction::Raise`], and
//! `OrbThrown` is [`ViewAction::Throw`]. The seam works out every pose
//! itself, tick by tick between renders, so the hunt need render only when
//! something begins. [`viewmodel_show`] takes new props without a render,
//! for a caller with a clock of its own, such as the bench.
//!
//! It draws in two scenes. The arms have a scene of their own, drawn over
//! everything from a fixed eye, so they never reach into the room. The orb,
//! the tracers and the soul-shard sparks belong to the room, so they go in
//! the world's scene, the one the husk is in, named by [`WorldView`].
//!
//! The shake turns the arms here. On the world it is the owner's to apply,
//! with [`shake_turn`], because the owner holds the world's camera. Neither
//! ever moves the camera's picture.

use std::cell::RefCell;
use std::rc::Rc;

use leptoskit::native::animation::Easing;
use leptoskit::native::animation::Keyframe;
use leptoskit::native::animation::Keyframes;
use leptoskit::native::reactive::For;
use leptoskit::prelude::*;
use leptoskit::scene3d::SceneId;
use leptoskit::scene3d::Transform;
use reactive_graph::owner::on_cleanup;
use reapers_rules::Archetype;
use reapers_rules::OrbStage;
use reapers_rules::OrbView;
use reapers_rules::WeaponKind;
use reapers_world::pose::to_platform;

use crate::render::arms;
use crate::render::arms::FLASH_FRAME_SECONDS;
use crate::render::arms::TICKS_PER_SECOND;
pub use crate::render::arms::ViewAction;
use crate::render::camera::Point;
use crate::render::curves::Ease;
use crate::render::effects::Burst;
use crate::render::effects::Tracer;
use crate::render::effects::WorldFrame;
use crate::render::effects::WorldFx;
use crate::render::juice;
pub use crate::render::juice::Jolt;
pub use crate::render::juice::Juice;
use crate::render::orb;
use crate::render::orb::OrbPlace;
use crate::render::scene::Key;
use crate::render::scene::scene;
use crate::render::turn::add;
use crate::render::turn::inverse;
use crate::render::turn::lerp;
use crate::render::turn::rotate;
use crate::render::turn::sub;
use crate::render::turn::view_turn;
use crate::render::viewmodel::VIEWMODEL_FOV;
use crate::render::viewmodel::ViewmodelRig;
use crate::render::viewmodel::place;
use crate::theme::BONE;

/// The arms' own scene.
pub const VIEWMODEL: Key = Key::Fixed("viewmodel");

/// How many guest frames a tracer lasts.
const TRACER_FRAMES: f32 = 4.0;
/// How long a burst of shards flies, in seconds.
const SHARD_SECONDS: f32 = 0.4;
/// How long a damage number floats, in milliseconds.
const NUMBER_MILLIS: f32 = 800.0;

/// The room the effects are drawn in: the world's scene and its camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldView {
	/// The scene the husk is in.
	pub scene: Key,
	/// The world's camera: where the eye is and which way it looks.
	pub camera: Transform,
	/// Its vertical field of view, in radians.
	pub fov_y: f32,
	/// The view's width over its height. The arms' view fills the same box.
	pub aspect: f32,
	/// The middle of the husk, and the ground under it, in the scene.
	pub husk: Point,
	pub ground: Point,
}

/// A blow that struck the husk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitView {
	/// The hunt's tick it lands on. It may be ahead of now: nothing shows
	/// until it lands.
	pub tick: i64,
	/// Where it struck, in the world's scene.
	pub at: Point,
	pub damage: i64,
	pub crit: bool,
	/// The husk's own light. Its shards fly in this colour, never white.
	pub color: [f32; 3],
}

/// Everything the arms and their effects show.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewmodelProps {
	/// What the selected slot holds.
	pub archetype: Archetype,
	/// The reaper's class weapon, in the heavy slot. It sets how a swing
	/// goes.
	pub heavy: WeaponKind,
	pub action: ViewAction,
	/// The hunt's tick the action began on.
	pub action_tick: i64,
	/// The hunt's tick at this render, and how many pass each second: sixty
	/// while the hunt runs, none while it is paused.
	pub now: i64,
	pub rate: f32,
	/// How fast the view is turning, in radians a second: to the right,
	/// and up. The arms trail it.
	pub turn_rate: [f32; 2],
	pub juice: Juice,
	/// The latest orb thrown, as the rules report it, from its throw to its
	/// end.
	pub orb: Option<OrbView>,
	/// The room, for the orb and the effects. Without one only the arms
	/// show.
	pub world: Option<WorldView>,
	/// The latest blow that struck.
	pub hit: Option<HitView>,
}

impl ViewmodelProps {
	/// The arms at rest with the weapon of a slot of `archetype` in hand, and
	/// `heavy` in the heavy slot, at tick `now`.
	pub fn at_rest(archetype: Archetype, heavy: WeaponKind, now: i64) -> Self {
		Self {
			archetype,
			heavy,
			action: ViewAction::Idle,
			action_tick: now,
			now,
			rate: TICKS_PER_SECOND,
			turn_rate: [0.0, 0.0],
			juice: Juice::default(),
			orb: None,
			world: None,
			hit: None,
		}
	}

	/// The manner the weapon in hand swings in.
	pub const fn manner(&self) -> arms::Swing {
		arms::manner(self.archetype, self.heavy)
	}

	/// Seconds the arms have moved through their action by `tick`: the
	/// ticks passed since it began, less any a hit-stop held.
	pub fn action_seconds(&self, tick: f64) -> f32 {
		let moved = self.juice.motion_tick(tick) - self.juice.motion_tick(self.action_tick as f64);
		(moved / f64::from(TICKS_PER_SECOND)).max(0.0) as f32
	}
}

/// How far the dive lens has gone in, from 0 to 1, `since` seconds into
/// `action`.
pub fn dive(action: ViewAction, since: f32) -> f32 {
	match action {
		ViewAction::DiveIn => (since / arms::DIVE_IN_SECONDS).clamp(0.0, 1.0),
		ViewAction::Stab { dived: true, .. } => 1.0,
		ViewAction::DiveOut => 1.0 - (since / arms::DIVE_OUT_SECONDS).clamp(0.0, 1.0),
		ViewAction::Swing { dived: true } => arms::swing_dive(since),
		_ => 0.0,
	}
}

/// The turn that shakes the world's camera at `tick`, for a view `height`
/// pixels tall seen through `fov_y`: never more than
/// [`juice::MAX_SHAKE_PIXELS`] on screen. Multiply it into the camera's
/// rotation.
pub fn shake_turn(juice: &Juice, tick: f64, fov_y: f32, height: f32) -> [f32; 4] {
	let shake = juice.shake_at(tick);
	let most = juice::pixels_to_angle(juice::MAX_SHAKE_PIXELS, height, fov_y);
	view_turn(shake.x * most, shake.y * most, shake.roll * most)
}

/// A damage number, rising from where the blow struck.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Number {
	/// The tick of the blow, which tells numbers apart.
	tick: i64,
	/// Where it rises from, in percent of the view's width and height.
	x: f32,
	y: f32,
	damage: i64,
	crit: bool,
}

/// The seam's state between renders.
struct Seam {
	props: Option<ViewmodelProps>,
	/// The hunt's tick the arms were last shown at: the rules' tick, and as
	/// far past it as the display has run since.
	shown: Option<f64>,
	/// The action the latest one cut short, for the arms to blend out of.
	cut: Option<arms::Cut>,
	rig: Option<ViewmodelRig>,
	/// The effects, and the scene they were spawned in: a world scene that
	/// was forgotten and made afresh needs them again.
	world: Option<(Key, SceneId, WorldFx)>,
	/// What lags behind the arms: their trail behind the view's turn, and
	/// the hand behind the forearm.
	trailing: arms::Trailing,
	/// Where the orb left the hand, for the throw released on that tick.
	thrown: Option<(i64, Point)>,
	/// The latest two blows: two bursts of shards in flight at once.
	shards: [Option<HitView>; 2],
	numbers: RwSignal<Vec<Number>>,
	mounted: u32,
	ticking: bool,
}

thread_local! {
	static SEAM: RefCell<Option<Rc<RefCell<Seam>>>> = const { RefCell::new(None) };
}

fn seam() -> Rc<RefCell<Seam>> {
	SEAM.with_borrow_mut(|seam| {
		seam.get_or_insert_with(|| {
			Rc::new(RefCell::new(Seam {
				props: None,
				shown: None,
				cut: None,
				rig: None,
				world: None,
				trailing: arms::Trailing::default(),
				thrown: None,
				shards: [None; 2],
				numbers: crate::ui::lasting(|| RwSignal::new(Vec::new())),
				mounted: 0,
				ticking: false,
			}))
		})
		.clone()
	})
}

/// Hands the arms new props without a render. The next tick shows them.
pub fn viewmodel_show(props: &ViewmodelProps) {
	let seam = seam();
	let mut seam = seam.borrow_mut();
	if seam.props.as_ref() == Some(props) {
		return;
	}
	if let Some(before) = seam.props {
		seam.cut = next_cut(&before, props, seam.cut);
	}
	seam.props = Some(*props);
}

/// What the arms blend out of once `after` follows `before`, which blended
/// out of `cut`: a new action cuts the one before short where it had got
/// to; the same action goes on blending; a hunt shown from an earlier tick
/// (a bench scrubbing back) has nothing to blend out of.
fn next_cut(
	before: &ViewmodelProps,
	after: &ViewmodelProps,
	cut: Option<arms::Cut>,
) -> Option<arms::Cut> {
	if (after.action, after.action_tick) == (before.action, before.action_tick) {
		return cut;
	}
	if after.action_tick < before.action_tick || after.now < before.now {
		return None;
	}
	Some(arms::Cut {
		from: before.action,
		played: before.action_seconds(after.action_tick as f64),
	})
}

/// The hunt's tick to show after `shown` once `delta` more seconds have
/// passed, for props at the rules' tick `now`, passing at `rate` a second:
/// on with the display, but never behind the rules nor more than one of
/// their ticks ahead of them. A clock that does not run shows its tick.
fn shown_tick(shown: Option<f64>, delta: f64, now: i64, rate: f32) -> f64 {
	let now = now as f64;
	match shown {
		Some(shown) if rate > 0.0 => (shown + delta * f64::from(rate)).clamp(now, now + 1.0),
		_ => now,
	}
}

/// Keeps the seam ticking while it is on screen: from this render until
/// the render that replaces it, or the screen goes.
fn mount(seam: &Rc<RefCell<Seam>>) {
	let mut state = seam.borrow_mut();
	state.mounted += 1;
	if !state.ticking {
		state.ticking = true;
		let ticking = Rc::downgrade(seam);
		crate::render::ticker::every_frame(move |delta| {
			let Some(seam) = ticking.upgrade() else {
				return false;
			};
			let mut seam = seam.borrow_mut();
			if seam.mounted == 0 {
				seam.ticking = false;
				seam.hide_world();
				return false;
			}
			seam.tick(delta);
			true
		});
	}
	drop(state);
	let unmount = send_wrapper::SendWrapper::new(Rc::downgrade(seam));
	on_cleanup(move || {
		if let Some(seam) = unmount.take().upgrade() {
			let mut seam = seam.borrow_mut();
			seam.mounted = seam.mounted.saturating_sub(1);
		}
	});
}

impl Seam {
	fn tick(&mut self, delta: f64) {
		let Some(props) = self.props else {
			return;
		};
		let shown = shown_tick(self.shown, delta, props.now, props.rate);
		self.shown = Some(shown);
		// Each pose is sampled where the hunt will be one frame from now, and
		// eased into over that frame, so it arrives when it is due.
		let seconds = (delta as f32).clamp(1.0 / 240.0, 0.1);
		let tick = shown + f64::from(seconds) * f64::from(props.rate);
		let since = props.action_seconds(tick);
		let clock = (props.juice.motion_tick(tick) / f64::from(TICKS_PER_SECOND)) as f32;
		let lively = !crate::platform::reduced_motion();
		let mut pose = arms::flowing(props.manner(), props.action, since, clock, self.cut, lively);
		// The arms' own clock over the frame, which a hit-stop holds.
		let before = tick - f64::from(seconds) * f64::from(props.rate);
		let moved = ((props.juice.motion_tick(tick) - props.juice.motion_tick(before))
			/ f64::from(TICKS_PER_SECOND))
		.max(0.0) as f32;
		self.trailing
			.follow(&mut pose, props.turn_rate, seconds, moved, lively);
		let arms_scene = scene(VIEWMODEL);
		let ops = arms_scene.ops();
		let rig = self.rig.get_or_insert_with(|| ViewmodelRig::spawn(ops));
		rig.show(
			ops,
			props.archetype,
			&pose,
			props.juice.shake_at(tick),
			seconds,
		);
		arms_scene.flush();
		match props.world {
			Some(world) => self.show_world(&props, &world, &pose, tick, seconds),
			None => self.hide_world(),
		}
	}

	/// Shows the orb, the tracer, the shards and the numbers in the world.
	fn show_world(
		&mut self,
		props: &ViewmodelProps,
		world: &WorldView,
		pose: &arms::Pose,
		tick: f64,
		seconds: f32,
	) {
		let world_scene = scene(world.scene);
		let ops = world_scene.ops();
		let id = world_scene.id();
		if self
			.world
			.as_ref()
			.is_none_or(|(key, spawned, _)| *key != world.scene || *spawned != id)
		{
			self.world = Some((world.scene, id, WorldFx::spawn(ops, None)));
			self.shards = [None; 2];
			self.thrown = None;
			self.trailing = arms::Trailing::default();
			self.numbers.set(Vec::new());
		}
		let mut frame = WorldFrame {
			eye: world.camera.translation,
			..WorldFrame::default()
		};
		if let Some(orb) = props.orb {
			frame.orb = Some(self.orb_look(&orb, world, tick));
		}
		if props.archetype == Archetype::Gun && props.action == ViewAction::Fire {
			frame.tracer = tracer(props, world, pose, tick);
		}
		if let Some(hit) = props.hit
			&& self.shards[0].is_none_or(|latest| latest.tick != hit.tick)
		{
			self.shards = [Some(hit), self.shards[0]];
		}
		for (slot, shard) in self.shards.iter().enumerate() {
			frame.sparks[slot] = shard.and_then(|hit| shards(&hit, tick));
		}
		if let Some(hit) = props.hit {
			self.number(world, &hit, tick);
		}
		if let Some((_, _, fx)) = &mut self.world {
			fx.show(ops, &frame, seconds);
		}
		world_scene.flush();
	}

	/// How `orb` looks at `tick`, thrown from the hand, and the husk it
	/// draws in.
	fn orb_look(&mut self, orb: &OrbView, world: &WorldView, tick: f64) -> (orb::OrbLook, Point) {
		// The hand lets go low and to the left, ahead of the eye.
		let release = [-0.07, -0.1, -0.45];
		if orb.stage == OrbStage::Flight && self.thrown.is_none_or(|(at, _)| at != orb.thrown_at) {
			self.thrown = Some((orb.thrown_at, carry(world, release)));
		}
		let from = self
			.thrown
			.map_or_else(|| carry(world, release), |(_, from)| from);
		// The rules say where it flies, from the eye: the husk, or where it
		// comes down when it misses.
		let target = add(
			world.camera.translation,
			to_platform(orb.target).map(|metres| metres as f32),
		);
		let place = OrbPlace {
			from,
			husk: target,
			ground: if orb.on_target { world.ground } else { target },
		};
		let ticks = (tick - orb.thrown_at as f64).max(0.0) as f32;
		(orb::look(orb, ticks, &place), target)
	}

	/// Puts up the number for `hit` the moment it lands.
	fn number(&self, world: &WorldView, hit: &HitView, tick: f64) {
		let landed = tick - hit.tick as f64;
		if !(0.0..f64::from(TICKS_PER_SECOND)).contains(&landed) {
			return;
		}
		if self
			.numbers
			.with_untracked(|numbers| numbers.iter().any(|number| number.tick == hit.tick))
		{
			return;
		}
		let Some((x, y)) = project(world, hit.at) else {
			return;
		};
		self.numbers.update(|numbers| {
			// Only numbers still rising are kept.
			numbers.retain(|number| hit.tick - number.tick < 60);
			numbers.push(Number {
				tick: hit.tick,
				x,
				y,
				damage: hit.damage,
				crit: hit.crit,
			});
		});
	}

	/// Hides every effect in the world.
	fn hide_world(&mut self) {
		self.shards = [None; 2];
		self.thrown = None;
		self.numbers.set(Vec::new());
		let Some((key, id, fx)) = &mut self.world else {
			return;
		};
		let world_scene = scene(*key);
		if world_scene.id() != *id {
			return;
		}
		fx.show(world_scene.ops(), &WorldFrame::default(), 0.0);
		world_scene.flush();
	}
}

/// The tracer of a shot `tick` into it: from the muzzle to what it struck,
/// or far ahead, over [`TRACER_FRAMES`] frames. Its head races out in two;
/// its tail follows, and it fades.
fn tracer(
	props: &ViewmodelProps,
	world: &WorldView,
	pose: &arms::Pose,
	tick: f64,
) -> Option<Tracer> {
	let frames = ((tick - props.action_tick as f64) / f64::from(TICKS_PER_SECOND)) as f32
		/ FLASH_FRAME_SECONDS;
	if !(0.0..TRACER_FRAMES).contains(&frames) {
		return None;
	}
	let muzzle = carry(world, place(props.archetype, pose).muzzle);
	let struck = props
		.hit
		.filter(|hit| (0..=6).contains(&(hit.tick - props.action_tick)))
		.map(|hit| hit.at);
	let target = struck
		.or_else(|| scene(world.scene).shot_target(props.action_tick))
		.unwrap_or_else(|| ahead(world, 25.0));
	let head = f32::midpoint(frames, 1.0).min(1.0);
	let tail = ((frames - 0.5) / (TRACER_FRAMES - 1.0)).clamp(0.0, 0.9);
	Some(Tracer {
		from: lerp(muzzle, target, tail),
		to: lerp(muzzle, target, head),
		brightness: 1.0 - frames / TRACER_FRAMES,
	})
}

/// The burst of shards from `hit` at `tick`, while it flies.
fn shards(hit: &HitView, tick: f64) -> Option<Burst> {
	let age = ((tick - hit.tick as f64) / f64::from(TICKS_PER_SECOND)) as f32;
	(0.0..SHARD_SECONDS).contains(&age).then(|| {
		Burst {
			at: hit.at,
			spread: Ease::Out.apply(age / (SHARD_SECONDS * 0.8)),
			brightness: 1.0 - age / SHARD_SECONDS,
			size: if hit.crit { 0.55 } else { 0.38 },
			color: hit.color,
		}
	})
}

/// The point in the world drawn where `point`, in the arms' view, is drawn:
/// as far from the eye, through the world's wider or narrower lens.
fn carry(world: &WorldView, point: Point) -> Point {
	let lens = (world.fov_y * 0.5).tan() / (VIEWMODEL_FOV * 0.5).tan();
	let local = [point[0] * lens, point[1] * lens, point[2]];
	add(
		world.camera.translation,
		rotate(world.camera.rotation, local),
	)
}

/// The point `distance` metres straight ahead of the world's eye.
fn ahead(world: &WorldView, distance: f32) -> Point {
	add(
		world.camera.translation,
		rotate(world.camera.rotation, [0.0, 0.0, -distance]),
	)
}

/// Where `point` is drawn, in percent of the view's width and height from
/// its top left, or `None` behind the eye.
fn project(world: &WorldView, point: Point) -> Option<(f32, f32)> {
	let local = rotate(
		inverse(world.camera.rotation),
		sub(point, world.camera.translation),
	);
	let depth = -local[2];
	if depth < 0.05 {
		return None;
	}
	let tan = (world.fov_y * 0.5).tan();
	let x = local[0] / (depth * tan * world.aspect.max(0.01));
	let y = local[1] / (depth * tan);
	Some((50.0 + 50.0 * x, 50.0 - 50.0 * y))
}

/// The arms over whatever is under them, and the damage numbers: a box
/// filling its parent, that takes no touches.
pub fn viewmodel_view(props: &ViewmodelProps) -> Element {
	let seam = seam();
	viewmodel_show(props);
	mount(&seam);
	let numbers = seam.borrow().numbers;
	let id = scene(VIEWMODEL).id();
	if let Some(world) = props.world {
		let target = scene(world.scene).id();
		thread_local! {
			static LAST_FORWARD: std::cell::Cell<Option<(SceneId, SceneId)>> =
				const { std::cell::Cell::new(None) };
		}
		if LAST_FORWARD.with(|last| last.get() != Some((id, target))) {
			crate::render::pointer::forward(id, target);
			LAST_FORWARD.with(|last| last.set(Some((id, target))));
		}
	}
	View()
		.class("absolute inset-0 overflow-hidden")
		.test_id("viewmodel")
		.aria_hidden(true)
		// The arms are what the player holds, not something to read: the
		// screen around them says what is happening.
		.child(
			Scene3d(id)
				.class("absolute inset-0 w-full h-full")
				.aria_hidden(true),
		)
		// The list itself covers the view: each number is placed by a
		// percentage of it.
		.child(
			For(move || numbers.get(), |number| number.tick, number_view)
				.class("absolute inset-0")
				.aria_hidden(true),
		)
}

/// A damage number: it pops, rises and fades on the platform's own clock.
/// A critical one is bigger, and gold.
fn number_view(number: Number) -> Element {
	let (size, colour, pop) = if number.crit {
		(34, "#ffc24a".to_owned(), 1.7)
	} else {
		(22, BONE.hex(), 1.35)
	};
	let rise = Keyframes::new(
		NUMBER_MILLIS,
		[
			Keyframe::at(0.0)
				.opacity(1.0)
				.transform(leptoskit::native::Transform::scale(pop))
				.easing(Easing::EASE_OUT),
			Keyframe::at(0.18)
				.opacity(1.0)
				.transform(leptoskit::native::Transform::IDENTITY),
			Keyframe::at(0.6).opacity(1.0),
			Keyframe::at(1.0)
				.opacity(0.0)
				.transform(leptoskit::native::Transform::translate(0.0, -44.0)),
		],
	);
	// The anchor sits on the blow; the label is centred on it. At rest it
	// is unseen, so it is gone once its rise ends.
	View()
		.class(format!(
			"absolute left-[{:.1}%] top-[{:.1}%] w-0 h-0",
			number.x, number.y
		))
		.child(
			View()
				.class(
					"absolute left-[-60px] top-[-24px] w-[120px] h-[48px] items-center justify-center",
				)
				.opacity(0.0)
				.keyframes(rise)
				.child(
					Text(number.damage.to_string())
						.class(format!("text-[{size}px] font-bold text-[{colour}]")),
				),
		)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn world() -> WorldView {
		WorldView {
			scene: Key::Fixed("test-world"),
			camera: Transform::IDENTITY,
			fov_y: 62_f32.to_radians(),
			aspect: 0.5,
			husk: [0.0, -0.4, -3.0],
			ground: [0.0, -1.2, -3.0],
		}
	}

	#[test]
	fn a_point_carried_into_the_world_is_drawn_where_it_was() {
		let world = world();
		let muzzle = [0.08, -0.12, -0.5];
		let carried = carry(&world, muzzle);
		let (x, y) = project(&world, carried).unwrap();
		let tan = (VIEWMODEL_FOV * 0.5).tan();
		let expected = (
			50.0 + 50.0 * muzzle[0] / (0.5 * tan * world.aspect),
			50.0 - 50.0 * muzzle[1] / (0.5 * tan),
		);
		assert!((x - expected.0).abs() < 1.0e-3 && (y - expected.1).abs() < 1.0e-3);
		assert_eq!(project(&world, [0.0, 0.0, -2.0]), Some((50.0, 50.0)));
		assert_eq!(project(&world, [0.0, 0.0, 1.0]), None);
	}

	#[test]
	fn a_tracer_lasts_four_frames_and_shards_wait_for_their_blow() {
		let world = world();
		let mut props = ViewmodelProps::at_rest(Archetype::Gun, WeaponKind::Scythe, 100);
		props.action = ViewAction::Fire;
		let pose = arms::pose(props.manner(), props.action, 0.0, 0.0);
		assert!(tracer(&props, &world, &pose, 100.0).is_some());
		assert!(tracer(&props, &world, &pose, 107.0).is_some());
		assert!(
			tracer(&props, &world, &pose, 108.0).is_none(),
			"four frames at thirty a second are eight ticks"
		);
		let hit = HitView {
			tick: 120,
			at: world.husk,
			damage: 12,
			crit: false,
			color: [0.9, 0.2, 0.3],
		};
		assert!(shards(&hit, 119.0).is_none());
		assert!(shards(&hit, 121.0).is_some_and(|burst| {
			burst
				.color
				.iter()
				.zip(hit.color)
				.all(|(shown, own)| (shown - own).abs() < 1.0e-6)
		}));
		assert!(shards(&hit, 120.0 + 0.4 * 60.0).is_none());
	}

	#[test]
	fn the_dive_lens_follows_the_dive() {
		assert_eq!(dive(ViewAction::Idle, 1.0), 0.0);
		assert_eq!(dive(ViewAction::DiveIn, arms::DIVE_IN_SECONDS), 1.0);
		let stab = |dived| ViewAction::Stab { nth: 1, dived };
		assert_eq!(dive(stab(true), 0.0), 1.0);
		assert_eq!(
			dive(stab(false), 0.0),
			0.0,
			"a stab in arm's reach is not dived"
		);
		assert_eq!(dive(ViewAction::DiveOut, arms::DIVE_OUT_SECONDS), 0.0);
		let swing = ViewAction::Swing { dived: true };
		assert_eq!(dive(swing, arms::CONTACT), 1.0, "held in until the blow");
		assert_eq!(
			dive(swing, arms::CONTACT + arms::DIVE_OUT_SECONDS),
			0.0,
			"and back out as the rules' dive comes out"
		);
		assert_eq!(dive(ViewAction::Strike, 0.0), 0.0);
	}

	#[test]
	fn the_shown_tick_runs_with_the_display_and_never_back() {
		// A hundred and twenty frames a second over rules at sixty: half a
		// tick a frame, on through the rules' ticks.
		let mut shown = None;
		let mut ticks = Vec::new();
		for frame in 0..8 {
			let now = 100 + frame / 2;
			let next = shown_tick(shown, 1.0 / 120.0, now, TICKS_PER_SECOND);
			shown = Some(next);
			ticks.push(next);
		}
		assert!(ticks.windows(2).all(|pair| pair[1] > pair[0]), "{ticks:?}");
		let steps: Vec<f64> = ticks.windows(2).map(|pair| pair[1] - pair[0]).collect();
		assert!(
			steps[1..].iter().all(|step| (step - 0.5).abs() < 1e-9),
			"even steps: {steps:?}"
		);
		// A rules clock that ran ahead is caught up with; one that stalled is
		// waited for, never run away from.
		assert_eq!(shown_tick(Some(100.2), 1.0 / 60.0, 104, 60.0), 104.0);
		assert_eq!(shown_tick(Some(100.2), 1.0, 100, 60.0), 101.0);
		// A clock held still, as the bench's frozen frames are, shows its tick.
		assert_eq!(shown_tick(Some(55.5), 1.0 / 60.0, 12, 0.0), 12.0);
	}

	#[test]
	fn a_new_action_cuts_the_one_before_short_where_it_had_got_to() {
		let mut firing = ViewmodelProps::at_rest(Archetype::Gun, WeaponKind::Scythe, 100);
		firing.action = ViewAction::Fire;
		firing.action_tick = 94;
		let mut later = firing;
		later.now = 101;
		assert_eq!(next_cut(&firing, &later, None), None, "the same shot");
		let mut reload = later;
		reload.action = ViewAction::Reload;
		reload.action_tick = 103;
		reload.now = 103;
		let cut = next_cut(&later, &reload, None).unwrap();
		assert_eq!(cut.from, ViewAction::Fire);
		assert!((cut.played - 9.0 / 60.0).abs() < 1e-6, "{}", cut.played);
		// Shown again from earlier, as a bench scrubs back: no blend.
		assert_eq!(next_cut(&reload, &firing, Some(cut)), None);
		// The reload going on keeps blending out of the shot.
		let mut on = reload;
		on.now = 104;
		assert_eq!(next_cut(&reload, &on, Some(cut)), Some(cut));
	}

	#[test]
	fn the_arms_hold_still_through_a_hit_stop() {
		let mut props = ViewmodelProps::at_rest(Archetype::Blade, WeaponKind::Scythe, 0);
		props.action = ViewAction::Stab {
			nth: 0,
			dived: true,
		};
		props.juice.land(Jolt::Stab, 6);
		let held = props.action_seconds(8.0);
		assert_eq!(held, props.action_seconds(10.0));
		assert!(props.action_seconds(12.0) > held);
	}
}
