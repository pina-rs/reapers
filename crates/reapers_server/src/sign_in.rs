//! Signing in: as a guest, and with a wallet.
//!
//! A game should not ask for a sign-up before the first hunt, so a fresh
//! install plays at once as a guest account. Nothing else about a player is
//! tied to how they signed in, so a wallet is linked to the guest account
//! later without touching what it owns.
//!
//! Accounts and sessions are `leptoskit_auth`'s: its `auth.accounts` and
//! `auth.sessions` tables, its session layer recognizing the tokens, and its
//! `/auth/refresh` route trading a refresh token for a new pair.
//! `leptoskit_auth` has no guest sign-in, no way for an app to start a session
//! for an account it vouches for, and its tables are written only by its own
//! code. So [`new_account`] and [`start_session`] write those two rows with
//! SQL, through `leptoskit_db`, in the sign-in's transaction, as
//! `leptoskit_auth`'s own sign-in writes them: a 256-bit random access token
//! and refresh token, of which only their SHA-256 is kept, with
//! `leptoskit_auth`'s default lifetimes. The tests check `leptoskit_auth`
//! recognizes, refreshes and ends these sessions as its own. When
//! `leptoskit_auth` gains an anonymous provider, these two functions go.

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use leptoskit_auth::Account;
use leptoskit_auth::SessionLifetimes;
use leptoskit_db::Connection;
use leptoskit_db::Id;
use leptoskit_db::Statement;
use leptoskit_db::Tx;
use leptoskit_db::Value;
use leptoskit_rpc as rpc;
use reapers_api::auth::SignInRefused;
use reapers_api::auth::SignedIn;
use reapers_api::auth::WalletSignIn;
use reapers_api::refused;
use reapers_rules::Refusal;
use sha2::Digest;
use sha2::Sha256;

use crate::books;
use crate::keeper::Keeper;
use crate::tables::NewWalletIdentity;
use crate::tables::WalletChallenge;
use crate::tables::WalletIdentity;
use crate::wallet;

/// What a session started here is listed as among the account's devices.
const DEVICE: &str = "Reapers";

/// How many times a sign-in runs again when another transaction gets in its
/// way.
const RETRIES: u32 = 5;

/// `/auth/guest` and `/auth/wallet`, beside `leptoskit_auth`'s own routes.
pub fn router(keeper: Arc<Keeper>) -> Router {
	Router::new()
		.route(reapers_api::auth::GUEST, post(guest))
		.route(reapers_api::auth::WALLET, post(wallet_sign_in))
		.with_state(keeper)
}

/// A refusal, as a sign-in route answers it.
struct Refused(rpc::Error);

