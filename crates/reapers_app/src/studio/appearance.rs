//! Appearance: the calling, form, clothing and hands, and the birth traits
//! the shuffle draws. The Flutter studio's first tab and its
//! `AppearanceControls`.

use leptoskit::prelude::*;
use reapers_rules::Aura;
use reapers_rules::GenomeChanges;
use reapers_rules::Omen;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;
use reapers_rules::WeaponKind;

use super::act;
use super::controls::chip;
use super::controls::chips;
use super::controls::help;
use super::controls::slider_row;
use super::controls::switch_row;
use super::controls::text_button;
use super::specimen::Section;
use super::specimen::Specimen;
use crate::l10n::use_l10n;
use crate::lookups::aura_name;
use crate::lookups::class_name;
use crate::lookups::gift_name;
use crate::lookups::omen_name;
use crate::lookups::temperament_name;
use crate::lookups::weapon_name;
use crate::render::appearance::ReaperAppearance;
use crate::render::appearance::ReaperClothing;
use crate::render::appearance::ReaperForm;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::gap;
use crate::ui::ledger_heading;

/// Which birth trait a row of numbered variants or a slider changes.
#[derive(Clone, Copy)]
enum Trait {
	Head,
	Mantle,
	Robe,
	Palette,
	Stature,
	Gauntness,
	Gaze,
	Tatter,
}

impl Trait {
	const fn id(self) -> &'static str {
		match self {
			Self::Head => "head",
			Self::Mantle => "mantle",
			Self::Robe => "robe",
			Self::Palette => "palette",
			Self::Stature => "stature",
			Self::Gauntness => "gauntness",
			Self::Gaze => "gaze",
			Self::Tatter => "tatter",
		}
	}

	const fn of(self, genome: &ReaperGenome) -> u32 {
		match self {
			Self::Head => genome.head,
			Self::Mantle => genome.mantle,
			Self::Robe => genome.robe,
			Self::Palette => genome.palette,
			Self::Stature => genome.stature,
			Self::Gauntness => genome.gauntness,
			Self::Gaze => genome.gaze,
			Self::Tatter => genome.tatter,
		}
	}

	fn change(self, value: u32) -> GenomeChanges {
		let mut changes = GenomeChanges::default();
		let slot = match self {
			Self::Head => &mut changes.head,
			Self::Mantle => &mut changes.mantle,
			Self::Robe => &mut changes.robe,
			Self::Palette => &mut changes.palette,
			Self::Stature => &mut changes.stature,
			Self::Gauntness => &mut changes.gauntness,
			Self::Gaze => &mut changes.gaze,
			Self::Tatter => &mut changes.tatter,
		};
		*slot = Some(value);
		changes
	}
}

