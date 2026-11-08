//! A real Solana JSON-RPC endpoint, for the Surfpool validator a developer
//! runs beside the simnet.
//!
//! This speaks only standard Solana methods, and it is not a way to
//! coordinate players: Surfpool is a validator, not a game server. The
//! simnet's ledger is told nothing of what happens here, and nothing here
//! touches the game's books.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::FutureExt as _;
use futures::future::LocalBoxFuture;
use futures::future::Shared;
use reapers_rules::Timestamp;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::platform;
use crate::platform::HttpTransport;
use crate::platform::Prefs;

/// Why a call to a local Solana RPC did not finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfpoolFailure {
	/// The endpoint could not be reached at all.
	Unreachable,
	/// The endpoint did not answer in time.
	TimedOut,
	/// The endpoint answered with an HTTP status that is not 200.
	Http,
	/// The JSON-RPC response reported an error.
	Rpc,
	/// The response was not the JSON-RPC the client asked for.
	Malformed,
	/// A sent transaction reported an error instead of confirming.
	Transaction,
	/// The faucet's written record cannot be read or saved. It is never
	/// quietly reset: a forgotten record would let the next tap pay twice.
	Record,
}

/// A failed call, with the method and enough detail to tell a developer
/// what went wrong. Only the simnet's developer panel shows the detail.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{method} ({failure:?}): {detail}")]
pub struct SurfpoolError {
	pub failure: SurfpoolFailure,
	/// The JSON-RPC method that failed, such as `getHealth`.
	pub method: String,
	pub detail: String,
}

impl SurfpoolError {
	fn new(failure: SurfpoolFailure, method: &str, detail: impl Into<String>) -> Self {
		Self {
			failure,
			method: method.to_owned(),
			detail: detail.into(),
		}
	}
}

pub type SurfpoolResult<T> = Result<T, SurfpoolError>;

/// What the validator says about one signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureStatus {
	pub slot: Option<i64>,
	pub confirmed: bool,
	pub error: Option<String>,
}

/// The validator's endpoint, and how to talk to it.
pub struct SurfpoolRpc {
	endpoint: String,
	transport: Rc<dyn HttpTransport>,
	timeout: Duration,
	poll: Duration,
	next_id: RefCell<u64>,
}

/// Whether `host` is where a developer's local validator is likely to be.
fn is_local_network(host: &str) -> bool {
	if matches!(
		host,
		"localhost" | "127.0.0.1" | "[::1]" | "::1" | "10.0.2.2" | "host.docker.internal"
	) || host.ends_with(".local")
	{
		return true;
	}
	let octets: Vec<Option<u8>> = host.split('.').map(|part| part.parse().ok()).collect();
	match octets[..] {
		[Some(10), Some(_), Some(_), Some(_)]
		| [Some(192), Some(168), Some(_), Some(_)]
		| [Some(169), Some(254), Some(_), Some(_)] => true,
		[Some(172), Some(second), Some(_), Some(_)] => (16..=31).contains(&second),
		_ => false,
	}
}

impl SurfpoolRpc {
	/// How long to wait between signature status checks.
	pub const POLL: Duration = Duration::from_millis(200);
	/// How long one call may take.
	pub const TIMEOUT: Duration = Duration::from_secs(4);

	/// Checks an endpoint is one this development build may talk to.
	///
	/// Cleartext HTTP is allowed on loopback, an emulator's host alias and a
	/// private network, where a Surfpool on the same machine or LAN lives. A
	/// public host must use HTTPS.
	pub fn validate(endpoint: &str) -> Result<String, String> {
		const PROBLEM: &str = "REAPERS_SURFPOOL_URL must be http://localhost:8899, http://127.0.0.1:8899, an emulator/LAN address, or an HTTPS Solana RPC URL";
		let Some((scheme, rest)) = endpoint.split_once("://") else {
			return Err(PROBLEM.into());
		};
		let authority = rest.split('/').next().unwrap_or_default();
		let host = authority
			.rsplit_once(':')
			.filter(|(_, port)| port.chars().all(|digit| digit.is_ascii_digit()))
			.map_or(authority, |(host, _)| host);
		let clean = !authority.contains('@') && !rest.contains(['?', '#']) && !host.is_empty();
		let allowed = match scheme {
			"https" => true,
			"http" => is_local_network(host),
			_ => false,
		};
		if clean && allowed {
			Ok(endpoint.to_owned())
		} else {
			Err(PROBLEM.into())
		}
	}

