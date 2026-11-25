//! The kit page: every control and mark of Second Ink (the art bible, §8)
//! in every state it can be in, on the vigil page, for whoever builds a
//! screen and whoever checks one against the bible. Debug builds only: a
//! release sends `/kit` home.
//!
//! The first control of each kind is live: it answers a press and the
//! keyboard. The rest are prints of one state, held.

use leptoskit::prelude::*;
use reapers_rules::Nft;
use reapers_rules::NftCollection;

use crate::l10n::use_l10n;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::PAPER;
use crate::theme::Step;
use crate::theme::class;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::ink;
use crate::ui::kit::Field;
use crate::ui::kit::FieldState;
use crate::ui::kit::PaperSheet;
use crate::ui::kit::Primary;
use crate::ui::kit::Quiet;
use crate::ui::kit::Row;
use crate::ui::kit::Secondary;
use crate::ui::kit::Slip;
use crate::ui::kit::SlipState;
use crate::ui::kit::State;
use crate::ui::kit::Tab;
use crate::ui::kit::TabIcon;
use crate::ui::kit::Toggle;
use crate::ui::kit::ToggleKind;
use crate::ui::kit::class_mark;
use crate::ui::kit::divider;
use crate::ui::kit::lamp;
use crate::ui::kit::meter;
use crate::ui::kit::pips;
use crate::ui::kit::soul_dial;
use crate::ui::kit::stamp;
use crate::ui::kit::tally;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::light as class_light;
use crate::ui::margin;

/// The page, or home in a release build.
pub fn page() -> Element {
	let game = Game::get();
	if game.flags.release {
		return crate::screens::home::page();
	}
	let words = use_l10n();
	let before = (ink::drawn(), ink::asked());
	let sections = vec![
		section("type", words.kit_section_type(), type_scale()),
		section("primary", words.kit_section_primary(), primaries()),
		section("secondary", words.kit_section_secondary(), secondaries()),
		section("quiet", words.kit_section_quiet(), quiets()),
		section("rows", words.kit_section_rows(), rows()),
		section("tabs", words.kit_section_tabs(), tabs()),
		section("slips", words.kit_section_slips(), slips()),
		section("stamps", words.kit_section_stamps(), stamps()),
		section("dial", words.kit_section_dial(), dials()),
		section("lamp", words.kit_section_lamp(), lamps()),
		section("meter", words.kit_section_meter(), meters()),
		section("tally", words.kit_section_tally(), tallies()),
		section("sheet", words.kit_section_sheet(), sheets()),
		section("fields", words.kit_section_fields(), fields()),
		section("toggles", words.kit_section_toggles(), toggles()),
	];
	let drawn = ink::drawn() - before.0;
	let asked = ink::asked() - before.1;
	let ink_line = words.kit_ink(drawn as i64, asked as i64, ink::drawn() as i64);
	crate::platform::log(&format!("kit: {ink_line}"));

	let column = View()
		.class("w-full max-w-[520px] self-center px-5 pt-safe-offset-2 pb-16")
		.child(ledger_back(None))
		.child(margin(words.kit_overline()).class("mt-4"))
		.child(display(words.kit_title(), Step::S7.size(), PAPER).class("mt-2"))
		.child(aside(words.kit_intro()).class("mt-3"))
		.children(sections)
		.child(
			section_frame("ink", words.kit_section_ink())
				.child(body(ink_line, Step::S3.size(), PAPER).test_id("kit-ink")),
		);
	View()
		.class(format!("flex-1 w-full h-full {}", class::VIGIL_BG))
		.test_id("kit")
		.child(ScrollView().class("flex-1 w-full").child(column))
}

/// The spot the specimens are printed in: the bound reaper's light, or the
/// Ferryman's lantern before an oath, as the boards print it. Paper, the
/// unbound light, would hide every flood and spot stroke the kit is here to
/// show.
fn light() -> crate::theme::Rgb {
	let bound = class_light();
	if bound == crate::theme::UNBOUND {
		crate::theme::LANTERN
	} else {
		bound
	}
}

/// A section's frame: its name over a rule.
fn section_frame(slug: &str, title: &str) -> Element {
	View()
		.class("w-full gap-4 pt-10 pb-4")
		.test_id(format!("kit-section-{slug}"))
		.child(label(title, Step::S2.size(), PAPER))
		.child(divider(crate::theme::seed_for(slug)))
}

