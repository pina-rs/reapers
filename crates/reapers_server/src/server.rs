//! The keeper, as one router: everything the binary serves, built in
//! process so the tests can serve the same thing.
//!
//! | Route              | What it is                                                         |
//! | ------------------ | ------------------------------------------------------------------ |
//! | `POST /api/<name>` | Every operation in `reapers_api`, as the signed-in player          |
//! | `GET /live`        | The live socket: the rift streams                                  |
//! | `/auth/guest`      | A new guest account and its session                                |
//! | `/auth/wallet`     | Wallet sign-in: link a guest, or restore the wallet's account      |
//! | `/auth/…`          | leptoskit_auth's own routes, such as `/auth/refresh`               |

use std::sync::Arc;

use axum::Router;
use futures::future::BoxFuture;
use leptoskit_auth::Auth;
use leptoskit_auth::AuthError;
use leptoskit_auth::Keys;
use leptoskit_auth::Mail;
use leptoskit_auth::MailError;
use leptoskit_auth::MailFuture;
use leptoskit_auth::Mailer;
use leptoskit_db::Db;
use leptoskit_rpc as rpc;
use leptoskit_rpc::Identify;
use leptoskit_rpc::Identity;
use leptoskit_rpc::server::IdentityLease;
use leptoskit_rpc::server::Live;
use reapers_api::keeper::KeeperHandle;

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
	let router = Router::new()
		.merge(rpc.router())
		.merge(sign_in::router(Arc::clone(&keeper)))
		.merge(auth.router())
		.merge(live.router())
		// Last, so it wraps every route: it finds each request's session.
		.layer(auth.layer());

	Ok(Reapers {
		router,
		live,
		auth,
		keeper,
	})
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
