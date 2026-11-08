//! The two instructions that absorb a husk: the Ed25519 verification of the
//! keeper's receipt, and `AbsorbCore` or `AbsorbToken` straight after it.
//!
//! The program reads the receipt from the instruction immediately before
//! `Absorb`, so these always go into a transaction together and in this order.
//! Compute budget instructions may come before both.

use reapers_receipt::Receipt;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::accounts::Escrow;
use crate::instructions::AbsorbCore;
use crate::instructions::AbsorbCoreInstructionData;
use crate::instructions::AbsorbToken;
use crate::instructions::AbsorbTokenInstructionData;

/// Solana's Ed25519 signature verification program.
pub const ED25519_PROGRAM_ID: Pubkey = Pubkey::new_from_array(reapers_receipt::ED25519_PROGRAM_ID);

/// The instructions sysvar, which `Absorb` reads the receipt from.
pub const INSTRUCTIONS_SYSVAR_ID: Pubkey =
	Pubkey::from_str_const("Sysvar1nstructions1111111111111111111111111");

/// The Metaplex Core program.
pub const CORE_PROGRAM_ID: Pubkey =
	Pubkey::from_str_const("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

/// The associated token account program.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
	Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

/// The Ed25519 program instruction that verifies `signature`, by `keeper`,
/// over `receipt`, in the self-contained shape the program accepts.
pub fn verify_receipt(keeper: &Pubkey, signature: &[u8; 64], receipt: &Receipt) -> Instruction {
	Instruction {
		program_id: ED25519_PROGRAM_ID,
		accounts: Vec::new(),
		data: reapers_receipt::ed25519_instruction_data(
			&keeper.to_bytes(),
			signature,
			&receipt.encode(),
		)
		.to_vec(),
	}
}

/// The escrow that holds `mint` once it is released.
pub fn escrow(mint: &Pubkey) -> Pubkey {
	Escrow::find_pda(mint).0
}

/// The escrow's token account for a classic NFT under `token_program`.
pub fn vault(mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[escrow(mint).as_ref(), token_program.as_ref(), mint.as_ref()],
		&ASSOCIATED_TOKEN_PROGRAM_ID,
	)
	.0
}

/// Absorbing a Core asset: the receipt's verification, then `AbsorbCore`.
/// `collection` is the asset's collection, if it has one.
pub fn absorb_core(
	keeper: &Pubkey,
	signature: &[u8; 64],
	receipt: &Receipt,
	collection: Option<Pubkey>,
) -> [Instruction; 2] {
	let player = Pubkey::new_from_array(receipt.player);
	let asset = Pubkey::new_from_array(receipt.mint);
	let absorb = AbsorbCore::new(
		player,
		escrow(&asset),
		asset,
		collection.unwrap_or(CORE_PROGRAM_ID),
		CORE_PROGRAM_ID,
		INSTRUCTIONS_SYSVAR_ID,
	);
	let mut instruction = absorb.instruction(
		AbsorbCoreInstructionData::new(|_| {})
			.unwrap_or_else(|error| panic!("an empty instruction encodes: {error}")),
	);
	// A real collection is written: burning shrinks it.
	if collection.is_some() {
		instruction.accounts[4].is_writable = true;
	}

	[verify_receipt(keeper, signature, receipt), instruction]
}

/// Absorbing a classic NFT under `token_program`: the receipt's verification,
/// then `AbsorbToken`.
pub fn absorb_token(
	keeper: &Pubkey,
	signature: &[u8; 64],
	receipt: &Receipt,
	token_program: &Pubkey,
) -> [Instruction; 2] {
	let player = Pubkey::new_from_array(receipt.player);
	let mint = Pubkey::new_from_array(receipt.mint);
	let absorb = AbsorbToken::new(
		player,
		mint,
		vault(&mint, token_program),
		*token_program,
		INSTRUCTIONS_SYSVAR_ID,
	);
	let instruction = absorb.instruction(
		AbsorbTokenInstructionData::new(|_| {})
			.unwrap_or_else(|error| panic!("an empty instruction encodes: {error}")),
	);

	[verify_receipt(keeper, signature, receipt), instruction]
}
