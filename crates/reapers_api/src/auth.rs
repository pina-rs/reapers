//! Signing in: as a guest, and with a wallet.
//!
//! These are plain routes beside `leptoskit_auth`'s own `/auth` routes rather
//! than operations, because each one starts a session. Both answer a native
//! app's token pair; `leptoskit_auth`'s [`REFRESH`] route trades the refresh
//! token for a new pair, and the access token goes in an
//! `Authorization: Bearer` header on every operation.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// `POST`, no body: a fresh install plays at once as a new guest account.
pub const GUEST: &str = "/auth/guest";

/// `POST` a [`WalletSignIn`], signed in as the guest the challenge was made
/// for.
///
/// The first proof links this guest to the wallet. A later proof, from any
/// guest, restores the account the wallet was first linked to: the answer's
/// account is then that one, and the guest's own books are left as they
/// were rather than merged into it.
pub const WALLET: &str = "/auth/wallet";

/// `POST` `{"refresh_token": "…"}`: `leptoskit_auth`'s refresh for native apps.
pub const REFRESH: &str = "/auth/refresh";

/// A signed wallet challenge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WalletSignIn {
	/// The challenge's `challenge_id`.
	pub challenge_id: String,

	/// The wallet's Ed25519 signature over the challenge's message, in
	/// standard base64.
	pub signature: String,
}

/// A session that just started.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignedIn {
	/// The account the session signs in.
	pub account: String,

	/// Sent as `Authorization: Bearer <access_token>`.
	pub access_token: String,

	/// Traded once at [`REFRESH`] for a new pair.
	pub refresh_token: String,

	/// Seconds until the access token stops working.
	pub expires_in: u64,
}

/// Why a sign-in route said no: its JSON body, beside a `4xx` status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignInRefused {
	/// The same message an operation would fail with: for a refusal of the
	/// rules, [`crate::REFUSED`] and the refusal's name.
	pub error: String,
}
