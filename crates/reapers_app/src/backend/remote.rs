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
	/// The cluster real husks live on, and the wallet that moves them, on
	/// a cluster build.
	cluster: Option<(Rc<Cluster>, Rc<dyn WalletProver>)>,
}

impl<A: ReapersApi + 'static> RemoteBackend<A> {
	pub fn new(api: A) -> Self {
		Self {
			api: Rc::new(api),
			signed_in: RefCell::new(false),
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
		let status = self.ask(async |api| api.chain_status().await).await?;
		let held = match status.wallet.as_deref() {
			Some(_) => {
				let address = linked(&status)?;
				cluster
					.wallet_husks(&status.husks, &address)
					.await?
					.into_iter()
					.find(|husk| husk.nft == nft && husk.place != HuskPlace::Burned)
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
		if !player.vault().contains(&nft) {
			self.ask(async |api| api.acquire(&nft.id()).await).await?;
		}
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
		let bytes = log.encode();
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

	/// Asks the server for the burn of `nft` again and has the wallet sign
	/// and send it, exactly as after the hunt. Its failure is the player's
	/// to see, since they asked.
	async fn finish_burn(&self, nft: Nft) -> KeeperResult<()> {
		let Some((cluster, wallet)) = &self.cluster else {
			return Err(KeeperError::unavailable("this build moves no husk"));
		};
		let receipt = self
			.ask(async |api| api.absorb_transaction(&nft.id()).await)
			.await?;
		self.burn(cluster, wallet.as_ref(), nft, &receipt).await
	}

	async fn wallet_husks(&self) -> KeeperResult<Vec<WalletHusk>> {
		let Some((cluster, _)) = &self.cluster else {
			return Ok(Vec::new());
		};
		let status = self.ask(async |api| api.chain_status().await).await?;
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
		let status = self.ask(async |api| api.chain_status().await).await?;
		let address = linked(&status)?;
		if receipt.wallet != address.to_string() {
			return Err(ChainFailure::WrongWallet.into());
		}
		let message = STANDARD
			.decode(&receipt.message)
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
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
	/// keeper's receipt for `wallet` and its unsigned transaction.
	fn saint_burn(wallet: &solana_kit_core::Address) -> ChainReceipt {
		use base64::Engine as _;
		use base64::engine::general_purpose::STANDARD;
		use solana_kit_core::Hash;
		use solana_kit_core::transaction::Lifetime;
		use solana_kit_core::transaction::Message;
		use solana_kit_core::transaction::Transaction;
		use solana_kit_core::transaction::Version;

		let keeper = solana_kit_core::crypto::Keypair::from_seed(&[5; 32]);
		let mint = solana_kit_core::Address::from_bytes([2; 32]);
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
		let instructions =
			crate::program::absorb_core(&keeper.address(), signature.as_bytes(), &receipt, None);
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
			husks: Vec::new(),
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
}
