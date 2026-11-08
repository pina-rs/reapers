//! The Reapers keeper's tables, on `leptoskit_db`, and their migrations.
//!
//! None has a policy: only the keeper's own code reads or writes them, as
//! the signed-in player, and no table is live or public, so no row is ever
//! sent to a client except through an operation's answer. Sign-in keeps its
//! own tables (accounts, sessions) beside these, under `auth`, and the
//! operations keep their receipts under `rpc`.
//!
//! After changing a table, run `cargo run -p reapers_db --bin
//! reapers-migration -- <name>` and commit what it writes under
//! `migrations/`.

pub mod migrations;

use leptoskit_auth::Account;
use leptoskit_db::Id;
use leptoskit_db::Json;
use leptoskit_db::Timestamp;
use leptoskit_db::table;
use reapers_rules::GeoFix;
use reapers_rules::PlayerState;
use reapers_rules::ReaperState;

leptoskit_db::packs!(leptoskit_auth);
leptoskit_db::packs!(leptoskit_rpc);

/// One player's books: everything they own and have done.
///
/// The state is the shared `PlayerState`, stored whole, in its save format.
/// The keeper never edits it field by field: every change is one of the
/// shared rules applied to the whole state, exactly as the offline game
/// applies it.
#[table(name = "player_books")]
pub struct PlayerBooks {
	pub id: Id<Self>,
	/// The account these books belong to.
	#[db(unique, on_delete = cascade)]
	pub account_id: Id<Account>,
	/// The player as they stand now.
	pub state: Json<PlayerState>,
	/// The latest foreground fix only; never a history. Null when tracking
	/// stops.
	pub location: Option<Json<GeoFix>>,
	/// When the books were opened.
	#[db(created_at)]
	pub created_at: Timestamp,
	/// When they last changed.
	#[db(updated_at)]
	pub updated_at: Timestamp,
}

/// A wallet proved to belong to an account. A wallet is linked to one
/// account, and an account to one wallet.
#[table(name = "wallet_identities")]
pub struct WalletIdentity {
	pub id: Id<Self>,
	/// The wallet's address.
	#[db(unique, max_len = 44)]
	pub address: String,
	#[db(unique, on_delete = cascade)]
	pub account_id: Id<Account>,
	#[db(created_at)]
	pub created_at: Timestamp,
}

/// The one open challenge of an account: a message for a wallet to sign.
#[table(name = "wallet_challenges")]
pub struct WalletChallenge {
	pub id: Id<Self>,
	/// What the game names the challenge by, and the message's nonce.
	#[db(unique, max_len = 64)]
	pub challenge_id: String,
	#[db(unique, on_delete = cascade)]
	pub account_id: Id<Account>,
	/// The wallet that must sign it.
	#[db(max_len = 44)]
	pub address: String,
	/// The exact text it must sign.
	#[db(max_len = 2000)]
	pub message: String,
	pub expires_at: Timestamp,
	/// Whether a signature has already been accepted for it.
	#[db(default = false)]
	pub consumed: bool,
}

/// One canonical growth record per NFT reaper, shared by every owner it
/// passes to.
#[table(name = "reaper_records")]
pub struct ReaperRecord {
	pub id: Id<Self>,
	/// `<chain>:<asset address>`.
	#[db(unique, max_len = 100)]
	pub asset_key: String,
	/// The collection the asset was verified in when first imported.
	#[db(max_len = 44)]
	pub collection: String,
	/// The reaper's growth: experience, equipment, codex and memory.
	pub state: Json<ReaperState>,
}

/// A loose husk that was escrowed on-chain when it was released: which mint
/// it is, and which wallet released it.
///
/// A mint is bound to one husk at most, so one escrow cannot earn two
/// receipts.
#[table(name = "husk_escrows", unique(account_id, sighting_id))]
pub struct HuskEscrow {
	pub id: Id<Self>,
	#[db(on_delete = cascade)]
	pub account_id: Id<Account>,
	/// The loose husk's sighting in the player's books.
	pub sighting_id: i64,
	/// `<chain>:<mint address>`.
	#[db(unique, max_len = 100)]
	pub mint_key: String,
	/// The husk's mint.
	#[db(max_len = 44)]
	pub mint: String,
	/// The wallet that released it.
	#[db(max_len = 44)]
	pub wallet: String,
	#[db(created_at)]
	pub created_at: Timestamp,
}

/// Every migration, in order, embedded in the keeper, which applies the
/// pending ones when it starts.
pub static MIGRATIONS: &[leptoskit_db::Migration] = leptoskit_db::migrations!("migrations");
