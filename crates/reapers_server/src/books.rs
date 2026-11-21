//! The one way a player's stored state is read or changed, and the ledger
//! of NFT reapers' growth beside it.
//!
//! Every change runs in one serializable transaction: the operation's own,
//! or one the keeper opens for a rift. Two requests from the same player
//! that would both spend the same anima cannot both commit; the second runs
//! again on the books the first left. Before any change, the player's NFT
//! reapers are checked against the chain and their canonical growth is read
//! from the ledger in the same transaction, so growth and books commit
//! together or not at all.

use std::time::Duration;

use leptoskit_auth::Account;
use leptoskit_db::Connection;
use leptoskit_db::DbError;
use leptoskit_db::Id;
use leptoskit_db::Json;
use leptoskit_rpc as rpc;
use reapers_api::refused;
use reapers_rules::GeoFix;
use reapers_rules::PlayerState;
use reapers_rules::ReaperState;
use reapers_rules::Refusal;
use reapers_rules::RuleResult;
use reapers_rules::Timestamp;

use crate::chain::Gateway;
use crate::tables::HuskEscrow;
use crate::tables::NewPlayerBooks;
use crate::tables::NewReaperRecord;
use crate::tables::PlayerBooks;
use crate::tables::ReaperRecord;
use crate::tables::WalletIdentity;
use crate::wallet;

/// How many times a serializable transaction the keeper opens itself retries
/// when another transaction contends with it.
pub const SERIALIZABLE_RETRIES: u32 = 5;

/// Sleeps with bounded jittered exponential backoff before retrying a
/// contended serializable transaction.
pub async fn retry_backoff(attempt: u32) {
	let base_ms = 8_u64.saturating_mul(1_u64 << attempt.min(5));
	let jitter_ms = u64::from(getrandom::u32().unwrap_or(0) % 17);
	tokio::time::sleep(Duration::from_millis(base_ms + jitter_ms)).await;
}

/// Runs `$work` (with `$tx: &Tx`) inside a `SERIALIZABLE` transaction on `$db`,
/// retrying up to [`crate::books::SERIALIZABLE_RETRIES`] times with jittered
/// exponential backoff whenever PostgreSQL reports contention.
macro_rules! transact_serializable {
	($db:expr, | $tx:ident | $work:expr) => {{
		let mut attempt: u32 = 0;
		loop {
			let tx = $db.begin_serializable().await?;
			let result = {
				let $tx = &tx;
				$work
			};
			let result = match result {
				Ok(value) => {
					tx.commit()
						.await
						.map(|()| value)
						.map_err(leptoskit_rpc::Error::from)
				}
				Err(error) => {
					let _ = tx.rollback().await;
					Err(error)
				}
			};
			match result {
				Err(error)
					if error.is_contended() && attempt < $crate::books::SERIALIZABLE_RETRIES =>
				{
					attempt += 1;
					$crate::books::retry_backoff(attempt).await;
				}
				result => break result,
			}
		}
	}};
}

pub(crate) use transact_serializable;

/// Maps a PostgreSQL unique-constraint violation (`DbError::Conflict`) to
/// [`rpc::Error::Contended`] so serializable retry loops cleanly retry races.
pub fn conflict_as_contended(error: DbError) -> rpc::Error {
	if matches!(error, DbError::Conflict { .. }) {
		rpc::Error::Contended
	} else {
		rpc::Error::from(error)
	}
}

/// The rules' refusal, as an operation fails with it.
pub fn breach<T>(result: RuleResult<T>) -> rpc::Result<T> {
	result.map_err(|breach| refused(breach.refusal))
}

/// `account`'s books, opened with the initial state if this is the first
/// the keeper has heard of them.
pub async fn open(tx: &impl Connection, account: Id<Account>) -> rpc::Result<PlayerBooks> {
	let found = PlayerBooks::query()
		.filter(|books| books.account_id.eq(account))
		.first(tx)
		.await?;

	if let Some(books) = found {
		return Ok(books);
	}

	NewPlayerBooks {
		account_id: account,
		state: Json(PlayerState::initial()),
		location: None,
	}
	.insert(tx)
	.await
	.map_err(conflict_as_contended)
}

/// Opened and chain-verified player books ready for a rule transition.
#[derive(Clone, Debug)]
pub struct Prepared {
	id: Id<PlayerBooks>,
	stored: PlayerState,
	/// The player state after chain verification and reconciliation.
	pub state: PlayerState,
}

