//! Fingers and mice on a scene's view.
//!
//! leptoskit's views hear a press and nothing finer, so the browser build
//! listens on the page itself and hands drags and taps to whichever scene's
//! canvas they land on, even under the words laid over it. Native hosts send
//! their scene view's touches through a separate stream from model events.

use leptoskit::scene3d::SceneId;

/// What a finger did on a scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
	/// Moved while down, by this many layout units (CSS pixels, dp or points).
	Drag { dx: f32, dy: f32 },
	/// Went down: where, from 0 to 1 across and down the view.
	Down { x: f32, y: f32 },
	/// Came up without having moved far: a tap at the place it went down.
	Tap { x: f32, y: f32 },
	/// Came up, after a drag or a tap.
	Up,
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod web {
	use std::cell::RefCell;
	use std::collections::HashMap;
	use std::rc::Rc;

	use leptoskit::scene3d::SceneId;
	use wasm_bindgen::JsCast as _;
	use wasm_bindgen::convert::FromWasmAbi;
	use wasm_bindgen::prelude::Closure;

	use super::Gesture;

	type Handler = Rc<dyn Fn(Gesture)>;

	#[derive(Default)]
	struct Pointers {
		handlers: HashMap<u32, Handler>,
		/// The scene a finger went down on, where it went down, and whether
		/// it has moved far enough to be a drag.
		active: Option<(u32, f64, f64, f64, f64, bool)>,
		installed: bool,
	}

	thread_local! {
		static POINTERS: RefCell<Pointers> = RefCell::new(Pointers::default());
	}

	/// How far a finger may move and still tap, in CSS pixels.
	const TAP_SLOP: f64 = 8.0;

	fn canvas_under(x: f64, y: f64) -> Option<(u32, f64, f64)> {
		let document = web_sys::window()?.document()?;
		let ids: Vec<u32> =
			POINTERS.with_borrow(|pointers| pointers.handlers.keys().copied().collect());
		ids.into_iter().find_map(|id| {
			let canvas = document
				.query_selector(&format!("canvas[data-lk-scene3d=\"{id}\"]"))
				.ok()??;
			let rect = canvas.get_bounding_client_rect();
			let inside = rect.width() > 0.0
				&& x >= rect.left()
				&& x <= rect.right()
				&& y >= rect.top()
				&& y <= rect.bottom();
			inside.then(|| {
				(
					id,
					(x - rect.left()) / rect.width(),
					(y - rect.top()) / rect.height(),
				)
			})
		})
	}

	fn send(id: u32, gesture: Gesture) {
		let handler = POINTERS.with_borrow(|pointers| pointers.handlers.get(&id).cloned());
		if let Some(handler) = handler {
			handler(gesture);
		}
	}

	fn listen<E: FromWasmAbi + 'static>(name: &str, handler: impl FnMut(E) + 'static) {
		let Some(document) = web_sys::window().and_then(|window| window.document()) else {
			return;
		};
		let closure = Closure::<dyn FnMut(E)>::new(handler);
		let _ = document.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref());
		closure.forget();
	}

	fn install() {
		// A drag across the words over a scene turns the scene. It must not
		// select them: a press on selected words drags the text instead, and
		// the browser cancels the pointer before the scene hears it move.
		for name in ["selectstart", "dragstart"] {
			listen(name, |event: web_sys::Event| {
				if POINTERS.with_borrow(|pointers| pointers.active.is_some()) {
					event.prevent_default();
				}
			});
		}
		listen("pointerdown", |event: web_sys::PointerEvent| {
			let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
			// A press on a button over the scene is the button's.
			let on_control = event
				.target()
				.and_then(|target| target.dyn_into::<web_sys::Element>().ok())
				.and_then(|element| {
					element
						.closest("button, [role=button], input, a")
						.ok()
						.flatten()
				})
				.is_some();
			if on_control {
				return;
			}
			let Some((id, fx, fy)) = canvas_under(x, y) else {
				return;
			};
			POINTERS.with_borrow_mut(|pointers| pointers.active = Some((id, x, y, fx, fy, false)));
			send(
				id,
				Gesture::Down {
					x: fx as f32,
					y: fy as f32,
				},
			);
		});
		listen("pointermove", |event: web_sys::PointerEvent| {
			let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
			let moved = POINTERS.with_borrow_mut(|pointers| {
				let (id, last_x, last_y, fx, fy, dragging) = pointers.active?;
				let far = dragging || (x - last_x).hypot(y - last_y) > TAP_SLOP;
				if !far {
					return None;
				}
				pointers.active = Some((id, x, y, fx, fy, true));
				Some((id, x - last_x, y - last_y))
			});
			if let Some((id, dx, dy)) = moved {
				send(
					id,
					Gesture::Drag {
						dx: dx as f32,
						dy: dy as f32,
					},
				);
			}
		});
		let up = |_event: web_sys::PointerEvent| {
			let ended = POINTERS.with_borrow_mut(|pointers| pointers.active.take());
			if let Some((id, _, _, fx, fy, dragged)) = ended {
				if !dragged {
					send(
						id,
						Gesture::Tap {
							x: fx as f32,
							y: fy as f32,
						},
					);
				}
				send(id, Gesture::Up);
			}
		};
		listen("pointerup", up);
		listen("pointercancel", up);
	}

	pub fn on_gesture(scene: SceneId, handler: impl Fn(Gesture) + 'static) {
		let install_now = POINTERS.with_borrow_mut(|pointers| {
			pointers.handlers.insert(scene.0, Rc::new(handler));
			!std::mem::replace(&mut pointers.installed, true)
		});
		if install_now {
			install();
		}
	}

	pub fn forget(scene: SceneId) {
		POINTERS.with_borrow_mut(|pointers| {
			pointers.handlers.remove(&scene.0);
		});
	}
}

