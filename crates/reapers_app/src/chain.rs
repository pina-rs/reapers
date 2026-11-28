//! A real cluster: devnet, or a developer's own validator, with the
//! Reapers program on it.
//!
//! The server keeps the books and signs the receipts; the player's wallet
//! holds the husks and signs every transaction that moves one. This module
//! is what the game does on the cluster between the two:
//!
//! - **The wallet's husks**: the demo collections' Core assets the server's
//!   manifest names (`chain_status`'s `husks`), read with
//!   `getMultipleAccounts`, and which of them the wallet owns or has
//!   released into escrow.
//! - **Releasing**: the program client's `ReleaseCore`, signed and sent by
//!   the wallet, followed until it is finalized, because the server checks
//!   the escrow at a finalized slot before the release counts.
//! - **Absorbing**: the transaction `settle_hunt` returns (the Ed25519 check
//!   of the keeper's receipt, then `AbsorbCore`), signed and sent by the
//!   wallet, and the asset checked to be the one-byte tombstone Core leaves.
//!
//! Each transaction that lands is told to whoever subscribed, as a ledger
//! entry the slip and the transaction sheet show. A program error is read
//! back as the program's own [`ReapersProgramError`].

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::future::LocalBoxFuture;
use num_traits::FromPrimitive as _;
use reapers_api::chain::Husks;
use reapers_program_client::accounts::Escrow;
use reapers_program_client::errors::ReapersProgramError;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use serde_json::Value;
use serde_json::json;
use solana_kit_core::Address;
use solana_kit_core::Hash;
use solana_kit_core::Signature;
use solana_kit_core::crypto::verify;
use solana_kit_core::transaction::Lifetime;
use solana_kit_core::transaction::Message;
use solana_kit_core::transaction::Transaction;
use solana_kit_core::transaction::Version;

use crate::backend::ChainStatus;
use crate::platform;
use crate::platform::HttpTransport;
use crate::program;
use crate::receipt::Receipt;
use crate::simnet::SimnetAccount;
use crate::simnet::SimnetInstruction;
use crate::simnet::SimnetRole;
use crate::simnet::SimnetTransaction;
use crate::wallet::WalletProver;

/// Why something on the cluster did not happen. Each one has words for the
/// player (`lookups::chain_failure_message`); the detail is for a log.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ChainFailure {
	/// This device can reach no wallet, or none is linked.
	#[error("no wallet")]
	NoWallet,
	/// The wallet was asked and said no, or was closed.
	#[error("the wallet declined")]
	Declined,
	/// The wallet signed as another account than the one linked.
	#[error("the wallet signed as another account")]
	WrongWallet,
	/// The Reapers program refused the transaction.
	#[error("the program refused: {0}")]
	Program(ReapersProgramError),
	/// The cluster refused the transaction for another reason.
	#[error("the cluster refused the transaction: {0}")]
	Rejected(String),
	/// The cluster could not be reached, or answered nonsense.
	#[error("the cluster is unreachable: {0}")]
	Unreachable(String),
	/// The transaction was sent and not finalized in time. It may still
	/// land: the husks are read again before anything is repeated.
	#[error("the transaction did not finalize in time: {0}")]
	NotFinalized(String),
}

pub type ChainResult<T> = Result<T, ChainFailure>;

/// Which cluster a build plays on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterName {
	/// Solana's devnet.
	Devnet,
	/// A validator on the developer's machine, such as Surfpool.
	Localnet,
}

impl ClusterName {
	/// The name a build flag gives it.
	pub fn from_name(name: &str) -> Option<Self> {
		match name {
			"devnet" => Some(Self::Devnet),
			"localnet" => Some(Self::Localnet),
			_ => None,
		}
	}

	/// The chain the server and the wallet call it.
	pub const fn chain(self) -> &'static str {
		match self {
			Self::Devnet => "solana:devnet",
			Self::Localnet => "solana:localnet",
		}
	}

	/// The public RPC a build uses when it names none.
	pub const fn default_rpc(self) -> &'static str {
		match self {
			Self::Devnet => "https://api.devnet.solana.com",
			Self::Localnet => "http://127.0.0.1:8899",
		}
	}
}

/// Where a husk of the demo collections is, as the cluster has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuskPlace {
	/// The wallet owns it: it can be released.
	InWallet,
	/// The wallet released it into the program's escrow.
	Escrowed,
	/// Core's one-byte tombstone: the asset was burned (on-chain tombstones
	/// record no prior owner or burner).
	Burned,
}

/// One of the demo collections' husks, as the cluster has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalletHusk {
	/// The husk of the rules it stands for.
	pub nft: Nft,
	/// Its Core asset.
	pub asset: Address,
	/// Its Core collection.
	pub collection: Address,
	pub place: HuskPlace,
}

/// The demo husk `nft` stands for, by the server's manifest.
pub fn husk_of(husks: &[Husks], nft: Nft) -> Option<(Address, Address)> {
	let husks = husks
		.iter()
		.find(|husks| husks.collection == nft.collection.name())?;
	let asset = husks.assets.get(nft.serial.checked_sub(1)?)?.parse().ok()?;
	Some((asset, husks.address.parse().ok()?))
}

/// Refuses `collection` when it is the reaper collection `status` names.
///
/// A reaper is never a demo husk, and one the wallet escrows leaves the
/// escrow only by being burned, so the wallet is never asked to release it.
pub(crate) fn reject_reaper_collection(
	status: &ChainStatus,
	collection: Address,
) -> ChainResult<()> {
	if status
		.collection
		.as_deref()
		.and_then(|collection| collection.parse::<Address>().ok())
		== Some(collection)
	{
		return Err(ChainFailure::Rejected(format!(
			"{collection} is the reaper collection, not a demo collection"
		)));
	}
	Ok(())
}

/// Waits for a duration; `false` where the host has no timer.
pub type Pause = Rc<dyn Fn(Duration) -> LocalBoxFuture<'static, bool>>;

/// Who is told of each transaction that lands.
type Listeners = RefCell<Vec<Box<dyn Fn(&SimnetTransaction)>>>;

/// Who is told of a failure nobody waited for.
type FailureListeners = RefCell<Vec<Box<dyn Fn(&ChainFailure)>>>;

/// A cluster with the Reapers program on it.
pub struct Cluster {
	pub name: ClusterName,
	rpc: ClusterRpc,
	pause: Pause,
	/// How long a transaction may take to finalize.
	patience: Duration,
	listeners: Listeners,
	failures: FailureListeners,
}

