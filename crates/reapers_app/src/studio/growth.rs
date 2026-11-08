//! Growth: levels one to thirty by the shared progression, an illustrative
//! absorption, and the weapon and outfit gates. The Flutter studio's second
//! tab.

use leptoskit::prelude::*;
use reapers_rules::Outfit;
use reapers_rules::Progression;
use reapers_rules::SoulRules;
use reapers_rules::Temper;

use super::act;
use super::controls::light;
use super::controls::slider_row;
use super::specimen::Section;
use super::specimen::Specimen;
use crate::l10n::use_l10n;
use crate::lookups::gift_name;
use crate::lookups::outfit_name;
use crate::lookups::temper_name;
use crate::platform;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_meter;
use crate::ui::ledger_pip;
use crate::ui::ledger_row;

/// The level slider, shared by growth and soul.
pub fn level_slider(specimen: &Specimen, id: &str) -> Element {
	let words = use_l10n();
	let level = specimen.level();
	slider_row(
		words.studio_level(),
		Some(format!("{level} / {}", Progression::LEVEL_CAP)),
		level as f64,
		(1.0, Progression::LEVEL_CAP as f64, 1.0),
		id,
		words.studio_level_value(level),
		|next| act(move |s| s.set_level(next.round() as i64)),
	)
}

/// The growth section.
pub fn growth(specimen: &Specimen) -> Element {
	let words = use_l10n();
	let light = light();
	let level = specimen.level();
	let (damage, _) = specimen.strikes();
	let soul = specimen.soul();

	let mut column = View()
		.child(level_slider(specimen, "studio-level"))
		.child(ledger_meter(
			if level == Progression::LEVEL_CAP {
				words.studio_capped()
			} else {
				words.studio_xp()
			},
			Progression::level_progress(specimen.xp),
			light,
		))
		.child(gap(12))
		.child(ledger_row(
			words.studio_strength(),
			words.studio_stronger(specimen.strength_percent()),
			true,
		))
		.child(ledger_row(
			words.studio_damage(),
			format!("{damage:.1}"),
			false,
		))
		.child(gap(12))
		.child(body(words.studio_growth_line(), 13, ASH))
		.child(gap(14))
		.child(
			LedgerButton::new(gift_name(&*words, soul.gift))
				.detail(words.soul_rank(specimen.soul_rank(), SoulRules::MAX_RANK))
				.quiet()
				.test_id("studio-growth-soul")
				.on_press(|| act(|s| s.section = Section::Soul)),
		)
		.child(gap(18))
		.child(
			LedgerButton::new(words.studio_absorb())
				.detail(words.studio_captures(specimen.captures))
				.test_id("studio-absorb")
				.on_press(|| {
					act(|s| {
						s.absorb(None);
					});
				}),
		);
	if let Some(earned) = specimen.reward {
		column = column.child(
			View()
				.class("pt-2")
				.aria_live(leptoskit::native::Live::Polite)
				.child(body(words.studio_reward(earned), 13, light).test_id("studio-reward")),
		);
	}
	column = column
		.child(gap(8))
		.child(body(words.studio_example(), 13, ASH))
		.child(ledger_heading(words.studio_weapon(), None));
	for temper in Temper::ALL {
		column = column.child(upgrade(
			temper_name(&*words, temper),
			temper.level_required(),
			temper.price(),
			level,
			specimen.temper == temper,
			&format!("studio-temper-{}", temper.name()),
			move || act(move |s| s.choose_temper(temper)),
		));
	}
	column = column.child(ledger_heading(words.studio_outfit(), None));
	for outfit in Outfit::ALL {
		column = column.child(upgrade(
			outfit_name(&*words, outfit),
			outfit.level_required(),
			outfit.price(),
			level,
			specimen.outfit == outfit,
			&format!("studio-outfit-{}", outfit.name()),
			move || act(move |s| s.choose_outfit(outfit)),
		));
	}
	column
		.child(gap(12))
		.child(body(words.studio_cosmetic(), 13, ASH))
		.child(gap(18))
		.child(
			LedgerButton::new(words.studio_potential())
				.test_id("studio-potential")
				.on_press(|| act(Specimen::see_potential)),
		)
		.child(gap(12))
		.child(body(words.studio_multiple(), 13, ASH))
		.child(gap(24))
}

/// One weapon temper or outfit: lit when worn, open once the level reaches
/// it, and marked with its gate and price.
fn upgrade(
	name: &str,
	required: i64,
	price: i64,
	level: i64,
	selected: bool,
	id: &str,
	on_press: impl Fn() + Send + Sync + 'static,
) -> Element {
	let words = use_l10n();
	let light = light();
	let open = level >= required;
	let mark = if open {
		ledger_pip(selected, light)
	} else {
		label("×", 12, ASH).aria_hidden(true)
	};
	let row = Pressable(name.to_owned())
		.class("min-h-[56px] flex-row items-center gap-[14px] py-2")
		.test_id(id.to_owned())
		.aria_selected(selected)
		.child(View().class("w-4 items-center").child(mark))
		.child(
			View()
				.class("flex-1 gap-[2px]")
				.child(body(
					name,
					15,
					if selected {
						light
					} else if open {
						BONE
					} else {
						ASH
					},
				))
				.child(body(words.studio_unlock(required, price), 12, ASH)),
		);
	if open {
		row.on_press(move || {
			platform::tap();
			on_press();
		})
	} else {
		row.enabled(false)
	}
}
