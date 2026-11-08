//! Talking to a cluster: keypair files, sending a transaction and waiting for
//! it, and reading accounts. Everything goes through Solana Kit.

use std::path::Path;
use std::time::Duration;

use solana_kit_client::Client;
use solana_kit_client::Limits;
use solana_kit_client::Presence;
use solana_kit_client::methods::Balance;
use solana_kit_client::methods::Commitment;
use solana_kit_client::methods::LatestBlockhash;
use solana_kit_client::methods::ReadConfig;
use solana_kit_client::methods::history::ConfirmationStatus;
use solana_kit_client::methods::submission::RequestAirdrop;
use solana_kit_client::methods::submission::SendConfig;
use solana_kit_client::methods::submission::SendTransaction;
use solana_kit_client::methods::submission::SignatureStatuses;
use solana_kit_client::methods::submission::SignedWire;
use solana_kit_client::methods::submission::WireEncoding;
use solana_kit_core::AccountMeta;
use solana_kit_core::AccountRole;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_kit_core::Signature;
use solana_kit_core::accounts::MaybeAccount;
use solana_kit_core::crypto::Keypair;
use solana_kit_core::transaction::Lifetime;
use solana_kit_core::transaction::Message;
use solana_kit_core::transaction::Transaction;
use solana_kit_core::transaction::Version;
use solana_kit_native::HttpConfig;
use solana_kit_native::NativeHttp;

/// A failure, already worded for the operator.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// The compute budget program.
const COMPUTE_BUDGET: Address = Address::from_bytes([
	3, 6, 70, 111, 229, 33, 23, 50, 255, 236, 173, 186, 114, 195, 155, 231, 188, 140, 229, 187,
	197, 247, 18, 107, 44, 67, 155, 58, 64, 0, 0, 0,
]);

/// A cluster, reached over JSON-RPC.
pub struct Chain {
	client: Client<NativeHttp>,
	/// Micro-lamports per compute unit offered on every transaction.
	pub priority_fee: u64,
	/// The URL, for messages.
	pub url: String,
}

impl Chain {
	pub fn new(url: &str, priority_fee: u64) -> Result<Self, Error> {
		let config = HttpConfig {
			exchange_timeout: Duration::from_secs(60),
			..HttpConfig::default()
		};
		let transport = NativeHttp::new(url, config)?;

		Ok(Self {
			client: Client::new(transport, Limits::default()),
			priority_fee,
			url: url.to_owned(),
		})
	}

	/// Sends `instructions` paid by the first signer, with a compute limit of
	/// `units` and the priority fee, and waits until the cluster confirms it.
	pub async fn send(
		&self,
		instructions: &[Instruction],
		signers: &[&Keypair],
		units: u32,
	) -> Result<Signature, Error> {
		let payer = signers.first().ok_or("a transaction needs a payer")?;
		let mut all = vec![set_compute_unit_limit(units)];
		if self.priority_fee > 0 {
			all.push(set_compute_unit_price(self.priority_fee));
		}
		all.extend_from_slice(instructions);

		let latest = self
			.client
			.call(LatestBlockhash(Some(confirmed())))
			.await?
			.value;
		let message = Message::compile(
			Version::Legacy,
			payer.address(),
			Lifetime::Blockhash {
				blockhash: latest.blockhash,
				last_valid_block_height: latest.last_valid_block_height,
			},
			&all,
			&[],
		)?;
		let mut transaction = Transaction::new(message);
		for signer in signers {
			transaction = transaction.sign(signer)?;
		}
		let wire = transaction.encode(true)?;
		let signature = self
			.client
			.call(SendTransaction {
				wire: SignedWire::new(&wire)?,
				encoding: WireEncoding::Base64,
				config: SendConfig {
					preflight_commitment: Some(Commitment::Confirmed),
					..SendConfig::default()
				},
			})
			.await
			.map_err(explain)?;

		self.confirm(signature).await?;

		Ok(signature)
	}

	/// Waits up to a minute and a half for `signature` to be confirmed, and
	/// fails if it executed with an error.
	pub async fn confirm(&self, signature: Signature) -> Result<(), Error> {
		for _ in 0..180 {
			let statuses = self
				.client
				.call(SignatureStatuses(&[signature], None))
				.await?
				.value;

			if let Some(Some(status)) = statuses.first() {
				if let Some(error) = &status.err {
					return Err(format!("transaction {signature} failed: {error:?}").into());
				}
				if matches!(
					status.confirmation_status,
					Some(ConfirmationStatus::Confirmed | ConfirmationStatus::Finalized)
				) {
					return Ok(());
				}
			}

			tokio::time::sleep(Duration::from_millis(500)).await;
		}

		Err(format!("transaction {signature} was not confirmed in 90 seconds").into())
	}