/// Translates the native host's primary pointer into the game's gestures.
#[cfg(any(
	test,
	all(target_arch = "wasm32", feature = "native", not(feature = "web"))
))]
#[derive(Default)]
pub(super) struct NativeGestures {
	down: Option<(f32, f32, bool)>,
}

#[cfg(any(
	test,
	all(target_arch = "wasm32", feature = "native", not(feature = "web"))
))]
impl NativeGestures {
	pub(super) fn accept(
		&mut self,
		input: leptoskit::scene3d::PointerInput,
	) -> [Option<Gesture>; 2] {
		use leptoskit::scene3d::PointerPhase;
		let gesture = match input.phase {
			PointerPhase::Down => {
				self.down = Some((input.x, input.y, false));
				Gesture::Down {
					x: input.x,
					y: input.y,
				}
			}
			PointerPhase::Move => {
				let Some((_, _, dragged)) = self.down.as_mut() else {
					return [None, None];
				};
				*dragged = true;
				Gesture::Drag {
					dx: input.dx,
					dy: input.dy,
				}
			}
			PointerPhase::Up => {
				if let Some((x, y, false)) = self.down.take() {
					return [Some(Gesture::Tap { x, y }), Some(Gesture::Up)];
				}
				Gesture::Up
			}
			PointerPhase::Cancel => {
				self.down = None;
				Gesture::Up
			}
		};
		[Some(gesture), None]
	}
}

#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
mod native {
	use std::cell::RefCell;
	use std::collections::HashMap;
	use std::collections::HashSet;
	use std::rc::Rc;

	use leptoskit::scene3d::client::Scene;

	use super::Gesture;
	use super::NativeGestures;
	use super::SceneId;

	type Handler = Rc<dyn Fn(Gesture)>;
	thread_local! {
		static HANDLERS: RefCell<HashMap<SceneId, Handler>> = RefCell::new(HashMap::new());
		static LISTENING: RefCell<HashSet<SceneId>> = RefCell::new(HashSet::new());
	}

