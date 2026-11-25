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
//! everything from a fixed eye, so they never reach into the room. The orb
//! and the soul-shard sparks of a blade's or a class weapon's blow belong to
//! the room, so they go in the world's scene, the one the husk is in, named
//! by [`WorldView`]. The pistol throws no light into the room: its shot and
//! its blow are printed flat over the view ([`viewmodel_marks`]).
//!
//! The shake turns the arms here. On the world it is the owner's to apply,
//! with [`shake_turn`], because the owner holds the world's camera. Neither
//! ever moves the camera's picture.

use std::cell::RefCell;
use std::rc::Rc;

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

use crate::hunt::blow;
use crate::hunt::blow::TagLook;
use crate::hunt::marks;
use crate::hunt::overlay::Sight;
use crate::ink;
use crate::render::arms;
use crate::render::arms::TICKS_PER_SECOND;
pub use crate::render::arms::ViewAction;
use crate::render::camera::Point;
use crate::render::curves::Ease;
use crate::render::effects::Burst;
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
use crate::render::turn::rotate;
use crate::render::turn::sub;
use crate::render::turn::view_turn;
use crate::render::viewmodel::MUZZLE;
use crate::render::viewmodel::VIEWMODEL_FOV;
use crate::render::viewmodel::ViewmodelRig;
use crate::render::viewmodel::place;
use crate::theme::Rgb;

/// The arms' own scene.
pub const VIEWMODEL: Key = Key::Fixed("viewmodel");

/// How long a burst of shards flies, in seconds.
const SHARD_SECONDS: f32 = 0.4;

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
	/// The view's height in layout points, for the marks printed over it.
	pub height: f32,
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
	/// Whether it struck an open seal, and the hits in a row it is, counting
	/// itself.
	pub seal: bool,
	pub cadence: i64,
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
	/// The reaper's spot, the second ink its marks are printed in.
	pub spot: Rgb,
	/// The sight at the middle of the view, while there is one: what a press
	/// would do now.
	pub sight: Option<Sight>,
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
			spot: ink::CHAFF,
			sight: None,
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

