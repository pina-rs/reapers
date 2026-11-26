//! The interface kit of Second Ink: every control and mark a rebuilt screen
//! is made of (the art bible, §8), drawn in Skribble's hand
//! ([`super::ink`]) from the tokens in [`crate::theme`].
//!
//! - Every control names its [`State`]s, and keeps a hit area of at least
//!   44 points (§15.3).
//! - It moves only as §13 allows: a press is a snap of 120 ms, a slip rises
//!   8 points in 333 ms, and nothing idles.
//! - Under reduced motion (§14) nothing slides: the primary sits in
//!   register and a press darkens its lip, and slips, stamps and sheets
//!   appear in place.
//!
//! A control follows the hand by itself: a press is its `active:` style, and
//! keyboard focus draws paper crop marks from the web shell
//! (`assets/app.css`). [`State::Pressed`] and [`State::Focused`] print those
//! looks held, for the kit page.

mod buttons;
mod fields;
mod marks;
mod surfaces;
mod tabs;

pub use buttons::Primary;
pub use buttons::Quiet;
pub use buttons::Secondary;
pub use fields::Field;
pub use fields::FieldState;
pub use fields::Toggle;
pub use fields::ToggleKind;
use leptoskit::prelude::*;
pub use marks::arrow;
pub use marks::class_mark;
pub use marks::lamp;
pub use marks::meter;
pub use marks::pips;
pub use marks::soul_dial;
pub use marks::stamp;
pub use marks::tally;
pub use surfaces::PaperSheet;
pub use surfaces::Row;
pub use surfaces::Slip;
pub use surfaces::SlipState;
pub use surfaces::divider;
pub use tabs::Tab;
pub use tabs::TabIcon;
pub use tabs::tab_bar;

use crate::platform;
use crate::theme::PAPER;
use crate::theme::motion;
use crate::theme::space;

/// A control's state (§8). Every control prints each one it can be in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum State {
	/// At rest, and answering the hand: printed pressed while held, and
	/// with crop marks while focused from the keyboard.
	#[default]
	Rest,
	/// Held down, printed so.
	Pressed,
	/// Reached from the keyboard: paper crop marks at the four corners.
	Focused,
	/// Working on what it was asked: it takes no second press.
	Busy,
	/// Not available: drawn in ash, saying why where it can.
	Disabled,
}

impl State {
	/// Every state, in the order the kit page prints them.
	pub const ALL: [Self; 5] = [
		Self::Rest,
		Self::Pressed,
		Self::Focused,
		Self::Busy,
		Self::Disabled,
	];

	/// Whether a press reaches the control's action. A held print on the
	/// kit page takes none, as a busy or disabled control takes none.
	pub const fn takes_presses(self) -> bool {
		matches!(self, Self::Rest)
	}

	/// Whether the control is not available.
	pub const fn is_disabled(self) -> bool {
		matches!(self, Self::Disabled)
	}
}

/// What a control does when it is pressed.
type Action = Box<dyn Fn() + Send + Sync + 'static>;

/// The press of `action`, with the tap under the thumb every control in the
/// game gives first.
fn with_tap(action: impl Fn() + Send + Sync + 'static) -> Action {
	Box::new(move || {
		platform::tap();
		action();
	})
}

/// Whether the reader has asked for less motion (§14): the device's
/// setting, read through the platform.
pub fn steady() -> bool {
	platform::reduced_motion()
}

/// The width the page's content is set to: the screen less its margins,
/// and never wider than the column (§9). Plates are drawn to it, so a rule
/// is drawn at the length it is shown.
pub fn measure() -> f32 {
	let (width, _) = platform::viewport();
	let margins = 2.0 * space::MARGIN as f32;
	(width as f32 - margins).clamp(240.0, space::COLUMN as f32)
}

/// The press of a control (§13): the snap spring, about 120 ms, as a
/// transition of its transform. Empty under reduced motion, where nothing
/// slides.
fn snap() -> String {
	if steady() {
		String::new()
	} else {
		format!(
			"transition-transform duration-[{}ms] ease-out",
			motion::SNAP.settles_ms
		)
	}
}