	fn send(scene: SceneId, gesture: Gesture) {
		let handler = HANDLERS.with_borrow(|handlers| handlers.get(&scene).cloned());
		if let Some(handler) = handler {
			handler(gesture);
		}
	}

	pub fn on_gesture(id: SceneId, handler: impl Fn(Gesture) + 'static) {
		HANDLERS.with_borrow_mut(|handlers| handlers.insert(id, Rc::new(handler)));
		if !LISTENING.with_borrow_mut(|listening| listening.insert(id)) {
			return;
		}
		crate::platform::spawn(async move {
			let scene = Scene::new(id);
			let mut gestures = NativeGestures::default();
			loop {
				match scene.next_pointer().await {
					Ok(Some(input)) => {
						for gesture in gestures.accept(input).into_iter().flatten() {
							send(id, gesture);
						}
					}
					Ok(None) => break,
					Err(error) => {
						crate::platform::log(&format!("a scene's pointer stream failed: {error}"));
						break;
					}
				}
			}
			send(id, Gesture::Up);
			HANDLERS.with_borrow_mut(|handlers| handlers.remove(&id));
			LISTENING.with_borrow_mut(|listening| listening.remove(&id));
		});
	}

	pub fn forget(id: SceneId) {
		HANDLERS.with_borrow_mut(|handlers| handlers.remove(&id));
	}
}

/// Hands every gesture on `scene`'s view to `handler`, replacing the one before.
pub fn on_gesture(scene: SceneId, handler: impl Fn(Gesture) + 'static) {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	web::on_gesture(scene, handler);
	#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
	native::on_gesture(scene, handler);
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	let _ = (scene, handler);
}

/// Stops listening for `scene`.
pub fn forget(scene: SceneId) {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	web::forget(scene);
	#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
	native::forget(scene);
	#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
	let _ = scene;
}

/// Whether this build can follow a finger across a scene.
pub const fn follows_fingers() -> bool {
	cfg!(all(
		target_arch = "wasm32",
		any(feature = "web", feature = "native")
	))
}

#[cfg(test)]
mod tests {
	use leptoskit::scene3d::PointerInput;
	use leptoskit::scene3d::PointerPhase;

	use super::*;

	fn input(phase: PointerPhase, dx: f32) -> PointerInput {
		PointerInput {
			phase,
			x: 0.25,
			y: 0.75,
			dx,
			dy: 0.0,
		}
	}

	#[test]
	fn native_drags_reach_the_stage_without_becoming_taps() {
		let mut gestures = NativeGestures::default();
		assert_eq!(
			gestures.accept(input(PointerPhase::Move, 90.0)),
			[None, None]
		);
		assert_eq!(
			gestures.accept(input(PointerPhase::Down, 0.0)),
			[Some(Gesture::Down { x: 0.25, y: 0.75 }), None]
		);
		assert_eq!(
			gestures.accept(input(PointerPhase::Move, 90.0)),
			[Some(Gesture::Drag { dx: 90.0, dy: 0.0 }), None]
		);
		assert_eq!(
			gestures.accept(input(PointerPhase::Up, 0.0)),
			[Some(Gesture::Up), None]
		);
	}

	#[test]
	fn native_tap_and_cancel_release_a_held_control() {
		let mut gestures = NativeGestures::default();
		gestures.accept(input(PointerPhase::Down, 0.0));
		assert_eq!(
			gestures.accept(input(PointerPhase::Up, 0.0)),
			[Some(Gesture::Tap { x: 0.25, y: 0.75 }), Some(Gesture::Up)]
		);
		gestures.accept(input(PointerPhase::Down, 0.0));
		assert_eq!(
			gestures.accept(input(PointerPhase::Cancel, 0.0)),
			[Some(Gesture::Up), None]
		);
		assert!(gestures.down.is_none());
	}
}