	/// A client for `endpoint`, once it passes [`Self::validate`].
	pub fn new(endpoint: &str, transport: Rc<dyn HttpTransport>) -> Result<Self, String> {
		Ok(Self {
			endpoint: Self::validate(endpoint)?,
			transport,
			timeout: Self::TIMEOUT,
			poll: Self::POLL,
			next_id: RefCell::new(0),
		})
	}

	/// The same client, waiting `timeout` for confirmations.
	#[must_use]
	pub const fn with_timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}

	pub fn endpoint(&self) -> &str {
		&self.endpoint
	}

	/// Asks the validator whether it is ready.
	pub async fn health(&self) -> SurfpoolResult<()> {
		match self.call("getHealth", json!([])).await? {
			Value::String(ok) if ok == "ok" => Ok(()),
			other => {
				Err(SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"getHealth",
					format!("expected \"ok\", got {other}"),
				))
			}
		}
	}

	/// The genesis hash, for telling one cluster from another.
	pub async fn genesis_hash(&self) -> SurfpoolResult<String> {
		match self.call("getGenesisHash", json!([])).await? {
			Value::String(hash) if !hash.is_empty() => Ok(hash),
			_ => {
				Err(SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"getGenesisHash",
					"the result was not a hash",
				))
			}
		}
	}

	/// The lamport balance of `address`. A balance is a whole, non-negative
	/// number; anything else is an answer this client does not understand,
	/// not one to round or flip.
	pub async fn balance(&self, address: &str) -> SurfpoolResult<u64> {
		require_address(address, "getBalance")?;
		self.call("getBalance", json!([address]))
			.await?
			.get("value")
			.and_then(Value::as_u64)
			.ok_or_else(|| {
				SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"getBalance",
					"the result had no non-negative whole lamports",
				)
			})
	}

	/// The latest blockhash, for a transaction the validator can take.
	pub async fn latest_blockhash(&self) -> SurfpoolResult<String> {
		self.call("getLatestBlockhash", json!([]))
			.await?
			.get("value")
			.and_then(|value| value.get("blockhash"))
			.and_then(Value::as_str)
			.map(str::to_owned)
			.ok_or_else(|| {
				SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"getLatestBlockhash",
					"the result had no blockhash",
				)
			})
	}

	/// Sends a signed wire transaction and returns its signature, without
	/// waiting for it to confirm.
	pub async fn send_transaction(&self, wire: &[u8]) -> SurfpoolResult<String> {
		match self
			.call(
				"sendTransaction",
				json!([BASE64.encode(wire), { "encoding": "base64" }]),
			)
			.await?
		{
			Value::String(signature) if !signature.is_empty() => Ok(signature),
			_ => {
				Err(SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"sendTransaction",
					"the result was not a signature",
				))
			}
		}
	}

	/// Asks the local faucet for `lamports` to `address` and returns the
	/// signature, without waiting for the transfer to confirm.
	///
	/// Solana's faucet is not idempotent: when the answer to this call is
	/// lost, whether it paid is unknowable from here. A caller that must not
	/// risk paying twice writes the signature down before awaiting it.
	pub async fn request_airdrop(&self, address: &str, lamports: u64) -> SurfpoolResult<String> {
		require_address(address, "requestAirdrop")?;
		match self
			.call("requestAirdrop", json!([address, lamports]))
			.await?
		{
			Value::String(signature) if !signature.is_empty() => Ok(signature),
			_ => {
				Err(SurfpoolError::new(
					SurfpoolFailure::Malformed,
					"requestAirdrop",
					"the result was not a signature",
				))
			}
		}
	}

	/// One reading of `signature`'s status.
	pub async fn signature_status(
		&self,
		signature: &str,
	) -> SurfpoolResult<Option<SignatureStatus>> {
		const METHOD: &str = "getSignatureStatuses";
		let result = self
			.call(
				METHOD,
				json!([[signature], { "searchTransactionHistory": true }]),
			)
			.await?;
		let values = result
			.get("value")
			.and_then(Value::as_array)
			.ok_or_else(|| {
				SurfpoolError::new(
					SurfpoolFailure::Malformed,
					METHOD,
					"the result had no value list",
				)
			})?;
		let Some(first) = values.first().filter(|first| !first.is_null()) else {
			return Ok(None);
		};
		let (Some(slot), Some(status)) = (
			first.get("slot").and_then(Value::as_i64),
			first.get("confirmationStatus").and_then(Value::as_str),
		) else {
			return Err(SurfpoolError::new(
				SurfpoolFailure::Malformed,
				METHOD,
				"the status entry was not a status",
			));
		};
		Ok(Some(SignatureStatus {
			slot: Some(slot),
			confirmed: matches!(status, "confirmed" | "finalized"),
			error: first
				.get("err")
				.filter(|error| !error.is_null())
				.map(Value::to_string),
		}))
	}

	/// Waits until `signature` is confirmed, or the client's timeout passes.
	pub async fn await_confirmation(&self, signature: &str) -> SurfpoolResult<SignatureStatus> {
		let attempts = (self.timeout.as_millis() / self.poll.as_millis().max(1)).max(1);
		for _ in 0..=attempts {
			let status = self.signature_status(signature).await?;
			if let Some(error) = status.as_ref().and_then(|status| status.error.clone()) {
				return Err(SurfpoolError::new(
					SurfpoolFailure::Transaction,
					"getSignatureStatuses",
					error,
				));
			}
			if let Some(status) = status.filter(|status| status.confirmed) {
				return Ok(status);
			}
			platform::sleep(self.poll).await;
		}
		Err(SurfpoolError::new(
			SurfpoolFailure::TimedOut,
			"getSignatureStatuses",
			format!("no confirmed status for {signature}"),
		))
	}

	async fn call(&self, method: &str, params: Value) -> SurfpoolResult<Value> {
		let id = {
			let mut next = self.next_id.borrow_mut();
			*next += 1;
			*next
		};
		let body = serde_json::to_vec(
			&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
		)
		.map_err(|error| {
			SurfpoolError::new(SurfpoolFailure::Malformed, method, error.to_string())
		})?;
		let reply = self
			.transport
			.post_json(&self.endpoint, body, self.timeout)
			.await
			.map_err(|error| {
				let failure = if error.to_string().contains("timed out")
					|| error.to_string().contains("Timeout")
				{
					SurfpoolFailure::TimedOut
				} else {
					SurfpoolFailure::Unreachable
				};
				SurfpoolError::new(failure, method, error.to_string())
			})?;
		if reply.status != 200 {
			return Err(SurfpoolError::new(
				SurfpoolFailure::Http,
				method,
				format!("HTTP {}", reply.status),
			));
		}
		let decoded: Value = serde_json::from_slice(&reply.body).map_err(|error| {
			SurfpoolError::new(SurfpoolFailure::Malformed, method, error.to_string())
		})?;
		let Some(object) = decoded.as_object() else {
			return Err(SurfpoolError::new(
				SurfpoolFailure::Malformed,
				method,
				"the response was not a JSON object",
			));
		};
		if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
			|| object.get("id").and_then(Value::as_u64) != Some(id)
		{
			return Err(SurfpoolError::new(
				SurfpoolFailure::Malformed,
				method,
				"the response did not match this request",
			));
		}
		if let Some(error) = object.get("error").filter(|error| error.is_object()) {
			let detail = [
				error.get("code").map(|code| format!("code {code}")),
				error
					.get("message")
					.and_then(Value::as_str)
					.map(str::to_owned),
			]
			.into_iter()
			.flatten()
			.collect::<Vec<_>>()
			.join(": ");
			return Err(SurfpoolError::new(SurfpoolFailure::Rpc, method, detail));
		}
		object.get("result").cloned().ok_or_else(|| {
			SurfpoolError::new(
				SurfpoolFailure::Malformed,
				method,
				"the response had neither a result nor an error",
			)
		})
	}
}

