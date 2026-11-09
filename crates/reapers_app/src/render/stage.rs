//! A reaper standing in the dark, turning as the player drags: the Flutter
//! `ReaperStage`, on `leptoskit_3d`.
//!
//! Each place that shows a stage keeps one scene for it (see [`super::scene`]),
//! and the stage keeps what it last showed there, so a render only sends the
//! difference: a new genome loads a new rig, trying on a weapon or an outfit
//! restyles the one standing there, and nothing at all is sent when nothing
//! changed. The camera turns and the robe stirs on [`super::ticker`]'s
//! clock while the stage is on screen.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use leptoskit::prelude::*;
use leptoskit::scene3d::Event;
use leptoskit::scene3d::SceneId;
use reactive_graph::owner::on_cleanup;
use reapers_rules::Outfit;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperMotion;
use reapers_rules::Weapon;

use super::appearance::ReaperAppearance;
use super::camera::STAGE_DISTANCE;
use super::camera::stage_camera;
use super::playback::ReaperPlayback;
use super::pointer::Gesture;
use super::rig::ReaperRig;
use super::rig::stature;
use super::scene::GameScene;
use super::scene::Key;
use super::scene::light_sanctum;
use super::scene::scene;

/// How fast a stage turns by itself, in radians a second.
const TURN_SPEED: f32 = 0.25;
/// How far a drag turns it, in radians per CSS pixel.
const DRAG_TURN: f32 = 0.012;
/// How often the robe's sway is sent to the host, in ticks.
const STIR_EVERY: u32 = 2;

/// What a stage shows, beyond the seam's props: the studio's controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageShow {
	pub genome: ReaperGenome,
	pub weapon: Weapon,
	pub outfit: Outfit,
	pub appearance: ReaperAppearance,
	pub turns_on_its_own: bool,
	pub focus_face: bool,
	/// How far back the camera stands.
	pub distance: f32,
}

/// Where a stage's model has got to, for the view to show a stand-in until
/// it arrives and if it never does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Loading {
	#[default]
	Waiting,
	Shown,
	Failed,
}

/// One stage's state between renders.
pub struct Stage {
	scene: Rc<GameScene>,
	shown: Option<StageShow>,
	rig: Option<ReaperRig>,
	/// The rig's clock.
	pub playback: ReaperPlayback,
	held_face: Option<(&'static str, f32)>,
	turn: f32,
	dragging: bool,
	mounted: u32,
	ticking: bool,
	ticks: u32,
	listening: bool,
	loading: RwSignal<Loading>,
}

thread_local! {
	static STAGES: RefCell<HashMap<SceneId, Rc<RefCell<Stage>>>> = RefCell::new(HashMap::new());
}

/// The stage kept under `key`.
pub fn stage(key: Key) -> Rc<RefCell<Stage>> {
	let scene = scene(key);
	STAGES.with_borrow_mut(|stages| {
		stages
			.entry(scene.id())
			.or_insert_with(|| {
				Rc::new(RefCell::new(Stage {
					scene,
					shown: None,
					rig: None,
					playback: ReaperPlayback::default(),
					held_face: None,
					turn: 0.0,
					dragging: false,
					mounted: 0,
					ticking: false,
					ticks: 0,
					listening: false,
					loading: crate::ui::lasting(|| RwSignal::new(Loading::Waiting)),
				}))
			})
			.clone()
	})
}

impl Stage {
	pub fn id(&self) -> SceneId {
		self.scene.id()
	}

	pub fn loading(&self) -> Loading {
		self.loading.get()
	}

	/// What the stage last showed.
	pub const fn shown(&self) -> Option<StageShow> {
		self.shown
	}

	/// The expression the face is held in, if any.
	pub const fn held_face(&self) -> Option<(&'static str, f32)> {
		self.held_face
	}