/// How a slip, a stamp or a sheet enters (§13): risen `rise` points over
/// the rise's 333 ms, or nothing under reduced motion, where it appears in
/// place (§14).
fn rise(rise: u32) -> String {
	if steady() {
		String::new()
	} else {
		format!(
			"starting:translate-y-[{rise}px] transition-transform duration-[{}ms] ease-out",
			motion::RISE_MS
		)
	}
}

/// Paper crop marks at the four corners of the box they are laid in: each
/// mark 2 points wide and 6 long, 4 outside the corner (§8). It is focus,
/// printed held, for the kit page; a live control draws the same marks from
/// the web shell when the keyboard reaches it.
pub fn crop_marks() -> Element {
	const MARKS: [&str; 8] = [
		"left-[-10px] top-[-1px] w-[6px] h-[2px]",
		"left-[-1px] top-[-10px] w-[2px] h-[6px]",
		"right-[-10px] top-[-1px] w-[6px] h-[2px]",
		"right-[-1px] top-[-10px] w-[2px] h-[6px]",
		"left-[-10px] bottom-[-1px] w-[6px] h-[2px]",
		"left-[-1px] bottom-[-10px] w-[2px] h-[6px]",
		"right-[-10px] bottom-[-1px] w-[6px] h-[2px]",
		"right-[-1px] bottom-[-10px] w-[2px] h-[6px]",
	];
	MARKS.into_iter().fold(
		View()
			.class("absolute inset-0")
			.enabled(false)
			.aria_hidden(true),
		|marks, place| marks.child(View().class(format!("absolute {place} bg-[{}]", PAPER.hex()))),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_a_control_at_rest_takes_a_press() {
		let taking: Vec<State> = State::ALL
			.into_iter()
			.filter(|state| state.takes_presses())
			.collect();
		assert_eq!(taking, [State::Rest]);
	}

	#[test]
	fn nothing_slides_under_reduced_motion() {
		platform::set_reduced_motion(true);
		assert_eq!(snap(), "");
		assert_eq!(rise(8), "");
		platform::set_reduced_motion(false);
		assert!(snap().contains("duration-[120ms]"), "{}", snap());
		assert!(
			rise(8).contains("starting:translate-y-[8px]"),
			"{}",
			rise(8)
		);
		assert!(rise(8).contains("duration-[333ms]"), "{}", rise(8));
	}

	#[test]
	fn the_measure_keeps_the_margins_and_the_column() {
		// The tests' phone is 390 wide: 350 between the margins.
		assert!((measure() - 350.0).abs() < f32::EPSILON, "{}", measure());
	}

	/// `make`, with the game's words to read, as the app provides them.
	fn with_words(make: impl FnOnce() -> Element) -> Element {
		reactive_graph::owner::Owner::new().with(|| {
			crate::l10n::L10n::new(&[]).provide();
			make()
		})
	}

	/// Every control of the kit, at rest and live, and each one busy or
	/// disabled, on a phone-sized page.
	fn controls(presses: &std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Element {
		use std::sync::atomic::Ordering;

		use crate::theme::CHAFF;
		let press = |presses: &std::sync::Arc<std::sync::atomic::AtomicUsize>| {
			let presses = std::sync::Arc::clone(presses);
			move || {
				presses.fetch_add(1, Ordering::SeqCst);
			}
		};
		let tabs = [
			(TabIcon::Lamp, "tab-lamp"),
			(TabIcon::Box, "tab-box"),
			(TabIcon::Blade, "tab-blade"),
			(TabIcon::Book, "tab-book"),
			(TabIcon::Crack, "tab-crack"),
		]
		.into_iter()
		.enumerate()
		.map(|(index, (icon, id))| {
			Tab {
				icon,
				label: id.to_owned(),
				test_id: id.to_owned(),
				active: index == 0,
				state: State::Rest,
				open: Box::new(press(presses)),
			}
		})
		.collect();
		View()
			.class("w-full px-5 gap-3")
			.child(
				Primary::new("Go hunting", CHAFF)
					.detail("2 nearby")
					.test_id("primary")
					.on_press(press(presses)),
			)
			.child(
				Primary::new("Throw the orb", CHAFF)
					.disabled("Needs 40 soul")
					.test_id("primary-disabled")
					.on_press(press(presses)),
			)
			.child(
				Secondary::new("Back to the sanctum")
					.test_id("secondary")
					.on_press(press(presses)),
			)
			.child(
				Secondary::new("Buy")
					.state(State::Busy)
					.test_id("secondary-busy")
					.on_press(press(presses)),
			)
			.child(
				Quiet::new("Colophon")
					.test_id("quiet")
					.on_press(press(presses)),
			)
			.child(Slip::new("A slip").test_id("slip").on_press(press(presses)))
			.child(
				Row::new("Penguin #1")
					.status("Loose")
					.figure("1:47")
					.test_id("row")
					.on_press(press(presses)),
			)
			.child(
				Toggle::new(ToggleKind::Switch, "Music", true, CHAFF)
					.test_id("switch")
					.on_press(press(presses)),
			)
			.child(
				Toggle::new(ToggleKind::Checkbox, "Haptics", false, CHAFF)
					.test_id("checkbox")
					.on_press(press(presses)),
			)
			.child(
				Toggle::new(ToggleKind::Radio, "Night", true, CHAFF)
					.test_id("radio")
					.on_press(press(presses)),
			)
			.child(tab_bar(tabs, CHAFF))
	}

	#[test]
	fn every_control_keeps_a_target_of_forty_four_points_and_only_rest_takes_presses() {
		use std::sync::Arc;
		use std::sync::atomic::AtomicUsize;
		use std::sync::atomic::Ordering;

		use leptoskit_testing::Finder;
		use leptoskit_testing::WidgetTester;

		let presses = Arc::new(AtomicUsize::new(0));
		let page = Arc::clone(&presses);
		let mut tester = WidgetTester::new(
			move || with_words(|| ScrollView().class("w-full h-screen").child(controls(&page))),
			390.0,
			1400.0,
		)
		.unwrap();
		let target = space::TARGET as f32;
		for id in [
			"primary",
			"primary-disabled",
			"secondary",
			"secondary-busy",
			"quiet",
			"slip",
			"row",
			"switch",
			"checkbox",
			"radio",
			"tab-lamp",
			"tab-box",
			"tab-blade",
			"tab-book",
			"tab-crack",
		] {
			let frame = tester.find(&Finder::test_id(id)).unwrap().frame;
			assert!(
				frame.height >= target && frame.width >= target,
				"{id} is {frame:?}"
			);
		}
		// The primary is 54 points tall, and grows rather than clip a label
		// the tests' host measures wider than the face does.
		let primary = tester.find(&Finder::test_id("primary")).unwrap().frame;
		assert!(
			primary.height >= space::PRIMARY as f32 && primary.width >= 349.0,
			"{primary:?}\n{}",
			tester.tree()
		);
		let tab = tester.find(&Finder::test_id("tab-box")).unwrap().frame;
		assert_eq!(
			(tab.width, tab.height),
			(space::TAB_WIDTH as f32, space::TAB_HEIGHT as f32)
		);

		for (id, count) in [
			("primary", 1),
			("secondary", 2),
			("quiet", 3),
			("slip", 4),
			("row", 5),
			("switch", 6),
			("tab-box", 7),
		] {
			tester.tap(&Finder::test_id(id)).unwrap();
			assert_eq!(presses.load(Ordering::SeqCst), count, "{id}");
		}
		for id in ["primary-disabled", "secondary-busy"] {
			let _refused = tester.tap(&Finder::test_id(id));
			assert_eq!(presses.load(Ordering::SeqCst), 7, "{id} took a press");
		}
	}

	#[test]
	fn a_toggle_says_its_state_in_words_and_to_a_screen_reader() {
		use leptoskit_testing::Finder;
		use leptoskit_testing::WidgetTester;

		use crate::theme::CHAFF;
		let tester = WidgetTester::new(
			|| {
				with_words(|| {
					View()
						.child(
							Toggle::new(ToggleKind::Switch, "Music", true, CHAFF)
								.test_id("on")
								.print(),
						)
						.child(
							Toggle::new(ToggleKind::Switch, "Haptics", false, CHAFF)
								.test_id("off")
								.print(),
						)
				})
			},
			390.0,
			844.0,
		)
		.unwrap();
		let tree = tester.tree();
		assert!(tree.contains("ON"), "{tree}");
		assert!(tree.contains("OFF"), "{tree}");
		tester.find(&Finder::test_id("on")).unwrap();
	}
}
