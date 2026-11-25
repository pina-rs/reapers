//! Form fields (§8): the input, and the switch, checkbox and radio.

use leptoskit::native::Invalid;
use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::Action;
use super::State;
use super::crop_marks;
use super::measure;
use super::with_tap;
use crate::l10n::use_l10n;
use crate::theme::ASH;
use crate::theme::PAPER;
use crate::theme::Rgb;
use crate::theme::Step;
use crate::theme::WOUND;
use crate::theme::seed_for;
use crate::theme::stroke;
use crate::ui::ink::Ink;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::text::body;
use crate::ui::text::label;
use crate::ui::text::wound_words;

/// What a field does with its words as they are written.
type OnChange = Box<dyn Fn(&str) + Send + Sync + 'static>;

/// What an input holds, and how it is printed (§8).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FieldState {
	/// Empty, waiting for words.
	#[default]
	Empty,
	/// Being written in: crop marks and a 2.2-point outline. A live field
	/// shows them whenever it has the keyboard.
	Focused,
	/// Holding words.
	Filled,
	/// Holding words that will not do, and saying why: a torn wound
	/// underline, and the reason in paper beside a drawn wound cross.
	Error(String),
	/// Not available: a dashed ash outline.
	Disabled,
}

/// A text input: a paper outline on the page, the words in Text, and its
/// label in Figures above it, in ash.
pub struct Field {
	label: String,
	value: String,
	placeholder: Option<String>,
	state: FieldState,
	test_id: String,
}

impl Field {
	/// A field named `label`, known to tests as `test_id`. Its box takes
	/// `test_id`, and the input inside it `test_id` and `-input`.
	pub fn new(label: impl Into<String>, test_id: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			value: String::new(),
			placeholder: None,
			state: FieldState::Empty,
			test_id: test_id.into(),
		}
	}

	#[must_use]
	pub fn value(mut self, value: impl Into<String>) -> Self {
		self.value = value.into();
		self
	}

	#[must_use]
	pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
		self.placeholder = Some(placeholder.into());
		self
	}

	/// Prints the field in `state`.
	#[must_use]
	pub fn state(mut self, state: FieldState) -> Self {
		self.state = state;
		self
	}

	/// The field, calling `on_change` with its words as they are written.
	pub fn on_change(self, on_change: impl Fn(&str) + Send + Sync + 'static) -> Element {
		self.build(Some(Box::new(on_change)))
	}

	/// A print of one of its states, taking no words.
	pub fn print(self) -> Element {
		self.build(None)
	}

	fn build(self, on_change: Option<OnChange>) -> Element {
		const HEIGHT: f32 = 48.0;
		let Self {
			label: name,
			value,
			placeholder,
			state,
			test_id,
		} = self;
		let width = measure();
		let seed = seed_for(&test_id);
		let disabled = state == FieldState::Disabled;
		let frame = Recipe::twice(Mark::frame(width, HEIGHT, 2.5), seed);
		// The focus outline is its own layer in the box's text colour, clear
		// at rest; the web shell inks it while the input has the keyboard.
		let focus_ink = if state == FieldState::Focused {
			format!("text-[{}]", PAPER.hex())
		} else {
			"text-[#00000000]".to_owned()
		};
		let plate = Plate::new(width, HEIGHT).stroke(&frame, Ink::Current, stroke::EMPHASIS);
		let plate = if disabled {
			plate.dashed(&frame, ASH, stroke::SLIP, 5.0, 5.0)
		} else {
			plate.stroke(&frame, PAPER, stroke::SLIP)
		};
		let words = if disabled { ASH } else { PAPER };
		let mut input = TextInput()
			.class(format!(
				"w-full bg-[#00000000] font-[var(--text)] text-[1.0625rem] text-[{}]",
				words.hex()
			))
			.value(value)
			.accessibility_label(name.clone())
			.test_id(format!("{test_id}-input"));
		if let Some(placeholder) = placeholder {
			input = input.placeholder(placeholder);
		}
		if let FieldState::Error(_) = state {
			input = input.aria_invalid(Invalid::True);
		}
		input = match on_change {
			Some(on_change) if !disabled => input.on_change(move |text| on_change(text)),
			_ => input.enabled(!disabled),
		};
		let mut field = View()
			.class(format!(
				"relative w-full min-h-[{HEIGHT}px] justify-center px-3 {focus_ink}"
			))
			.test_id(test_id)
			.child(plate.over())
			.child(input);
		if state == FieldState::Focused {
			field = field.child(crop_marks());
		}
		let mut column = View()
			.class("w-full gap-2")
			.child(label(&name, Step::S2.size(), ASH))
			.child(field);
		if let FieldState::Error(reason) = state {
			let tear: Vec<(f32, f32)> = (0..=24)
				.map(|step| {
					let x = step as f32 * width / 24.0;
					let y = if step % 2 == 0 { 1.0 } else { 3.5 };
					(x, y)
				})
				.collect();
			column = column
				.child(
					View()
						.class("relative w-full h-[5px] mt-[-6px]")
						.aria_hidden(true)
						.child(
							Plate::new(width, 5.0)
								.stroke(
									&Recipe::once(Mark::path(&tear, false), seed.wrapping_add(1)),
									WOUND,
									stroke::EMPHASIS,
								)
								.over(),
						),
				)
				.child(wound_words(
					body(reason, Step::S3.size(), PAPER),
					Step::S3.size(),
				));
		}
		column
	}
}

/// What kind of choice a toggle is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToggleKind {
	/// On or off, at once.
	Switch,
	/// One of several that may all be chosen.
	Checkbox,
	/// One of several, only one of which is chosen.
	Radio,
}