fn require_address(address: &str, method: &str) -> SurfpoolResult<()> {
	if address.is_empty() {
		Err(SurfpoolError::new(
			SurfpoolFailure::Malformed,
			method,
			"an empty address",
		))
	} else {
		Ok(())
	}
}

/// One airdrop this install has asked a validator for, pending or confirmed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaucetReceipt {
	#[serde(skip)]
	pub wallet: String,
	pub genesis: String,
	pub signature: String,
	pub slot: Option<i64>,
	#[serde(with = "iso8601")]
	pub at: Timestamp,
	pub confirmed: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct FaucetRecord {
	wallets: BTreeMap<String, FaucetReceipt>,
}

type Pending = Shared<LocalBoxFuture<'static, SurfpoolResult<FaucetReceipt>>>;

/// Pays the simnet's burner wallet from a local validator once per chain.
///
/// The moment the faucet returns a signature it is written down with the
/// validator's genesis and `confirmed: false`, before the transfer is
/// awaited. Every later tap, and a freshly reopened app, settles that
/// signature again instead of asking the faucet a second time, and only a
/// confirmed status turns the record into a receipt.
pub struct SurfpoolFaucet {
	prefs: Rc<dyn Prefs>,
	rpc: Rc<SurfpoolRpc>,
	clock: Rc<dyn Fn() -> Timestamp>,
	pending: Rc<RefCell<HashMap<String, Pending>>>,
}

