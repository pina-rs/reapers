//! The pieces every screen is ruled with: buttons, rows, meters and marks,
//! all drawn like entries in a ledger.

use leptoskit::prelude::*;

use super::light;
use super::sketch::sketch;
use super::text::body;
use super::text::display;
use super::text::figure;
use super::text::label;
use super::text::margin;
use crate::l10n::use_l10n;
use crate::platform;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::CONTENT_WIDTH;
use crate::theme::Rgb;
use crate::theme::class;
use crate::theme::seed_for;

/// A full-width action. It is an outline, drawn by hand, with a figure at
/// its right edge: a price, a count, or an arrow when there is nothing to
/// count.
pub struct LedgerButton {
	label: String,
	detail: Option<String>,
	quiet: bool,
	test_id: Option<String>,
}

impl LedgerButton {
	pub fn new(label: impl Into<String>) -> Self {
		Self {
			label: label.into(),
			detail: None,
			quiet: false,
			test_id: None,
		}
	}

	/// A figure shown at the right edge.
	#[must_use]
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// A quiet button has no outline, for the lesser of two choices.
	#[must_use]
	pub const fn quiet(mut self) -> Self {
		self.quiet = true;
		self
	}

	#[must_use]
	pub fn test_id(mut self, id: impl Into<String>) -> Self {
		self.test_id = Some(id.into());
		self
	}

	/// The button, doing `on_press` when it is pressed.
	pub fn on_press(self, on_press: impl Fn() + Send + Sync + 'static) -> Element {
		self.build(true).on_press(move || {
			platform::tap();
			on_press();
		})
	}

	/// The button drawn as unavailable.
	pub fn disabled(self) -> Element {
		self.build(false).enabled(false)
	}

	fn build(self, enabled: bool) -> Element {
		let foreground = if enabled { BONE } else { ASH };
		let accessible = match &self.detail {
			Some(detail) => format!("{}, {detail}", self.label),
			None => self.label.clone(),
		};
		let mut button = Pressable(accessible)
			.class("relative w-full min-h-[56px] px-[18px] py-4 flex-row flex-wrap items-center justify-between gap-x-4 gap-y-[6px]");
		if !self.quiet {
			let rule_alpha = if enabled { 0.7 } else { 0.18 };
			button = button.child(sketch(seed_for(&self.label), BONE, rule_alpha));
		}
		button = button.child(label(&self.label, 13, foreground));
		if let Some(detail) = &self.detail {
			button = button.child(figure(detail.clone(), 13, foreground));
		} else if enabled {
			button = button.child(figure("→", 16, foreground));
		}
		if let Some(id) = self.test_id {
			button = button.test_id(id);
		}
		button
	}
}

/// The way back, at the top of a screen.
pub fn ledger_back(label_text: Option<&str>) -> Element {
	let words = use_l10n();
	let text = match label_text {
		Some(label_text) => format!("←  {}", label_text.to_uppercase()),
		None => "←".to_owned(),
	};
	Pressable(label_text.unwrap_or(words.common_back()).to_owned())
		.class("min-w-[48px] min-h-[48px] justify-center self-start")
		.test_id("ledger-back")
		.on_press(crate::screens::back)
		.child(label(&text, 13, BONE))
}

/// One line of a ledger: a name on the left, a figure on the right, and a
/// rule of dots between them.
pub fn ledger_row(name: &str, value: impl Into<String>, emphasised: bool) -> Element {
	let colour = if emphasised { light() } else { BONE };
	View()
		.class("flex-row items-end py-[7px] flex-wrap")
		.child(margin(name))
		.child(leader())
		.child(figure(value, 15, colour))
}

