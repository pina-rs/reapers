//! The first thing an unbound player sees: the name, the premise, and the
//! two ways in.

use std::sync::LazyLock;

use leptoskit::prelude::*;

use super::open;
use crate::brand::reapers_mark;
use crate::l10n::use_l10n;
use crate::routes::Route;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;

/// The blade's spine and edge, across a 100 by 100 page: the same arc the
/// player will later sweep with their phone.
static SPINE: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse("M108 5 C55 2 2 20 -6 52").expect("the spine's path is valid")
});
static EDGE: LazyLock<PathData> = LazyLock::new(|| {
	PathData::parse("M108 13 C62 12 16 26 -6 52").expect("the edge's path is valid")
});

/// One long blade drawn across the page.
fn blade() -> Element {
	let ink = |alpha| Color::rgba(0xE9, 0xE4, 0xD4, alpha);
	Vector(ViewBox::new(0.0, 0.0, 100.0, 100.0).expect("a positive size"))
		.scaling(VectorScaling::Stretch)
		.path(
			VectorPath::new(SPINE.clone())
				.stroke(Stroke::new(ink(0x2E), 1.0).expect("a positive width")),
		)
		.path(
			VectorPath::new(EDGE.clone())
				.stroke(Stroke::new(ink(0xD9), 1.4).expect("a positive width")),
		)
		.class("absolute inset-0 starting:opacity-0 transition-opacity duration-1000")
		.aria_hidden(true)
}

/// The threshold.
pub fn page() -> Element {
	let words = use_l10n();
	View()
		.class("relative flex-1 min-h-screen w-full")
		.test_id("threshold")
		.child(blade())
		.child(
			View()
				.class(format!(
					"flex-1 w-full {CONTENT_WIDTH} self-center justify-end px-7 pt-safe-offset-6 pb-safe-offset-7 \
					 starting:opacity-0 transition-opacity duration-700 delay-500"
				))
				.child(reapers_mark(88, false))
				.child(gap(20))
				.child(display(words.app_title(), 84, BONE).test_id("threshold-title"))
				.child(gap(18))
				.child(body(words.threshold_tagline(), 16, ASH))
				.child(gap(34))
				.child(
					LedgerButton::new(words.threshold_oath())
						.detail(words.threshold_oath_detail())
						.test_id("threshold-oath")
						.on_press(|| open(Route::Oath)),
				)
				.child(gap(4))
				.child(
					LedgerButton::new(words.threshold_shade())
						.detail(words.threshold_shade_detail())
						.quiet()
						.test_id("threshold-shade")
						.on_press(|| open(Route::Practice)),
				),
		)
}
