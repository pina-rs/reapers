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
	/// It was absorbed: Core's one-byte tombstone.
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

	/// Every demo husk the server's manifest names that `wallet` owns, has
	/// in escrow, or has burned, read at a confirmed slot.
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
		let assets: Vec<Address> = known.iter().map(|(_, asset, _)| *asset).collect();
		let escrows: Vec<Address> = assets.iter().map(program::escrow).collect();
		let accounts = self.rpc.accounts(&assets).await?;
		let escrow_accounts = self.rpc.accounts(&escrows).await?;

		Ok(known
			.into_iter()
			.zip(accounts.into_iter().zip(escrow_accounts))
			.filter_map(|((nft, asset, collection), (account, escrow))| {
				let place = place_of(account.as_ref(), escrow.as_ref(), wallet, &asset)?;
				Some(WalletHusk {
					nft,
					asset,
					collection,
					place,
				})
			})
			.collect())
	}

	/// Releases `husk` from `wallet` into the program's escrow with
	/// `ReleaseCore`, signed by `prover`, and waits until it is finalized.
	pub async fn release(
		&self,
		status: &ChainStatus,
		prover: &dyn WalletProver,
		wallet: &Address,
		husk: &WalletHusk,
	) -> ChainResult<SimnetTransaction> {
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
		let unsigned = BASE64
			.decode(transaction)
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
		let mint = Receipt::decode(receipt)
			.map(|receipt| Address::from_bytes(receipt.mint))
			.map_err(|error| ChainFailure::Rejected(error.to_string()))?;
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
			(self.pause)(Self::POLL).await;
		}
		Err(ChainFailure::NotFinalized(signature.to_owned()))
	}
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
fn place_of(
	asset: Option<&AccountData>,
	escrow: Option<&AccountData>,
	wallet: &Address,
	mint: &Address,
) -> Option<HuskPlace> {
	let asset = asset.filter(|asset| asset.owner == program::core_program())?;
	if asset.data == [0] {
		return Some(HuskPlace::Burned);
	}
	// An `AssetV1`: its key, then its owner.
	if asset.data.first() != Some(&1) {
		return None;
	}
	let owner = Address::from_bytes(asset.data.get(1..33)?.try_into().ok()?);
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
		let asset = |owner: &Address| {
			let mut data = vec![1];
			data.extend_from_slice(owner.as_bytes());
			data.push(0);
			AccountData {
				owner: program::core_program(),
				data,
			}
		};
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
			place_of(Some(&asset(&wallet)), None, &wallet, &mint),
			Some(HuskPlace::InWallet)
		);
		assert_eq!(
			place_of(
				Some(&asset(&program::escrow(&mint))),
				Some(&escrow),
				&wallet,
				&mint
			),
			Some(HuskPlace::Escrowed)
		);
		assert_eq!(
			place_of(
				Some(&asset(&program::escrow(&mint))),
				Some(&escrow),
				&Address::from_bytes([3; 32]),
				&mint
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
				&mint
			),
			Some(HuskPlace::Burned)
		);
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
}
