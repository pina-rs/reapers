//! The buttons (§8): the primary, one to a screen at most, printed in the
//! spot; the secondary, a paper outline; and the quiet, a word with a rule
//! under it. Each calls [`platform::tap`] before its action, as every
//! button in the game does.

use leptoskit::prelude::*;

use super::Action;
use super::State;
use super::crop_marks;
use super::marks::arrow;
use super::measure;
use super::snap;
use super::steady;
use super::with_tap;
use crate::theme::ASH;
use crate::theme::KEY;
use crate::theme::PAPER;
use crate::theme::RAISED;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::WOUND;
use crate::theme::fit_display_line;
use crate::theme::seed_for;
use crate::theme::space;
use crate::theme::stroke;
use crate::ui::ink::Ink;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::text::aside;
use crate::ui::text::body;
use crate::ui::text::display_words;
use crate::ui::text::figure;

/// The lip's hatch at rest: key at 55%.
const LIP_AT_REST: &str = "#16121a8c";
/// How tall the hatched lip is, at the foot of the primary.
const LIP: f32 = 10.0;
/// Room the label leaves at the button's ends.
const INSET: f32 = 18.0;

/// The one primary action on a screen: a spot flood under a rough key
/// frame, the label in Display, and a hatched lip at its foot.
///
/// - **Rest:** the spot plate sits 3 points down and right, out of
///   register. Under reduced motion it sits in register.
/// - **Pressed:** the frame and its label slide into register with the spot
///   (the snap spring). Under reduced motion the lip darkens instead.
/// - **Focused:** crop marks.
/// - **Busy:** the lip prints dark and the button takes no second press.
/// - **Disabled:** the spot gives way to a key hatch on raised, the label
///   prints in ash, and the reason is set underneath.
pub struct Primary {
	label: String,
	detail: Option<String>,
	spot: Rgb,
	state: State,
	reason: Option<String>,
	test_id: Option<String>,
}

impl Primary {
	/// A primary that says `label`, printed in `spot`, the bound reaper's
	/// light.
	pub fn new(label: impl Into<String>, spot: Rgb) -> Self {
		Self {
			label: label.into(),
			detail: None,
			spot,
			state: State::Rest,
			reason: None,
			test_id: None,
		}
	}

	/// A figure at its right edge, with a drawn arrow after it: "2 nearby".
	#[must_use]
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Prints the button in `state`.
	#[must_use]
	pub const fn state(mut self, state: State) -> Self {
		self.state = state;
		self
	}

