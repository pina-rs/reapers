//! The simnet's explorer: the burner wallet, the program's addresses, and
//! every transaction the wallet has signed, newest first.

use std::rc::Rc;

use leptoskit::prelude::*;
use solana_kit_core::Hash;

use super::home;
use crate::l10n::use_l10n;
use crate::lookups::simnet_instruction_name;
use crate::lookups::simnet_role_name;
use crate::platform;
use crate::simnet::FaucetReceipt;
use crate::simnet::SimnetChain;
use crate::simnet::SimnetInstruction;
use crate::simnet::SimnetTransaction;
use crate::simnet::SurfpoolFaucet;
use crate::simnet::SurfpoolRpc;
use crate::state::Game;
use crate::state::Overlay;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::WOUND;
use crate::theme::short_address;
use crate::theme::sol_amount;
use crate::ui::LedgerButton;
use crate::ui::aside;
use crate::ui::body;
use crate::ui::dialog;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_page;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::ui::margin;
use crate::ui::sheet;

/// An address in full, under its name, for copying.
fn address(name: &str, value: impl Into<String>) -> Element {
	View()
		.class("py-[7px] gap-1")
		.child(margin(name))
		.child(figure(value, 12, BONE))
}

/// The explorer.
pub fn page() -> Element {
	let game = Game::get();
	let words = use_l10n();
	let (Some(chain), Some(player)) = (game.simnet(), game.player()) else {
		return View();
	};
	let transactions = game.ledger.get();
	let burned = transactions
		.iter()
		.filter(|transaction| transaction.instruction == SimnetInstruction::Burn)
		.count();
	let wallet = chain.wallet();

	let mut entries = vec![aside(words.simnet_intro()), gap(12)];
	if let Some(rpc) = game.surfpool() {
		entries.push(surfpool_panel(game, rpc, &wallet));
		entries.push(gap(18));
	} else if let Some(problem) = game.surfpool_problem.get() {
		entries.push(
			body(words.simnet_local_rpc_config(&problem), 13, WOUND)
				.test_id("simnet-local-rpc-config"),
		);
		entries.push(gap(18));
	}
	entries.extend([
		ledger_row(words.roster_network(), words.simnet_network_name(), false),
		ledger_row(words.simnet_burned(), burned.to_string(), true),
		address(words.simnet_wallet(), wallet),
		address(
			words.simnet_role_program(),
			SimnetChain::program().to_string(),
		),
		address(
			words.simnet_role_keeper(),
			SimnetChain::keeper().to_string(),
		),
		ledger_heading(
			words.simnet_transactions(),
			Some(transactions.len().to_string()),
		),
	]);
	if transactions.is_empty() {
		entries.push(body(words.simnet_empty(), 14, ASH));
	}
	for transaction in transactions.iter().rev() {
		entries.push(transaction_row(game, transaction));
	}
	entries.push(gap(24));
	entries.push(
		LedgerButton::new(words.simnet_reset())
			.detail(words.simnet_reset_detail())
			.quiet()
			.test_id("simnet-reset")
			.on_press(move || game.overlay.set(Some(Overlay::ConfirmReset))),
	);

	ledger_page(
		words.simnet_overline(),
		Some(words.simnet_title()),
		Some(figure(
			words.common_sol(&sol_amount(player.lamports())),
			14,
			BONE,
		)),
		entries,
	)
}

fn transaction_row(game: Game, transaction: &SimnetTransaction) -> Element {
	let words = use_l10n();
	let instruction = simnet_instruction_name(&*words, transaction.instruction);
	let subject = transaction.nft.map_or_else(
		|| words.common_sol(&sol_amount(transaction.lamports)),
		|nft| nft.title(),
	);
	let slot = words.simnet_slot_value(transaction.slot);
	let signature = transaction.signature.clone();
	Pressable(format!("{instruction}, {subject}, {slot}"))
		.class("flex-row items-center py-[9px] gap-2")
		.test_id(format!("simnet-tx-{}", transaction.signature))
		.child(
			View()
				.class("w-[72px]")
				.child(label(instruction, 11, light())),
		)
		.child(
			View()
				.class("flex-1 gap-[2px]")
				.child(body(subject, 14, BONE))
				.child(body(slot, 12, ASH)),
		)
		.child(figure(short_address(&transaction.signature), 12, ASH))
		.on_press(move || {
			game.overlay
				.set(Some(Overlay::Transaction(signature.clone())))
		})
}