impl Cluster {
	/// How often a sent transaction's status is read.
	const POLL: Duration = Duration::from_millis(500);

	/// The cluster `name` at the RPC `url`, reached through `http`.
	pub fn new(name: ClusterName, url: &str, http: Rc<dyn HttpTransport>) -> Self {
		Self {
			name,
			rpc: ClusterRpc {
				url: url.to_owned(),
				http,
				next_id: RefCell::new(0),
			},
			pause: Rc::new(|duration| Box::pin(platform::sleep(duration))),
			patience: Duration::from_secs(90),
			listeners: RefCell::new(Vec::new()),
			failures: RefCell::new(Vec::new()),
		}
	}

	/// The same cluster, waiting with `pause` between status reads.
	#[must_use]
	pub fn with_pause(mut self, pause: Pause) -> Self {
		self.pause = pause;
		self
	}

	/// The RPC's URL.
	pub fn url(&self) -> &str {
		&self.rpc.url
	}

	/// Whether the server's reported cluster and Reapers program match this
	/// build's cluster and program ID.
	pub fn matches(&self, status: &ChainStatus) -> bool {
		status.chain == self.name.chain()
			&& status
				.program
				.as_deref()
				.and_then(|program| program.parse::<Address>().ok())
				== Some(program::program_id())
	}

	/// Ensures the server's reported cluster and Reapers program match this
	/// build before touching the cluster or prompting the wallet.
	pub fn check_status(&self, status: &ChainStatus) -> ChainResult<()> {
		if status.chain != self.name.chain() {
			return Err(ChainFailure::Rejected(format!(
				"server chain {} does not match client cluster {}",
				status.chain,
				self.name.chain()
			)));
		}
		let expected = program::program_id();
		let actual = status
			.program
			.as_deref()
			.and_then(|program| program.parse::<Address>().ok());
		if actual != Some(expected) {
			return Err(ChainFailure::Rejected(format!(
				"server program {:?} does not match client program {expected}",
				status.program
			)));
		}
		Ok(())
	}

	/// Calls `listener` with each transaction as it lands.
	pub fn subscribe(&self, listener: impl Fn(&SimnetTransaction) + 'static) {
		self.listeners.borrow_mut().push(Box::new(listener));
	}

	/// Calls `listener` with each failure that happened after the request
	/// that caused it had already succeeded: a burn after a settled hunt.
	pub fn on_failure(&self, listener: impl Fn(&ChainFailure) + 'static) {
		self.failures.borrow_mut().push(Box::new(listener));
	}

	/// Tells whoever listens that `failure` happened.
	pub fn fail(&self, failure: &ChainFailure) {
		for listener in self.failures.borrow().iter() {
			listener(failure);
		}
	}

	/// Every demo husk the server's manifest names that `wallet` owns or has
	/// in escrow, plus any tombstoned manifest asset (reported as
	/// [`HuskPlace::Burned`], because on-chain Core tombstones do not record
	/// who burned them), read at a confirmed slot.
	pub async fn wallet_husks(
		&self,
		husks: &[Husks],
		wallet: &Address,
	) -> ChainResult<Vec<WalletHusk>> {
		let mut known = Vec::new();
		for collection in husks {
			let Some(rules) = NftCollection::from_name(&collection.collection) else {
				continue;
			};
			let Ok(address) = collection.address.parse::<Address>() else {
				continue;
			};
			for (index, asset) in collection.assets.iter().enumerate() {
				if let Ok(asset) = asset.parse::<Address>() {
					known.push((Nft::new(rules, index + 1), asset, address));
				}
			}
		}
		if known.is_empty() {
			return Ok(Vec::new());
		}
		let count = known.len();
		let mut addresses = Vec::with_capacity(count * 2);
		for (_, asset, _) in &known {
			addresses.push(*asset);
		}
		for (_, asset, _) in &known {
			addresses.push(program::escrow(asset));
		}
		let mut fetched = self.rpc.accounts(&addresses).await?;
		let escrow_accounts = fetched.split_off(count);

		Ok(known
			.into_iter()
			.zip(fetched.into_iter().zip(escrow_accounts))
			.filter_map(|((nft, asset, collection), (account, escrow))| {
				let place = place_of(
					account.as_ref(),
					escrow.as_ref(),
					wallet,
					&asset,
					&collection,
				)?;
				Some(WalletHusk {
					nft,
					asset,
					collection,
					place,
				})
			})
			.collect())
	}

	/// Reads a single demo husk (`nft`) and its escrow in one 2-account RPC.
	pub async fn wallet_husk(
		&self,
		husks: &[Husks],
		wallet: &Address,
		nft: Nft,
	) -> ChainResult<Option<WalletHusk>> {
		let Some((asset, collection)) = husk_of(husks, nft) else {
			return Ok(None);
		};
		let escrow = program::escrow(&asset);
		let accounts = self.rpc.accounts(&[asset, escrow]).await?;
		let place = place_of(
			accounts.first().and_then(Option::as_ref),
			accounts.get(1).and_then(Option::as_ref),
			wallet,
			&asset,
			&collection,
		);
		Ok(place.map(|place| {
			WalletHusk {
				nft,
				asset,
				collection,
				place,
			}
		}))
	}

	/// Releases `husk` from `wallet` into the program's escrow with
	/// `ReleaseCore`, signed by `prover`, and waits until it is finalized.
	///
	/// A husk of the reaper collection `status` names is refused: that is
	/// never a demo collection, and a reaper in escrow leaves it only by
	/// being burned.
	pub async fn release(
		&self,
		status: &ChainStatus,
		prover: &dyn WalletProver,
		wallet: &Address,
		husk: &WalletHusk,
	) -> ChainResult<SimnetTransaction> {
		self.check_status(status)?;
		reject_reaper_collection(status, husk.collection)?;
		let instruction = program::release_core(wallet, &husk.asset, Some(&husk.collection));
		let (blockhash, last_valid_block_height) = self.rpc.latest_blockhash().await?;
		let message = Message::compile(
			Version::Legacy,
			*wallet,
			Lifetime::Blockhash {
				blockhash,
				last_valid_block_height,
			},
			&[instruction],
			&[],
		)
		.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
		let unsigned = Transaction::new(message)
			.encode(false)
			.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
		let (signature, wire) = self
			.sign_and_send(status, prover, wallet, unsigned, 0)
			.await?;
		let slot = self.settle(&signature, Commitment::Finalized, 0).await?;
		let transaction = SimnetTransaction {
			signature,
			slot,
			at: platform::now(),
			instruction: SimnetInstruction::Escrow,
			accounts: vec![
				SimnetAccount::new(SimnetRole::Wallet, wallet),
				SimnetAccount::new(SimnetRole::Mint, &husk.asset),
				SimnetAccount::new(SimnetRole::Escrow, &program::escrow(&husk.asset)),
				SimnetAccount::new(SimnetRole::Program, &program::program_id()),
			],
			lamports: 0,
			nft: Some(husk.nft),
			receipt: None,
			receipt_signature: None,
			wire: Some(wire),
		};
		self.tell(&transaction);
		Ok(transaction)
	}