/// A switch, a checkbox or a radio: a paper outline on the page, the word
/// "On" or "Off" in Figures beside it, and its label, in a target at least
/// 44 points tall (§8).
///
/// - **On:** spot hachure inside, a paper knob or tick, and "On".
/// - **Off:** an ash outline, and "Off".
/// - **Pressed:** the hachure prints at once, held.
/// - **Focused:** crop marks.
/// - **Disabled:** dashed, in ash.
pub struct Toggle {
	kind: ToggleKind,
	label: String,
	on: bool,
	spot: Rgb,
	state: State,
	test_id: Option<String>,
}

impl Toggle {
	pub fn new(kind: ToggleKind, label: impl Into<String>, on: bool, spot: Rgb) -> Self {
		Self {
			kind,
			label: label.into(),
			on,
			spot,
			state: State::Rest,
			test_id: None,
		}
	}

	/// Prints the toggle in `state`.
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

	/// The toggle, doing `action` when it is pressed at rest.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	/// A print of one of its states, with nothing to do.
	pub fn print(self) -> Element {
		self.build(None)
	}

	/// The drawn control: its outline, its hachure in the toggle's text
	/// colour, and its knob or tick.
	fn control(&self, seed: u32) -> (Element, f32) {
		let disabled = self.state.is_disabled();
		let outline = if disabled || !self.on { ASH } else { PAPER };
		let (width, height) = match self.kind {
			ToggleKind::Switch => (44.0, 24.0),
			ToggleKind::Checkbox | ToggleKind::Radio => (24.0, 24.0),
		};
		let shape = match self.kind {
			ToggleKind::Switch | ToggleKind::Checkbox => Mark::frame(width, height, 1.5),
			ToggleKind::Radio => Mark::circle(12.0, 12.0, 10.5),
		};
		let inside: Vec<(f32, f32)> = match self.kind {
			ToggleKind::Switch | ToggleKind::Checkbox => {
				vec![
					(2.0, 2.0),
					(width - 2.0, 2.0),
					(width - 2.0, height - 2.0),
					(2.0, height - 2.0),
				]
			}
			ToggleKind::Radio => {
				(0..12)
					.map(|step| {
						let angle = step as f32 / 12.0 * std::f32::consts::TAU;
						(12.0 + 8.5 * angle.cos(), 12.0 + 8.5 * angle.sin())
					})
					.collect()
			}
		};
		let frame = Recipe::twice(shape, seed);
		let mut plate = Plate::new(width, height).stroke(
			&Recipe::once(Mark::hatch(&inside, 3.0, 60), seed.wrapping_add(1)),
			Ink::Current,
			1.3,
		);
		plate = if disabled {
			plate.dashed(&frame, ASH, stroke::TOGGLE, 4.0, 4.0)
		} else {
			plate.stroke(&frame, outline, stroke::TOGGLE)
		};
		if self.on && !disabled {
			plate = match self.kind {
				ToggleKind::Switch => plate.flood(width - 20.0, 4.0, 16.0, 16.0, PAPER),
				ToggleKind::Checkbox => {
					plate.stroke(
						&Recipe::twice(
							Mark::path(&[(5.0, 12.5), (10.0, 17.5), (19.5, 5.5)], false),
							seed.wrapping_add(2),
						),
						PAPER,
						stroke::EMPHASIS,
					)
				}
				ToggleKind::Radio => {
					plate.fill(&Recipe::once(Mark::waxing(12.0, 12.0, 4.5, 1.0), 0), PAPER)
				}
			};
		} else if self.kind == ToggleKind::Switch {
			plate = plate.stroke(
				&Recipe::once(Mark::frame(16.0, 16.0, 0.0), seed.wrapping_add(3)),
				outline,
				1.2,
			);
		}
		(plate.sized(), width)
	}

	fn build(self, action: Option<Action>) -> Element {
		let words = use_l10n();
		let seed = seed_for(&self.label);
		let disabled = self.state.is_disabled();
		let (control, _) = self.control(seed);
		let hachure = if (self.on && !disabled) || self.state == State::Pressed {
			format!("text-[{}]", self.spot.hex())
		} else if self.state == State::Rest {
			format!("text-[#00000000] active:text-[{}]", self.spot.hex())
		} else {
			"text-[#00000000]".to_owned()
		};
		let role = match self.kind {
			ToggleKind::Switch => Role::Switch,
			ToggleKind::Checkbox => Role::Checkbox,
			ToggleKind::Radio => Role::Radio,
		};
		let name = if disabled { ASH } else { PAPER };
		let word = if self.on {
			words.common_on()
		} else {
			words.common_off()
		};
		let word_colour = if self.on && !disabled { PAPER } else { ASH };
		let mut toggle = Pressable(self.label.clone())
			.class(format!(
				"relative w-full min-h-[44px] flex-row items-center gap-3 {hachure}"
			))
			.role(role)
			.aria_checked(self.on)
			.child(control)
			.child(
				View()
					.class("w-[30px]")
					.child(label(word, Step::S1.size(), word_colour)),
			)
			.child(body(self.label, 17, name).class("flex-1"));
		if let Some(id) = self.test_id {
			toggle = toggle.test_id(id);
		}
		if self.state == State::Focused {
			toggle = toggle.child(crop_marks());
		}
		match (action, self.state) {
			(Some(action), State::Rest) => toggle.on_press(action),
			(_, State::Disabled) => toggle.enabled(false),
			_ => toggle,
		}
	}
}
