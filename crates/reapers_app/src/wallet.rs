//! The player's wallet: proving it to the server, and signing the
//! transactions that move a husk.
//!
//! The wallet keeps every key. To prove itself it is asked to authorize the
//! app, then to sign the server's short-lived challenge for its address; the
//! server checks the Ed25519 signature and links the wallet to the player.
//! On a cluster build it is also asked to sign each release and each burn
//! ([`crate::chain`]), which the game sends itself; the wallet is shown the
//! whole transaction and signs only its own slot.
//!
//! Android speaks Mobile Wallet Adapter, through `solana_kit_leptoskit`'s
//! guest session and the `solana-wallet` runtime service the Android host
//! registers ([`MobileWalletProver`]). The browser reaches the wallets the
//! page has through the Wallet Standard ([`BrowserWallet`]). iOS has no MWA
//! wallets, so there the roster shows the wallet as unavailable
//! ([`NoWallet`]), as the Flutter game did.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::future::LocalBoxFuture;

use crate::backend::ChainStatus;
use crate::backend::KeeperError;
use crate::backend::KeeperResult;
use crate::backend::WalletChallenge;
use crate::chain::ChainFailure;
use crate::chain::ChainResult;

/// A wallet's answer to the server's challenge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletProof {
	pub challenge_id: String,
	/// The Ed25519 signature over the challenge message, in base64.
	pub signature: String,
}

/// Asks the server for a challenge for one base58 address.
pub type ChallengeFn<'a> =
	&'a dyn Fn(String) -> LocalBoxFuture<'static, KeeperResult<WalletChallenge>>;

/// Something that can prove control of a wallet.
#[async_trait::async_trait(?Send)]
pub trait WalletProver {
	/// Whether this device can reach a wallet at all.
	fn available(&self) -> bool;

	/// Authorizes the app with a wallet, asks `challenge` for the message to
	/// sign for its first account, and returns the signature.
	async fn prove(
		&self,
		status: &ChainStatus,
		challenge: ChallengeFn<'_>,
	) -> KeeperResult<WalletProof>;

	/// Has the wallet sign its slot of `transaction`, an unsigned wire
	/// transaction it pays for, and returns the transaction signed. The
	/// caller checks the wallet signed exactly what it was handed.
	async fn sign_transaction(
		&self,
		status: &ChainStatus,
		transaction: Vec<u8>,
	) -> ChainResult<Vec<u8>> {
		let _ = (status, transaction);
		Err(ChainFailure::NoWallet)
	}
}

/// The wallet's answer was not the signed message the game asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SignedMessageError {
	#[error("the wallet's payload is not base64")]
	NotBase64,
	#[error("the wallet returned a different message")]
	DifferentMessage,
}

/// The signature in a Mobile Wallet Adapter signed payload, which is the
/// message followed by its 64-byte signature. The exact message the game
/// asked for must come back, or nothing is sent to the server.
pub fn signed_message_signature(
	message: &str,
	payload: &str,
) -> Result<String, SignedMessageError> {
	let bytes = BASE64
		.decode(payload)
		.map_err(|_| SignedMessageError::NotBase64)?;
	let expected = message.as_bytes();
	if bytes.len() != expected.len() + 64 || &bytes[..expected.len()] != expected {
		return Err(SignedMessageError::DifferentMessage);
	}
	Ok(BASE64.encode(&bytes[expected.len()..]))
}

/// No wallet on this device.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoWallet;

#[async_trait::async_trait(?Send)]
impl WalletProver for NoWallet {
	fn available(&self) -> bool {
		false
	}

	async fn prove(
		&self,
		_status: &ChainStatus,
		_challenge: ChallengeFn<'_>,
	) -> KeeperResult<WalletProof> {
		Err(KeeperError::unavailable("no wallet on this device"))
	}
}

