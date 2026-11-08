//! Solana wallets in the browser, through the Wallet Standard
//! (`@wallet-standard/app`'s protocol, without its JavaScript), as
//! `leptoskit_auth`'s web client reaches them: the page announces itself with
//! a `wallet-standard:app-ready` event and wallets register with
//! `wallet-standard:register-wallet`, both carrying a `register` callback.
//!
//! A wallet is used when it is on a Solana chain and can connect, sign a
//! message and sign a transaction. Proving signs the server's challenge
//! with `solana:signMessage`; a release or a burn is signed with
//! `solana:signTransaction` and sent by the game.
//!
//! `solana_kit_browser`'s wallet would do this, but at the pinned Solana Kit
//! revision it requires wasm-bindgen 0.2.128 while leptoskit's CLI embeds
//! 0.2.129, and one build can hold only one.

use std::cell::Cell;
use std::cell::RefCell;

use base64::Engine as _;
use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Reflect;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;

use super::BASE64;
use super::ChainFailure;
use super::ChainResult;
use super::ChainStatus;
use super::ChallengeFn;
use super::KeeperError;
use super::KeeperResult;
use super::WalletProof;
use super::WalletProver;

const CONNECT: &str = "standard:connect";
const SIGN_MESSAGE: &str = "solana:signMessage";
const SIGN_TRANSACTION: &str = "solana:signTransaction";

thread_local! {
	/// Every wallet that registered, in order.
	static WALLETS: RefCell<Vec<JsValue>> = const { RefCell::new(Vec::new()) };
	/// Whether this page has announced itself.
	static LISTENING: Cell<bool> = const { Cell::new(false) };
}

/// The first Solana wallet the page has.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrowserWallet;

/// Why a wallet call did not answer.
enum Thrown {
	/// The person said no, or closed it.
	Declined,
	/// It failed, or answered nonsense.
	Failed(String),
}

#[async_trait::async_trait(?Send)]
impl WalletProver for BrowserWallet {
	fn available(&self) -> bool {
		listen().is_ok() && wallet().is_some()
	}

	async fn prove(
		&self,
		status: &ChainStatus,
		challenge: ChallengeFn<'_>,
	) -> KeeperResult<WalletProof> {
		let wallet = wallet().ok_or_else(|| KeeperError::unavailable("no Solana wallet here"))?;
		let account = connect(&wallet, status, None)
			.await
			.map_err(|thrown| KeeperError::unavailable(thrown.reason()))?;
		let address = text(&account, "address")
			.ok_or_else(|| KeeperError::unavailable("an account without an address"))?;
		let issued = challenge(address).await?;
		let request = Object::new();
		set(&request, "account", &account)?;
		set(
			&request,
			"message",
			&Uint8Array::from(issued.message.as_bytes()).into(),
		)?;
		let sign = feature(&wallet, SIGN_MESSAGE)
			.ok_or_else(|| KeeperError::unavailable("the wallet cannot sign messages"))?;
		let outputs = call(&sign, "signMessage", &[request.into()])
			.await
			.map_err(|thrown| KeeperError::unavailable(thrown.reason()))?;
		let output = Array::from(&outputs).get(0);
		// Some wallets sign a prefixed message; only the exact text counts.
		if bytes(&output, "signedMessage").is_some_and(|signed| signed != issued.message.as_bytes())
		{
			return Err(KeeperError::unavailable(
				"the wallet signed a different message",
			));
		}
		let signature: [u8; 64] = bytes(&output, "signature")
			.and_then(|signature| signature.try_into().ok())
			.ok_or_else(|| KeeperError::unavailable("the wallet returned no signature"))?;
		Ok(WalletProof {
			challenge_id: issued.challenge_id,
			signature: BASE64.encode(signature),
		})
	}

