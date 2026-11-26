//! The surfaces (§8): the list row and its divider, the paper slip, and
//! the paper sheet.

use leptoskit::prelude::*;

use super::Action;
use super::State;
use super::crop_marks;
use super::marks::arrow;
use super::measure;
use super::rise;
use super::steady;
use super::with_tap;
use crate::theme::ASH;
use crate::theme::KEY;
use crate::theme::PAPER;
use crate::theme::RAISED;
use crate::theme::Step;
use crate::theme::WOUND;
use crate::theme::motion;
use crate::theme::seed_for;
use crate::theme::stroke;
use crate::ui::ink::Ink;
use crate::ui::ink::Mark;
use crate::ui::ink::Plate;
use crate::ui::ink::Recipe;
use crate::ui::text::body;
use crate::ui::text::figure;
use crate::ui::text::label;

/// A rule between rows: raised, 1.4 points, one pass, with a slight bow
/// (§8). It is drawn at the page's measure.
pub fn divider(seed: u32) -> Element {
	let width = measure();
	View()
		.class("relative w-full h-[4px]")
		.aria_hidden(true)
		.child(
			Plate::new(width, 4.0)
				.stroke(
					&Recipe::once(Mark::line(0.0, 2.0, width, 2.4), seed),
					RAISED,
					stroke::DIVIDER,
				)
				.over(),
		)
}

/// One row of a ledger: a name in Text, an optional line of status words
/// under it, a figure at the right, and a drawn arrow when it goes
/// somewhere. Rows are ruled apart by a [`divider`] at their foot.
///
/// - **Pressed:** the row prints on raised. Every word on a row is paper,
///   which reads on raised (§15.1).
/// - **Focused:** crop marks.
/// - **Busy:** printed pressed, taking no second press.
/// - **Disabled:** ash, without the arrow.
pub struct Row {
	name: String,
	status: Option<String>,
	figure: Option<String>,
	state: State,
	test_id: Option<String>,
}

impl Row {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			status: None,
			figure: None,
			state: State::Rest,
			test_id: None,
		}
	}

	/// Status words under the name: "Held", "Loose".
	#[must_use]
	pub fn status(mut self, status: impl Into<String>) -> Self {
		self.status = Some(status.into());
		self
	}

	/// A figure at the right edge.
	#[must_use]
	pub fn figure(mut self, figure: impl Into<String>) -> Self {
		self.figure = Some(figure.into());
		self
	}

	/// Prints the row in `state`.
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

	/// The row, opening what it names when it is pressed at rest.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	/// A print of one of its states, with nothing to do.
	pub fn print(self) -> Element {
		self.build(None)
	}

	fn build(self, action: Option<Action>) -> Element {
		let Self {
			name,
			status,
			figure: figure_text,
			state,
			test_id,
		} = self;
		let seed = seed_for(&name);
		let disabled = state.is_disabled();
		let words = if disabled { ASH } else { PAPER };
		let accessible = [
			Some(name.as_str()),
			status.as_deref(),
			figure_text.as_deref(),
		]
		.into_iter()
		.flatten()
		.collect::<Vec<_>>()
		.join(", ");
		let ground = match state {
			State::Pressed | State::Busy => format!("bg-[{}]", RAISED.hex()),
			State::Rest => format!("active:bg-[{}]", RAISED.hex()),
			State::Focused | State::Disabled => String::new(),
		};
		let mut names = View()
			.class("flex-1 gap-[2px]")
			.child(body(name, 17, words));
		if let Some(status) = status {
			names = names.child(label(&status, Step::S2.size(), words));
		}
		let mut row = Pressable(accessible)
			.class(format!(
				"relative w-full min-h-[56px] flex-row items-center gap-3 px-1 py-3 {ground}"
			))
			.child(names);
		if let Some(figure_text) = figure_text {
			row = row.child(figure(figure_text, Step::S3.size(), words));
		}
		if !disabled {
			row = row.child(arrow(words, seed.wrapping_add(3)));
		}
		row = row.child(
			View()
				.class("absolute left-0 right-0 bottom-[-2px]")
				.child(divider(seed)),
		);
		if let Some(id) = test_id {
			row = row.test_id(id);
		}
		if state == State::Focused {
			row = row.child(crop_marks());
		}
		match (action, state) {
			(Some(action), State::Rest) => row.on_press(action),
			(_, State::Busy) => row.aria_busy(true).enabled(false),
			(_, State::Disabled) => row.enabled(false),
			_ => row,
		}
	}
}

/// Where a slip is in its life (§8, §13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SlipState {
	/// Pinned, and rising into place the first time it is shown: 8 points
	/// in 333 ms, or in place under reduced motion.
	#[default]
	Rest,
	/// Printed where it rises from, 8 points low, for the kit page.
	Entering,
	/// Printed at the last of its three shrinks as it goes, for the kit
	/// page.
	Leaving,
	/// Held down, when the slip is a control: a key hatch inside.
	Pressed,
}

/// A slip: opaque paper with a rough key outline, turned by its seed, and
/// key words on it (§8). Every hunt word is set on one. An error slip has
/// its left edge torn in wound.
pub struct Slip {
	words: String,
	state: SlipState,
	torn: bool,
	test_id: Option<String>,
}

impl Slip {
	pub fn new(words: impl Into<String>) -> Self {
		Self {
			words: words.into(),
			state: SlipState::Rest,
			torn: false,
			test_id: None,
		}
	}

	/// Prints the slip in `state`.
	#[must_use]
	pub const fn state(mut self, state: SlipState) -> Self {
		self.state = state;
		self
	}

