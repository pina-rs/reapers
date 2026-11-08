//! A husk escrowed with the Reapers program: where its escrow account is,
//! and what that account records.
//!
//! TODO(integration): replace with the program's generated client. The
//! seeds and the account layout here are the keeper's reading of the
//! program, not the program's own declaration; the generated client's
//! `Escrow` decoder and PDA helper take their place when the program lands.
//!
//! The escrow account is the program-derived address of
//! `["escrow", mint]` under the program. Its data is a one-byte
//! discriminator ([`DISCRIMINATOR`]), the wallet that released the husk,
//! the husk's mint, and the bump.

use solana_kit_core::Address;
use solana_kit_core::crypto::find_program_address;

/// The seed every escrow address starts with.
pub const SEED: &[u8] = b"escrow";

/// The first byte of an escrow account's data.
pub const DISCRIMINATOR: u8 = 1;

/// How long an escrow account's data is.
pub const LEN: usize = 1 + 32 + 32 + 1;

/// What an escrow account records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Escrow {
	/// The wallet that released the husk, and the only one its receipt may
	/// be made out to.
	pub wallet: Address,
	/// The husk's mint.
	pub mint: Address,
	pub bump: u8,
}

impl Escrow {
	/// The escrow account's data.
	pub fn encode(&self) -> [u8; LEN] {
		let mut bytes = [0; LEN];

		bytes[0] = DISCRIMINATOR;
		bytes[1..33].copy_from_slice(self.wallet.as_bytes());
		bytes[33..65].copy_from_slice(self.mint.as_bytes());
		bytes[65] = self.bump;

		bytes
	}

	/// The escrow `data` records, or `None` when it is not an escrow
	/// account's data.
	pub fn decode(data: &[u8]) -> Option<Self> {
		let data: &[u8; LEN] = data.try_into().ok()?;

		if data[0] != DISCRIMINATOR {
			return None;
		}

		Some(Self {
			wallet: Address::from_bytes(data[1..33].try_into().ok()?),
			mint: Address::from_bytes(data[33..65].try_into().ok()?),
			bump: data[65],
		})
	}
}

/// The escrow account for `mint` under `program`, and its bump.
pub fn address(program: &Address, mint: &Address) -> Option<(Address, u8)> {
	find_program_address(&[SEED, mint.as_bytes()], program)
		.ok()
		.map(|derived| (derived.address(), derived.bump()))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_escrow_reads_back_as_written() {
		let escrow = Escrow {
			wallet: Address::from_bytes([3; 32]),
			mint: Address::from_bytes([4; 32]),
			bump: 254,
		};

		assert_eq!(Escrow::decode(&escrow.encode()), Some(escrow));
		assert_eq!(Escrow::decode(&escrow.encode()[1..]), None);

		let mut other = escrow.encode();

		other[0] = 2;

		assert_eq!(Escrow::decode(&other), None);
	}

	#[test]
	fn each_mint_has_its_own_escrow() {
		let program = Address::from_bytes([9; 32]);
		let first = address(&program, &Address::from_bytes([1; 32]));
		let second = address(&program, &Address::from_bytes([2; 32]));

		assert!(first.is_some());
		assert_ne!(first, second);
	}
}
