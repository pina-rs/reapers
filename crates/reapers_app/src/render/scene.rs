//! The game's 3D scenes: who holds which, and getting their ops to the host.
//!
//! A scene lives on the host (the page's renderer, or the native app's) for
//! as long as the game runs; a screen only shows it, in a `Scene3d` view
//! with its id. Screens here are drawn afresh whenever what they read
//! changes, so a scene is never made by a render: each place that shows one
//! names its slot, and the slot keeps its scene from one render to the next.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use leptoskit::scene3d::Backdrop;
use leptoskit::scene3d::Event;
use leptoskit::scene3d::Light;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::SceneId;
use leptoskit::scene3d::client::Scene;

/// A scene the game keeps, and the ops it has sent (in tests, where there is
/// no host to send them to).
pub struct GameScene {
	scene: Scene,
	shot_target: RefCell<Option<(i64, super::camera::Point)>>,
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	sent: RefCell<Vec<Op>>,
}

impl GameScene {
	fn new(id: SceneId) -> Self {
		#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
		{
			// Android can restart the guest inside the same process. Its
			// host still owns the old IDs, assets and loading-event queues.
			// close sends immediately, before this scene's first batch.
			let closed = Scene::new(id).close();
			crate::platform::spawn(async move {
				let _ = closed.await;
			});
		}
		Self {
			scene: Scene::new(id),
			shot_target: RefCell::new(None),
			#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
			sent: RefCell::new(Vec::new()),
		}
	}

	pub fn id(&self) -> SceneId {
		self.scene.id()
	}

	/// The scene, to queue ops on.
	pub fn ops(&self) -> &Scene {
		&self.scene
	}

	/// The real surface a missed shot met, for its tracer to stop there.
	pub fn set_shot_target(&self, tick: i64, at: super::camera::Point) {
		*self.shot_target.borrow_mut() = Some((tick, at));
	}

	pub fn shot_target(&self, tick: i64) -> Option<super::camera::Point> {
		self.shot_target
			.borrow()
			.filter(|(shot, _)| *shot == tick)
			.map(|(_, at)| at)
	}

	/// Sends what is queued. The host applies batches in the order they were
	/// sent; a failure is logged, never shown: a scene that cannot draw
	/// leaves its view empty.
	pub fn flush(&self) {
		if self.scene.pending() == 0 {
			return;
		}
		#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
		{
			let sent = self.scene.flush();
			crate::platform::spawn(async move {
				if let Err(error) = sent.await {
					crate::platform::log(&format!("a scene's ops were not taken: {error}"));
				}
			});
		}
		#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
		self.sent.borrow_mut().extend(self.scene.take_ops());
	}

	/// Waits for what the host has to say about the scene.
	#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
	pub async fn events(&self) -> Vec<Event> {
		self.scene.next_events().await.unwrap_or_default()
	}

	/// Without a host there is nothing to say.
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	pub fn events(&self) -> std::future::Ready<Vec<Event>> {
		std::future::ready(Vec::new())
	}

	/// Every op sent so far, for tests.
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	pub fn sent(&self) -> Vec<Op> {
		self.sent.borrow().clone()
	}
}

/// What a scene is kept under: a place in the game that shows one, named,
/// or the line of code that shows it. Each keeps its own scene, so the host
/// holds as many as there are such places (it allows sixteen).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
	Fixed(&'static str),
	Caller(&'static str, u32),
}

impl Key {
	/// The line that called the function this is made in.
	#[track_caller]
	pub fn caller() -> Self {
		let location = std::panic::Location::caller();
		Self::Caller(location.file(), location.line())
	}
}

/// The hunt's scene, the rift's and the benches'.
pub const HUNT: Key = Key::Fixed("hunt");
pub const RIFT: Key = Key::Fixed("rift");
pub const STUDIO: Key = Key::Fixed("studio");
pub const PROBE: Key = Key::Fixed("probe");

thread_local! {
	static SCENES: RefCell<HashMap<Key, Rc<GameScene>>> = RefCell::new(HashMap::new());
	static NEXT_ID: std::cell::Cell<u32> = const { std::cell::Cell::new(1) };
}

/// The scene kept under `key`, made on first use.
pub fn scene(key: Key) -> Rc<GameScene> {
	SCENES.with_borrow_mut(|scenes| {
		scenes
			.entry(key)
			.or_insert_with(|| {
				let id = NEXT_ID.with(|next| {
					let id = next.get();
					next.set(id + 1);
					id
				});
				Rc::new(GameScene::new(SceneId(id)))
			})
			.clone()
	})
}

/// Forgets `key`'s scene, so the next use starts afresh with a new id, and
/// closes the host's copy.
pub fn forget(key: Key) {
	let Some(scene) = SCENES.with_borrow_mut(|scenes| scenes.remove(&key)) else {
		return;
	};
	#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
	if let Ok(scene) = Rc::try_unwrap(scene) {
		let closed = scene.scene.close();
		crate::platform::spawn(async move {
			let _ = closed.await;
		});
	}
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	drop(scene);
}

/// The light a reaper stands in alone in the dark: a cold key from above
/// and in front, which leaves the face in the shadow of whatever the reaper
/// wears on its head, and a dim sky. The Flutter `VeilLook.sanctum`.
pub fn light_sanctum(scene: &Scene) {
	scene.push(Op::SetLight(Light {
		direction: [-0.35, -1.0, -0.55],
		color: [0.82, 0.9, 1.0],
		intensity: 4.6,
		ambient: 0.55,
		shadow: 0.55,
	}));
	scene.push(Op::SetBackdrop(Backdrop::Transparent));
	scene.push(Op::SetExposure(1.3));
}

/// The light for husks over the camera's picture, or the veil where there is
/// none: nothing here may darken or tint the whole frame, because the
/// background is the real world. The Flutter `VeilLook.hunt`.
pub fn light_hunt(scene: &Scene, camera: bool) {
	scene.push(Op::SetLight(Light {
		direction: [0.3, -1.0, -0.4],
		color: [0.8, 0.9, 1.0],
		// Kept low enough that a pale coat still shows its shape.
		intensity: 1.7,
		ambient: 0.6,
		shadow: 0.6,
	}));
	scene.push(Op::SetBackdrop(if camera {
		Backdrop::Camera
	} else {
		Backdrop::Transparent
	}));
	scene.push(Op::SetExposure(1.0));
}