impl IntoResponse for Refused {
	fn into_response(self) -> Response {
		let status =
			StatusCode::from_u16(self.0.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

		(
			status,
			Json(SignInRefused {
				error: self.0.message().to_owned(),
			}),
		)
			.into_response()
	}
}

impl From<rpc::Error> for Refused {
	fn from(error: rpc::Error) -> Self {
		Self(error)
	}
}

async fn guest(State(keeper): State<Arc<Keeper>>) -> Result<Json<SignedIn>, Refused> {
	Ok(Json(sign_in_guest(&keeper).await?))
}

/// Opens a new guest account, with its books, and starts its session.
pub async fn sign_in_guest(keeper: &Keeper) -> rpc::Result<SignedIn> {
	transact(keeper, |tx| {
		Box::pin(async move {
			let account = new_account(tx).await?;

			books::open(tx, account).await?;
			start_session(tx, account).await
		})
	})
	.await
}

async fn wallet_sign_in(
	State(keeper): State<Arc<Keeper>>,
	me: Option<leptoskit_auth::SignedIn>,
	Json(proof): Json<WalletSignIn>,
) -> Result<Json<SignedIn>, Refused> {
	let Some(me) = me else {
		return Err(Refused(rpc::Error::unauthorized("sign in first")));
	};

	Ok(Json(sign_in_wallet(&keeper, me.account, &proof).await?))
}

/// Checks `proof` answers `account`'s open challenge, and starts a session.
///
/// The first proof links this account to the wallet. A later proof, from
/// any account, restores the account the wallet was first linked to:
/// accounts are switched rather than two players' saves silently merged. A
/// challenge is used once, only by the account it was made for, and only
/// before it expires.
pub async fn sign_in_wallet(
	keeper: &Keeper,
	account: Id<Account>,
	proof: &WalletSignIn,
) -> rpc::Result<SignedIn> {
	transact(keeper, |tx| {
		let proof = proof.clone();

		Box::pin(async move {
			books::open(tx, account).await?;

			let challenge = WalletChallenge::query()
				.filter(|challenge| {
					challenge.challenge_id.eq(proof.challenge_id.clone())
						& challenge.account_id.eq(account)
				})
				.first(tx)
				.await?;
			let Some(challenge) = challenge.filter(|challenge| {
				!challenge.consumed
					&& challenge.expires_at > chrono::Utc::now()
					&& wallet::verify(&challenge.address, &challenge.message, &proof.signature)
			}) else {
				return Err(refused(Refusal::WalletUnproven));
			};
			let linked = WalletIdentity::query()
				.filter(|identity| identity.address.eq(challenge.address.clone()))
				.first(tx)
				.await?;
			let owner = if let Some(linked) = linked {
				linked.account_id
			} else {
				if books::wallet_of(tx, account).await?.is_some() {
					return Err(refused(Refusal::AlreadyOwned));
				}

				NewWalletIdentity {
					address: challenge.address.clone(),
					account_id: account,
				}
				.insert(tx)
				.await?;
				account
			};

			WalletChallenge::update()
				.filter(|row| row.id.eq(challenge.id))
				.set(|row| row.consumed, true)
				.run(tx)
				.await?;
			start_session(tx, owner).await
		})
	})
	.await
}

/// Starts a native app's session for `account`: an access token and a
/// refresh token, as `leptoskit_auth`'s own sign-in starts one.
pub async fn start_session(tx: &impl Connection, account: Id<Account>) -> rpc::Result<SignedIn> {
	let lifetimes = SessionLifetimes::default();
	let span = |duration: std::time::Duration| {
		chrono::TimeDelta::from_std(duration)
			.map_err(|_| rpc::Error::server("a session lifetime is too long"))
	};
	let now = leptoskit_db::now();
	let expires_at = now + span(lifetimes.absolute)?;
	let access_expires_at = (now + span(lifetimes.access)?).min(expires_at);
	let access = token()?;
	let refresh = token()?;

	tx.execute(Statement {
		sql: "INSERT INTO \"auth\".\"sessions\" (\"id\", \"account_id\", \"token_hash\", \
		      \"access_expires_at\", \"refresh_hash\", \"aal\", \"authenticated_at\", \
		      \"last_seen_at\", \"idle_expires_at\", \"expires_at\", \"device\", \"created_at\") \
		      VALUES ($1, $2, $3, $4, $5, 'aal1', $6, $6, $7, $8, $9, $6)"
			.to_owned(),
		args: vec![
			Value::Uuid(Id::<Account>::new().uuid()),
			Value::Uuid(account.uuid()),
			Value::Bytes(hash(&access)),
			Value::Timestamp(access_expires_at),
			Value::Bytes(hash(&refresh)),
			Value::Timestamp(now),
			Value::Timestamp((now + span(lifetimes.idle)?).min(expires_at)),
			Value::Timestamp(expires_at),
			Value::Text(DEVICE.to_owned()),
		],
	})
	.await?;

	Ok(SignedIn {
		account: account.uuid().to_string(),
		access_token: access,
		refresh_token: refresh,
		expires_in: u64::try_from((access_expires_at - now).num_seconds()).unwrap_or_default(),
	})
}

/// A new account in `leptoskit_auth`'s `auth.accounts`: a guest, with no way
/// to sign in but the session started beside it.
pub async fn new_account(tx: &impl Connection) -> rpc::Result<Id<Account>> {
	let account = Id::<Account>::new();

	tx.execute(Statement {
		sql: "INSERT INTO \"auth\".\"accounts\" (\"id\", \"created_at\") VALUES ($1, $2)"
			.to_owned(),
		args: vec![
			Value::Uuid(account.uuid()),
			Value::Timestamp(leptoskit_db::now()),
		],
	})
	.await?;

	Ok(account)
}

/// 256 random bits, as `leptoskit_auth` writes a session token.
fn token() -> rpc::Result<String> {
	let mut bytes = [0; 32];

	getrandom::fill(&mut bytes).map_err(|_| rpc::Error::server("no secure random numbers"))?;

	Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// What `leptoskit_auth` keeps of a token.
fn hash(token: &str) -> Vec<u8> {
	Sha256::digest(token.as_bytes()).to_vec()
}

/// Runs `work` in a serializable transaction of the keeper's own, again
/// when another gets in its way.
async fn transact<T>(
	keeper: &Keeper,
	work: impl for<'t> Fn(&'t Tx) -> futures::future::BoxFuture<'t, rpc::Result<T>>,
) -> rpc::Result<T> {
	let mut attempt = 0;

	loop {
		let tx = keeper.db().begin_serializable().await?;
		let result = match work(&tx).await {
			Ok(value) => tx.commit().await.map(|()| value).map_err(rpc::Error::from),
			Err(error) => {
				let _ = tx.rollback().await;
				Err(error)
			}
		};

		match result {
			Err(error) if error.is_contended() && attempt < RETRIES => attempt += 1,
			result => return result,
		}
	}
}
