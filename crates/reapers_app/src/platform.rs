//! What the game asks of whatever it runs on: the time, a wait, a task in
//! the background, randomness, the save, the network, a link and a tap of
//! the motor.
//!
//! There are three hosts, and each answers differently:
//!
//! - the browser (`web`): `Date.now`, `setTimeout`, `localStorage`, `fetch`;
//! - a native app's Wasm guest (`native`): the clock the host reports before
//!   every entry point, its timer, store, HTTP, crypto, linking and haptics
//!   plugins;
//! - anything else, which is the tests: the system clock, a pool the test
//!   runs by hand, and no timers, so nothing waits on the wall clock.
//!
//! Screens and keepers never name a host; they call these functions.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use reapers_rules::Timestamp;
use serde_json::Value;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod host {
	pub use super::web::*;
}

#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
mod host {
	pub use super::guest::*;
}

#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
mod host {
	pub use super::local::*;
}

/// Something the platform could not do.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PlatformError {
	/// This host has no such capability, or it is not granted.
	#[error("unavailable on this platform: {0}")]
	Unavailable(&'static str),

	/// The capability is there and failed.
	#[error("{0}")]
	Failed(String),
}

/// The wall clock.
pub fn now() -> Timestamp {
	Timestamp::from_millis(host::now_millis())
}

/// Runs `task` in the background on this host's executor.
pub fn spawn(task: impl Future<Output = ()> + 'static) {
	host::spawn(task);
}

/// Waits for `duration`. Returns `false`, at once, where this host has no
/// timer: a loop that ticks on it should stop rather than spin.
pub async fn sleep(duration: Duration) -> bool {
	host::sleep(duration).await
}

/// `N` bytes from the platform's secure random source.
pub async fn random_bytes<const N: usize>() -> Result<[u8; N], PlatformError> {
	host::random_bytes::<N>().await
}

/// A fresh seed for the rules: 32 random bits, as the Dart keeper drew them.
pub async fn random_seed() -> Result<i64, PlatformError> {
	let bytes = random_bytes::<4>().await?;
	Ok(i64::from(u32::from_le_bytes(bytes)))
}

/// A light tap under the thumb, where the device has a motor.
pub fn tap() {
	host::haptic(false);
}

/// A heavy thud: a reaper bound, a husk surfacing.
pub fn thud() {
	host::haptic(true);
}

/// Opens `url` outside the game: the browser, or the phone's own handler.
pub async fn open_url(url: &str) -> Result<(), PlatformError> {
	host::open_url(url).await
}

/// Writes `message` to the developer's log: the browser's console, the
/// standard error of a test. A player never reads it.
pub fn log(message: &str) {
	host::log(message);
}

/// Whether the person has asked their device for less motion.
pub fn reduced_motion() -> bool {
	host::reduced_motion()
}

/// The screen's size in layout units, width then height: the browser's
/// window, or the phone screen the native apps are laid out for.
pub fn viewport() -> (f64, f64) {
	host::viewport()
}

/// The phone screen the apps are laid out for (`[native] viewport_width`
/// and `viewport_height` in leptoskit.toml).
const PHONE: (f64, f64) = (390.0, 844.0);

/// One HTTP exchange's answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpReply {
	pub status: u16,
	pub body: Vec<u8>,
}

