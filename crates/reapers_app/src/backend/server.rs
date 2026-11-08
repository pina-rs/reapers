//! The Reapers server's client: [`ReapersApi`] over `reapers_api`'s
//! operations.
//!
//! Every operation is `reapers_api`'s own argument struct, sent the way
//! `leptoskit_rpc`'s client sends one: `POST /api/<name>` with the arguments
//! as JSON, a fresh idempotency key on every mutation, and the session's
//! access token as `Authorization: Bearer`. A failure is read with
//! `leptoskit_rpc`'s own wire decoding, and a refusal by
//! `reapers_api::refusal_of`. The requests go through the game's
//! [`HttpTransport`], so the browser, the native guests and a desktop test
//! run the same client against the same server.
//!
//! Signing in is a native app's, as `leptoskit_auth`'s client does it: the
//! access token is held in memory, the refresh token is kept in the
//! device's secret store ([`crate::platform::DeviceSecrets`]), and the pair
//! is traded at `leptoskit_auth`'s `/auth/refresh` before the access token
//! runs out or after the server refuses it, one trade at a time. A fresh
//! install opens a guest account at `/auth/guest`; proving a wallet at
//! `/auth/wallet` switches the session to the wallet's account. The web
//! game signs in the same way: it is served from its own origin, so it
//! carries a bearer token rather than a cookie.
//!
//! A rift is `enter_rift` on `leptoskit_rpc`'s live socket (`/live`), signed
//! with the same token, with each report going up as `report_rift`. Only
//! the browser and the native guests have a socket.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt as _;
use futures::stream::LocalBoxStream;
use reapers_api::Shared;
use reapers_api::auth::SignedIn;
use reapers_api::auth::WalletSignIn;
use reapers_api::chain::Chain;
use reapers_api::chain::ChainStatus as ChainStatusOp;
use reapers_api::chain::ImportReaper;
use reapers_api::chain::WalletChallenge as WalletChallengeOp;
use reapers_api::player;
use reapers_api::player::ChainReceipt;
use reapers_api::player::SettledHunt;
use reapers_api::rift::ReportRift;
use reapers_api::rpc::FnError;
use reapers_api::rpc::Kind;
use reapers_api::rpc::Operation;
use reapers_rules::GeoFix;
use reapers_rules::PlayerState;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;
use reapers_rules::Timestamp;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::remote::ApiError;
use super::remote::ApiResult;
use super::remote::ChainStatus;
use super::remote::ReapersApi;
use super::remote::WalletChallenge;
use crate::platform;
use crate::platform::Header;
use crate::platform::HttpReply;
use crate::platform::HttpTransport;
use crate::platform::Prefs;

/// How long one request to the server may take.
const TIMEOUT: Duration = Duration::from_secs(20);

/// How long before the access token runs out it is traded, at most.
const RENEW_EARLY: i64 = 60;

/// The access token's header, shared with the live socket's credentials,
/// which must be `Send`.
type SharedToken = Arc<Mutex<Option<String>>>;

/// The session: the access token in memory, the refresh token in the
/// secret store.
struct Session {
	secrets: Rc<dyn Prefs>,
	access: RefCell<Option<Access>>,
	/// Held while a sign-in or a trade is in flight, so concurrent requests
	/// wait for it instead of spending the refresh token twice.
	renewing: futures::lock::Mutex<()>,
	/// The bearer header the live socket signs with.
	shared: SharedToken,
}

struct Access {
	header: String,
	renew_at: Timestamp,
}

/// What `/auth/refresh` answers.
#[derive(Deserialize)]
struct Tokens {
	access_token: String,
	refresh_token: String,
	expires_in: u64,
}

impl From<SignedIn> for Tokens {
	fn from(signed_in: SignedIn) -> Self {
		Self {
			access_token: signed_in.access_token,
			refresh_token: signed_in.refresh_token,
			expires_in: signed_in.expires_in,
		}
	}
}

#[derive(Serialize)]
struct RefreshTokens<'a> {
	refresh_token: &'a str,
}

