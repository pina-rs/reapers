//! What the operations' bodies hand their work to, on the server.
//!
//! The bodies in this crate only pass a request on. The server implements
//! [`Keeper`] (in `reapers_server`) and puts a [`KeeperHandle`] in every
//! request's context, beside the signed-in session, so each body finds the
//! keeper of the server that received it. Nothing global: two keepers in
//! one process, as the tests run them, never meet.

use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use leptoskit_rpc as rpc;
use reapers_rules::GeoFix;
use reapers_rules::PlayerState;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;

use crate::chain::Chain;
use crate::chain::Challenge;
use crate::player::ChainReceipt;
use crate::player::KeeperKey;
use crate::player::SettledHunt;

/// The requests that are one shared rule applied to the player's books,
/// with the identifier the game sent, unparsed: the keeper parses it, and a
/// malformed one is a refusal, not a server error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
	/// No rule: the books as they stand, opened on first use.
	Load,
	TakeOath,
	SelectReaper(String),
	Acquire(String),
	Release {
		nft: String,
		escrow: Option<String>,
	},
	BuyWeapon(String),
	Wield(String),
	BuyOutfit(String),
	Wear(String),
	Attune(String),
	ImportReaper(String),
}

/// The keeper: whatever answers the operations.
pub trait Keeper: Send + Sync + 'static {
	/// Applies `change` to the signed-in player's books, in the mutation's
	/// transaction.
	fn change<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		change: Change,
	) -> BoxFuture<'a, rpc::Result<PlayerState>>;

	/// Replays and settles a hunt; see [`crate::player::settle_hunt`].
	fn settle_hunt<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		sighting: i64,
		log: String,
	) -> BoxFuture<'a, rpc::Result<SettledHunt>>;

	/// The burn of an absorbed, still escrowed husk again; see
	/// [`crate::player::absorb_transaction`].
	fn absorb_transaction<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		nft: String,
	) -> BoxFuture<'a, rpc::Result<ChainReceipt>>;

	/// Keeps or clears the player's latest location.
	fn update_location<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		fix: Option<GeoFix>,
	) -> BoxFuture<'a, rpc::Result<()>>;

	/// The player's latest location, while it is fresh.
	fn location<'a>(&'a self, cx: &'a rpc::QueryCx) -> BoxFuture<'a, rpc::Result<Option<GeoFix>>>;

	/// The key receipts are signed with.
	fn receipt_key(&self) -> KeeperKey;

	/// The chain, and the player's wallet on it.
	fn chain_status<'a>(&'a self, cx: &'a rpc::ActionCx) -> BoxFuture<'a, rpc::Result<Chain>>;

	/// A message for the wallet at `address` to sign.
	fn wallet_challenge<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		address: String,
	) -> BoxFuture<'a, rpc::Result<Challenge>>;

	/// Seats the player in a rift and follows it.
	fn enter_rift<'a>(
		&'a self,
		cx: &'a rpc::QueryCx,
		site: String,
		demo: bool,
	) -> BoxFuture<'a, rpc::Result<BoxStream<'static, rpc::Result<RiftFrame>>>>;

	/// Takes in a report from the player's seat in their rift.
	fn report_rift<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		report: RiftReport,
	) -> BoxFuture<'a, rpc::Result<()>>;
}

/// The keeper, as a request's context carries it.
#[derive(Clone)]
pub struct KeeperHandle(pub Arc<dyn Keeper>);

impl std::fmt::Debug for KeeperHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("KeeperHandle")
	}
}

/// The keeper the request in `identity` reached.
///
/// # Errors
///
/// A server error when the server put no keeper in the context, which is a
/// bug in how it was wired, not something a player did.
pub fn keeper(identity: &rpc::Identity) -> rpc::Result<Arc<dyn Keeper>> {
	identity
		.context
		.get::<KeeperHandle>()
		.map(|handle| Arc::clone(&handle.0))
		.ok_or_else(|| rpc::Error::server("the keeper is not running"))
}