/// One request header: its name, in lower case, and its value.
pub type Header = (&'static str, String);

/// Something that posts a body to a URL and returns the reply.
///
/// The Solana RPC clients and the server client take one, so their tests
/// hand in a fake or a desktop client and the game hands in [`DeviceHttp`].
#[async_trait(?Send)]
pub trait HttpTransport {
	/// Posts `body` to `url` with `headers`.
	async fn post(
		&self,
		url: &str,
		headers: Vec<Header>,
		body: Vec<u8>,
		timeout: Duration,
	) -> Result<HttpReply, PlatformError>;

	/// Posts a JSON `body` to `url`.
	async fn post_json(
		&self,
		url: &str,
		body: Vec<u8>,
		timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		self.post(
			url,
			vec![("content-type", "application/json".to_owned())],
			body,
			timeout,
		)
		.await
	}
}

/// The host's own HTTP: `fetch` in a browser, the HTTP plugin in an app.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceHttp;

#[async_trait(?Send)]
impl HttpTransport for DeviceHttp {
	async fn post(
		&self,
		url: &str,
		headers: Vec<Header>,
		body: Vec<u8>,
		timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		host::post(url, headers, body, timeout).await
	}
}

/// Where the game keeps what it must remember between launches: the save,
/// the simnet's ledger, a setting or two. Values are JSON.
#[async_trait(?Send)]
pub trait Prefs {
	async fn get(&self, key: &str) -> Result<Option<Value>, PlatformError>;
	async fn set(&self, key: &str, value: Value) -> Result<(), PlatformError>;
	async fn remove(&self, key: &str) -> Result<(), PlatformError>;
}

/// Preferences held in memory and forgotten when dropped: for tests, and for
/// a host that cannot keep anything.
#[derive(Debug, Default)]
pub struct MemoryPrefs(RefCell<BTreeMap<String, Value>>);

impl MemoryPrefs {
	pub fn new() -> Self {
		Self::default()
	}

	/// Every key held, in order.
	pub fn keys(&self) -> Vec<String> {
		self.0.borrow().keys().cloned().collect()
	}
}

#[async_trait(?Send)]
impl Prefs for MemoryPrefs {
	async fn get(&self, key: &str) -> Result<Option<Value>, PlatformError> {
		Ok(self.0.borrow().get(key).cloned())
	}

	async fn set(&self, key: &str, value: Value) -> Result<(), PlatformError> {
		self.0.borrow_mut().insert(key.to_owned(), value);
		Ok(())
	}

	async fn remove(&self, key: &str) -> Result<(), PlatformError> {
		self.0.borrow_mut().remove(key);
		Ok(())
	}
}

/// The device's own store: `localStorage` in a browser, the store plugin's
/// file in an app. Every write is committed at once, so a closed app keeps
/// what it was last told.
#[derive(Debug, Default)]
pub struct DevicePrefs;

#[async_trait(?Send)]
impl Prefs for DevicePrefs {
	async fn get(&self, key: &str) -> Result<Option<Value>, PlatformError> {
		host::store_get(key).await
	}

	async fn set(&self, key: &str, value: Value) -> Result<(), PlatformError> {
		host::store_set(key, Some(value)).await
	}

	async fn remove(&self, key: &str) -> Result<(), PlatformError> {
		host::store_set(key, None).await
	}
}

/// Where the game keeps a secret it must remember between launches: the
/// server session's refresh token. A native app keeps it in the platform's
/// secure store (the Keychain, the Android Keystore); a browser has none,
/// so it stays in `localStorage` beside the save, as a web page's session
/// does.
#[derive(Debug, Default)]
pub struct DeviceSecrets;

#[async_trait(?Send)]
impl Prefs for DeviceSecrets {
	async fn get(&self, key: &str) -> Result<Option<Value>, PlatformError> {
		host::secret_get(key).await
	}

	async fn set(&self, key: &str, value: Value) -> Result<(), PlatformError> {
		host::secret_set(key, Some(value)).await
	}

	async fn remove(&self, key: &str) -> Result<(), PlatformError> {
		host::secret_set(key, None).await
	}
}

fn failed(error: impl std::fmt::Display) -> PlatformError {
	PlatformError::Failed(error.to_string())
}

/// The origin of `url`, which the browser's HTTP client must be told before
/// it will send anything there.
#[cfg_attr(not(all(target_arch = "wasm32", feature = "web")), allow(dead_code))]
fn origin_of(url: &str) -> Option<String> {
	let (scheme, rest) = url.split_once("://")?;
	let host = rest.split(['/', '?', '#']).next()?;
	(!host.is_empty()).then(|| format!("{scheme}://{host}"))
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod web {
	use std::cell::RefCell;
	use std::future::Future;
	use std::time::Duration;

	use leptoskit_plugin_store::Change;
	use leptoskit_plugin_store::Save;
	use serde_json::Value;
	use wasm_bindgen::JsCast;
	use wasm_bindgen::prelude::Closure;

	use super::HttpReply;
	use super::PlatformError;
	use super::failed;

	pub fn now_millis() -> i64 {
		js_sys::Date::now() as i64
	}

	pub fn spawn(task: impl Future<Output = ()> + 'static) {
		wasm_bindgen_futures::spawn_local(task);
	}

	pub async fn sleep(duration: Duration) -> bool {
		let Some(window) = web_sys::window() else {
			return false;
		};
		let millis = i32::try_from(duration.as_millis()).unwrap_or(i32::MAX);
		let promise = js_sys::Promise::new(&mut |resolve, _reject| {
			let done = Closure::once_into_js(move || {
				let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
			});
			let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
				done.unchecked_ref(),
				millis,
			);
		});
		wasm_bindgen_futures::JsFuture::from(promise).await.is_ok()
	}

	#[allow(
		clippy::unused_async,
		reason = "the guest's answer is async, and both share one signature"
	)]
	pub async fn random_bytes<const N: usize>() -> Result<[u8; N], PlatformError> {
		let mut bytes = [0; N];
		leptoskit_plugin_crypto::fill_random(&mut bytes).map_err(failed)?;
		Ok(bytes)
	}

	pub fn haptic(heavy: bool) {
		let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
			return;
		};
		let _ = navigator.vibrate_with_duration(if heavy { 24 } else { 6 });
	}

	#[allow(
		clippy::unused_async,
		reason = "the guest's answer is async, and both share one signature"
	)]
	pub async fn open_url(url: &str) -> Result<(), PlatformError> {
		let window = web_sys::window().ok_or(PlatformError::Unavailable("window"))?;
		window
			.open_with_url_and_target(url, "_blank")
			.map_err(|_| PlatformError::Failed("the browser refused to open the link".into()))?;
		Ok(())
	}

	pub fn log(message: &str) {
		web_sys::console::warn_1(&message.into());
	}

	pub fn viewport() -> (f64, f64) {
		let size =
			|value: Result<wasm_bindgen::JsValue, _>| value.ok().and_then(|value| value.as_f64());
		web_sys::window()
			.and_then(|window| Some((size(window.inner_width())?, size(window.inner_height())?)))
			.filter(|(width, height)| *width > 0.0 && *height > 0.0)
			.unwrap_or(super::PHONE)
	}

	pub fn reduced_motion() -> bool {
		web_sys::window()
			.and_then(|window| {
				window
					.match_media("(prefers-reduced-motion: reduce)")
					.ok()
					.flatten()
			})
			.is_some_and(|query| query.matches())
	}

	pub async fn post(
		url: &str,
		headers: Vec<super::Header>,
		body: Vec<u8>,
		timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		use leptoskit_plugin_http::Client;
		use leptoskit_plugin_http::Config;
		use leptoskit_plugin_http::Header;
		use leptoskit_plugin_http::Method;
		use leptoskit_plugin_http::Request;

		let origin = super::origin_of(url)
			.ok_or_else(|| PlatformError::Failed(format!("not a URL: {url}")))?;
		let mut config = Config::new([origin]).map_err(failed)?;
		config.timeout = timeout;
		let client = Client::new(config).map_err(failed)?;
		let reply = client
			.execute(Request {
				method: Method::Post,
				url: url.to_owned(),
				headers: headers
					.into_iter()
					.map(|(name, value)| {
						Header {
							name: name.into(),
							value: value.into_bytes(),
						}
					})
					.collect(),
				body,
			})
			.await
			.map_err(failed)?;
		Ok(HttpReply {
			status: reply.status,
			body: reply.body,
		})
	}

	/// A browser has no secure store: secrets stay in `localStorage`.
	pub async fn secret_get(key: &str) -> Result<Option<Value>, PlatformError> {
		store_get(key).await
	}

	pub async fn secret_set(key: &str, value: Option<Value>) -> Result<(), PlatformError> {
		store_set(key, value).await
	}

	thread_local! {
		static STORE: RefCell<Option<leptoskit_plugin_store::web::Store>> = const { RefCell::new(None) };
	}

	fn with_store<T, E: std::fmt::Display>(
		use_store: impl FnOnce(&mut leptoskit_plugin_store::web::Store) -> Result<T, E>,
	) -> Result<T, PlatformError> {
		STORE.with_borrow_mut(|store| {
			if store.is_none() {
				*store = Some(leptoskit_plugin_store::web::Store::open().map_err(failed)?);
			}
			let store = store
				.as_mut()
				.ok_or(PlatformError::Unavailable("localStorage"))?;
			use_store(store).map_err(failed)
		})
	}

	#[allow(
		clippy::unused_async,
		reason = "the guest's store is async, and both share one signature"
	)]
	pub async fn store_get(key: &str) -> Result<Option<Value>, PlatformError> {
		with_store(|store| store.get(key).map(Option::<&Value>::cloned))
	}

	#[allow(
		clippy::unused_async,
		reason = "the guest's store is async, and both share one signature"
	)]
	pub async fn store_set(key: &str, value: Option<Value>) -> Result<(), PlatformError> {
		let change = match value {
			Some(value) => Change::Set(key.to_owned(), value),
			None => Change::Remove(key.to_owned()),
		};
		with_store(|store| store.apply(&[change], Save::Commit))
	}
}