impl SurfpoolFaucet {
	/// The faucet's amount: a local demo amount, not the game's SOL.
	pub const LAMPORTS: u64 = 1_000_000_000;
	pub const STORAGE_KEY: &'static str = "reapers.surfpool.faucet.v1";

	pub fn new(
		prefs: Rc<dyn Prefs>,
		rpc: Rc<SurfpoolRpc>,
		clock: Rc<dyn Fn() -> Timestamp>,
	) -> Self {
		Self {
			prefs,
			rpc,
			clock,
			pending: Rc::new(RefCell::new(HashMap::new())),
		}
	}

	/// The airdrop already asked for `wallet`, pending or confirmed.
	pub async fn receipt_for(&self, wallet: &str) -> SurfpoolResult<Option<FaucetReceipt>> {
		Ok(read(&*self.prefs)
			.await?
			.wallets
			.remove(wallet)
			.map(|receipt| {
				FaucetReceipt {
					wallet: wallet.to_owned(),
					..receipt
				}
			}))
	}

	/// Airdrops to `wallet` once per validator chain, settling the request
	/// that is already written down when there is one.
	///
	/// A tap that arrives while the first is in flight shares it. A
	/// transaction error is definitive: the record is cleared and the next
	/// call may ask again. If the request's own answer is lost before a
	/// signature comes back, the faucet may still have paid, and nothing is
	/// written; no client can do better against a faucet that is not
	/// idempotent.
	pub async fn fund(&self, wallet: &str) -> SurfpoolResult<FaucetReceipt> {
		let existing = self.pending.borrow().get(wallet).cloned();
		let pending = if let Some(pending) = existing {
			pending
		} else {
			let prefs = self.prefs.clone();
			let rpc = self.rpc.clone();
			let clock = self.clock.clone();
			let owner = wallet.to_owned();
			let shared = async move { pay(&*prefs, &rpc, &*clock, &owner).await }
				.boxed_local()
				.shared();
			self.pending
				.borrow_mut()
				.insert(wallet.to_owned(), shared.clone());
			shared
		};
		let result = pending.await;
		self.pending.borrow_mut().remove(wallet);
		result
	}
}

