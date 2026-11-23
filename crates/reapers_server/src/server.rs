//! The keeper, as one router: everything the binary serves, built in
//! process so the tests can serve the same thing.
//!
//! | Route              | What it is                                                         |
//! | ------------------ | ------------------------------------------------------------------ |
//! | `POST /api/<name>` | Every operation in `reapers_api`, as the signed-in player          |
//! | `GET /live`        | The live socket: the rift streams                                  |
//! | `/auth/guest`      | A new guest account and its session                                |
//! | `/auth/wallet`     | Wallet sign-in: link a guest, or restore the wallet's account      |
//! | `/auth/session`    | Current session inspection (`GET`)                                 |
//! | `/auth/refresh`    | Native/web bearer token refresh (`POST`)                           |
//! | `/auth/sign-out`   | End the current session (`POST`)                                   |
//!
//! With an origin, the routes also answer that origin's pages across
//! origins (CORS), with the bearer token in a header and no cookie: the web
//! game is served from its own host, and signs in as a native app does.

use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::Instant;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use futures::future::BoxFuture;
use leptoskit_auth::Account;
use leptoskit_auth::Auth;
use leptoskit_auth::AuthError;
use leptoskit_auth::CurrentSession;
use leptoskit_auth::Keys;
use leptoskit_auth::Mail;
use leptoskit_auth::MailError;
use leptoskit_auth::MailFuture;
use leptoskit_auth::Mailer;
use leptoskit_auth::PublicSession;
use leptoskit_auth::RefreshTokens;
use leptoskit_auth::RequestInfo;
use leptoskit_auth::Session;
use leptoskit_auth::SessionResponse;
use leptoskit_auth::SessionToken;
use leptoskit_auth::Tokens;
use leptoskit_db::Db;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use leptoskit_rpc::Identify;
use leptoskit_rpc::Identity;
use leptoskit_rpc::server::IdentityLease;
use leptoskit_rpc::server::Live;
use reapers_api::keeper::KeeperHandle;
use tower_http::CompressionLevel;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::DecompressionLayer;

use crate::chain::Gateway;
use crate::config::RunMode;
use crate::keeper::Keeper;
use crate::keeper::Shared;
use crate::rift::RiftHall;
use crate::sign_in;
use crate::signer::ReceiptSigner;

/// How to start a keeper.
#[derive(Debug)]
pub struct Options {
	pub mode: RunMode,
	/// The chain, as the keeper reads it.
	pub gateway: Gateway,
	/// The key receipts are signed with.
	pub signer: ReceiptSigner,
	/// Where rifts are hosted.
	pub hall: RiftHall,
	/// The site's origin, such as `https://reapers.example`, which the live
	/// socket checks pages against. Native apps send none.
	pub origin: Option<String>,
}

/// A running keeper: its router, and the parts the binary starts and stops.
pub struct Reapers {
	/// Everything the keeper serves.
	pub router: Router,
	/// The live socket's server, to start and to drain on shutdown.
	pub live: Live,
	/// Sign-in.
	pub auth: Auth,
	/// The keeper itself.
	pub keeper: Arc<Keeper>,
}

impl std::fmt::Debug for Reapers {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Reapers")
			.field("keeper", &self.keeper)
			.finish_non_exhaustive()
	}
}

/// Builds the keeper on a migrated `db`, signing in with `keys`.
///
/// # Errors
///
/// When sign-in cannot be built from `keys`.
pub fn start(db: &Db, keys: Keys, options: Options) -> Result<Reapers, AuthError> {
	let auth = Auth::builder(db, keys).app_name("Reapers").mailer(NoMail);
	let auth = match &options.origin {
		Some(origin) => auth.origin(origin),
		None => auth,
	}
	.build()?;
	let keeper = Arc::new(Keeper::new(
		options.mode,
		db,
		options.gateway,
		options.signer,
		options.hall,
	));
	let identify = KeeperIdentity {
		auth: auth.clone(),
		keeper: KeeperHandle(Arc::new(Shared(Arc::clone(&keeper)))),
	};
	let rpc = rpc::server::Server::new(db.clone()).identify(identify);
	let live = match &options.origin {
		Some(origin) => rpc.live().origin(origin),
		None => rpc.live(),
	};
	let limits = Arc::new(RateLimits::for_mode(options.mode));
	// Compress HTTP API/auth/session answers (Brotli level 4 keeps latency
	// sub-millisecond while cutting ~10 KiB JSON answers to ~1.5 KiB) and
	// accept gzip-compressed request bodies, leaving the WebSocket route alone.
	let http_routes = Router::new()
		.merge(rpc.router())
		.merge(sign_in::router(Arc::clone(&keeper)))
		.merge(session_router(
			auth.clone(),
			db.system("auth: session inspection"),
		))
		.layer(DecompressionLayer::new())
		.layer(CompressionLayer::new().quality(CompressionLevel::Precise(4)));
	let router = http_routes
		.merge(live.router())
		// Rate limiting runs after auth resolution so authenticated accounts
		// get per-account buckets and unauthenticated sign-in endpoints are
		// bounded per client IP.
		.layer(middleware::from_fn_with_state(limits, rate_limit_layer))
		// It finds each request's session.
		.layer(auth.layer())
		.layer(middleware::from_fn(security_headers_layer));
	// Outermost, so a preflight is answered before anything asks who it is.
	let router = match options.origin.as_deref().map(cors).transpose()? {
		Some(cors) => router.layer(cors),
		None => router,
	};

	Ok(Reapers {
		router,
		live,
		auth,
		keeper,
	})
}

