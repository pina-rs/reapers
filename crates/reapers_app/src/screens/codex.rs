//! The record of everything this reaper has absorbed, collection by
//! collection, and how far each set is from complete.

use leptoskit::prelude::*;
use reapers_rules::Feeding;
use reapers_rules::NftCollection;

use super::soul::soul_ledger;
use crate::l10n::use_l10n;
use crate::lookups::feeding_name;
use crate::lookups::gift_name;
use crate::lookups::grade_name;
use crate::lookups::temperament_name;
use crate::seams::HuskPortraitProps;
use crate::seams::husk_portrait;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_page;
use crate::ui::ledger_row;
use crate::ui::light;

/// Whether the soul's ledger is open: the Flutter `ExpansionTile`.
fn soul_open() -> RwSignal<bool> {
	thread_local! {
		static OPEN: RwSignal<bool> = crate::ui::lasting(|| RwSignal::new(false));
	}
	OPEN.with(|open| *open)
}

/// The codex.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let light = light();
	let Some(player) = game.player() else {
		return View();
	};
	let mut entries = Vec::new();

	if let Some(reaper) = player.active_reaper() {
		let open = soul_open();
		let soul = reaper.soul();
		let mut soul_block = View().class("w-full").child(
			Pressable(words.soul_title().to_owned())
				.class("py-3 gap-1")
				.test_id("codex-soul")
				.aria_expanded(open.get())
				.on_press(move || open.update(|open| *open = !*open))
				.child(display(words.soul_title(), 24, BONE))
				.child(body(
					words.soul_birth_link(
						gift_name(&*words, soul.gift),
						temperament_name(&*words, soul.temperament),
					),
					13,
					ASH,
				)),
		);
		if open.get() {
			soul_block = soul_block.child(soul_ledger(
				reaper.genome(),
				reaper.level(),
				reaper.memory(),
			));
		}
		entries.push(soul_block);
	}

	if player.codex().is_empty() {
		entries.push(
			View()
				.class("pt-[14px]")
				.child(body(words.codex_empty(), 15, ASH)),
		);
	}

	for collection in NftCollection::ALL {
		let absorbed = player.absorbed_from(collection);
		if absorbed == 0 {
			continue;
		}
		let complete = absorbed >= NftCollection::SET_SIZE;
		entries.push(ledger_heading(
			collection.title(),
			Some(words.codex_set(absorbed as i64, collection.rarities().len() as i64)),
		));
		entries.push(body(
			if complete {
				words.codex_set_complete().to_owned()
			} else {
				words.codex_set_progress((NftCollection::SET_SIZE - absorbed) as i64)
			},
			13,
			if complete { light } else { ASH },
		));
		entries.push(gap(8));
		for entry in player
			.codex()
			.iter()
			.filter(|entry| entry.nft.collection == collection)
		{
			entries.push(
				View()
					.class("flex-row items-center gap-[14px] py-[6px]")
					.child(husk_portrait(HuskPortraitProps {
						nft: entry.nft,
						size: 44,
						spent: false,
					}))
					.child(
						View()
							.class("flex-1")
							.child(body(entry.nft.title(), 15, BONE)),
					)
					.child(
						View()
							.class("items-end gap-[3px]")
							.child(label(grade_name(&*words, entry.grade), 10, ASH))
							.child(body(
								feeding_name(&*words, entry.feeding),
								12,
								if entry.feeding == Feeding::Feast {
									light
								} else {
									ASH
								},
							)),
					),
			);
		}
	}

	if player.hunts_lost() > 0 {
		entries.push(gap(18));
		entries.push(ledger_row(
			words.codex_lost(),
			player.hunts_lost().to_string(),
			false,
		));
	}

	ledger_page(
		words.codex_overline(),
		Some(words.codex_title()),
		None,
		entries,
	)
}
