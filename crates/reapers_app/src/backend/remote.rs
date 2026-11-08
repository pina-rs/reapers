//! The books kept on the Reapers server.
//!
//! The server applies the same rules crate the device does, with its own
//! seeds and its own clock, and replays every hunt before paying for it.
//! [`RemoteBackend`] only carries requests there and turns the answers back
//! into the rules' types.
//!
//! What it speaks to is [`ReapersApi`]: one method per endpoint of the Dart
//! server (`PlayerEndpoint`, `SolanaEndpoint`, `RiftEndpoint`), in the
//! rules' types. TODO(integration): `reapers_api`. The Rust server's client
//! lives in `reapers_api`, written beside this in another branch; the
//! coordinator implements [`ReapersApi`] over it (a thin adapter of a dozen
//! calls) and hands it to [`RemoteBackend::new`]. Until then a server build
//! gets [`UnwiredApi`], which answers every call as unreachable.

use std::cell::RefCell;
use std::rc::Rc;

use async_trait::async_trait;
use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::stream::LocalBoxStream;
use reapers_rules::Art;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntSettlement;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;
use reapers_rules::RiftSite;
use reapers_rules::Weapon;

use super::GameBackend;
use super::KeeperError;
use super::KeeperKind;
use super::KeeperResult;
use super::RiftFrames;
use super::RiftSession;
use crate::wallet::WalletProver;

/// How the server answered a call that did not succeed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
	/// The server's rules refused, naming the refusal as the rules crate
	/// spells it (`Refusal::name`).
	#[error("refused: {0}")]
	Refused(String),

	/// The server could not be reached, timed out, or answered nonsense.
	#[error("unavailable: {0}")]
	Unavailable(String),
}

pub type ApiResult<T> = Result<T, ApiError>;

/// What the server says about the chain it verifies reapers against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainStatus {
	/// `solana:devnet` or `solana:mainnet`.
	pub chain: String,
	/// The app identity wallets are asked to trust, with a trailing slash.
	/// `https://reapers.invalid/` means wallet connection is off.
	pub identity_uri: String,
	/// The wallet linked to this player, when there is one.
	pub wallet: Option<String>,
	/// That wallet's real finalized balance.
	pub lamports: Option<i64>,
	/// The verified Metaplex Core collection reapers are imported from.
	pub collection: Option<String>,
	/// Where the collection can be bought.
	pub marketplace_uri: Option<String>,
}

impl ChainStatus {
	/// Whether wallets may be connected at all.
	pub fn wallets_enabled(&self) -> bool {
		!self
			.identity_uri
			.split("://")
			.nth(1)
			.and_then(|rest| rest.split(['/', ':']).next())
			.is_none_or(|host| host.ends_with(".invalid"))
	}
}

/// A sign-in challenge for one wallet address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletChallenge {
	pub challenge_id: String,
	/// The exact text the wallet must sign.
	pub message: String,
}

/// The Reapers server, one method per endpoint.
///
/// Implementations sign in on their own: a stored session is restored, or a
/// guest account is opened on first use, and the session is kept on the
/// device. Every method answers in the rules' types.
#[async_trait(?Send)]
pub trait ReapersApi {
	/// Restores the stored session, or opens a guest account.
	async fn sign_in(&self) -> ApiResult<()>;

	// PlayerEndpoint.
	async fn load(&self) -> ApiResult<PlayerState>;
	async fn update_location(&self, fix: Option<GeoFix>) -> ApiResult<()>;
	async fn take_oath(&self) -> ApiResult<PlayerState>;
	async fn select_reaper(&self, id: &str) -> ApiResult<PlayerState>;
	async fn acquire(&self, nft_id: &str) -> ApiResult<PlayerState>;
	async fn release(&self, nft_id: &str) -> ApiResult<PlayerState>;
	/// `log` is [`HuntLog::encode`]'s bytes.
	async fn settle_hunt(&self, sighting_id: i64, log: Vec<u8>) -> ApiResult<HuntSettlement>;
	async fn buy_weapon(&self, weapon_id: &str) -> ApiResult<PlayerState>;
	async fn wield(&self, weapon_id: &str) -> ApiResult<PlayerState>;
	async fn buy_outfit(&self, outfit: &str) -> ApiResult<PlayerState>;
	async fn wear(&self, outfit: &str) -> ApiResult<PlayerState>;
	async fn attune(&self, art: &str) -> ApiResult<PlayerState>;

