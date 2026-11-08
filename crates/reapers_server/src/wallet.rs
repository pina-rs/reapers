//! Proving a wallet: the message it signs, and the check of its signature.
//! No transaction is signed and no key is held.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::DateTime;
use chrono::SecondsFormat;
use chrono::Utc;
use solana_kit_core::Address;
use solana_kit_core::Signature;

use crate::config::SolanaConfig;

/// How long a challenge may be answered.
pub const LIFETIME: chrono::TimeDelta = chrono::TimeDelta::minutes(2);

/// The message the wallet at `address` signs: a Sign-In With Solana message
/// bound to the keeper's identity origin, the chain and `nonce`, which says
/// in words that it authorizes no payment.
pub fn message(
	config: &SolanaConfig,
	address: &str,
	nonce: &str,
	issued: DateTime<Utc>,
	expires: DateTime<Utc>,
) -> String {
	let domain = config.identity_uri.host_str().unwrap_or_default();
	let time = |moment: DateTime<Utc>| moment.to_rfc3339_opts(SecondsFormat::Millis, true);

	format!(
		"{domain} wants you to sign in with your Solana account:\n{address}\n\nSign in to \
		 Reapers. This does not authorize a payment.\n\nURI: {uri}\nVersion: 1\nChain ID: \
		 {chain}\nNonce: {nonce}\nIssued At: {issued}\nExpiration Time: {expires}",
		uri = config.identity_uri,
		chain = config.chain,
		issued = time(issued),
		expires = time(expires),
	)
}

/// Whether `signature`, in standard base64, is the wallet at `wallet`'s
/// Ed25519 signature over exactly `message`.
///
/// Strict: a weak key, a malleable signature, a signature of the wrong
/// length or anything that is not base64 is refused.
pub fn verify(wallet: &str, message: &str, signature: &str) -> bool {
	if signature.len() > 88 {
		return false;
	}

	let Ok(address) = wallet.parse::<Address>() else {
		return false;
	};
	let Ok(bytes) = STANDARD.decode(signature) else {
		return false;
	};
	let Ok(bytes) = <[u8; 64]>::try_from(bytes) else {
		return false;
	};

	solana_kit_core::crypto::verify(&address, message.as_bytes(), &Signature::from_bytes(bytes))
		.is_ok()
}

/// Whether `text` is a Solana address.
pub fn is_address(text: &str) -> bool {
	text.parse::<Address>().is_ok()
}

#[cfg(test)]
mod tests {
	use solana_kit_core::crypto::Keypair;

	use super::*;

	fn sign(keypair: &Keypair, message: &str) -> String {
		STANDARD.encode(keypair.sign(message.as_bytes()).as_bytes())
	}

	#[test]
	fn a_proof_binds_the_wallet_and_the_exact_message() {
		let signer = Keypair::from_seed(&[1; 32]);
		let other = Keypair::from_seed(&[2; 32]);
		let address = signer.address().to_string();
		let signature = sign(&signer, "Reapers nonce 123");

		assert!(verify(&address, "Reapers nonce 123", &signature));
		assert!(!verify(
			&other.address().to_string(),
			"Reapers nonce 123",
			&signature
		));
		assert!(!verify(&address, "Reapers nonce 124", &signature));
		assert!(!verify(&address, "Reapers nonce 123", "bad!"));
		assert!(!verify("not an address", "Reapers nonce 123", &signature));
	}

	#[test]
	fn a_weak_key_proves_nothing() {
		// The all-zero address is a small-order point: any message "verifies"
		// against it with a lax check, which strict verification refuses.
		let weak = "11111111111111111111111111111111";

		assert!(!verify(weak, "anything", &STANDARD.encode([0; 64])));
	}

	#[test]
	fn the_message_names_the_origin_the_chain_and_that_it_is_no_payment() {
		let config = SolanaConfig::new(
			"solana:devnet",
			"https://api.devnet.solana.com",
			"https://reapers.example/",
			None,
			None,
			None,
		)
		.expect("valid settings");
		let issued = DateTime::parse_from_rfc3339("2026-11-08T12:00:00Z")
			.expect("a time")
			.with_timezone(&Utc);
		let text = message(&config, "Wallet111", "nonce-1", issued, issued + LIFETIME);

		assert!(text.starts_with("reapers.example wants you to sign in"));
		assert!(text.contains("Chain ID: solana:devnet"));
		assert!(text.contains("does not authorize a payment"));
		assert!(text.contains("Nonce: nonce-1"));
		assert!(text.contains("Expiration Time: 2026-11-08T12:02:00.000Z"));
	}
}