	/// Signs and sends the absorb transaction the server built for `nft`
	/// (base64 `transaction`, carrying `receipt` signed by `keeper`), and
	/// checks the asset is burned.
	#[allow(
		clippy::too_many_arguments,
		reason = "everything a receipt names, as the server handed it"
	)]
	pub async fn absorb(
		&self,
		status: &ChainStatus,
		prover: &dyn WalletProver,
		wallet: &Address,
		nft: Nft,
		transaction: &str,
		receipt: &[u8],
		receipt_signature: &[u8],
		keeper: &str,
	) -> ChainResult<SimnetTransaction> {
		self.check_status(status)?;
		let (unsigned, mint) = verify_absorb_transaction(
			status,
			wallet,
			nft,
			transaction,
			receipt,
			receipt_signature,
			keeper,
		)?;
		// The check of the receipt comes first, then the burn.
		let (signature, wire) = self
			.sign_and_send(status, prover, wallet, unsigned, 1)
			.await?;
		let slot = self.settle(&signature, Commitment::Confirmed, 1).await?;
		let burned = self.rpc.accounts(&[mint]).await?;
		if !matches!(burned.first(), Some(Some(account)) if account.data == [0]) {
			return Err(ChainFailure::Rejected(format!(
				"{mint} is not burned after {signature}"
			)));
		}
		let transaction = SimnetTransaction {
			signature,
			slot,
			at: platform::now(),
			instruction: SimnetInstruction::Burn,
			accounts: vec![
				SimnetAccount::new(SimnetRole::Wallet, wallet),
				SimnetAccount::new(SimnetRole::Mint, &mint),
				SimnetAccount::new(SimnetRole::Escrow, &program::escrow(&mint)),
				SimnetAccount::new(SimnetRole::Program, &program::program_id()),
				SimnetAccount::new(SimnetRole::Keeper, &keeper),
			],
			lamports: 0,
			nft: Some(nft),
			receipt: Some(receipt.to_vec()),
			receipt_signature: Some(receipt_signature.to_vec()),
			wire: Some(wire),
		};
		self.tell(&transaction);
		Ok(transaction)
	}

	fn tell(&self, transaction: &SimnetTransaction) {
		for listener in self.listeners.borrow().iter() {
			listener(transaction);
		}
	}

	/// Has `prover` sign the wallet's slot of `unsigned`, checks it signed
	/// exactly that as `wallet`, and sends it. `program_index` is where the
	/// Reapers program's instruction is, to read its errors by.
	async fn sign_and_send(
		&self,
		status: &ChainStatus,
		prover: &dyn WalletProver,
		wallet: &Address,
		unsigned: Vec<u8>,
		program_index: usize,
	) -> ChainResult<(String, Vec<u8>)> {
		let expected = Transaction::decode(&unsigned)
			.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
		if expected.message().signers().first() != Some(wallet) {
			return Err(ChainFailure::WrongWallet);
		}
		let signed = prover.sign_transaction(status, unsigned).await?;
		let transaction = Transaction::decode(&signed)
			.ok()
			.filter(|signed| signed.message() == expected.message() && signed.verify().is_ok())
			.ok_or(ChainFailure::WrongWallet)?;
		let signature = transaction
			.signatures()
			.first()
			.copied()
			.flatten()
			.ok_or(ChainFailure::WrongWallet)?
			.to_string();
		self.rpc
			.send(&signed)
			.await
			.map_err(|failure| program_failure(failure, program_index))?;
		Ok((signature, signed))
	}

	/// Waits until `signature` reaches `commitment`, and returns its slot.
	async fn settle(
		&self,
		signature: &str,
		commitment: Commitment,
		program_index: usize,
	) -> ChainResult<i64> {
		let attempts = (self.patience.as_millis() / Self::POLL.as_millis()).max(1);
		for _ in 0..attempts {
			if let Some((slot, reached, error)) = self.rpc.status(signature).await? {
				if let Some(error) = error {
					return Err(program_failure(
						ChainFailure::Rejected(error.to_string()),
						program_index,
					));
				}
				if reached >= commitment {
					return Ok(slot);
				}
			}
			if !(self.pause)(Self::POLL).await {
				break;
			}
		}
		Err(ChainFailure::NotFinalized(signature.to_owned()))
	}
}

/// Verified fields extracted from an absorption receipt and its signature.
struct VerifiedReceipt {
	receipt: Receipt,
	mint: Address,
	keeper: Address,
	signature: [u8; 64],
	/// The Core collection the manifest names for the husk.
	collection: Address,
}

/// Decodes and cryptographically verifies an absorption receipt against
/// `wallet`, `status.husks`, and `keeper`.
///
/// The server must name the wallet the burn is for: a status with none is
/// refused rather than read as a guest's, since only an escrowed husk of a
/// linked wallet is ever burned.
///
/// The husk must be one the manifest maps: the game escrows only those, so
/// a burn of any other was never asked for, and nothing would bind the
/// receipt's mint or the collection the burn writes to.
fn verify_absorb_receipt(
	status: &ChainStatus,
	wallet: &Address,
	nft: Nft,
	receipt: &[u8],
	receipt_signature: &[u8],
	keeper: &str,
) -> ChainResult<VerifiedReceipt> {
	let Some(linked) = status.wallet.as_deref() else {
		return Err(ChainFailure::WrongWallet);
	};
	if linked.parse::<Address>().ok() != Some(*wallet) {
		return Err(ChainFailure::WrongWallet);
	}
	let decoded =
		Receipt::decode(receipt).map_err(|error| ChainFailure::Rejected(error.to_string()))?;
	if decoded.player != *wallet.as_bytes() {
		return Err(ChainFailure::Rejected(
			"absorb receipt player does not match linked wallet".into(),
		));
	}
	let mint = Address::from_bytes(decoded.mint);
	let Some((expected_mint, collection)) = husk_of(&status.husks, nft) else {
		return Err(ChainFailure::Rejected(format!(
			"{nft} is not a husk of the server's manifest"
		)));
	};
	if mint != expected_mint {
		return Err(ChainFailure::Rejected(format!(
			"absorb receipt mint {mint} does not match manifest asset {expected_mint} for {nft}"
		)));
	}
	let keeper_addr: Address = keeper
		.parse()
		.map_err(|_| ChainFailure::Rejected("invalid keeper address".into()))?;
	let sig_bytes: [u8; 64] = receipt_signature
		.try_into()
		.map_err(|_| ChainFailure::Rejected("receipt signature must be 64 bytes".into()))?;
	verify(
		&keeper_addr,
		&decoded.encode(),
		&Signature::from_bytes(sig_bytes),
	)
	.map_err(|_| ChainFailure::Rejected("receipt signature verification failed".into()))?;
	Ok(VerifiedReceipt {
		receipt: decoded,
		mint,
		keeper: keeper_addr,
		signature: sig_bytes,
		collection,
	})
}