/// Only the `leptoskit_auth` session-lifecycle endpoints Reapers actually uses:
/// `GET /auth/session`, `POST /auth/refresh`, and `POST /auth/sign-out`.
///
/// Unused sign-in flows (email OTP, passwords, passkeys, TOTP, OAuth, account
/// merging, and `leptoskit_auth`'s own wallet routes) are deliberately not
/// mounted and answer `404 Not Found`.
fn session_router(auth: Auth, db: Db) -> Router {
	Router::new()
		.route("/auth/session", get(auth_session))
		.route(reapers_api::auth::REFRESH, post(auth_refresh))
		.route("/auth/sign-out", post(auth_sign_out))
		.with_state(SessionState { auth, db })
}

#[derive(Clone)]
struct SessionState {
	auth: Auth,
	db: Db,
}

async fn auth_session(
	State(state): State<SessionState>,
	Extension(current): Extension<CurrentSession>,
) -> Result<Json<SessionResponse>, AuthError> {
	let session = match current.signed_in.or(current.enrolling) {
		Some(me) => {
			Session::find(&state.db, me.session)
				.await?
				.map(PublicSession::from)
		}
		None => None,
	};

	Ok(Json(SessionResponse { session }))
}

async fn auth_refresh(
	State(state): State<SessionState>,
	request: RequestInfo,
	Json(body): Json<RefreshTokens>,
) -> Result<Json<Tokens>, AuthError> {
	let sign_in = state
		.auth
		.refresh(&SessionToken::new(body.refresh_token), &request)
		.await?;

	sign_in.tokens().map(Json).ok_or(AuthError::SignedOut)
}

async fn auth_sign_out(
	State(state): State<SessionState>,
	headers: HeaderMap,
	request: RequestInfo,
) -> Result<Response, AuthError> {
	if let Some(token) = bearer_token(&headers) {
		state.auth.sign_out(&token, &request).await?;
	}

	let mut response = StatusCode::NO_CONTENT.into_response();
	response
		.headers_mut()
		.insert(header::SET_COOKIE, state.auth.cleared_cookie());
	Ok(response)
}

fn bearer_token(headers: &HeaderMap) -> Option<SessionToken> {
	let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
	let (scheme, token) = value.split_once(' ')?;

	scheme
		.eq_ignore_ascii_case("bearer")
		.then(|| SessionToken::new(token.trim()))
}

/// Hardens every HTTP answer with API security headers. Public static keys
/// (`/api/receipt_key`) may be cached briefly; everything else is `no-store`.
async fn security_headers_layer(request: Request<Body>, next: Next) -> Response {
	let cache_control = if request.uri().path() == "/api/receipt_key" {
		HeaderValue::from_static("public, max-age=300")
	} else {
		HeaderValue::from_static("no-store, max-age=0")
	};
	let mut response = next.run(request).await;
	let headers = response.headers_mut();

	headers.insert(header::CACHE_CONTROL, cache_control);
	headers.insert(
		header::X_CONTENT_TYPE_OPTIONS,
		HeaderValue::from_static("nosniff"),
	);
	headers.insert(
		header::REFERRER_POLICY,
		HeaderValue::from_static("no-referrer"),
	);
	headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

	response
}

/// Zero-allocation actor key for per-account / per-IP rate limiting.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ActorKey {
	Account(Id<Account>),
	Ip(IpAddr),
}

