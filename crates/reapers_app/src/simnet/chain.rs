//! A pretend devnet that lives on the device, for showing the game where no
//! cluster has the Reapers program.
//!
//! It holds a burner wallet (a real Ed25519 key, made here, that never holds
//! anything of value), the program's addresses, and a ledger of every
//! transaction the wallet has signed. It decides nothing about the game:
//! amounts and NFTs are handed to it by whoever keeps the books.
//!
//! Every transaction but the airdrop is a real Solana wire transaction,
//! compiled and signed with Solana Kit: a System transfer to the market for
//! a purchase, a Memo for each step, and on a burn an Ed25519 program
//! instruction that makes the cluster itself check the keeper's receipt.
//! Offline its blockhash is chained from the previous transaction, so no
//! cluster would accept it as it stands; [`SimnetChain::rebuild`] signs the
//! same instructions again over a real blockhash for a validator that can
//! take them. The airdrop has no transaction: a cluster's faucet pays it.
//!
//! The wallet's seed is kept in plain preferences. It is a simnet key and
//! must never be used anywhere real.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use reapers_rules::Nft;
use reapers_rules::Timestamp;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest as _;
use sha2::Sha256;
use solana_kit_core::AccountMeta;
use solana_kit_core::Address;
use solana_kit_core::Hash;
use solana_kit_core::Instruction;
use solana_kit_core::Signature;
use solana_kit_core::address_constants::ED25519_PROGRAM;
use solana_kit_core::codecs::Limits;
use solana_kit_core::crypto::Keypair;
use solana_kit_core::crypto::find_program_address;
use solana_kit_core::crypto::verify;
use solana_kit_core::transaction::Lifetime;
use solana_kit_core::transaction::Message;
use solana_kit_core::transaction::Transaction;
use solana_kit_core::transaction::Version;
use solana_kit_system::TransferSol;
use solana_kit_system::TransferSolAccounts;

use crate::backend::KeeperError;
use crate::backend::KeeperResult;
use crate::platform;
use crate::platform::Prefs;

/// What a simnet transaction does. These are the four things the chain is
/// meant to do for Reapers (see docs/nfts.md); everything else in the game
/// stays off-chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimnetInstruction {
	/// The faucet drops the starting SOL into a new wallet.
	Airdrop,
	/// The wallet pays the market and receives an NFT.
	Buy,
	/// The wallet moves an NFT into the program's escrow: the husk is loose.
	Escrow,
	/// The program burns an escrowed NFT, carrying the keeper's receipt.
	Burn,
}

impl SimnetInstruction {
	pub const fn name(self) -> &'static str {
		match self {
			Self::Airdrop => "airdrop",
			Self::Buy => "buy",
			Self::Escrow => "escrow",
			Self::Burn => "burn",
		}
	}
}

/// What an account does in a transaction, as an explorer would label it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimnetRole {
	Wallet,
	Faucet,
	Market,
	Mint,
	Escrow,
	Program,
	Keeper,
}

impl SimnetRole {
	pub const fn name(self) -> &'static str {
		match self {
			Self::Wallet => "wallet",
			Self::Faucet => "faucet",
			Self::Market => "market",
			Self::Mint => "mint",
			Self::Escrow => "escrow",
			Self::Program => "program",
			Self::Keeper => "keeper",
		}
	}
}

/// One account a transaction touched.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimnetAccount {
	pub role: SimnetRole,
	pub address: String,
}

impl SimnetAccount {
	pub fn new(role: SimnetRole, address: impl ToString) -> Self {
		Self {
			role,
			address: address.to_string(),
		}
	}
}

/// What to post: the instruction and everything it moves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimnetPost {
	pub instruction: SimnetInstruction,
	pub accounts: Vec<SimnetAccount>,
	/// SOL moved, in lamports.
	pub lamports: i64,
	pub nft: Option<Nft>,
	/// For a burn: the 98-byte receipt and the keeper's signature over it.
	pub receipt: Option<Vec<u8>>,
	pub receipt_signature: Option<Vec<u8>>,
}

