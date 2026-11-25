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

use std::collections::HashMap;
use std::time::Duration;

use leptoskit_auth::Account;
use leptoskit_db::Connection;
use leptoskit_db::Db;
use leptoskit_db::DbError;
use leptoskit_db::Id;
use leptoskit_db::Json;
use leptoskit_db::Read;
use leptoskit_db::Statement;
use leptoskit_db::Value;
use leptoskit_rpc as rpc;
use reapers_api::refused;
use reapers_rules::GeoFix;
use reapers_rules::PlayerState;
use reapers_rules::ReaperState;
use reapers_rules::Refusal;
use reapers_rules::RiftRules;
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

/// Inserts initial books for a newly created account without a prior SELECT.
pub async fn create_initial(
	tx: &impl Connection,
	account: Id<Account>,
) -> rpc::Result<PlayerBooks> {
	NewPlayerBooks {
		account_id: account,
		state: Json(PlayerState::initial()),
		location: None,
	}
	.insert(tx)
	.await
	.map_err(conflict_as_contended)
}

/// Returns whether `account` already has a `player_books` row without fetching
/// or parsing its JSONB `state`.
pub async fn exists(tx: &impl leptoskit_db::Read, account: Id<Account>) -> rpc::Result<bool> {
	let rows = tx
		.fetch(Statement {
			sql: "SELECT 1 FROM \"player_books\" WHERE \"account_id\" = $1 LIMIT 1".to_owned(),
			args: vec![Value::Uuid(account.uuid())],
		})
		.await?;

	Ok(!rows.is_empty())
}

/// Fast-path loaded `player_books` row with its raw JSONB snapshot for
/// optimistic compare-and-swap updates without `SERIALIZABLE` overhead.
pub struct FastBooks {
	pub id: Value,
	pub raw_state: serde_json::Value,
	pub state: PlayerState,
}

/// Minimal escrow binding fields needed to verify and update a hunted escrow.
#[derive(Clone, Debug)]
pub struct BoundEscrow {
	pub id: Value,
	pub wallet: String,
	pub mint: String,
}

/// Single-query prefetch of `player_books`, `wallet_identities`, and any
/// matching `husk_escrows` row for `settle_hunt`.
pub struct HuntPrefetch {
	pub book_id: Value,
	pub raw_state: serde_json::Value,
	pub state: PlayerState,
	pub wallet: Option<String>,
	pub bound: Option<BoundEscrow>,
}

/// Fast-path `READ COMMITTED` read of `account`'s books returning the raw JSONB
/// snapshot alongside the parsed state when no NFT reapers are present.
pub async fn load_fast_raw(db: &Db, account: Id<Account>) -> rpc::Result<Option<FastBooks>> {
	let rows = db
		.fetch(Statement {
			sql: "SELECT \"id\", \"state\" FROM \"player_books\" WHERE \"account_id\" = $1 LIMIT 1"
				.to_owned(),
			args: vec![Value::Uuid(account.uuid())],
		})
		.await?;

	let Some(row) = rows.into_iter().next() else {
		return Ok(None);
	};
	let mut cols = row.into_iter();
	let Some(id @ Value::Uuid(_)) = cols.next() else {
		return Err(rpc::Error::server("player_books.id was not a uuid"));
	};
	let Some(Value::Json(raw_state)) = cols.next() else {
		return Err(rpc::Error::server("player_books.state was not json"));
	};
	let state: PlayerState = serde_json::from_value(raw_state.clone())
		.map_err(|error| rpc::Error::server(format!("invalid player_books.state: {error}")))?;

	if state.reapers().iter().any(ReaperState::is_nft) {
		return Ok(None);
	}

	Ok(Some(FastBooks {
		id,
		raw_state,
		state,
	}))
}

/// Fast-path `READ COMMITTED` read of `account`'s books when the row already
/// exists and carries no NFT reapers requiring chain reconciliation.
pub async fn load_fast(
	db: &impl leptoskit_db::Read,
	account: Id<Account>,
) -> rpc::Result<Option<PlayerState>> {
	let rows = db
		.fetch(Statement {
			sql: "SELECT \"state\" FROM \"player_books\" WHERE \"account_id\" = $1 LIMIT 1"
				.to_owned(),
			args: vec![Value::Uuid(account.uuid())],
		})
		.await?;

	let Some(Value::Json(raw_state)) = rows.into_iter().next().and_then(|r| r.into_iter().next())
	else {
		return Ok(None);
	};
	let state: PlayerState = serde_json::from_value(raw_state)
		.map_err(|error| rpc::Error::server(format!("invalid player_books.state: {error}")))?;

	if state.reapers().iter().any(ReaperState::is_nft) {
		return Ok(None);
	}

	Ok(Some(state))
}