/// What happened when a transaction was sent to the local validator.
fn sent() -> RwSignal<Option<Result<String, String>>> {
	thread_local! {
		static SENT: RwSignal<Option<Result<String, String>>> = RwSignal::new(None);
	}
	SENT.with(|sent| *sent)
}

/// Signs `transaction` again over the validator's blockhash and sends it.
fn send_local(chain: Rc<SimnetChain>, rpc: Rc<SurfpoolRpc>, transaction: SimnetTransaction) {
	let sent = sent();
	sent.set(None);
	platform::spawn(async move {
		let outcome = async {
			let blockhash: Hash = rpc
				.latest_blockhash()
				.await
				.map_err(|error| error.to_string())?
				.parse()
				.map_err(|_| "the validator's blockhash is not one".to_owned())?;
			let wire = chain
				.rebuild(&transaction, blockhash)
				.map_err(|error| error.to_string())?
				.ok_or_else(|| "a faucet pays an airdrop".to_owned())?;
			let signature = rpc
				.send_transaction(&wire)
				.await
				.map_err(|error| error.to_string())?;
			rpc.await_confirmation(&signature)
				.await
				.map_err(|error| error.to_string())?;
			Ok(signature)
		}
		.await;
		sent.set(Some(outcome));
	});
}

/// One transaction in full, as an explorer would show it.
pub fn transaction_sheet(game: Game, signature: &str) -> Element {
	let words = use_l10n();
	let Some(transaction) = game
		.ledger
		.get()
		.into_iter()
		.find(|transaction| transaction.signature == signature)
	else {
		return View();
	};
	let mut entries = vec![
		margin(simnet_instruction_name(&*words, transaction.instruction)),
		display(
			transaction.nft.map_or_else(
				|| words.common_sol(&sol_amount(transaction.lamports)),
				|nft| nft.title(),
			),
			28,
			BONE,
		),
		gap(12),
		ledger_row(words.simnet_status(), words.simnet_finalized(), false),
		ledger_row(words.simnet_slot(), transaction.slot.to_string(), false),
	];
	if transaction.lamports > 0 {
		entries.push(ledger_row(
			words.simnet_amount(),
			words.common_sol(&sol_amount(transaction.lamports)),
			false,
		));
	}
	entries.push(address(
		words.simnet_signature(),
		transaction.signature.clone(),
	));
	entries.push(ledger_row(
		words.simnet_wire(),
		match &transaction.wire {
			Some(wire) if transaction.wire_verifies() => words.simnet_wire_value(wire.len() as i64),
			Some(_) => words.simnet_receipt_fails().to_owned(),
			None => words.simnet_wire_none().to_owned(),
		},
		false,
	));
	entries.push(ledger_heading(words.simnet_accounts(), None));
	for account in &transaction.accounts {
		entries.push(address(
			simnet_role_name(&*words, account.role),
			account.address.clone(),
		));
	}
	if transaction.instruction == SimnetInstruction::Burn {
		let verifies = transaction.receipt_verifies();
		entries.push(ledger_heading(words.simnet_receipt(), None));
		entries.push(aside(words.simnet_receipt_body()));
		entries.push(gap(8));
		entries.push(ledger_row(
			words.simnet_receipt(),
			if verifies {
				words.simnet_receipt_passes()
			} else {
				words.simnet_receipt_fails()
			},
			verifies,
		));
	}
	if let (Some(_), Some(_), Some(_)) = (game.simnet(), game.surfpool(), &transaction.wire) {
		entries.push(gap(18));
		match sent().get() {
			Some(Ok(signature)) => {
				entries.push(ledger_row(
					words.simnet_sent_local(),
					short_address(&signature),
					true,
				));
			}
			Some(Err(reason)) => {
				entries.push(body(words.simnet_local_rpc_failure(&reason), 13, WOUND));
			}
			None => {}
		}
		let transaction = transaction.clone();
		entries.push(
			LedgerButton::new(words.simnet_send_local())
				.test_id("simnet-send-local")
				.on_press(move || {
					if let (Some(chain), Some(rpc)) = (game.simnet(), game.surfpool()) {
						send_local(chain, rpc, transaction.clone());
					}
				}),
		);
	}
	sheet(game, entries)
}

