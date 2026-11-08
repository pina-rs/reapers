//! The keeper's signing key: what absorption receipts are signed with, and
//! the one key the Reapers program trusts.

use std::fmt;
use std::fmt::Write as _;

use reapers_api::player::KeeperKey;
use sha2::Digest;
use sha2::Sha256;
use solana_kit_core::Address;
use solana_kit_core::Signature;
use solana_kit_core::crypto::Keypair;

/// The keeper's Ed25519 key.
///
/// It is derived from one secret, `receiptSigningKey`: the SHA-256 of the
/// secret is the key's seed, as the first keeper derived it, so a deployment
/// only has to keep a string and the key it had before is the key it has
/// now. Whoever holds that secret can sign an absorption of anything, so it
/// is one of the secrets a deployed keeper refuses to start without.
pub struct ReceiptSigner {
	keypair: Keypair,
	address: Address,
	development: bool,
}

impl ReceiptSigner {
	/// What the key is derived from in development and in tests, where no
	/// secret is configured. It is public, so receipts signed with it prove
	/// nothing: that is the point of refusing to deploy without a real one.
	pub const DEVELOPMENT_SECRET: &'static str = "reapers-development-receipt-key";
	/// The name of the secret the key is derived from.
	pub const SECRET: &'static str = "receiptSigningKey";

	/// The signer whose key is derived from `secret`.
	pub fn from_secret(secret: &str) -> Self {
		let seed: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
		let keypair = Keypair::from_seed(&seed);

		Self {
			address: keypair.address(),
			keypair,
			development: secret == Self::DEVELOPMENT_SECRET,
		}
	}

	/// The signer anyone can derive, for development and tests.
	pub fn development() -> Self {
		Self::from_secret(Self::DEVELOPMENT_SECRET)
	}

	/// The 32-byte public key receipts are checked against, as an address.
	pub const fn address(&self) -> Address {
		self.address
	}

	/// Whether this is the development key anyone can derive.
	pub const fn is_development(&self) -> bool {
		self.development
	}

	/// Signs `message`.
	pub fn sign(&self, message: &[u8]) -> Signature {
		self.keypair.sign(message)
	}

	/// Whether `signature` is this signer's over `message`.
	pub fn verifies(&self, message: &[u8], signature: &Signature) -> bool {
		solana_kit_core::crypto::verify(&self.address, message, signature).is_ok()
	}

	/// The key, as the game asks for it.
	pub fn key(&self) -> KeeperKey {
		KeeperKey {
			hex: hex(self.address.as_bytes()),
			address: self.address.to_string(),
			development: self.development,
		}
	}
}

impl fmt::Debug for ReceiptSigner {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ReceiptSigner")
			.field("address", &self.address.to_string())
			.field("development", &self.development)
			.finish_non_exhaustive()
	}
}

/// `bytes` as lowercase hexadecimal digits.
pub fn hex(bytes: &[u8]) -> String {
	bytes
		.iter()
		.fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
			let _ = write!(text, "{byte:02x}");
			text
		})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_different_secret_signs_with_a_different_key() {
		let ours = ReceiptSigner::development();
		let theirs = ReceiptSigner::from_secret("some other secret");
		let message = [7; 98];

		assert_ne!(theirs.address(), ours.address());
		assert!(!ours.verifies(&message, &theirs.sign(&message)));
		assert!(ours.verifies(&message, &ours.sign(&message)));
		assert!(ours.is_development());
		assert!(!theirs.is_development());
	}

	#[test]
	fn the_key_is_given_as_hex_and_as_an_address() {
		let signer = ReceiptSigner::development();
		let key = signer.key();

		assert_eq!(key.hex.len(), 64);
		assert_eq!(key.address, signer.address().to_string());
		assert!(key.development);
	}
}