	/// The button unavailable, with `reason` underneath ("Needs 40 soul").
	#[must_use]
	pub fn disabled(mut self, reason: impl Into<String>) -> Self {
		self.state = State::Disabled;
		self.reason = Some(reason.into());
		self
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The button, doing `action` when it is pressed at rest.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	/// The button with nothing to do: a print of one of its states.
	pub fn print(self) -> Element {
		self.build(None)
	}

	fn build(self, action: Option<Action>) -> Element {
		let Self {
			label,
			detail,
			spot,
			state,
			reason,
			test_id,
		} = self;
		let width = measure();
		let height = space::PRIMARY as f32;
		let seed = seed_for(&label);
		let steady = steady();
		let disabled = state.is_disabled();

		let accessible = match &detail {
			Some(detail) => format!("{label}, {detail}"),
			None => label.clone(),
		};
		let words = if disabled { ASH } else { KEY };
		let room = if detail.is_some() {
			(width - 2.0 * INSET) * 0.62
		} else {
			width - 2.0 * INSET
		};
		let size = fit_display_line(&label, room, Step::S6);
		let mut face = Pressable(accessible)
			.class(format!(
				"relative w-full min-h-[{height}px] flex-row items-center justify-between gap-3 pl-[{INSET}px] pr-[16px] {} {}",
				snap(),
				primary_hand(state, steady)
			))
			.child(primary_plate(width, height, seed, disabled).over())
			.child(display_words(label, size, words).class("shrink"));
		if let Some(detail) = detail {
			face = face.child(
				View()
					.class("flex-row items-center gap-2")
					.child(crate::ui::text::label(&detail, Step::S2.size(), words))
					.child(arrow(words, seed.wrapping_add(3))),
			);
		}
		if let Some(id) = test_id {
			face = face.test_id(id);
		}
		face = match (action, state) {
			(Some(action), State::Rest) => face.on_press(action),
			(_, State::Busy) => face.aria_busy(true).enabled(false),
			(_, State::Disabled) => face.enabled(false),
			_ => face,
		};
		if state == State::Focused {
			face = face.child(crop_marks());
		}

		let mut button = View().class("relative w-full");
		if !disabled {
			button = button.child(spot_plate(spot, steady));
		}
		button = button.child(face);
		match reason {
			Some(reason) if disabled => {
				View()
					.class("w-full gap-2")
					.child(button)
					.child(aside(reason))
			}
			_ => button,
		}
	}
}

/// What the hand does to the primary's face: it slides into register with
/// the spot, or, under reduced motion, its lip darkens (§14). The lip is
/// inked in the face's text colour.
fn primary_hand(state: State, steady: bool) -> String {
	let register = space::REGISTER;
	let key = KEY.hex();
	match (state, steady) {
		(State::Busy, _) | (State::Pressed, true) => format!("text-[{key}]"),
		(State::Rest, true) => format!("text-[{LIP_AT_REST}] active:text-[{key}]"),
		(State::Pressed, false) => {
			format!("text-[{LIP_AT_REST}] translate-x-[{register}px] translate-y-[{register}px]")
		}
		(State::Rest, false) => {
			format!(
				"text-[{LIP_AT_REST}] active:translate-x-[{register}px] active:translate-y-[{register}px]"
			)
		}
		(State::Focused | State::Disabled, _) => format!("text-[{LIP_AT_REST}]"),
	}
}

/// The primary's key plate: the rough frame and the hatched lip, or, when
/// it is disabled, a key hatch over raised in place of the spot.
fn primary_plate(width: f32, height: f32, seed: u32, disabled: bool) -> Plate {
	let frame = Recipe::twice(Mark::frame(width, height, 4.0), seed);
	if disabled {
		return Plate::new(width, height)
			.flood(0.0, 0.0, width, height, RAISED)
			.stroke(
				&Recipe::once(
					Mark::hatch_box(0.0, 0.0, width, height, 4.0, 60),
					seed.wrapping_add(2),
				),
				KEY.color_alpha(0.55),
				1.2,
			)
			.stroke(&frame, KEY, stroke::PRIMARY);
	}
	let lip = Recipe::once(
		Mark::hatch_box(0.0, height - LIP, width, LIP, 4.0, 60),
		seed.wrapping_add(1),
	);
	Plate::new(width, height)
		.stroke(&lip, Ink::Current, 1.1)
		.stroke(&frame, KEY, stroke::PRIMARY)
}

/// The spot plate under the primary: its own layer, so misregistration is
/// a translate of it and nothing is redrawn (§8). Out of register by 3
/// points, or in register under reduced motion.
fn spot_plate(spot: Rgb, steady: bool) -> Element {
	let register = space::REGISTER;
	let offset = if steady {
		String::new()
	} else {
		format!("translate-x-[{register}px] translate-y-[{register}px]")
	};
	View()
		.class(format!("absolute inset-0 bg-[{}] {offset}", spot.hex()))
		.enabled(false)
		.aria_hidden(true)
}

/// A secondary action: a rough paper outline on the page, the label in
/// Text, and a figure or a drawn arrow at its right edge. As a destructive
/// action it carries a wound strike under its words, which say the loss.
///
/// - **Pressed:** a paper hatch at 60° inside, at 60%.
/// - **Focused:** crop marks.
/// - **Busy:** the hatch prints and it takes no second press.
/// - **Disabled:** a dashed ash outline, and an ash label.
pub struct Secondary {
	label: String,
	detail: Option<String>,
	state: State,
	destructive: bool,
	test_id: Option<String>,
}

/// The hatch a pressed outline shows: paper at 60%.
const PRESSED_HATCH: &str = "#ece3cf99";

impl Secondary {
	pub fn new(label: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			detail: None,
			state: State::Rest,
			destructive: false,
			test_id: None,
		}
	}

	/// A figure at the right edge: a price, a count.
	#[must_use]
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Prints the button in `state`.
	#[must_use]
	pub const fn state(mut self, state: State) -> Self {
		self.state = state;
		self
	}