impl Session {
	/// Where the refresh token is kept.
	const REFRESH_KEY: &'static str = "reapers.server.refresh";

	fn fresh(&self) -> Option<String> {
		let now = platform::now();
		self.access
			.borrow()
			.as_ref()
			.filter(|access| now < access.renew_at)
			.map(|access| access.header.clone())
	}

	fn current(&self) -> Option<String> {
		self.access
			.borrow()
			.as_ref()
			.map(|access| access.header.clone())
	}

	async fn hold(&self, tokens: Tokens) -> ApiResult<()> {
		let lifetime = i64::try_from(tokens.expires_in).unwrap_or(i64::MAX);
		let early = RENEW_EARLY.min(lifetime / 2);
		let header = format!("Bearer {}", tokens.access_token);
		*self.access.borrow_mut() = Some(Access {
			header: header.clone(),
			renew_at: platform::now().plus_seconds(lifetime - early),
		});
		*self
			.shared
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner) = Some(header);
		live::reauthenticate();
		self.secrets
			.set(Self::REFRESH_KEY, Value::from(tokens.refresh_token))
			.await
			.map_err(unavailable)
	}

	async fn forget(&self) {
		*self.access.borrow_mut() = None;
		*self
			.shared
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner) = None;
		let _ = self.secrets.remove(Self::REFRESH_KEY).await;
	}

	async fn refresh_token(&self) -> Option<String> {
		match self.secrets.get(Self::REFRESH_KEY).await {
			Ok(Some(Value::String(token))) if !token.is_empty() => Some(token),
			_ => None,
		}
	}
}

/// The server at one origin, over one transport. Clones share the session.
#[derive(Clone)]
pub struct ServerApi(Rc<Client>);

struct Client {
	origin: String,
	http: Rc<dyn HttpTransport>,
	session: Session,
}

impl ServerApi {
	/// A client for the server at `url`, sending through `http` and keeping
	/// the refresh token in `secrets`.
	pub fn new(url: &str, http: Rc<dyn HttpTransport>, secrets: Rc<dyn Prefs>) -> Self {
		let origin = url.trim_end_matches('/').to_owned();
		let shared = SharedToken::default();
		live::configure(&origin, &shared);
		Self(Rc::new(Client {
			origin,
			http,
			session: Session {
				secrets,
				access: RefCell::new(None),
				renewing: futures::lock::Mutex::new(()),
				shared,
			},
		}))
	}
}

impl Client {
	/// The access token's header, signing in or trading the refresh token
	/// first when it has run out.
	async fn authorization(&self) -> ApiResult<String> {
		if let Some(header) = self.session.fresh() {
			return Ok(header);
		}
		let _renewing = self.session.renewing.lock().await;
		// Another request may have signed in while this one waited.
		if let Some(header) = self.session.fresh() {
			return Ok(header);
		}
		self.renew().await?;
		self.session
			.current()
			.ok_or_else(|| ApiError::Unavailable("no session".into()))
	}

	/// Trades the stored refresh token for a new pair, or opens a guest
	/// account when there is none or the server has ended its session. Call
	/// it holding `renewing`.
	async fn renew(&self) -> ApiResult<()> {
		if let Some(refresh_token) = self.session.refresh_token().await {
			let reply = self
				.post(
					reapers_api::auth::REFRESH,
					None,
					None,
					&RefreshTokens {
						refresh_token: &refresh_token,
					},
				)
				.await?;
			match reply.status {
				200..=299 => return self.session.hold(decode(&reply)?).await,
				// The session is over: start again as a new guest. A server
				// that could not answer may still know the token, so it is
				// kept for the next try.
				400..=499 => self.session.forget().await,
				_ => return Err(failure(&reply)),
			}
		}
		let reply = self
			.post(reapers_api::auth::GUEST, None, None, &Value::Null)
			.await?;
		if !(200..300).contains(&reply.status) {
			return Err(failure(&reply));
		}
		let signed_in: SignedIn = decode(&reply)?;
		self.session.hold(signed_in.into()).await
	}

