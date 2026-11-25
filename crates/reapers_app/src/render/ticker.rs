//! One clock for everything the game moves by itself between renders: a
//! stage turning, a robe stirring, the arms and the husks. It runs only while
//! something is registered, once for every frame the display draws, and stops
//! where the host has no clock (the tests).
//!
//! The browser calls it from `requestAnimationFrame`, a native app from its
//! display link (`leptoskit::guest::next_frame`), so whatever moves is told
//! where to be as often as the screen shows it: thirty, sixty or a hundred
//! and twenty times a second. A host without a frame clock falls back to a
//! timer at [`INTERVAL`]. Every tick is told how long the last frame took, and
//! whatever it moves must move the same at any of those rates.

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

/// How long a tick waits for the next where the host has no frame clock:
/// about one frame of a sixty-hertz display.
pub const INTERVAL: Duration = Duration::from_millis(16);

/// The longest a tick is told a frame took: a stall is not played back as a
/// leap.
pub const LONGEST_FRAME: f64 = 0.1;

/// Calls `tick` once a display frame until it answers `false`.
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

	let Some(window) = web_sys::window() else {
		RUNNING.with(|running| running.set(false));
		return;
	};
	let last = Rc::new(Cell::new(None::<f64>));
	let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
	let cb_inner = cb.clone();
	let win_inner = window.clone();
	*cb.borrow_mut() = Some(Closure::new(move |timestamp_ms: f64| {
		// Every frame the browser draws is a tick; the first is told it took
		// as long as a frame of a sixty-hertz display.
		let delta = last
			.replace(Some(timestamp_ms))
			.map_or(INTERVAL.as_secs_f64(), |previous| {
				frame_seconds(timestamp_ms - previous)
			});
		let keep_going = !step_ticks(delta);
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

/// Ticks on the display's own frames: the host's display link tells each
/// frame when it began.
#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
async fn run() {
	let mut last = None::<u64>;
	loop {
		let frame = match leptoskit::guest::next_frame().await {
			Ok(frame) => frame,
			// A host built before the frame clock: its timer stands in.
			Err(leptoskit::guest::Error::Unavailable) => return run_on_timer().await,
			Err(_) => break,
		};
		let delta = match last.replace(frame.timestamp_ns) {
			Some(previous) => {
				frame_seconds(frame.timestamp_ns.saturating_sub(previous) as f64 / 1e6)
			}
			None if frame.period_ns > 0 => frame_seconds(frame.period_ns as f64 / 1e6),
			None => INTERVAL.as_secs_f64(),
		};
		if step_ticks(delta) {
			break;
		}
	}
	RUNNING.with(|running| running.set(false));
}

#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
async fn run() {
	run_on_timer().await;
}

/// Ticks every [`INTERVAL`] on the host's timer, measuring each gap.
#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
async fn run_on_timer() {
	let mut last = crate::platform::now();
	loop {
		if !crate::platform::sleep(INTERVAL).await {
			break;
		}
		let now = crate::platform::now();
		let delta = frame_seconds(now.micros_since(last) as f64 / 1e3);
		last = now;
		if step_ticks(delta) {
			break;
		}
	}
	RUNNING.with(|running| running.set(false));
}

/// A frame that took `millis` milliseconds, in seconds, no longer than
/// [`LONGEST_FRAME`].
fn frame_seconds(millis: f64) -> f64 {
	if millis.is_finite() {
		(millis / 1000.0).clamp(0.0, LONGEST_FRAME)
	} else {
		0.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_frame_is_measured_in_seconds_and_a_stall_is_cut_short() {
		assert!((frame_seconds(8.333) - 0.008_333).abs() < 1e-9);
		assert!((frame_seconds(16.7) - 0.0167).abs() < 1e-9);
		assert_eq!(frame_seconds(2_000.0), LONGEST_FRAME);
		assert_eq!(frame_seconds(-3.0), 0.0);
		assert_eq!(frame_seconds(f64::NAN), 0.0);
		assert!(
			INTERVAL.as_secs_f64() < 1.0 / 59.0,
			"the fallback keeps sixty"
		);
	}
}