/// Fetches `player_books`, linked wallet, and matching `husk_escrows` row for
/// `(account, sighting)` in a single SQL round-trip.
pub async fn prefetch_hunt(
	db: &Db,
	account: Id<Account>,
	sighting: i64,
) -> rpc::Result<Option<HuntPrefetch>> {
	let rows = db
		.fetch(Statement {
			sql: "SELECT b.\"id\", b.\"state\", w.\"address\", e.\"id\", e.\"wallet\", e.\"mint\" \
			      FROM \"player_books\" b \
			      LEFT JOIN \"wallet_identities\" w ON w.\"account_id\" = b.\"account_id\" \
			      LEFT JOIN \"husk_escrows\" e \
			        ON e.\"account_id\" = b.\"account_id\" AND e.\"sighting_id\" = $2 \
			      WHERE b.\"account_id\" = $1 \
			      LIMIT 1"
				.to_owned(),
			args: vec![Value::Uuid(account.uuid()), Value::Int(sighting)],
		})
		.await?;

	let Some(row) = rows.into_iter().next() else {
		return Ok(None);
	};
	let mut cols = row.into_iter();
	let Some(book_id @ Value::Uuid(_)) = cols.next() else {
		return Err(rpc::Error::server("player_books.id was not a uuid"));
	};
	let Some(Value::Json(raw_state)) = cols.next() else {
		return Err(rpc::Error::server("player_books.state was not json"));
	};
	let wallet = match cols.next() {
		Some(Value::Text(address)) => Some(address),
		_ => None,
	};
	let bound = match (cols.next(), cols.next(), cols.next()) {
		(Some(id @ Value::Uuid(_)), Some(Value::Text(escrow_wallet)), Some(Value::Text(mint))) => {
			Some(BoundEscrow {
				id,
				wallet: escrow_wallet,
				mint,
			})
		}
		_ => None,
	};
	let state: PlayerState = serde_json::from_value(raw_state.clone())
		.map_err(|error| rpc::Error::server(format!("invalid player_books.state: {error}")))?;

	Ok(Some(HuntPrefetch {
		book_id,
		raw_state,
		state,
		wallet,
		bound,
	}))
}

/// Atomically updates `player_books.state` only if it still equals `expected`.
/// Returns `true` if the row was updated, or `false` if a concurrent change
/// modified the row first.
pub async fn compare_and_swap_state(
	conn: &impl Connection,
	book_id: &Value,
	expected: &serde_json::Value,
	next: &PlayerState,
) -> rpc::Result<bool> {
	let next_json = serde_json::to_value(next)
		.map_err(|error| rpc::Error::server(format!("serialize player_books.state: {error}")))?;
	let updated = conn
		.execute(Statement {
			sql: "UPDATE \"player_books\" SET \"state\" = $1, \"updated_at\" = $2 \
			      WHERE \"id\" = $3 AND \"state\" = $4"
				.to_owned(),
			args: vec![
				Value::Json(next_json),
				Value::Timestamp(leptoskit_db::now()),
				book_id.clone(),
				Value::Json(expected.clone()),
			],
		})
		.await?;

	Ok(updated == 1)
}