	async fn post(
		&self,
		path: &str,
		authorization: Option<String>,
		idempotency_key: Option<String>,
		body: &impl Serialize,
	) -> ApiResult<HttpReply> {
		let mut headers: Vec<Header> = vec![
			("accept", "application/json".to_owned()),
			("content-type", "application/json".to_owned()),
		];
		if let Some(authorization) = authorization {
			headers.push(("authorization", authorization));
		}
		if let Some(key) = idempotency_key {
			headers.push(("idempotency-key", key));
		}
		let body = serde_json::to_vec(body).map_err(unavailable)?;
		self.http
			.post(&format!("{}{path}", self.origin), headers, body, TIMEOUT)
			.await
			.map_err(unavailable)
	}

	/// Sends `body` to the sign-in route `path` as the signed-in session,
	/// trading the token once if the server refuses it.
	async fn signed_post(
		&self,
		path: &str,
		key: Option<String>,
		body: &impl Serialize,
	) -> ApiResult<HttpReply> {
		let sent = self.authorization().await?;
		let reply = self
			.post(path, Some(sent.clone()), key.clone(), body)
			.await?;
		if reply.status != 401 {
			return Ok(reply);
		}
		{
			let _renewing = self.session.renewing.lock().await;
			// Renewed since this request was sent, or renew now.
			if self.session.current().is_none_or(|current| current == sent) {
				self.renew().await?;
			}
		}
		let authorization = self.authorization().await?;
		self.post(path, Some(authorization), key, body).await
	}

	/// Calls the operation `op` as the signed-in player.
	async fn call<O: Operation>(&self, op: O) -> ApiResult<O::Output> {
		let key = match O::KIND {
			Kind::Mutation => Some(idempotency_key().await?),
			_ => None,
		};
		let reply = self
			.signed_post(&format!("/api/{}", O::NAME), key, &op)
			.await?;
		if (200..300).contains(&reply.status) {
			let value: Value = if reply.body.iter().all(u8::is_ascii_whitespace) {
				Value::Null
			} else {
				serde_json::from_slice(&reply.body).map_err(unavailable)?
			};
			serde_json::from_value(value).map_err(unavailable)
		} else {
			Err(failure(&reply))
		}
	}
}

/// A fresh idempotency key: 128 random bits, in hexadecimal.
async fn idempotency_key() -> ApiResult<String> {
	let bytes = platform::random_bytes::<16>().await.map_err(unavailable)?;
	Ok(bytes.iter().fold(String::new(), |mut key, byte| {
		use std::fmt::Write as _;
		let _ = write!(key, "{byte:02x}");
		key
	}))
}

fn decode<T: DeserializeOwned>(reply: &HttpReply) -> ApiResult<T> {
	serde_json::from_slice(&reply.body).map_err(unavailable)
}

/// What a failed answer says: a refusal of the rules by name, or the server
/// unavailable. An operation's failure is `leptoskit_rpc`'s wire text; a
/// sign-in route's is a `SignInRefused` body.
fn failure(reply: &HttpReply) -> ApiError {
	let text = String::from_utf8_lossy(&reply.body);
	let message = serde_json::from_str::<Value>(&text)
		.ok()
		.and_then(|body| body.get("error")?.as_str().map(str::to_owned))
		.unwrap_or_else(|| FnError::from_wire(&text).message().to_owned());
	refusal_or(&message, || {
		format!("the server answered HTTP {}: {message}", reply.status)
	})
}

fn refusal_or(message: &str, otherwise: impl FnOnce() -> String) -> ApiError {
	match message.strip_prefix(reapers_api::REFUSED) {
		Some(name) => ApiError::Refused(name.to_owned()),
		None => ApiError::Unavailable(otherwise()),
	}
}

fn unavailable(error: impl std::fmt::Display) -> ApiError {
	ApiError::Unavailable(error.to_string())
}

