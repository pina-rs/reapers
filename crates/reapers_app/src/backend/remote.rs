//! The books kept on the Reapers server.
//!
//! The server applies the same rules crate the device does, with its own
//! seeds and its own clock, and replays every hunt before paying for it.
//! [`RemoteBackend`] only carries requests there and turns the answers back
//! into the rules' types.
//!
//! What it speaks to is [`ReapersApi`]: one method per operation of
//! `reapers_api`, in the rules' types. [`super::ServerApi`] is the real one;
//! the tests hand in their own.
//!
//! On a cluster build ([`Self::with_cluster`]) the keeper also moves real
//! husks with the player's wallet: releasing a demo husk the wallet holds
//! signs and sends `ReleaseCore`, waits for it to be finalized and then
//! tells the server which escrow it went into; settling a hunt that
//! absorbed one has the wallet sign and send the burn the server built. A
//! burn that did not land is finished later with the same receipt, which
//! the server builds again.

use std::cell::RefCell;
use std::rc::Rc;

use async_trait::async_trait;
use futures::StreamExt as _;
use futures::channel::mpsc;
use futures::stream::LocalBoxStream;
use reapers_api::chain::Husks;
use reapers_api::player::ChainReceipt;
use reapers_api::player::SettledHunt;
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
use crate::chain::ChainFailure;
use crate::chain::Cluster;
use crate::chain::HuskPlace;
use crate::chain::WalletHusk;
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
	/// The Reapers program husks are escrowed with, when one is configured.
	pub program: Option<String>,
	/// The demo husk collections, and the Core asset each husk stands for.
	pub husks: Vec<Husks>,
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
	/// Monotonic counter incremented whenever the underlying auth session
	/// switches accounts (such as silent guest fallback after 401 or wallet
	/// login).
	fn session_generation(&self) -> u64 {
		0
	}

	/// Restores the stored session, or opens a guest account.
	async fn sign_in(&self) -> ApiResult<()>;

	// PlayerEndpoint.
	async fn load(&self) -> ApiResult<PlayerState>;
	async fn update_location(&self, fix: Option<GeoFix>) -> ApiResult<()>;
	async fn take_oath(&self) -> ApiResult<PlayerState>;
	async fn select_reaper(&self, id: &str) -> ApiResult<PlayerState>;
	async fn acquire(&self, nft_id: &str) -> ApiResult<PlayerState>;
	/// `escrow` is the real husk's mint, when the wallet escrowed it.
	async fn release(&self, nft_id: &str, escrow: Option<&str>) -> ApiResult<PlayerState>;
	/// `log` is [`HuntLog::encode`]'s bytes.
	async fn settle_hunt(&self, sighting_id: i64, log: Vec<u8>) -> ApiResult<SettledHunt>;
	/// The burn of the absorbed, still escrowed husk `nft_id` again: the
	/// same receipt, on a fresh blockhash.
	async fn absorb_transaction(&self, nft_id: &str) -> ApiResult<ChainReceipt>;
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

/// The server's keeper, over any [`ReapersApi`].
pub struct RemoteBackend<A> {
	api: Rc<A>,
	/// Whether this install has signed in. A failed sign-in is tried again
	/// on the next request.
	signed_in: RefCell<bool>,
	/// Cached chain configuration & linked wallet, keyed by the
	/// `api.session_generation()` it was asked under; invalidated on login or
	/// session reset.
	cached_status: RefCell<Option<(u64, ChainStatus)>>,
	/// The cluster real husks live on, and the wallet that moves them, on
	/// a cluster build.
	cluster: Option<(Rc<Cluster>, Rc<dyn WalletProver>)>,
}