/// A blow that landed, as its marks remember it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Landed {
	hit: HitView,
	/// Where it struck from the husk's middle, in the scene: its splat goes
	/// where the husk goes.
	from_husk: Point,
	/// The way the shot crossed the screen, as a unit vector.
	toward: marks::At,
	/// Where the middle of its tag stands before it rises, and the slip's
	/// size.
	tag: marks::At,
	slip: marks::At,
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
	/// The latest blows that landed, oldest first, while their marks show.
	landed: Vec<Landed>,
	/// The tags the marks' layer shows.
	tags: RwSignal<Vec<TagLook>>,
	/// What was last printed over the view, and the print the marks' layer
	/// shows, numbered so that a changed print replaces the last.
	print: Option<marks::Print>,
	prints: u64,
	printed: RwSignal<Vec<(u64, marks::Print)>>,
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
				landed: Vec::new(),
				tags: crate::ui::lasting(|| RwSignal::new(Vec::new())),
				print: None,
				prints: 0,
				printed: crate::ui::lasting(|| RwSignal::new(Vec::new())),
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
		let untrailed = pose;
		// The arms' own clock over the frame, which a hit-stop holds.
		let before = tick - f64::from(seconds) * f64::from(props.rate);
		let moved = ((props.juice.motion_tick(tick) - props.juice.motion_tick(before))
			/ f64::from(TICKS_PER_SECOND))
		.max(0.0) as f32;
		self.trailing
			.follow(&mut pose, props.turn_rate, seconds, moved, lively);
		let (print, tags) = match props.world {
			Some(world) => {
				let shot = Shot {
					props: &props,
					since,
					clock,
					cut: self.cut,
					lively,
				};
				let mut print = shot.print(&world, shown, &untrailed, &pose);
				self.land(&props, &world, shown, &pose, print.view, lively);
				let tags = self.blows(&mut print, &props, &world, shown, lively);
				(Some(print), tags)
			}
			None => (None, Vec::new()),
		};
		self.show_print(print);
		self.show_tags(tags);
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
			Some(world) => self.show_world(&props, &world, tick, seconds),
			None => self.hide_world(),
		}
	}

	/// Shows the orb and the shards in the world.
	fn show_world(&mut self, props: &ViewmodelProps, world: &WorldView, tick: f64, seconds: f32) {
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
			self.landed.clear();
		}
		let mut frame = WorldFrame {
			eye: world.camera.translation,
			..WorldFrame::default()
		};
		if let Some(orb) = props.orb {
			frame.orb = Some(self.orb_look(&orb, world, tick));
		}
		// The pistol's blows throw no shards in the husk's light: they are
		// printed in the reaper's spot instead, one ink (art bible §3.2).
		if let Some(hit) = props.hit
			&& props.archetype != Archetype::Gun
			&& self.shards[0].is_none_or(|latest| latest.tick != hit.tick)
		{
			self.shards = [Some(hit), self.shards[0]];
		}
		for (slot, shard) in self.shards.iter().enumerate() {
			frame.sparks[slot] = shard.and_then(|hit| shards(&hit, tick));
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

	/// Remembers the latest blow once it has landed, by the hunt's tick
	/// `shown`, with the arms at `pose`, on a view `view` points big: where it
	/// struck the husk, the way the shot crossed the screen, and where its tag
	/// stands. A blow soon after the last stacks its tag over the last's,
	/// where that has risen to unless the player asks for less motion
	/// (`lively` false). Blows whose marks have all gone are forgotten, and no
	/// more than [`blow::MOST_TAGS`] are kept: a fourth sends the oldest at
	/// once.
	fn land(
		&mut self,
		props: &ViewmodelProps,
		world: &WorldView,
		shown: f64,
		pose: &arms::Pose,
		view: marks::At,
		lively: bool,
	) {
		let showing = |tick: i64| blow::beat(marks::frames_since(shown, tick)) < blow::TAG_GONE;
		self.landed.retain(|landed| showing(landed.hit.tick));
		let Some(hit) = props.hit else {
			return;
		};
		if marks::frames_since(shown, hit.tick) < 0
			|| !showing(hit.tick)
			|| self.landed.iter().any(|landed| landed.hit.tick == hit.tick)
		{
			return;
		}
		let Some(at) = screen_point(world, hit.at, view) else {
			return;
		};
		// The shot crossed the screen from the muzzle; anything else is
		// taken to come up from the hand, low and to the right.
		let toward = on_screen(place(props.archetype, pose).muzzle, view)
			.map(|from| (at.0 - from.0, at.1 - from.1))
			.filter(|(across, down)| across.hypot(*down) > 1.0)
			.map_or((-0.6, -0.8), |(across, down)| {
				let length = across.hypot(down);
				(across / length, down / length)
			});
		let figure = crate::app::messages().hunt_hit_figure(hit.damage);
		let slip = blow::slip_size(figure.chars().count(), blow::figure_size(hit.cadence));
		let tag = match self.landed.last() {
			Some(last) if hit.tick - last.hit.tick < blow::STACK_BEATS => {
				let beat = blow::beat(marks::frames_since(shown, last.hit.tick));
				let risen = blow::tag(beat, lively).map_or(0.0, |(_, rise)| rise);
				blow::keep_clear((last.tag.0, last.tag.1 - risen - blow::STACK), view, slip)
			}
			_ => blow::tag_place(at, hit.tick, view, slip),
		};
		self.landed.push(Landed {
			hit,
			from_husk: sub(hit.at, world.husk),
			toward,
			tag,
			slip,
		});
		if self.landed.len() > blow::MOST_TAGS {
			self.landed.remove(0);
		}
	}

	/// Adds the splats of the blows that landed to `print`, with the sight
	/// stepped out for the latest, and returns their tags, at the hunt's tick
	/// `shown`. A splat goes where the husk goes; a tag stays where it was
	/// put, and rises, unless the player asks for less motion (`lively`
	/// false).
	fn blows(
		&self,
		print: &mut marks::Print,
		props: &ViewmodelProps,
		world: &WorldView,
		shown: f64,
		lively: bool,
	) -> Vec<TagLook> {
		let words = crate::app::messages();
		let mut out = 0.0_f64;
		let mut tags = Vec::new();
		for landed in &self.landed {
			let hit = landed.hit;
			let beat = blow::beat(marks::frames_since(shown, hit.tick));
			out = out.max(blow::sight_out(beat));
			if blow::coverage(beat, lively) > 0.0
				&& let Some(at) = screen_point(world, add(world.husk, landed.from_husk), print.view)
			{
				print.splats.push(blow::Splat {
					at,
					radius: if hit.seal {
						blow::SEAL_SPLAT
					} else {
						blow::BODY_SPLAT
					},
					toward: landed.toward,
					seed: hit.tick,
					beat,
					lively,
				});
			}
			if let Some((scale, rise)) = blow::tag(beat, lively) {
				tags.push(TagLook {
					tick: hit.tick,
					centre: (landed.tag.0, landed.tag.1 - rise),
					slip: landed.slip,
					turn: blow::tag_turn(hit.tick),
					scale,
					figure: words.hunt_hit_figure(hit.damage),
					size: blow::figure_size(hit.cadence),
					chip: hit.seal.then(|| words.hunt_hit_seal_chip().to_owned()),
					spot: props.spot,
				});
			}
		}
		print.sight = props.sight.map(|sight| (sight, out));
		// Only the newest tag keeps its chip: the next stacks over the last's
		// slip, and a chip left showing under it would read as its own.
		if let Some((_, older)) = tags.split_last_mut() {
			for tag in older {
				tag.chip = None;
			}
		}
		tags
	}

	/// Hands the marks' layer `tags`, if they differ from what it shows.
	fn show_tags(&self, tags: Vec<TagLook>) {
		if self.tags.with_untracked(|shown| *shown == tags) {
			return;
		}
		self.tags.set(tags);
	}

	/// Hands the marks' layer `print`, if it differs from what it shows.
	fn show_print(&mut self, print: Option<marks::Print>) {
		let print = print.filter(|print| !print.is_blank());
		if self.print == print {
			return;
		}
		self.print.clone_from(&print);
		self.prints += 1;
		let number = self.prints;
		self.printed
			.set(print.map(|print| (number, print)).into_iter().collect());
	}

	/// Hides every effect in the world.
	fn hide_world(&mut self) {
		self.shards = [None; 2];
		self.thrown = None;
		self.landed.clear();
		self.show_print(None);
		self.show_tags(Vec::new());
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

/// Where `point`, in the world's scene, is drawn on a view `view` points
/// wide and high, or `None` behind the eye.
fn screen_point(world: &WorldView, point: Point, view: marks::At) -> Option<marks::At> {
	project(world, point)
		.map(|(x, y)| (f64::from(x) / 100.0 * view.0, f64::from(y) / 100.0 * view.1))
}

/// Where `point`, in the arms' view, is drawn on a view `view` points wide
/// and high, or `None` behind the eye.
fn on_screen(point: Point, view: marks::At) -> Option<marks::At> {
	let depth = f64::from(-point[2]);
	if depth < 0.01 {
		return None;
	}
	let focal = view.1 / 2.0 / f64::from((VIEWMODEL_FOV * 0.5).tan());
	Some((
		view.0 / 2.0 + focal * f64::from(point[0]) / depth,
		view.1 / 2.0 - focal * f64::from(point[1]) / depth,
	))
}

/// A shot, as the arms show it on one frame: what its marks are worked out
/// from.
struct Shot<'a> {
	props: &'a ViewmodelProps,
	/// Seconds into the shot, and the arms' own clock, as the frame shows them.
	since: f32,
	clock: f32,
	/// What the shot cut short.
	cut: Option<arms::Cut>,
	/// Whether the arms move freely: not for a player who asks for less
	/// motion.
	lively: bool,
}

impl Shot<'_> {
	/// What is printed over `world` at the hunt's tick `shown`, with the arms
	/// at `pose`, which trails the view's turn, and at `untrailed` before it
	/// did: the pistol's burst at its muzzle, and the smear of its kick from
	/// where the muzzle was the three frames before.
	fn print(
		&self,
		world: &WorldView,
		shown: f64,
		untrailed: &arms::Pose,
		pose: &arms::Pose,
	) -> marks::Print {
		let props = self.props;
		let view = (
			f64::from(world.height * world.aspect),
			f64::from(world.height),
		);
		let mut print = marks::Print::blank(view, props.spot);
		if props.archetype != Archetype::Gun || props.action != ViewAction::Fire {
			return print;
		}
		let frame = marks::frames_since(shown, props.action_tick);
		let muzzle = |pose: &arms::Pose| on_screen(place(props.archetype, pose).muzzle, view);
		let Some(now) = muzzle(pose) else {
			return print;
		};
		if marks::burst_shows(frame, self.lively) {
			print.burst = Some((
				burst_at(props.archetype, pose, view).unwrap_or(now),
				props.action_tick,
			));
		}
		if marks::smear_shows(frame, self.lively) {
			// Where the muzzle was each frame before, without the trail, moved
			// as far as the trail moves it now.
			let Some(still) = muzzle(untrailed) else {
				return print;
			};
			let trail = (now.0 - still.0, now.1 - still.1);
			let mut points = [now; 4];
			for (back, point) in points.iter_mut().enumerate().skip(1) {
				let ago = back as f32 / TICKS_PER_SECOND;
				let then = arms::flowing(
					props.manner(),
					props.action,
					(self.since - ago).max(0.0),
					self.clock - ago,
					self.cut,
					self.lively,
				);
				if let Some(at) = muzzle(&then) {
					*point = (at.0 + trail.0, at.1 + trail.1);
				}
			}
			print.smear = Some(points);
		}
		print
	}
}