#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
mod guest {
	use std::future::Future;
	use std::sync::atomic::AtomicI64;
	use std::sync::atomic::Ordering;
	use std::time::Duration;

	use leptoskit_plugin_store::Change;
	use leptoskit_plugin_store::Save;
	use serde_json::Value;

	use super::HttpReply;
	use super::PlatformError;
	use super::failed;

	/// The wall clock the host last reported, in milliseconds since 1970.
	static CLOCK: AtomicI64 = AtomicI64::new(0);

	/// Records the host's report of the time. `crate::guest` calls this from
	/// the `leptoskit_clock` export, before every entry point.
	pub fn report_clock(unix_millis: i64) {
		if unix_millis > 0 {
			CLOCK.store(unix_millis, Ordering::Relaxed);
		}
	}

	pub fn now_millis() -> i64 {
		CLOCK.load(Ordering::Relaxed)
	}

	pub fn spawn(task: impl Future<Output = ()> + 'static) {
		leptoskit::guest::spawn(task);
	}

	pub async fn sleep(duration: Duration) -> bool {
		leptoskit::guest::sleep(duration).await.is_ok()
	}

	pub async fn random_bytes<const N: usize>() -> Result<[u8; N], PlatformError> {
		let crypto = leptoskit_plugin_crypto::guest::Crypto::new().map_err(failed)?;
		let bytes = crypto.random_bytes(N).await.map_err(failed)?;
		<[u8; N]>::try_from(&bytes[..])
			.map_err(|_| PlatformError::Failed("short random read".into()))
	}

