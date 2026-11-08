//! The two Metaplex Core instructions the program calls: `TransferV1` into
//! escrow and `BurnV1` out of it.
//!
//! A generated CPI crate for all of Core would be far larger than the two
//! fixed instructions used here, so they are written out by hand. Both pass
//! `None` for every optional argument: Core's convention for an absent account
//! is the Core program's own address, and an absent `compression_proof` is a
//! single zero byte.

use pina::pinocchio::cpi::invoke;
use pina::pinocchio::cpi::invoke_signed;
use pina::*;

use crate::ReapersError;

/// The Metaplex Core program, `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`.
pub const CORE_PROGRAM_ID: Address = address!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

/// The first byte of an `AssetV1` account (`Key::AssetV1`).
const ASSET_V1: u8 = 1;

/// `TransferV1 { compression_proof: None }`.
const TRANSFER_V1: [u8; 2] = [14, 0];

/// `BurnV1 { compression_proof: None }`.
const BURN_V1: [u8; 2] = [12, 0];

/// The owner recorded in a Core asset, after checking that the account is one.
///
/// An `AssetV1` starts with its key byte and then its 32-byte owner.
pub fn asset_owner(asset: &AccountView) -> Result<Address, ProgramError> {
	if !asset.owned_by(&CORE_PROGRAM_ID) {
		return Err(ReapersError::NotAnNft.into());
	}

	let data = asset.try_borrow()?;

	if data.len() < 33 || data[0] != ASSET_V1 {
		return Err(ReapersError::NotAnNft.into());
	}

	Address::try_from(&data[1..33]).map_err(|_| ReapersError::NotAnNft.into())
}

/// Moves `asset` from `owner`, who signs and pays, to `new_owner`.
///
/// `collection` is the asset's collection, or the Core program when it has
/// none. Core checks that it is the right one.
pub fn transfer(
	asset: &AccountView,
	collection: &AccountView,
	owner: &AccountView,
	new_owner: &AccountView,
	core_program: &AccountView,
) -> ProgramResult {
	let accounts = [
		InstructionAccount::writable(asset.address()),
		InstructionAccount::readonly(collection.address()),
		InstructionAccount::writable_signer(owner.address()),
		// No authority: Core takes the payer as the authority.
		InstructionAccount::readonly(&CORE_PROGRAM_ID),
		InstructionAccount::readonly(new_owner.address()),
		InstructionAccount::readonly(&CORE_PROGRAM_ID),
		InstructionAccount::readonly(&CORE_PROGRAM_ID),
	];
	let instruction = InstructionView {
		program_id: &CORE_PROGRAM_ID,
		data: &TRANSFER_V1,
		accounts: &accounts,
	};

	invoke(
		&instruction,
		&[
			asset,
			collection,
			owner,
			core_program,
			new_owner,
			core_program,
			core_program,
		],
	)
}

/// Burns `asset`, whose owner `authority` signs through `signers`. `payer`
/// signs too.
///
/// Core does not delete the account: it shrinks it to a one-byte tombstone
/// (`Key::Uninitialized`), so the address can never hold an asset again, and
/// refunds the rest of the asset's rent to `payer`.
pub fn burn(
	asset: &AccountView,
	collection: &AccountView,
	payer: &AccountView,
	authority: &AccountView,
	core_program: &AccountView,
	signers: &[Signer<'_, '_>],
) -> ProgramResult {
	// Burning decrements the collection's size, so a real collection is
	// written. The Core program standing in for "none" cannot be.
	let collection_account = if collection.address() == &CORE_PROGRAM_ID {
		InstructionAccount::readonly(collection.address())
	} else {
		InstructionAccount::writable(collection.address())
	};
	let accounts = [
		InstructionAccount::writable(asset.address()),
		collection_account,
		InstructionAccount::writable_signer(payer.address()),
		InstructionAccount::readonly_signer(authority.address()),
		InstructionAccount::readonly(&CORE_PROGRAM_ID),
		InstructionAccount::readonly(&CORE_PROGRAM_ID),
	];
	let instruction = InstructionView {
		program_id: &CORE_PROGRAM_ID,
		data: &BURN_V1,
		accounts: &accounts,
	};

	invoke_signed(
		&instruction,
		&[
			asset,
			collection,
			payer,
			authority,
			core_program,
			core_program,
		],
		signers,
	)
}