/// Verifies a server-supplied absorb transaction and receipt before asking the
/// wallet to sign.
///
/// Rebuilds the expected `[Ed25519SigVerify, AbsorbCore]` legacy message
/// locally, with the manifest's collection, and refuses unless the server's
/// message matches byte for byte and the keeper's Ed25519 signature over the
/// receipt verifies.
fn verify_absorb_transaction(
	status: &ChainStatus,
	wallet: &Address,
	nft: Nft,
	transaction: &str,
	receipt: &[u8],
	receipt_signature: &[u8],
	keeper: &str,
) -> ChainResult<(Vec<u8>, Address)> {
	let verified_receipt =
		verify_absorb_receipt(status, wallet, nft, receipt, receipt_signature, keeper)?;

	let unsigned = BASE64
		.decode(transaction)
		.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
	let server_tx = Transaction::decode(&unsigned)
		.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
	if server_tx.signatures() != [None] {
		return Err(ChainFailure::Rejected(
			"absorb transaction must carry one unsigned signature slot".into(),
		));
	}
	let server_msg = server_tx.message();
	if server_msg.version() != Version::Legacy || !server_msg.lookups().is_empty() {
		return Err(ChainFailure::Rejected(
			"absorb transaction must be a legacy message".into(),
		));
	}
	if server_msg.signers() != [*wallet] || server_msg.header().readonly_signers != 0 {
		return Err(ChainFailure::Rejected(
			"absorb transaction must be signed solely by the linked wallet".into(),
		));
	}
	if server_msg.instructions().len() != 2 {
		return Err(ChainFailure::Rejected(
			"absorb transaction must contain exactly two instructions".into(),
		));
	}

	let expected_instructions = program::absorb_core(
		&verified_receipt.keeper,
		&verified_receipt.signature,
		&verified_receipt.receipt,
		Some(&verified_receipt.collection),
	);
	let expected_message = Message::compile(
		Version::Legacy,
		*wallet,
		Lifetime::Blockhash {
			blockhash: server_msg.lifetime().token(),
			last_valid_block_height: 0,
		},
		&expected_instructions,
		&[],
	)
	.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
	if server_msg.bytes() != expected_message.bytes() {
		return Err(ChainFailure::Rejected(
			"absorb transaction did not match expected Ed25519 + AbsorbCore instructions".into(),
		));
	}

	let verified = Transaction::new(expected_message)
		.encode(false)
		.map_err(|error| ChainFailure::Rejected(format!("{error:?}")))?;
	Ok((verified, verified_receipt.mint))
}

/// A refused transaction's failure, as the program's error when it was the
/// program's instruction at `program_index` that refused.
fn program_failure(failure: ChainFailure, program_index: usize) -> ChainFailure {
	match &failure {
		ChainFailure::Rejected(detail) => {
			serde_json::from_str::<Value>(detail)
				.ok()
				.and_then(|error| custom_code(&error))
				.filter(|(index, _)| *index == program_index)
				.and_then(|(_, code)| ReapersProgramError::from_u32(code))
				.map_or(failure, ChainFailure::Program)
		}
		_ => failure,
	}
}

/// `{"InstructionError": [index, {"Custom": code}]}`, as a cluster reports a
/// program's error.
fn custom_code(error: &Value) -> Option<(usize, u32)> {
	let pair = error.get("InstructionError")?.as_array()?;
	let index = usize::try_from(pair.first()?.as_u64()?).ok()?;
	let code = u32::try_from(pair.get(1)?.get("Custom")?.as_u64()?).ok()?;
	Some((index, code))
}

/// Where a husk is: its asset account and its escrow's, against `wallet`.
/// An asset that is not a member of `collection`, the collection the
/// manifest names with it, is no husk at all.
///
/// Any one-byte Core tombstone is reported as [`HuskPlace::Burned`] regardless
/// of `wallet`, since Metaplex Core tombstones store no prior owner or burner.
fn place_of(
	asset: Option<&AccountData>,
	escrow: Option<&AccountData>,
	wallet: &Address,
	mint: &Address,
	collection: &Address,
) -> Option<HuskPlace> {
	/// `UpdateAuthority::Collection`: the asset is a member of the
	/// collection whose address follows.
	const MEMBER_OF: u8 = 2;

	let asset = asset.filter(|asset| asset.owner == program::core_program())?;
	if asset.data == [0] {
		// Core tombstones keep no prior owner; any burned manifest asset maps here.
		return Some(HuskPlace::Burned);
	}
	// An `AssetV1`: its key, its owner, then its update authority.
	if asset.data.first() != Some(&1) {
		return None;
	}
	let owner = Address::from_bytes(asset.data.get(1..33)?.try_into().ok()?);
	if asset.data.get(33) != Some(&MEMBER_OF) || asset.data.get(34..66)? != collection.as_bytes() {
		return None;
	}
	if owner == *wallet {
		return Some(HuskPlace::InWallet);
	}
	let escrow = escrow.filter(|escrow| escrow.owner == program::program_id())?;
	let recorded = Escrow::from_bytes(&escrow.data).ok()?;
	(owner == program::escrow(mint)
		&& recorded.player.to_bytes() == *wallet.as_bytes()
		&& recorded.mint.to_bytes() == *mint.as_bytes())
	.then_some(HuskPlace::Escrowed)
}

/// How settled a transaction is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Commitment {
	Processed,
	Confirmed,
	Finalized,
}

/// One account, as the RPC returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct AccountData {
	owner: Address,
	data: Vec<u8>,
}

