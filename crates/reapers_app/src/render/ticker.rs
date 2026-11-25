//! One clock for everything the game moves by itself between renders: a
//! stage turning, a robe stirring. It runs only while something is
//! registered, at about thirty ticks a second, and stops where the host has
//! no timer (the tests).

use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;

/// What a tick does: given the seconds since the last, answers whether it
/// still wants ticks.
type Tick = Box<dyn FnMut(f64) -> bool>;

thread_local! {
	static TICKS: RefCell<Vec<(u64, Tick)>> = const { RefCell::new(Vec::new()) };
	static RUNNING: Cell<bool> = const { Cell::new(false) };
	static NEXT: Cell<u64> = const { Cell::new(1) };
}

/// How long a tick waits for the next.
pub const INTERVAL: Duration = Duration::from_millis(33);

/// Calls `tick` about thirty times a second until it answers `false`.
pub fn every_frame(tick: impl FnMut(f64) -> bool + 'static) -> u64 {
	let id = NEXT.with(|next| {
		let id = next.get();
		next.set(id + 1);
		id
	});
	TICKS.with_borrow_mut(|ticks| ticks.push((id, Box::new(tick))));
	if !RUNNING.with(Cell::get) {
		RUNNING.with(|running| running.set(true));
		start();
	}
	id
}

/// Stops the tick `id` registered.
pub fn cancel(id: u64) {
	TICKS.with_borrow_mut(|ticks| ticks.retain(|(tick, _)| *tick != id));
}

fn step_ticks(delta: f64) -> bool {
	// A tick may register another; run them outside the borrow.
	let mut running = TICKS.with_borrow_mut(std::mem::take);
	running.retain_mut(|(_, tick)| tick(delta));
	TICKS.with_borrow_mut(|ticks| {
		running.append(ticks);
		*ticks = running;
		ticks.is_empty()
	})
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn start() {
	use std::rc::Rc;

	use wasm_bindgen::JsCast as _;
	use wasm_bindgen::prelude::Closure;

	const MIN_STEP_MS: f64 = 30.0;

	let Some(window) = web_sys::window() else {
		RUNNING.with(|running| running.set(false));
		return;
	};
	let last = Rc::new(Cell::new(None::<f64>));
	let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
	let cb_inner = cb.clone();
	let win_inner = window.clone();
	*cb.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
		let prev = last.get().unwrap_or(timestamp_ms);
		let elapsed_ms = timestamp_ms - prev;
		let keep_going = if elapsed_ms >= MIN_STEP_MS || last.get().is_none() {
			last.set(Some(timestamp_ms));
			let delta = if elapsed_ms > 0.0 {
				(elapsed_ms / 1000.0).clamp(0.0, 0.1)
			} else {
				INTERVAL.as_secs_f64()
			};
			!step_ticks(delta)
		} else {
			!TICKS.with_borrow(Vec::is_empty)
		};
		if keep_going
			&& let Some(callback) = cb_inner.borrow().as_ref()
			&& win_inner
				.request_animation_frame(callback.as_ref().unchecked_ref())
				.is_ok()
		{
			return;
		}
		RUNNING.with(|running| running.set(false));
		*cb_inner.borrow_mut() = None;
	}));
	if let Some(callback) = cb.borrow().as_ref()
		&& window
			.request_animation_frame(callback.as_ref().unchecked_ref())
			.is_err()
	{
		RUNNING.with(|running| running.set(false));
		*cb.borrow_mut() = None;
	}
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
fn start() {
	crate::platform::spawn(run());
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
async fn run() {
	let mut last = crate::platform::now();
	loop {
		if !crate::platform::sleep(INTERVAL).await {
			break;
		}
		let now = crate::platform::now();
		let delta = (now.micros_since(last) as f64 / 1e6).clamp(0.0, 0.1);
		last = now;
		if step_ticks(delta) {
			break;
		}
	}
	RUNNING.with(|running| running.set(false));
}
