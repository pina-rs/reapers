//! The keeper: what answers every operation in `reapers_api`.
//!
//! Each answer finds the player from the request's session, so nothing the
//! game sends names a player, and each change is one shared rule applied to
//! the stored state. Anything the keeper must decide (a seed, the time, how
//! well a reaper fed) is decided here, never sent by the game.

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use leptoskit_auth::Account;
use leptoskit_auth::SignedIn;
use leptoskit_db::Db;
use leptoskit_db::Id;
use leptoskit_db::Json;
use leptoskit_rpc as rpc;
use reapers_api::chain::Chain;
use reapers_api::chain::Challenge;
use reapers_api::chain::Husks;
use reapers_api::keeper::Change;
use reapers_api::player::ChainReceipt;
use reapers_api::player::KeeperKey;
use reapers_api::player::SettledHunt;
use reapers_api::refused;
use reapers_rules::Absorption;
use reapers_rules::Art;
use reapers_rules::Difficulty;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntLogError;
use reapers_rules::HuntSettlement;
use reapers_rules::Judging;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftSite;
use reapers_rules::RuleResult;
use reapers_rules::Timestamp;
use reapers_rules::Weapon;
use sha2::Digest;
use sha2::Sha256;
use solana_kit_core::Address;
use solana_kit_core::transaction::Lifetime;

use crate::absorb;
use crate::books;
use crate::books::breach;
use crate::chain::Gateway;
use crate::config::RunMode;
use crate::receipt;
use crate::receipt::Receipt;
use crate::rift::Chair;
use crate::rift::RiftHall;
use crate::signer::ReceiptSigner;
use crate::tables::HuskEscrow;
use crate::tables::NewHuskEscrow;
use crate::tables::NewWalletChallenge;
use crate::tables::WalletChallenge;
use crate::wallet;

/// How much kinder the keeper's replay of a hunt is than the phone's own
/// play of it. See `HuntStats::derive`.
pub const REPLAY_LENIENCY: f64 = 1.03;

/// Maximum base64 length accepted for a packed [`HuntLog`], bounding memory
/// allocation before decoding (~384 KiB decoded).
const MAX_ENCODED_LOG_BYTES: usize = 512 * 1024;

/// Maximum concurrent CPU-intensive hunt replays on blocking threads.
const MAX_CONCURRENT_REPLAYS: usize = 8;

/// The keeper, with everything it reaches outside the database.
pub struct Keeper {
	mode: RunMode,
	db: Db,
	gateway: Gateway,
	signer: ReceiptSigner,
	hall: RiftHall,
	replay_permits: Arc<tokio::sync::Semaphore>,
}

impl std::fmt::Debug for Keeper {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Keeper")
			.field("mode", &self.mode)
			.field("gateway", &self.gateway)
			.field("signer", &self.signer)
			.finish_non_exhaustive()
	}
}

/// The time now, as the rules take it.
pub fn now() -> Timestamp {
	Timestamp::from_micros(chrono::Utc::now().timestamp_micros())
}

/// A seed only the keeper knows, from the operating system's secure random
/// numbers.
///
/// # Errors
///
/// A server error when the operating system has none to give.
pub fn seed() -> rpc::Result<i64> {
	getrandom::u32()
		.map(i64::from)
		.map_err(|_| rpc::Error::server("no secure random numbers"))
}

/// The signed-in account, or unauthorized.
pub fn me(identity: &rpc::Identity) -> rpc::Result<Id<Account>> {
	identity
		.context
		.get::<SignedIn>()
		.map(|signed_in| signed_in.account)
		.ok_or_else(|| rpc::Error::unauthorized("sign in first"))
}

impl Keeper {
	/// A keeper in `mode` that keeps its books in `db`, reads the chain
	/// through `gateway`, signs with `signer` and hosts rifts in `hall`.
	pub fn new(
		mode: RunMode,
		db: &Db,
		gateway: Gateway,
		signer: ReceiptSigner,
		hall: RiftHall,
	) -> Self {
		let permits = std::thread::available_parallelism()
			.map_or(MAX_CONCURRENT_REPLAYS, std::num::NonZeroUsize::get)
			.clamp(4, MAX_CONCURRENT_REPLAYS * 2);

		Self {
			mode,
			db: db.system("the keeper's own transactions"),
			gateway,
			signer,
			hall,
			replay_permits: Arc::new(tokio::sync::Semaphore::new(permits)),
		}
	}

	/// The keeper's run mode.
	pub const fn mode(&self) -> RunMode {
		self.mode
	}

	/// The chain, as the keeper reads it.
	pub const fn gateway(&self) -> &Gateway {
		&self.gateway
	}

	/// The key receipts are signed with.
	pub const fn signer(&self) -> &ReceiptSigner {
		&self.signer
	}

	/// The rift hall, for draining on graceful shutdown.
	pub const fn hall(&self) -> &RiftHall {
		&self.hall
	}

	/// The database, bypassing policies: for the keeper's own transactions
	/// and its sign-in routes.
	pub const fn db(&self) -> &Db {
		&self.db
	}