	// SolanaEndpoint.
	async fn chain_status(&self) -> ApiResult<ChainStatus>;
	async fn import_reaper(&self, asset: &str) -> ApiResult<PlayerState>;
	async fn challenge(&self, address: &str) -> ApiResult<WalletChallenge>;
	/// Proves the wallet and switches the stored session to its account.
	async fn login(&self, challenge_id: &str, signature: &str) -> ApiResult<()>;

	// RiftEndpoint: one streaming call per reaper. Frames come down; this
	// reaper's reports go up, and ending `reports` tells the server the
	// reaper has gone.
	async fn enter_rift(
		&self,
		site: &str,
		demo: bool,
		reports: LocalBoxStream<'static, RiftReport>,
	) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>>;
}

/// A server build before `reapers_api` is wired in: every call is
/// unreachable, and the game says so.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnwiredApi;

const UNWIRED: &str = "TODO(integration): reapers_api is not wired into this build";

#[async_trait(?Send)]
impl ReapersApi for UnwiredApi {
	async fn sign_in(&self) -> ApiResult<()> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn load(&self) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn update_location(&self, _fix: Option<GeoFix>) -> ApiResult<()> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn take_oath(&self) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn select_reaper(&self, _id: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn acquire(&self, _nft_id: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn release(&self, _nft_id: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn settle_hunt(&self, _sighting_id: i64, _log: Vec<u8>) -> ApiResult<HuntSettlement> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn buy_weapon(&self, _weapon_id: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn wield(&self, _weapon_id: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn buy_outfit(&self, _outfit: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn wear(&self, _outfit: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn attune(&self, _art: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn chain_status(&self) -> ApiResult<ChainStatus> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn import_reaper(&self, _asset: &str) -> ApiResult<PlayerState> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn challenge(&self, _address: &str) -> ApiResult<WalletChallenge> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn login(&self, _challenge_id: &str, _signature: &str) -> ApiResult<()> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}

	async fn enter_rift(
		&self,
		_site: &str,
		_demo: bool,
		_reports: LocalBoxStream<'static, RiftReport>,
	) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>> {
		Err(ApiError::Unavailable(UNWIRED.into()))
	}
}

/// The server's keeper, over any [`ReapersApi`].
pub struct RemoteBackend<A> {
	api: Rc<A>,
	/// Whether this install has signed in. A failed sign-in is tried again
	/// on the next request.
	signed_in: RefCell<bool>,
}

impl<A: ReapersApi + 'static> RemoteBackend<A> {
	pub fn new(api: A) -> Self {
		Self {
			api: Rc::new(api),
			signed_in: RefCell::new(false),
		}
	}

	/// Sends one request, signing in as a guest first when needed.
	async fn ask<T>(&self, request: impl AsyncFnOnce(&A) -> ApiResult<T>) -> KeeperResult<T> {
		if !*self.signed_in.borrow() {
			self.api.sign_in().await.map_err(answer)?;
			*self.signed_in.borrow_mut() = true;
		}
		match request(&self.api).await {
			Ok(value) => Ok(value),
			Err(error @ ApiError::Unavailable(_)) => {
				// A session the server no longer knows is restored, or a new
				// guest opened, before the next request.
				*self.signed_in.borrow_mut() = false;
				Err(answer(error))
			}
			Err(error) => Err(answer(error)),
		}
	}
}

/// The server names a refusal by the rules' name for it. A name this build
/// does not know means the server is newer than the game.
fn answer(error: ApiError) -> KeeperError {
	match error {
		ApiError::Refused(name) => {
			match Refusal::from_name(&name) {
				Some(refusal) => refusal.into(),
				None => {
					KeeperError::unavailable(format!(
						"unknown refusal \"{name}\" from a newer server"
					))
				}
			}
		}
		ApiError::Unavailable(reason) => KeeperError::Unavailable(reason),
	}
}

#[async_trait(?Send)]
impl<A: ReapersApi + 'static> GameBackend for RemoteBackend<A> {
	fn kind(&self) -> KeeperKind {
		KeeperKind::Server
	}

	async fn load(&self) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.load().await).await
	}

	async fn locate(&self, fix: Option<GeoFix>) -> KeeperResult<()> {
		self.ask(async |api| api.update_location(fix).await).await
	}

	async fn take_oath(&self) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.take_oath().await).await
	}

	async fn select_reaper(&self, id: &str) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.select_reaper(id).await).await
	}

	async fn acquire(&self, nft: Nft) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.acquire(&nft.id()).await).await
	}

	async fn release(&self, nft: Nft) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.release(&nft.id()).await).await
	}

	async fn settle_hunt(&self, sighting_id: i64, log: &HuntLog) -> KeeperResult<HuntSettlement> {
		let bytes = log.encode();
		self.ask(async |api| api.settle_hunt(sighting_id, bytes).await)
			.await
	}

	async fn buy_weapon(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.buy_weapon(&weapon.id()).await)
			.await
	}

	async fn wield(&self, weapon: Weapon) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.wield(&weapon.id()).await).await
	}

	async fn buy_outfit(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.buy_outfit(outfit.name()).await)
			.await
	}

	async fn wear(&self, outfit: Outfit) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.wear(outfit.name()).await).await
	}

	async fn attune(&self, art: Art) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.attune(art.name()).await).await
	}

	async fn enter_rift(&self, site: RiftSite) -> KeeperResult<Box<dyn RiftSession>> {
		self.enter(site, false).await
	}

	async fn enter_demo_rift(&self, site: RiftSite) -> KeeperResult<Box<dyn RiftSession>> {
		self.enter(site, true).await
	}

	async fn chain_status(&self) -> KeeperResult<Option<ChainStatus>> {
		self.ask(async |api| api.chain_status().await)
			.await
			.map(Some)
	}

	async fn connect_wallet(
		&self,
		status: &ChainStatus,
		prover: &dyn WalletProver,
	) -> KeeperResult<()> {
		let api = self.api.clone();
		let proof = prover
			.prove(status, &|address: String| {
				let api = api.clone();
				Box::pin(async move { api.challenge(&address).await.map_err(answer) })
			})
			.await?;
		self.ask(async |api| api.login(&proof.challenge_id, &proof.signature).await)
			.await
	}

	async fn import_reaper(&self, asset: &str) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.import_reaper(asset).await).await
	}
}