	async fn sign_transaction(
		&self,
		status: &ChainStatus,
		transaction: Vec<u8>,
	) -> ChainResult<Vec<u8>> {
		let wallet = wallet().ok_or(ChainFailure::NoWallet)?;
		let sign = feature(&wallet, SIGN_TRANSACTION).ok_or(ChainFailure::NoWallet)?;
		let account = connect(&wallet, status, status.wallet.as_deref())
			.await
			.map_err(Thrown::into_failure)?;
		let request = Object::new();
		set(&request, "account", &account).map_err(|_| ChainFailure::NoWallet)?;
		set(
			&request,
			"transaction",
			&Uint8Array::from(transaction.as_slice()).into(),
		)
		.map_err(|_| ChainFailure::NoWallet)?;
		set(&request, "chain", &JsValue::from_str(&status.chain))
			.map_err(|_| ChainFailure::NoWallet)?;
		let outputs = call(&sign, "signTransaction", &[request.into()])
			.await
			.map_err(Thrown::into_failure)?;
		bytes(&Array::from(&outputs).get(0), "signedTransaction").ok_or(ChainFailure::WrongWallet)
	}
}

impl Thrown {
	fn reason(&self) -> String {
		match self {
			Self::Declined => "the wallet request was declined".to_owned(),
			Self::Failed(reason) => reason.clone(),
		}
	}

	fn into_failure(self) -> ChainFailure {
		match self {
			Self::Declined => ChainFailure::Declined,
			Self::Failed(reason) => ChainFailure::Unreachable(reason),
		}
	}
}

/// The first wallet that can do everything the game asks.
fn wallet() -> Option<JsValue> {
	listen().ok()?;
	WALLETS.with_borrow(|wallets| wallets.iter().find(|wallet| usable(wallet)).cloned())
}

/// `standard:connect`, and the account to use: the one at `address` when
/// one is named, the first Solana account otherwise.
async fn connect(
	wallet: &JsValue,
	status: &ChainStatus,
	address: Option<&str>,
) -> Result<JsValue, Thrown> {
	let connect = feature(wallet, CONNECT).ok_or_else(|| Thrown::Failed("no connect".into()))?;
	let connected = call(&connect, "connect", &[]).await?;
	let accounts = Reflect::get(&connected, &"accounts".into())
		.ok()
		.filter(Array::is_array)
		.map(|accounts| Array::from(&accounts))
		.ok_or_else(|| Thrown::Failed("no accounts".into()))?;
	accounts
		.iter()
		.find(|account| {
			match address {
				Some(wanted) => text(account, "address").as_deref() == Some(wanted),
				None => on_chain(account, &status.chain),
			}
		})
		.ok_or_else(|| Thrown::Failed("the wallet has no account to sign with".into()))
}

/// Announces the page to wallets, once, and listens for wallets that load
/// later.
fn listen() -> Result<(), ()> {
	if LISTENING.get() {
		return Ok(());
	}
	let window = web_sys::window().ok_or(())?;
	// Wallets call `register(...wallets)`; four covers every wallet seen in
	// practice, which registers one at a time.
	let register = Closure::<dyn Fn(JsValue, JsValue, JsValue, JsValue) -> JsValue>::new(
		|first: JsValue, second: JsValue, third: JsValue, fourth: JsValue| {
			WALLETS.with_borrow_mut(|wallets| {
				for wallet in [first, second, third, fourth] {
					if wallet.is_object() && !wallets.iter().any(|known| Object::is(known, &wallet))
					{
						wallets.push(wallet);
					}
				}
			});
			// What `register` returns: a function that unregisters, which the
			// page never calls.
			Closure::<dyn Fn()>::new(|| {}).into_js_value()
		},
	);
	let api = Object::new();
	Reflect::set(&api, &"register".into(), register.as_ref()).map_err(drop)?;
	// The page lives as long as these listeners.
	register.forget();

	let api_for_wallets = api.clone();
	let on_register = Closure::<dyn Fn(web_sys::Event)>::new(move |event: web_sys::Event| {
		let detail = event
			.dyn_ref::<web_sys::CustomEvent>()
			.map(web_sys::CustomEvent::detail);
		if let Some(callback) = detail.and_then(|detail| detail.dyn_into::<Function>().ok()) {
			// A wallet's own callback throwing is that wallet's problem.
			let _ = callback.call1(&JsValue::UNDEFINED, &api_for_wallets);
		}
	});
	window
		.add_event_listener_with_callback(
			"wallet-standard:register-wallet",
			on_register.as_ref().unchecked_ref(),
		)
		.map_err(drop)?;
	on_register.forget();

	let init = web_sys::CustomEventInit::new();
	init.set_detail(&api);
	let ready = web_sys::CustomEvent::new_with_event_init_dict("wallet-standard:app-ready", &init)
		.map_err(drop)?;
	window.dispatch_event(&ready).map_err(drop)?;
	LISTENING.set(true);
	Ok(())
}