/// The cluster's JSON-RPC: the five standard methods the game needs.
struct ClusterRpc {
	url: String,
	http: Rc<dyn HttpTransport>,
	next_id: RefCell<u64>,
}

impl ClusterRpc {
	const TIMEOUT: Duration = Duration::from_secs(20);

	/// Each account of `addresses`, `None` where there is none.
	async fn accounts(&self, addresses: &[Address]) -> ChainResult<Vec<Option<AccountData>>> {
		let mut found = Vec::with_capacity(addresses.len());
		for chunk in addresses.chunks(100) {
			let keys: Vec<String> = chunk.iter().map(ToString::to_string).collect();
			let result = self
				.call(
					"getMultipleAccounts",
					json!([keys, { "encoding": "base64", "commitment": "confirmed" }]),
				)
				.await?;
			let values = result
				.get("value")
				.and_then(Value::as_array)
				.filter(|values| values.len() == chunk.len())
				.ok_or_else(|| malformed("getMultipleAccounts"))?;
			for value in values {
				found.push(if value.is_null() {
					None
				} else {
					let owner = value
						.get("owner")
						.and_then(Value::as_str)
						.and_then(|owner| owner.parse().ok())
						.ok_or_else(|| malformed("getMultipleAccounts"))?;
					let data = value
						.get("data")
						.and_then(|data| data.get(0))
						.and_then(Value::as_str)
						.and_then(|data| BASE64.decode(data).ok())
						.ok_or_else(|| malformed("getMultipleAccounts"))?;
					Some(AccountData { owner, data })
				});
			}
		}
		Ok(found)
	}

	async fn latest_blockhash(&self) -> ChainResult<(Hash, u64)> {
		let result = self
			.call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))
			.await?;
		let value = result
			.get("value")
			.ok_or_else(|| malformed("getLatestBlockhash"))?;
		let blockhash = value
			.get("blockhash")
			.and_then(Value::as_str)
			.and_then(|hash| hash.parse().ok())
			.ok_or_else(|| malformed("getLatestBlockhash"))?;
		let height = value
			.get("lastValidBlockHeight")
			.and_then(Value::as_u64)
			.ok_or_else(|| malformed("getLatestBlockhash"))?;
		Ok((blockhash, height))
	}

	/// Sends a signed transaction, simulating it first, so a program's
	/// refusal comes back before anything lands.
	async fn send(&self, wire: &[u8]) -> ChainResult<String> {
		let result = self
			.call(
				"sendTransaction",
				json!([BASE64.encode(wire), {
					"encoding": "base64",
					"preflightCommitment": "confirmed",
				}]),
			)
			.await?;
		result
			.as_str()
			.map(str::to_owned)
			.ok_or_else(|| malformed("sendTransaction"))
	}

	/// `signature`'s slot, how settled it is and its error, if the cluster
	/// has seen it.
	async fn status(
		&self,
		signature: &str,
	) -> ChainResult<Option<(i64, Commitment, Option<Value>)>> {
		let result = self
			.call(
				"getSignatureStatuses",
				json!([[signature], { "searchTransactionHistory": true }]),
			)
			.await?;
		let Some(status) = result
			.get("value")
			.and_then(|values| values.get(0))
			.filter(|status| !status.is_null())
		else {
			return Ok(None);
		};
		let slot = status
			.get("slot")
			.and_then(Value::as_i64)
			.ok_or_else(|| malformed("getSignatureStatuses"))?;
		let commitment = match status.get("confirmationStatus").and_then(Value::as_str) {
			Some("finalized") => Commitment::Finalized,
			Some("confirmed") => Commitment::Confirmed,
			_ => Commitment::Processed,
		};
		let error = status.get("err").filter(|error| !error.is_null()).cloned();
		Ok(Some((slot, commitment, error)))
	}

	async fn call(&self, method: &str, params: Value) -> ChainResult<Value> {
		let id = {
			let mut next = self.next_id.borrow_mut();
			*next += 1;
			*next
		};
		let body = serde_json::to_vec(
			&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
		)
		.map_err(|error| ChainFailure::Unreachable(error.to_string()))?;
		let reply = self
			.http
			.post_json(&self.url, body, Self::TIMEOUT)
			.await
			.map_err(|error| ChainFailure::Unreachable(format!("{method}: {error}")))?;
		if reply.status != 200 {
			return Err(ChainFailure::Unreachable(format!(
				"{method}: HTTP {}",
				reply.status
			)));
		}
		let answer: Value = serde_json::from_slice(&reply.body).map_err(|_| malformed(method))?;
		if let Some(error) = answer.get("error").filter(|error| !error.is_null()) {
			// A transaction the cluster simulated and refused carries the
			// instruction's error, which the caller reads as the program's.
			if let Some(failed) = error.get("data").and_then(|data| data.get("err")) {
				return Err(ChainFailure::Rejected(failed.to_string()));
			}
			return Err(ChainFailure::Rejected(format!("{method}: {error}")));
		}
		answer
			.get("result")
			.cloned()
			.ok_or_else(|| malformed(method))
	}
}

fn malformed(method: &str) -> ChainFailure {
	ChainFailure::Unreachable(format!("{method}: not the answer asked for"))
}

#[cfg(test)]
mod tests {
	use solana_kit_core::AccountRole;

	use super::*;

	#[test]
	fn a_program_error_is_read_from_its_instruction() {
		let error = json!({ "InstructionError": [1, { "Custom": 8 }] });
		assert_eq!(
			program_failure(ChainFailure::Rejected(error.to_string()), 1),
			ChainFailure::Program(ReapersProgramError::WrongKeeper)
		);
		assert_eq!(
			program_failure(ChainFailure::Rejected(error.to_string()), 0),
			ChainFailure::Rejected(error.to_string()),
			"another instruction's custom error is not the program's"
		);
	}