	/// Shows `show`, sending only what changed since the last.
	pub fn show(&mut self, show: StageShow) {
		let ops = self.scene.ops();
		let before = self.shown;
		let fresh = before.is_none_or(|before| before.genome != show.genome);
		if fresh {
			if let Some(rig) = self.rig.take() {
				rig.remove(ops);
			} else {
				light_sanctum(ops);
			}
			self.loading.set(Loading::Waiting);
			let mut rig = ReaperRig::spawn(
				ops,
				None,
				show.genome,
				show.weapon,
				show.outfit,
				show.appearance,
			);
			rig.perform(ops, &self.playback);
			if self.held_face.is_some() {
				rig.hold_face(ops, self.held_face);
			}
			self.rig = Some(rig);
			if before.is_some() {
				self.turn = 0.0;
			}
		} else if let (Some(before), Some(rig)) = (before, self.rig.as_mut()) {
			if before.weapon != show.weapon {
				rig.wield(ops, show.weapon);
			}
			if before.outfit != show.outfit {
				rig.wear(ops, show.outfit);
			}
			if before.appearance != show.appearance {
				rig.dress(ops, show.appearance);
				rig.perform(ops, &self.playback);
			}
		}
		if show.focus_face
			&& (fresh || before.is_some_and(|before| before.focus_face != show.focus_face))
		{
			self.turn = 0.0;
		}
		self.shown = Some(show);
		if fresh || before != Some(show) {
			self.aim();
		}
		self.scene.flush();
	}

	/// Plays `motion` from its start.
	pub fn perform(&mut self, motion: ReaperMotion, play: bool) {
		self.playback.select(motion, play);
		self.replay_clock();
	}

	/// Sends the clock's clip to the host again, as it now stands.
	pub fn replay_clock(&mut self) {
		if let Some(rig) = self.rig.as_mut() {
			rig.perform(self.scene.ops(), &self.playback);
			self.scene.flush();
		}
	}