impl From<Chain> for ChainStatus {
	fn from(chain: Chain) -> Self {
		Self {
			chain: chain.chain,
			identity_uri: chain.identity_uri,
			wallet: chain.wallet,
			lamports: chain.lamports.map(|lamports| lamports as i64),
			collection: chain.collection,
			marketplace_uri: chain.marketplace_uri,
			program: chain.program,
			husks: chain.husks,
		}
	}
}

#[async_trait(?Send)]
impl ReapersApi for ServerApi {
	async fn sign_in(&self) -> ApiResult<()> {
		self.0.authorization().await.map(drop)
	}

	async fn load(&self) -> ApiResult<PlayerState> {
		self.0.call(player::Load {}).await
	}

	async fn update_location(&self, fix: Option<GeoFix>) -> ApiResult<()> {
		self.0
			.call(player::UpdateLocation {
				fix: fix.map(Shared),
			})
			.await
	}

	async fn take_oath(&self) -> ApiResult<PlayerState> {
		self.0.call(player::TakeOath {}).await
	}

	async fn select_reaper(&self, id: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::SelectReaper { id: id.to_owned() })
			.await
	}

	async fn acquire(&self, nft_id: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::Acquire {
				nft: nft_id.to_owned(),
			})
			.await
	}

	async fn release(&self, nft_id: &str, escrow: Option<&str>) -> ApiResult<PlayerState> {
		self.0
			.call(player::Release {
				nft: nft_id.to_owned(),
				escrow: escrow.map(str::to_owned),
			})
			.await
	}

	async fn settle_hunt(&self, sighting_id: i64, log: Vec<u8>) -> ApiResult<SettledHunt> {
		self.0
			.call(player::SettleHunt {
				sighting: sighting_id,
				log: player::encode_log(&log),
			})
			.await
	}

	async fn absorb_transaction(&self, nft_id: &str) -> ApiResult<ChainReceipt> {
		self.0
			.call(player::AbsorbTransaction {
				nft: nft_id.to_owned(),
			})
			.await
	}

	async fn buy_weapon(&self, weapon_id: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::BuyWeapon {
				weapon: weapon_id.to_owned(),
			})
			.await
	}

	async fn wield(&self, weapon_id: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::Wield {
				weapon: weapon_id.to_owned(),
			})
			.await
	}

	async fn buy_outfit(&self, outfit: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::BuyOutfit {
				outfit: outfit.to_owned(),
			})
			.await
	}

	async fn wear(&self, outfit: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::Wear {
				outfit: outfit.to_owned(),
			})
			.await
	}

	async fn attune(&self, art: &str) -> ApiResult<PlayerState> {
		self.0
			.call(player::Attune {
				art: art.to_owned(),
			})
			.await
	}

	async fn chain_status(&self) -> ApiResult<ChainStatus> {
		self.0.call(ChainStatusOp {}).await.map(ChainStatus::from)
	}

	async fn import_reaper(&self, asset: &str) -> ApiResult<PlayerState> {
		self.0
			.call(ImportReaper {
				asset: asset.to_owned(),
			})
			.await
	}

	async fn challenge(&self, address: &str) -> ApiResult<WalletChallenge> {
		self.0
			.call(WalletChallengeOp {
				address: address.to_owned(),
			})
			.await
			.map(|challenge| {
				WalletChallenge {
					challenge_id: challenge.challenge_id,
					message: challenge.message,
				}
			})
	}

	async fn login(&self, challenge_id: &str, signature: &str) -> ApiResult<()> {
		let reply = self
			.0
			.signed_post(
				reapers_api::auth::WALLET,
				None,
				&WalletSignIn {
					challenge_id: challenge_id.to_owned(),
					signature: signature.to_owned(),
				},
			)
			.await?;
		if !(200..300).contains(&reply.status) {
			return Err(failure(&reply));
		}
		let signed_in: SignedIn = decode(&reply)?;
		let _renewing = self.0.session.renewing.lock().await;
		self.0.session.hold(signed_in.into()).await
	}

	async fn enter_rift(
		&self,
		site: &str,
		demo: bool,
		reports: LocalBoxStream<'static, RiftReport>,
	) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>> {
		// The socket signs with the token as it stands.
		self.0.authorization().await?;
		let frames = live::enter_rift(site, demo)?;
		let reporter = self.0.clone();
		platform::spawn(async move {
			let mut reports = reports;
			while let Some(report) = reports.next().await {
				// A report the server could not take ends the rift's stream
				// with the reason, so nothing more is said of it here.
				let _ = reporter
					.call(ReportRift {
						report: Shared(report),
					})
					.await;
			}
		});
		Ok(frames)
	}
}

