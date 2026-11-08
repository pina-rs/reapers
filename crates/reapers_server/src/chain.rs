//! The keeper's only way to the chain: typed Solana RPC through Solana Kit,
//! every read at a finalized slot, every failure a refusal.
//!
//! External chain failures stay failures. A timeout, a malformed answer or
//! the wrong cluster never turns into an empty roster or lets an unverified
//! asset earn growth: each one refuses the request as `chainUnavailable` or
//! `chainUnconfigured`, and the request changes nothing. No RPC URL ever
//! reaches a player.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use reapers_rules::ReaperGenome;
use reapers_rules::Refusal;
use solana_kit_client::Client;
use solana_kit_client::Limits;
use solana_kit_client::Reply;
use solana_kit_client::Request;
use solana_kit_client::Transport;
use solana_kit_client::TransportError;
use solana_kit_client::methods::AccountInfo;
use solana_kit_client::methods::Balance;
use solana_kit_client::methods::Commitment;
use solana_kit_client::methods::LatestBlockhash;
use solana_kit_client::methods::Method;
use solana_kit_client::methods::ReadConfig;
use solana_kit_client::methods::accounts::AccountConfig;
use solana_kit_client::methods::accounts::AccountData;
use solana_kit_client::methods::accounts::AccountEncoding;
use solana_kit_client::methods::accounts::MultipleAccounts;
use solana_kit_client::methods::node::GenesisHash;
use solana_kit_core::Address;
use solana_kit_core::Hash;
use solana_kit_core::accounts::Account;
use solana_kit_core::accounts::MaybeAccount;
use solana_kit_core::codecs::Limits as CodecLimits;
use solana_kit_mpl_core::AssetV1;
use solana_kit_mpl_core::UpdateAuthority;
use solana_kit_mpl_core::decode_asset_v1;

use crate::config::SolanaConfig;
use crate::escrow;
use crate::escrow::Escrow;

/// How long any one chain read may take before the request is refused.
pub const TIMEOUT: Duration = Duration::from_secs(8);

/// The most bytes of a Core account the keeper decodes. Bounds untrusted
/// decoding work before the decoder reads its strings.
const MAX_ASSET_BYTES: usize = 32 * 1024;

/// A Solana JSON-RPC endpoint: one request, one bounded reply.
///
/// The keeper's own is Solana Kit's native HTTP transport; the tests answer
/// with controlled chain responses instead, as the Dart tests did.
pub trait Rpc: Send + Sync + 'static {
	/// Posts `request` and receives its reply.
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>>;
}

impl Rpc for solana_kit_native::NativeHttp {
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>> {
		Box::pin(Transport::exchange(self, request))
	}
}

/// An [`Rpc`] as Solana Kit's client takes a transport.
#[derive(Clone)]
pub struct SharedRpc(Arc<dyn Rpc>);

impl Transport for SharedRpc {
	type Exchange<'a> = BoxFuture<'a, Result<Reply, TransportError>>;

	fn exchange(&self, request: Request) -> Self::Exchange<'_> {
		self.0.exchange(request)
	}
}

/// The chain, as the keeper reads it.
pub struct Gateway {
	config: SolanaConfig,
	client: Client<SharedRpc>,
}

impl std::fmt::Debug for Gateway {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Gateway")
			.field("chain", &self.config.chain)
			.finish_non_exhaustive()
	}
}

impl Gateway {
	/// Reads the chain `config` names through `rpc`.
	pub fn new(config: SolanaConfig, rpc: impl Rpc) -> Self {
		Self {
			config,
			client: Client::new(SharedRpc(Arc::new(rpc)), Limits::default()),
		}
	}

	/// Reads the chain `config` names through its RPC URL, over HTTPS.
	///
	/// # Errors
	///
	/// When the URL is not one Solana Kit's native transport accepts.
	pub fn connect(config: SolanaConfig) -> Result<Self, crate::config::ConfigError> {
		let http = solana_kit_native::NativeHttp::new(
			&config.rpc_url,
			solana_kit_native::HttpConfig::default(),
		)
		.map_err(|error| crate::config::ConfigError(format!("REAPERS_SOLANA_RPC_URL: {error}")))?;

		Ok(Self::new(config, http))
	}

	/// The settings the gateway reads by.
	pub const fn config(&self) -> &SolanaConfig {
		&self.config
	}

	async fn call<M>(&self, method: M) -> Result<M::Response, Refusal>
	where
		M: Method + Send,
		M::Response: Send,
	{
		within(self.client.call(method)).await
	}

	/// Checks the RPC is the configured cluster, on every operation rather
	/// than trusting a URL or a cached answer.
	async fn check_cluster(&self) -> Result<(), Refusal> {
		let genesis = self.call(GenesisHash).await?;

		if genesis.to_string() == self.config.genesis_hash() {
			Ok(())
		} else {
			Err(Refusal::ChainUnconfigured)
		}
	}

	/// `wallet`'s finalized balance, in lamports.
	pub async fn balance(&self, wallet: &Address) -> Result<u64, Refusal> {
		self.check_cluster().await?;

		let balance = self.call(Balance(*wallet, Some(finalized()))).await?;

		Ok(balance.value)
	}

	/// The newest finalized blockhash and the height it lasts until, for a
	/// transaction the keeper hands a wallet to sign.
	pub async fn latest_blockhash(&self) -> Result<(Hash, u64), Refusal> {
		self.check_cluster().await?;

		let latest = self.call(LatestBlockhash(Some(finalized()))).await?;

		Ok((latest.value.blockhash, latest.value.last_valid_block_height))
	}