/// Opens `account`'s books, verifies any NFT reapers against the chain, and
/// cleans up unabsorbed escrow rows for any unsettled sightings dropped during
/// roster reconciliation.
pub async fn prepare(
	tx: &impl Connection,
	gateway: &Gateway,
	account: Id<Account>,
	import: Option<&str>,
) -> rpc::Result<Prepared> {
	let books = open(tx, account).await?;
	let verified = refresh(tx, gateway, account, &books.state.0, import).await?;

	// If losing an active NFT reaper returned unsettled sightings back to the
	// vault during reconciliation, remove their unabsorbed escrow bindings so
	// the player's on-chain mint is not permanently locked with `AlreadyHeld`.
	for before_sighting in books.state.0.sightings() {
		if !verified
			.sightings()
			.iter()
			.any(|after| after.id == before_sighting.id)
		{
			HuskEscrow::query()
				.filter(|escrow| {
					escrow.account_id.eq(account)
						& escrow.sighting_id.eq(before_sighting.id)
						& escrow.absorption.is_null()
				})
				.delete(tx)
				.await?;
		}
	}

	Ok(Prepared {
		id: books.id,
		stored: books.state.0,
		state: verified,
	})
}

/// Persists `next` after [`prepare`], updating only changed NFT reaper ledger
/// records and `player_books` when the state actually changed.
pub async fn commit(
	tx: &impl Connection,
	gateway: &Gateway,
	prepared: &Prepared,
	next: PlayerState,
) -> rpc::Result<PlayerState> {
	if next == prepared.stored {
		return Ok(next);
	}

	save(tx, gateway, &prepared.state, &next).await?;
	PlayerBooks::update()
		.filter(|row| row.id.eq(prepared.id))
		.set(|row| row.state, Json(next.clone()))
		.set(|row| row.updated_at, leptoskit_db::now())
		.run(tx)
		.await?;

	Ok(next)
}

/// Applies `rule` to `account`'s verified state and stores what it
/// returns, with the NFT reapers' growth saved to the ledger beside it.
///
/// `import` is an asset to add to the roster first: it must be owned by
/// the player's proved wallet, and starts at level one unless the ledger
/// already knows it.
pub async fn change(
	tx: &impl Connection,
	gateway: &Gateway,
	account: Id<Account>,
	import: Option<&str>,
	rule: impl FnOnce(PlayerState) -> rpc::Result<PlayerState>,
) -> rpc::Result<PlayerState> {
	let prepared = prepare(tx, gateway, account, import).await?;
	let next = rule(prepared.state.clone())?;

	commit(tx, gateway, &prepared, next).await
}

/// Keeps `fix` as `account`'s latest location, or clears it.
///
/// It writes the same row as every other change, so a location update and
/// a purchase arriving together cannot overwrite each other: whichever
/// commits second runs again on what the first left. A reading that does
/// not [`GeoFix::replaces`] the one kept is ignored: a delayed upload never
/// moves a player back, and a coarser copy of the same instant does not
/// replace a tighter one.
pub async fn locate(
	tx: &impl Connection,
	account: Id<Account>,
	fix: Option<GeoFix>,
	now: Timestamp,
) -> rpc::Result<()> {
	if fix.is_some_and(|fix| !fix.is_usable_at(now)) {
		return Err(refused(Refusal::LocationUnusable));
	}

	let books = open(tx, account).await?;

	if let (Some(fix), Some(Json(before))) = (fix, &books.location)
		&& !fix.replaces(before)
	{
		return Ok(());
	}

	PlayerBooks::update()
		.filter(|row| row.id.eq(books.id))
		.set(|row| row.location, fix.map(Json))
		.run(tx)
		.await?;

	Ok(())
}

/// `account`'s latest location, while it is still fresh enough at `now` to
/// be one.
pub async fn location(
	db: &impl leptoskit_db::Read,
	account: Id<Account>,
	now: Timestamp,
) -> rpc::Result<Option<GeoFix>> {
	let books = PlayerBooks::query()
		.filter(|books| books.account_id.eq(account))
		.first(db)
		.await?;

	Ok(books
		.and_then(|books| books.location)
		.map(|Json(fix)| fix)
		.filter(|fix| fix.is_usable_at(now)))
}