/// Where a shot's burst is printed with the arms at `pose`, on a view `view`
/// points big: [`marks::BURST_AHEAD`] past the pistol's muzzle, along the
/// barrel as it is drawn. `None` behind the eye.
fn burst_at(archetype: Archetype, pose: &arms::Pose, view: marks::At) -> Option<marks::At> {
	let placed = place(archetype, pose);
	let muzzle = on_screen(placed.muzzle, view)?;
	let further = on_screen(placed.weapon.point(add(MUZZLE, [0.0, 0.0, 0.05])), view)?;
	let (across, down) = (further.0 - muzzle.0, further.1 - muzzle.1);
	let length = across.hypot(down);
	if length < 1.0 {
		return Some(muzzle);
	}
	Some((
		muzzle.0 + across / length * marks::BURST_AHEAD,
		muzzle.1 + down / length * marks::BURST_AHEAD,
	))
}

/// The marks the reaper's blows print flat over the view: a box filling its
/// parent, over the room's marks and under the thumbs' controls, that takes
/// no touches. Every frame's marks are worked out by the seam that
/// [`viewmodel_view`] keeps ticking.
pub fn viewmodel_marks() -> Element {
	let (printed, tags) = {
		let seam = seam();
		let seam = seam.borrow();
		(seam.printed, seam.tags)
	};
	View()
		.class("absolute inset-0 overflow-hidden")
		.test_id("viewmodel-marks")
		.aria_hidden(true)
		.child(
			For(
				move || printed.get(),
				|(number, _)| *number,
				|(_, print)| marks::print_view(&print),
			)
			.class("absolute inset-0")
			.aria_hidden(true),
		)
		// A blow's tag over its splat: each is placed, turned and scaled by
		// itself, and made afresh as it moves.
		.child(
			For(
				move || tags.get(),
				|tag| {
					(
						tag.tick,
						(tag.centre.1 * 4.0).round() as i64,
						(tag.scale * 100.0).round() as i64,
					)
				},
				blow::tag_view,
			)
			.class("absolute inset-0")
			.aria_hidden(true),
		)
}