/// A section of specimens.
fn section(slug: &str, title: &str, specimens: Vec<Element>) -> Element {
	section_frame(slug, title).children(specimens)
}

/// One specimen under the name of its state.
fn specimen(state: &str, element: Element) -> Element {
	View()
		.class("w-full gap-3 pt-2")
		.child(margin(state))
		.child(element)
}

/// The name of a control's state, in the kit's words.
fn state_name(state: State) -> &'static str {
	let words = use_l10n();
	match state {
		State::Rest => words.kit_state_rest(),
		State::Pressed => words.kit_state_pressed(),
		State::Focused => words.kit_state_focused(),
		State::Busy => words.kit_state_busy(),
		State::Disabled => words.kit_state_disabled(),
	}
}

/// A step's caption, from its tokens: "s7 40/38 · Display". The faces'
/// names are the bible's, and this bench's words stay out of the
/// translations, as the other benches' do.
fn step_caption(index: usize, step: Step, face: &str) -> String {
	format!("s{} {}/{} · {face}", index + 1, step.size(), step.leading())
}

fn type_scale() -> Vec<Element> {
	let words = use_l10n();
	let name = words.kit_sample_field_value();
	let caption = |step: Step, face: &str| {
		let index = Step::ALL
			.iter()
			.position(|each| *each == step)
			.unwrap_or_default();
		step_caption(index, step, face)
	};
	vec![
		specimen(
			&caption(Step::S7, "Display"),
			display(name, Step::S7.size(), PAPER),
		),
		specimen(
			&caption(Step::S6, "Display"),
			display(name, Step::S6.size(), PAPER),
		),
		specimen(
			&caption(Step::S5, "Display"),
			display(name, Step::S5.size(), light()),
		),
		specimen(
			&caption(Step::S4, "Text"),
			body(words.kit_sample_slip(), Step::S4.size(), PAPER),
		),
		specimen(
			&caption(Step::S3, "Text"),
			body(words.kit_intro(), Step::S3.size(), PAPER),
		),
		specimen(&caption(Step::S3, "Text"), aside(words.kit_intro())),
		specimen(
			&caption(Step::S2, "Figures"),
			figure("214/400 · 1:47 · 3.00", Step::S2.size(), PAPER),
		),
		specimen(
			&caption(Step::S1, "Figures"),
			label(words.sanctum_loose(), Step::S1.size(), ASH),
		),
	]
}

fn primaries() -> Vec<Element> {
	let words = use_l10n();
	State::ALL
		.into_iter()
		.map(|state| {
			let primary =
				Primary::new(words.kit_sample_primary(), light()).detail(words.kit_sample_nearby());
			let primary = match state {
				State::Rest => primary.test_id("kit-live-primary").on_press(|| {}),
				State::Disabled => primary.disabled(words.kit_sample_reason()).print(),
				_ => primary.state(state).print(),
			};
			specimen(state_name(state), primary)
		})
		.collect()
}

fn secondaries() -> Vec<Element> {
	let words = use_l10n();
	let mut specimens: Vec<Element> = State::ALL
		.into_iter()
		.map(|state| {
			let secondary = Secondary::new(words.kit_sample_secondary());
			let secondary = if state == State::Rest {
				secondary.test_id("kit-live-secondary").on_press(|| {})
			} else {
				secondary.state(state).print()
			};
			specimen(state_name(state), secondary)
		})
		.collect();
	specimens.push(specimen(
		words.kit_state_destructive(),
		Secondary::new(words.vault_release()).destructive().print(),
	));
	specimens
}

fn quiets() -> Vec<Element> {
	let words = use_l10n();
	[State::Rest, State::Pressed, State::Focused, State::Disabled]
		.into_iter()
		.map(|state| {
			let quiet = Quiet::new(words.sanctum_colophon());
			let quiet = if state == State::Rest {
				quiet.test_id("kit-live-quiet").on_press(|| {})
			} else {
				quiet.state(state).print()
			};
			specimen(state_name(state), quiet)
		})
		.collect()
}