/// The appearance section.
pub fn appearance(specimen: &Specimen) -> Element {
	let words = use_l10n();
	let genome = specimen.genome;
	let look = specimen.appearance;
	let soul = specimen.soul();

	let mut column = View()
		.child(gap(14))
		.child(body(words.studio_appearance_help(), 13, ASH))
		.child(ledger_heading(words.studio_class(), None))
		.child(chips(
			ReaperClass::ALL
				.into_iter()
				.map(|calling| {
					chip(
						class_name(&*words, calling),
						genome.reaper_class == calling,
						format!("studio-class-{}", calling.name()),
						Some(Box::new(move || act(move |s| s.choose_class(calling)))),
					)
				})
				.collect(),
		))
		.child(ledger_heading(words.studio_form(), None))
		.child(chips(
			[
				(ReaperForm::Spirit, words.studio_spirit(), "spirit"),
				(ReaperForm::Humanoid, words.studio_humanoid(), "humanoid"),
			]
			.into_iter()
			.map(|(form, name, id)| {
				chip(
					name,
					look.form == form,
					format!("studio-form-{id}"),
					Some(Box::new(move || {
						act(move |s| {
							s.dress(ReaperAppearance {
								form,
								..s.appearance
							});
						});
					})),
				)
			})
			.collect(),
		))
		.child(help(words.studio_form_help()))
		.child(ledger_heading(words.studio_clothing(), None))
		.child(chips(
			[
				(ReaperClothing::Robe, words.studio_long_robe(), "robe"),
				(ReaperClothing::Tunic, words.studio_tunic(), "tunic"),
				(ReaperClothing::Unrobed, words.studio_no_robe(), "unrobed"),
			]
			.into_iter()
			.map(|(clothing, name, id)| {
				chip(
					name,
					look.clothing == clothing,
					format!("studio-clothing-{id}"),
					Some(Box::new(move || {
						act(move |s| {
							s.dress(ReaperAppearance {
								clothing,
								..s.appearance
							});
						});
					})),
				)
			})
			.collect(),
		))
		.child(switch_row(
			words.studio_show_mantle(),
			None,
			look.mantle,
			"studio-mantle",
			|mantle| {
				act(move |s| {
					s.dress(ReaperAppearance {
						mantle,
						..s.appearance
					});
				});
			},
		))
		.child(switch_row(
			words.studio_hold_object(),
			Some(words.studio_hold_help()),
			look.holding,
			"studio-holding",
			|holding| {
				act(move |s| {
					s.dress(ReaperAppearance {
						holding,
						..s.appearance
					});
				});
			},
		));
	if look.holding {
		let held = specimen.weapon().kind;
		column = column
			.child(ledger_heading(words.studio_held_object(), None))
			.child(chips(
				WeaponKind::ALL
					.into_iter()
					.map(|kind| {
						chip(
							weapon_name(&*words, kind),
							held == kind,
							format!("studio-held-{}", kind.name()),
							Some(Box::new(move || act(move |s| s.hold(kind)))),
						)
					})
					.collect(),
			));
	}
	column = column.child(gap(14)).child(
		LedgerButton::new(words.soul_birth_link(
			gift_name(&*words, soul.gift),
			temperament_name(&*words, soul.temperament),
		))
		.detail(words.soul_tab())
		.quiet()
		.test_id("studio-soul-link")
		.on_press(|| act(|s| s.section = Section::Soul)),
	);
	for (name, count, which) in [
		(
			words.studio_head(),
			ReaperGenome::HEAD_VARIANTS,
			Trait::Head,
		),
		(
			words.studio_mantle(),
			ReaperGenome::MANTLE_VARIANTS,
			Trait::Mantle,
		),
		(
			words.studio_robe(),
			ReaperGenome::ROBE_VARIANTS,
			Trait::Robe,
		),
		(
			words.studio_palette(),
			ReaperGenome::PALETTE_VARIANTS,
			Trait::Palette,
		),
	] {
		column = column.child(variants(name, count, which, which.of(&genome)));
	}
	column = column
		.child(ledger_heading(words.studio_omen(), None))
		.child(chips(
			Omen::ALL
				.into_iter()
				.map(|omen| {
					chip(
						omen_name(&*words, omen),
						genome.omen == omen,
						format!("studio-omen-{}", omen.name()),
						Some(Box::new(move || {
							act(move |s| {
								s.change(GenomeChanges {
									omen: Some(omen),
									..GenomeChanges::default()
								});
							});
						})),
					)
				})
				.collect(),
		))
		.child(ledger_heading(words.studio_aura(), None))
		.child(chips(
			Aura::ALL
				.into_iter()
				.map(|aura| {
					chip(
						aura_name(&*words, aura),
						genome.aura == aura,
						format!("studio-aura-{}", aura.name()),
						Some(Box::new(move || {
							act(move |s| {
								s.change(GenomeChanges {
									aura: Some(aura),
									..GenomeChanges::default()
								});
							});
						})),
					)
				})
				.collect(),
		));
	for (name, which) in [
		(words.studio_stature(), Trait::Stature),
		(words.studio_gauntness(), Trait::Gauntness),
		(words.studio_gaze(), Trait::Gaze),
		(words.studio_tatter(), Trait::Tatter),
	] {
		let value = which.of(&genome);
		let scale = ReaperGenome::TRAIT_SCALE;
		column = column.child(slider_row(
			name,
			None,
			f64::from(value),
			(0.0, f64::from(scale), f64::from(scale) / 20.0),
			&format!("studio-trait-{}", which.id()),
			format!("{}%", (f64::from(value) / 10.0).round()),
			move |next| act(move |s| s.change(which.change(next.round() as u32))),
		));
	}
	column = column
		.child(gap(18))
		.child(
			LedgerButton::new(words.studio_shuffle())
				.test_id("studio-shuffle")
				.on_press(|| act(Specimen::shuffle)),
		)
		.child(gap(12))
		.child(body(words.studio_birth_traits(), 13, ASH))
		.child(text_button(words.studio_copy(), "studio-copy", move || {
			let copied = copy_text(&genome.code());
			act(move |s| {
				s.code_shown = true;
				s.code_copied = copied;
			});
		}));
	if specimen.code_shown {
		if specimen.code_copied {
			column = column.child(body(words.studio_copied(), 13, BONE));
		}
		column = column.child(
			TextInput()
				.value(genome.code())
				.aria_read_only(true)
				.accessibility_label(words.studio_genome().to_owned())
				.test_id("studio-code")
				.class(format!(
					"w-full min-h-[44px] px-3 text-[13px] text-[{}] border border-[#e9e4d42e]",
					BONE.hex()
				)),
		);
	}
	column.child(gap(24))
}

/// A heading and one numbered chip per variant.
fn variants(name: &str, count: u32, which: Trait, value: u32) -> Element {
	let words = use_l10n();
	View().child(ledger_heading(name, None)).child(chips(
		(0..count)
			.map(|index| {
				let number = words.studio_variant(i64::from(index) + 1);
				chip(
					&number,
					index == value,
					format!("studio-{}-{index}", which.id()),
					Some(Box::new(move || {
						act(move |s| s.change(which.change(index)));
					})),
				)
				.accessibility_label(format!("{name} {number}"))
			})
			.collect(),
	))
}

/// Puts `text` on the clipboard where the platform offers one: the
/// browser's. The apps' guest has none, so the code is shown to copy by
/// hand.
#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn copy_text(text: &str) -> bool {
	use wasm_bindgen::JsCast;
	let Some(window) = web_sys::window() else {
		return false;
	};
	let navigator = window.navigator();
	let Ok(clipboard) = js_sys::Reflect::get(&navigator, &"clipboard".into()) else {
		return false;
	};
	let Ok(write) = js_sys::Reflect::get(&clipboard, &"writeText".into()) else {
		return false;
	};
	write
		.dyn_into::<js_sys::Function>()
		.is_ok_and(|write| write.call1(&clipboard, &text.into()).is_ok())
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
const fn copy_text(_text: &str) -> bool {
	false
}