	/// Holds the face in an expression, or lets the clips move it.
	pub fn hold_face(&mut self, expression: Option<(&'static str, f32)>) {
		if self.held_face == expression {
			return;
		}
		self.held_face = expression;
		if let Some(rig) = &self.rig {
			rig.hold_face(self.scene.ops(), expression);
			self.scene.flush();
		}
	}

	fn aim(&self) {
		let Some(show) = self.shown else {
			return;
		};
		let face = show
			.focus_face
			.then(|| (show.genome.reaper_class, stature(&show.genome)));
		self.scene
			.ops()
			.push(stage_camera(self.turn, show.distance, face));
	}

	fn tick(&mut self, delta: f64) {
		let Some(show) = self.shown else {
			return;
		};
		let still = crate::platform::reduced_motion();
		if !still {
			self.playback.advance(delta);
		}
		if show.turns_on_its_own && !still && !self.dragging && !show.focus_face {
			self.turn += delta as f32 * TURN_SPEED;
			self.aim();
		}
		self.ticks = self.ticks.wrapping_add(1);
		if self.ticks.is_multiple_of(STIR_EVERY)
			&& let Some(rig) = &self.rig
		{
			rig.stir(self.scene.ops(), self.playback.seconds());
		}
		self.scene.flush();
	}

	fn gesture(&mut self, gesture: Gesture) {
		match gesture {
			Gesture::Drag { dx, .. } => {
				self.dragging = true;
				self.turn += dx * DRAG_TURN;
				self.aim();
				self.scene.flush();
			}
			Gesture::Up => self.dragging = false,
			Gesture::Down { .. } | Gesture::Tap { .. } => {}
		}
	}
}

/// Keeps `stage` ticking and listening while it is on screen: from this
/// render until the render that replaces it, or the screen goes.
fn mount(stage: &Rc<RefCell<Stage>>) {
	let mut state = stage.borrow_mut();
	state.mounted += 1;
	let id = state.id();
	if !state.listening {
		state.listening = true;
		let listener = Rc::downgrade(stage);
		super::pointer::on_gesture(id, move |gesture| {
			if let Some(stage) = listener.upgrade() {
				stage.borrow_mut().gesture(gesture);
			}
		});
		let watcher = Rc::downgrade(stage);
		let scene = state.scene.clone();
		crate::platform::spawn(async move {
			loop {
				let events = scene.events().await;
				let Some(stage) = watcher.upgrade() else {
					return;
				};
				if events.is_empty() {
					return;
				}
				let loading = stage.borrow().loading;
				for event in events {
					match event {
						Event::AssetLoaded { .. } => loading.set(Loading::Shown),
						Event::AssetFailed { message, .. } => {
							crate::platform::log(&format!("a reaper's model failed: {message}"));
							loading.set(Loading::Failed);
						}
						_ => {}
					}
				}
			}
		});
	}
	if !state.ticking {
		state.ticking = true;
		let ticking = Rc::downgrade(stage);
		super::ticker::every_frame(move |delta| {
			let Some(stage) = ticking.upgrade() else {
				return false;
			};
			let mut stage = stage.borrow_mut();
			if stage.mounted == 0 {
				stage.ticking = false;
				return false;
			}
			stage.tick(delta);
			true
		});
	}
	drop(state);
	let unmount = Rc::downgrade(stage);
	on_cleanup(send_wrapper(move || {
		if let Some(stage) = unmount.upgrade() {
			let mut stage = stage.borrow_mut();
			stage.mounted = stage.mounted.saturating_sub(1);
		}
	}));
}

/// `on_cleanup` asks for `Send`; the game runs on one thread, so a stage's
/// handle is wrapped to cross that bound without ever leaving it.
fn send_wrapper(cleanup: impl FnOnce() + 'static) -> impl FnOnce() + Send + Sync + 'static {
	let cleanup = send_wrapper::SendWrapper::new(cleanup);
	move || (cleanup.take())()
}

/// The stage under `key`, showing `show`: a `Scene3d` view filling its box
/// on transparent ground, with the reaper's silhouette standing in until the
/// model has loaded, and for good where it cannot.
pub fn stage_view(key: Key, show: StageShow) -> (Rc<RefCell<Stage>>, Element) {
	let stage = stage(key);
	stage.borrow_mut().show(show);
	mount(&stage);
	let (id, loading) = {
		let state = stage.borrow();
		(state.id(), state.loading.get())
	};
	let mut view = View()
		.class("absolute inset-0 overflow-hidden")
		.test_id("reaper-stage");
	if loading != Loading::Shown {
		view = view.child(super::silhouette::silhouette(&show.genome));
	}
	if loading != Loading::Failed {
		view = view.child(
			Scene3d(id)
				.test_id("reaper-scene")
				.class("absolute inset-0 w-full h-full")
				.aria_hidden(true),
		);
	}
	(stage, view)
}

/// The defaults the seam's props leave out.
pub const fn plain_show(
	genome: ReaperGenome,
	weapon: Weapon,
	outfit: Outfit,
	turns_on_its_own: bool,
	focus_face: bool,
) -> StageShow {
	StageShow {
		genome,
		weapon,
		outfit,
		appearance: ReaperAppearance {
			form: super::appearance::ReaperForm::Spirit,
			clothing: super::appearance::ReaperClothing::Robe,
			mantle: true,
			holding: true,
		},
		turns_on_its_own,
		focus_face,
		distance: STAGE_DISTANCE,
	}
}

#[cfg(test)]
mod tests {
	use leptoskit::scene3d::Op;
	use reapers_rules::Temper;
	use reapers_rules::WeaponKind;

	use super::*;

	fn show(seed: i64) -> StageShow {
		plain_show(
			ReaperGenome::shuffle(seed),
			Weapon::new(WeaponKind::Scythe, Temper::Shrouded),
			Outfit::Shrouded,
			false,
			false,
		)
	}

	#[test]
	fn a_stage_sends_only_what_changed() {
		let stage = stage(Key::Fixed("test-a"));
		let mut stage = stage.borrow_mut();
		stage.show(show(1));
		let first = stage.scene.sent().len();
		assert!(first > 20, "a fresh stage loads, dresses and lights a rig");
		stage.show(show(1));
		assert_eq!(
			stage.scene.sent().len(),
			first,
			"nothing changed, nothing sent"
		);
		let mut armed = show(1);
		armed.weapon = Weapon::new(WeaponKind::Spear, Temper::Scorched);
		stage.show(armed);
		let sent = stage.scene.sent();
		assert!(
			!sent[first..]
				.iter()
				.any(|op| matches!(op, Op::LoadAsset { .. })),
			"trying on a weapon never reloads the reaper"
		);
		stage.show(show(2));
		let sent = stage.scene.sent();
		assert!(
			sent.iter()
				.filter(|op| matches!(op, Op::LoadAsset { .. }))
				.count()
				>= 3
		);
		assert!(sent.iter().any(|op| matches!(op, Op::Remove { .. })));
	}

	#[test]
	fn the_face_camera_comes_close_and_a_drag_turns_the_stage() {
		let stage = stage(Key::Fixed("test-b"));
		let mut stage = stage.borrow_mut();
		let mut close = show(3);
		close.focus_face = true;
		stage.show(close);
		let camera = |stage: &Stage| {
			stage
				.scene
				.sent()
				.into_iter()
				.rev()
				.find_map(|op| {
					match op {
						Op::SetCamera { transform, .. } => Some(transform.translation),
						_ => None,
					}
				})
				.unwrap()
		};
		assert!(camera(&stage)[2] < 2.0);
		stage.show(show(3));
		assert!(camera(&stage)[2] > 8.0);
		let mut gestures = super::super::pointer::NativeGestures::default();
		for (phase, dx) in [
			(leptoskit::scene3d::PointerPhase::Down, 0.0),
			(leptoskit::scene3d::PointerPhase::Move, 100.0),
		] {
			let input = leptoskit::scene3d::PointerInput {
				phase,
				x: 0.5,
				y: 0.5,
				dx,
				dy: 0.0,
			};
			for gesture in gestures.accept(input).into_iter().flatten() {
				stage.gesture(gesture);
			}
		}
		assert!(camera(&stage)[0] > 1.0, "the camera has gone round");
		assert!(stage.dragging);
		stage.gesture(Gesture::Up);
		assert!(!stage.dragging);
	}

	#[test]
	fn a_held_face_outlasts_a_new_reaper() {
		let stage = stage(Key::Fixed("test-d"));
		let mut stage = stage.borrow_mut();
		stage.show(show(7));
		stage.hold_face(Some(("joy", 1.0)));
		assert_eq!(stage.held_face(), Some(("joy", 1.0)));
		let before = stage.scene.sent().len();
		stage.show(show(8));
		assert_eq!(
			stage.shown().map(|shown| shown.genome),
			Some(show(8).genome)
		);
		assert!(
			stage.scene.sent()[before..].iter().any(
				|op| matches!(op, Op::SetMorphWeight { target, weight, .. } if target == "joy" && *weight > 0.9)
			),
			"the new rig wears the held face"
		);
	}

	#[test]
	fn a_ticking_stage_turns_by_itself_and_stirs_its_robe() {
		let stage = stage(Key::Fixed("test-c"));
		let mut stage = stage.borrow_mut();
		let mut turning = show(5);
		turning.turns_on_its_own = true;
		stage.show(turning);
		let before = stage.scene.sent().len();
		stage.tick(0.5);
		stage.tick(0.5);
		let sent = &stage.scene.sent()[before..];
		assert!(sent.iter().any(|op| matches!(op, Op::SetCamera { .. })));
		assert!(
			sent.iter()
				.any(|op| matches!(op, Op::SetMorphWeight { target, .. } if target == "sway_x"))
		);
		assert!((stage.turn - 0.25).abs() < 1e-6);
	}
}