impl SimnetPost {
	pub fn new(instruction: SimnetInstruction, accounts: Vec<SimnetAccount>) -> Self {
		Self {
			instruction,
			accounts,
			lamports: 0,
			nft: None,
			receipt: None,
			receipt_signature: None,
		}
	}
}

/// A transaction on the simnet, as the ledger keeps it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimnetTransaction {
	/// The wallet's signature, in base58, as an explorer shows it.
	pub signature: String,
	pub slot: i64,
	#[serde(with = "iso8601")]
	pub at: Timestamp,
	pub instruction: SimnetInstruction,
	pub accounts: Vec<SimnetAccount>,
	/// SOL moved, in lamports. Zero for a transaction that moves only an NFT.
	#[serde(default)]
	pub lamports: i64,
	/// The NFT the transaction moved or burned.
	#[serde(default, with = "nft_id")]
	pub nft: Option<Nft>,
	/// For a burn: the 98-byte receipt the burn carries, in base64.
	#[serde(default, with = "bytes")]
	pub receipt: Option<Vec<u8>>,
	/// For a burn: the keeper's Ed25519 signature over the receipt.
	#[serde(default, with = "bytes")]
	pub receipt_signature: Option<Vec<u8>>,
	/// The signed Solana wire transaction, in base64. The airdrop has none.
	#[serde(default, with = "bytes")]
	pub wire: Option<Vec<u8>>,
}

impl SimnetTransaction {
	/// The address that plays `role`, if any account does.
	pub fn account(&self, role: SimnetRole) -> Option<&str> {
		self.accounts
			.iter()
			.find(|account| account.role == role)
			.map(|account| account.address.as_str())
	}

	/// Whether the receipt this burn carries is the keeper's: the check the
	/// Ed25519 program makes before the burn is allowed.
	pub fn receipt_verifies(&self) -> bool {
		let (Some(receipt), Some(signature), Some(keeper)) = (
			&self.receipt,
			&self.receipt_signature,
			self.account(SimnetRole::Keeper),
		) else {
			return false;
		};
		let (Ok(keeper), Ok(signature)) = (
			keeper.parse::<Address>(),
			Signature::try_from(signature.as_slice()),
		) else {
			return false;
		};
		verify(&keeper, receipt, &signature).is_ok()
	}

	/// Whether the wire transaction is well formed and every signature in it
	/// verifies.
	pub fn wire_verifies(&self) -> bool {
		self.wire
			.as_deref()
			.and_then(|wire| Transaction::decode(wire).ok())
			.is_some_and(|transaction| transaction.verify().is_ok())
	}
}

/// The stored simnet: the wallet's seed, when the chain began, and what has
/// happened on it.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Genesis {
	#[serde(with = "seed")]
	secret: [u8; 32],
	#[serde(with = "iso8601")]
	genesis: Timestamp,
	transactions: Vec<SimnetTransaction>,
}

/// The pretend devnet.
pub struct SimnetChain {
	prefs: Rc<dyn Prefs>,
	clock: Rc<dyn Fn() -> Timestamp>,
	latency: Duration,
	state: RefCell<Genesis>,
	listeners: RefCell<Vec<Box<dyn Fn(&SimnetTransaction)>>>,
}

/// The name an address is made from, so every device's simnet agrees on it.
fn named(name: &str) -> Address {
	Address::from_bytes(Sha256::digest(format!("reapers-simnet:{name}").as_bytes()).into())
}

/// The keeper that signs absorption receipts on the simnet. Its key is made
/// from a public name, so its receipts prove only that the layout and the
/// check work, which is what a rehearsal needs.
fn keeper_key() -> Keypair {
	Keypair::from_seed(&Sha256::digest(b"reapers-simnet:keeper").into())
}

const LIMITS: Limits = Limits::new(1232, 64);

impl SimnetChain {
	/// How long a transaction takes to confirm, as one sent to a cluster
	/// would.
	pub const LATENCY: Duration = Duration::from_millis(650);
	/// How long a slot lasts, as on Solana's clusters.
	pub const SLOT: Duration = Duration::from_millis(400);
	/// Where the simnet is stored: the Flutter game's key.
	pub const STORAGE_KEY: &'static str = "reapers.simnet.v1";

