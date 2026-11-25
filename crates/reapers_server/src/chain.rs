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
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::Instant;

use futures::future::BoxFuture;
use reapers_rules::ReaperGenome;
use reapers_rules::Refusal;
use solana_kit_client::Client;
use solana_kit_client::Limits;
use solana_kit_client::Reply;
use solana_kit_client::Request;
use solana_kit_client::Transport;
use solana_kit_client::TransportError;
use solana_kit_client::TransportFailureKind;
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

use crate::absorb::Burn;
use crate::config::SolanaConfig;
use crate::escrow;
use crate::escrow::Escrow;
use crate::escrow::HuskKind;

/// How long any one chain read may take before the request is refused.
pub const TIMEOUT: Duration = Duration::from_secs(8);

/// The most bytes of a Core account the keeper decodes. Bounds untrusted
/// decoding work before the decoder reads its strings.
const MAX_ASSET_BYTES: usize = 32 * 1024;

/// How long a finalized blockhash stays cached on live RPC gateways. Blockhashes
/// remain valid for ~150 slots (~60 s), so 3 s saves redundant RPC round trips
/// across bursts without risking stale lifetimes.
const BLOCKHASH_TTL: Duration = Duration::from_secs(3);

/// A Solana JSON-RPC endpoint: one request, one bounded reply.
///
/// The keeper's own uses a pooled HTTP/2 + keep-alive client; the tests answer
/// with controlled chain responses instead.
pub trait Rpc: Send + Sync + 'static {
	/// Posts `request` and receives its reply.
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>>;
}

impl Rpc for solana_kit_native::NativeHttp {
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>> {
		Box::pin(Transport::exchange(self, request))
	}
}

/// Pooled HTTP/2 + keep-alive Solana JSON-RPC transport that avoids a fresh
/// TCP + TLS handshake on every RPC call while preserving strict response size
/// bounds, no redirects, and no ambient proxy configuration.
struct PooledHttp {
	url: String,
	client: reqwest::Client,
}

impl PooledHttp {
	fn new(url: &str) -> Result<Self, crate::config::ConfigError> {
		// Validate the endpoint with Solana Kit's strict URL parser first so
		// invalid endpoints produce the exact same ConfigError.
		let _ = solana_kit_native::NativeHttp::new(url, solana_kit_native::HttpConfig::default())
			.map_err(|error| {
			crate::config::ConfigError(format!("REAPERS_SOLANA_RPC_URL: {error}"))
		})?;

		let client = reqwest::Client::builder()
			.pool_max_idle_per_host(16)
			.pool_idle_timeout(Duration::from_secs(90))
			.tcp_nodelay(true)
			.connect_timeout(Duration::from_secs(5))
			.timeout(TIMEOUT)
			.no_proxy()
			.redirect(reqwest::redirect::Policy::none())
			.build()
			.map_err(|error| {
				crate::config::ConfigError(format!("REAPERS_SOLANA_RPC_URL: {error}"))
			})?;

		Ok(Self {
			url: url.to_owned(),
			client,
		})
	}
}

impl Rpc for PooledHttp {
	fn exchange(&self, request: Request) -> BoxFuture<'_, Result<Reply, TransportError>> {
		Box::pin(async move {
			let mut response = self
				.client
				.post(&self.url)
				.header(reqwest::header::CONTENT_TYPE, "application/json")
				.body(request.body)
				.send()
				.await
				.map_err(|error| {
					let kind = if error.is_timeout() {
						TransportFailureKind::Deadline
					} else {
						TransportFailureKind::Io
					};
					TransportError::with_source(kind, error)
				})?;

			let status = response.status().as_u16();
			let capacity = response
				.content_length()
				.and_then(|len| usize::try_from(len).ok())
				.map_or(4096, |len| len.min(request.max_reply_bytes));
			let mut body = Vec::with_capacity(capacity);

			while let Some(chunk) = response
				.chunk()
				.await
				.map_err(|error| TransportError::with_source(TransportFailureKind::Io, error))?
			{
				if body.len().saturating_add(chunk.len()) > request.max_reply_bytes {
					return Err(TransportError::new(TransportFailureKind::BodyLimit));
				}
				body.extend_from_slice(&chunk);
			}

			Ok(Reply { status, body })
		})
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

/// How long a verified genesis hash stays trusted before `getGenesisHash` is
/// re-checked. Failures and mismatches are never cached.
const GENESIS_TTL: Duration = Duration::from_secs(300);

/// The chain, as the keeper reads it.
pub struct Gateway {
	config: SolanaConfig,
	reapers_uri_prefix: String,
	client: Client<SharedRpc>,
	genesis_verified_at: Mutex<Option<Instant>>,
	genesis_lock: tokio::sync::Mutex<()>,
	cache_blockhash: bool,
	cached_blockhash: Mutex<Option<(Hash, u64, Instant)>>,
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
		let reapers_uri_prefix = format!("{}reapers/", config.identity_uri);
		Self {
			config,
			reapers_uri_prefix,
			client: Client::new(SharedRpc(Arc::new(rpc)), Limits::default()),
			genesis_verified_at: Mutex::new(None),
			genesis_lock: tokio::sync::Mutex::new(()),
			cache_blockhash: false,
			cached_blockhash: Mutex::new(None),
		}
	}