impl<A: ReapersApi + 'static> RemoteBackend<A> {
	async fn enter(&self, site: RiftSite, demo: bool) -> KeeperResult<Box<dyn RiftSession>> {
		let (reports, outgoing) = mpsc::unbounded();
		let frames = self
			.ask(async |api| {
				api.enter_rift(site.name(), demo, outgoing.boxed_local())
					.await
			})
			.await?;
		Ok(Box::new(ServerRiftSession {
			reports,
			frames: Some(frames.map(|frame| frame.map_err(answer)).boxed_local()),
		}))
	}
}

/// A rift hosted by the server: frames come down one stream and this
/// reaper's reports go up another.
struct ServerRiftSession {
	reports: mpsc::UnboundedSender<RiftReport>,
	frames: Option<RiftFrames>,
}

impl RiftSession for ServerRiftSession {
	fn frames(&mut self) -> Option<RiftFrames> {
		self.frames.take()
	}

	fn report(&self, report: &RiftReport) {
		let _ = self.reports.unbounded_send(report.clone());
	}

	/// Closing the reports is what tells the server this reaper has gone.
	fn leave(&self) {
		self.reports.close_channel();
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;

	use futures::executor::block_on;

	use super::*;

	/// A server that refuses everything but signing in, and counts sign-ins.
	#[derive(Default)]
	struct Refusing {
		sign_ins: Cell<u32>,
		refusal: &'static str,
	}

	#[async_trait(?Send)]
	impl ReapersApi for Refusing {
		async fn sign_in(&self) -> ApiResult<()> {
			self.sign_ins.set(self.sign_ins.get() + 1);
			Ok(())
		}

		async fn load(&self) -> ApiResult<PlayerState> {
			Ok(PlayerState::initial())
		}

		async fn update_location(&self, _fix: Option<GeoFix>) -> ApiResult<()> {
			Err(ApiError::Unavailable("timed out".into()))
		}

		async fn take_oath(&self) -> ApiResult<PlayerState> {
			Err(ApiError::Refused(self.refusal.into()))
		}

		async fn select_reaper(&self, _id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn acquire(&self, _nft_id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn release(&self, _nft_id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn settle_hunt(&self, _sighting_id: i64, _log: Vec<u8>) -> ApiResult<HuntSettlement> {
			unimplemented!()
		}

		async fn buy_weapon(&self, _weapon_id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn wield(&self, _weapon_id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn buy_outfit(&self, _outfit: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn wear(&self, _outfit: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn attune(&self, _art: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn chain_status(&self) -> ApiResult<ChainStatus> {
			unimplemented!()
		}

		async fn import_reaper(&self, _asset: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn challenge(&self, _address: &str) -> ApiResult<WalletChallenge> {
			unimplemented!()
		}

		async fn login(&self, _challenge_id: &str, _signature: &str) -> ApiResult<()> {
			unimplemented!()
		}

		async fn enter_rift(
			&self,
			_site: &str,
			_demo: bool,
			_reports: LocalBoxStream<'static, RiftReport>,
		) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>> {
			unimplemented!()
		}
	}

	#[test]
	fn a_refusal_names_the_rule() {
		let keeper = RemoteBackend::new(Refusing {
			refusal: "oathAlreadyTaken",
			..Refusing::default()
		});
		let error = block_on(keeper.take_oath()).unwrap_err();
		assert_eq!(error.refusal(), Some(Refusal::OathAlreadyTaken));
	}

	#[test]
	fn an_unknown_refusal_is_a_newer_server() {
		let keeper = RemoteBackend::new(Refusing {
			refusal: "somethingNew",
			..Refusing::default()
		});
		assert!(matches!(
			block_on(keeper.take_oath()),
			Err(KeeperError::Unavailable(_))
		));
	}

	#[test]
	fn sign_in_happens_once_until_the_server_is_lost() {
		let keeper = RemoteBackend::new(Refusing::default());
		block_on(keeper.load()).unwrap();
		block_on(keeper.load()).unwrap();
		assert_eq!(keeper.api.sign_ins.get(), 1);

		let _ = block_on(keeper.locate(None));
		block_on(keeper.load()).unwrap();
		assert_eq!(keeper.api.sign_ins.get(), 2);
	}

	#[test]
	fn the_unwired_server_is_unreachable() {
		let keeper = RemoteBackend::new(UnwiredApi);
		assert!(matches!(
			block_on(keeper.load()),
			Err(KeeperError::Unavailable(_))
		));
	}

	#[test]
	fn an_invalid_identity_turns_wallets_off() {
		let status = ChainStatus {
			chain: "solana:devnet".into(),
			identity_uri: "https://reapers.invalid/".into(),
			wallet: None,
			lamports: None,
			collection: None,
			marketplace_uri: None,
		};
		assert!(!status.wallets_enabled());
		assert!(
			ChainStatus {
				identity_uri: "https://reapers.example/".into(),
				..status
			}
			.wallets_enabled()
		);
	}
}
