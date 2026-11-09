//! The native runtime's entry points: the C names the host calls to start
//! the app, hand it events, tell it the time and let it run its tasks.

#![allow(unsafe_code)] // Exported C names form the native runtime's entry points.

use reactive_graph::owner::Owner;

/// Mounts the game in the native Wasm runtime.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_main() {
	let owner = Owner::new();
	let navigation = crate::app::navigation();
	owner.with(|| crate::app::start(navigation.clone()));
	leptoskit::guest::mount(move || owner.with(|| crate::app::screen(&navigation)));
}

/// Forwards platform-neutral events to the tree's handlers.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_event(node: i64, kind: i32, ptr: i32, len: i32) -> i32 {
	leptoskit::guest::dispatch(node, kind, ptr, len)
}

/// Settles the values application state bound, after a turn.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_flush() {
	leptoskit::guest::flush();
}

/// Runs the tasks a plugin reply or a timer woke.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_poll() {
	leptoskit::guest::poll();
}

/// Allocates the native runtime's event payload buffer.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_alloc(len: i32) -> i32 {
	leptoskit::guest::alloc(len)
}

/// Reports the native UI bridge ABI this app implements.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_abi_version() -> i32 {
	leptoskit::guest::ABI_VERSION as i32
}

/// Called by the runtime with the wall clock, in milliseconds since 1970,
/// before every entry point: the guest has no clock of its own.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_clock(unix_ms: i64) {
	crate::platform::report_clock(unix_ms);
}

/// Keeps the hunt's lens and HUD in the host's actual view, including rotation.
#[unsafe(no_mangle)]
pub extern "C" fn leptoskit_viewport(width: f32, height: f32) {
	leptoskit::guest::set_viewport(width, height);
}