/// The live socket, where this build has one.
#[cfg(all(target_arch = "wasm32", any(feature = "web", feature = "native")))]
mod live {
	use futures::StreamExt as _;
	use futures::stream::LocalBoxStream;
	use reapers_api::rpc::client;
	use reapers_api::rpc::client::StreamEvent;
	use reapers_api::rpc::live::EndReason;
	use reapers_rules::RiftFrame;

	use super::ApiError;
	use super::ApiResult;
	use super::SharedToken;

	/// How many frames may wait unread before the server holds the rest.
	const WINDOW: u32 = 64;

	/// Signs the live socket with the session's access token.
	struct Bearer(SharedToken);

	impl client::Credentials for Bearer {
		fn authorization(
			&self,
		) -> futures::future::BoxFuture<'_, Result<Option<String>, reapers_api::rpc::FnError>>
		{
			let header = self
				.0
				.lock()
				.unwrap_or_else(std::sync::PoisonError::into_inner)
				.clone();
			Box::pin(std::future::ready(Ok(header)))
		}
	}

	/// Points `leptoskit_rpc`'s client, and its live socket, at the server.
	pub fn configure(origin: &str, token: &SharedToken) {
		client::configure(
			client::ClientConfig::new()
				.origin(origin)
				.credentials(Bearer(token.clone())),
		);
	}

	/// The socket reads as whoever the token now says.
	pub fn reauthenticate() {
		client::reauthenticate();
	}

	/// The rift at `site`, frame by frame.
	pub fn enter_rift(
		site: &str,
		demo: bool,
	) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>> {
		if !client::live::available() {
			return Err(ApiError::Unavailable("no live connection".into()));
		}
		let stream = client::live::client().stream(
			<reapers_api::rift::EnterRift as reapers_api::rpc::StreamOperation>::NAME,
			serde_json::to_value(reapers_api::rift::EnterRift {
				site: site.to_owned(),
				demo,
			})
			.map_err(|error| ApiError::Unavailable(error.to_string()))?,
			WINDOW,
			false,
		);
		Ok(stream
			.take_while(|event| {
				std::future::ready(!matches!(event, StreamEvent::End(EndReason::Done)))
			})
			.map(|event| {
				match event {
					StreamEvent::Item(value) => {
						serde_json::from_value(value)
							.map_err(|error| ApiError::Unavailable(error.to_string()))
					}
					StreamEvent::End(EndReason::Failed(error)) => {
						Err(super::refusal_or(&error.message, || error.message.clone()))
					}
					StreamEvent::End(reason) => {
						Err(ApiError::Unavailable(format!(
							"the rift ended: {}",
							reason.as_str()
						)))
					}
				}
			})
			.boxed_local())
	}
}

/// No socket: the tests' desktop host.
#[cfg(not(all(target_arch = "wasm32", any(feature = "web", feature = "native"))))]
mod live {
	use futures::stream::LocalBoxStream;
	use reapers_rules::RiftFrame;

	use super::ApiError;
	use super::ApiResult;
	use super::SharedToken;

	pub fn configure(_origin: &str, _token: &SharedToken) {}

	pub fn reauthenticate() {}

	pub fn enter_rift(
		_site: &str,
		_demo: bool,
	) -> ApiResult<LocalBoxStream<'static, ApiResult<RiftFrame>>> {
		Err(ApiError::Unavailable(
			"this host has no live connection for rifts".into(),
		))
	}
}