	/// A destructive action (release, sell, leave): a wound strike under the
	/// words. Its words must say the loss, and a sheet confirms it.
	#[must_use]
	pub const fn destructive(mut self) -> Self {
		self.destructive = true;
		self
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The button, doing `action` when it is pressed at rest.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	/// A print of one of its states, with nothing to do.
	pub fn print(self) -> Element {
		self.build(None)
	}

	fn build(self, action: Option<Action>) -> Element {
		let Self {
			label,
			detail,
			state,
			destructive,
			test_id,
		} = self;
		let width = measure();
		let height = space::SECONDARY as f32;
		let seed = seed_for(&label);
		let disabled = state.is_disabled();
		let words = if disabled { ASH } else { PAPER };
		let accessible = match &detail {
			Some(detail) => format!("{label}, {detail}"),
			None => label.clone(),
		};

		let hatch_ink = match state {
			State::Pressed | State::Busy => format!("text-[{PRESSED_HATCH}]"),
			State::Rest => format!("text-[#00000000] active:text-[{PRESSED_HATCH}]"),
			State::Focused | State::Disabled => "text-[#00000000]".to_owned(),
		};
		let frame = Recipe::twice(Mark::frame(width, height, 2.5), seed);
		let hatch = Recipe::once(
			Mark::hatch_box(2.0, 2.0, width - 4.0, height - 4.0, 4.0, 60),
			seed.wrapping_add(1),
		);
		let plate = Plate::new(width, height).stroke(&hatch, Ink::Current, 1.1);
		let plate = if disabled {
			plate.dashed(&frame, ASH, stroke::SLIP, 5.0, 5.0)
		} else {
			plate.stroke(&frame, PAPER, stroke::SLIP)
		};

		let mut words_box = View()
			.class("relative shrink")
			.child(body(label, 17, words));
		if destructive {
			words_box = words_box.child(
				View()
					.class("absolute left-0 right-0 bottom-[-6px] h-[6px]")
					.child(
						Plate::new(100.0, 6.0)
							.stroke(
								&Recipe::once(
									Mark::line(1.0, 3.0, 99.0, 3.6),
									seed.wrapping_add(4),
								),
								WOUND,
								stroke::EMPHASIS,
							)
							.over(),
					),
			);
		}
		let mut button = Pressable(accessible)
			.class(format!(
				"relative w-full min-h-[{height}px] flex-row flex-wrap items-center justify-between gap-x-4 gap-y-1 px-4 py-[10px] {hatch_ink} {}",
				snap()
			))
			.child(plate.over())
			.child(words_box);
		if let Some(detail) = detail {
			button = button.child(figure(detail, Step::S3.size(), words));
		} else if !disabled {
			button = button.child(arrow(words, seed.wrapping_add(3)));
		}
		if let Some(id) = test_id {
			button = button.test_id(id);
		}
		if state == State::Focused {
			button = button.child(crop_marks());
		}
		match (action, state) {
			(Some(action), State::Rest) => button.on_press(action),
			(_, State::Busy) => button.aria_busy(true).enabled(false),
			(_, State::Disabled) => button.enabled(false),
			_ => button,
		}
	}
}

/// A quiet action: words in Text with a one-pass paper rule under them, for
/// the lesser of two choices and for links. A press thickens the rule to
/// 2.2 points.
pub struct Quiet {
	label: String,
	state: State,
	test_id: Option<String>,
}

impl Quiet {
	pub fn new(label: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			state: State::Rest,
			test_id: None,
		}
	}

	/// Prints the action in `state`.
	#[must_use]
	pub const fn state(mut self, state: State) -> Self {
		self.state = state;
		self
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The action, doing `action` when it is pressed at rest.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	/// A print of one of its states, with nothing to do.
	pub fn print(self) -> Element {
		self.build(None)
	}

	fn build(self, action: Option<Action>) -> Element {
		let Self {
			label,
			state,
			test_id,
		} = self;
		let seed = seed_for(&label);
		let disabled = state.is_disabled();
		let words = if disabled { ASH } else { PAPER };
		let thick = match state {
			State::Pressed | State::Busy => format!("text-[{}]", PAPER.hex()),
			State::Rest => format!("text-[#00000000] active:text-[{}]", PAPER.hex()),
			State::Focused | State::Disabled => "text-[#00000000]".to_owned(),
		};
		let rule = Recipe::once(Mark::line(0.5, 3.0, 99.5, 3.4), seed);
		let mut underline = Plate::new(100.0, 6.0).stroke(&rule, Ink::Current, stroke::EMPHASIS);
		if !disabled {
			underline = underline.stroke(&rule, PAPER, stroke::LINK);
		}
		let mut quiet = Pressable(label.clone())
			.class(format!(
				"relative min-h-[{}px] justify-center self-start px-1 {thick}",
				space::TARGET
			))
			.child(
				View()
					.class("relative")
					.child(body(label, Step::S3.size(), words))
					.child(
						View()
							.class("absolute left-0 right-0 bottom-[-5px] h-[6px]")
							.child(underline.over()),
					),
			);
		if let Some(id) = test_id {
			quiet = quiet.test_id(id);
		}
		if state == State::Focused {
			quiet = quiet.child(crop_marks());
		}
		match (action, state) {
			(Some(action), State::Rest) => quiet.on_press(action),
			(_, State::Busy) => quiet.aria_busy(true).enabled(false),
			(_, State::Disabled) => quiet.enabled(false),
			_ => quiet,
		}
	}
}