/// Whether a wallet works with Solana and can do all the game asks.
fn usable(wallet: &JsValue) -> bool {
	on_chain(wallet, "solana:")
		&& [CONNECT, SIGN_MESSAGE, SIGN_TRANSACTION]
			.iter()
			.all(|name| feature(wallet, name).is_some())
}

/// Whether `object` names a chain starting with `chain`.
fn on_chain(object: &JsValue, chain: &str) -> bool {
	Reflect::get(object, &"chains".into())
		.ok()
		.filter(Array::is_array)
		.is_some_and(|chains| {
			Array::from(&chains).iter().any(|named| {
				named
					.as_string()
					.is_some_and(|named| named.starts_with(chain) || chain.starts_with(&named))
			})
		})
}

fn feature(wallet: &JsValue, name: &str) -> Option<JsValue> {
	let features = Reflect::get(wallet, &"features".into()).ok()?;
	Reflect::get(&features, &name.into())
		.ok()
		.filter(JsValue::is_object)
}

/// Calls `object.method(...arguments)` and waits for its promise.
async fn call(object: &JsValue, method: &str, arguments: &[JsValue]) -> Result<JsValue, Thrown> {
	let function: Function = Reflect::get(object, &method.into())
		.ok()
		.and_then(|function| function.dyn_into().ok())
		.ok_or_else(|| Thrown::Failed(format!("no {method}")))?;
	let arguments: Array = arguments.iter().collect();
	let promise: Promise = function
		.apply(object, &arguments)
		.map_err(|error| refused(&error))?
		.dyn_into()
		.map_err(|_| Thrown::Failed(format!("{method} returned no promise")))?;
	JsFuture::from(promise)
		.await
		.map_err(|error| refused(&error))
}

fn set(object: &Object, key: &str, value: &JsValue) -> KeeperResult<()> {
	Reflect::set(object, &key.into(), value)
		.map(drop)
		.map_err(|_| KeeperError::unavailable("the wallet request could not be built"))
}

fn text(object: &JsValue, key: &str) -> Option<String> {
	Reflect::get(object, &key.into()).ok()?.as_string()
}

fn bytes(object: &JsValue, key: &str) -> Option<Vec<u8>> {
	Reflect::get(object, &key.into())
		.ok()?
		.dyn_into::<Uint8Array>()
		.ok()
		.map(|array| array.to_vec())
}

/// What a wallet's refusal means: the person said no, or it failed.
fn refused(thrown: &JsValue) -> Thrown {
	let code = Reflect::get(thrown, &"code".into())
		.ok()
		.and_then(|code| code.as_f64());
	let words = [
		text(thrown, "name").unwrap_or_default(),
		text(thrown, "message").unwrap_or_default(),
	]
	.join(" ");
	let lower = words.to_lowercase();
	// 4001 is the provider-wide "user rejected the request".
	if code == Some(4001.0) || lower.contains("reject") || lower.contains("cancel") {
		Thrown::Declined
	} else {
		Thrown::Failed(words)
	}
}