/// Asks, then starts the simnet over and goes back to the threshold.
pub fn reset_dialog(game: Game) -> Element {
	let words = use_l10n();
	dialog(
		game,
		words.simnet_reset(),
		vec![
			margin(words.simnet_reset()),
			body(words.simnet_reset_body(), 15, BONE),
			gap(10),
			LedgerButton::new(words.simnet_reset())
				.test_id("simnet-reset-confirm")
				.on_press(move || {
					game.overlay.set(None);
					game.start_over(|done| {
						if done {
							home();
						}
					});
				}),
			LedgerButton::new(words.simnet_cancel())
				.quiet()
				.on_press(move || game.overlay.set(None)),
		],
	)
}

/// The panel's own state, for the wallet it was opened for.
#[derive(Clone, Copy)]
struct Local {
	wallet: RwSignal<String>,
	checking: RwSignal<bool>,
	airdropping: RwSignal<bool>,
	healthy: RwSignal<Option<bool>>,
	genesis: RwSignal<Option<String>>,
	balance: RwSignal<Option<u64>>,
	receipt: RwSignal<Option<FaucetReceipt>>,
	failure: RwSignal<Option<String>>,
}

fn local() -> Local {
	thread_local! {
		static LOCAL: Local = Local {
			wallet: RwSignal::new(String::new()),
			checking: RwSignal::new(false),
			airdropping: RwSignal::new(false),
			healthy: RwSignal::new(None),
			genesis: RwSignal::new(None),
			balance: RwSignal::new(None),
			receipt: RwSignal::new(None),
			failure: RwSignal::new(None),
		};
	}
	LOCAL.with(|local| *local)
}

fn faucet(game: Game, rpc: &Rc<SurfpoolRpc>) -> SurfpoolFaucet {
	SurfpoolFaucet::new(game.prefs(), rpc.clone(), Rc::new(platform::now))
}

/// Reads the validator: its health, its genesis, and the burner's balance
/// there. The faucet's record is read first: one that cannot be read is the
/// panel's own problem, and must not be mistaken for a fresh slate.
fn check(game: Game, rpc: Rc<SurfpoolRpc>, wallet: String) {
	let local = local();
	local.checking.set(true);
	local.failure.set(None);
	platform::spawn(async move {
		let outcome = async {
			let receipt = faucet(game, &rpc).receipt_for(&wallet).await?;
			rpc.health().await?;
			let genesis = rpc.genesis_hash().await?;
			let balance = rpc.balance(&wallet).await?;
			Ok::<_, crate::simnet::SurfpoolError>((receipt, genesis, balance))
		}
		.await;
		match outcome {
			Ok((receipt, genesis, balance)) => {
				local.healthy.set(Some(true));
				// A restarted Surfpool has a new genesis and has forgotten
				// the old airdrop, so the faucet is offered again.
				local
					.receipt
					.set(receipt.filter(|receipt| receipt.genesis == genesis));
				local.genesis.set(Some(genesis));
				local.balance.set(Some(balance));
			}
			Err(error) => {
				local.healthy.set(Some(false));
				local.failure.set(Some(error.detail));
			}
		}
		local.checking.set(false);
	});
}