	pub fn haptic(heavy: bool) {
		use leptoskit_plugin_haptics::Impact;
		use leptoskit_plugin_haptics::guest::Haptics;

		leptoskit::guest::spawn(async move {
			if let Ok(haptics) = Haptics::new() {
				let _ = if heavy {
					haptics.impact(Impact::Heavy).await
				} else {
					haptics.selection().await
				};
			}
		});
	}

	pub async fn open_url(url: &str) -> Result<(), PlatformError> {
		use leptoskit_plugin_linking::Link;
		use leptoskit_plugin_linking::guest::Linking;

		let link = Link::parse(url).map_err(failed)?;
		Linking::new()
			.map_err(failed)?
			.open(&link)
			.await
			.map_err(failed)
	}

	pub fn reduced_motion() -> bool {
		false
	}

	/// The guest has no console of its own; the host keeps no log of it yet.
	pub fn log(_message: &str) {}

	pub fn viewport() -> (f64, f64) {
		leptoskit::guest::viewport()
			.filter(|(width, height)| *width > 0.0 && *height > 0.0)
			.map_or(super::PHONE, |(width, height)| {
				(f64::from(width), f64::from(height))
			})
	}

	pub async fn post(
		url: &str,
		headers: Vec<super::Header>,
		body: Vec<u8>,
		_timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		use leptoskit_plugin_http::Header;
		use leptoskit_plugin_http::Method;
		use leptoskit_plugin_http::Request;
		use leptoskit_plugin_http::guest::HttpClient;

		let client = HttpClient::connect().map_err(failed)?;
		let reply = client
			.execute(Request {
				method: Method::Post,
				url: url.to_owned(),
				headers: headers
					.into_iter()
					.map(|(name, value)| {
						Header {
							name: name.into(),
							value: value.into_bytes(),
						}
					})
					.collect(),
				body,
			})
			.await
			.map_err(failed)?;
		Ok(HttpReply {
			status: reply.status,
			body: reply.body,
		})
	}

	fn secure_store() -> Result<leptoskit_plugin_secure_store::guest::SecureStore, PlatformError> {
		leptoskit_plugin_secure_store::guest::SecureStore::new().map_err(failed)
	}

	fn secret_key(key: &str) -> Result<leptoskit_plugin_secure_store::Key<'_>, PlatformError> {
		leptoskit_plugin_secure_store::Key::new("reapers", key).map_err(failed)
	}

	/// The platform's secure store: the Keychain, the Android Keystore.
	pub async fn secret_get(key: &str) -> Result<Option<Value>, PlatformError> {
		let Some(bytes) = secure_store()?
			.get(secret_key(key)?)
			.await
			.map_err(failed)?
		else {
			return Ok(None);
		};
		serde_json::from_slice(&bytes).map(Some).map_err(failed)
	}

	pub async fn secret_set(key: &str, value: Option<Value>) -> Result<(), PlatformError> {
		let store = secure_store()?;
		match value {
			Some(value) => {
				let bytes = serde_json::to_vec(&value).map_err(failed)?;
				store.set(secret_key(key)?, &bytes).await.map_err(failed)
			}
			None => store.delete(secret_key(key)?).await.map_err(failed),
		}
	}

	fn store() -> Result<leptoskit_plugin_store::guest::Store, PlatformError> {
		leptoskit_plugin_store::guest::Store::new().map_err(failed)
	}

	pub async fn store_get(key: &str) -> Result<Option<Value>, PlatformError> {
		store()?.get(key).await.map_err(failed)
	}

	pub async fn store_set(key: &str, value: Option<Value>) -> Result<(), PlatformError> {
		let change = match value {
			Some(value) => Change::Set(key.to_owned(), value),
			None => Change::Remove(key.to_owned()),
		};
		store()?
			.apply(&[change], Save::Commit)
			.await
			.map_err(failed)
	}
}