async fn pay(
	prefs: &dyn Prefs,
	rpc: &SurfpoolRpc,
	clock: &dyn Fn() -> Timestamp,
	wallet: &str,
) -> SurfpoolResult<FaucetReceipt> {
	let genesis = rpc.genesis_hash().await?;
	if let Some(known) = read(prefs)
		.await?
		.wallets
		.remove(wallet)
		.filter(|known| known.genesis == genesis)
	{
		let known = FaucetReceipt {
			wallet: wallet.to_owned(),
			..known
		};
		if known.confirmed {
			return Ok(known);
		}
		// A request for this chain is already outstanding: settle its
		// signature, because asking again could pay twice.
		return settle(prefs, rpc, known).await;
	}

	let signature = rpc
		.request_airdrop(wallet, SurfpoolFaucet::LAMPORTS)
		.await?;
	let asked = FaucetReceipt {
		wallet: wallet.to_owned(),
		genesis,
		signature,
		slot: None,
		at: clock(),
		confirmed: false,
	};
	store(prefs, &asked).await?;
	settle(prefs, rpc, asked).await
}

async fn settle(
	prefs: &dyn Prefs,
	rpc: &SurfpoolRpc,
	asked: FaucetReceipt,
) -> SurfpoolResult<FaucetReceipt> {
	match rpc.await_confirmation(&asked.signature).await {
		Ok(status) => {
			let paid = FaucetReceipt {
				slot: status.slot,
				confirmed: true,
				..asked
			};
			store(prefs, &paid).await?;
			Ok(paid)
		}
		Err(error) => {
			if error.failure == SurfpoolFailure::Transaction {
				let mut record = read(prefs).await?;
				if record
					.wallets
					.get(&asked.wallet)
					.is_some_and(|known| known.signature == asked.signature)
				{
					record.wallets.remove(&asked.wallet);
					write(prefs, &record).await?;
				}
			}
			Err(error)
		}
	}
}

async fn read(prefs: &dyn Prefs) -> SurfpoolResult<FaucetRecord> {
	let record_error =
		|detail: String| SurfpoolError::new(SurfpoolFailure::Record, "faucet", detail);
	match prefs
		.get(SurfpoolFaucet::STORAGE_KEY)
		.await
		.map_err(|error| record_error(error.to_string()))?
	{
		None => Ok(FaucetRecord::default()),
		Some(value) => {
			serde_json::from_value(value).map_err(|error| {
				record_error(format!("the saved faucet record is unreadable: {error}"))
			})
		}
	}
}

async fn store(prefs: &dyn Prefs, receipt: &FaucetReceipt) -> SurfpoolResult<()> {
	let mut record = read(prefs).await?;
	record
		.wallets
		.insert(receipt.wallet.clone(), receipt.clone());
	write(prefs, &record).await
}

