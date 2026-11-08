//! The studio's small controls, ruled like the rest of the ledger: chips
//! for a choice among a few, a switch on a line, and a slider under its
//! heading. They stand in for the Flutter studio's `ChoiceChip`,
//! `SwitchListTile`, `Slider` and `SegmentedButton`.

use leptoskit::prelude::*;

use crate::platform;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::class;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::label;
use crate::ui::ledger_heading;

thread_local! {
	/// The specimen's light, which the studio is lit with as it renders.
	static LIGHT: std::cell::Cell<Rgb> = const { std::cell::Cell::new(crate::theme::UNBOUND) };
}

/// Lights the studio's controls with `light`: the specimen's, as the
/// Flutter studio themed itself with it.
pub fn set_light(light: Rgb) {
	LIGHT.with(|cell| cell.set(light));
}

/// The light the studio is lit with.
pub fn light() -> Rgb {
	LIGHT.with(std::cell::Cell::get)
}

/// What a control does when pressed.
pub type Action = Box<dyn Fn() + Send + Sync>;

/// A row of chips that wraps.
pub fn chips(children: Vec<Element>) -> Element {
	View()
		.class("flex-row flex-wrap gap-[6px]")
		.children(children)
}

/// One choice among a few: outlined, and lit when chosen. `on_press` is
/// `None` for a choice that cannot be made now.
pub fn chip(
	text: &str,
	selected: bool,
	id: impl Into<String>,
	on_press: Option<Action>,
) -> Element {
	let enabled = on_press.is_some();
	let light = light();
	let (border, fill, colour) = if selected {
		(light.hex(), format!("{}2e", light.hex()), light)
	} else if enabled {
		("#e9e4d459".to_owned(), "#00000000".to_owned(), BONE)
	} else {
		("#e9e4d41f".to_owned(), "#00000000".to_owned(), ASH)
	};
	let chip = Pressable(text.to_owned())
		.class(format!(
			"min-h-[40px] px-3 justify-center border border-[{border}] bg-[{fill}] rounded-[2px]"
		))
		.test_id(id.into())
		.aria_selected(selected)
		.child(body(text, 13, colour));
	match on_press {
		Some(on_press) => {
			chip.on_press(move || {
				platform::tap();
				on_press();
			})
		}
		None => chip.enabled(false),
	}
}

/// A line with a switch at its end, and help beneath.
pub fn switch_row(
	text: &str,
	help: Option<&str>,
	checked: bool,
	id: &str,
	on_toggle: impl Fn(bool) + Send + Sync + 'static,
) -> Element {
	let mut words = View().class("flex-1 gap-1").child(body(text, 15, BONE));
	if let Some(help) = help {
		words = words.child(body(help, 12, ASH));
	}
	View()
		.class("flex-row items-center gap-4 py-[10px]")
		.child(words)
		.child(
			Switch(checked)
				.accessibility_label(text.to_owned())
				.test_id(id.to_owned())
				.on_toggle(on_toggle),
		)
}

/// A heading with a slider under it, over `min..=max` in steps of `step`.
/// `value` is shown at the nearest step; a move proposes the step it lands
/// on.
pub fn slider_row(
	text: &str,
	trailing: Option<String>,
	value: f64,
	(min, max, step): (f64, f64, f64),
	id: &str,
	value_text: String,
	on_change: impl Fn(f64) + Send + Sync + 'static,
) -> Element {
	let snapped = (((value - min) / step).round() * step + min).clamp(min, max);
	let slider = SliderState::new(min, max, snapped, Some(step))
		.or_else(|_| SliderState::new(min, max, min, Some(step)))
		.unwrap_or_default();
	View().child(ledger_heading(text, trailing)).child(
		Slider(slider)
			.class("w-full h-[32px]")
			.accessibility_label(text.to_owned())
			.aria_value_text(value_text)
			.test_id(id.to_owned())
			.on_value_change(on_change),
	)
}

/// The tabs across a section: names on a rule, the chosen one lit.
pub fn tabs(entries: Vec<(&str, &str, bool, Action)>) -> Element {
	let light = light();
	let mut row = View().class(format!("flex-row border-b {}", class::HAIRLINE_BORDER));
	for (name, id, chosen, on_press) in entries {
		let underline = if chosen {
			light.hex()
		} else {
			"#00000000".to_owned()
		};
		row = row.child(
			Pressable(name.to_owned())
				.class(format!(
					"flex-1 min-h-[48px] items-center justify-center border-b-2 border-[{underline}]"
				))
				.test_id(id.to_owned())
				.aria_selected(chosen)
				.on_press(move || {
					platform::tap();
					on_press();
				})
				.child(label(name, 12, if chosen { BONE } else { ASH })),
		);
	}
	row
}

/// A small action written as a word, for the stage's corners and the
/// lesser actions of a section.
pub fn text_button(text: &str, id: &str, on_press: impl Fn() + Send + Sync + 'static) -> Element {
	Pressable(text.to_owned())
		.class("min-h-[44px] justify-center px-2")
		.test_id(id.to_owned())
		.on_press(move || {
			platform::tap();
			on_press();
		})
		.child(label(text, 12, BONE))
}

/// Running help under a control.
pub fn help(text: impl Into<String>) -> Element {
	View().class("pt-2").child(aside(text))
}