	/// Which of `assets` `wallet` owns as reapers of the configured
	/// collection, each with the genome its metadata URI names, all read at
	/// one finalized slot.
	///
	/// An asset is owned when its account is a non-executable account owned
	/// by Metaplex Core, decodes as an `AssetV1`, belongs to the collection,
	/// is owned by `wallet`, and its URI is the identity origin's
	/// `reapers/<genome>.json` with a valid genome. No metadata URL is
	/// fetched. Anything else is simply not owned.
	pub async fn owned(
		&self,
		wallet: &str,
		assets: &[String],
	) -> Result<HashMap<String, ReaperGenome>, Refusal> {
		let Some(collection) = self.config.collection else {
			return Err(Refusal::ChainUnconfigured);
		};

		if assets.is_empty() {
			return Ok(HashMap::new());
		}

		let addresses = assets
			.iter()
			.map(|asset| asset.parse::<Address>())
			.collect::<Result<Vec<_>, _>>()
			.map_err(|_| Refusal::NotOwned)?;

		self.check_cluster().await?;

		let accounts = self
			.call(MultipleAccounts(
				addresses,
				Some(AccountConfig {
					read: finalized(),
					encoding: Some(AccountEncoding::Base64),
					data_slice: None,
				}),
			))
			.await?;

		if accounts.value.len() != assets.len() {
			return Err(Refusal::ChainUnavailable);
		}

		let mut owned = HashMap::new();

		for (asset, account) in assets.iter().zip(accounts.value) {
			let Some(asset_v1) = core_asset(account) else {
				continue;
			};

			if asset_v1.owner.to_string() != wallet
				|| asset_v1.update_authority != UpdateAuthority::Collection(collection)
			{
				continue;
			}

			if let Some(genome) = self.genome_from_uri(&asset_v1.uri) {
				owned.insert(asset.clone(), genome);
			}
		}

		Ok(owned)
	}

	/// Checks, at a finalized slot, that `mint` is escrowed with the Reapers
	/// program, released by `wallet`: its escrow account exists, the
	/// program owns it, and it records this wallet and this mint.
	///
	/// Refused as `chainUnconfigured` when no program is configured, and as
	/// `notInVault` when the escrow is not there or is someone else's.
	pub async fn escrowed(&self, wallet: &Address, mint: &Address) -> Result<(), Refusal> {
		let Some(program) = self.config.program else {
			return Err(Refusal::ChainUnconfigured);
		};
		let Some((address, bump)) = escrow::address(&program, mint) else {
			return Err(Refusal::NotInVault);
		};

		self.check_cluster().await?;

		let account = self.call(AccountInfo(address, finalized())).await?;
		let MaybeAccount::Present(account) = account.value else {
			return Err(Refusal::NotInVault);
		};
		let recorded = Escrow::decode(&account.data);
		let held = account.owner == program
			&& !account.executable
			&& recorded.is_some_and(|recorded| {
				recorded.wallet == *wallet && recorded.mint == *mint && recorded.bump == bump
			});

		if held {
			Ok(())
		} else {
			Err(Refusal::NotInVault)
		}
	}

	/// The genome a reaper's metadata URI names, when it is exactly the
	/// identity origin's `reapers/<genome code>.json` and the code is a
	/// valid genome. The trusted collection binds the URI; the first import
	/// freezes that genome in the keeper's ledger.
	pub fn genome_from_uri(&self, uri: &str) -> Option<ReaperGenome> {
		let prefix = format!("{}reapers/", self.config.identity_uri);
		let code = uri.strip_prefix(&prefix)?.strip_suffix(".json")?;

		ReaperGenome::parse(code).ok()
	}
}

/// A finalized read: ownership is never judged on a slot that could roll
/// back.
const fn finalized() -> ReadConfig {
	ReadConfig {
		commitment: Some(Commitment::Finalized),
		min_context_slot: None,
	}
}

/// The Core asset an account holds, or `None` when it is not an owned,
/// non-executable, bounded `AssetV1` account of Metaplex Core.
fn core_asset(account: MaybeAccount<AccountData>) -> Option<AssetV1> {
	let MaybeAccount::Present(account) = account else {
		return None;
	};
	let AccountData::Bytes { bytes, slice: None } = account.data else {
		return None;
	};

	if account.owner != solana_kit_mpl_core::PROGRAM
		|| account.executable
		|| bytes.is_empty()
		|| bytes.len() > MAX_ASSET_BYTES
	{
		return None;
	}

	let encoded = Account {
		address: account.address,
		owner: account.owner,
		lamports: account.lamports,
		executable: account.executable,
		space: account.space,
		rent_epoch: account.rent_epoch,
		data: bytes,
	};

	decode_asset_v1(encoded, CodecLimits::new(MAX_ASSET_BYTES, MAX_ASSET_BYTES))
		.ok()
		.map(|decoded| decoded.data.base)
}

/// `request`, failing as `chainUnavailable` when it fails or takes longer
/// than [`TIMEOUT`].
async fn within<T, E>(request: impl Future<Output = Result<T, E>>) -> Result<T, Refusal> {
	match tokio::time::timeout(TIMEOUT, request).await {
		Ok(Ok(value)) => Ok(value),
		Ok(Err(_)) | Err(_) => Err(Refusal::ChainUnavailable),
	}
}