async fn write(prefs: &dyn Prefs, record: &FaucetRecord) -> SurfpoolResult<()> {
	let record_error =
		|detail: String| SurfpoolError::new(SurfpoolFailure::Record, "faucet", detail);
	let value = serde_json::to_value(record).map_err(|error| record_error(error.to_string()))?;
	prefs
		.set(SurfpoolFaucet::STORAGE_KEY, value)
		.await
		.map_err(|error| record_error(format!("the faucet record could not be saved: {error}")))
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
		Timestamp::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;
	use std::collections::VecDeque;

	use async_trait::async_trait;
	use futures::executor::block_on;

	use super::*;
	use crate::platform::HttpReply;
	use crate::platform::MemoryPrefs;
	use crate::platform::PlatformError;

	/// A validator that answers from a script, method by method, echoing
	/// each request's id.
	#[derive(Default)]
	struct Script {
		answers: RefCell<HashMap<&'static str, VecDeque<Result<Value, Value>>>>,
		calls: RefCell<Vec<String>>,
		status: Cell<u16>,
		lost: Cell<bool>,
	}

	impl Script {
		fn answer(&self, method: &'static str, result: Value) -> &Self {
			self.answers
				.borrow_mut()
				.entry(method)
				.or_default()
				.push_back(Ok(result));
			self
		}

		fn fail(&self, method: &'static str, error: Value) -> &Self {
			self.answers
				.borrow_mut()
				.entry(method)
				.or_default()
				.push_back(Err(error));
			self
		}

		fn count(&self, method: &str) -> usize {
			self.calls
				.borrow()
				.iter()
				.filter(|call| *call == method)
				.count()
		}
	}

	#[async_trait(?Send)]
	impl HttpTransport for Script {
		async fn post_json(
			&self,
			_url: &str,
			body: Vec<u8>,
			_timeout: Duration,
		) -> Result<HttpReply, PlatformError> {
			if self.lost.get() {
				return Err(PlatformError::Failed("connection reset".into()));
			}
			let request: Value = serde_json::from_slice(&body).unwrap();
			let method = request["method"].as_str().unwrap().to_owned();
			self.calls.borrow_mut().push(method.clone());
			let answer = self
				.answers
				.borrow_mut()
				.get_mut(method.as_str())
				.and_then(|queue| {
					if queue.len() > 1 {
						queue.pop_front()
					} else {
						queue.front().cloned()
					}
				})
				.unwrap_or(Ok(Value::Null));
			let mut reply = json!({ "jsonrpc": "2.0", "id": request["id"] });
			match answer {
				Ok(result) => reply["result"] = result,
				Err(error) => reply["error"] = error,
			}
			Ok(HttpReply {
				status: match self.status.get() {
					0 => 200,
					status => status,
				},
				body: serde_json::to_vec(&reply).unwrap(),
			})
		}
	}

	fn client(script: &Rc<Script>) -> Rc<SurfpoolRpc> {
		Rc::new(
			SurfpoolRpc::new("http://localhost:8899", script.clone())
				.unwrap()
				.with_timeout(Duration::from_millis(600)),
		)
	}

	fn status(confirmation: &str) -> Value {
		json!({ "value": [{ "slot": 7, "err": null, "confirmationStatus": confirmation }] })
	}

	const WALLET: &str = "Burner11111111111111111111111111111111111111";

	#[test]
	fn health_genesis_and_balance_are_standard_json_rpc_calls() {
		let script = Rc::new(Script::default());
		script
			.answer("getHealth", json!("ok"))
			.answer("getGenesisHash", json!("genesis"))
			.answer(
				"getBalance",
				json!({ "context": { "slot": 1 }, "value": 2_500_000_000_u64 }),
			);
		let rpc = client(&script);
		block_on(rpc.health()).unwrap();
		assert_eq!(block_on(rpc.genesis_hash()).unwrap(), "genesis");
		assert_eq!(block_on(rpc.balance(WALLET)).unwrap(), 2_500_000_000);
	}

	#[test]
	fn a_fractional_or_negative_balance_is_refused_not_rounded() {
		let script = Rc::new(Script::default());
		script.answer("getBalance", json!({ "value": 1.5 }));
		let rpc = client(&script);
		assert_eq!(
			block_on(rpc.balance(WALLET)).unwrap_err().failure,
			SurfpoolFailure::Malformed
		);

		let script = Rc::new(Script::default());
		script.answer("getBalance", json!({ "value": -1 }));
		assert_eq!(
			block_on(client(&script).balance(WALLET))
				.unwrap_err()
				.failure,
			SurfpoolFailure::Malformed
		);
	}

	#[test]
	fn an_rpc_error_and_an_http_error_are_told_apart() {
		let script = Rc::new(Script::default());
		script.fail(
			"getHealth",
			json!({ "code": -32005, "message": "node is behind" }),
		);
		let error = block_on(client(&script).health()).unwrap_err();
		assert_eq!(error.failure, SurfpoolFailure::Rpc);
		assert!(error.detail.contains("node is behind"));

		let script = Rc::new(Script::default());
		script.status.set(503);
		assert_eq!(
			block_on(client(&script).health()).unwrap_err().failure,
			SurfpoolFailure::Http
		);
	}

	#[test]
	fn a_transaction_error_is_not_a_confirmation() {
		let script = Rc::new(Script::default());
		script.answer(
			"getSignatureStatuses",
			json!({ "value": [{ "slot": 7, "err": { "InstructionError": [0, "Custom"] }, "confirmationStatus": "processed" }] }),
		);
		let error = block_on(client(&script).await_confirmation("sig")).unwrap_err();
		assert_eq!(error.failure, SurfpoolFailure::Transaction);
	}

	#[test]
	fn an_unconfirmed_signature_times_out_instead_of_hanging_the_game() {
		let script = Rc::new(Script::default());
		script.answer("getSignatureStatuses", json!({ "value": [null] }));
		let error = block_on(client(&script).await_confirmation("sig")).unwrap_err();
		assert_eq!(error.failure, SurfpoolFailure::TimedOut);
	}

	#[test]
	fn only_a_local_endpoint_may_use_cleartext_http() {
		for allowed in [
			"http://localhost:8899",
			"http://127.0.0.1:8899",
			"http://10.0.2.2:8899",
			"http://192.168.1.20:8899",
			"http://172.16.0.4:8899",
			"https://rpc.example.com",
		] {
			assert!(SurfpoolRpc::validate(allowed).is_ok(), "{allowed}");
		}
		for refused in [
			"http://rpc.example.com",
			"http://172.32.0.4:8899",
			"ftp://localhost:8899",
			"http://user@localhost:8899",
			"http://localhost:8899?x=1",
			"localhost:8899",
		] {
			assert!(SurfpoolRpc::validate(refused).is_err(), "{refused}");
		}
	}

	fn open_faucet(script: &Rc<Script>, prefs: &Rc<MemoryPrefs>) -> SurfpoolFaucet {
		SurfpoolFaucet::new(
			prefs.clone(),
			client(script),
			Rc::new(|| Timestamp::utc(2026, 11, 4, 12, 0, 0)),
		)
	}

	#[test]
	fn a_confirmed_faucet_receipt_is_written_once_and_reused() {
		let script = Rc::new(Script::default());
		script
			.answer("getGenesisHash", json!("genesis"))
			.answer("requestAirdrop", json!("airdrop"))
			.answer("getSignatureStatuses", status("confirmed"));
		let prefs = Rc::new(MemoryPrefs::new());
		let faucet = open_faucet(&script, &prefs);

		let paid = block_on(faucet.fund(WALLET)).unwrap();
		assert!(paid.confirmed);
		assert_eq!(paid.slot, Some(7));
		let again = block_on(faucet.fund(WALLET)).unwrap();
		assert_eq!(again, paid);
		assert_eq!(script.count("requestAirdrop"), 1);
		assert_eq!(block_on(faucet.receipt_for(WALLET)).unwrap(), Some(paid));
	}

	#[test]
	fn a_restarted_validator_has_a_new_chain_and_may_pay_again() {
		let script = Rc::new(Script::default());
		script
			.answer("getGenesisHash", json!("first"))
			.answer("getGenesisHash", json!("second"))
			.answer("requestAirdrop", json!("airdrop"))
			.answer("getSignatureStatuses", status("finalized"));
		let prefs = Rc::new(MemoryPrefs::new());
		let faucet = open_faucet(&script, &prefs);

		block_on(faucet.fund(WALLET)).unwrap();
		let second = block_on(faucet.fund(WALLET)).unwrap();
		assert_eq!(second.genesis, "second");
		assert_eq!(script.count("requestAirdrop"), 2);
	}

	#[test]
	fn a_pending_airdrop_is_settled_not_asked_for_again() {
		let script = Rc::new(Script::default());
		script
			.answer("getGenesisHash", json!("genesis"))
			.answer("requestAirdrop", json!("airdrop"))
			.answer("getSignatureStatuses", json!({ "value": [null] }));
		let prefs = Rc::new(MemoryPrefs::new());
		let faucet = open_faucet(&script, &prefs);

		assert_eq!(
			block_on(faucet.fund(WALLET)).unwrap_err().failure,
			SurfpoolFailure::TimedOut
		);
		let written = block_on(faucet.receipt_for(WALLET)).unwrap().unwrap();
		assert!(
			!written.confirmed,
			"the signature is written before it confirms"
		);

		script.answers.borrow_mut().insert(
			"getSignatureStatuses",
			VecDeque::from([Ok(status("confirmed"))]),
		);
		let reopened = open_faucet(&script, &prefs);
		assert!(block_on(reopened.fund(WALLET)).unwrap().confirmed);
		assert_eq!(script.count("requestAirdrop"), 1);
	}

	#[test]
	fn a_failed_transfer_clears_the_record_so_the_next_tap_may_ask_again() {
		let script = Rc::new(Script::default());
		script
			.answer("getGenesisHash", json!("genesis"))
			.answer("requestAirdrop", json!("airdrop"))
			.answer(
				"getSignatureStatuses",
				json!({ "value": [{ "slot": 7, "err": "InsufficientFundsForFee", "confirmationStatus": "processed" }] }),
			);
		let prefs = Rc::new(MemoryPrefs::new());
		let faucet = open_faucet(&script, &prefs);

		assert_eq!(
			block_on(faucet.fund(WALLET)).unwrap_err().failure,
			SurfpoolFailure::Transaction
		);
		assert_eq!(block_on(faucet.receipt_for(WALLET)).unwrap(), None);
	}

	#[test]
	fn a_lost_answer_writes_nothing() {
		let script = Rc::new(Script::default());
		script.lost.set(true);
		let prefs = Rc::new(MemoryPrefs::new());
		let faucet = open_faucet(&script, &prefs);

		assert_eq!(
			block_on(faucet.fund(WALLET)).unwrap_err().failure,
			SurfpoolFailure::Unreachable
		);
		assert!(prefs.keys().is_empty());
	}

	#[test]
	fn an_unreadable_record_is_never_quietly_reset() {
		let script = Rc::new(Script::default());
		script.answer("getGenesisHash", json!("genesis"));
		let prefs = Rc::new(MemoryPrefs::new());
		block_on(prefs.set(SurfpoolFaucet::STORAGE_KEY, json!({ "wallets": 3 }))).unwrap();
		let faucet = open_faucet(&script, &prefs);

		assert_eq!(
			block_on(faucet.fund(WALLET)).unwrap_err().failure,
			SurfpoolFailure::Record
		);
		assert_eq!(script.count("requestAirdrop"), 0);
	}

	#[test]
	fn a_blockhash_and_a_sent_transaction_are_standard_calls_too() {
		let script = Rc::new(Script::default());
		script
			.answer(
				"getLatestBlockhash",
				json!({ "value": { "blockhash": "hash", "lastValidBlockHeight": 9 } }),
			)
			.answer("sendTransaction", json!("sent"));
		let rpc = client(&script);
		assert_eq!(block_on(rpc.latest_blockhash()).unwrap(), "hash");
		assert_eq!(block_on(rpc.send_transaction(&[1, 2, 3])).unwrap(), "sent");
	}
}
