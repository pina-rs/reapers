//! Fingers and mice on a scene's view, in the browser.
//!
//! leptoskit's views hear a press and nothing finer, so the browser build
//! listens on the page itself and hands drags and taps to whichever scene's
//! canvas they land on, even under the words laid over it. The native apps
//! have no drag yet: their stages turn on their own, and their hunts follow
//! the phone.

use leptoskit::scene3d::SceneId;

/// What a finger did on a scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
	/// Moved while down, by this many CSS pixels.
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

/// Hands every gesture on `scene`'s view to `handler`, replacing the one
/// before. Nothing happens where the host has no pointer to listen to.
pub fn on_gesture(scene: SceneId, handler: impl Fn(Gesture) + 'static) {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	web::on_gesture(scene, handler);
	#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
	let _ = (scene, handler);
}

/// Stops listening for `scene`.
pub fn forget(scene: SceneId) {
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	web::forget(scene);
	#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
	let _ = scene;
}

/// Whether this build can follow a finger across a scene.
pub const fn follows_fingers() -> bool {
	cfg!(all(target_arch = "wasm32", feature = "web"))
}