#[cfg(all(target_arch = "wasm32", feature = "native", not(feature = "web")))]
pub use guest::report_clock;

/// The tests' host: the system clock, a pool run by hand and no timers.
#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
mod local {
	use std::cell::RefCell;
	use std::future::Future;
	use std::time::Duration;

	use futures::executor::LocalPool;
	use futures::executor::LocalSpawner;
	use futures::task::LocalSpawnExt as _;
	use serde_json::Value;

	use super::HttpReply;
	use super::PlatformError;

	thread_local! {
		static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
		// Taken once, so a task that spawns another while the pool runs it
		// need not borrow the pool.
		static SPAWNER: LocalSpawner = POOL.with_borrow(LocalPool::spawner);
	}

	pub fn now_millis() -> i64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map_or(0, |elapsed| elapsed.as_millis() as i64)
	}

	pub fn spawn(task: impl Future<Output = ()> + 'static) {
		SPAWNER.with(|spawner| {
			let _ = spawner.spawn_local(task);
		});
	}

	/// Runs every spawned task that can make progress.
	pub fn run_until_stalled() {
		POOL.with_borrow_mut(LocalPool::run_until_stalled);
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn sleep(_duration: Duration) -> bool {
		false
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn random_bytes<const N: usize>() -> Result<[u8; N], PlatformError> {
		let mut bytes = [0; N];
		leptoskit_plugin_crypto::fill_random(&mut bytes).map_err(super::failed)?;
		Ok(bytes)
	}

	pub fn haptic(_heavy: bool) {}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn open_url(_url: &str) -> Result<(), PlatformError> {
		Err(PlatformError::Unavailable("links"))
	}

	pub fn reduced_motion() -> bool {
		false
	}

	pub fn log(message: &str) {
		eprintln!("{message}");
	}

	pub const fn viewport() -> (f64, f64) {
		super::PHONE
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn post(
		_url: &str,
		_headers: Vec<super::Header>,
		_body: Vec<u8>,
		_timeout: Duration,
	) -> Result<HttpReply, PlatformError> {
		Err(PlatformError::Unavailable("http"))
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn secret_get(_key: &str) -> Result<Option<Value>, PlatformError> {
		Err(PlatformError::Unavailable("secure store"))
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn secret_set(_key: &str, _value: Option<Value>) -> Result<(), PlatformError> {
		Err(PlatformError::Unavailable("secure store"))
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn store_get(_key: &str) -> Result<Option<Value>, PlatformError> {
		Err(PlatformError::Unavailable("store"))
	}

	#[allow(clippy::unused_async, reason = "every host shares one signature")]
	pub async fn store_set(_key: &str, _value: Option<Value>) -> Result<(), PlatformError> {
		Err(PlatformError::Unavailable("store"))
	}
}

/// Runs every task spawned so far that can make progress. Only the tests'
/// host has a pool to run; the browser and the app run their own.
#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
pub fn run_until_stalled() {
	local::run_until_stalled();
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_origin_is_the_scheme_and_the_host() {
		assert_eq!(
			origin_of("http://localhost:8899"),
			Some("http://localhost:8899".into())
		);
		assert_eq!(
			origin_of("https://rpc.example/path?x=1"),
			Some("https://rpc.example".into())
		);
		assert_eq!(origin_of("nonsense"), None);
	}

	#[test]
	fn memory_prefs_keep_and_forget() {
		let prefs = MemoryPrefs::new();
		futures::executor::block_on(async {
			prefs.set("a", Value::from(1)).await.unwrap();
			assert_eq!(prefs.get("a").await.unwrap(), Some(Value::from(1)));
			prefs.remove("a").await.unwrap();
			assert_eq!(prefs.get("a").await.unwrap(), None);
		});
	}

	#[test]
	fn a_seed_is_32_bits() {
		let seed = futures::executor::block_on(random_seed()).unwrap();
		assert!((0..1 << 32).contains(&seed));
	}
}