/// This device's wallet: Mobile Wallet Adapter on Android, the page's
/// Wallet Standard wallets in a browser, none elsewhere.
pub fn device_wallet() -> Box<dyn WalletProver> {
	#[cfg(all(target_arch = "wasm32", feature = "android"))]
	{
		Box::new(mobile::MobileWalletProver)
	}
	#[cfg(all(target_arch = "wasm32", feature = "web"))]
	{
		Box::new(browser::BrowserWallet)
	}
	#[cfg(not(all(target_arch = "wasm32", any(feature = "android", feature = "web"))))]
	{
		Box::new(NoWallet)
	}
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod browser;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub use browser::BrowserWallet;
#[cfg(all(target_arch = "wasm32", feature = "android"))]
pub use mobile::MobileWalletProver;

#[cfg(all(target_arch = "wasm32", feature = "android"))]
mod mobile {
	use base64::Engine as _;
	use leptoskit_bridge::guest::Events;
	use serde_json::Map;
	use serde_json::Value;
	use serde_json::json;
	use solana_kit_core::Address;
	use solana_kit_leptoskit::GuestMobileWallet;
	use solana_kit_wallet::mwa::Method;

	use super::BASE64;
	use super::ChainFailure;
	use super::ChainResult;
	use super::ChainStatus;
	use super::ChallengeFn;
	use super::KeeperError;
	use super::KeeperResult;
	use super::WalletProof;
	use super::WalletProver;
	use super::signed_message_signature;

	/// Mobile Wallet Adapter, through the Android host's `solana-wallet`
	/// service. It owns the guest's event stream only while a proof runs.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct MobileWalletProver;

	fn object(value: Value) -> Map<String, Value> {
		match value {
			Value::Object(map) => map,
			_ => Map::new(),
		}
	}

	/// An authorized session with the wallet, and the account it chose, as
	/// MWA's base64 and as an address.
	async fn authorize(
		events: &mut Events,
		status: &ChainStatus,
	) -> Result<(GuestMobileWallet, String, Address), String> {
		// Nothing else in the game listens for plugin events, so whatever
		// else arrives while the wallet is open is let go.
		let mut wallet = GuestMobileWallet::connect_android(events, |_| Ok(()))
			.await
			.map_err(|error| error.to_string())?;
		let authorized = wallet
			.call(
				Method::Authorize,
				object(json!({
					"identity": { "name": "Reapers", "uri": status.identity_uri },
					"chain": status.chain,
				})),
			)
			.await;
		let account = authorized
			.map_err(|error| error.to_string())
			.and_then(|authorized| {
				authorized
					.get("accounts")
					.and_then(|accounts| accounts.get(0))
					.and_then(|account| account.get("address"))
					.and_then(Value::as_str)
					.map(str::to_owned)
					.ok_or_else(|| "no authorized wallet account".to_owned())
			});
		let account = match account {
			Ok(account) => account,
			Err(error) => {
				let _ = wallet.close().await;
				return Err(error);
			}
		};
		let key: Option<[u8; 32]> = BASE64
			.decode(&account)
			.ok()
			.and_then(|bytes| bytes.try_into().ok());
		let Some(key) = key else {
			let _ = wallet.close().await;
			return Err("invalid wallet address".to_owned());
		};
		Ok((wallet, account, Address::from_bytes(key)))
	}

	#[async_trait::async_trait(?Send)]
	impl WalletProver for MobileWalletProver {
		fn available(&self) -> bool {
			true
		}

		async fn prove(
			&self,
			status: &ChainStatus,
			challenge: ChallengeFn<'_>,
		) -> KeeperResult<WalletProof> {
			let mut events = Events::take().map_err(KeeperError::unavailable)?;
			let (mut wallet, account, address) = authorize(&mut events, status)
				.await
				.map_err(KeeperError::unavailable)?;

			let proof = async {
				let issued = challenge(address.to_string()).await?;

				let signed = wallet
					.call(
						Method::SignMessages,
						object(json!({
							"addresses": [account],
							"payloads": [BASE64.encode(issued.message.as_bytes())],
						})),
					)
					.await
					.map_err(KeeperError::unavailable)?;
				let payloads = signed
					.get("signed_payloads")
					.and_then(Value::as_array)
					.filter(|payloads| payloads.len() == 1)
					.ok_or_else(|| KeeperError::unavailable("expected one signed message"))?;
				let payload = payloads[0]
					.as_str()
					.ok_or_else(|| KeeperError::unavailable("expected one signed message"))?;

				Ok(WalletProof {
					challenge_id: issued.challenge_id,
					signature: signed_message_signature(&issued.message, payload)
						.map_err(KeeperError::unavailable)?,
				})
			}
			.await;

			let _ = wallet.close().await;
			proof
		}

		async fn sign_transaction(
			&self,
			status: &ChainStatus,
			transaction: Vec<u8>,
		) -> ChainResult<Vec<u8>> {
			let mut events =
				Events::take().map_err(|error| ChainFailure::Unreachable(error.to_string()))?;
			let (mut wallet, _account, _address) = authorize(&mut events, status)
				.await
				.map_err(|_| ChainFailure::Declined)?;
			let signed = wallet
				.call(
					Method::SignTransactions,
					object(json!({ "payloads": [BASE64.encode(&transaction)] })),
				)
				.await
				.map_err(|_| ChainFailure::Declined);
			let _ = wallet.close().await;
			signed?
				.get("signed_payloads")
				.and_then(Value::as_array)
				.filter(|payloads| payloads.len() == 1)
				.and_then(|payloads| payloads[0].as_str())
				.and_then(|payload| BASE64.decode(payload).ok())
				.ok_or(ChainFailure::WrongWallet)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn payload(message: &[u8], signature: &[u8; 64]) -> String {
		BASE64.encode([message, signature].concat())
	}

	#[test]
	fn the_signature_follows_the_exact_message() {
		let signature = [7; 64];
		assert_eq!(
			signed_message_signature("sign in", &payload(b"sign in", &signature)),
			Ok(BASE64.encode(signature))
		);
	}

	#[test]
	fn a_different_message_is_refused() {
		assert_eq!(
			signed_message_signature("sign in", &payload(b"sign IN", &[7; 64])),
			Err(SignedMessageError::DifferentMessage)
		);
		assert_eq!(
			signed_message_signature("sign in", &BASE64.encode(b"sign in")),
			Err(SignedMessageError::DifferentMessage)
		);
		assert_eq!(
			signed_message_signature("sign in", "not base64!"),
			Err(SignedMessageError::NotBase64)
		);
	}

	#[test]
	fn no_wallet_is_unavailable() {
		assert!(!NoWallet.available());
	}
}