	/// The program, the market and the faucet: no key exists for any of them.
	pub fn program() -> Address {
		named("program")
	}

	pub fn market() -> Address {
		named("market")
	}

	pub fn faucet() -> Address {
		named("faucet")
	}

	pub fn keeper() -> Address {
		keeper_key().address()
	}

	/// Opens the simnet stored in `prefs`, making a new one with a fresh
	/// wallet if there is none.
	pub async fn open(
		prefs: Rc<dyn Prefs>,
		clock: Rc<dyn Fn() -> Timestamp>,
		latency: Duration,
	) -> KeeperResult<Self> {
		let stored = prefs.get(Self::STORAGE_KEY).await?;
		let fresh = stored.is_none();
		let state = match stored {
			Some(value) => serde_json::from_value(value).map_err(KeeperError::unavailable)?,
			None => {
				Genesis {
					secret: platform::random_bytes().await?,
					genesis: clock(),
					transactions: Vec::new(),
				}
			}
		};
		let chain = Self {
			prefs,
			clock,
			latency,
			state: RefCell::new(state),
			listeners: RefCell::new(Vec::new()),
		};
		if fresh {
			chain.save().await?;
		}
		Ok(chain)
	}

	fn wallet_key(&self) -> Keypair {
		Keypair::from_seed(&self.state.borrow().secret)
	}

	/// The burner wallet's address.
	pub fn wallet(&self) -> String {
		self.wallet_key().address().to_string()
	}

	/// Every transaction, oldest first.
	pub fn transactions(&self) -> Vec<SimnetTransaction> {
		self.state.borrow().transactions.clone()
	}

	/// The slot the chain is at now.
	pub fn slot(&self) -> i64 {
		self.slot_at((self.clock)())
	}

	fn slot_at(&self, at: Timestamp) -> i64 {
		let elapsed = at.micros_since(self.state.borrow().genesis) / 1_000;
		1 + elapsed / Self::SLOT.as_millis() as i64
	}

	/// Calls `listener` with each transaction as it is confirmed.
	pub fn subscribe(&self, listener: impl Fn(&SimnetTransaction) + 'static) {
		self.listeners.borrow_mut().push(Box::new(listener));
	}

	/// Where `nft` waits while it is loose: the program's escrow for its mint.
	pub fn escrow_for(nft: Nft) -> Address {
		let mint: [u8; 32] = Sha256::digest(nft.mock_mint().as_bytes()).into();
		find_program_address(&[b"escrow", &mint], &Self::program())
			.map(|pda| pda.address())
			.expect("a 32-byte seed always has a viable bump")
	}

	/// Signs `receipt` as the simnet keeper.
	pub fn sign_receipt(receipt: &[u8]) -> Vec<u8> {
		keeper_key().sign(receipt).to_bytes().to_vec()
	}

	/// Signs and records a transaction, and returns it once it is confirmed.
	pub async fn submit(&self, post: SimnetPost) -> KeeperResult<SimnetTransaction> {
		platform::sleep(self.latency).await;
		let at = (self.clock)();
		let slot = self.slot_at(at);
		let previous = self
			.state
			.borrow()
			.transactions
			.last()
			.map_or_else(|| self.wallet(), |last| last.signature.clone());
		let blockhash =
			Hash::from_bytes(Sha256::digest(format!("{previous}:{slot}").as_bytes()).into());

		let (signature, wire) = match self.instructions(&post) {
			Some(instructions) => {
				let transaction = self.sign(&instructions, blockhash)?;
				let signature = transaction
					.signatures()
					.first()
					.copied()
					.flatten()
					.ok_or_else(|| KeeperError::unavailable("an unsigned transaction"))?;
				(
					signature,
					Some(
						transaction
							.encode(true)
							.map_err(|error| KeeperError::unavailable(format!("{error:?}")))?,
					),
				)
			}
			// The airdrop is the faucet's: the wallet signs a description of
			// it, so the ledger still has a signature to show.
			None => {
				(
					self.wallet_key().sign(&self.description(&post, blockhash)),
					None,
				)
			}
		};

		let transaction = SimnetTransaction {
			signature: signature.to_string(),
			slot,
			at,
			instruction: post.instruction,
			accounts: post.accounts,
			lamports: post.lamports,
			nft: post.nft,
			receipt: post.receipt,
			receipt_signature: post.receipt_signature,
			wire,
		};
		self.state
			.borrow_mut()
			.transactions
			.push(transaction.clone());
		self.save().await?;
		for listener in self.listeners.borrow().iter() {
			listener(&transaction);
		}
		Ok(transaction)
	}

