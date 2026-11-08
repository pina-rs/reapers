//! Reading the keeper's receipt out of the Ed25519 instruction before `Absorb`.
//!
//! The Ed25519 program verifies signatures before any program in the
//! transaction runs, and fails the whole transaction if one does not check. So
//! when an Ed25519 instruction is present, its signature is good. What this
//! module establishes is *what* was signed and *by whom*, which the Ed25519
//! program does not care about:
//!
//! - the instruction is the Ed25519 program's, and sits immediately before the
//!   current one;
//! - it verifies exactly one signature;
//! - the public key, the signature and the message all lie inside that
//!   instruction's own data. An offset may name another instruction by index,
//!   and the Ed25519 program will happily verify bytes there, so an attacker
//!   could have a real signature over some other message verified while the
//!   bytes a careless program reads say something else. Every index must be
//!   `u16::MAX` ("this instruction") or the instruction's own index;
//! - the public key is the configured keeper;
//! - the message is a 98-byte version 2 receipt.
//!
//! What the receipt says about the player and the husk is checked by the
//! caller, against the signer and the escrow.
//!
//! Pina has no helper for this yet; it is a candidate for `pina::introspection`.

use pina::sysvars::instructions::Instructions;
use pina::*;
use reapers_receipt::Receipt;
use reapers_receipt::ed25519;

use crate::ReapersError;

/// Solana's Ed25519 signature verification program.
pub const ED25519_PROGRAM_ID: Address =
	Address::new_from_array(reapers_receipt::ED25519_PROGRAM_ID);

/// The receipt the keeper signed, read from the Ed25519 instruction that
/// immediately precedes the current one.
pub fn verified_receipt(
	instructions_sysvar: &AccountView,
	keeper: &Address,
) -> Result<Receipt, ProgramError> {
	// `try_from` refuses any account but the instructions sysvar.
	let instructions = Instructions::try_from(instructions_sysvar)?;
	let current = instructions.load_current_index();
	let Some(index) = current.checked_sub(1) else {
		return Err(ReapersError::MissingReceipt.into());
	};
	let instruction = instructions.load_instruction_at(usize::from(index))?;

	if instruction.get_program_id() != &ED25519_PROGRAM_ID {
		return Err(ReapersError::MissingReceipt.into());
	}

	let receipt = signed_message(instruction.get_instruction_data(), index, keeper)?;

	Receipt::decode(receipt).map_err(|_| ReapersError::MalformedReceipt.into())
}

/// The message an Ed25519 instruction's data signs, once its shape and key
/// have been checked. `index` is the instruction's own position.
pub fn signed_message<'a>(
	data: &'a [u8],
	index: u16,
	keeper: &Address,
) -> Result<&'a [u8], ProgramError> {
	if data.len() < ed25519::HEADER + ed25519::OFFSETS || data[0] != 1 {
		return Err(ReapersError::UnsupportedReceiptInstruction.into());
	}

	let read = |field: usize| {
		let at = ed25519::HEADER + field * 2;
		u16::from_le_bytes([data[at], data[at + 1]])
	};
	let signature_offset = usize::from(read(0));
	let public_key_offset = usize::from(read(2));
	let message_offset = usize::from(read(4));
	let message_size = usize::from(read(5));

	for instruction_index in [read(1), read(3), read(6)] {
		if instruction_index != ed25519::THIS_INSTRUCTION && instruction_index != index {
			return Err(ReapersError::UnsupportedReceiptInstruction.into());
		}
	}

	let signature = data.get(signature_offset..signature_offset + 64);
	let public_key = data.get(public_key_offset..public_key_offset + 32);
	let message = data.get(message_offset..message_offset + message_size);
	let (Some(_), Some(public_key), Some(message)) = (signature, public_key, message) else {
		return Err(ReapersError::UnsupportedReceiptInstruction.into());
	};

	if public_key != keeper.as_ref() {
		return Err(ReapersError::WrongKeeper.into());
	}
	if message.len() != reapers_receipt::LEN {
		return Err(ReapersError::MalformedReceipt.into());
	}

	Ok(message)
}