	/// An error notice: the slip's left edge torn in wound. Its words say
	/// what went wrong.
	#[must_use]
	pub const fn torn(mut self) -> Self {
		self.torn = true;
		self
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The slip as words on paper.
	pub fn view(self) -> Element {
		self.build(None)
	}

	/// The slip as a control, doing `action` when it is pressed.
	pub fn on_press(self, action: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(Some(with_tap(action)))
	}

	fn build(self, action: Option<Action>) -> Element {
		const WIDTH: f32 = 300.0;
		const HEIGHT: f32 = 48.0;
		let Self {
			words,
			state,
			torn,
			test_id,
		} = self;
		let seed = seed_for(&words);
		// Turned by its seed, within 2° either way.
		let tenths = seed % 41;
		let turn = if tenths >= 20 {
			format!("rotate-[{:.1}deg]", (tenths - 20) as f32 / 10.0)
		} else {
			format!("-rotate-[{:.1}deg]", (20 - tenths) as f32 / 10.0)
		};
		let motion_class = match state {
			SlipState::Rest => rise(motion::RISE_PT),
			SlipState::Entering => format!("translate-y-[{}px]", motion::RISE_PT),
			SlipState::Leaving => "scale-[0.82]".to_owned(),
			SlipState::Pressed => String::new(),
		};
		let hatch_ink = match (state, action.is_some()) {
			(SlipState::Pressed, _) => format!("text-[{}]", KEY.hex()),
			(_, true) => format!("text-[#00000000] active:text-[{}]", KEY.hex()),
			_ => "text-[#00000000]".to_owned(),
		};
		let mut plate = Plate::new(WIDTH, HEIGHT)
			.stroke(
				&Recipe::once(
					Mark::hatch_box(3.0, 3.0, WIDTH - 6.0, HEIGHT - 6.0, 4.0, 60),
					seed.wrapping_add(1),
				),
				Ink::Current,
				1.0,
			)
			.stroke(
				&Recipe::twice(Mark::frame(WIDTH, HEIGHT, 2.5), seed),
				KEY,
				stroke::SLIP,
			);
		if torn {
			let points: Vec<(f32, f32)> = (0..=12)
				.map(|step| {
					let y = step as f32 * HEIGHT / 12.0;
					let x = if step % 2 == 0 { 1.0 } else { 4.5 };
					(x, y)
				})
				.collect();
			plate = plate.stroke(
				&Recipe::once(Mark::path(&points, false), seed.wrapping_add(2)),
				WOUND,
				stroke::EMPHASIS,
			);
		}
		let inset = if torn { "pl-[22px]" } else { "pl-4" };
		let content = |element: Element| {
			element
				.class(format!(
					"relative self-center justify-center min-h-[{HEIGHT}px] max-w-full bg-[{}] {inset} pr-4 py-[10px] {turn} {motion_class} {hatch_ink}",
					PAPER.hex()
				))
				.child(plate.over())
				.child(body(words.clone(), Step::S3.size(), KEY))
		};
		let mut slip = match action {
			Some(action) => content(Pressable(words.clone())).on_press(action),
			None => content(View()),
		};
		if let Some(id) = test_id {
			slip = slip.test_id(id);
		}
		slip
	}
}

/// A sheet risen from the foot of the page: full paper, key words, a
/// deckled top edge and a key handle 36 by 3 points, with no rounded corners
/// and nothing dimmed behind it (§8).
///
/// It rises 4 held steps over 333 ms, or appears in place under reduced
/// motion. Dragging it and its settle when let go are not built yet:
/// leptoskit has no drag gesture a sheet can follow.
pub struct PaperSheet {
	seed: u32,
	children: Vec<Element>,
	test_id: Option<String>,
}

impl PaperSheet {
	pub fn new(seed: u32, children: Vec<Element>) -> Self {
		Self {
			seed,
			children,
			test_id: None,
		}
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The sheet, `risen` as it enters, or printed open.
	pub fn view(self, risen: bool) -> Element {
		const DECKLE: f32 = 7.0;
		let width = measure() + 40.0;
		// A torn top edge, cut by the seed: never the same twice, always the
		// same for one sheet.
		let mut state = self.seed | 1;
		let mut points = vec![(0.0, DECKLE + 1.0)];
		let mut x = 0.0;
		while x < width {
			state ^= state << 13;
			state ^= state >> 17;
			state ^= state << 5;
			let lift = (state % 100) as f32 / 100.0 * (DECKLE - 1.5);
			points.push((x, 1.0 + lift));
			x += 3.0 + (state % 7) as f32 * 0.6;
		}
		points.push((width, DECKLE));
		points.push((width, DECKLE + 1.0));
		let entering = if risen && !steady() {
			format!(
				"starting:translate-y-[120px] transition-transform duration-[{}ms] ease-out",
				motion::RISE_MS
			)
		} else {
			String::new()
		};
		let mut sheet = View()
			.class(format!("relative w-full items-stretch {entering}"))
			.role(leptoskit::native::Role::Dialog)
			.child(
				View()
					.class(format!("relative w-full h-[{DECKLE}px] mb-[-1px]"))
					.aria_hidden(true)
					.child(Plate::new(width, DECKLE + 1.0).shape(&points, PAPER).over()),
			)
			.child(
				View()
					.class(format!(
						"w-full bg-[{}] px-5 pt-3 pb-safe-offset-6 gap-3",
						PAPER.hex()
					))
					.child(
						View()
							.class(format!(
								"self-center w-[36px] h-[3px] bg-[{}] mb-2",
								KEY.hex()
							))
							.aria_hidden(true),
					)
					.children(self.children),
			);
		if let Some(id) = self.test_id {
			sheet = sheet.test_id(id);
		}
		sheet
	}
}