	/// The same transaction, signed again over a real `blockhash`, for a
	/// validator that can take it. `None` for the airdrop.
	pub fn rebuild(
		&self,
		transaction: &SimnetTransaction,
		blockhash: Hash,
	) -> KeeperResult<Option<Vec<u8>>> {
		let post = SimnetPost {
			instruction: transaction.instruction,
			accounts: transaction.accounts.clone(),
			lamports: transaction.lamports,
			nft: transaction.nft,
			receipt: transaction.receipt.clone(),
			receipt_signature: transaction.receipt_signature.clone(),
		};
		let Some(instructions) = self.instructions(&post) else {
			return Ok(None);
		};
		let signed = self.sign(&instructions, blockhash)?;
		signed
			.encode(true)
			.map(Some)
			.map_err(|error| KeeperError::unavailable(format!("{error:?}")))
	}

	/// Forgets every transaction and makes a new wallet: a new player at the
	/// next table.
	pub async fn reset(&self) -> KeeperResult<()> {
		let secret = platform::random_bytes().await?;
		*self.state.borrow_mut() = Genesis {
			secret,
			genesis: (self.clock)(),
			transactions: Vec::new(),
		};
		self.save().await
	}

	async fn save(&self) -> KeeperResult<()> {
		let value =
			serde_json::to_value(&*self.state.borrow()).map_err(KeeperError::unavailable)?;
		Ok(self.prefs.set(Self::STORAGE_KEY, value).await?)
	}

	fn sign(&self, instructions: &[Instruction], blockhash: Hash) -> KeeperResult<Transaction> {
		let wallet = self.wallet_key();
		let message = Message::compile(
			Version::Legacy,
			wallet.address(),
			Lifetime::Blockhash {
				blockhash,
				last_valid_block_height: u64::MAX,
			},
			instructions,
			&[],
		)
		.map_err(|error| KeeperError::unavailable(format!("{error:?}")))?;
		Transaction::new(message)
			.sign(&wallet)
			.map_err(|error| KeeperError::unavailable(format!("{error:?}")))
	}

	/// The Solana instructions for a post: `None` for the airdrop, which no
	/// wallet signs.
	fn instructions(&self, post: &SimnetPost) -> Option<Vec<Instruction>> {
		let wallet = self.wallet_key().address();
		let memo = |text: String| solana_kit_memo::add_memo(&text, &[wallet], LIMITS).ok();
		let mint = post.nft.map(|nft| nft.mock_mint()).unwrap_or_default();

		match post.instruction {
			SimnetInstruction::Airdrop => None,
			SimnetInstruction::Buy => {
				Some(vec![
					TransferSol {
						amount: post.lamports.max(0) as u64,
					}
					.instruction(
						TransferSolAccounts {
							source: wallet,
							destination: Self::market(),
						},
						LIMITS,
					)
					.ok()?,
					memo(format!("reapers-simnet/1 buy {mint}"))?,
				])
			}
			SimnetInstruction::Escrow => {
				Some(vec![memo(format!(
					"reapers-simnet/1 escrow {mint} into {}",
					post.nft.map(Self::escrow_for)?
				))?])
			}
			SimnetInstruction::Burn => {
				let receipt = post.receipt.as_deref()?;
				let signature = post.receipt_signature.as_deref()?;
				Some(vec![
					ed25519_verify(&Self::keeper(), signature, receipt)?,
					memo(format!("reapers-simnet/1 burn {mint}"))?,
				])
			}
		}
	}

