//! A husk escrowed with the Reapers program: where its escrow account is,
//! and what that account records.
//!
//! The address and the layout are the program's own, from its generated
//! client: the escrow is the program-derived address of `["escrow", mint]`,
//! and its data is the program's `Escrow` account. Reading it here through
//! the client means a change to the program changes the keeper with it.

use reapers_program_client::REAPERS_PROGRAM_ID;
use reapers_program_client::accounts::Escrow as ProgramEscrow;
use solana_kit_core::Address;
use solana_pubkey::Pubkey;

/// How long an escrow account's data is.
pub const LEN: usize = ProgramEscrow::LEN;

/// What kind of husk an escrow holds, which decides how it is burned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HuskKind {
	/// A Metaplex Core asset, owned by the escrow.
	Core,
	/// An SPL Token mint, in a token account the escrow owns.
	Token,
	/// A Token-2022 mint, in a token account the escrow owns.
	Token2022,
}

impl HuskKind {
	/// The kind the program stores as `byte`.
	pub const fn from_byte(byte: u8) -> Option<Self> {
		match byte {
			1 => Some(Self::Core),
			2 => Some(Self::Token),
			3 => Some(Self::Token2022),
			_ => None,
		}
	}

	/// How the program stores this kind.
	pub const fn byte(self) -> u8 {
		match self {
			Self::Core => 1,
			Self::Token => 2,
			Self::Token2022 => 3,
		}
	}
}

/// What an escrow account records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Escrow {
	/// The wallet that released the husk, and the only one its receipt may
	/// be made out to.
	pub wallet: Address,
	/// The husk's mint, or its asset address for a Core husk.
	pub mint: Address,
	pub kind: HuskKind,
	pub bump: u8,
}

impl Escrow {
	/// The escrow account's data, exactly as the program writes it.
	pub fn encode(&self) -> Vec<u8> {
		let mut data = vec![0; LEN];

		ProgramEscrow::initialize(&mut data, |account| {
			account.player = pubkey(&self.wallet);
			account.mint = pubkey(&self.mint);
			account.kind = self.kind.byte();
			account.bump = self.bump;
		})
		.expect("an escrow account's data is exactly its length");

		data
	}

	/// The escrow `data` records, or `None` when it is not the program's
	/// escrow account data.
	pub fn decode(data: &[u8]) -> Option<Self> {
		let account = ProgramEscrow::from_bytes(data).ok()?;

		Some(Self {
			wallet: address(&account.player),
			mint: address(&account.mint),
			kind: HuskKind::from_byte(account.kind)?,
			bump: account.bump,
		})
	}
}

/// The escrow account for `mint`, and its bump.
pub fn address_of(mint: &Address) -> (Address, u8) {
	let (escrow, bump) = ProgramEscrow::find_pda(&pubkey(mint));

	(address(&escrow), bump)
}

/// The Reapers program the client was generated for: the only program the
/// keeper can build instructions for.
pub fn program() -> Address {
	address(&REAPERS_PROGRAM_ID)
}

/// The same 32 bytes, as the program client's key.
pub fn pubkey(address: &Address) -> Pubkey {
	Pubkey::new_from_array(*address.as_bytes())
}

/// The same 32 bytes, as Solana Kit's address.
pub fn address(pubkey: &Pubkey) -> Address {
	Address::from_bytes(pubkey.to_bytes())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn an_escrow_reads_back_as_written() {
		let escrow = Escrow {
			wallet: Address::from_bytes([3; 32]),
			mint: Address::from_bytes([4; 32]),
			kind: HuskKind::Core,
			bump: 254,
		};

		assert_eq!(Escrow::decode(&escrow.encode()), Some(escrow));
		assert_eq!(Escrow::decode(&escrow.encode()[1..]), None);

		let mut other = escrow.encode();

		other[0] = other[0].wrapping_add(1);

		assert_eq!(
			Escrow::decode(&other),
			None,
			"another account's discriminator"
		);
	}

	#[test]
	fn each_mint_has_its_own_escrow_under_the_program() {
		let first = address_of(&Address::from_bytes([1; 32]));
		let second = address_of(&Address::from_bytes([2; 32]));

		assert_ne!(first, second);
		assert_eq!(
			first,
			{
				let (escrow, bump) =
					Pubkey::find_program_address(&[b"escrow", &[1; 32]], &REAPERS_PROGRAM_ID);
				(address(&escrow), bump)
			},
			"the seeds are [\"escrow\", mint]"
		);
	}
}