	/// Reads the chain `config` names through its RPC URL, over pooled HTTPS.
	///
	/// # Errors
	///
	/// When the URL is not one Solana Kit's native transport accepts.
	pub fn connect(config: SolanaConfig) -> Result<Self, crate::config::ConfigError> {
		let http = PooledHttp::new(&config.rpc_url)?;
		let mut gateway = Self::new(config, http);
		gateway.cache_blockhash = true;
		Ok(gateway)
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
		within(std::any::type_name::<M>(), self.client.call(method)).await
	}

	/// Checks the RPC is the configured cluster, re-verifying after
	/// [`GENESIS_TTL`] so steady-state reads do not spend an extra RPC round
	/// trip on every call. Single-flighted so concurrent callers never
	/// thundering-herd `getGenesisHash`. Mismatches and transport failures are
	/// never cached. A localnet has no genesis to check: it is whatever
	/// validator the developer started on this machine.
	async fn check_cluster(&self) -> Result<(), Refusal> {
		if self.config.is_localnet() {
			return Ok(());
		}

		if self
			.genesis_verified_at
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.is_some_and(|at| at.elapsed() < GENESIS_TTL)
		{
			return Ok(());
		}

		let _flight = self.genesis_lock.lock().await;

		if self
			.genesis_verified_at
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.is_some_and(|at| at.elapsed() < GENESIS_TTL)
		{
			return Ok(());
		}

		let genesis = self.call(GenesisHash).await?;

		if genesis.to_string() == self.config.genesis_hash() {
			*self
				.genesis_verified_at
				.lock()
				.unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
			Ok(())
		} else {
			*self
				.genesis_verified_at
				.lock()
				.unwrap_or_else(PoisonError::into_inner) = None;
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
		if self.cache_blockhash
			&& let Some((hash, height, at)) = *self
				.cached_blockhash
				.lock()
				.unwrap_or_else(PoisonError::into_inner)
			&& at.elapsed() < BLOCKHASH_TTL
		{
			return Ok((hash, height));
		}

		self.check_cluster().await?;

		let latest = self.call(LatestBlockhash(Some(finalized()))).await?;
		let pair = (latest.value.blockhash, latest.value.last_valid_block_height);

		if self.cache_blockhash {
			*self
				.cached_blockhash
				.lock()
				.unwrap_or_else(PoisonError::into_inner) = Some((pair.0, pair.1, Instant::now()));
		}

		Ok(pair)
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

		let wallet = wallet
			.parse::<Address>()
			.map_err(|_| Refusal::WalletUnproven)?;
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

			if asset_v1.owner != wallet
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
	/// program, released by `wallet`, and returns its decoded [`Escrow`].
	///
	/// Refused as `chainUnconfigured` when no program is configured, and as
	/// `notInVault` when the escrow is not there or is someone else's.
	pub async fn escrow_of(&self, wallet: &Address, mint: &Address) -> Result<Escrow, Refusal> {
		let recorded = self.escrow(mint).await?;

		if recorded.wallet == *wallet {
			Ok(recorded)
		} else {
			Err(Refusal::NotInVault)
		}
	}

	/// Checks, at a finalized slot, that `mint` is escrowed with the Reapers
	/// program, released by `wallet`: its escrow account exists, the
	/// program owns it, and it records this wallet and this mint.
	///
	/// Refused as `chainUnconfigured` when no program is configured, and as
	/// `notInVault` when the escrow is not there or is someone else's.
	pub async fn escrowed(&self, wallet: &Address, mint: &Address) -> Result<(), Refusal> {
		self.escrow_of(wallet, mint).await.map(drop)
	}

	/// How the escrowed husk `mint` is burned: its kind, from its escrow,
	/// and for a Core asset its collection, read from the asset. Both at a
	/// finalized slot.
	///
	/// Refused as for [`Self::escrowed`], and as `notInVault` when a Core
	/// husk's asset is no longer an asset owned by its escrow PDA.
	pub async fn absorb_target(&self, mint: &Address) -> Result<Burn, Refusal> {
		let recorded = self.escrow(mint).await?;

		self.absorb_target_for(mint, &recorded).await
	}

	/// How the already-verified escrowed husk `mint` is burned, avoiding a
	/// second escrow account RPC read.
	pub async fn absorb_target_for(
		&self,
		mint: &Address,
		recorded: &Escrow,
	) -> Result<Burn, Refusal> {
		let collection = match recorded.kind {
			HuskKind::Core => {
				let (escrow_pda, _) = escrow::address_of(mint);

				self.check_cluster().await?;

				let assets = self
					.call(MultipleAccounts(
						vec![*mint],
						Some(AccountConfig {
							read: finalized(),
							encoding: Some(AccountEncoding::Base64),
							data_slice: None,
						}),
					))
					.await?;
				let Some(asset) = assets.value.into_iter().next().and_then(core_asset) else {
					return Err(Refusal::NotInVault);
				};

				if asset.owner != escrow_pda {
					return Err(Refusal::NotInVault);
				}

				match asset.update_authority {
					UpdateAuthority::Collection(collection) => Some(collection),
					_ => None,
				}
			}
			HuskKind::Token | HuskKind::Token2022 => None,
		};

		Ok(Burn::of(recorded.kind, collection))
	}

	/// The escrow the program keeps for `mint`, read at a finalized slot,
	/// when the program owns it and it is the escrow of exactly this mint.
	async fn escrow(&self, mint: &Address) -> Result<Escrow, Refusal> {
		if self.config.program.is_none() {
			return Err(Refusal::ChainUnconfigured);
		}

		let (address, bump) = escrow::address_of(mint);

		self.check_cluster().await?;

		let account = self.call(AccountInfo(address, finalized())).await?;
		let MaybeAccount::Present(account) = account.value else {
			return Err(Refusal::NotInVault);
		};
		match Escrow::decode(&account.data) {
			Some(recorded)
				if account.owner == escrow::program()
					&& !account.executable
					&& recorded.mint == *mint
					&& recorded.bump == bump =>
			{
				Ok(recorded)
			}
			_ => Err(Refusal::NotInVault),
		}
	}

	/// The genome a reaper's metadata URI names, when it is exactly the
	/// identity origin's `reapers/<genome code>.json` and the code is a
	/// valid genome. The trusted collection binds the URI; the first import
	/// freezes that genome in the keeper's ledger.
	pub fn genome_from_uri(&self, uri: &str) -> Option<ReaperGenome> {
		let code = uri
			.strip_prefix(self.reapers_uri_prefix.as_str())?
			.strip_suffix(".json")?;

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
/// than [`TIMEOUT`]. Logs the method name without exposing any RPC URL.
async fn within<T, E>(
	method: &'static str,
	request: impl Future<Output = Result<T, E>>,
) -> Result<T, Refusal> {
	match tokio::time::timeout(TIMEOUT, request).await {
		Ok(Ok(value)) => Ok(value),
		Ok(Err(_)) => {
			eprintln!("reapers_server: solana rpc {method} failed");
			Err(Refusal::ChainUnavailable)
		}
		Err(_) => {
			eprintln!("reapers_server: solana rpc {method} timed out after {TIMEOUT:?}");
			Err(Refusal::ChainUnavailable)
		}
	}
}