/// The row of dots that leads the eye from a name to its figure.
fn leader() -> Element {
	let dot = Stroke::new(Color::rgba(0xE9, 0xE4, 0xD4, 0x2E), 1.2)
		.expect("a positive width")
		.round()
		.dashes(&[0.01, 5.0], 0.0)
		.expect("positive dashes");
	Vector(ViewBox::new(0.0, 0.0, 100.0, 2.0).expect("a positive size"))
		.scaling(VectorScaling::Stretch)
		.path(
			VectorPath::new(
				PathData::line(
					leptoskit::native::vector::Point::new(0.0, 1.0),
					leptoskit::native::vector::Point::new(100.0, 1.0),
				)
				.expect("a finite line"),
			)
			.stroke(dot),
		)
		.class("flex-1 min-w-[12px] h-[2px] mx-[10px] mb-[4px]")
		.aria_hidden(true)
}

/// A thin bar with a name: how much of something is left, from 0 to 1.
pub fn ledger_meter(name: &str, fraction: f64, colour: Rgb) -> Element {
	let percent = (fraction.clamp(0.0, 1.0) * 100.0).round() as u32;
	View()
		.class("flex-row items-center")
		.accessibility_label(name.to_owned())
		.aria_value_text(format!("{percent}%"))
		.child(View().class("w-[62px]").child(margin(name)))
		.child(
			View()
				.class("relative flex-1 h-[5px]")
				.child(View().class(format!(
					"absolute left-0 right-0 top-[2px] h-px {}",
					class::HAIRLINE_BG
				)))
				.child(View().class(format!(
					"absolute left-0 top-0 h-full w-[{percent}%] bg-[{}]",
					colour.hex()
				)))
				.child(View().class(format!(
					"absolute top-[-3px] h-[11px] w-[2px] left-[{percent}%] ml-[-1px] bg-[{}]",
					BONE.hex()
				))),
		)
}

/// A small diamond that is either lit or spent.
pub fn ledger_pip(lit: bool, colour: Rgb) -> Element {
	let pip = View().class(format!(
		"w-2 h-2 rotate-45 border border-[{}]",
		if lit { colour.hex() } else { ASH.hex() }
	));
	if lit {
		pip.class(format!("bg-[{}]", colour.hex()))
	} else {
		pip
	}
}

/// A heading in the margin, with a rule running to the edge of the page and
/// a figure at its end.
pub fn ledger_heading(text: &str, trailing: Option<String>) -> Element {
	let mut row = View()
		.class("flex-row items-center gap-3 pt-[22px] pb-[10px]")
		.child(label(text, 11, BONE).class("shrink"))
		.child(View().class(format!("flex-1 h-px {}", class::HAIRLINE_BG)));
	if let Some(trailing) = trailing {
		row = row.child(figure(trailing, 12, ASH));
	}
	row
}

/// A whole page given over to one thing that went wrong, with the way out.
pub fn ledger_notice(
	title: &str,
	text: &str,
	action: &str,
	on_action: impl Fn() + Send + Sync + 'static,
) -> Element {
	View()
		.class("flex-1 min-h-screen justify-center p-7 gap-[14px]")
		.child(display(title, 40, BONE))
		.child(body(text, 15, ASH).class("mb-[14px]"))
		.child(
			LedgerButton::new(action)
				.test_id("notice-action")
				.on_press(on_action),
		)
}

/// A page of the ledger: a way back, a heading in two sizes, and a column of
/// content that scrolls.
pub fn ledger_page(
	overline: &str,
	title: Option<&str>,
	trailing: Option<Element>,
	children: Vec<Element>,
) -> Element {
	let mut top = View()
		.class("flex-row items-center justify-between")
		.child(ledger_back(None));
	if let Some(trailing) = trailing {
		top = top.child(trailing);
	}
	let mut column = View()
		.class(format!(
			"w-full {CONTENT_WIDTH} self-center px-6 pt-2 pb-10"
		))
		.child(top.class("mb-[18px]"))
		.child(margin(overline));
	if let Some(title) = title {
		column = column.child(display(title, 34, BONE).class("mt-[10px]"));
	}
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column.child(View().class("h-2")).children(children))
}

/// Space between entries, in layout units.
pub fn gap(height: u32) -> Element {
	View().class(format!("h-[{height}px]"))
}