impl<A: ReapersApi + 'static> RemoteBackend<A> {
	pub fn new(api: A) -> Self {
		Self {
			api: Rc::new(api),
			signed_in: RefCell::new(false),
			cached_status: RefCell::new(None),
			cluster: None,
		}
	}

	/// The same keeper, moving real husks on `cluster` with `wallet`.
	#[must_use]
	pub fn with_cluster(mut self, cluster: Rc<Cluster>, wallet: Rc<dyn WalletProver>) -> Self {
		self.cluster = Some((cluster, wallet));
		self
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
				*self.cached_status.borrow_mut() = None;
				Err(answer(error))
			}
			Err(error) => Err(answer(error)),
		}
	}

	/// The chain status, from the cache when it is this session's.
	async fn status_cached(&self) -> KeeperResult<ChainStatus> {
		let generation = self.api.session_generation();
		if let Some((cached_generation, status)) = self.cached_status.borrow().as_ref()
			&& *cached_generation == generation
		{
			return Ok(status.clone());
		}
		self.status_fresh().await
	}

	/// The chain status, asked for now and kept for the session it was asked
	/// under. A reply that lands after the session changed (a 401 that
	/// opened a new guest, or a wallet login) was the previous account's: it
	/// is dropped and asked for again, once, under the new session.
	async fn status_fresh(&self) -> KeeperResult<ChainStatus> {
		for _ in 0..2 {
			let generation = self.api.session_generation();
			let status = self.ask(async |api| api.chain_status().await).await?;
			if self.api.session_generation() == generation {
				*self.cached_status.borrow_mut() = Some((generation, status.clone()));
				return Ok(status);
			}
		}
		Err(KeeperError::unavailable(
			"the session changed while its chain status was read",
		))
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

/// The linked wallet's address, when the server knows one.
fn linked(status: &ChainStatus) -> KeeperResult<solana_kit_core::Address> {
	status
		.wallet
		.as_deref()
		.and_then(|wallet| wallet.parse().ok())
		.ok_or(KeeperError::Chain(ChainFailure::NoWallet))
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

	/// Releases `nft`. On a cluster build, a demo husk the wallet holds is
	/// released for real: bought into the vault if it is not there yet (the
	/// stand-in market's price), moved into the program's escrow by the
	/// wallet, and only once that is finalized released to the server with
	/// its escrow. One the wallet already escrowed skips the chain.
	async fn release(&self, nft: Nft) -> KeeperResult<PlayerState> {
		let Some((cluster, wallet)) = &self.cluster else {
			return self
				.ask(async |api| api.release(&nft.id(), None).await)
				.await;
		};
		let status = self.status_cached().await?;
		cluster.check_status(&status)?;
		let held = match status.wallet.as_deref() {
			Some(_) => {
				let address = linked(&status)?;
				cluster
					.wallet_husk(&status.husks, &address, nft)
					.await?
					.filter(|husk| husk.place != HuskPlace::Burned)
					.map(|husk| (address, husk))
			}
			None => None,
		};
		let Some((address, husk)) = held else {
			return self
				.ask(async |api| api.release(&nft.id(), None).await)
				.await;
		};
		let player = self.ask(async |api| api.load().await).await?;
		if !player.oath_taken() {
			return Err(Refusal::OathRequired.into());
		}
		if player.sightings().len() >= PlayerState::MAX_SIGHTINGS {
			return Err(Refusal::VeilCrowded.into());
		}
		let player = if player.vault().contains(&nft) {
			player
		} else {
			player.acquire(nft)?;
			self.ask(async |api| api.acquire(&nft.id()).await).await?
		};
		player.release(nft, 0, reapers_rules::Timestamp::from_unix_seconds(0))?;
		if husk.place == HuskPlace::InWallet {
			cluster
				.release(&status, wallet.as_ref(), &address, &husk)
				.await?;
		}
		let mint = husk.asset.to_string();
		self.ask(async |api| api.release(&nft.id(), Some(&mint)).await)
			.await
	}

	/// Settles the hunt. On a cluster build, a hunt that absorbed an
	/// escrowed husk comes back with the burn the server built, which the
	/// wallet signs and sends here. The books are settled either way: a
	/// burn that fails is put in front of the player, and the husk stays in
	/// escrow.
	async fn settle_hunt(&self, sighting_id: i64, log: &HuntLog) -> KeeperResult<HuntSettlement> {
		let bytes = log.encode_compact();
		let settled = self
			.ask(async |api| api.settle_hunt(sighting_id, bytes).await)
			.await?;
		if let (Some((cluster, wallet)), Some(receipt)) = (&self.cluster, &settled.receipt) {
			let nft = settled
				.settlement
				.state
				.codex()
				.last()
				.map(|absorption| absorption.nft);
			if let Some(nft) = nft
				&& let Err(error) = self.burn(cluster, wallet.as_ref(), nft, receipt).await
			{
				cluster.fail(&match error {
					KeeperError::Chain(failure) => failure,
					other => ChainFailure::Unreachable(other.to_string()),
				});
			}
		}
		Ok(settled.settlement)
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
		self.status_fresh().await.map(Some)
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
			.await?;
		*self.cached_status.borrow_mut() = None;
		Ok(())
	}

	async fn import_reaper(&self, asset: &str) -> KeeperResult<PlayerState> {
		self.ask(async |api| api.import_reaper(asset).await).await
	}

	/// Asks the server for the burn of `nft` again and has the wallet sign
	/// and send it, exactly as after the hunt. Its failure is the player's
	/// to see, since they asked.
	async fn finish_burn(&self, nft: Nft) -> KeeperResult<()> {
		let Some((cluster, wallet)) = &self.cluster else {
			return Err(KeeperError::unavailable("this build moves no husk"));
		};
		let status = self.status_cached().await?;
		cluster.check_status(&status)?;
		let receipt = self
			.ask(async |api| api.absorb_transaction(&nft.id()).await)
			.await?;
		self.burn(cluster, wallet.as_ref(), nft, &receipt).await
	}

	async fn wallet_husks(&self) -> KeeperResult<Vec<WalletHusk>> {
		let Some((cluster, _)) = &self.cluster else {
			return Ok(Vec::new());
		};
		let status = self.status_cached().await?;
		cluster.check_status(&status)?;
		if status.wallet.is_none() {
			return Ok(Vec::new());
		}
		Ok(cluster
			.wallet_husks(&status.husks, &linked(&status)?)
			.await?)
	}
}

impl<A: ReapersApi + 'static> RemoteBackend<A> {
	/// Has the wallet sign and send the burn `receipt` carries.
	async fn burn(
		&self,
		cluster: &Cluster,
		wallet: &dyn WalletProver,
		nft: Nft,
		receipt: &ChainReceipt,
	) -> KeeperResult<()> {
		use base64::Engine as _;
		use base64::engine::general_purpose::STANDARD;

		let Some(transaction) = &receipt.transaction else {
			return Err(ChainFailure::Rejected("the server built no burn".into()).into());
		};
		let status = self.status_cached().await?;
		cluster.check_status(&status)?;
		let address = linked(&status)?;
		if receipt.wallet.parse::<solana_kit_core::Address>().ok() != Some(address) {
			return Err(ChainFailure::WrongWallet.into());
		}
		let mint: solana_kit_core::Address = receipt
			.mint
			.parse()
			.map_err(|_| ChainFailure::Rejected("invalid receipt mint".into()))?;
		let message = STANDARD
			.decode(&receipt.message)
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
		let decoded = crate::receipt::Receipt::decode(&message)
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
		if solana_kit_core::Address::from_bytes(decoded.mint) != mint {
			return Err(ChainFailure::Rejected("receipt mint mismatch".into()).into());
		}
		let signature = STANDARD
			.decode(&receipt.signature)
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
		cluster
			.absorb(
				&status,
				wallet,
				&address,
				nft,
				transaction,
				&message,
				&signature,
				&receipt.keeper,
			)
			.await?;
		Ok(())
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
	/// With `chain`, it names the linked wallet; with `burn`, it builds that
	/// burn again.
	#[derive(Default)]
	struct Refusing {
		sign_ins: Cell<u32>,
		refusal: &'static str,
		chain: Option<ChainStatus>,
		burn: Option<ChainReceipt>,
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

		async fn release(&self, _nft_id: &str, _escrow: Option<&str>) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn settle_hunt(&self, _sighting_id: i64, _log: Vec<u8>) -> ApiResult<SettledHunt> {
			unimplemented!()
		}

		async fn absorb_transaction(&self, _nft_id: &str) -> ApiResult<ChainReceipt> {
			self.burn
				.clone()
				.ok_or_else(|| ApiError::Refused(self.refusal.into()))
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
			Ok(self.chain.clone().expect("a chain"))
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

	/// A cluster that takes every transaction, confirms it, and shows every
	/// account as Core's one-byte tombstone.
	#[derive(Default)]
	struct Burning {
		calls: RefCell<Vec<String>>,
	}

	#[async_trait(?Send)]
	impl crate::platform::HttpTransport for Burning {
		async fn post(
			&self,
			_url: &str,
			_headers: Vec<crate::platform::Header>,
			body: Vec<u8>,
			_timeout: std::time::Duration,
		) -> Result<crate::platform::HttpReply, crate::platform::PlatformError> {
			use base64::Engine as _;
			use serde_json::json;

			let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
			let method = request["method"].as_str().unwrap().to_owned();
			let tombstone = base64::engine::general_purpose::STANDARD.encode([0]);
			let result = match method.as_str() {
				"sendTransaction" => json!("sent"),
				"getSignatureStatuses" => {
					json!({ "value": [{ "slot": 7, "confirmationStatus": "confirmed", "err": null }] })
				}
				"getMultipleAccounts" => {
					json!({ "value": [{
						"owner": crate::program::core_program().to_string(),
						"data": [tombstone, "base64"],
					}] })
				}
				other => panic!("unexpected {other}"),
			};
			self.calls.borrow_mut().push(method);
			Ok(crate::platform::HttpReply {
				status: 200,
				body: serde_json::to_vec(
					&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result }),
				)
				.unwrap(),
			})
		}
	}

	/// A wallet holding `0`, or one that declines.
	struct Wallet(Option<solana_kit_core::crypto::Keypair>);

	#[async_trait(?Send)]
	impl WalletProver for Wallet {
		fn available(&self) -> bool {
			true
		}

		async fn prove(
			&self,
			_status: &ChainStatus,
			_challenge: crate::wallet::ChallengeFn<'_>,
		) -> KeeperResult<crate::wallet::WalletProof> {
			unimplemented!()
		}

		async fn sign_transaction(
			&self,
			_status: &ChainStatus,
			transaction: Vec<u8>,
		) -> crate::chain::ChainResult<Vec<u8>> {
			use solana_kit_core::transaction::Transaction;

			let keypair = self.0.as_ref().ok_or(ChainFailure::Declined)?;
			Transaction::decode(&transaction)
				.and_then(|transaction| transaction.sign(keypair))
				.and_then(|signed| signed.encode(true))
				.map_err(|_| ChainFailure::WrongWallet)
		}
	}

	/// What the server answers for the saint's burn, built again: the
	/// keeper's receipt for `wallet` and its unsigned transaction. The saint
	/// is `degenApes:1`, which [`demo_manifest`] maps to the asset `[1; 32]`
	/// of the collection `[9; 32]`.
	fn saint_burn(wallet: &solana_kit_core::Address) -> ChainReceipt {
		use base64::Engine as _;
		use base64::engine::general_purpose::STANDARD;
		use solana_kit_core::Hash;
		use solana_kit_core::transaction::Lifetime;
		use solana_kit_core::transaction::Message;
		use solana_kit_core::transaction::Transaction;
		use solana_kit_core::transaction::Version;

		let keeper = solana_kit_core::crypto::Keypair::from_seed(&[5; 32]);
		let mint = solana_kit_core::Address::from_bytes([1; 32]);
		let collection = solana_kit_core::Address::from_bytes([9; 32]);
		let receipt = crate::receipt::Receipt {
			player: *wallet.as_bytes(),
			mint: *mint.as_bytes(),
			grade: reapers_receipt::Grade::from_byte(1).unwrap(),
			feeding: reapers_receipt::Feeding::from_byte(2).unwrap(),
			xp: 120,
			anima: 30,
			at: 1_790_000_000,
			nonce: 7,
		};
		let signature = keeper.sign(&receipt.encode());
		let instructions = crate::program::absorb_core(
			&keeper.address(),
			signature.as_bytes(),
			&receipt,
			Some(&collection),
		);
		let message = Message::compile(
			Version::Legacy,
			*wallet,
			Lifetime::Blockhash {
				blockhash: Hash::from_bytes([3; 32]),
				last_valid_block_height: 100,
			},
			&instructions,
			&[],
		)
		.unwrap();
		ChainReceipt {
			message: STANDARD.encode(receipt.encode()),
			signature: STANDARD.encode(signature.as_bytes()),
			keeper: keeper.address().to_string(),
			wallet: wallet.to_string(),
			mint: mint.to_string(),
			transaction: Some(STANDARD.encode(Transaction::new(message).encode(false).unwrap())),
		}
	}

	fn linked_to(wallet: &solana_kit_core::Address) -> ChainStatus {
		ChainStatus {
			chain: "solana:localnet".into(),
			identity_uri: "https://reapers.localhost/".into(),
			wallet: Some(wallet.to_string()),
			lamports: None,
			collection: None,
			marketplace_uri: None,
			program: Some(crate::program::program_id().to_string()),
			husks: demo_manifest(),
		}
	}

	fn cluster(rpc: &Rc<Burning>) -> Rc<Cluster> {
		Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		)
	}

	#[test]
	fn a_burn_that_did_not_land_is_finished_with_the_receipt_built_again() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let rpc = Rc::new(Burning::default());
		let chain = cluster(&rpc);
		let landed = Rc::new(RefCell::new(Vec::new()));
		chain.subscribe({
			let landed = landed.clone();
			move |transaction| landed.borrow_mut().push(transaction.clone())
		});
		let burn = saint_burn(&address);
		let keeper = RemoteBackend::new(Refusing {
			chain: Some(linked_to(&address)),
			burn: Some(burn.clone()),
			..Refusing::default()
		})
		.with_cluster(chain, Rc::new(Wallet(Some(keypair))));
		let saint = Nft::parse("degenApes:1").unwrap();

		block_on(keeper.finish_burn(saint)).expect("the burn lands");

		assert_eq!(
			*rpc.calls.borrow(),
			[
				"sendTransaction",
				"getSignatureStatuses",
				"getMultipleAccounts"
			],
			"sent, confirmed, and the tombstone checked"
		);
		let landed = landed.borrow();
		assert_eq!(landed.len(), 1);
		assert_eq!(
			landed[0].instruction,
			crate::simnet::SimnetInstruction::Burn
		);
		assert_eq!(landed[0].nft, Some(saint));
		assert!(landed[0].receipt_verifies(), "the keeper's receipt");
		assert!(landed[0].wire_verifies(), "signed by the wallet");
	}

	#[test]
	fn a_burn_the_server_will_not_build_again_or_the_wallet_declines_sends_nothing() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let saint = Nft::parse("degenApes:1").unwrap();
		let rpc = Rc::new(Burning::default());
		let burned = RemoteBackend::new(Refusing {
			refusal: "notInVault",
			chain: Some(linked_to(&address)),
			..Refusing::default()
		})
		.with_cluster(cluster(&rpc), Rc::new(Wallet(Some(keypair))));

		assert_eq!(
			block_on(burned.finish_burn(saint)).unwrap_err().refusal(),
			Some(Refusal::NotInVault)
		);

		let declining = RemoteBackend::new(Refusing {
			chain: Some(linked_to(&address)),
			burn: Some(saint_burn(&address)),
			..Refusing::default()
		})
		.with_cluster(cluster(&rpc), Rc::new(Wallet(None)));

		assert_eq!(
			block_on(declining.finish_burn(saint)),
			Err(KeeperError::Chain(ChainFailure::Declined))
		);
		assert!(rpc.calls.borrow().is_empty(), "nothing reached the cluster");

		let offline = RemoteBackend::new(Refusing::default());
		assert!(matches!(
			block_on(offline.finish_burn(saint)),
			Err(KeeperError::Unavailable(_))
		));
	}

	#[test]
	fn finish_burn_refuses_mismatched_chain_receipt_mint() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let saint = Nft::parse("degenApes:1").unwrap();
		let rpc = Rc::new(Burning::default());
		let mut burn = saint_burn(&address);
		burn.mint = solana_kit_core::Address::from_bytes([99; 32]).to_string();
		let keeper = RemoteBackend::new(Refusing {
			chain: Some(linked_to(&address)),
			burn: Some(burn),
			..Refusing::default()
		})
		.with_cluster(cluster(&rpc), Rc::new(Wallet(Some(keypair))));

		assert!(matches!(
			block_on(keeper.finish_burn(saint)),
			Err(KeeperError::Chain(ChainFailure::Rejected(_)))
		));
		assert!(rpc.calls.borrow().is_empty(), "nothing reached the cluster");
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
			program: None,
			husks: Vec::new(),
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

	/// RPC stub that serves a wallet-owned Core asset and accepts `ReleaseCore`.
	struct ReleasingRpc {
		owner: solana_kit_core::Address,
		calls: RefCell<Vec<String>>,
	}

	#[async_trait(?Send)]
	impl crate::platform::HttpTransport for ReleasingRpc {
		async fn post(
			&self,
			_url: &str,
			_headers: Vec<crate::platform::Header>,
			body: Vec<u8>,
			_timeout: std::time::Duration,
		) -> Result<crate::platform::HttpReply, crate::platform::PlatformError> {
			use base64::Engine as _;
			use serde_json::json;

			let request: serde_json::Value = serde_json::from_slice(&body).unwrap();
			let method = request["method"].as_str().unwrap().to_owned();
			self.calls.borrow_mut().push(method.clone());
			let mut asset_data = vec![1u8];
			asset_data.extend_from_slice(self.owner.as_bytes());
			asset_data.push(0);
			let result = match method.as_str() {
				"getMultipleAccounts" => {
					let count = request["params"][0].as_array().map_or(2, Vec::len);
					let half = count / 2;
					let encoded = base64::engine::general_purpose::STANDARD.encode(asset_data);
					let mut values = Vec::with_capacity(count);
					for index in 0..count {
						if index < half {
							values.push(json!({
								"owner": crate::program::core_program().to_string(),
								"data": [encoded, "base64"],
							}));
						} else {
							values.push(serde_json::Value::Null);
						}
					}
					json!({ "value": values })
				}
				"getLatestBlockhash" => {
					json!({ "value": {
						"blockhash": solana_kit_core::Hash::from_bytes([7; 32]).to_string(),
						"lastValidBlockHeight": 100
					} })
				}
				"sendTransaction" => json!("escrowed-sig"),
				"getSignatureStatuses" => {
					json!({ "value": [{ "slot": 11, "confirmationStatus": "finalized", "err": null }] })
				}
				other => panic!("unexpected RPC method {other}"),
			};
			Ok(crate::platform::HttpReply {
				status: 200,
				body: serde_json::to_vec(
					&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result }),
				)
				.unwrap(),
			})
		}
	}

	struct CountingWallet {
		keypair: solana_kit_core::crypto::Keypair,
		signed: Cell<u32>,
	}

	#[async_trait(?Send)]
	impl WalletProver for CountingWallet {
		fn available(&self) -> bool {
			true
		}

		async fn prove(
			&self,
			_status: &ChainStatus,
			_challenge: crate::wallet::ChallengeFn<'_>,
		) -> KeeperResult<crate::wallet::WalletProof> {
			unimplemented!()
		}

		async fn sign_transaction(
			&self,
			_status: &ChainStatus,
			transaction: Vec<u8>,
		) -> crate::chain::ChainResult<Vec<u8>> {
			use solana_kit_core::transaction::Transaction;

			self.signed.set(self.signed.get() + 1);
			Transaction::decode(&transaction)
				.and_then(|tx| tx.sign(&self.keypair))
				.and_then(|tx| tx.encode(true))
				.map_err(|_| ChainFailure::WrongWallet)
		}
	}

	struct ReleaseStub {
		player: PlayerState,
		chain: ChainStatus,
		release_refusal: &'static str,
	}

	#[async_trait(?Send)]
	impl ReapersApi for ReleaseStub {
		async fn sign_in(&self) -> ApiResult<()> {
			Ok(())
		}

		async fn load(&self) -> ApiResult<PlayerState> {
			Ok(self.player.clone())
		}

		async fn update_location(&self, _fix: Option<GeoFix>) -> ApiResult<()> {
			unimplemented!()
		}

		async fn take_oath(&self) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn select_reaper(&self, _id: &str) -> ApiResult<PlayerState> {
			unimplemented!()
		}

		async fn acquire(&self, _nft_id: &str) -> ApiResult<PlayerState> {
			Ok(self.player.clone())
		}

		async fn release(&self, _nft_id: &str, _escrow: Option<&str>) -> ApiResult<PlayerState> {
			Err(ApiError::Refused(self.release_refusal.into()))
		}

		async fn settle_hunt(&self, _sighting_id: i64, _log: Vec<u8>) -> ApiResult<SettledHunt> {
			unimplemented!()
		}

		async fn absorb_transaction(&self, _nft_id: &str) -> ApiResult<ChainReceipt> {
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
			Ok(self.chain.clone())
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

	fn demo_manifest() -> Vec<Husks> {
		vec![Husks {
			collection: "degenApes".into(),
			address: solana_kit_core::Address::from_bytes([9; 32]).to_string(),
			assets: (1..=8_u8)
				.map(|n| solana_kit_core::Address::from_bytes([n; 32]).to_string())
				.collect(),
		}]
	}

	#[test]
	fn release_checks_veil_capacity_before_irreversible_on_chain_escrow() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let now = reapers_rules::Timestamp::utc(2026, 11, 4, 12, 0, 0);
		let mut player = PlayerState::initial().take_oath(1).unwrap();
		for serial in 1..=3 {
			let nft = Nft::new(reapers_rules::NftCollection::DegenApes, serial);
			player = player
				.acquire(nft)
				.unwrap()
				.release(nft, serial as i64, now)
				.unwrap();
		}
		let fourth = Nft::new(reapers_rules::NftCollection::DegenApes, 4);
		player = player.acquire(fourth).unwrap();
		assert_eq!(player.sightings().len(), PlayerState::MAX_SIGHTINGS);

		let rpc = Rc::new(ReleasingRpc {
			owner: address,
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let wallet = Rc::new(CountingWallet {
			keypair,
			signed: Cell::new(0),
		});
		let status = ChainStatus {
			husks: demo_manifest(),
			..linked_to(&address)
		};
		let keeper = RemoteBackend::new(ReleaseStub {
			player,
			chain: status,
			release_refusal: "veilCrowded",
		})
		.with_cluster(cluster, wallet.clone());

		let error = block_on(keeper.release(fourth)).unwrap_err();
		assert_eq!(error.refusal(), Some(Refusal::VeilCrowded));
		assert_eq!(
			wallet.signed.get(),
			0,
			"must not ask wallet to sign irreversible ReleaseCore when 3 husks are already loose"
		);
		assert!(
			!rpc.calls.borrow().iter().any(|m| m == "sendTransaction"),
			"must not broadcast ReleaseCore when veil is crowded"
		);
	}

	#[test]
	fn release_checks_oath_before_irreversible_on_chain_escrow() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let player = PlayerState::initial();
		assert!(!player.oath_taken());
		let saint = Nft::new(reapers_rules::NftCollection::DegenApes, 1);

		let rpc = Rc::new(ReleasingRpc {
			owner: address,
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let wallet = Rc::new(CountingWallet {
			keypair,
			signed: Cell::new(0),
		});
		let status = ChainStatus {
			husks: demo_manifest(),
			..linked_to(&address)
		};
		let keeper = RemoteBackend::new(ReleaseStub {
			player,
			chain: status,
			release_refusal: "oathRequired",
		})
		.with_cluster(cluster, wallet.clone());

		let error = block_on(keeper.release(saint)).unwrap_err();
		assert_eq!(error.refusal(), Some(Refusal::OathRequired));
		assert_eq!(
			wallet.signed.get(),
			0,
			"must not ask wallet to sign irreversible ReleaseCore before taking the oath"
		);
		assert!(
			!rpc.calls.borrow().iter().any(|m| m == "sendTransaction"),
			"must not broadcast ReleaseCore when oath has not been taken"
		);
	}

	#[test]
	fn release_checks_acquire_affordability_before_irreversible_on_chain_escrow() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let mut value = serde_json::to_value(PlayerState::initial().take_oath(1).unwrap()).unwrap();
		value["lamports"] = serde_json::json!(0);
		let player: PlayerState = serde_json::from_value(value).unwrap();
		let expensive = Nft::new(reapers_rules::NftCollection::DegenApes, 8);
		assert!(!player.vault().contains(&expensive));
		assert!(player.lamports() < expensive.rarity().lamports());

		let rpc = Rc::new(ReleasingRpc {
			owner: address,
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let wallet = Rc::new(CountingWallet {
			keypair,
			signed: Cell::new(0),
		});
		let status = ChainStatus {
			husks: demo_manifest(),
			..linked_to(&address)
		};
		let keeper = RemoteBackend::new(ReleaseStub {
			player,
			chain: status,
			release_refusal: "insufficientFunds",
		})
		.with_cluster(cluster, wallet.clone());

		let error = block_on(keeper.release(expensive)).unwrap_err();
		assert_eq!(error.refusal(), Some(Refusal::InsufficientFunds));
		assert_eq!(wallet.signed.get(), 0);
		assert!(!rpc.calls.borrow().iter().any(|m| m == "sendTransaction"));
	}

	#[test]
	fn release_refuses_when_server_chain_mismatches_client_cluster() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let saint = Nft::new(reapers_rules::NftCollection::DegenApes, 1);
		let player = PlayerState::initial()
			.take_oath(1)
			.unwrap()
			.acquire(saint)
			.unwrap();

		let rpc = Rc::new(ReleasingRpc {
			owner: address,
			calls: RefCell::new(Vec::new()),
		});
		// Client is built for Devnet, but server reports Localnet.
		let cluster = Rc::new(
			Cluster::new(crate::chain::ClusterName::Devnet, "http://rpc", rpc.clone())
				.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let wallet = Rc::new(CountingWallet {
			keypair,
			signed: Cell::new(0),
		});
		let status = ChainStatus {
			chain: "solana:localnet".into(),
			husks: demo_manifest(),
			..linked_to(&address)
		};
		let keeper = RemoteBackend::new(ReleaseStub {
			player,
			chain: status,
			release_refusal: "notInVault",
		})
		.with_cluster(cluster, wallet.clone());

		let result = block_on(keeper.release(saint));
		assert!(matches!(
			result,
			Err(KeeperError::Chain(ChainFailure::Rejected(_)))
		));
		assert_eq!(
			wallet.signed.get(),
			0,
			"must not sign ReleaseCore on devnet when the server runs on solana:localnet"
		);
		assert!(
			!rpc.calls.borrow().iter().any(|m| m == "sendTransaction"),
			"must not broadcast ReleaseCore when client cluster and server chain disagree"
		);
	}

	#[test]
	fn release_refuses_when_server_program_mismatches_or_is_missing() {
		let address = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]).address();
		let saint = Nft::new(reapers_rules::NftCollection::DegenApes, 1);
		let player = PlayerState::initial()
			.take_oath(1)
			.unwrap()
			.acquire(saint)
			.unwrap();

		let wrong_program = Some(solana_kit_core::Address::from_bytes([88; 32]).to_string());
		for program in [wrong_program, None] {
			let rpc = Rc::new(ReleasingRpc {
				owner: address,
				calls: RefCell::new(Vec::new()),
			});
			let cluster = Rc::new(
				Cluster::new(
					crate::chain::ClusterName::Localnet,
					"http://rpc",
					rpc.clone(),
				)
				.with_pause(Rc::new(|_| Box::pin(async { true }))),
			);
			let wallet = Rc::new(CountingWallet {
				keypair: solana_kit_core::crypto::Keypair::from_seed(&[6; 32]),
				signed: Cell::new(0),
			});
			let status = ChainStatus {
				program,
				husks: demo_manifest(),
				..linked_to(&address)
			};
			assert!(!cluster.matches(&status));
			let keeper = RemoteBackend::new(ReleaseStub {
				player: player.clone(),
				chain: status,
				release_refusal: "notInVault",
			})
			.with_cluster(cluster, wallet.clone());

			let result = block_on(keeper.release(saint));
			assert!(matches!(
				result,
				Err(KeeperError::Chain(ChainFailure::Rejected(_)))
			));
			assert_eq!(wallet.signed.get(), 0);
			assert!(rpc.calls.borrow().is_empty());
		}
	}

	#[test]
	fn wallet_husks_and_finish_burn_refuse_on_cluster_or_program_mismatch() {
		let keypair = solana_kit_core::crypto::Keypair::from_seed(&[6; 32]);
		let address = keypair.address();
		let saint = Nft::parse("degenApes:1").unwrap();
		let rpc = Rc::new(Burning::default());
		let devnet_cluster = Rc::new(
			Cluster::new(crate::chain::ClusterName::Devnet, "http://rpc", rpc.clone())
				.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let wallet = Rc::new(CountingWallet {
			keypair,
			signed: Cell::new(0),
		});
		// Server reports localnet while client cluster is devnet.
		let keeper = RemoteBackend::new(Refusing {
			chain: Some(linked_to(&address)),
			burn: Some(saint_burn(&address)),
			..Refusing::default()
		})
		.with_cluster(devnet_cluster, wallet.clone());

		assert!(matches!(
			block_on(keeper.wallet_husks()),
			Err(KeeperError::Chain(ChainFailure::Rejected(_)))
		));
		assert!(matches!(
			block_on(keeper.finish_burn(saint)),
			Err(KeeperError::Chain(ChainFailure::Rejected(_)))
		));
		assert_eq!(wallet.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	struct SequenceHttp {
		replies: RefCell<Vec<crate::platform::HttpReply>>,
	}

	#[async_trait(?Send)]
	impl crate::platform::HttpTransport for SequenceHttp {
		async fn post(
			&self,
			_url: &str,
			_headers: Vec<crate::platform::Header>,
			_body: Vec<u8>,
			_timeout: std::time::Duration,
		) -> Result<crate::platform::HttpReply, crate::platform::PlatformError> {
			Ok(self.replies.borrow_mut().remove(0))
		}
	}

	#[test]
	fn session_reset_on_401_invalidates_cached_chain_status() {
		use crate::backend::ServerApi;
		use crate::platform::HttpReply;
		use crate::platform::MemoryPrefs;
		use crate::platform::Prefs;

		let wallet = solana_kit_core::Address::from_bytes([6; 32]);
		let secrets = Rc::new(MemoryPrefs::new());
		block_on(secrets.set("reapers.server.refresh", serde_json::json!("old-refresh"))).unwrap();

		let old_chain = reapers_api::chain::Chain {
			chain: "solana:localnet".into(),
			identity_uri: "https://reapers.localhost/".into(),
			collection: None,
			marketplace_uri: None,
			program: Some(crate::program::program_id().to_string()),
			husks: demo_manifest(),
			wallet: Some(wallet.to_string()),
			lamports: Some(1_000_000),
		};
		let new_guest_chain = reapers_api::chain::Chain {
			wallet: None,
			lamports: None,
			..old_chain.clone()
		};
		let http = Rc::new(SequenceHttp {
			replies: RefCell::new(vec![
				// 1. Initial sign_in via /auth/refresh for the wallet-linked account.
				HttpReply {
					status: 200,
					body: serde_json::to_vec(&serde_json::json!({
						"access_token": "old-access",
						"refresh_token": "old-refresh-2",
						"expires_in": 900
					}))
					.unwrap(),
				},
				// 2. chain_status populates RemoteBackend::cached_status with old wallet.
				HttpReply {
					status: 200,
					body: serde_json::to_vec(&old_chain).unwrap(),
				},
				// 3. load() fails with 401 (session revoked).
				HttpReply {
					status: 401,
					body: b"unauthorized".to_vec(),
				},
				// 4. /auth/refresh fails with 401 -> ServerApi forgets refresh token.
				HttpReply {
					status: 401,
					body: b"revoked".to_vec(),
				},
				// 5. /auth/guest silently opens a brand-new unlinked guest account.
				HttpReply {
					status: 200,
					body: serde_json::to_vec(&serde_json::json!({
						"account": "fresh-guest",
						"access_token": "new-access",
						"refresh_token": "new-refresh",
						"expires_in": 900
					}))
					.unwrap(),
				},
				// 6. Retried /api/load succeeds as the new unlinked guest.
				HttpReply {
					status: 200,
					body: serde_json::to_vec(&PlayerState::initial()).unwrap(),
				},
				// 7. If cached_status was invalidated, wallet_husks fetches fresh chain_status.
				HttpReply {
					status: 200,
					body: serde_json::to_vec(&new_guest_chain).unwrap(),
				},
			]),
		});

		let rpc = Rc::new(ReleasingRpc {
			owner: wallet,
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let api = ServerApi::new("https://keeper.invalid", http, secrets);
		let keeper = RemoteBackend::new(api).with_cluster(cluster, Rc::new(Wallet(None)));

		// Populate cached_status for the wallet-linked account.
		let initial = block_on(keeper.chain_status()).unwrap().unwrap();
		assert_eq!(initial.wallet.as_deref(), Some(wallet.to_string().as_str()));

		// Session is revoked -> ServerApi silently creates a new unlinked guest.
		let _ = block_on(keeper.load()).unwrap();

		// The new unlinked guest must NOT see the previous account's wallet husks.
		let husks = block_on(keeper.wallet_husks()).unwrap();
		assert!(
			husks.is_empty(),
			"RemoteBackend::cached_status still held the previous account's linked wallet after 401 guest fallback"
		);
	}

	/// Answers each route from its own queue; the first `chain_status`
	/// waits until `gate` opens, as a reply still on the wire would.
	struct GatedHttp {
		routes: RefCell<
			std::collections::HashMap<
				&'static str,
				std::collections::VecDeque<crate::platform::HttpReply>,
			>,
		>,
		gate: RefCell<Option<futures::channel::oneshot::Receiver<()>>>,
		seen: RefCell<Vec<(String, Option<String>)>>,
	}

	#[async_trait(?Send)]
	impl crate::platform::HttpTransport for GatedHttp {
		async fn post(
			&self,
			url: &str,
			headers: Vec<crate::platform::Header>,
			_body: Vec<u8>,
			_timeout: std::time::Duration,
		) -> Result<crate::platform::HttpReply, crate::platform::PlatformError> {
			let path = url.trim_start_matches("https://keeper.invalid").to_owned();
			let bearer = headers
				.iter()
				.find(|(name, _)| *name == "authorization")
				.map(|(_, value)| value.clone());
			self.seen.borrow_mut().push((path.clone(), bearer));
			if path == "/api/chain_status" {
				let gate = self.gate.borrow_mut().take();
				if let Some(gate) = gate {
					let _ = gate.await;
				}
			}
			let reply = self
				.routes
				.borrow_mut()
				.get_mut(path.as_str())
				.and_then(std::collections::VecDeque::pop_front);
			Ok(reply.unwrap_or_else(|| panic!("unscripted request {path}")))
		}
	}

	/// A keeper on the real `ServerApi`, signed in as the account `wallet`
	/// is linked to. The first `chain_status` reply (the linked account's)
	/// is held back until the returned sender fires; the next `load` after
	/// the first finds the session revoked, and the client opens a new
	/// guest, whose status names no wallet.
	fn a_session_about_to_reset(
		wallet: solana_kit_core::Address,
	) -> (
		RemoteBackend<crate::backend::ServerApi>,
		Rc<GatedHttp>,
		futures::channel::oneshot::Sender<()>,
	) {
		use std::collections::HashMap;
		use std::collections::VecDeque;

		use crate::backend::ServerApi;
		use crate::platform::HttpReply;
		use crate::platform::MemoryPrefs;
		use crate::platform::Prefs;

		let secrets = Rc::new(MemoryPrefs::new());
		block_on(secrets.set("reapers.server.refresh", serde_json::json!("old-refresh"))).unwrap();
		let linked_chain = reapers_api::chain::Chain {
			chain: "solana:localnet".into(),
			identity_uri: "https://reapers.localhost/".into(),
			collection: None,
			marketplace_uri: None,
			program: Some(crate::program::program_id().to_string()),
			husks: demo_manifest(),
			wallet: Some(wallet.to_string()),
			lamports: Some(1_000_000),
		};
		let guest_chain = reapers_api::chain::Chain {
			wallet: None,
			lamports: None,
			..linked_chain.clone()
		};
		let ok = |value: serde_json::Value| {
			HttpReply {
				status: 200,
				body: serde_json::to_vec(&value).unwrap(),
			}
		};
		let revoked = || {
			HttpReply {
				status: 401,
				body: b"unauthorized".to_vec(),
			}
		};
		let tokens = |access: &str, refresh: &str| {
			serde_json::json!({
				"account": "an-account",
				"access_token": access,
				"refresh_token": refresh,
				"expires_in": 900
			})
		};
		let player = serde_json::to_value(PlayerState::initial()).unwrap();
		let (open, gate) = futures::channel::oneshot::channel::<()>();
		let mut routes: HashMap<&'static str, VecDeque<HttpReply>> = HashMap::new();
		routes.insert(
			"/auth/refresh",
			VecDeque::from([ok(tokens("old-access", "old-refresh-2")), revoked()]),
		);
		routes.insert(
			"/auth/guest",
			VecDeque::from([ok(tokens("new-access", "new-refresh"))]),
		);
		routes.insert(
			"/api/load",
			VecDeque::from([ok(player.clone()), revoked(), ok(player)]),
		);
		routes.insert(
			"/api/chain_status",
			VecDeque::from([
				ok(serde_json::to_value(&linked_chain).unwrap()),
				ok(serde_json::to_value(&guest_chain).unwrap()),
				ok(serde_json::to_value(&guest_chain).unwrap()),
			]),
		);
		let http = Rc::new(GatedHttp {
			routes: RefCell::new(routes),
			gate: RefCell::new(Some(gate)),
			seen: RefCell::new(Vec::new()),
		});
		let rpc = Rc::new(ReleasingRpc {
			owner: wallet,
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Rc::new(
			Cluster::new(
				crate::chain::ClusterName::Localnet,
				"http://rpc",
				rpc.clone(),
			)
			.with_pause(Rc::new(|_| Box::pin(async { true }))),
		);
		let api = ServerApi::new("https://keeper.invalid", http.clone(), secrets);
		let keeper = RemoteBackend::new(api).with_cluster(cluster, Rc::new(Wallet(None)));
		(keeper, http, open)
	}

	/// A status the linked account was answered with, still on the wire
	/// while another request finds the session revoked and opens a new
	/// guest, is that account's: it is neither shown to the guest nor kept
	/// as the guest's.
	#[test]
	fn a_status_fetched_across_a_session_reset_is_not_kept_for_the_new_session() {
		let wallet = solana_kit_core::Address::from_bytes([6; 32]);
		let (keeper, http, open) = a_session_about_to_reset(wallet);
		block_on(keeper.load()).unwrap();

		let (during, reset) = block_on(futures::future::join(keeper.wallet_husks(), async {
			let reset = keeper.load().await;
			open.send(()).unwrap();
			reset
		}));
		reset.unwrap();
		let during = during.unwrap();
		assert!(
			during.is_empty(),
			"the new guest was shown {} husks of the old wallet {wallet}; requests: {:?}",
			during.len(),
			http.seen.borrow()
		);

		let husks = block_on(keeper.wallet_husks()).unwrap();
		assert!(
			husks.is_empty(),
			"the new guest was served {} husks of the old wallet {wallet}; requests: {:?}",
			husks.len(),
			http.seen.borrow()
		);
	}

	/// The same race through `chain_status`, which the roster asks for
	/// directly: the guest is answered with its own status, and that is
	/// the one kept.
	#[test]
	fn a_chain_status_read_across_a_session_reset_is_the_new_sessions() {
		let wallet = solana_kit_core::Address::from_bytes([6; 32]);
		let (keeper, http, open) = a_session_about_to_reset(wallet);
		block_on(keeper.load()).unwrap();

		let (during, reset) = block_on(futures::future::join(keeper.chain_status(), async {
			let reset = keeper.load().await;
			open.send(()).unwrap();
			reset
		}));
		reset.unwrap();
		let during = during.unwrap().expect("a status");
		assert_eq!(
			during.wallet,
			None,
			"the new guest was told it is linked to the old wallet; requests: {:?}",
			http.seen.borrow()
		);

		let husks = block_on(keeper.wallet_husks()).unwrap();
		assert!(
			husks.is_empty(),
			"the new guest was served {} husks of the old wallet {wallet}; requests: {:?}",
			husks.len(),
			http.seen.borrow()
		);
	}
}
