//! The keeper: what answers every operation in `reapers_api`.
//!
//! Each answer finds the player from the request's session, so nothing the
//! game sends names a player, and each change is one shared rule applied to
//! the stored state. Anything the keeper must decide (a seed, the time, how
//! well a reaper fed) is decided here, never sent by the game.

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use leptoskit_auth::Account;
use leptoskit_auth::SignedIn;
use leptoskit_db::Db;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use reapers_api::chain::Chain;
use reapers_api::chain::Challenge;
use reapers_api::chain::Husks;
use reapers_api::keeper::Change;
use reapers_api::player::ChainReceipt;
use reapers_api::player::KeeperKey;
use reapers_api::player::SettledHunt;
use reapers_api::refused;
use reapers_rules::Art;
use reapers_rules::Difficulty;
use reapers_rules::GeoFix;
use reapers_rules::HuntLog;
use reapers_rules::HuntLogError;
use reapers_rules::HuntSettlement;
use reapers_rules::Judging;
use reapers_rules::Nft;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Refusal;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftSite;
use reapers_rules::RuleResult;
use reapers_rules::Timestamp;
use reapers_rules::Weapon;
use sha2::Digest;
use sha2::Sha256;
use solana_kit_core::Address;
use solana_kit_core::transaction::Lifetime;

use crate::absorb;
use crate::books;
use crate::books::breach;
use crate::chain::Gateway;
use crate::config::RunMode;
use crate::receipt;
use crate::receipt::Receipt;
use crate::rift::Chair;
use crate::rift::RiftHall;
use crate::signer::ReceiptSigner;
use crate::tables::HuskEscrow;
use crate::tables::NewHuskEscrow;
use crate::tables::NewWalletChallenge;
use crate::tables::WalletChallenge;
use crate::wallet;

/// How much kinder the keeper's replay of a hunt is than the phone's own
/// play of it. See `HuntStats::derive`.
pub const REPLAY_LENIENCY: f64 = 1.03;

/// How many times a transaction the keeper opens itself runs again when
/// another gets in its way, as a mutation's does.
const RETRIES: u32 = 5;

/// The keeper, with everything it reaches outside the database.
pub struct Keeper {
	mode: RunMode,
	db: Db,
	gateway: Gateway,
	signer: ReceiptSigner,
	hall: RiftHall,
}

impl std::fmt::Debug for Keeper {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Keeper")
			.field("mode", &self.mode)
			.field("gateway", &self.gateway)
			.field("signer", &self.signer)
			.finish_non_exhaustive()
	}
}

/// The time now, as the rules take it.
pub fn now() -> Timestamp {
	Timestamp::from_micros(chrono::Utc::now().timestamp_micros())
}

/// A seed only the keeper knows, from the operating system's secure random
/// numbers.
///
/// # Errors
///
/// A server error when the operating system has none to give.
pub fn seed() -> rpc::Result<i64> {
	getrandom::u32()
		.map(i64::from)
		.map_err(|_| rpc::Error::server("no secure random numbers"))
}

/// The signed-in account, or unauthorized.
pub fn me(identity: &rpc::Identity) -> rpc::Result<Id<Account>> {
	identity
		.context
		.get::<SignedIn>()
		.map(|signed_in| signed_in.account)
		.ok_or_else(|| rpc::Error::unauthorized("sign in first"))
}

impl Keeper {
	/// A keeper in `mode` that keeps its books in `db`, reads the chain
	/// through `gateway`, signs with `signer` and hosts rifts in `hall`.
	pub fn new(
		mode: RunMode,
		db: &Db,
		gateway: Gateway,
		signer: ReceiptSigner,
		hall: RiftHall,
	) -> Self {
		Self {
			mode,
			db: db.system("the keeper's own transactions"),
			gateway,
			signer,
			hall,
		}
	}

	/// The keeper's run mode.
	pub const fn mode(&self) -> RunMode {
		self.mode
	}

	/// The chain, as the keeper reads it.
	pub const fn gateway(&self) -> &Gateway {
		&self.gateway
	}

	/// The key receipts are signed with.
	pub const fn signer(&self) -> &ReceiptSigner {
		&self.signer
	}

	/// The database, bypassing policies: for the keeper's own transactions
	/// and its sign-in routes.
	pub const fn db(&self) -> &Db {
		&self.db
	}

	fn simulation(&self) -> rpc::Result<()> {
		self.mode.require_simulation().map_err(refused)
	}

