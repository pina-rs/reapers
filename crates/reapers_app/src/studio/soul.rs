//! Soul: the birth gift and temperament, every gift of the calling to try,
//! and illustrative memories along the three paths. The Flutter studio's
//! third tab; the ledger itself is the codex's ([`soul_ledger`]).

use leptoskit::prelude::*;
use reapers_rules::ReaperGift;
use reapers_rules::SoulMemory;
use reapers_rules::SoulPath;

use super::act;
use super::controls::chip;
use super::controls::chips;
use super::controls::text_button;
use super::growth::level_slider;
use super::specimen::Specimen;
use crate::l10n::use_l10n;
use crate::lookups::gift_name;
use crate::lookups::soul_path_name;
use crate::screens::soul::soul_ledger;
use crate::theme::ASH;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::gap;
use crate::ui::ledger_heading;

/// The soul section.
pub fn soul(specimen: &Specimen) -> Element {
	let words = use_l10n();
	let calling = specimen.genome.reaper_class;
	let born = specimen.soul().gift;

	let mut column = View()
		.child(gap(20))
		.child(body(words.soul_intro(), 15, ASH))
		.child(level_slider(specimen, "studio-soul-level"))
		.child(ledger_heading(words.soul_gift_choice(), None))
		.child(chips(
			ReaperGift::ALL
				.into_iter()
				.filter(|gift| gift.reaper_class() == calling)
				.map(|gift| {
					chip(
						gift_name(&*words, gift),
						born == gift,
						format!("studio-gift-{}", gift.name()),
						Some(Box::new(move || act(move |s| s.try_gift(gift)))),
					)
				})
				.collect(),
		))
		.child(soul_ledger(
			&specimen.genome,
			specimen.level(),
			&specimen.memory,
		))
		.child(gap(20))
		.child(body(words.soul_preview_help(), 13, ASH))
		.child(gap(12));
	for path in SoulPath::ALL {
		column = column
			.child(
				LedgerButton::new(words.soul_preview_mark(soul_path_name(&*words, path)))
					.detail(specimen.memory.marks(path).to_string())
					.test_id(format!("studio-memory-{}", path.name()))
					.on_press(move || {
						act(move |s| {
							s.absorb(Some(path));
						});
					}),
			)
			.child(gap(8));
	}
	column
		.child(text_button(
			words.soul_preview_reset(),
			"studio-memory-reset",
			|| act(|s| s.memory = SoulMemory::default()),
		))
		.child(gap(24))
}