	fn simulation(&self) -> rpc::Result<()> {
		self.mode.require_simulation().map_err(refused)
	}

	/// The difficulties a hunt may have been played at. Easy and hard are
	/// for testing, so a keeper that pays for real takes only the game's
	/// own.
	fn judging(&self) -> Judging {
		Judging {
			leniency: REPLAY_LENIENCY,
			difficulties: if self.mode == RunMode::Production {
				vec![Difficulty::Normal]
			} else {
				Difficulty::ALL.to_vec()
			},
		}
	}

	/// Applies `rule` to `account`'s books in a transaction of the keeper's
	/// own, run again when another gets in its way: for work that happens
	/// outside any operation's transaction, such as paying for a rift.
	pub async fn change_alone(
		&self,
		account: Id<Account>,
		mut rule: impl FnMut(PlayerState) -> rpc::Result<PlayerState>,
	) -> rpc::Result<PlayerState> {
		books::transact_serializable!(&self.db, |tx| {
			books::change(tx, &self.gateway, account, None, &mut rule).await
		})
	}

	async fn load(&self, account: Id<Account>) -> rpc::Result<PlayerState> {
		if let Some(state) = books::load_fast(&self.db, account).await? {
			return Ok(state);
		}

		self.change_alone(account, Ok).await
	}

	async fn fast_action(
		&self,
		account: Id<Account>,
		mut rule: impl FnMut(PlayerState) -> rpc::Result<PlayerState>,
	) -> rpc::Result<PlayerState> {
		if let Some(fast) = books::load_fast_raw(&self.db, account).await? {
			let next = rule(fast.state.clone())?;
			if next == fast.state {
				return Ok(next);
			}
			if books::compare_and_swap_state(&self.db, &fast.id, &fast.raw_state, &next).await? {
				return Ok(next);
			}
		}

		self.change_alone(account, rule).await
	}

	async fn apply_action(&self, account: Id<Account>, change: Change) -> rpc::Result<PlayerState> {
		match change {
			Change::SelectReaper(id) => {
				self.fast_action(account, move |state| breach(state.select_reaper(&id)))
					.await
			}
			Change::Wield(id) => {
				let weapon = parsed(Weapon::parse(&id).ok(), Refusal::NotOwned)
					.map_err(|b| refused(b.refusal))?;
				self.fast_action(account, move |state| breach(state.wield(weapon)))
					.await
			}
			Change::Wear(name) => {
				let outfit = parsed(Outfit::from_name(&name), Refusal::NotOwned)
					.map_err(|b| refused(b.refusal))?;
				self.fast_action(account, move |state| breach(state.wear(outfit)))
					.await
			}
			Change::Attune(name) => {
				let art = parsed(Art::from_name(&name), Refusal::NotOwned)
					.map_err(|b| refused(b.refusal))?;
				self.fast_action(account, move |state| breach(state.attune(art)))
					.await
			}
			other => {
				Err(rpc::Error::server(format!(
					"change {other:?} requires a mutation transaction"
				)))
			}
		}
	}

