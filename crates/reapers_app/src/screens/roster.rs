//! Every reaper the player holds, the one that hunts tonight, and the
//! wallet that proves the NFT ones.

use std::rc::Rc;

use leptoskit::prelude::*;
use reapers_rules::Refusal;
use solana_kit_core::Address;

use super::open;
use crate::backend::ChainStatus;
use crate::backend::KeeperError;
use crate::backend::KeeperKind;
use crate::l10n::use_l10n;
use crate::lookups::gift_name;
use crate::lookups::refusal_message;
use crate::lookups::temperament_name;
use crate::platform;
use crate::routes::Route;
use crate::state::Game;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::WOUND;
use crate::theme::sol_amount;
use crate::ui::LedgerButton;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::ledger_heading;
use crate::ui::ledger_page;
use crate::ui::ledger_pip;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::wallet::device_wallet;

/// What the roster has asked the server about the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Chain {
	Unasked,
	Asking,
	Known(Option<ChainStatus>),
	Unreachable,
}

#[derive(Clone, Copy)]
struct Panel {
	chain: RwSignal<Chain>,
	busy: RwSignal<bool>,
	failure: RwSignal<Option<String>>,
	asset: RwSignal<String>,
}

fn panel() -> Panel {
	thread_local! {
		static PANEL: Panel = crate::ui::lasting(|| Panel {
			chain: RwSignal::new(Chain::Unasked),
			busy: RwSignal::new(false),
			failure: RwSignal::new(None),
			asset: RwSignal::new(String::new()),
		});
	}
	PANEL.with(|panel| *panel)
}

/// Asks the server what chain it verifies against.
fn ask_chain(game: Game, panel: Panel) {
	let Some(keeper) = game.keeper() else {
		return;
	};
	panel.chain.set(Chain::Asking);
	platform::spawn(async move {
		panel.chain.set(match keeper.chain_status().await {
			Ok(status) => Chain::Known(status),
			Err(_) => Chain::Unreachable,
		});
	});
}

/// Runs a wallet action, one at a time, and says what went wrong when it
/// does. Wallet cancellation, missing transports and launcher failures are
/// a failed action, never a successful link or import.
fn run(game: Game, panel: Panel, action: impl AsyncFnOnce() -> Result<(), KeeperError> + 'static) {
	if panel.busy.get_untracked() {
		return;
	}
	panel.busy.set(true);
	panel.failure.set(None);
	platform::spawn(async move {
		let result = action().await;
		panel.busy.set(false);
		if let Err(error) = result {
			panel.failure.set(Some(match error.refusal() {
				Some(refusal) => refusal_message(game_words(), refusal).to_owned(),
				None => game_words().common_unreachable().to_owned(),
			}));
		} else {
			game.reload().await;
			ask_chain(game, panel);
		}
	});
}

/// The words for a message set after the page that asked has rendered.
fn game_words() -> &'static dyn crate::l10n::Messages {
	use leptoskit_l10n::Translations as _;
	crate::l10n::AppTranslations::messages(&crate::l10n::AppTranslations::supported_locales()[0])
}

/// The roster.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let light = light();
	let panel = panel();
	let Some(player) = game.player() else {
		return View();
	};
	let busy = panel.busy.get();

	let mut entries = vec![body(words.roster_intro(), 15, ASH), gap(16)];
	for reaper in player.reapers() {
		let active = player.active_reaper_id() == Some(reaper.id());
		let soul = reaper.soul();
		let id = reaper.id().to_owned();
		let kind = if reaper.is_nft() {
			words.roster_nft()
		} else {
			words.roster_starter()
		};
		let mut row = Pressable(reaper.genome().name())
			.class("flex-row items-center gap-3 py-3")
			.test_id(format!("roster-reaper-{}", reaper.id()))
			.aria_selected(active)
			.child(
				View()
					.class("flex-1 gap-1")
					.child(display(reaper.genome().name(), 22, BONE))
					.child(body(
						format!("{kind} · {}", words.studio_level_value(reaper.level())),
						13,
						ASH,
					))
					.child(body(
						format!(
							"{} · {}",
							gift_name(&*words, soul.gift),
							temperament_name(&*words, soul.temperament)
						),
						13,
						ASH,
					)),
			);
		if active {
			row = row.child(
				ledger_pip(true, light).accessibility_label(words.roster_active().to_owned()),
			);
		}
		entries.push(if busy || active {
			row.enabled(!busy)
		} else {
			row.on_press(move || {
				let id = id.clone();
				game.change(
					move |keeper| async move { keeper.select_reaper(&id).await },
					|_| {},
				);
			})
		});
	}
	entries.push(gap(16));
	entries.push(
		LedgerButton::new(words.sanctum_studio())
			.detail(words.sanctum_studio_detail())
			.on_press(|| open(Route::Studio)),
	);
	entries.push(ledger_heading(words.roster_linked(), None));

	match game.keeper_kind.get() {
		Some(KeeperKind::Simnet) => {
			let wallet = game
				.simnet()
				.map(|chain| chain.wallet())
				.unwrap_or_default();
			entries.push(ledger_row(
				words.roster_network(),
				words.simnet_network_name(),
				false,
			));
			entries.push(figure(wallet, 12, BONE));
			entries.push(ledger_row(
				words.roster_balance(),
				sol_amount(player.lamports()),
				false,
			));
			entries.push(gap(12));
			entries.push(aside(words.roster_simnet()));
			entries.push(gap(12));
			entries
				.push(LedgerButton::new(words.sanctum_simnet()).on_press(|| open(Route::Simnet)));
		}
		Some(KeeperKind::Server) => entries.extend(chain_panel(game, panel)),
		_ => entries.push(body(words.roster_offline(), 15, ASH)),
	}

	if busy {
		entries.push(View().class("pt-[18px]").child(LinearProgressIndicator(
			leptoskit::native::Progress::Indeterminate,
		)));
	}
	if let Some(message) = panel.failure.get() {
		entries.push(
			View()
				.class("pt-[18px]")
				.child(Alert(message).class(format!("text-[15px] text-[{}]", WOUND.hex()))),
		);
	}

	ledger_page(words.roster_title(), None, None, entries)
}

