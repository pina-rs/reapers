//! A reaper's birth contract and its earned growth, in the codex and the
//! studio.

use leptoskit::prelude::*;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperSoul;
use reapers_rules::SoulMemory;
use reapers_rules::SoulPath;
use reapers_rules::SoulRules;

use crate::l10n::use_l10n;
use crate::lookups::gift_effect;
use crate::lookups::gift_lore;
use crate::lookups::gift_name;
use crate::lookups::soul_path_help;
use crate::lookups::soul_path_name;
use crate::lookups::temperament_help;
use crate::lookups::temperament_name;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::ledger_heading;
use crate::ui::ledger_pip;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::ui::prose;

/// The soul of the reaper born of `genome`, at `level`, remembering
/// `memory`.
pub fn soul_ledger(genome: &ReaperGenome, level: i64, memory: &SoulMemory) -> Element {
	let words = use_l10n();
	let soul = ReaperSoul::from_genome(genome);
	let gift = soul.gift;
	let rank = SoulRules::rank(level, memory, gift);
	let max = SoulRules::MAX_RANK;
	let light = light();
	let next = rank as usize + 1;

	let mut column = View()
		.child(gap(22))
		.child(display(gift_name(&*words, gift), 28, light))
		.child(gap(10))
		.child(prose(gift_effect(&*words, gift, rank + 1)))
		.child(gap(8))
		.child(body(words.soul_rank(rank, max), 13, light))
		.child(gap(12))
		.child(aside(if rank == max {
			words.soul_complete().to_owned()
		} else {
			words.soul_next(
				SoulRules::RANK_LEVELS[next],
				SoulRules::RANK_MARKS[next],
				soul_path_name(&*words, gift.path()),
				memory.marks(gift.path()),
			)
		}))
		.child(gap(22))
		.child(display(
			words.soul_temperament(temperament_name(&*words, soul.temperament)),
			22,
			BONE,
		))
		.child(gap(8))
		.child(body(temperament_help(&*words, soul.temperament), 15, ASH))
		.child(ledger_heading(words.soul_marks(), None));
	for path in SoulPath::ALL {
		column = column
			.child(ledger_row(
				soul_path_name(&*words, path),
				memory.marks(path).to_string(),
				path == gift.path(),
			))
			.child(aside(soul_path_help(&*words, path)))
			.child(gap(10));
	}
	column = column
		.child(aside(words.soul_memory_help()))
		.child(ledger_heading(words.soul_lore(), None))
		.child(body(gift_lore(&*words, gift), 15, ASH));
	for milestone in 1..=max {
		let index = milestone as usize;
		column = column.child(
			View()
				.class("flex-row items-start gap-[14px] pt-4")
				.child(
					View()
						.class("pt-[6px]")
						.child(ledger_pip(milestone <= rank, light)),
				)
				.child(
					View()
						.class("flex-1 gap-1")
						.child(prose(words.soul_milestone(
							SoulRules::RANK_LEVELS[index],
							SoulRules::RANK_MARKS[index],
							soul_path_name(&*words, gift.path()),
						)))
						.child(aside(gift_effect(&*words, gift, milestone + 1))),
				),
		);
	}
	column
}
