//! Everything a player can ask of their own books.
//!
//! Each change is one shared rule applied to the stored state, inside the
//! mutation's transaction. Anything the keeper must decide (a seed, the
//! time, how well a reaper fed) is decided by the keeper, never sent by the
//! game.

use leptoskit_rpc as rpc;
use reapers_rules::GeoFix;
use reapers_rules::HuntSettlement;
use reapers_rules::PlayerState;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::Shared;
#[cfg(feature = "server")]
use crate::keeper::Change;
#[cfg(feature = "server")]
use crate::keeper::keeper;

/// The player as they stand now, with their books opened on first use.
#[rpc::mutation]
pub async fn load(cx: &rpc::MutationCx) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?.change(cx, Change::Load).await
}

/// Shares only this player's latest foreground location, or clears it with
/// `None`. A reading older than the one kept is ignored; a coarse, stale or
/// future-dated one is refused as `locationUnusable`.
#[rpc::mutation]
pub async fn update_location(cx: &rpc::MutationCx, fix: Option<Shared<GeoFix>>) -> rpc::Result<()> {
	keeper(cx.identity())?
		.update_location(cx, fix.map(|fix| fix.0))
		.await
}

/// This player's latest location, while it is still fresh enough to be one.
#[rpc::query]
pub async fn location(cx: &rpc::QueryCx) -> rpc::Result<Option<GeoFix>> {
	keeper(cx.identity())?.location(cx).await
}

/// Binds a reaper drawn from a seed only the keeper knows.
#[rpc::mutation]
pub async fn take_oath(cx: &rpc::MutationCx) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?.change(cx, Change::TakeOath).await
}

/// Makes the reaper `id` names the active one: the oathbound starter, or an
/// NFT reaper by its asset address.
#[rpc::mutation]
pub async fn select_reaper(cx: &rpc::MutationCx, id: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::SelectReaper(id))
		.await
}

/// Buys the stand-in NFT `nft` names (`degenApes:1`) into the vault. Only a
/// development or test keeper sells them.
#[rpc::mutation]
pub async fn acquire(cx: &rpc::MutationCx, nft: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::Acquire(nft))
		.await
}

/// Releases the NFT `nft` names from the vault into the veil.
///
/// `escrow` is the husk's real mint when the player has already escrowed it
/// with the Reapers program. The keeper checks the escrow on-chain before
/// the release counts, and binds that mint to the loose husk, so absorbing
/// it later earns a receipt the program will burn it with.
#[rpc::mutation]
pub async fn release(
	cx: &rpc::MutationCx,
	nft: String,
	#[serde(default)] escrow: Option<String>,
) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::Release { nft, escrow })
		.await
}

/// Replays the recorded hunt for the loose husk `sighting` and settles what
/// it earned.
///
/// `log` is the packed `HuntLog`, in standard base64: see [`encode_log`].
/// The verdict is the replay's; nothing the game claims about how the hunt
/// went is read.
#[rpc::mutation]
pub async fn settle_hunt(
	cx: &rpc::MutationCx,
	sighting: i64,
	log: String,
) -> rpc::Result<SettledHunt> {
	keeper(cx.identity())?.settle_hunt(cx, sighting, log).await
}

/// The burn of the absorbed husk `nft` names (`degenApes:1`) again, for
/// when the wallet's first one did not land: the same receipt the hunt was
/// settled with, its signature the one in the codex, in a transaction with
/// a fresh blockhash.
///
/// Only for a husk this player released from escrow and a hunt absorbed,
/// while the escrow still holds it, checked at a finalized slot. A husk
/// whose escrow is gone, burned or never there, is refused as
/// `notInVault`. The keeper signs nothing the settlement did not: an action
/// rather than a mutation, since nothing in the books changes.
#[rpc::action]
pub async fn absorb_transaction(cx: &rpc::ActionCx, nft: String) -> rpc::Result<ChainReceipt> {
	keeper(cx.identity())?.absorb_transaction(cx, nft).await
}

/// The key absorption receipts are signed with.
#[rpc::query]
pub async fn receipt_key(cx: &rpc::QueryCx) -> rpc::Result<KeeperKey> {
	Ok(keeper(cx.identity())?.receipt_key())
}

/// Buys the weapon `weapon` names (`scythe:sworn`) with anima.
#[rpc::mutation]
pub async fn buy_weapon(cx: &rpc::MutationCx, weapon: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::BuyWeapon(weapon))
		.await
}

/// Takes the bought weapon `weapon` names in hand.
#[rpc::mutation]
pub async fn wield(cx: &rpc::MutationCx, weapon: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::Wield(weapon))
		.await
}

/// Buys the outfit `outfit` names with anima.
#[rpc::mutation]
pub async fn buy_outfit(cx: &rpc::MutationCx, outfit: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::BuyOutfit(outfit))
		.await
}

/// Puts on the bought outfit `outfit` names.
#[rpc::mutation]
pub async fn wear(cx: &rpc::MutationCx, outfit: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::Wear(outfit))
		.await
}

/// Makes the art `art` names the one the reaper throws. Only an art the
/// reaper has learned can be.
#[rpc::mutation]
pub async fn attune(cx: &rpc::MutationCx, art: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?.change(cx, Change::Attune(art)).await
}

/// What a judged hunt came to, and, for a husk escrowed on-chain, the
/// receipt that burns it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettledHunt {
	/// The verdict, what it paid, and the player afterwards. An absorbed
	/// husk's codex entry carries the keeper's signature.
	pub settlement: HuntSettlement,

	/// What the wallet takes to the chain to burn the escrowed husk, when
	/// the husk was released from escrow and the hunt absorbed it.
	#[serde(default)]
	pub receipt: Option<ChainReceipt>,
}

/// A signed absorption receipt, ready for the Reapers program.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChainReceipt {
	/// The 98-byte receipt message, in standard base64.
	pub message: String,

	/// The keeper's Ed25519 signature over `message`, in standard base64.
	pub signature: String,

	/// The keeper's public key, as a Solana address.
	pub keeper: String,

	/// The wallet the receipt was made out to.
	pub wallet: String,

	/// The escrowed husk's mint.
	pub mint: String,

	/// An unsigned legacy transaction, in standard base64, carrying the
	/// Ed25519 check of the receipt and the program's `Absorb`, for the
	/// wallet to sign and send. `None` when the keeper could not build one,
	/// such as when the chain had no recent blockhash for it.
	#[serde(default)]
	pub transaction: Option<String>,
}

/// The public key absorption receipts are checked against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KeeperKey {
	/// The 32-byte Ed25519 key, as 64 hexadecimal digits.
	pub hex: String,

	/// The same key, as a Solana address: the keeper the program trusts.
	pub address: String,

	/// Whether this key is derived from a public, development-only secret.
	/// Receipts signed with it prove nothing; a deployed keeper never uses
	/// one.
	pub development: bool,
}

/// `log`, packed by `HuntLog::encode`, as [`settle_hunt`] takes it.
pub fn encode_log(log: &[u8]) -> String {
	use base64::Engine;

	base64::engine::general_purpose::STANDARD.encode(log)
}
