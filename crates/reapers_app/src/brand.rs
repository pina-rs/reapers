//! The Reapers mark, and the arrival that opens the game with it.
//!
//! The mark is Bound Sigil, the default of the four masters in
//! `art/branding` (see docs/branding.md): an ivory knot on the veil's ink.
//! The browser shows the same `mark.png` and `splash_mark.png` the Flutter
//! game and its launch screens use. A native app cannot name a bundled
//! picture to an `Image` yet, so there the mark is drawn: the knot's outer
//! triangle and its ring, in the same ivory.

use std::sync::LazyLock;

use leptoskit::prelude::*;

/// The knot, simplified to its outer strokes, in a 100 by 100 box.
static KNOT: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse(
		"M50 4 L96 84 L4 84 Z M50 22 L80 74 L20 74 Z M24 70 C30 92 70 92 76 70 M50 38 C44 50 36 58 26 60 \
		 M50 38 C56 50 64 58 74 60 M50 60 L56 70 L50 78 L44 70 Z",
	)
	.expect("the knot's path is valid")
});

/// The mark, `size` layout units square. `splash` picks the launch
/// screen's still, framed for Android 12's safe circle.
pub fn reapers_mark(size: u32, splash: bool) -> Element {
	let frame = format!("w-[{size}px] h-[{size}px]");
	if cfg!(all(target_arch = "wasm32", feature = "web")) {
		let source = if splash {
			"assets/branding/splash_mark.png"
		} else {
			"assets/branding/mark.png"
		};
		Image(source).class(frame).aria_hidden(true)
	} else {
		let ivory = Color::rgba(0xE9, 0xE4, 0xD4, 0xFF);
		Vector(ViewBox::new(0.0, 0.0, 100.0, 100.0).expect("a positive size"))
			.path(
				VectorPath::new(KNOT.clone())
					.fill_rule(FillRule::Evenodd)
					.stroke(Stroke::new(ivory, 2.5).expect("a positive width").round()),
			)
			.class(frame)
			.aria_hidden(true)
	}
}
