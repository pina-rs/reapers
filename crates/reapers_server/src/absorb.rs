//! The transaction that burns an escrowed husk: the native Ed25519 check of
//! the keeper's receipt, then the program's `Absorb`.
//!
//! The program reads the receipt from the Ed25519 instruction through the
//! instructions sysvar, so both travel in one transaction and the burn
//! happens only when the signature checks.

use solana_kit_core::AccountMeta;
use solana_kit_core::AccountRole;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_kit_core::Signature;
use solana_kit_core::address_constants::ED25519_PROGRAM;
use solana_kit_core::address_constants::SYSVAR_INSTRUCTIONS;
use solana_kit_core::transaction::Lifetime;
use solana_kit_core::transaction::Message;
use solana_kit_core::transaction::Transaction;
use solana_kit_core::transaction::Version;

use crate::escrow;

/// The native Ed25519 program's instruction checking `signature` by
/// `signer` over `message`, all carried in the instruction itself.
///
/// Its data is a count of one, a padding byte, one fourteen-byte offsets
/// entry, then the key, the signature and the message. An instruction index
/// of `u16::MAX` points each offset at this same instruction.
pub fn ed25519_instruction(
	signer: &Address,
	signature: &Signature,
	message: &[u8],
) -> Option<Instruction> {
	const HEADER: usize = 2 + 14;
	const THIS: u16 = u16::MAX;

	let key_at = HEADER;
	let signature_at = key_at + 32;
	let message_at = signature_at + 64;
	let offset = |at: usize| u16::try_from(at).ok();
	let mut data = Vec::with_capacity(message_at + message.len());

	data.extend_from_slice(&[1, 0]);

	for field in [
		offset(signature_at)?,
		THIS,
		offset(key_at)?,
		THIS,
		offset(message_at)?,
		offset(message.len())?,
		THIS,
	] {
		data.extend_from_slice(&field.to_le_bytes());
	}

	data.extend_from_slice(signer.as_bytes());
	data.extend_from_slice(signature.as_bytes());
	data.extend_from_slice(message);

	Some(Instruction {
		program_address: ED25519_PROGRAM,
		accounts: Vec::new(),
		data,
	})
}

/// The program's `Absorb`: burn the husk escrowed for `mint`, released by
/// `wallet`, on the strength of the receipt in the transaction's Ed25519
/// instruction.
///
/// TODO(integration): replace with the program's generated client. This is
/// a minimal hand-written builder: the discriminator and the account order
/// are the keeper's reading of the program.
pub fn absorb_instruction(
	program: &Address,
	wallet: &Address,
	mint: &Address,
) -> Option<Instruction> {
	/// `Absorb`'s discriminator.
	const ABSORB: u8 = 1;

	let (escrow, _) = escrow::address(program, mint)?;

	Some(Instruction {
		program_address: *program,
		accounts: vec![
			AccountMeta {
				address: *wallet,
				role: AccountRole::WritableSigner,
			},
			AccountMeta {
				address: escrow,
				role: AccountRole::Writable,
			},
			AccountMeta {
				address: *mint,
				role: AccountRole::Writable,
			},
			AccountMeta {
				address: solana_kit_mpl_core::PROGRAM,
				role: AccountRole::Readonly,
			},
			AccountMeta {
				address: SYSVAR_INSTRUCTIONS,
				role: AccountRole::Readonly,
			},
		],
		data: vec![ABSORB],
	})
}

/// The unsigned legacy transaction the wallet signs and sends: the receipt's
/// check, then the burn, paid for by the wallet.
pub fn transaction(
	program: &Address,
	keeper: &Address,
	wallet: &Address,
	mint: &Address,
	receipt: &[u8],
	signature: &Signature,
	lifetime: Lifetime,
) -> Option<Vec<u8>> {
	let instructions = [
		ed25519_instruction(keeper, signature, receipt)?,
		absorb_instruction(program, wallet, mint)?,
	];
	let message = Message::compile(Version::Legacy, *wallet, lifetime, &instructions, &[]).ok()?;

	Transaction::new(message).encode(false).ok()
}

#[cfg(test)]
mod tests {
	use solana_kit_core::Hash;
	use solana_kit_core::crypto::Keypair;

	use super::*;

	#[test]
	fn the_ed25519_instruction_points_at_its_own_data() {
		let keeper = Keypair::from_seed(&[5; 32]);
		let message = [8; 98];
		let signature = keeper.sign(&message);
		let instruction =
			ed25519_instruction(&keeper.address(), &signature, &message).expect("it fits");
		let data = &instruction.data;
		let read = |at: usize| u16::from_le_bytes([data[at], data[at + 1]]);

		assert_eq!(instruction.program_address, ED25519_PROGRAM);
		assert_eq!(data[0], 1);
		assert_eq!(usize::from(read(2)), 48, "signature offset");
		assert_eq!(read(4), u16::MAX);
		assert_eq!(usize::from(read(6)), 16, "public key offset");
		assert_eq!(usize::from(read(10)), 112, "message offset");
		assert_eq!(usize::from(read(12)), 98, "message size");
		assert_eq!(&data[16..48], keeper.address().as_bytes());
		assert_eq!(&data[48..112], signature.as_bytes());
		assert_eq!(&data[112..], &message);
		assert_eq!(data.len(), 210);
	}

	#[test]
	fn the_wallet_pays_for_and_signs_the_burn() {
		let program = Address::from_bytes([7; 32]);
		let keeper = Keypair::from_seed(&[5; 32]);
		let wallet = Keypair::from_seed(&[6; 32]);
		let mint = Address::from_bytes([9; 32]);
		let receipt = [1; 98];
		let bytes = transaction(
			&program,
			&keeper.address(),
			&wallet.address(),
			&mint,
			&receipt,
			&keeper.sign(&receipt),
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