	/// The difficulties a hunt may have been played at. Easy and hard are
	/// for testing, so a keeper that pays for real takes only the game's
	/// own.
	fn judging(&self) -> Judging {
		Judging {
			leniency: REPLAY_LENIENCY,
			difficulties: if self.mode == RunMode::Production {
				vec![Difficulty::Normal]
			} else {
				Difficulty::ALL.to_vec()
			},
		}
	}

	/// Applies `rule` to `account`'s books in a transaction of the keeper's
	/// own, run again when another gets in its way: for work that happens
	/// outside any operation's transaction, such as paying for a rift.
	pub async fn change_alone(
		&self,
		account: Id<Account>,
		mut rule: impl FnMut(PlayerState) -> rpc::Result<PlayerState>,
	) -> rpc::Result<PlayerState> {
		let mut attempt = 0;

		loop {
			let tx = self.db.begin_serializable().await?;
			let result = match books::change(&tx, &self.gateway, account, None, &mut rule).await {
				Ok(state) => tx.commit().await.map(|()| state).map_err(rpc::Error::from),
				Err(error) => {
					let _ = tx.rollback().await;
					Err(error)
				}
			};

			match result {
				Err(error) if error.is_contended() && attempt < RETRIES => attempt += 1,
				result => return result,
			}
		}
	}

