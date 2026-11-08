//! Wallet proof, the chain's status and verified NFT reaper imports.
//!
//! Payment and NFT exchange happen in the configured marketplace, signed by
//! the player's own wallet. The keeper only reads the chain: it holds no
//! wallet secret and signs no transaction.

use leptoskit_rpc as rpc;
use reapers_rules::PlayerState;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "server")]
use crate::keeper::Change;
#[cfg(feature = "server")]
use crate::keeper::keeper;

/// The chain this keeper reads, and this player's wallet on it.
///
/// An action rather than a query: it asks the chain for the wallet's
/// balance, which is outside the database.
#[rpc::action]
pub async fn chain_status(cx: &rpc::ActionCx) -> rpc::Result<Chain> {
	keeper(cx.identity())?.chain_status(cx).await
}

/// A two-minute, single-use message for the wallet at `address` to sign,
/// bound to this player, the keeper's identity origin and the chain. Send
/// the signature to the wallet sign-in route ([`crate::auth::WALLET`]).
#[rpc::mutation]
pub async fn wallet_challenge(cx: &rpc::MutationCx, address: String) -> rpc::Result<Challenge> {
	keeper(cx.identity())?.wallet_challenge(cx, address).await
}

/// Imports the Metaplex Core asset at `asset`, which the player's proved
/// wallet must own, as an NFT reaper. A reaper the keeper has never seen
/// starts at level one; one it has seen keeps the growth it earned.
#[rpc::mutation]
pub async fn import_reaper(cx: &rpc::MutationCx, asset: String) -> rpc::Result<PlayerState> {
	keeper(cx.identity())?
		.change(cx, Change::ImportReaper(asset))
		.await
}

/// The chain the keeper reads, and the player's wallet on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Chain {
	/// `solana:devnet` or `solana:mainnet`.
	pub chain: String,

	/// The keeper's identity origin, which wallet messages are bound to.
	pub identity_uri: String,

	/// The verified collection NFT reapers come from, when there is one.
	pub collection: Option<String>,

	/// Where the collection is bought and traded, when there is such a page.
	pub marketplace_uri: Option<String>,

	/// The Reapers program husks are escrowed with and burned by, when one
	/// is configured.
	pub program: Option<String>,

	/// The demo husk collections, when the keeper has a manifest of them:
	/// which real Core asset each husk of the rules stands for. A husk
	/// released with an escrow must be escrowed as its own asset.
	#[serde(default)]
	pub husks: Vec<Husks>,

	/// The player's proved wallet, if they have one.
	pub wallet: Option<String>,

	/// That wallet's finalized balance, in lamports.
	pub lamports: Option<u64>,
}

/// One demo husk collection on the chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Husks {
	/// The rules' collection it stands for, by name, such as `thugbirdz`.
	pub collection: String,

	/// Its Metaplex Core collection.
	pub address: String,

	/// Its assets in order: the first is the husk numbered 1.
	pub assets: Vec<String>,
}

/// A message for a wallet to sign, proving it to the keeper.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Challenge {
	/// Names this challenge when the signature comes back.
	pub challenge_id: String,

	/// The wallet that must sign it.
	pub address: String,

	/// The exact text to sign, as UTF-8.
	pub message: String,

	/// When it stops being accepted, in RFC 3339.
	pub expires_at: String,
}
