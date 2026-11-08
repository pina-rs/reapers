//! The transaction that burns an escrowed husk: the native Ed25519 check of
//! the keeper's receipt, then the program's `AbsorbCore` or `AbsorbToken`.
//!
//! The program reads the receipt from the Ed25519 instruction immediately
//! before the burn, through the instructions sysvar, so both travel in one
//! transaction and the burn happens only when the signature checks. Both
//! instructions come from the program's generated client, the same builders
//! its own tests and `reapers_ops` use.

use reapers_program_client::absorb;
use reapers_receipt::Receipt;
use solana_kit_core::AccountMeta;
use solana_kit_core::AccountRole;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_kit_core::Signature;
use solana_kit_core::transaction::Lifetime;
use solana_kit_core::transaction::Message;
use solana_kit_core::transaction::Transaction;
use solana_kit_core::transaction::Version;
use solana_pubkey::Pubkey;

use crate::escrow;
use crate::escrow::HuskKind;

/// The SPL Token program.
pub const TOKEN_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// The Token-2022 program.
pub const TOKEN_2022_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// What the burn needs to know about the husk beyond its mint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Burn {
	/// A Core asset, with its collection when it has one: burning a member
	/// of a collection writes to the collection.
	Core { collection: Option<Address> },
	/// A classic NFT under the SPL Token program.
	Token,
	/// A classic NFT under the Token-2022 program.
	Token2022,
}

impl Burn {
	/// The burn for a husk of `kind`; `collection` matters only to Core.
	pub const fn of(kind: HuskKind, collection: Option<Address>) -> Self {
		match kind {
			HuskKind::Core => Self::Core { collection },
			HuskKind::Token => Self::Token,
			HuskKind::Token2022 => Self::Token2022,
		}
	}
}

/// The receipt's verification and the burn, in the order the program needs.
pub fn instructions(
	keeper: &Address,
	receipt: &Receipt,
	signature: &Signature,
	burn: Burn,
) -> [Instruction; 2] {
	let keeper = escrow::pubkey(keeper);
	let signature = signature.as_bytes();
	let pair = match burn {
		Burn::Core { collection } => {
			absorb::absorb_core(
				&keeper,
				signature,
				receipt,
				collection.as_ref().map(escrow::pubkey),
			)
		}
		Burn::Token => absorb::absorb_token(&keeper, signature, receipt, &TOKEN_PROGRAM),
		Burn::Token2022 => absorb::absorb_token(&keeper, signature, receipt, &TOKEN_2022_PROGRAM),
	};

	pair.map(instruction)
}

/// The unsigned legacy transaction the wallet signs and sends: the receipt's
/// check, then the burn, paid for by the wallet the receipt names.
pub fn transaction(
	keeper: &Address,
	receipt: &Receipt,
	signature: &Signature,
	burn: Burn,
	lifetime: Lifetime,
) -> Option<Vec<u8>> {
	let wallet = Address::from_bytes(receipt.player);
	let instructions = instructions(keeper, receipt, signature, burn);
	let message = Message::compile(Version::Legacy, wallet, lifetime, &instructions, &[]).ok()?;

	Transaction::new(message).encode(false).ok()
}

/// The client's instruction, in Solana Kit's terms.
fn instruction(instruction: solana_instruction::Instruction) -> Instruction {
	Instruction {
		program_address: escrow::address(&instruction.program_id),
		accounts: instruction
			.accounts
			.iter()
			.map(|meta| {
				AccountMeta {
					address: escrow::address(&meta.pubkey),
					role: match (meta.is_signer, meta.is_writable) {
						(true, true) => AccountRole::WritableSigner,
						(true, false) => AccountRole::ReadonlySigner,
						(false, true) => AccountRole::Writable,
						(false, false) => AccountRole::Readonly,
					},
				}
			})
			.collect(),
		data: instruction.data,
	}
}

#[cfg(test)]
mod tests {
	use reapers_receipt::Feeding;
	use reapers_receipt::Grade;
	use solana_kit_core::Hash;
	use solana_kit_core::address_constants::ED25519_PROGRAM;
	use solana_kit_core::crypto::Keypair;

	use super::*;

	fn receipt(wallet: &Keypair) -> Receipt {
		Receipt {
			player: *wallet.address().as_bytes(),
			mint: [9; 32],
			grade: Grade::from_byte(1).unwrap(),
			feeding: Feeding::from_byte(1).unwrap(),
			xp: 120,
			anima: 4,
			at: 1_790_000_000,
			nonce: 7,
		}
	}

	#[test]
	fn the_ed25519_check_carries_the_receipt_and_the_burn_follows_it() {
		let keeper = Keypair::from_seed(&[5; 32]);
		let wallet = Keypair::from_seed(&[6; 32]);
		let receipt = receipt(&wallet);
		let signature = keeper.sign(&receipt.encode());

		for burn in [
			Burn::Core { collection: None },
			Burn::Core {
				collection: Some(Address::from_bytes([4; 32])),
			},
			Burn::Token,
			Burn::Token2022,
		] {
			let [check, absorb] = instructions(&keeper.address(), &receipt, &signature, burn);

			assert_eq!(check.program_address, ED25519_PROGRAM, "{burn:?}");
			assert!(check.data.ends_with(&receipt.encode()), "{burn:?}");
			assert_eq!(absorb.program_address, escrow::program(), "{burn:?}");
			assert_eq!(
				absorb.accounts[0],
				AccountMeta {
					address: wallet.address(),
					role: AccountRole::WritableSigner,
				},
				"the wallet signs the burn: {burn:?}"
			);
		}
	}

	#[test]
	fn the_wallet_pays_for_and_signs_the_burn() {
		let keeper = Keypair::from_seed(&[5; 32]);
		let wallet = Keypair::from_seed(&[6; 32]);
		let receipt = receipt(&wallet);
		let bytes = transaction(
			&keeper.address(),
			&receipt,
			&keeper.sign(&receipt.encode()),
			Burn::Core { collection: None },
			Lifetime::Blockhash {
				blockhash: Hash::from_bytes([3; 32]),
				last_valid_block_height: 100,
			},
		)
		.expect("it compiles");
		let unsigned = Transaction::decode(&bytes).expect("it decodes");

		assert_eq!(unsigned.message().signers(), &[wallet.address()]);
		assert_eq!(unsigned.message().instructions().len(), 2);

		let signed = unsigned.sign(&wallet).expect("the wallet signs");

		assert!(signed.verify().is_ok());
	}
}