/// The server's chain, the linked wallet, and the ways to link one and to
/// bring a reaper NFT in.
fn chain_panel(game: Game, panel: Panel) -> Vec<Element> {
	let words = use_l10n();
	let busy = panel.busy.get();
	let status = match panel.chain.get() {
		Chain::Unasked => {
			ask_chain(game, panel);
			return vec![LinearProgressIndicator(
				leptoskit::native::Progress::Indeterminate,
			)];
		}
		Chain::Asking => {
			return vec![LinearProgressIndicator(
				leptoskit::native::Progress::Indeterminate,
			)];
		}
		Chain::Unreachable => {
			return vec![
				body(words.common_unreachable(), 15, WOUND),
				LedgerButton::new(words.common_retry())
					.quiet()
					.on_press(move || ask_chain(game, panel)),
			];
		}
		Chain::Known(None) => return vec![body(words.roster_offline(), 15, ASH)],
		Chain::Known(Some(status)) => status,
	};

	let wallet = device_wallet();
	let can_connect = wallet.available();
	let mut entries = vec![ledger_row(
		words.roster_network(),
		status.chain.clone(),
		false,
	)];
	if let Some(address) = &status.wallet {
		entries.push(figure(address.clone(), 12, BONE));
		if let Some(lamports) = status.lamports {
			entries.push(ledger_row(
				words.roster_balance(),
				sol_amount(lamports),
				false,
			));
		}
	}
	entries.push(gap(12));
	entries.push(aside(words.roster_link_help()));
	entries.push(gap(12));
	if !can_connect {
		entries.push(aside(words.roster_wallet_unsupported()));
	}
	let connect = LedgerButton::new(words.roster_connect()).test_id("roster-connect");
	entries.push(if busy || !can_connect || !status.wallets_enabled() {
		connect.disabled()
	} else {
		let status = status.clone();
		connect.on_press(move || {
			let status = status.clone();
			run(game, panel, async move || {
				let keeper = game
					.keeper()
					.ok_or_else(|| KeeperError::unavailable("no keeper"))?;
				let prover: Rc<dyn crate::wallet::WalletProver> = Rc::from(device_wallet());
				keeper.connect_wallet(&status, &*prover).await
			});
		})
	});
	entries.push(gap(8));
	let refresh = LedgerButton::new(words.roster_refresh());
	entries.push(if busy {
		refresh.disabled()
	} else {
		refresh.on_press(move || run(game, panel, async move || Ok(())))
	});

	if status.collection.is_none() {
		entries.push(
			View()
				.class("pt-[18px]")
				.child(body(words.roster_unconfigured(), 15, ASH)),
		);
		return entries;
	}

	entries.push(ledger_heading(words.roster_import(), None));
	entries.push(aside(words.roster_import_help()));
	entries.push(gap(12));
	entries.push(
		TextInput()
			.class(
				"w-full min-h-[48px] px-3 border border-[#e9e4d42e] text-[15px] text-[#e9e4d4] bg-[#0f1116]",
			)
			.accessibility_label(words.roster_asset().to_owned())
			.placeholder(words.roster_asset())
			.value(panel.asset.get())
			.autocomplete("off")
			.test_id("roster-asset")
			.on_change(move |text: &str| panel.asset.set(text.to_owned())),
	);
	entries.push(gap(12));
	let import = LedgerButton::new(words.roster_import()).test_id("roster-import");
	entries.push(if busy || status.wallet.is_none() {
		import.disabled()
	} else {
		import.on_press(move || {
			let address = panel.asset.get_untracked().trim().to_owned();
			run(game, panel, async move || {
				if address.parse::<Address>().is_err() {
					return Err(Refusal::NotOwned.into());
				}
				let keeper = game
					.keeper()
					.ok_or_else(|| KeeperError::unavailable("no keeper"))?;
				keeper.import_reaper(&address).await?;
				panel.asset.set(String::new());
				Ok(())
			});
		})
	});

	if let Some(market) = status.marketplace_uri.clone() {
		entries.push(gap(18));
		entries.push(aside(words.roster_buy_help()));
		entries.push(gap(12));
		let buy = LedgerButton::new(words.roster_buy());
		entries.push(if busy {
			buy.disabled()
		} else {
			buy.on_press(move || {
				let market = market.clone();
				run(game, panel, async move || {
					if !market.starts_with("https://") {
						return Err(KeeperError::unavailable(
							"the marketplace is not an HTTPS page",
						));
					}
					platform::open_url(&market)
						.await
						.map_err(KeeperError::unavailable)
				});
			})
		});
	}
	entries
}