	#[test]
	fn a_husk_is_where_its_asset_and_escrow_say() {
		let wallet = Address::from_bytes([1; 32]);
		let mint = Address::from_bytes([2; 32]);
		let collection = Address::from_bytes([9; 32]);
		// An `AssetV1` owned by `owner`, with the update authority `authority`.
		let asset_of = |owner: &Address, authority: &[u8]| {
			let mut data = vec![1];
			data.extend_from_slice(owner.as_bytes());
			data.extend_from_slice(authority);
			AccountData {
				owner: program::core_program(),
				data,
			}
		};
		let member_of = |collection: &Address| [&[2], collection.as_bytes().as_slice()].concat();
		let asset = |owner: &Address| asset_of(owner, &member_of(&collection));
		let mut escrow = vec![0; Escrow::LEN];
		Escrow::initialize(&mut escrow, |account| {
			account.player = solana_pubkey::Pubkey::new_from_array([1; 32]);
			account.mint = solana_pubkey::Pubkey::new_from_array([2; 32]);
			account.kind = 1;
		})
		.unwrap();
		let escrow = AccountData {
			owner: program::program_id(),
			data: escrow,
		};

		assert_eq!(
			place_of(Some(&asset(&wallet)), None, &wallet, &mint, &collection),
			Some(HuskPlace::InWallet)
		);
		assert_eq!(
			place_of(
				Some(&asset(&program::escrow(&mint))),
				Some(&escrow),
				&wallet,
				&mint,
				&collection
			),
			Some(HuskPlace::Escrowed)
		);
		assert_eq!(
			place_of(
				Some(&asset(&program::escrow(&mint))),
				Some(&escrow),
				&Address::from_bytes([3; 32]),
				&mint,
				&collection
			),
			None,
			"another player's escrow"
		);
		assert_eq!(
			place_of(
				Some(&AccountData {
					owner: program::core_program(),
					data: vec![0],
				}),
				None,
				&wallet,
				&mint,
				&collection
			),
			Some(HuskPlace::Burned)
		);
		for (authority, what) in [
			(
				member_of(&Address::from_bytes([77; 32])),
				"a member of another collection",
			),
			(
				[&[1], collection.as_bytes().as_slice()].concat(),
				"an asset whose update authority is a plain address",
			),
			(vec![0], "an asset with no update authority"),
		] {
			assert_eq!(
				place_of(
					Some(&asset_of(&wallet, &authority)),
					None,
					&wallet,
					&mint,
					&collection
				),
				None,
				"{what} is no husk of the manifest's collection"
			);
		}
	}

	#[test]
	fn the_manifest_names_each_husks_asset() {
		let husks = vec![Husks {
			collection: "thugbirdz".into(),
			address: Address::from_bytes([9; 32]).to_string(),
			assets: (1..=8)
				.map(|n| Address::from_bytes([n; 32]).to_string())
				.collect(),
		}];
		assert_eq!(
			husk_of(&husks, Nft::new(NftCollection::Thugbirdz, 2)),
			Some((Address::from_bytes([2; 32]), Address::from_bytes([9; 32])))
		);
		assert_eq!(husk_of(&husks, Nft::new(NftCollection::Sollamas, 2)), None);
	}

	struct RecordingRpc {
		calls: RefCell<Vec<String>>,
	}