/// Records the absorbed husk and signed absorption entry on a `husk_escrows` row.
pub async fn record_escrow_absorption(
	conn: &impl Connection,
	escrow_id: &Value,
	absorption: &reapers_rules::Absorption,
) -> rpc::Result<()> {
	let absorption_json = serde_json::to_value(absorption)
		.map_err(|error| rpc::Error::server(format!("serialize absorption: {error}")))?;
	conn.execute(Statement {
		sql: "UPDATE \"husk_escrows\" SET \"nft\" = $1, \"absorption\" = $2 WHERE \"id\" = $3"
			.to_owned(),
		args: vec![
			Value::Text(absorption.nft.id()),
			Value::Json(absorption_json),
			escrow_id.clone(),
		],
	})
	.await?;

	Ok(())
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

	create_initial(tx, account).await
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
/// Updates only the `location` column without fetching or deserializing the
/// `state` JSONB blob, so concurrent purchases and hunts touching `state` are
/// never overwritten. A reading that does not [`GeoFix::replaces`] the one
/// kept is ignored: a delayed upload never moves a player back, and a coarser
/// copy of the same instant does not replace a tighter one. Nor is one
/// observed less than [`RiftRules::LOCATION_REPORT_INTERVAL`] after it: the
/// books keep at most one reading every interval.
pub async fn locate(
	db: &Db,
	account: Id<Account>,
	fix: Option<GeoFix>,
	now: Timestamp,
) -> rpc::Result<()> {
	if fix.is_some_and(|fix| !fix.is_usable_at(now)) {
		return Err(refused(Refusal::LocationUnusable));
	}

	let Some(fix) = fix else {
		let updated = db
			.execute(Statement {
				sql: "UPDATE \"player_books\" SET \"location\" = NULL WHERE \"account_id\" = $1"
					.to_owned(),
				args: vec![Value::Uuid(account.uuid())],
			})
			.await?;
		if updated == 0 {
			transact_serializable!(db, |tx| open(tx, account).await.map(drop))?;
		}
		return Ok(());
	};

	let next_json = serde_json::to_value(fix)
		.map_err(|error| rpc::Error::server(format!("serialize location: {error}")))?;

	// Optimistic compare-and-swap on the small `location` column alone avoids
	// both SERIALIZABLE transaction overhead and reading the large `state`
	// JSONB blob, while preserving strict `GeoFix::replaces` ordering under
	// concurrent uploads.
	for _ in 0..4 {
		let rows = db
			.fetch(Statement {
				sql: "SELECT \"id\", \"location\" FROM \"player_books\" WHERE \"account_id\" = $1 LIMIT 1"
					.to_owned(),
				args: vec![Value::Uuid(account.uuid())],
			})
			.await?;

		let Some(row) = rows.into_iter().next() else {
			return transact_serializable!(db, |tx| {
				async {
					let books = open(tx, account).await?;
					if let Some(Json(before)) = &books.location
						&& !keeps(&fix, before)
					{
						return Ok(());
					}
					PlayerBooks::update()
						.filter(|row| row.id.eq(books.id))
						.set(|row| row.location, Some(Json(fix)))
						.run(tx)
						.await?;
					Ok(())
				}
				.await
			});
		};

		let mut cols = row.into_iter();
		let Some(Value::Uuid(book_id)) = cols.next() else {
			return Err(rpc::Error::server("player_books.id was not a uuid"));
		};
		let before_value = cols.next().unwrap_or(Value::Null);
		if let Value::Json(ref raw) = before_value
			&& let Ok(before) = serde_json::from_value::<GeoFix>(raw.clone())
			&& !keeps(&fix, &before)
		{
			return Ok(());
		}

		let updated = db
			.execute(Statement {
				sql: "UPDATE \"player_books\" SET \"location\" = $1 \
				      WHERE \"id\" = $2 AND \"location\" IS NOT DISTINCT FROM $3"
					.to_owned(),
				args: vec![
					Value::Json(next_json.clone()),
					Value::Uuid(book_id),
					before_value,
				],
			})
			.await?;

		if updated == 1 {
			return Ok(());
		}
	}

	// Fall back to a serializable transaction if concurrent updates contended
	// repeatedly.
	transact_serializable!(db, |tx| {
		async {
			let books = open(tx, account).await?;
			if let Some(Json(before)) = &books.location
				&& !keeps(&fix, before)
			{
				return Ok(());
			}
			PlayerBooks::update()
				.filter(|row| row.id.eq(books.id))
				.set(|row| row.location, Some(Json(fix)))
				.run(tx)
				.await?;
			Ok(())
		}
		.await
	})
}

/// Whether the books keep `fix` over `before`: it [`GeoFix::replaces`] it,
/// and either tightens the same instant or was observed at least
/// [`RiftRules::LOCATION_REPORT_INTERVAL`] after it.
fn keeps(fix: &GeoFix, before: &GeoFix) -> bool {
	let interval =
		i64::try_from(RiftRules::LOCATION_REPORT_INTERVAL.as_micros()).unwrap_or(i64::MAX);

	fix.replaces(before)
		&& (fix.observed_at == before.observed_at
			|| fix.observed_at.micros_since(before.observed_at) >= interval)
}

/// `account`'s latest location, while it is still fresh enough at `now` to
/// be one. Projects only `location` to skip fetching and parsing `state`.
pub async fn location(
	db: &impl leptoskit_db::Read,
	account: Id<Account>,
	now: Timestamp,
) -> rpc::Result<Option<GeoFix>> {
	let rows = db
		.fetch(Statement {
			sql: "SELECT \"location\" FROM \"player_books\" WHERE \"account_id\" = $1 LIMIT 1"
				.to_owned(),
			args: vec![Value::Uuid(account.uuid())],
		})
		.await?;

	let Some(Value::Json(raw)) = rows
		.into_iter()
		.next()
		.and_then(|row| row.into_iter().next())
	else {
		return Ok(None);
	};
	let fix: GeoFix = serde_json::from_value(raw)
		.map_err(|error| rpc::Error::server(format!("invalid stored location: {error}")))?;

	Ok(fix.is_usable_at(now).then_some(fix))
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

	let keys: Vec<String> = assets
		.iter()
		.filter(|address| owned.contains_key(*address))
		.map(|address| asset_key(gateway, address))
		.collect();
	let mut existing: HashMap<String, ReaperRecord> = if keys.is_empty() {
		HashMap::new()
	} else {
		ReaperRecord::query()
			.filter(|record| record.asset_key.in_list(keys.clone()))
			.all(tx)
			.await?
			.into_iter()
			.map(|record| (record.asset_key.clone(), record))
			.collect()
	};

	let mut roster = Vec::with_capacity(assets.len());

	for address in assets {
		let Some(genome) = owned.get(&address) else {
			continue;
		};
		let key = asset_key(gateway, &address);
		let Some(record) = existing.remove(&key) else {
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