	async fn change(&self, cx: &rpc::MutationCx, change: Change) -> rpc::Result<PlayerState> {
		let account = me(cx.identity())?;
		let tx = cx.tx();
		let apply = |rule| self.apply(tx, account, rule);

		match change {
			Change::Load => apply(Box::new(Ok)).await,
			Change::TakeOath => {
				let seed = seed()?;

				apply(Box::new(move |state| state.take_oath(seed))).await
			}
			Change::SelectReaper(id) => {
				apply(Box::new(move |state| state.select_reaper(&id))).await
			}
			Change::Acquire(id) => {
				self.simulation()?;
				apply(Box::new(move |state| {
					state.acquire(parsed(Nft::parse(&id).ok(), Refusal::NotListed)?)
				}))
				.await
			}
			Change::Release { nft, escrow } => self.release(cx, account, &nft, escrow).await,
			Change::BuyWeapon(id) => {
				apply(Box::new(move |state| {
					state.buy_weapon(parsed(Weapon::parse(&id).ok(), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Wield(id) => {
				apply(Box::new(move |state| {
					state.wield(parsed(Weapon::parse(&id).ok(), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::BuyOutfit(name) => {
				apply(Box::new(move |state| {
					state.buy_outfit(parsed(Outfit::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Wear(name) => {
				apply(Box::new(move |state| {
					state.wear(parsed(Outfit::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Attune(name) => {
				apply(Box::new(move |state| {
					state.attune(parsed(Art::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::ImportReaper(asset) => {
				books::change(tx, &self.gateway, account, Some(&asset), Ok).await
			}
		}
	}

	/// Applies `rule` to `account`'s books in the operation's transaction.
	async fn apply(
		&self,
		tx: &leptoskit_db::Tx,
		account: Id<Account>,
		rule: Box<dyn FnOnce(PlayerState) -> RuleResult<PlayerState> + Send + '_>,
	) -> rpc::Result<PlayerState> {
		books::change(tx, &self.gateway, account, None, |state| {
			breach(rule(state))
		})
		.await
	}

	/// Releases `nft` into the veil; with `escrow`, binds the husk to the
	/// mint the player escrowed on-chain, once the chain shows the escrow.
	async fn release(
		&self,
		cx: &rpc::MutationCx,
		account: Id<Account>,
		nft: &str,
		escrow: Option<String>,
	) -> rpc::Result<PlayerState> {
		self.simulation()?;

		let tx = cx.tx();
		let seed = seed()?;
		let now = now();
		let bound = match escrow {
			Some(mint) => {
				self.demo_husk(nft, &mint)?;
				let (wallet, mint, _escrow) = self.escrowed(tx, account, &mint).await?;
				Some((wallet, mint))
			}
			None => None,
		};
		let mut sighting = None;
		let state = books::change(tx, &self.gateway, account, None, |state| {
			let nft = parsed(Nft::parse(nft).ok(), Refusal::NotListed)
				.map_err(|breach| refused(breach.refusal))?;

			sighting = Some(state.next_sighting_id());
			breach(state.release(nft, seed, now))
		})
		.await?;

		if let (Some((wallet, mint)), Some(sighting)) = (bound, sighting) {
			NewHuskEscrow {
				account_id: account,
				sighting_id: sighting,
				mint_key: format!("{}:{mint}", self.gateway.config().chain),
				mint: mint.to_string(),
				wallet,
				nft: None,
				absorption: None,
			}
			.insert(tx)
			.await
			.map_err(|error| {
				if matches!(error, leptoskit_db::DbError::Conflict { .. }) {
					refused(Refusal::AlreadyHeld)
				} else {
					rpc::Error::from(error)
				}
			})?;
		}

		Ok(state)
	}

	/// Checks `mint` is the asset the demo husk manifest says the husk `nft`
	/// stands for, so a cheap asset cannot be burned for a rare husk's pay.
	/// Without a manifest no mint stands for any husk, and an escrowed
	/// release is refused as `chainUnconfigured`.
	fn demo_husk(&self, nft: &str, mint: &str) -> rpc::Result<()> {
		let config = self.gateway.config();

		if config.husks.is_empty() {
			return Err(refused(Refusal::ChainUnconfigured));
		}

		let asset = Nft::parse(nft)
			.ok()
			.and_then(|nft| config.husk_asset(nft))
			.map(|asset| asset.to_string());

		if asset.as_deref() == Some(mint) {
			Ok(())
		} else {
			Err(refused(Refusal::NotInVault))
		}
	}

	/// Checks `mint` is escrowed with the program by `wallet`, returning the
	/// parsed mint address and decoded on-chain [`Escrow`](crate::escrow::Escrow).
	async fn escrowed_with_wallet(
		&self,
		wallet: &str,
		mint: &str,
	) -> rpc::Result<(Address, crate::escrow::Escrow)> {
		let Ok(address) = mint.parse::<Address>() else {
			return Err(refused(Refusal::NotInVault));
		};
		let Ok(wallet_address) = wallet.parse::<Address>() else {
			return Err(refused(Refusal::WalletUnproven));
		};

		let recorded = self
			.gateway
			.escrow_of(&wallet_address, &address)
			.await
			.map_err(refused)?;

		Ok((address, recorded))
	}

	/// Checks `mint` is escrowed with the program by `account`'s proved
	/// wallet, and returns the wallet, the mint, and the decoded on-chain
	/// [`Escrow`](crate::escrow::Escrow).
	async fn escrowed(
		&self,
		tx: &impl leptoskit_db::Read,
		account: Id<Account>,
		mint: &str,
	) -> rpc::Result<(String, Address, crate::escrow::Escrow)> {
		let Some(wallet) = books::wallet_of(tx, account).await? else {
			return Err(refused(Refusal::WalletUnproven));
		};
		let (address, recorded) = self.escrowed_with_wallet(&wallet, mint).await?;

		Ok((wallet, address, recorded))
	}

	async fn settle_hunt(
		&self,
		account: Id<Account>,
		sighting: i64,
		log: &str,
	) -> rpc::Result<SettledHunt> {
		self.simulation()?;

		// Pre-check base64 size bound before allocating, reading DB, or decoding.
		if log.len() > MAX_ENCODED_LOG_BYTES {
			return Err(refused(Refusal::HuntUnproven));
		}

		let now = now();
		let feeding_seed = seed()?;
		let judging = self.judging();

		// Fetch player_books, linked wallet, and matching husk_escrows row in
		// one SQL round-trip outside any transaction.
		let prefetch = books::prefetch_hunt(&self.db, account, sighting).await?;
		let (fast_snapshot, wallet, bound, initial_state) = match prefetch {
			Some(prefetch) => {
				let has_nft = prefetch
					.state
					.reapers()
					.iter()
					.any(reapers_rules::ReaperState::is_nft);
				let fast_snapshot = (!has_nft).then_some((prefetch.book_id, prefetch.raw_state));
				(
					fast_snapshot,
					prefetch.wallet,
					prefetch.bound,
					Some(prefetch.state),
				)
			}
			None => (None, None, None, None),
		};

		// Verify on-chain escrow outside any DB transaction using the
		// already-fetched wallet (eliminating duplicate wallet_of queries).
		let verified_escrow = if let Some(bound) = &bound {
			if wallet.as_deref() != Some(bound.wallet.as_str()) {
				return Err(refused(Refusal::WalletUnproven));
			}

			let (_, recorded) = self
				.escrowed_with_wallet(&bound.wallet, &bound.mint)
				.await?;
			Some(recorded)
		} else {
			None
		};

		let initial_state = match (fast_snapshot.as_ref(), initial_state) {
			(Some(_), Some(state)) => state,
			_ => self.change_alone(account, Ok).await?,
		};

		breach(initial_state.huntable(sighting, now))?;

		// Run base64 decoding + deterministic replay off the Tokio worker pool
		// and completely outside any PostgreSQL transaction.
		let replay_state = initial_state.clone();
		let replay_judging = judging.clone();
		let encoded_log = log.to_owned();
		let permit = self
			.replay_permits
			.clone()
			.acquire_owned()
			.await
			.map_err(|_| rpc::Error::server("hunt replay pool closed"))?;
		let (decoded, judged) = tokio::task::spawn_blocking(move || {
			let _permit = permit;
			let decoded = decode_log(&encoded_log)?;
			let judged = breach(replay_state.judge_hunt(
				sighting,
				&decoded,
				now,
				feeding_seed,
				&replay_judging,
			))?;
			Ok::<_, rpc::Error>((decoded, judged))
		})
		.await
		.map_err(|_| rpc::Error::server("hunt replay task aborted"))??;

		let player = player_bytes(account, wallet.as_deref());
		let (pre_settlement, pre_receipt) = self.sign(&judged, player, bound.as_ref())?;

		let (settlement, receipt) = self
			.commit_hunt_settlement(
				account,
				sighting,
				fast_snapshot.as_ref(),
				&initial_state,
				&decoded,
				now,
				feeding_seed,
				&judging,
				player,
				bound.as_ref(),
				&pre_settlement,
				pre_receipt.as_ref(),
			)
			.await?;

		// Build the on-chain burn transaction outside any DB transaction.
		let receipt = match (receipt, &bound, verified_escrow.as_ref()) {
			(Some(receipt), Some(_), escrow) => Some(self.chain_receipt(receipt, escrow).await),
			_ => None,
		};

		Ok(SettledHunt {
			settlement,
			receipt,
		})
	}

	/// Persists a settled hunt. On the common uncontended non-NFT fast path,
	/// commits with a single atomic compare-and-swap `UPDATE` (plus escrow
	/// update if escrowed), falling back to a DB-only `SERIALIZABLE`
	/// transaction with re-judging if a concurrent mutation modified the row
	/// or if NFT reaper ledger records must be reconciled.
	#[expect(clippy::too_many_arguments, reason = "internal commit helper")]
	async fn commit_hunt_settlement(
		&self,
		account: Id<Account>,
		sighting: i64,
		fast_snapshot: Option<&(leptoskit_db::Value, serde_json::Value)>,
		initial_state: &PlayerState,
		decoded: &HuntLog,
		now: Timestamp,
		feeding_seed: i64,
		judging: &Judging,
		player: [u8; 32],
		bound: Option<&books::BoundEscrow>,
		pre_settlement: &HuntSettlement,
		pre_receipt: Option<&SignedReceipt>,
	) -> rpc::Result<(HuntSettlement, Option<SignedReceipt>)> {
		// Uncontended non-NFT fast path: 1 atomic CAS UPDATE (0 BEGIN/COMMIT
		// when no escrow row needs updating; short READ COMMITTED tx if escrowed).
		if let Some((book_id, expected_json)) = fast_snapshot {
			let needs_escrow_write = pre_receipt.is_some() && bound.is_some();
			if !needs_escrow_write {
				if books::compare_and_swap_state(
					&self.db,
					book_id,
					expected_json,
					&pre_settlement.state,
				)
				.await?
				{
					return Ok((pre_settlement.clone(), pre_receipt.cloned()));
				}
			} else if let (Some(signed_receipt), Some(bound)) = (pre_receipt, bound) {
				let tx = self.db.begin().await?;
				let swapped = books::compare_and_swap_state(
					&tx,
					book_id,
					expected_json,
					&pre_settlement.state,
				)
				.await?;
				if swapped {
					books::record_escrow_absorption(&tx, &bound.id, &signed_receipt.absorption)
						.await?;
					tx.commit().await?;
					return Ok((pre_settlement.clone(), Some(signed_receipt.clone())));
				}
				let _ = tx.rollback().await;
			}
		}

		// Contended or NFT-reaper fallback: full SERIALIZABLE reconcile + commit.
		books::transact_serializable!(&self.db, |tx| {
			async {
				let prepared = books::prepare(tx, &self.gateway, account, None).await?;
				let (settlement, receipt) = if &prepared.state == initial_state {
					(pre_settlement.clone(), pre_receipt.cloned())
				} else {
					let rejudged = breach(prepared.state.judge_hunt(
						sighting,
						decoded,
						now,
						feeding_seed,
						judging,
					))?;
					self.sign(&rejudged, player, bound)?
				};
				books::commit(tx, &self.gateway, &prepared, settlement.state.clone()).await?;

				if let (Some(signed_receipt), Some(bound)) = (&receipt, bound) {
					books::record_escrow_absorption(tx, &bound.id, &signed_receipt.absorption)
						.await?;
				}

				Ok::<_, rpc::Error>((settlement, receipt))
			}
			.await
		})
	}

	/// `judged` with the absorption it recorded signed. A husk that got
	/// away leaves nothing to sign.
	///
	/// The receipt is made out to the player's proved wallet, for the
	/// escrowed husk's mint. A husk of the stand-in market has no mint and a
	/// guest no wallet, so their places hold the SHA-256 of the stand-in
	/// identifiers instead: such a receipt is signed and kept with the
	/// books, and no chain could act on it.
	fn sign(
		&self,
		judged: &HuntSettlement,
		player: [u8; 32],
		bound: Option<&books::BoundEscrow>,
	) -> rpc::Result<(HuntSettlement, Option<SignedReceipt>)> {
		if !judged.outcome.absorbed() {
			return Ok((judged.clone(), None));
		}

		let Some(absorption) = judged.state.codex().last() else {
			return Err(rpc::Error::server("an absorbed husk left no codex entry"));
		};
		let mint = match bound {
			Some(bound) => {
				bound
					.mint
					.parse::<Address>()
					.map(|mint| *mint.as_bytes())
					.map_err(|_| rpc::Error::server("an escrow binding holds no address"))?
			}
			None => Sha256::digest(absorption.nft.id().as_bytes()).into(),
		};
		let Some(receipt) = receipt::of(player, mint, absorption) else {
			return Err(rpc::Error::server("an absorption does not fit its receipt"));
		};
		let message = receipt.encode();
		let signature = self.signer.sign(&message);
		let encoded = STANDARD.encode(signature.as_bytes());
		let mut settlement = judged.clone();

		settlement.state = judged.state.sign_absorption(absorption.nft, &encoded);

		Ok((
			settlement,
			Some(SignedReceipt {
				receipt,
				signature,
				absorption: absorption.signed_with(encoded),
			}),
		))
	}

	/// The burn of the absorbed husk `nft` names, again: the receipt its
	/// hunt was settled with, rebuilt from the absorption kept with its
	/// escrow binding, in a transaction with a fresh blockhash.
	///
	/// The keeper signs exactly what it signed then. Its signature is
	/// deterministic, so the receipt rebuilt from the same fields must carry
	/// the signature the codex holds; one that does not (the keeper's key
	/// has changed) is a server error, never a new receipt.
	async fn absorb_transaction(
		&self,
		account: Id<Account>,
		nft: &str,
	) -> rpc::Result<ChainReceipt> {
		self.simulation()?;

		let Ok(parsed_nft) = Nft::parse(nft) else {
			return Err(refused(Refusal::NotInVault));
		};
		let canonical_nft = parsed_nft.id();

		let bound_fut = async {
			HuskEscrow::query()
				.filter(|escrow| {
					escrow.account_id.eq(account) & escrow.nft.eq(canonical_nft.clone())
				})
				.order_by(|escrow| escrow.created_at.desc())
				.first(&self.db)
				.await
				.map_err(rpc::Error::from)
		};
		let (bound, wallet) = tokio::try_join!(bound_fut, books::wallet_of(&self.db, account))?;
		let Some((bound, Json(absorption))) = bound.and_then(|bound| {
			bound
				.absorption
				.clone()
				.map(|absorption| (bound, absorption))
		}) else {
			return Err(refused(Refusal::NotInVault));
		};

		// Burned only by the wallet that released it, while the escrow it
		// was released into still holds it.
		if wallet.as_deref() != Some(bound.wallet.as_str()) {
			return Err(refused(Refusal::WalletUnproven));
		}

		let (mint, recorded) = self
			.escrowed_with_wallet(&bound.wallet, &bound.mint)
			.await?;
		let player = player_bytes(account, Some(&bound.wallet));
		let Some(receipt) = receipt::of(player, *mint.as_bytes(), &absorption) else {
			return Err(rpc::Error::server("an absorption does not fit its receipt"));
		};
		let signature = self.signer.sign(&receipt.encode());

		if absorption.signature.as_deref() != Some(STANDARD.encode(signature.as_bytes()).as_str()) {
			return Err(rpc::Error::server(
				"the receipt no longer carries the signature it was settled with",
			));
		}

		let chain_receipt = self
			.chain_receipt(
				SignedReceipt {
					receipt,
					signature,
					absorption,
				},
				Some(&recorded),
			)
			.await;

		if chain_receipt.transaction.is_none() {
			return Err(refused(Refusal::ChainUnavailable));
		}

		Ok(chain_receipt)
	}

	/// The receipt as the wallet takes it to the chain, with the absorb
	/// transaction when the chain says how to burn the husk and gives a
	/// blockhash for it.
	async fn chain_receipt(
		&self,
		signed: SignedReceipt,
		escrow: Option<&crate::escrow::Escrow>,
	) -> ChainReceipt {
		let message = signed.receipt.encode();
		let wallet = Address::from_bytes(signed.receipt.player);
		let mint = Address::from_bytes(signed.receipt.mint);
		let keeper = self.signer.address();
		let burn_fut = async {
			if let Some(recorded) = escrow {
				self.gateway.absorb_target_for(&mint, recorded).await
			} else {
				self.gateway.absorb_target(&mint).await
			}
		};
		let (burn_res, blockhash_res) = tokio::join!(burn_fut, self.gateway.latest_blockhash());
		let transaction = match (burn_res, blockhash_res) {
			(Ok(burn), Ok((blockhash, last_valid_block_height))) => {
				absorb::transaction(
					&keeper,
					&signed.receipt,
					&signed.signature,
					burn,
					Lifetime::Blockhash {
						blockhash,
						last_valid_block_height,
					},
				)
				.map(|bytes| STANDARD.encode(bytes))
			}
			_ => None,
		};

		ChainReceipt {
			message: STANDARD.encode(message),
			signature: STANDARD.encode(signed.signature.as_bytes()),
			keeper: keeper.to_string(),
			wallet: wallet.to_string(),
			mint: mint.to_string(),
			transaction,
		}
	}

	async fn chain_status(&self, account: Id<Account>) -> rpc::Result<Chain> {
		let wallet = books::wallet_of(&self.db, account).await?;
		let lamports = match &wallet {
			Some(wallet) => {
				let address = wallet
					.parse::<Address>()
					.map_err(|_| refused(Refusal::WalletUnproven))?;

				Some(self.gateway.balance(&address).await.map_err(refused)?)
			}
			None => None,
		};
		let config = self.gateway.config();

		Ok(Chain {
			chain: config.chain.clone(),
			identity_uri: config.identity_uri.to_string(),
			collection: config.collection.map(|address| address.to_string()),
			marketplace_uri: config.marketplace_uri.as_ref().map(ToString::to_string),
			program: config.program.map(|address| address.to_string()),
			husks: config
				.husks
				.iter()
				.map(|husks| {
					Husks {
						collection: husks.collection.name().to_owned(),
						address: husks.address.to_string(),
						assets: husks.assets.iter().map(ToString::to_string).collect(),
					}
				})
				.collect(),
			wallet,
			lamports,
		})
	}

	async fn wallet_challenge(
		&self,
		cx: &rpc::MutationCx,
		address: &str,
	) -> rpc::Result<Challenge> {
		if !self.gateway.config().wallets_enabled() {
			return Err(refused(Refusal::ChainUnconfigured));
		}

		if !wallet::is_address(address) {
			return Err(refused(Refusal::WalletUnproven));
		}

		let account = me(cx.identity())?;
		let tx = cx.tx();

		// Issuing a sign-in challenge only needs local books; check existence
		// without fetching or parsing the large `state` JSONB on existing accounts.
		if !books::exists(tx, account).await? {
			books::open(tx, account).await?;
		}

		let issued = chrono::Utc::now();
		let expires = issued + wallet::LIFETIME;
		let nonce = challenge_id()?;
		let message = wallet::message(self.gateway.config(), address, &nonce, issued, expires);

		WalletChallenge::query()
			.filter(|challenge| challenge.account_id.eq(account))
			.delete(tx)
			.await?;
		NewWalletChallenge {
			challenge_id: nonce.clone(),
			account_id: account,
			address: address.to_owned(),
			message: message.clone(),
			expires_at: expires,
			consumed: false,
		}
		.insert(tx)
		.await
		.map_err(books::conflict_as_contended)?;

		Ok(Challenge {
			challenge_id: nonce,
			address: address.to_owned(),
			message,
			expires_at: expires.to_rfc3339(),
		})
	}

	/// Seats `account` in the rift at `site` and follows it: the stream the
	/// `enter_rift` operation sends.
	pub async fn enter_rift(
		self: Arc<Self>,
		account: Id<Account>,
		site: &str,
		demo: bool,
	) -> rpc::Result<BoxStream<'static, rpc::Result<RiftFrame>>> {
		// Reported results are not replayed yet; their rewards belong only in
		// the simulated development economy. This is also what keeps the demo
		// out of staging and production: there is deliberately no bypass.
		self.simulation()?;

		let Some(site) = RiftSite::from_name(site) else {
			return Err(refused(Refusal::NotAttuned));
		};
		let (entrant, reaper) = if let Some(state) = books::load_fast(&self.db, account).await? {
			let entrant = breach(if demo {
				demo_entrant(&state)
			} else {
				state.entrant_for(site)
			})?;
			(entrant, state.active_reaper_id().map(str::to_owned))
		} else {
			let mut entering = None;

			self.change_alone(account, |state| {
				let entrant = breach(if demo {
					demo_entrant(&state)
				} else {
					state.entrant_for(site)
				})?;

				entering = Some((entrant, state.active_reaper_id().map(str::to_owned)));
				Ok(state)
			})
			.await?;

			entering.ok_or_else(|| rpc::Error::server("the reaper was not read"))?
		};

		let settler_keeper = Arc::clone(&self);
		let settler: crate::rift::RiftSettler =
			Arc::new(move |seat_account, seat_reaper, reward| {
				let keeper = Arc::clone(&settler_keeper);
				Box::pin(async move {
					keeper
						.change_alone(seat_account, move |state| {
							// Unverified rift rewards only settle on the oathbound
							// starter (never tradable NFT reapers).
							if let Some(id) = seat_reaper.as_deref()
								&& state.reapers().iter().any(|r| r.id() == id && r.is_nft())
							{
								return Err(refused(Refusal::RiftUnproven));
							}

							breach(state.settle_rift(
								reward.xp,
								reward.anima,
								seat_reaper.as_deref(),
							))
						})
						.await
						.map(drop)
				})
			});
		let chair = self
			.hall
			.join(account, reaper, site, entrant, demo, seed()?, Some(settler))?;
		let following = Following {
			chair,
			step: Step::First,
			last_lobby_frame: None,
		};

		Ok(Box::pin(futures::stream::unfold(
			following,
			Following::next,
		)))
	}

	/// Takes in `account`'s report from their seat: what the `report_rift`
	/// operation does.
	pub fn report_rift(&self, account: Id<Account>, report: &RiftReport) -> rpc::Result<()> {
		if report
			.standing
			.is_some_and(|standing| !standing.is_usable_at(now()))
		{
			let refusal = refused(Refusal::LocationUnusable);

			self.hall.fail(account, refusal.clone());
			return Err(refusal);
		}

		self.hall.report(account, report)
	}
}

/// What a signed absorption carries to the chain, and the codex entry it
/// was signed for.
#[derive(Clone)]
struct SignedReceipt {
	receipt: Receipt,
	signature: solana_kit_core::Signature,
	absorption: Absorption,
}

/// The 32 bytes a receipt names its player by: their proved wallet, or,
/// for a guest with none, the SHA-256 of their account.
fn player_bytes(account: Id<Account>, wallet: Option<&str>) -> [u8; 32] {
	wallet
		.and_then(|wallet| wallet.parse::<Address>().ok())
		.map_or_else(
			|| Sha256::digest(account.uuid().to_string().as_bytes()).into(),
			|wallet| *wallet.as_bytes(),
		)
}

/// A fresh, unguessable challenge identifier.
fn challenge_id() -> rpc::Result<String> {
	let mut bytes = [0; 16];

	getrandom::fill(&mut bytes).map_err(|_| rpc::Error::server("no secure random numbers"))?;

	Ok(crate::signer::hex(&bytes))
}

/// `value`, or the refusal `otherwise` names. The identifiers come from the
/// game, so a malformed one is a refusal the player can be told about, not
/// a server error.
fn parsed<T>(value: Option<T>, otherwise: Refusal) -> RuleResult<T> {
	value.ok_or(reapers_rules::RuleBreach { refusal: otherwise })
}

/// The hunt log `encoded` packs, in standard base64.
fn decode_log(encoded: &str) -> rpc::Result<HuntLog> {
	if encoded.len() > MAX_ENCODED_LOG_BYTES {
		return Err(refused(Refusal::HuntUnproven));
	}

	let bytes = STANDARD
		.decode(encoded)
		.map_err(|_| refused(Refusal::HuntUnproven))?;

	HuntLog::decode(&bytes).map_err(|error| {
		if error == HuntLogError::UnsupportedVersion {
			refused(Refusal::HuntVersionUnsupported)
		} else {
			refused(Refusal::HuntUnproven)
		}
	})
}

/// What a reaper takes into a demo circle.
///
/// The starter is the only reaper allowed into a rift until rift reports
/// can be replayed, and the demo keeps that rule. It waives attunement
/// alone, and only a development or test keeper reaches this.
fn demo_entrant(state: &PlayerState) -> RuleResult<RiftEntrant> {
	let Some(reaper) = state.active_reaper() else {
		return Err(Refusal::OathRequired.into());
	};

	if reaper.is_nft() {
		return Err(Refusal::RiftUnproven.into());
	}

	Ok(RiftEntrant {
		name: state.genome().name(),
		loadout: state.loadout(),
	})
}

/// Where a reaper's stream of a rift has got to.
enum Step {
	First,
	Following,
	Done,
}

/// One reaper following their rift: the frames they are sent. Rewards are
/// settled directly by the rift's clock when it seals so a late disconnect
/// never forfeits earned progress.
struct Following {
	chair: Chair,
	step: Step,
	last_lobby_frame: Option<RiftFrame>,
}

impl Following {
	async fn next(mut self) -> Option<(rpc::Result<RiftFrame>, Self)> {
		match self.step {
			Step::Done => None,
			Step::First => {
				self.step = Step::Following;

				let frame = self.frame();
				if let Ok(ref f) = frame
					&& f.phase == RiftPhase::Gathering
					&& !f.counting_down
				{
					self.last_lobby_frame = Some(f.clone());
				}

				Some((frame, self))
			}
			Step::Following => {
				loop {
					// A report that ended the rift is told before the next beat.
					if let Some(failure) = self.chair.failure() {
						self.step = Step::Done;
						return Some((Err(failure), self));
					}

					if !self.chair.beat().await {
						return None;
					}

					let frame = match self.chair.failure().map_or_else(|| self.frame(), Err) {
						Ok(frame) => frame,
						Err(error) => {
							self.step = Step::Done;
							return Some((Err(error), self));
						}
					};

					// Suppress redundant 10 Hz frames while sitting idle in an
					// unchanging Gathering lobby before countdown starts.
					if frame.phase == RiftPhase::Gathering && !frame.counting_down {
						if self.last_lobby_frame.as_ref() == Some(&frame) {
							continue;
						}
						self.last_lobby_frame = Some(frame.clone());
					} else {
						self.last_lobby_frame = None;
					}

					if !frame.phase.is_over() {
						return Some((Ok(frame), self));
					}

					self.step = Step::Done;

					if frame.phase == RiftPhase::Sealed
						&& let Some(error) = self.chair.settlement_error()
					{
						return Some((Err(error), self));
					}

					return Some((Ok(frame), self));
				}
			}
		}
	}

	fn frame(&self) -> rpc::Result<RiftFrame> {
		self.chair
			.frame()
			.ok_or_else(|| rpc::Error::server("the reaper has no seat"))
	}
}

/// The keeper, as the operations' bodies reach it.
#[derive(Clone, Debug)]
pub struct Shared(pub Arc<Keeper>);

impl reapers_api::keeper::Keeper for Shared {
	fn load<'a>(&'a self, cx: &'a rpc::ActionCx) -> BoxFuture<'a, rpc::Result<PlayerState>> {
		Box::pin(async move { self.0.load(me(cx.identity())?).await })
	}

	fn change<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		change: Change,
	) -> BoxFuture<'a, rpc::Result<PlayerState>> {
		Box::pin(self.0.change(cx, change))
	}

	fn apply_action<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		change: Change,
	) -> BoxFuture<'a, rpc::Result<PlayerState>> {
		Box::pin(async move { self.0.apply_action(me(cx.identity())?, change).await })
	}

	fn settle_hunt<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		sighting: i64,
		log: String,
	) -> BoxFuture<'a, rpc::Result<SettledHunt>> {
		Box::pin(async move { self.0.settle_hunt(me(cx.identity())?, sighting, &log).await })
	}

	fn absorb_transaction<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		nft: String,
	) -> BoxFuture<'a, rpc::Result<ChainReceipt>> {
		Box::pin(async move { self.0.absorb_transaction(me(cx.identity())?, &nft).await })
	}

	fn update_location<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		fix: Option<GeoFix>,
	) -> BoxFuture<'a, rpc::Result<()>> {
		Box::pin(async move { books::locate(&self.0.db, me(cx.identity())?, fix, now()).await })
	}

	fn location<'a>(&'a self, cx: &'a rpc::QueryCx) -> BoxFuture<'a, rpc::Result<Option<GeoFix>>> {
		Box::pin(async move { books::location(&self.0.db, me(cx.identity())?, now()).await })
	}

	fn receipt_key(&self) -> KeeperKey {
		self.0.signer.key()
	}

	fn chain_status<'a>(&'a self, cx: &'a rpc::ActionCx) -> BoxFuture<'a, rpc::Result<Chain>> {
		Box::pin(async move { self.0.chain_status(me(cx.identity())?).await })
	}

	fn wallet_challenge<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		address: String,
	) -> BoxFuture<'a, rpc::Result<Challenge>> {
		Box::pin(async move { self.0.wallet_challenge(cx, &address).await })
	}

	fn enter_rift<'a>(
		&'a self,
		cx: &'a rpc::QueryCx,
		site: String,
		demo: bool,
	) -> BoxFuture<'a, rpc::Result<BoxStream<'static, rpc::Result<RiftFrame>>>> {
		Box::pin(async move {
			let account = me(cx.identity())?;

			Arc::clone(&self.0).enter_rift(account, &site, demo).await
		})
	}

	fn report_rift<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		report: RiftReport,
	) -> BoxFuture<'a, rpc::Result<()>> {
		Box::pin(async move { self.0.report_rift(me(cx.identity())?, &report) })
	}
}

/// An account's books, opened, for the sign-in routes and tests.
pub async fn opened(db: &Db, account: Id<Account>) -> rpc::Result<PlayerState> {
	Ok(books::open(db, account).await?.state.0)
}