type RateBuckets = HashMap<(ActorKey, &'static str), (Instant, u32)>;

/// Fixed-window per-client/per-bucket rate limiter with bounded cardinality so
/// unauthenticated endpoints (`/auth/guest`, `/auth/wallet`, `/auth/refresh`)
/// and expensive RPCs cannot be flooded into DB/RPC exhaustion.
struct RateLimits {
	window: Duration,
	auth_guest_limit: u32,
	auth_wallet_limit: u32,
	api_limit: u32,
	buckets: Mutex<RateBuckets>,
}

impl RateLimits {
	const MAX_BUCKETS: usize = 8192;

	fn for_mode(mode: RunMode) -> Self {
		// Tests run many `oneshot` calls from the same synthetic peer; give
		// `Test` generous headroom while still exercising the limiter.
		let (auth_guest_limit, auth_wallet_limit, api_limit) = if mode == RunMode::Test {
			(1_000, 1_000, 5_000)
		} else {
			(30, 60, 600)
		};

		Self {
			window: Duration::from_secs(60),
			auth_guest_limit,
			auth_wallet_limit,
			api_limit,
			buckets: Mutex::new(HashMap::new()),
		}
	}

	fn allow(&self, actor: ActorKey, bucket: &'static str, limit: u32) -> Result<(), u64> {
		let now = Instant::now();
		let mut map = self.buckets.lock().unwrap_or_else(PoisonError::into_inner);

		if map.len() >= Self::MAX_BUCKETS {
			map.retain(|_, (started, _)| now.duration_since(*started) < self.window);
			if map.len() >= Self::MAX_BUCKETS {
				map.clear();
			}
		}

		let entry = map.entry((actor, bucket)).or_insert((now, 0));
		let elapsed = now.duration_since(entry.0);

		if elapsed >= self.window {
			*entry = (now, 1);
			return Ok(());
		}

		if entry.1 >= limit {
			let remaining = self.window.saturating_sub(elapsed).as_secs().max(1);
			return Err(remaining);
		}

		entry.1 += 1;
		Ok(())
	}
}

async fn rate_limit_layer(
	State(limits): State<Arc<RateLimits>>,
	request: Request<Body>,
	next: Next,
) -> Response {
	if *request.method() == Method::OPTIONS {
		return next.run(request).await;
	}

	let path = request.uri().path();
	let (bucket, limit) = match path {
		reapers_api::auth::GUEST => ("auth_guest", limits.auth_guest_limit),
		reapers_api::auth::WALLET | reapers_api::auth::REFRESH => {
			("auth_wallet", limits.auth_wallet_limit)
		}
		p if p.starts_with("/api/") => ("api", limits.api_limit),
		_ => return next.run(request).await,
	};

	let actor = request
		.extensions()
		.get::<CurrentSession>()
		.and_then(|session| {
			session
				.signed_in
				.as_ref()
				.map(|me| ActorKey::Account(me.account))
		})
		.or_else(|| {
			request
				.extensions()
				.get::<ConnectInfo<SocketAddr>>()
				.map(|ConnectInfo(addr)| ActorKey::Ip(addr.ip()))
		})
		.unwrap_or_else(|| ActorKey::Ip(IpAddr::from([127, 0, 0, 1])));

	if let Err(retry_after) = limits.allow(actor, bucket, limit) {
		let mut response = (
			StatusCode::TOO_MANY_REQUESTS,
			Json(serde_json::json!({
				"error": "rate_limited",
				"retry_after": retry_after,
			})),
		)
			.into_response();

		if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
			response.headers_mut().insert(header::RETRY_AFTER, value);
		}

		return response;
	}

	next.run(request).await
}

/// Lets the pages of `origin` call the keeper: its operations and sign-in
/// routes, with the headers the game's client sends. No credentials: the
/// web game sends its token as a bearer header, never a cookie.
fn cors(origin: &str) -> Result<CorsLayer, AuthError> {
	let origin = HeaderValue::from_str(origin.trim_end_matches('/'))
		.map_err(|_| AuthError::Config(format!("REAPERS_ORIGIN: not an origin: {origin}")))?;

	Ok(CorsLayer::new()
		.allow_origin(origin)
		.allow_methods([Method::GET, Method::POST])
		.allow_headers([
			header::ACCEPT,
			header::AUTHORIZATION,
			header::CONTENT_TYPE,
			HeaderName::from_static("idempotency-key"),
		])
		.max_age(std::time::Duration::from_secs(600)))
}

/// The keeper sends no mail: players sign in as guests and with wallets,
/// never with an emailed code, so `leptoskit_auth`'s email sign-in always
/// fails to send.
struct NoMail;

impl Mailer for NoMail {
	fn send(&self, _mail: Mail) -> MailFuture<'_> {
		Box::pin(std::future::ready(Err(MailError(
			"Reapers signs in with guests and wallets, not email".to_owned(),
		))))
	}
}

/// Who a request is from, as `leptoskit_auth` says, with the keeper beside
/// them in the context, where the operations' bodies find it.
#[derive(Clone)]
pub struct KeeperIdentity {
	pub auth: Auth,
	pub keeper: KeeperHandle,
}

impl KeeperIdentity {
	fn with_keeper(&self, mut identity: Identity) -> Identity {
		identity.context = identity.context.with(self.keeper.clone());
		identity
	}
}

impl Identify for KeeperIdentity {
	fn identify<'a>(
		&'a self,
		request: &'a http::request::Parts,
	) -> BoxFuture<'a, rpc::Result<Identity>> {
		Box::pin(async move { Ok(self.with_keeper(self.auth.identify(request).await?)) })
	}

	fn lease<'a>(&'a self, identity: &'a Identity) -> BoxFuture<'a, rpc::Result<IdentityLease>> {
		self.auth.lease(identity)
	}

	fn identify_token<'a>(&'a self, token: &'a str) -> BoxFuture<'a, rpc::Result<Identity>> {
		Box::pin(async move { Ok(self.with_keeper(self.auth.identify_token(token).await?)) })
	}
}