/// The wallet `account` has proved, if any.
pub async fn wallet_of(
	tx: &impl leptoskit_db::Read,
	account: Id<Account>,
) -> rpc::Result<Option<String>> {
	Ok(WalletIdentity::query()
		.filter(|identity| identity.account_id.eq(account))
		.first(tx)
		.await?
		.map(|identity| identity.address))
}

/// `state` with its NFT reapers checked against the chain and their growth
/// read from the ledger.
///
/// Only a verified first import creates growth, at level one. The player's
/// roster, a pasted genome or an offline experience count is never trusted.
/// A reaper the wallet no longer owns leaves the roster and keeps its
/// growth in the ledger for its next owner. Offline saves never enter the
/// ledger. With no NFT reapers and no import, the chain is not read.
async fn refresh(
	tx: &impl Connection,
	gateway: &Gateway,
	account: Id<Account>,
	state: &PlayerState,
	import: Option<&str>,
) -> rpc::Result<PlayerState> {
	let mut assets: Vec<String> = state
		.reapers()
		.iter()
		.filter(|reaper| reaper.is_nft())
		.map(|reaper| reaper.id().to_owned())
		.collect();

	if let Some(import) = import {
		if !wallet::is_address(import) {
			return Err(refused(Refusal::NotOwned));
		}

		if assets.iter().any(|asset| asset == import) {
			return Err(refused(Refusal::AlreadyOwned));
		}

		if assets.len() >= PlayerState::MAX_NFT_REAPERS {
			return Err(refused(Refusal::RosterFull));
		}

		assets.push(import.to_owned());
	}

	if assets.is_empty() {
		return Ok(state.clone());
	}

	let Some(wallet) = wallet_of(tx, account).await? else {
		return Err(refused(Refusal::WalletUnproven));
	};
	let owned = gateway.owned(&wallet, &assets).await.map_err(refused)?;

	if import.is_some_and(|import| !owned.contains_key(import)) {
		return Err(refused(Refusal::NotOwned));
	}

	let Some(collection) = gateway.config().collection else {
		return Err(refused(Refusal::ChainUnconfigured));
	};
	let collection = collection.to_string();

	// A stable order, so two owners refreshing overlapping rosters during a
	// transfer read the same records in the same order.
	assets.sort();

	let mut roster = Vec::new();

	for address in assets {
		let Some(genome) = owned.get(&address) else {
			continue;
		};
		let key = asset_key(gateway, &address);
		let record = ReaperRecord::query()
			.filter(|record| record.asset_key.eq(key.clone()))
			.first(tx)
			.await?;

		let Some(record) = record else {
			let reaper = ReaperState::bound(address, *genome);

			NewReaperRecord {
				asset_key: key,
				collection: collection.clone(),
				state: Json(reaper.clone()),
			}
			.insert(tx)
			.await
			.map_err(conflict_as_contended)?;
			roster.push(reaper);
			continue;
		};

		if record.collection != collection || record.state.0.genome() != genome {
			// Fail explicitly when the player is importing this asset; for an
			// existing roster entry whose metadata was mutated on-chain, drop
			// only that asset instead of locking the player out of their books.
			if import == Some(address.as_str()) {
				return Err(refused(Refusal::NotOwned));
			}
			continue;
		}

		roster.push(record.state.0);
	}

	breach(state.reconcile_reapers(roster))
}

/// Writes NFT reapers whose growth or equipment changed in `next` (relative to
/// `before`) back to the ledger.
async fn save(
	tx: &impl Connection,
	gateway: &Gateway,
	before: &PlayerState,
	next: &PlayerState,
) -> rpc::Result<()> {
	for reaper in next.reapers().iter().filter(|reaper| reaper.is_nft()) {
		let unchanged = before
			.reapers()
			.iter()
			.any(|previous| previous.id() == reaper.id() && previous == reaper);

		if unchanged {
			continue;
		}

		let key = asset_key(gateway, reaper.id());
		let updated = ReaperRecord::update()
			.filter(|record| record.asset_key.eq(key))
			.set(|record| record.state, Json(reaper.clone()))
			.run(tx)
			.await?;

		if updated != 1 {
			return Err(rpc::Error::server(
				"an NFT reaper has no verified ledger record",
			));
		}
	}

	Ok(())
}

/// The ledger's key for the asset at `address` on the keeper's chain.
pub fn asset_key(gateway: &Gateway, address: &str) -> String {
	format!("{}:{address}", gateway.config().chain)
}
