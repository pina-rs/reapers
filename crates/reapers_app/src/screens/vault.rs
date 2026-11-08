//! The vault and the market in one ledger: every NFT of every collection,
//! and where each one stands with this player.
//!
//! Buying here is a stand-in. See docs/nfts.md for what replaces it.

use leptoskit::prelude::*;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use reapers_rules::PlayerState;
use reapers_rules::market_listings;

use super::home;
use crate::l10n::use_l10n;
use crate::lookups::rarity_name;
use crate::seams::HuskPortraitProps;
use crate::seams::husk_portrait;
use crate::state::Game;
use crate::state::Overlay;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::short_address;
use crate::theme::sol_amount;
use crate::ui::LedgerButton;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::ledger_heading;
use crate::ui::ledger_page;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::ui::margin;
use crate::ui::sheet;

/// Where an NFT stands with this player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Holding {
	Listed,
	Held,
	Loose,
	Absorbed,
}

fn holding_of(player: &PlayerState, nft: Nft) -> Holding {
	if player.vault().contains(&nft) {
		Holding::Held
	} else if player
		.sightings()
		.iter()
		.any(|sighting| sighting.nft == nft)
	{
		Holding::Loose
	} else if player.codex().iter().any(|entry| entry.nft == nft) {
		Holding::Absorbed
	} else {
		Holding::Listed
	}
}

/// The vault.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let Some(player) = game.player() else {
		return View();
	};
	let mut entries = vec![aside(words.vault_stand_in())];
	for collection in NftCollection::ALL {
		entries.push(ledger_heading(
			collection.title(),
			Some(words.codex_set(
				player.absorbed_from(collection) as i64,
				collection.rarities().len() as i64,
			)),
		));
		for nft in market_listings()
			.into_iter()
			.filter(|nft| nft.collection == collection)
		{
			entries.push(nft_row(game, nft, holding_of(&player, nft)));
		}
	}
	ledger_page(
		words.vault_overline(),
		Some(words.vault_title()),
		Some(
			figure(words.common_sol(&sol_amount(player.lamports())), 14, BONE)
				.test_id("vault-wallet"),
		),
		entries,
	)
}

fn nft_row(game: Game, nft: Nft, holding: Holding) -> Element {
	let words = use_l10n();
	let (status, colour): (String, Rgb) = match holding {
		Holding::Listed => (words.common_sol(&sol_amount(nft.rarity().lamports())), BONE),
		Holding::Held => (words.vault_held().to_owned(), light()),
		Holding::Loose => (words.vault_loose().to_owned(), light()),
		Holding::Absorbed => (words.vault_absorbed().to_owned(), ASH),
	};
	let rarity = rarity_name(&*words, nft.rarity());
	Pressable(format!("{}, {rarity}, {status}", nft.title()))
		.class("flex-row items-center gap-[14px] py-[7px]")
		.test_id(format!("vault-nft-{}", nft.id()))
		.on_press(move || game.overlay.set(Some(Overlay::Nft(nft))))
		.child(husk_portrait(HuskPortraitProps {
			nft,
			size: 48,
			spent: holding == Holding::Absorbed,
		}))
		.child(
			View()
				.class("flex-1 gap-[3px]")
				.child(body(nft.title(), 15, BONE))
				.child(crate::ui::label(rarity, 10, ASH)),
		)
		.child(figure(status, 13, colour))
}

/// One NFT in full: its image, what it is worth, and what can be done with
/// it now.
pub fn nft_sheet(game: Game, nft: Nft) -> Element {
	let words = use_l10n();
	let Some(player) = game.player() else {
		return View();
	};
	let holding = holding_of(&player, nft);
	let rarity = nft.rarity();
	let busy = game.busy.get();
	let mint = nft.mock_mint();

	let action = match holding {
		Holding::Listed => {
			let button = LedgerButton::new(words.vault_buy())
				.detail(words.common_sol(&sol_amount(rarity.lamports())))
				.test_id("vault-buy");
			if busy {
				button.disabled()
			} else {
				button.on_press(move || {
					game.change(
						move |keeper| async move { keeper.acquire(nft).await },
						|_| {},
					);
				})
			}
		}
		Holding::Held => {
			let button = LedgerButton::new(words.vault_release()).test_id("vault-release");
			View()
				.class("gap-3")
				.child(aside(words.vault_release_note()))
				.child(if busy {
					button.disabled()
				} else {
					// Releasing sends the player back to watch for the husk.
					button.on_press(move || {
						game.change(
							move |keeper| async move { keeper.release(nft).await },
							move |done| {
								if done {
									game.overlay.set(None);
									home();
								}
							},
						);
					})
				})
		}
		Holding::Loose => LedgerButton::new(words.vault_loose()).disabled(),
		Holding::Absorbed => LedgerButton::new(words.vault_absorbed()).disabled(),
	};

	sheet(
		game,
		vec![
			View()
				.class("flex-row items-start gap-[18px]")
				.child(husk_portrait(HuskPortraitProps {
					nft,
					size: 132,
					spent: false,
				}))
				.child(
					View()
						.class("flex-1 gap-2")
						.child(margin(rarity_name(&*words, rarity)))
						.child(display(nft.title(), 28, BONE)),
				),
			gap(16),
			ledger_row(words.vault_ward(), rarity.ward().to_string(), false),
			ledger_row(
				words.vault_worth(),
				words.vault_worth_value(rarity.xp(), rarity.anima()),
				false,
			),
			ledger_row(words.vault_mint(), short_address(&mint), false),
			gap(18),
			action,
		],
	)
}