	/// What the airdrop's signature is over: the instruction, its accounts,
	/// the amount and the chained blockhash.
	fn description(&self, post: &SimnetPost, blockhash: Hash) -> Vec<u8> {
		let mut lines = vec![
			"reapers-simnet/1".to_owned(),
			post.instruction.name().to_owned(),
			blockhash.to_string(),
		];
		lines.extend(
			post.accounts
				.iter()
				.map(|account| format!("{}={}", account.role.name(), account.address)),
		);
		lines.push(post.lamports.to_string());
		lines.push(post.nft.map(|nft| nft.mock_mint()).unwrap_or_default());
		lines.join("\n").into_bytes()
	}
}

/// An Ed25519 program instruction that checks `signature` by `signer` over
/// `message` before anything after it in the transaction runs: the check a
/// Reapers program would require of a burn's receipt.
fn ed25519_verify(signer: &Address, signature: &[u8], message: &[u8]) -> Option<Instruction> {
	const OFFSETS: u16 = 2;
	const KEY: u16 = OFFSETS + 14;
	const SIGNATURE: u16 = KEY + 32;
	const MESSAGE: u16 = SIGNATURE + 64;
	const THIS_INSTRUCTION: u16 = u16::MAX;

	if signature.len() != 64 {
		return None;
	}
	let mut data = vec![1, 0];
	for field in [
		SIGNATURE,
		THIS_INSTRUCTION,
		KEY,
		THIS_INSTRUCTION,
		MESSAGE,
		u16::try_from(message.len()).ok()?,
		THIS_INSTRUCTION,
	] {
		data.extend_from_slice(&field.to_le_bytes());
	}
	data.extend_from_slice(signer.as_bytes());
	data.extend_from_slice(signature);
	data.extend_from_slice(message);

	Some(Instruction {
		program_address: ED25519_PROGRAM,
		accounts: Vec::<AccountMeta>::new(),
		data,
	})
}

mod iso8601 {
	use reapers_rules::Timestamp;
	use serde::Deserialize as _;

	pub fn serialize<S: serde::Serializer>(
		at: &Timestamp,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&at.to_iso8601())
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> Result<Timestamp, D::Error> {
		let text = String::deserialize(deserializer)?;
		Timestamp::parse(&text).map_err(serde::de::Error::custom)
	}
}

mod nft_id {
	use reapers_rules::Nft;
	use serde::Deserialize as _;

	#[allow(
		clippy::ref_option,
		reason = "serde's `with` hands the field by reference"
	)]
	pub fn serialize<S: serde::Serializer>(
		nft: &Option<Nft>,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		match nft {
			Some(nft) => serializer.serialize_some(&nft.id()),
			None => serializer.serialize_none(),
		}
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> Result<Option<Nft>, D::Error> {
		Option::<String>::deserialize(deserializer)?
			.map(|id| Nft::parse(&id).map_err(serde::de::Error::custom))
			.transpose()
	}
}

mod bytes {
	use base64::Engine as _;
	use serde::Deserialize as _;

	use super::BASE64;

	#[allow(
		clippy::ref_option,
		reason = "serde's `with` hands the field by reference"
	)]
	pub fn serialize<S: serde::Serializer>(
		bytes: &Option<Vec<u8>>,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		match bytes {
			Some(bytes) => serializer.serialize_some(&BASE64.encode(bytes)),
			None => serializer.serialize_none(),
		}
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> Result<Option<Vec<u8>>, D::Error> {
		Option::<String>::deserialize(deserializer)?
			.map(|text| BASE64.decode(text).map_err(serde::de::Error::custom))
			.transpose()
	}
}

mod seed {
	use base64::Engine as _;
	use serde::Deserialize as _;

	use super::BASE64;

	pub fn serialize<S: serde::Serializer>(
		secret: &[u8; 32],
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&BASE64.encode(secret))
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> Result<[u8; 32], D::Error> {
		let text = String::deserialize(deserializer)?;
		BASE64
			.decode(text)
			.map_err(serde::de::Error::custom)?
			.try_into()
			.map_err(|_| serde::de::Error::custom("a wallet seed is 32 bytes"))
	}
}