	#[async_trait::async_trait(?Send)]
	impl HttpTransport for RecordingRpc {
		async fn post(
			&self,
			_url: &str,
			_headers: Vec<platform::Header>,
			body: Vec<u8>,
			_timeout: Duration,
		) -> Result<platform::HttpReply, platform::PlatformError> {
			let request: Value = serde_json::from_slice(&body).unwrap();
			let method = request["method"].as_str().unwrap().to_owned();
			self.calls.borrow_mut().push(method.clone());
			let tombstone = BASE64.encode([0]);
			let result = match method.as_str() {
				"sendTransaction" => json!("drained"),
				"getSignatureStatuses" => {
					json!({ "value": [{ "slot": 9, "confirmationStatus": "confirmed", "err": null }] })
				}
				"getMultipleAccounts" => {
					json!({ "value": [{
						"owner": program::core_program().to_string(),
						"data": [tombstone, "base64"],
					}] })
				}
				other => panic!("unexpected RPC method {other}"),
			};
			Ok(platform::HttpReply {
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
		signed: std::cell::Cell<u32>,
	}

	#[async_trait::async_trait(?Send)]
	impl WalletProver for CountingWallet {
		fn available(&self) -> bool {
			true
		}

		async fn prove(
			&self,
			_status: &ChainStatus,
			_challenge: crate::wallet::ChallengeFn<'_>,
		) -> crate::backend::KeeperResult<crate::wallet::WalletProof> {
			unimplemented!()
		}

		async fn sign_transaction(
			&self,
			_status: &ChainStatus,
			transaction: Vec<u8>,
		) -> ChainResult<Vec<u8>> {
			self.signed.set(self.signed.get() + 1);
			Transaction::decode(&transaction)
				.and_then(|tx| tx.sign(&self.keypair))
				.and_then(|tx| tx.encode(true))
				.map_err(|_| ChainFailure::WrongWallet)
		}
	}

	fn sample_receipt(wallet: &Address, mint: &Address) -> Receipt {
		Receipt {
			player: *wallet.as_bytes(),
			mint: *mint.as_bytes(),
			grade: reapers_receipt::Grade::from_byte(1).unwrap(),
			feeding: reapers_receipt::Feeding::from_byte(1).unwrap(),
			xp: 120,
			anima: 30,
			at: 1_790_000_000,
			nonce: 7,
		}
	}

	/// The collection of the saint, `degenApes:1`.
	fn saint_collection() -> Address {
		Address::from_bytes([9; 32])
	}

	/// A manifest that maps the saint, `degenApes:1`, to the asset `[2; 32]`.
	fn saint_manifest() -> Vec<Husks> {
		vec![Husks {
			collection: "degenApes".into(),
			address: saint_collection().to_string(),
			assets: vec![Address::from_bytes([2; 32]).to_string()],
		}]
	}

	fn devnet_status(wallet: &Address, husks: Vec<Husks>) -> ChainStatus {
		ChainStatus {
			chain: "solana:devnet".into(),
			identity_uri: "https://reapers.example/".into(),
			wallet: Some(wallet.to_string()),
			lamports: None,
			collection: None,
			marketplace_uri: None,
			program: Some(program::program_id().to_string()),
			husks,
		}
	}

	fn encode_tx(
		version: Version,
		payer: Address,
		instructions: &[solana_kit_core::Instruction],
	) -> String {
		let message = Message::compile(
			version,
			payer,
			Lifetime::Blockhash {
				blockhash: Hash::from_bytes([3; 32]),
				last_valid_block_height: 100,
			},
			instructions,
			&[],
		)
		.unwrap();
		BASE64.encode(Transaction::new(message).encode(false).unwrap())
	}

	#[test]
	fn absorb_refuses_opaque_non_absorb_instructions_from_server() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::codecs::Limits;
		use solana_kit_core::crypto::Keypair;
		use solana_kit_system::TransferSol;
		use solana_kit_system::TransferSolAccounts;

		let wallet = Keypair::from_seed(&[6; 32]);
		let attacker = Address::from_bytes([99; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);

		// A compromised server substitutes a SOL drain for Ed25519 + AbsorbCore.
		let drain_ix = TransferSol {
			amount: 1_000_000_000,
		}
		.instruction(
			TransferSolAccounts {
				source: wallet.address(),
				destination: attacker,
			},
			Limits::new(1232, 64),
		)
		.unwrap();
		let malicious_tx = encode_tx(Version::Legacy, wallet.address(), &[drain_ix]);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = devnet_status(&prover.keypair.address(), saint_manifest());

		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&malicious_tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(
			result.is_err() && prover.signed.get() == 0 && rpc.calls.borrow().is_empty(),
			"Cluster::absorb must verify Ed25519(receipt) + AbsorbCore before asking the wallet to sign (signed={}, rpc={:?})",
			prover.signed.get(),
			rpc.calls.borrow()
		);
	}

	#[test]
	fn absorb_refuses_appended_transfer_instruction() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::codecs::Limits;
		use solana_kit_core::crypto::Keypair;
		use solana_kit_system::TransferSol;
		use solana_kit_system::TransferSolAccounts;

		let wallet = Keypair::from_seed(&[6; 32]);
		let attacker = Address::from_bytes([99; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);

		let [verify_ix, absorb_ix] = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		let drain_ix = TransferSol {
			amount: 5_000_000_000,
		}
		.instruction(
			TransferSolAccounts {
				source: wallet.address(),
				destination: attacker,
			},
			Limits::new(1232, 64),
		)
		.unwrap();
		let tx = encode_tx(
			Version::Legacy,
			wallet.address(),
			&[verify_ix, absorb_ix, drain_ix],
		);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = devnet_status(&prover.keypair.address(), saint_manifest());

		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(result.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	#[test]
	fn absorb_refuses_tampered_absorb_core_account_or_role() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);

		// Case 1: tampered account address (config PDA replaced).
		let [verify_ix, mut bad_account] = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		bad_account.accounts[1].address = Address::from_bytes([88; 32]);
		let bad_account_tx =
			encode_tx(Version::Legacy, wallet.address(), &[verify_ix, bad_account]);

		// Case 2: tampered account role (readonly config promoted to writable).
		let [verify_ix, mut bad_role] = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		bad_role.accounts[1].role = AccountRole::Writable;
		let bad_role_tx = encode_tx(Version::Legacy, wallet.address(), &[verify_ix, bad_role]);

		// Case 3: the manifest's collection, which the burn writes, demoted to
		// readonly.
		let [verify_ix, mut readonly_collection] = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		readonly_collection.accounts[4].role = AccountRole::Readonly;
		let readonly_collection_tx = encode_tx(
			Version::Legacy,
			wallet.address(),
			&[verify_ix, readonly_collection],
		);

		for tx in [bad_account_tx, bad_role_tx, readonly_collection_tx] {
			let rpc = Rc::new(RecordingRpc {
				calls: RefCell::new(Vec::new()),
			});
			let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
				.with_pause(Rc::new(|_| Box::pin(async { true })));
			let prover = CountingWallet {
				keypair: Keypair::from_seed(&[6; 32]),
				signed: Cell::new(0),
			};
			let status = devnet_status(&prover.keypair.address(), saint_manifest());
			let result = block_on(cluster.absorb(
				&status,
				&prover,
				&prover.keypair.address(),
				Nft::parse("degenApes:1").unwrap(),
				&tx,
				&receipt_bytes,
				receipt_sig.as_bytes(),
				&keeper.address().to_string(),
			));
			assert!(result.is_err());
			assert_eq!(prover.signed.get(), 0);
			assert!(rpc.calls.borrow().is_empty());
		}
	}

	#[test]
	fn absorb_refuses_forged_receipt_signature() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let imposter = Keypair::from_seed(&[7; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let forged_sig = imposter.sign(&receipt_bytes);

		let ixs = program::absorb_core(
			&keeper.address(),
			forged_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		let tx = encode_tx(Version::Legacy, wallet.address(), &ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = devnet_status(&prover.keypair.address(), saint_manifest());

		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&tx,
			&receipt_bytes,
			forged_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(result.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	#[test]
	fn absorb_refuses_mismatched_player_or_mint() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let other_player = Address::from_bytes([42; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let manifest_mint = Address::from_bytes([2; 32]);
		let wrong_mint = Address::from_bytes([77; 32]);
		let collection = Address::from_bytes([9; 32]);
		let manifest = vec![Husks {
			collection: "degenApes".into(),
			address: collection.to_string(),
			assets: vec![manifest_mint.to_string()],
		}];

		// Case 1: receipt.player != linked wallet.
		let wrong_player_receipt = sample_receipt(&other_player, &manifest_mint);
		let wrong_player_bytes = wrong_player_receipt.encode();
		let wrong_player_sig = keeper.sign(&wrong_player_bytes);
		let wrong_player_ixs = program::absorb_core(
			&keeper.address(),
			wrong_player_sig.as_bytes(),
			&wrong_player_receipt,
			Some(&collection),
		);
		let wrong_player_tx = encode_tx(Version::Legacy, other_player, &wrong_player_ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: Keypair::from_seed(&[6; 32]),
			signed: Cell::new(0),
		};
		let status = devnet_status(&wallet.address(), manifest.clone());
		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&wallet.address(),
			Nft::parse("degenApes:1").unwrap(),
			&wrong_player_tx,
			&wrong_player_bytes,
			wrong_player_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(result.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());

		// Case 2: receipt.mint != manifest asset for degenApes:1.
		let wrong_mint_receipt = sample_receipt(&wallet.address(), &wrong_mint);
		let wrong_mint_bytes = wrong_mint_receipt.encode();
		let wrong_mint_sig = keeper.sign(&wrong_mint_bytes);
		let wrong_mint_ixs = program::absorb_core(
			&keeper.address(),
			wrong_mint_sig.as_bytes(),
			&wrong_mint_receipt,
			Some(&collection),
		);
		let wrong_mint_tx = encode_tx(Version::Legacy, wallet.address(), &wrong_mint_ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&wrong_mint_tx,
			&wrong_mint_bytes,
			wrong_mint_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(result.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	/// An absorb is always for the wallet the server linked: a status that
	/// names none is refused before the transaction is read, rather than
	/// letting a direct caller burn under a guest's status.
	#[test]
	fn absorb_refuses_a_status_with_no_linked_wallet() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);
		let ixs = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		let tx = encode_tx(Version::Legacy, wallet.address(), &ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = ChainStatus {
			wallet: None,
			..devnet_status(&prover.keypair.address(), saint_manifest())
		};

		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert_eq!(result, Err(ChainFailure::WrongWallet));
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	/// A receipt the keeper signed for another player is refused for naming
	/// that player, before the transaction is read: here in transactions the
	/// linked wallet pays for, one with the other player as a second signer
	/// and one with the wallet in the player's slot of `AbsorbCore`.
	#[test]
	fn absorb_refuses_a_receipt_for_another_player_in_a_transaction_the_wallet_pays() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let other_player = Address::from_bytes([42; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let receipt = sample_receipt(&other_player, &Address::from_bytes([2; 32]));
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);
		let co_signed = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		let mut wallet_as_player = co_signed.clone();
		wallet_as_player[1].accounts[0].address = wallet.address();

		for ixs in [co_signed, wallet_as_player] {
			let tx = encode_tx(Version::Legacy, wallet.address(), &ixs);
			let rpc = Rc::new(RecordingRpc {
				calls: RefCell::new(Vec::new()),
			});
			let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
				.with_pause(Rc::new(|_| Box::pin(async { true })));
			let prover = CountingWallet {
				keypair: Keypair::from_seed(&[6; 32]),
				signed: Cell::new(0),
			};
			let status = devnet_status(&wallet.address(), saint_manifest());
			let result = block_on(cluster.absorb(
				&status,
				&prover,
				&wallet.address(),
				Nft::parse("degenApes:1").unwrap(),
				&tx,
				&receipt_bytes,
				receipt_sig.as_bytes(),
				&keeper.address().to_string(),
			));
			assert_eq!(prover.signed.get(), 0);
			assert!(rpc.calls.borrow().is_empty());
			assert!(
				matches!(&result, Err(ChainFailure::Rejected(detail)) if detail.contains("receipt player")),
				"refused for another reason than the receipt's player: {result:?}"
			);
		}
	}

	#[test]
	fn absorb_refuses_v0_message() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);

		let ixs = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&saint_collection()),
		);
		let v0_tx = encode_tx(Version::V0, wallet.address(), &ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = devnet_status(&prover.keypair.address(), saint_manifest());

		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::parse("degenApes:1").unwrap(),
			&v0_tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(result.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());
	}

	#[test]
	fn absorb_accepts_verified_transaction_and_enforces_manifest_collection() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let mint = Address::from_bytes([2; 32]);
		let collection = Address::from_bytes([9; 32]);
		let wrong_collection = Address::from_bytes([10; 32]);
		let manifest = vec![Husks {
			collection: "degenApes".into(),
			address: collection.to_string(),
			assets: vec![mint.to_string()],
		}];
		let receipt = sample_receipt(&wallet.address(), &mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);
		let status = devnet_status(&wallet.address(), manifest);

		// Mismatched collection vs manifest must be rejected.
		let bad_ixs = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&wrong_collection),
		);
		let bad_tx = encode_tx(Version::Legacy, wallet.address(), &bad_ixs);
		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: Keypair::from_seed(&[6; 32]),
			signed: Cell::new(0),
		};
		let rejected = block_on(cluster.absorb(
			&status,
			&prover,
			&wallet.address(),
			Nft::parse("degenApes:1").unwrap(),
			&bad_tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(rejected.is_err());
		assert_eq!(prover.signed.get(), 0);
		assert!(rpc.calls.borrow().is_empty());

		// Matching manifest collection passes verification, signs once, and sends.
		let good_ixs = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&collection),
		);
		let good_tx = encode_tx(Version::Legacy, wallet.address(), &good_ixs);
		let accepted = block_on(cluster.absorb(
			&status,
			&prover,
			&wallet.address(),
			Nft::parse("degenApes:1").unwrap(),
			&good_tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert!(accepted.is_ok());
		assert_eq!(prover.signed.get(), 1);
		assert_eq!(
			*rpc.calls.borrow(),
			[
				"sendTransaction",
				"getSignatureStatuses",
				"getMultipleAccounts"
			]
		);
	}

	/// The game escrows only husks the manifest maps, so a burn for any
	/// other husk is never one it asked for. Here the server says a Sollamas
	/// husk (absent from the manifest) was absorbed, with a receipt for this
	/// player's escrowed Epic `degenApes:8` and a collection of its own
	/// choosing: the wallet must not be asked to burn it.
	#[test]
	fn absorb_refuses_a_husk_the_manifest_does_not_map() {
		use std::cell::Cell;

		use futures::executor::block_on;
		use solana_kit_core::crypto::Keypair;

		let wallet = Keypair::from_seed(&[6; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let manifest = vec![Husks {
			collection: "degenApes".into(),
			address: Address::from_bytes([9; 32]).to_string(),
			assets: (1..=8_u8)
				.map(|n| Address::from_bytes([n; 32]).to_string())
				.collect(),
		}];
		let epic_mint = Address::from_bytes([8; 32]);
		let receipt = sample_receipt(&wallet.address(), &epic_mint);
		let receipt_bytes = receipt.encode();
		let receipt_sig = keeper.sign(&receipt_bytes);
		let chosen_collection = Address::from_bytes([77; 32]);
		let ixs = program::absorb_core(
			&keeper.address(),
			receipt_sig.as_bytes(),
			&receipt,
			Some(&chosen_collection),
		);
		let tx = encode_tx(Version::Legacy, wallet.address(), &ixs);

		let rpc = Rc::new(RecordingRpc {
			calls: RefCell::new(Vec::new()),
		});
		let cluster = Cluster::new(ClusterName::Devnet, "http://rpc", rpc.clone())
			.with_pause(Rc::new(|_| Box::pin(async { true })));
		let prover = CountingWallet {
			keypair: wallet,
			signed: Cell::new(0),
		};
		let status = devnet_status(&prover.keypair.address(), manifest);
		let result = block_on(cluster.absorb(
			&status,
			&prover,
			&prover.keypair.address(),
			Nft::new(NftCollection::Sollamas, 1),
			&tx,
			&receipt_bytes,
			receipt_sig.as_bytes(),
			&keeper.address().to_string(),
		));
		assert_eq!(
			prover.signed.get(),
			0,
			"the wallet was asked to burn {epic_mint} (degenApes:8) for a Sollamas husk the manifest does not name; rpc={:?}",
			rpc.calls.borrow()
		);
		assert!(rpc.calls.borrow().is_empty());
		assert!(matches!(result, Err(ChainFailure::Rejected(_))));
	}
}