	async fn change(&self, cx: &rpc::MutationCx, change: Change) -> rpc::Result<PlayerState> {
		let account = me(cx.identity())?;
		let tx = cx.tx();
		let apply = |rule| self.apply(tx, account, rule);

		match change {
			Change::Load => apply(Box::new(Ok)).await,
			Change::TakeOath => {
				let seed = seed()?;

				apply(Box::new(move |state| state.take_oath(seed))).await
			}
			Change::SelectReaper(id) => {
				apply(Box::new(move |state| state.select_reaper(&id))).await
			}
			Change::Acquire(id) => {
				self.simulation()?;
				apply(Box::new(move |state| {
					state.acquire(parsed(Nft::parse(&id).ok(), Refusal::NotListed)?)
				}))
				.await
			}
			Change::Release { nft, escrow } => self.release(cx, account, &nft, escrow).await,
			Change::BuyWeapon(id) => {
				apply(Box::new(move |state| {
					state.buy_weapon(parsed(Weapon::parse(&id).ok(), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Wield(id) => {
				apply(Box::new(move |state| {
					state.wield(parsed(Weapon::parse(&id).ok(), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::BuyOutfit(name) => {
				apply(Box::new(move |state| {
					state.buy_outfit(parsed(Outfit::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Wear(name) => {
				apply(Box::new(move |state| {
					state.wear(parsed(Outfit::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::Attune(name) => {
				apply(Box::new(move |state| {
					state.attune(parsed(Art::from_name(&name), Refusal::NotOwned)?)
				}))
				.await
			}
			Change::ImportReaper(asset) => {
				books::change(tx, &self.gateway, account, Some(&asset), Ok).await
			}
		}
	}

	/// Applies `rule` to `account`'s books in the operation's transaction.
	async fn apply(
		&self,
		tx: &leptoskit_db::Tx,
		account: Id<Account>,
		rule: Box<dyn FnOnce(PlayerState) -> RuleResult<PlayerState> + Send + '_>,
	) -> rpc::Result<PlayerState> {
		books::change(tx, &self.gateway, account, None, |state| {
			breach(rule(state))
		})
		.await
	}

	/// Releases `nft` into the veil; with `escrow`, binds the husk to the
	/// mint the player escrowed on-chain, once the chain shows the escrow.
	async fn release(
		&self,
		cx: &rpc::MutationCx,
		account: Id<Account>,
		nft: &str,
		escrow: Option<String>,
	) -> rpc::Result<PlayerState> {
		self.simulation()?;

		let tx = cx.tx();
		let seed = seed()?;
		let now = now();
		let bound = match escrow {
			Some(mint) => {
				self.demo_husk(nft, &mint)?;
				Some(self.escrowed(tx, account, &mint).await?)
			}
			None => None,
		};
		let mut sighting = None;
		let state = books::change(tx, &self.gateway, account, None, |state| {
			let nft = parsed(Nft::parse(nft).ok(), Refusal::NotListed)
				.map_err(|breach| refused(breach.refusal))?;

			sighting = Some(state.next_sighting_id());
			breach(state.release(nft, seed, now))
		})
		.await?;

		if let (Some((wallet, mint)), Some(sighting)) = (bound, sighting) {
			NewHuskEscrow {
				account_id: account,
				sighting_id: sighting,
				mint_key: format!("{}:{mint}", self.gateway.config().chain),
				mint: mint.to_string(),
				wallet,
			}
			.insert(tx)
			.await
			.map_err(|error| {
				if matches!(error, leptoskit_db::DbError::Conflict { .. }) {
					refused(Refusal::AlreadyHeld)
				} else {
					rpc::Error::from(error)
				}
			})?;
		}

		Ok(state)
	}

	/// With a demo husk manifest, checks `mint` is the asset the husk `nft`
	/// stands for, so a cheap asset cannot be burned for a rare husk's pay.
	/// Without one, any escrowed mint may stand for any husk.
	fn demo_husk(&self, nft: &str, mint: &str) -> rpc::Result<()> {
		let config = self.gateway.config();

		if config.husks.is_empty() {
			return Ok(());
		}

		let asset = Nft::parse(nft)
			.ok()
			.and_then(|nft| config.husk_asset(nft))
			.map(|asset| asset.to_string());

		if asset.as_deref() == Some(mint) {
			Ok(())
		} else {
			Err(refused(Refusal::NotInVault))
		}
	}

	/// Checks `mint` is escrowed with the program by `account`'s proved
	/// wallet, and returns the wallet and the mint.
	async fn escrowed(
		&self,
		tx: &impl leptoskit_db::Read,
		account: Id<Account>,
		mint: &str,
	) -> rpc::Result<(String, Address)> {
		let Ok(address) = mint.parse::<Address>() else {
			return Err(refused(Refusal::NotInVault));
		};
		let Some(wallet) = books::wallet_of(tx, account).await? else {
			return Err(refused(Refusal::WalletUnproven));
		};
		let Ok(wallet_address) = wallet.parse::<Address>() else {
			return Err(refused(Refusal::WalletUnproven));
		};

		self.gateway
			.escrowed(&wallet_address, &address)
			.await
			.map_err(refused)?;

		Ok((wallet, address))
	}

	async fn settle_hunt(
		&self,
		cx: &rpc::MutationCx,
		sighting: i64,
		log: &str,
	) -> rpc::Result<SettledHunt> {
		self.simulation()?;

		let account = me(cx.identity())?;
		let tx = cx.tx();
		let log = decode_log(log)?;
		let now = now();
		let feeding_seed = seed()?;
		let judging = self.judging();
		let bound = HuskEscrow::query()
			.filter(|escrow| escrow.account_id.eq(account) & escrow.sighting_id.eq(sighting))
			.first(tx)
			.await?;
		let wallet = books::wallet_of(tx, account).await?;

		// A husk escrowed on-chain is hunted only while it is still in
		// escrow, by the wallet that released it.
		if let Some(bound) = &bound {
			if wallet.as_deref() != Some(bound.wallet.as_str()) {
				return Err(refused(Refusal::WalletUnproven));
			}

			self.escrowed(tx, account, &bound.mint).await?;
		}

		let player = player_bytes(account, wallet.as_deref());
		let mut settled = None;

		books::change(tx, &self.gateway, account, None, |state| {
			let judged = breach(state.judge_hunt(sighting, &log, now, feeding_seed, &judging))?;
			let signed = self.sign(&judged, player, bound.as_ref())?;
			let next = signed.0.state.clone();

			settled = Some(signed);
			Ok(next)
		})
		.await?;

		let Some((settlement, receipt)) = settled else {
			return Err(rpc::Error::server("the hunt was not settled"));
		};
		let receipt = match (receipt, &bound) {
			(Some(receipt), Some(_)) => Some(self.chain_receipt(receipt).await),
			_ => None,
		};

		Ok(SettledHunt {
			settlement,
			receipt,
		})
	}

	/// `judged` with the absorption it recorded signed. A husk that got
	/// away leaves nothing to sign.
	///
	/// The receipt is made out to the player's proved wallet, for the
	/// escrowed husk's mint. A husk of the stand-in market has no mint and a
	/// guest no wallet, so their places hold the SHA-256 of the stand-in
	/// identifiers instead: such a receipt is signed and kept with the
	/// books, and no chain could act on it.
	fn sign(
		&self,
		judged: &HuntSettlement,
		player: [u8; 32],
		bound: Option<&HuskEscrow>,
	) -> rpc::Result<(HuntSettlement, Option<SignedReceipt>)> {
		if !judged.outcome.absorbed() {
			return Ok((judged.clone(), None));
		}

		let Some(absorption) = judged.state.codex().last() else {
			return Err(rpc::Error::server("an absorbed husk left no codex entry"));
		};
		let mint = match bound {
			Some(bound) => {
				bound
					.mint
					.parse::<Address>()
					.map(|mint| *mint.as_bytes())
					.map_err(|_| rpc::Error::server("an escrow binding holds no address"))?
			}
			None => Sha256::digest(absorption.nft.id().as_bytes()).into(),
		};
		let Some(receipt) = receipt::of(player, mint, absorption) else {
			return Err(rpc::Error::server("an absorption does not fit its receipt"));
		};
		let message = receipt.encode();
		let signature = self.signer.sign(&message);
		let encoded = STANDARD.encode(signature.as_bytes());
		let mut settlement = judged.clone();

		settlement.state = judged.state.sign_absorption(absorption.nft, &encoded);

		Ok((settlement, Some(SignedReceipt { receipt, signature })))
	}

	/// The receipt as the wallet takes it to the chain, with the absorb
	/// transaction when the chain says how to burn the husk and gives a
	/// blockhash for it.
	async fn chain_receipt(&self, signed: SignedReceipt) -> ChainReceipt {
		let message = signed.receipt.encode();
		let wallet = Address::from_bytes(signed.receipt.player);
		let mint = Address::from_bytes(signed.receipt.mint);
		let keeper = self.signer.address();
		let transaction = match (
			self.gateway.absorb_target(&mint).await,
			self.gateway.latest_blockhash().await,
		) {
			(Ok(burn), Ok((blockhash, last_valid_block_height))) => {
				absorb::transaction(
					&keeper,
					&signed.receipt,
					&signed.signature,
					burn,
					Lifetime::Blockhash {
						blockhash,
						last_valid_block_height,
					},
				)
				.map(|bytes| STANDARD.encode(bytes))
			}
			_ => None,
		};

		ChainReceipt {
			message: STANDARD.encode(message),
			signature: STANDARD.encode(signed.signature.as_bytes()),
			keeper: keeper.to_string(),
			wallet: wallet.to_string(),
			mint: mint.to_string(),
			transaction,
		}
	}

	async fn chain_status(&self, account: Id<Account>) -> rpc::Result<Chain> {
		let wallet = books::wallet_of(&self.db, account).await?;
		let lamports = match &wallet {
			Some(wallet) => {
				let address = wallet
					.parse::<Address>()
					.map_err(|_| refused(Refusal::WalletUnproven))?;

				Some(self.gateway.balance(&address).await.map_err(refused)?)
			}
			None => None,
		};
		let config = self.gateway.config();

		Ok(Chain {
			chain: config.chain.clone(),
			identity_uri: config.identity_uri.to_string(),
			collection: config.collection.map(|address| address.to_string()),
			marketplace_uri: config.marketplace_uri.as_ref().map(ToString::to_string),
			program: config.program.map(|address| address.to_string()),
			husks: config
				.husks
				.iter()
				.map(|husks| {
					Husks {
						collection: husks.collection.name().to_owned(),
						address: husks.address.to_string(),
						assets: husks.assets.iter().map(ToString::to_string).collect(),
					}
				})
				.collect(),
			wallet,
			lamports,
		})
	}

	async fn wallet_challenge(
		&self,
		cx: &rpc::MutationCx,
		address: &str,
	) -> rpc::Result<Challenge> {
		if !self.gateway.config().wallets_enabled() {
			return Err(refused(Refusal::ChainUnconfigured));
		}

		if !wallet::is_address(address) {
			return Err(refused(Refusal::WalletUnproven));
		}

		let account = me(cx.identity())?;
		let tx = cx.tx();

		// Opens the books, and checks the NFT reapers already in them.
		books::change(tx, &self.gateway, account, None, Ok).await?;

		let issued = chrono::Utc::now();
		let expires = issued + wallet::LIFETIME;
		let nonce = challenge_id()?;
		let message = wallet::message(self.gateway.config(), address, &nonce, issued, expires);

		WalletChallenge::query()
			.filter(|challenge| challenge.account_id.eq(account))
			.delete(tx)
			.await?;
		NewWalletChallenge {
			challenge_id: nonce.clone(),
			account_id: account,
			address: address.to_owned(),
			message: message.clone(),
			expires_at: expires,
			consumed: false,
		}
		.insert(tx)
		.await?;

		Ok(Challenge {
			challenge_id: nonce,
			address: address.to_owned(),
			message,
			expires_at: expires.to_rfc3339(),
		})
	}

	/// Seats `account` in the rift at `site` and follows it: the stream the
	/// `enter_rift` operation sends.
	pub async fn enter_rift(
		self: Arc<Self>,
		account: Id<Account>,
		site: &str,
		demo: bool,
	) -> rpc::Result<BoxStream<'static, rpc::Result<RiftFrame>>> {
		// Reported results are not replayed yet; their rewards belong only in
		// the simulated development economy. This is also what keeps the demo
		// out of staging and production: there is deliberately no bypass.
		self.simulation()?;

		let Some(site) = RiftSite::from_name(site) else {
			return Err(refused(Refusal::NotAttuned));
		};
		let mut entering = None;

		self.change_alone(account, |state| {
			let entrant = breach(if demo {
				demo_entrant(&state)
			} else {
				state.entrant_for(site)
			})?;

			entering = Some((entrant, state.active_reaper_id().map(str::to_owned)));
			Ok(state)
		})
		.await?;

		let Some((entrant, reaper)) = entering else {
			return Err(rpc::Error::server("the reaper was not read"));
		};
		let chair = self.hall.join(account, site, entrant, demo, seed()?)?;
		let following = Following {
			keeper: self,
			chair,
			account,
			reaper,
			step: Step::First,
		};

		Ok(Box::pin(futures::stream::unfold(
			following,
			Following::next,
		)))
	}

	/// Takes in `account`'s report from their seat: what the `report_rift`
	/// operation does.
	pub fn report_rift(&self, account: Id<Account>, report: &RiftReport) -> rpc::Result<()> {
		if report
			.standing
			.is_some_and(|standing| !standing.is_usable_at(now()))
		{
			let refusal = refused(Refusal::LocationUnusable);

			self.hall.fail(account, refusal.clone());
			return Err(refusal);
		}

		self.hall.report(account, report)
	}
}

/// What a signed absorption carries to the chain.
struct SignedReceipt {
	receipt: Receipt,
	signature: solana_kit_core::Signature,
}

/// The 32 bytes a receipt names its player by: their proved wallet, or,
/// for a guest with none, the SHA-256 of their account.
fn player_bytes(account: Id<Account>, wallet: Option<&str>) -> [u8; 32] {
	wallet
		.and_then(|wallet| wallet.parse::<Address>().ok())
		.map_or_else(
			|| Sha256::digest(account.uuid().to_string().as_bytes()).into(),
			|wallet| *wallet.as_bytes(),
		)
}

/// A fresh, unguessable challenge identifier.
fn challenge_id() -> rpc::Result<String> {
	let mut bytes = [0; 16];

	getrandom::fill(&mut bytes).map_err(|_| rpc::Error::server("no secure random numbers"))?;

	Ok(crate::signer::hex(&bytes))
}

/// `value`, or the refusal `otherwise` names. The identifiers come from the
/// game, so a malformed one is a refusal the player can be told about, not
/// a server error.
fn parsed<T>(value: Option<T>, otherwise: Refusal) -> RuleResult<T> {
	value.ok_or(reapers_rules::RuleBreach { refusal: otherwise })
}

/// The hunt log `encoded` packs, in standard base64.
fn decode_log(encoded: &str) -> rpc::Result<HuntLog> {
	let bytes = STANDARD
		.decode(encoded)
		.map_err(|_| refused(Refusal::HuntUnproven))?;

	HuntLog::decode(&bytes).map_err(|error| {
		if error == HuntLogError::UnsupportedVersion {
			refused(Refusal::HuntVersionUnsupported)
		} else {
			refused(Refusal::HuntUnproven)
		}
	})
}

/// What a reaper takes into a demo circle.
///
/// The starter is the only reaper allowed into a rift until rift reports
/// can be replayed, and the demo keeps that rule. It waives attunement
/// alone, and only a development or test keeper reaches this.
fn demo_entrant(state: &PlayerState) -> RuleResult<RiftEntrant> {
	let Some(reaper) = state.active_reaper() else {
		return Err(Refusal::OathRequired.into());
	};

	if reaper.is_nft() {
		return Err(Refusal::RiftUnproven.into());
	}

	Ok(RiftEntrant {
		name: state.genome().name(),
		loadout: state.loadout(),
	})
}

/// Where a reaper's stream of a rift has got to.
enum Step {
	First,
	Following,
	Done,
}

/// One reaper following their rift: the frames they are sent, and their
/// payment when it is sealed.
struct Following {
	keeper: Arc<Keeper>,
	chair: Chair,
	account: Id<Account>,
	reaper: Option<String>,
	step: Step,
}

impl Following {
	async fn next(mut self) -> Option<(rpc::Result<RiftFrame>, Self)> {
		match self.step {
			Step::Done => None,
			Step::First => {
				self.step = Step::Following;

				let frame = self.frame();

				Some((frame, self))
			}
			Step::Following => {
				// A report that ended the rift is told before the next beat.
				if let Some(failure) = self.chair.failure() {
					self.step = Step::Done;
					return Some((Err(failure), self));
				}

				if !self.chair.beat().await {
					return None;
				}

				let frame = match self.chair.failure().map_or_else(|| self.frame(), Err) {
					Ok(frame) => frame,
					Err(error) => {
						self.step = Step::Done;
						return Some((Err(error), self));
					}
				};

				if !frame.phase.is_over() {
					return Some((Ok(frame), self));
				}

				self.step = Step::Done;

				if frame.phase == RiftPhase::Sealed
					&& let Err(error) = self.settle().await
				{
					return Some((Err(error), self));
				}

				Some((Ok(frame), self))
			}
		}
	}

	fn frame(&self) -> rpc::Result<RiftFrame> {
		self.chair
			.frame()
			.ok_or_else(|| rpc::Error::server("the reaper has no seat"))
	}

	/// Pays this reaper's own books from their own share of the ward. The
	/// room reports a sealed rift once, so the books see exactly one
	/// payment.
	async fn settle(&self) -> rpc::Result<()> {
		let reward = self.chair.reward();
		let reaper = self.reaper.clone();

		self.keeper
			.change_alone(self.account, move |state| {
				breach(state.settle_rift(reward.xp, reward.anima, reaper.as_deref()))
			})
			.await
			.map(drop)
	}
}

/// The keeper, as the operations' bodies reach it.
#[derive(Clone, Debug)]
pub struct Shared(pub Arc<Keeper>);

impl reapers_api::keeper::Keeper for Shared {
	fn change<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		change: Change,
	) -> BoxFuture<'a, rpc::Result<PlayerState>> {
		Box::pin(self.0.change(cx, change))
	}

	fn settle_hunt<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		sighting: i64,
		log: String,
	) -> BoxFuture<'a, rpc::Result<SettledHunt>> {
		Box::pin(async move { self.0.settle_hunt(cx, sighting, &log).await })
	}

	fn update_location<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		fix: Option<GeoFix>,
	) -> BoxFuture<'a, rpc::Result<()>> {
		Box::pin(async move { books::locate(cx.tx(), me(cx.identity())?, fix, now()).await })
	}

	fn location<'a>(&'a self, cx: &'a rpc::QueryCx) -> BoxFuture<'a, rpc::Result<Option<GeoFix>>> {
		Box::pin(async move { books::location(cx.db(), me(cx.identity())?, now()).await })
	}

	fn receipt_key(&self) -> KeeperKey {
		self.0.signer.key()
	}

	fn chain_status<'a>(&'a self, cx: &'a rpc::ActionCx) -> BoxFuture<'a, rpc::Result<Chain>> {
		Box::pin(async move { self.0.chain_status(me(cx.identity())?).await })
	}

	fn wallet_challenge<'a>(
		&'a self,
		cx: &'a rpc::MutationCx,
		address: String,
	) -> BoxFuture<'a, rpc::Result<Challenge>> {
		Box::pin(async move { self.0.wallet_challenge(cx, &address).await })
	}

	fn enter_rift<'a>(
		&'a self,
		cx: &'a rpc::QueryCx,
		site: String,
		demo: bool,
	) -> BoxFuture<'a, rpc::Result<BoxStream<'static, rpc::Result<RiftFrame>>>> {
		Box::pin(async move {
			let account = me(cx.identity())?;

			Arc::clone(&self.0).enter_rift(account, &site, demo).await
		})
	}

	fn report_rift<'a>(
		&'a self,
		cx: &'a rpc::ActionCx,
		report: RiftReport,
	) -> BoxFuture<'a, rpc::Result<()>> {
		Box::pin(async move { self.0.report_rift(me(cx.identity())?, &report) })
	}
}

/// An account's books, opened, for the sign-in routes and tests.
pub async fn opened(db: &Db, account: Id<Account>) -> rpc::Result<PlayerState> {
	Ok(books::open(db, account).await?.state.0)
}