fn rows() -> Vec<Element> {
	let words = use_l10n();
	let title = Nft::new(NftCollection::PeskyPenguins, 1).title();
	[State::Rest, State::Pressed, State::Focused, State::Disabled]
		.into_iter()
		.map(|state| {
			let row = Row::new(title.clone())
				.status(words.kit_sample_status())
				.figure("1:47");
			let row = if state == State::Rest {
				row.test_id("kit-live-row").on_press(|| {})
			} else {
				row.state(state).print()
			};
			specimen(state_name(state), row)
		})
		.collect()
}

/// The five tabs, with the sanctum active, the vault in `vault` and the
/// codex in `codex`.
fn tab_set(vault: State, codex: State, live: bool) -> Vec<Tab> {
	let words = use_l10n();
	[
		(
			TabIcon::Lamp,
			words.sanctum_overline(),
			"sanctum",
			State::Rest,
		),
		(TabIcon::Box, words.sanctum_vault(), "vault", vault),
		(
			TabIcon::Blade,
			words.sanctum_armory(),
			"armory",
			State::Rest,
		),
		(TabIcon::Book, words.sanctum_codex(), "codex", codex),
		(TabIcon::Crack, words.sanctum_rifts(), "rifts", State::Rest),
	]
	.into_iter()
	.enumerate()
	.map(|(index, (icon, text, slug, state))| {
		Tab {
			icon,
			label: text.to_owned(),
			test_id: if live && index == 1 {
				"kit-live-tab".to_owned()
			} else {
				format!("kit-tab-{slug}-{}", if live { "live" } else { "print" })
			},
			active: index == 0,
			state: if live { State::Rest } else { state },
			open: Box::new(|| {}),
		}
	})
	.collect()
}

fn tabs() -> Vec<Element> {
	let words = use_l10n();
	vec![
		specimen(
			&format!("{} · {}", words.kit_state_active(), words.kit_state_rest()),
			crate::ui::kit::tab_bar(tab_set(State::Rest, State::Rest, true), light()),
		),
		specimen(
			&format!(
				"{} · {}",
				words.kit_state_pressed(),
				words.kit_state_focused()
			),
			crate::ui::kit::tab_bar(tab_set(State::Pressed, State::Focused, false), light()),
		),
	]
}

fn slips() -> Vec<Element> {
	let words = use_l10n();
	vec![
		specimen(
			words.kit_state_rest(),
			Slip::new(words.kit_sample_slip()).view(),
		),
		specimen(
			words.kit_state_entering(),
			Slip::new(words.kit_sample_slip())
				.state(SlipState::Entering)
				.view(),
		),
		specimen(
			words.kit_state_leaving(),
			Slip::new(words.kit_sample_slip())
				.state(SlipState::Leaving)
				.view(),
		),
		specimen(
			words.kit_state_pressed(),
			Slip::new(words.kit_sample_slip())
				.state(SlipState::Pressed)
				.view(),
		),
		specimen(
			words.kit_state_error(),
			Slip::new(words.kit_sample_error()).torn().view(),
		),
	]
}

fn stamps() -> Vec<Element> {
	let spot = light();
	vec![
		specimen(
			"IV · V · XII",
			View()
				.class("flex-row gap-6 items-center")
				.child(stamp("IV", spot, 3))
				.child(stamp("V", spot, 8))
				.child(stamp("XII", spot, 13)),
		),
		specimen(
			"1 · 2 · 3 · 4",
			View()
				.class("flex-row gap-6 items-center")
				.child(class_mark(spot, 5))
				.child(pips(1, 21))
				.child(pips(2, 22))
				.child(pips(3, 23))
				.child(pips(4, 24)),
		),
	]
}

fn dials() -> Vec<Element> {
	let words = use_l10n();
	let spot = light();
	vec![specimen(
		&format!("{} · {}", words.kit_state_under(), words.kit_state_ready()),
		View()
			.class("flex-row gap-8 items-start")
			.child(soul_dial(22.0, spot, 31))
			.child(soul_dial(40.0, spot, 32))
			.child(soul_dial(86.0, spot, 33)),
	)]
}

fn lamps() -> Vec<Element> {
	let words = use_l10n();
	let spot = light();
	vec![specimen(
		&format!("{} · {}", words.kit_state_waxing(), words.kit_state_full()),
		View()
			.class("flex-row gap-6 items-center")
			.child(lamp(60.0, 0.62, spot, 41))
			.child(lamp(60.0, 1.0, spot, 42)),
	)]
}

