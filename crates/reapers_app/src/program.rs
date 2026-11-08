//! The Reapers program's instructions, as the app sends them: built by the
//! program's generated client (`reapers_program_client`), the same builders
//! the server, `reapers_ops` and the program's own tests use, and turned
//! into Solana Kit's instructions for signing.
//!
//! The simnet and the real cluster both go through here, so a simnet
//! release or burn is exactly the transaction a cluster takes.

use reapers_program_client::REAPERS_PROGRAM_ID;
use reapers_program_client::absorb;
use reapers_program_client::instructions::ReleaseCore;
use reapers_program_client::instructions::ReleaseCoreInstructionData;
use reapers_receipt::Receipt;
use solana_kit_core::AccountMeta;
use solana_kit_core::AccountRole;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_pubkey::Pubkey;

/// The Reapers program.
pub fn program_id() -> Address {
	address(&REAPERS_PROGRAM_ID)
}

/// Metaplex Core.
pub fn core_program() -> Address {
	address(&absorb::CORE_PROGRAM_ID)
}

/// The escrow that holds the husk `mint` once it is released:
/// `["escrow", mint]` under the program.
pub fn escrow(mint: &Address) -> Address {
	address(&absorb::escrow(&pubkey(mint)))
}

/// `ReleaseCore`: `player` moves the Core asset `asset` into its escrow.
/// `collection` is the asset's collection, if it has one.
pub fn release_core(
	player: &Address,
	asset: &Address,
	collection: Option<&Address>,
) -> Instruction {
	let asset = pubkey(asset);
	let data = ReleaseCoreInstructionData::new(|_| {})
		.unwrap_or_else(|error| panic!("an empty instruction encodes: {error}"));
	let mut instruction = ReleaseCore::new(
		pubkey(player),
		absorb::escrow(&asset),
		asset,
		collection.map_or(absorb::CORE_PROGRAM_ID, pubkey),
		absorb::CORE_PROGRAM_ID,
	)
	.instruction(data);
	// The program reads the collection; Core writes it only on a burn.
	instruction.accounts[4].is_writable = false;
	kit(instruction)
}

/// The keeper's receipt check and `AbsorbCore`, in the order the program
/// requires: `signature` is `keeper`'s over `receipt`.
pub fn absorb_core(
	keeper: &Address,
	signature: &[u8; 64],
	receipt: &Receipt,
	collection: Option<&Address>,
) -> [Instruction; 2] {
	absorb::absorb_core(&pubkey(keeper), signature, receipt, collection.map(pubkey)).map(kit)
}

/// The client's instruction, in Solana Kit's terms.
pub fn kit(instruction: solana_instruction::Instruction) -> Instruction {
	Instruction {
		program_address: address(&instruction.program_id),
		accounts: instruction
			.accounts
			.iter()
			.map(|meta| {
				AccountMeta {
					address: address(&meta.pubkey),
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

fn pubkey(address: &Address) -> Pubkey {
	Pubkey::new_from_array(*address.as_bytes())
}

fn address(pubkey: &Pubkey) -> Address {
	Address::from_bytes(pubkey.to_bytes())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_release_is_signed_by_the_player_and_moves_the_asset_into_its_escrow() {
		let player = Address::from_bytes([1; 32]);
		let asset = Address::from_bytes([2; 32]);
		let release = release_core(&player, &asset, None);

		assert_eq!(release.program_address, program_id());
		assert_eq!(release.accounts[0].address, player);
		assert_eq!(release.accounts[0].role, AccountRole::WritableSigner);
		assert_eq!(release.accounts[2].address, escrow(&asset));
		assert_eq!(release.accounts[3].address, asset);
		assert_eq!(release.accounts[4].address, core_program(), "no collection");
	}
}
