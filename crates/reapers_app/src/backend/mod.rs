//! Who keeps a player's books.
//!
//! Every request returns the player as they stand afterwards, or a
//! [`KeeperError`]: a [`RuleBreach`] when the rules refuse, or
//! [`KeeperError::Unavailable`] when the keeper could not be reached. There
//! are three keepers: the device itself ([`LocalBackend`]), the device
//! posting to its simnet ([`crate::simnet::SimnetBackend`]), and the Reapers
//! server ([`RemoteBackend`]), which judges every request with the same
//! rules crate.

mod local;
mod remote;
mod rift_session;

use async_trait::async_trait;
pub use local::DeviceDice;
pub use local::Dice;
pub use local::LocalBackend;
pub use local::ReceiptSigner;
pub use local::SeededDice;
use reapers_rules::Art;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntSettlement;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::RiftSite;
use reapers_rules::RuleBreach;
use reapers_rules::Weapon;
pub use remote::ChainStatus;
pub use remote::ReapersApi;
pub use remote::RemoteBackend;
pub use remote::UnwiredApi;
pub use remote::WalletChallenge;
pub use rift_session::LocalRiftSession;
pub use rift_session::RiftFrames;
pub use rift_session::RiftSession;

use crate::platform::PlatformError;
use crate::wallet::WalletProver;

/// Why a keeper did not do what it was asked.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum KeeperError {
	/// The rules refused, and say why.
	#[error(transparent)]
	Refused(#[from] RuleBreach),

	/// The keeper could not be reached, or answered with something that is
	/// not an answer. A timed-out change may have reached it: read the books
	/// again before deciding whether to repeat it. The reason is for a log,
	/// never shown as it is.
	#[error("the keeper is unavailable: {0}")]
	Unavailable(String),

	/// The books were read and are not a player: a damaged save, or a
	/// keeper answering in a format this build does not know.
	#[error("the books are damaged: {0}")]
	Damaged(String),
}

impl KeeperError {
	pub fn unavailable(reason: impl std::fmt::Display) -> Self {
		Self::Unavailable(reason.to_string())
	}

	/// The refusal, when the rules refused.
	pub const fn refusal(&self) -> Option<Refusal> {
		match self {
			Self::Refused(breach) => Some(breach.refusal),
			Self::Unavailable(_) | Self::Damaged(_) => None,
		}
	}
}

impl From<Refusal> for KeeperError {
	fn from(refusal: Refusal) -> Self {
		Self::Refused(RuleBreach { refusal })
	}
}

impl From<PlatformError> for KeeperError {
	fn from(error: PlatformError) -> Self {
		Self::unavailable(error)
	}
}

/// What every keeper answers.
pub type KeeperResult<T> = Result<T, KeeperError>;

/// Which kind of keeper this is, for the screens that say so: the roster's
/// wallet panel and the sanctum's balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeeperKind {
	Local,
	Simnet,
	Server,
}

/// Whoever keeps the books for a player.
#[async_trait(?Send)]
pub trait GameBackend {
	fn kind(&self) -> KeeperKind;

	/// The player as they stand now, created on first use.
	async fn load(&self) -> KeeperResult<PlayerState>;

	/// Shares the latest foreground fix, or clears it when tracking stops.
	async fn locate(&self, fix: Option<GeoFix>) -> KeeperResult<()>;

	/// Binds a reaper. The keeper, never the player, draws it.
	async fn take_oath(&self) -> KeeperResult<PlayerState>;

	async fn select_reaper(&self, id: &str) -> KeeperResult<PlayerState>;

	/// Buys `nft` into the vault.
	async fn acquire(&self, nft: Nft) -> KeeperResult<PlayerState>;

	/// Releases `nft` from the vault into the veil, where it becomes a husk.
	async fn release(&self, nft: Nft) -> KeeperResult<PlayerState>;

	/// Judges the recorded hunt for `sighting_id` and settles it.
	async fn settle_hunt(&self, sighting_id: i64, log: &HuntLog) -> KeeperResult<HuntSettlement>;

	async fn buy_weapon(&self, weapon: Weapon) -> KeeperResult<PlayerState>;

	async fn wield(&self, weapon: Weapon) -> KeeperResult<PlayerState>;

	async fn buy_outfit(&self, outfit: Outfit) -> KeeperResult<PlayerState>;

	async fn wear(&self, outfit: Outfit) -> KeeperResult<PlayerState>;

	/// Makes `art` the one the reaper throws.
	async fn attune(&self, art: Art) -> KeeperResult<PlayerState>;

	/// Takes the player to the rift at `site` and seats them in its circle.
	async fn enter_rift(&self, site: RiftSite) -> KeeperResult<Box<dyn RiftSession>>;

	/// Seats the player in the development demo circle at `site`. Only a
	/// development server has one.
	async fn enter_demo_rift(&self, _site: RiftSite) -> KeeperResult<Box<dyn RiftSession>> {
		Err(KeeperError::unavailable("this build has no demo rift"))
	}

	/// The chain the keeper verifies reapers against, when it has one: only
	/// the server does.
	async fn chain_status(&self) -> KeeperResult<Option<ChainStatus>> {
		Ok(None)
	}

	/// Proves control of a wallet with `prover` and links it to this player.
	async fn connect_wallet(
		&self,
		_status: &ChainStatus,
		_prover: &dyn WalletProver,
	) -> KeeperResult<()> {
		Err(KeeperError::unavailable("this keeper links no wallet"))
	}

	/// Imports the reaper NFT at `asset`, owned by the linked wallet.
	async fn import_reaper(&self, _asset: &str) -> KeeperResult<PlayerState> {
		Err(KeeperError::unavailable("this keeper imports no reaper"))
	}

	/// Forgets the player, as if the game had just been installed. Only the
	/// simnet offers this, to start over for the next person.
	async fn forget(&self) -> KeeperResult<()> {
		Err(KeeperError::unavailable("this keeper keeps its books"))
	}
}