fn meters() -> Vec<Element> {
	let words = use_l10n();
	vec![
		specimen(
			words.kit_state_draining(),
			meter(0.6, "60%", Some(words.kit_sample_resolve()), 200.0, 51),
		),
		specimen(
			words.kit_state_critical(),
			meter(0.1, "1/6", Some(words.kit_sample_resolve()), 200.0, 52),
		),
		specimen(words.kit_state_ward(), meter(0.1, "10%", None, 200.0, 53)),
	]
}

fn tallies() -> Vec<Element> {
	let words = use_l10n();
	let spot = light();
	vec![specimen(
		words.kit_state_newest(),
		View()
			.class("gap-4")
			.child(
				View()
					.class("flex-row items-center gap-4")
					.child(tally(38, 8, spot, 61))
					.child(figure("38", Step::S4.size(), PAPER)),
			)
			.child(
				View()
					.class("flex-row items-center gap-4")
					.child(tally(40, 8, spot, 62))
					.child(figure("40", Step::S4.size(), PAPER)),
			)
			.child(
				View()
					.class("flex-row items-center gap-4")
					.child(tally(3, 8, spot, 63))
					.child(figure("3", Step::S4.size(), PAPER)),
			),
	)]
}

fn sheets() -> Vec<Element> {
	let words = use_l10n();
	vec![specimen(
		words.kit_state_open(),
		// Full bleed, as a sheet rises: the column's margins are 20.
		View().class("mx-[-20px]").child(
			PaperSheet::new(
				71,
				vec![
					body(words.kit_sample_sheet(), Step::S4.size(), crate::theme::KEY),
					label(words.vault_release(), Step::S2.size(), crate::theme::KEY),
				],
			)
			.view(false),
		),
	)]
}

fn fields() -> Vec<Element> {
	let words = use_l10n();
	let name = words.kit_sample_field();
	let value = words.kit_sample_field_value();
	vec![
		specimen(
			words.kit_state_empty(),
			Field::new(name, "field-kit-live")
				.placeholder(value)
				.on_change(|_| {}),
		),
		specimen(
			words.kit_state_focused(),
			Field::new(name, "field-kit-focused")
				.value(value)
				.state(FieldState::Focused)
				.print(),
		),
		specimen(
			words.kit_state_filled(),
			Field::new(name, "field-kit-filled")
				.value(value)
				.state(FieldState::Filled)
				.print(),
		),
		specimen(
			words.kit_state_error(),
			Field::new(name, "field-kit-error")
				.value("  ")
				.state(FieldState::Error(words.kit_sample_field_error().to_owned()))
				.print(),
		),
		specimen(
			words.kit_state_disabled(),
			Field::new(name, "field-kit-disabled")
				.value(value)
				.state(FieldState::Disabled)
				.print(),
		),
	]
}

fn toggles() -> Vec<Element> {
	let words = use_l10n();
	let spot = light();
	let kinds = [
		(ToggleKind::Switch, words.kit_sample_switch()),
		(ToggleKind::Checkbox, words.kit_sample_checkbox()),
		(ToggleKind::Radio, words.kit_sample_radio()),
	];
	let mut specimens = Vec::new();
	for (kind, name) in kinds {
		specimens.push(specimen(
			&format!("{} · {}", words.kit_state_on(), words.kit_state_off()),
			View()
				.class("w-full gap-1")
				.child(Toggle::new(kind, name, true, spot).on_press(|| {}))
				.child(Toggle::new(kind, name, false, spot).on_press(|| {})),
		));
		specimens.push(specimen(
			&format!(
				"{} · {} · {}",
				words.kit_state_pressed(),
				words.kit_state_focused(),
				words.kit_state_disabled()
			),
			View()
				.class("w-full gap-1")
				.child(
					Toggle::new(kind, name, false, spot)
						.state(State::Pressed)
						.print(),
				)
				.child(
					Toggle::new(kind, name, true, spot)
						.state(State::Focused)
						.print(),
				)
				.child(
					Toggle::new(kind, name, true, spot)
						.state(State::Disabled)
						.print(),
				),
		));
	}
	specimens
}
