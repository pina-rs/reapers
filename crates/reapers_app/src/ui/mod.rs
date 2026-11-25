//! The interface kit: Second Ink's printed controls, rows and marks, in
//! paper on a dark page, lit by the bound reaper's own light.

mod ledger;
mod overlay;
mod sketch;
mod text;

pub use ledger::LedgerButton;
pub use ledger::gap;
pub use ledger::ledger_back;
pub use ledger::ledger_heading;
pub use ledger::ledger_meter;
pub use ledger::ledger_notice;
pub use ledger::ledger_page;
pub use ledger::ledger_pip;
pub use ledger::ledger_row;
pub use overlay::dialog;
pub use overlay::sheet;
pub use sketch::sketch;
pub use text::aside;
pub use text::body;
pub use text::display;
pub use text::figure;
pub use text::label;
pub use text::margin;
pub use text::prose;
pub use text::wound_words;

use crate::state::Game;
use crate::theme::Rgb;
use crate::theme::UNBOUND;

/// The foot of a stage falling into the page's ink, over the lowest
/// `height` percent of its parent, to `alpha` at the bottom. leptoskit has
/// no gradients, so the fall is drawn as thin smoothstep strips from clear
/// to ink without a hard top seam.
pub fn foot_scrim(height: u32, alpha: u8) -> leptoskit::prelude::Element {
	use leptoskit::prelude::*;
	const STEPS: u32 = 24;
	let mut scrim = View()
		.enabled(false)
		.class(format!("absolute left-0 right-0 bottom-0 h-[{height}%]"));
	for step in 0..STEPS {
		let t = f64::from(step) / f64::from(STEPS - 1);
		let smooth = t * t * (3.0 - 2.0 * t);
		let shade = (f64::from(alpha) * smooth).round() as u8;
		scrim = scrim.child(View().class(format!(
			"flex-1 w-full bg-[{}{shade:02x}]",
			crate::theme::KEY.hex()
		)));
	}
	scrim
}

/// The light the interface is lit with: the bound reaper's class's spot
/// (§3.2), or paper before any reaper is bound.
pub fn light() -> Rgb {
	use reactive_graph::traits::With as _;

	reactive_graph::owner::use_context::<Game>().map_or(UNBOUND, |game| {
		game.books.with(|books| {
			match books {
				crate::state::Books::Open(player) if player.oath_taken() => {
					crate::theme::light(player.genome().reaper_class)
				}
				_ => UNBOUND,
			}
		})
	})
}

thread_local! {
	/// An owner that lives as long as the app, for state a screen keeps
	/// between renders. A signal made during a render belongs to that render
	/// and is gone by the next one.
	static LASTING: std::cell::RefCell<Option<reactive_graph::owner::Owner>> =
		const { std::cell::RefCell::new(None) };
}

/// Makes the current owner's child the home of [`lasting`] state. The app
/// calls it once, where it starts.
pub fn install_lasting_owner() {
	if let Some(owner) = reactive_graph::owner::Owner::current() {
		LASTING.with_borrow_mut(|lasting| *lasting = Some(owner.child()));
	}
}

/// Runs `make` under the app's lasting owner, so the signals it creates
/// outlive the render that first asked for them.
pub fn lasting<T>(make: impl FnOnce() -> T) -> T {
	match LASTING.with_borrow(Clone::clone) {
		Some(owner) => owner.with(make),
		None => make(),
	}
}