/// The arms over whatever is under them: a box filling its parent, that
/// takes no touches.
pub fn viewmodel_view(props: &ViewmodelProps) -> Element {
	let seam = seam();
	viewmodel_show(props);
	mount(&seam);
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
			height: 800.0,
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
	fn shards_wait_for_their_blow() {
		let world = world();
		let hit = HitView {
			tick: 120,
			at: world.husk,
			damage: 12,
			crit: false,
			seal: false,
			cadence: 1,
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

	/// The pistol's blow is printed in the reaper's spot and nothing else:
	/// it throws no shards in the husk's light into the room, as the
	/// blade's and the class weapon's still do.
	#[test]
	fn a_pistol_blow_throws_no_shards() {
		let (mut props, world) = seal_shot();
		let seam = seam();
		let mut seam = seam.borrow_mut();
		seam.shards = [None; 2];
		seam.show_world(&props, &world, 101.0, 1.0 / 60.0);
		assert_eq!(seam.shards, [None; 2], "a shot's blow");
		props.archetype = Archetype::Blade;
		props.action = ViewAction::Stab {
			nth: 0,
			dived: true,
		};
		seam.show_world(&props, &world, 101.0, 1.0 / 60.0);
		assert_eq!(seam.shards[0], props.hit, "a stab's blow");
		seam.hide_world();
	}

	/// What a shot on tick 100 prints at the hunt's tick `shown`, on a phone
	/// 390 × 844 points, with the arms moving freely or not.
	fn shot_print(archetype: Archetype, shown: f64, lively: bool) -> marks::Print {
		let mut world = world();
		world.height = 844.0;
		world.aspect = 390.0 / 844.0;
		let mut props =
			ViewmodelProps::at_rest(archetype, WeaponKind::Scythe, shown.floor() as i64);
		props.action = ViewAction::Fire;
		props.action_tick = 100;
		let since = props.action_seconds(shown);
		let pose = arms::flowing(props.manner(), props.action, since, since, None, lively);
		let shot = Shot {
			props: &props,
			since,
			clock: since,
			cut: None,
			lively,
		};
		shot.print(&world, shown, &pose, &pose)
	}

	#[test]
	fn a_shot_prints_its_burst_for_two_frames_and_its_smear_for_one() {
		assert!(
			shot_print(Archetype::Gun, 100.0, true).is_blank(),
			"not yet heard"
		);
		// Its first frame, at sixty frames a second and at a hundred and twenty.
		for shown in [101.0, 101.5] {
			let first = shot_print(Archetype::Gun, shown, true);
			assert!(
				first.burst.is_some() && first.smear.is_some(),
				"f0 at {shown}"
			);
		}
		let second = shot_print(Archetype::Gun, 102.0, true);
		assert!(
			second.burst.is_some() && second.smear.is_none(),
			"f1: the burst alone"
		);
		assert!(
			shot_print(Archetype::Gun, 103.0, true).is_blank(),
			"f2: gone"
		);
		// The burst is low and right of the sight, past the muzzle along the
		// barrel as it is drawn, so the star clears the barrel's mouth. The
		// smear runs from the muzzle back down the kick.
		let first = shot_print(Archetype::Gun, 101.0, true);
		let (burst, seed) = first.burst.unwrap();
		assert_eq!(seed, 100, "turned the same way each frame of the shot");
		assert!(
			burst.0 > 215.0 && burst.1 > 422.0 + marks::CLEAR,
			"{burst:?}"
		);
		let smear = first.smear.unwrap();
		let muzzle = smear[0];
		let ahead = (burst.0 - muzzle.0).hypot(burst.1 - muzzle.1);
		assert!((ahead - marks::BURST_AHEAD).abs() < 1.0e-9, "{ahead} pt");
		let mut props = ViewmodelProps::at_rest(Archetype::Gun, WeaponKind::Scythe, 101);
		props.action = ViewAction::Fire;
		props.action_tick = 100;
		let since = props.action_seconds(101.0);
		let pose = arms::flowing(props.manner(), props.action, since, since, None, true);
		let breech = place(Archetype::Gun, &pose)
			.weapon
			.point([0.0, MUZZLE[1], 0.064]);
		let breech = on_screen(breech, (390.0, 844.0)).unwrap();
		let barrel = (muzzle.0 - breech.0, muzzle.1 - breech.1);
		let along = (barrel.0 * (burst.0 - muzzle.0) + barrel.1 * (burst.1 - muzzle.1))
			/ (barrel.0.hypot(barrel.1) * ahead);
		assert!(along > 0.999, "{burst:?} along the barrel from {muzzle:?}");
		assert!(smear[3].1 > smear[0].1, "the muzzle rose: {smear:?}");
		// Only the pistol fires so.
		assert!(shot_print(Archetype::Blade, 101.0, true).is_blank());
	}

	/// A seal shot on tick 100 for 27, its pistol at rest, on a phone.
	fn seal_shot() -> (ViewmodelProps, WorldView) {
		let mut world = world();
		world.height = 844.0;
		world.aspect = 390.0 / 844.0;
		let mut props = ViewmodelProps::at_rest(Archetype::Gun, WeaponKind::Scythe, 100);
		props.action = ViewAction::Fire;
		props.action_tick = 100;
		props.world = Some(world);
		props.sight = Some(Sight::Seal);
		props.hit = Some(HitView {
			tick: 100,
			at: world.husk,
			damage: 27,
			crit: true,
			seal: true,
			cadence: 1,
			color: [0.9, 0.2, 0.3],
		});
		(props, world)
	}

	/// What the seam prints for `props` at the hunt's tick `shown`: the
	/// print, and the tags.
	fn printed_at(
		seam: &mut Seam,
		props: &ViewmodelProps,
		world: &WorldView,
		shown: f64,
		lively: bool,
	) -> (marks::Print, Vec<TagLook>) {
		let pose = arms::flowing(props.manner(), ViewAction::Idle, 0.0, 0.0, None, true);
		let mut print = marks::Print::blank((390.0, 844.0), props.spot);
		seam.land(props, world, shown, &pose, print.view, lively);
		let tags = seam.blows(&mut print, props, world, shown, lively);
		(print, tags)
	}

	#[test]
	fn a_blow_keeps_to_the_hit_table_on_the_frame_its_sound_plays() {
		let (props, world) = seal_shot();
		let seam = seam();
		let mut seam = seam.borrow_mut();
		seam.landed.clear();
		let mut splats = Vec::new();
		let mut out = Vec::new();
		let mut tags = Vec::new();
		for shown in 100..=152 {
			let (print, tagged) = printed_at(&mut seam, &props, &world, f64::from(shown), true);
			if let Some(splat) = print.splats.first() {
				splats.push((shown, splat.beat, splat.radius));
			}
			if print.sight.is_some_and(|(_, stepped)| stepped > 0.0) {
				out.push(shown);
			}
			if let Some(tag) = tagged.first() {
				tags.push((shown, tag.scale, tag.centre.1));
			}
		}
		// The blow's tick is heard on tick 101, the shot's first frame, with
		// the impact's sound: the splat lands then, as the bible's f1.
		assert_eq!(splats.first(), Some(&(101, 1, blow::SEAL_SPLAT)));
		assert_eq!(
			splats.last().map(|splat| splat.0),
			Some(133),
			"wiped by f34"
		);
		assert_eq!(out, [101, 102], "the sight steps out on f1 and f2");
		assert_eq!(
			tags.first().map(|tag| (tag.0, tag.1)),
			Some((102, 1.15)),
			"f2"
		);
		assert_eq!(tags[2].1, 1.0, "f4");
		assert_eq!(tags.last().map(|tag| tag.0), Some(150), "gone at 850 ms");
		assert!(tags[30].2 < tags[2].2 - 17.0, "it has risen: {tags:?}");
		assert!(seam.landed.is_empty(), "and forgotten");
		// Its tag says what it did.
		let (_, tagged) = printed_at(&mut seam, &props, &world, 105.0, true);
		let tag = &tagged[0];
		assert_eq!(tag.figure, "−27");
		assert_eq!(tag.size, 40);
		assert_eq!(tag.chip.as_deref(), Some("Seal"));
	}

	#[test]
	fn with_less_motion_a_shot_bursts_once_and_its_marks_hold_still_then_go() {
		// The burst is one frame and there is no smear.
		let first = shot_print(Archetype::Gun, 101.0, false);
		assert!(first.burst.is_some() && first.smear.is_none());
		assert!(shot_print(Archetype::Gun, 102.0, false).is_blank());
		// The splat, scrawl and tag appear in place, held, and go in one step;
		// the tag neither pops, rises nor shrinks.
		let (props, world) = seal_shot();
		let seam = seam();
		let mut seam = seam.borrow_mut();
		seam.landed.clear();
		let mut coverage = Vec::new();
		let mut tags = Vec::new();
		for shown in 100..=152 {
			let (print, tagged) = printed_at(&mut seam, &props, &world, f64::from(shown), false);
			if let Some(splat) = print.splats.first() {
				assert!(!splat.lively);
				coverage.push(blow::coverage(splat.beat, splat.lively));
			}
			tags.extend(tagged.first().map(|tag| (tag.scale, tag.centre)));
		}
		assert_eq!(coverage, [1.0; 33], "f1 to f33, whole");
		assert_eq!(tags.len(), 49, "f2 to f50");
		assert!(tags.iter().all(|tag| *tag == tags[0]), "held still");
		assert_eq!(tags[0].0, 1.0, "at its size");
	}

	#[test]
	fn a_quick_second_blow_stacks_its_tag_and_a_fourth_sends_the_first() {
		let (mut props, world) = seal_shot();
		let seam = seam();
		let mut seam = seam.borrow_mut();
		seam.landed.clear();
		printed_at(&mut seam, &props, &world, 101.0, true);
		let mut next = props.hit.unwrap();
		next.tick = 120;
		props.hit = Some(next);
		let (_, tags) = printed_at(&mut seam, &props, &world, 121.0, true);
		let first = seam.landed[0].tag;
		let second = seam.landed[1].tag;
		assert_eq!(tags.len(), 1, "the second's tag waits for f2");
		// It stacks over the first's tag where that has risen to, not where
		// it was put, which would print it over the first's figure.
		let risen = tags[0].centre.1;
		assert!(first.1 - risen > 17.0, "the first has risen: {risen}");
		assert!(
			(second.1 - (risen - blow::STACK)).abs() < 1.0e-9,
			"{risen} {second:?}"
		);
		// Once the second's tag shows, the seal's chip is on it alone.
		let (_, tags) = printed_at(&mut seam, &props, &world, 122.0, true);
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].chip, None, "the first gives its chip up");
		assert!(tags[1].chip.is_some());
		for tick in [130, 140] {
			next.tick = tick;
			props.hit = Some(next);
			printed_at(&mut seam, &props, &world, tick as f64 + 1.0, true);
		}
		assert_eq!(seam.landed.len(), blow::MOST_TAGS);
		assert_eq!(seam.landed[0].hit.tick, 120, "the oldest left at once");
		// A miss lands nothing.
		seam.landed.clear();
		props.hit = None;
		let (print, tags) = printed_at(&mut seam, &props, &world, 101.0, true);
		assert!(print.splats.is_empty() && tags.is_empty());
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