	/// The account at `address`, or `None` if there is none.
	pub async fn account(
		&self,
		address: Address,
	) -> Result<Option<solana_kit_core::accounts::Account<Vec<u8>>>, Error> {
		let account = self
			.client
			.fetch_encoded_account(address, confirmed())
			.await?
			.value;

		Ok(match account {
			MaybeAccount::Present(account) => Some(account),
			MaybeAccount::Missing(_) => None,
		})
	}

	pub async fn balance(&self, address: Address) -> Result<u64, Error> {
		Ok(self
			.client
			.call(Balance(address, Some(confirmed())))
			.await?
			.value)
	}

	/// Asks the cluster's faucet for `lamports`, and waits for them.
	pub async fn airdrop(&self, address: Address, lamports: u64) -> Result<(), Error> {
		let signature = self
			.client
			.call(RequestAirdrop(address, lamports, None))
			.await?;

		self.confirm(signature).await
	}
}

/// A refused transaction, with the node's message and, when the message does
/// not already carry them, the simulation's logs.
fn explain(error: solana_kit_client::Error) -> Error {
	let solana_kit_client::Error::Remote(remote) = error else {
		return error.into();
	};
	let mut text = remote.message.clone();

	if !text.contains("log messages")
		&& let Presence::Present(data) = &remote.data
		&& let Ok(data) = serde_json::from_str::<serde_json::Value>(data.get())
	{
		let logs = data["logs"].as_array().cloned().unwrap_or_default();
		for line in logs.iter().filter_map(serde_json::Value::as_str) {
			text.push('\n');
			text.push_str(line);
		}
	}

	text.into()
}

fn confirmed() -> ReadConfig {
	ReadConfig {
		commitment: Some(Commitment::Confirmed),
		min_context_slot: None,
	}
}

fn set_compute_unit_limit(units: u32) -> Instruction {
	let mut data = vec![2];
	data.extend_from_slice(&units.to_le_bytes());
	Instruction {
		program_address: COMPUTE_BUDGET,
		accounts: Vec::new(),
		data,
	}
}

fn set_compute_unit_price(micro_lamports: u64) -> Instruction {
	let mut data = vec![3];
	data.extend_from_slice(&micro_lamports.to_le_bytes());
	Instruction {
		program_address: COMPUTE_BUDGET,
		accounts: Vec::new(),
		data,
	}
}

/// A Solana Kit instruction from one the generated client built.
pub fn kit(instruction: solana_instruction::Instruction) -> Instruction {
	Instruction {
		program_address: address(&instruction.program_id),
		accounts: instruction
			.accounts
			.iter()
			.map(|meta| {
				AccountMeta {
					address: address(&meta.pubkey),
					role: AccountRole::from_privileges(meta.is_signer, meta.is_writable),
				}
			})
			.collect(),
		data: instruction.data,
	}
}

pub fn address(pubkey: &solana_pubkey::Pubkey) -> Address {
	Address::from_bytes(pubkey.to_bytes())
}

pub fn pubkey(address: &Address) -> solana_pubkey::Pubkey {
	solana_pubkey::Pubkey::new_from_array(address.to_bytes())
}

/// Reads a keypair file in the Solana CLI's format: a JSON array of the 64
/// bytes of the seed and the public key.
pub fn read_keypair(path: &Path) -> Result<Keypair, Error> {
	let text = std::fs::read_to_string(path)
		.map_err(|error| format!("cannot read keypair {}: {error}", path.display()))?;
	let bytes: Vec<u8> = serde_json::from_str(&text)
		.map_err(|error| format!("{} is not a keypair file: {error}", path.display()))?;

	Keypair::from_bytes(&bytes)
		.map_err(|error| format!("{} is not a keypair: {error:?}", path.display()).into())
}

/// A new keypair from the operating system's random numbers, with its seed so
/// it can be written to a file.
pub fn random_keypair() -> Result<([u8; 32], Keypair), Error> {
	let mut seed = [0; 32];
	getrandom::fill(&mut seed).map_err(|error| format!("no random numbers: {error}"))?;
	let keypair = Keypair::from_seed(&seed);

	Ok((seed, keypair))
}

/// Writes a keypair file in the Solana CLI's format.
pub fn write_keypair(path: &Path, seed: &[u8; 32], keypair: &Keypair) -> Result<(), Error> {
	let mut bytes = seed.to_vec();
	bytes.extend_from_slice(&keypair.address().to_bytes());
	std::fs::write(path, serde_json::to_string(&bytes)?)?;

	Ok(())
}