fn fund(game: Game, rpc: Rc<SurfpoolRpc>, wallet: String) {
	let local = local();
	local.airdropping.set(true);
	local.failure.set(None);
	platform::spawn(async move {
		match faucet(game, &rpc).fund(&wallet).await {
			Ok(receipt) => {
				local.receipt.set(Some(receipt));
				check(game, rpc, wallet);
			}
			Err(error) => local.failure.set(Some(error.detail)),
		}
		local.airdropping.set(false);
	});
}

/// The development validator beside the simnet: a real RPC to check, and a
/// faucet that pays the burner wallet there without touching the game.
fn surfpool_panel(game: Game, rpc: Rc<SurfpoolRpc>, wallet: &str) -> Element {
	let words = use_l10n();
	let local = local();
	if local.wallet.get_untracked() != wallet {
		local.wallet.set(wallet.to_owned());
		local.receipt.set(None);
		local.balance.set(None);
		check(game, rpc.clone(), wallet.to_owned());
	}
	let status = if local.checking.get() {
		words.simnet_local_rpc_checking()
	} else if local.healthy.get() == Some(true) {
		words.simnet_local_rpc_healthy()
	} else {
		words.simnet_local_rpc_unreachable()
	};

	let mut panel = View()
		.test_id("simnet-local-rpc")
		.child(ledger_heading(words.simnet_local_rpc(), None))
		.child(aside(words.simnet_local_rpc_body()))
		.child(gap(6))
		.child(address(
			words.simnet_local_rpc_endpoint(),
			rpc.endpoint().to_owned(),
		))
		.child(ledger_row(words.simnet_local_rpc_health(), status, false));
	if let Some(genesis) = local.genesis.get() {
		panel = panel.child(address(words.simnet_local_rpc_genesis(), genesis));
	}
	if let Some(lamports) = local.balance.get() {
		panel = panel.child(ledger_row(
			words.simnet_local_rpc_balance(),
			words.common_sol(&sol_amount(lamports as i64)),
			false,
		));
	}
	panel = panel
		.child(gap(8))
		.child(aside(words.simnet_local_rpc_airdrop_body()));
	let receipt = local.receipt.get();
	if let Some(paid) = &receipt {
		panel = panel.child(gap(4)).child(address(
			words.simnet_local_rpc_signature(),
			paid.signature.clone(),
		));
	}
	if let Some(detail) = local.failure.get() {
		panel = panel.child(gap(6)).child(
			body(words.simnet_local_rpc_failure(&detail), 13, WOUND)
				.test_id("simnet-local-rpc-failure"),
		);
	}

	let check_button =
		LedgerButton::new(words.simnet_local_rpc_check()).test_id("simnet-local-rpc-check");
	let check_button = if local.checking.get() {
		check_button.disabled()
	} else {
		let wallet = wallet.to_owned();
		check_button.on_press(move || {
			if let Some(rpc) = game.surfpool() {
				check(game, rpc, wallet.clone());
			}
		})
	};
	let airdrop_label = match &receipt {
		None => words.simnet_local_rpc_airdrop(&sol_amount(SurfpoolFaucet::LAMPORTS as i64)),
		Some(paid) if paid.confirmed => words.simnet_local_rpc_airdropped().to_owned(),
		Some(_) => words.simnet_local_rpc_check().to_owned(),
	};
	let mut airdrop = LedgerButton::new(airdrop_label).test_id("simnet-local-rpc-airdrop");
	if let Some(paid) = &receipt {
		airdrop = airdrop.detail(short_address(&paid.signature));
	}
	let airdrop = if receipt.as_ref().is_some_and(|paid| paid.confirmed) || local.airdropping.get()
	{
		airdrop.disabled()
	} else {
		let wallet = wallet.to_owned();
		airdrop.on_press(move || {
			if let Some(rpc) = game.surfpool() {
				fund(game, rpc, wallet.clone());
			}
		})
	};
	panel.child(gap(10)).child(
		View()
			.class("flex-row gap-[10px]")
			.child(View().class("flex-1").child(check_button))
			.child(View().class("flex-1").child(airdrop)),
	)
}
