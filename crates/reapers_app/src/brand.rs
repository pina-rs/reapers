//! The Reapers mark, and the arrival that opens the game with it.
//!
//! The mark is Bound Sigil, the default of the four masters in
//! `art/branding` (see docs/branding.md): an ivory knot on the veil's ink.
//! It is the same `mark.png` and `splash_mark.png` the launch screens use,
//! bundled from `assets/branding`.

use leptoskit::prelude::*;

/// The mark, `size` layout units square. `splash` picks the launch
/// screen's still, framed for Android 12's safe circle.
pub fn reapers_mark(size: u32, splash: bool) -> Element {
	let frame = format!("w-[{size}px] h-[{size}px]");
	let source = if splash {
		"asset://branding/splash_mark.png"
	} else {
		"asset://branding/mark.png"
	};
	Image(source).class(frame).aria_hidden(true)
}
