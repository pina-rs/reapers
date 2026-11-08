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
		crate::platform::spawn(run());
	}
	id
}

/// Stops the tick `id` registered.
pub fn cancel(id: u64) {
	TICKS.with_borrow_mut(|ticks| ticks.retain(|(tick, _)| *tick != id));
}

async fn run() {
	let mut last = crate::platform::now();
	loop {
		if !crate::platform::sleep(INTERVAL).await {
			break;
		}
		let now = crate::platform::now();
		let delta = (now.micros_since(last) as f64 / 1e6).clamp(0.0, 0.1);
		last = now;
		// A tick may register another; run them outside the borrow.
		let mut running = TICKS.with_borrow_mut(std::mem::take);
		running.retain_mut(|(_, tick)| tick(delta));
		let empty = TICKS.with_borrow_mut(|ticks| {
			running.append(ticks);
			*ticks = running;
			ticks.is_empty()
		});
		if empty {
			break;
		}
	}
	RUNNING.with(|running| running.set(false));
}
