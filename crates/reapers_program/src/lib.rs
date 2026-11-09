//! The Reapers program: husks go into escrow, and come out only as ash.
//!
//! A husk is a dead NFT a player owns. Releasing it moves it into an escrow
//! the program owns, and there is no way back: from then on it can only be
//! hunted. When the server has replayed a hunt and judged the husk absorbed,
//! its keeper key signs a receipt (`reapers_receipt`), and the player posts a
//! transaction that verifies that signature with the Ed25519 program and then
//! calls `Absorb`, which burns the husk and closes its escrow.
//!
//! Two kinds of husk are supported: Metaplex Core assets, and classic NFTs
//! that are SPL Token or Token-2022 mints with a supply of one and no
//! decimals.
//!
//! See `docs/program.md` for the accounts, the instructions and the argument
//! that a receipt cannot be used twice.

#![allow(clippy::inline_always)]
// Pina's idioms, which the workspace's pedantic lints dislike: `use pina::*`,
// `&AccountView` everywhere (an 8-byte handle), and the `Clone` impls and
// `self` receivers its macros generate.
#![allow(
	clippy::wildcard_imports,
	clippy::trivially_copy_pass_by_ref,
	clippy::expl_impl_clone_on_copy,
	clippy::unused_self
)]
#![no_std]

#[cfg(all(
	not(target_arch = "bpf"),
	not(target_os = "solana"),
	not(feature = "bpf-entrypoint"),
	not(test)
))]
extern crate std;

#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint;

pub mod mpl_core;
pub mod receipt;

use pina::token_2022::state::ExtensionType;
use pina::*;

declare_id!("HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP");

/// Seed of the one configuration account.
pub const CONFIG_SEED: &[u8] = b"config";

/// Seed prefix of a husk's escrow; the husk's mint or asset address follows.
pub const ESCROW_SEED: &[u8] = b"escrow";

/// The BPF upgradeable loader, which owns this program's executable data.
pub const BPF_LOADER_UPGRADEABLE_ID: Address =
	address!("BPFLoaderUpgradeab1e11111111111111111111111");

/// Token-2022 extensions a husk's mint may carry. Each one describes the NFT,
/// and none of them can move, freeze or burn it behind the escrow's back, or
/// let the mint be closed and made again at the same address.
pub const ALLOWED_MINT_EXTENSIONS: [ExtensionType; 4] = [
	ExtensionType::MetadataPointer,
	ExtensionType::TokenMetadata,
	ExtensionType::GroupMemberPointer,
	ExtensionType::TokenGroupMember,
];

/// The Metaplex Token Metadata program. A classic NFT's mint and freeze
/// authority is its master edition, an account of this program that never
/// mints more of its own mint.
pub const TOKEN_METADATA_ID: Address = address!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

const TOKEN_PROGRAM_IDS: [Address; 2] = [token::ID, token_2022::ID];

// ---------------------------------------------------------------------------
// Discriminators
// ---------------------------------------------------------------------------

#[discriminator(entrypoint)]
pub enum ReapersInstruction {
	Initialize = 0,
	SetKeeper = 1,
	SetAuthority = 2,
	SetPaused = 3,
	ReleaseCore = 4,
	ReleaseToken = 5,
	AbsorbCore = 6,
	AbsorbToken = 7,
}

#[discriminator]
pub enum ReapersAccount {
	Config = 1,
	Escrow = 2,
}

#[discriminator]
pub enum ReapersEvent {
	HuskReleased = 1,
	HuskAbsorbed = 2,
}

/// What kind of NFT an escrow holds. Stored as a byte in [`Escrow::kind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HuskKind {
	/// A Metaplex Core asset, held by the escrow as its owner.
	Core = 1,
	/// An SPL Token mint, held in a token account the escrow owns.
	Token = 2,
	/// A Token-2022 mint, held in a token account the escrow owns.
	Token2022 = 3,
}

#[error]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReapersError {
	/// Releasing and absorbing are paused.
	Paused = 0,
	/// The signer is not the configured authority.
	NotAuthority = 1,
	/// The signer is not the program's upgrade authority.
	NotUpgradeAuthority = 2,
	/// The husk is not an NFT the program accepts.
	NotAnNft = 3,
	/// The player does not own the husk.
	NotOwner = 4,
	/// The mint carries a Token-2022 extension the program does not accept.
	UnsupportedExtension = 5,
	/// The instruction before `Absorb` is not an Ed25519 verification.
	MissingReceipt = 6,
	/// The Ed25519 instruction is not one signature over its own data.
	UnsupportedReceiptInstruction = 7,
	/// The receipt is not signed by the configured keeper.
	WrongKeeper = 8,
	/// The signed message is not a version 2 receipt.
	MalformedReceipt = 9,
	/// The receipt names another player, or the escrow belongs to another.
	WrongPlayer = 10,
	/// The receipt names another husk than the escrow holds.
	WrongHusk = 11,
	/// The escrow holds another kind of husk than this instruction absorbs.
	WrongKind = 12,
	/// The escrow does not hold the husk.
	EscrowEmpty = 13,
	/// The mint could still mint or freeze: its supply is not fixed.
	SupplyNotFixed = 14,
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// The one configuration account: who administers the program, and whose
/// receipts it trusts.
#[account(discriminator = ReapersAccount)]
#[pda(seeds = [CONFIG_SEED], bump = bump)]
pub struct Config {
	/// Rotates the keeper, pauses, and hands itself on.
	pub authority: Address,
	/// The Ed25519 public key whose receipts `Absorb` accepts.
	pub keeper: Address,
	/// While set, nothing can be released or absorbed.
	pub paused: bool,
	pub bump: u8,
}

/// A released husk, waiting to be hunted. Its address is derived from the
/// husk, so a husk has at most one escrow, and the escrow is what holds it:
/// as the Core asset's owner, or as the owner of the token account.
#[account(discriminator = ReapersAccount)]
#[pda(seeds = [ESCROW_SEED, mint: Address], bump = bump)]
pub struct Escrow {
	/// The wallet that released the husk; the only one that can absorb it.
	pub player: Address,
	/// The husk's mint or asset address.
	pub mint: Address,
	/// A [`HuskKind`].
	pub kind: u8,
	pub bump: u8,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// A husk went into escrow.
#[event(discriminator = ReapersEvent)]
#[derive(Debug)]
pub struct HuskReleased {
	pub player: Address,
	pub mint: Address,
	pub kind: u8,
}

/// A husk was burned, with the receipt that allowed it.
#[event(discriminator = ReapersEvent)]
#[derive(Debug)]
pub struct HuskAbsorbed {
	pub player: Address,
	pub mint: Address,
	pub grade: u8,
	pub feeding: u8,
	pub xp: u32,
	pub anima: u32,
	pub at: i64,
	pub nonce: u64,
}

// ---------------------------------------------------------------------------
// Instruction data
// ---------------------------------------------------------------------------

#[instruction(discriminator = ReapersInstruction::Initialize)]
pub struct InitializeInstruction {
	/// Who administers the program from now on.
	pub authority: Address,
	/// The keeper's Ed25519 public key.
	pub keeper: Address,
}

#[instruction(discriminator = ReapersInstruction::SetKeeper)]
pub struct SetKeeperInstruction {
	pub keeper: Address,
}

#[instruction(discriminator = ReapersInstruction::SetAuthority)]
pub struct SetAuthorityInstruction {}

#[instruction(discriminator = ReapersInstruction::SetPaused)]
pub struct SetPausedInstruction {
	pub paused: bool,
}

#[instruction(discriminator = ReapersInstruction::ReleaseCore)]
pub struct ReleaseCoreInstruction {}

#[instruction(discriminator = ReapersInstruction::ReleaseToken)]
pub struct ReleaseTokenInstruction {}

#[instruction(discriminator = ReapersInstruction::AbsorbCore)]
pub struct AbsorbCoreInstruction {}

#[instruction(discriminator = ReapersInstruction::AbsorbToken)]
pub struct AbsorbTokenInstruction {}

// ---------------------------------------------------------------------------
// Initialize
// ---------------------------------------------------------------------------

/// Creates the configuration. Only the program's upgrade authority can, so
/// nobody can initialize a freshly deployed program before its owner does.
#[derive(Accounts, Debug)]
pub struct InitializeAccounts<'a> {
	pub upgrade_authority: &'a mut AccountView,
	pub config: &'a mut AccountView,
	/// This program's executable data account, which names the upgrade
	/// authority.
	pub program_data: &'a AccountView,
	pub system_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for InitializeAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = InitializeInstruction::try_from_bytes(data)?;

		self.upgrade_authority.assert_signer()?;
		self.system_program.assert_address(&system::ID)?;
		assert_upgrade_authority(self.program_data, self.upgrade_authority.address())?;

		let authority = args.authority;
		let keeper = args.keeper;

		CreateProgramAccount {
			account: self.config,
			payer: self.upgrade_authority,
			owner: &ID,
			seeds: &Config::seeds().as_slices(),
		}
		.invoke_with_bump::<Config>(|config, bump| {
			config.authority = authority;
			config.keeper = keeper;
			config.bump = bump;
			Ok(())
		})?;

		Ok(())
	}
}

/// Checks that `program_data` is this program's data account and that
/// `signer` is its upgrade authority.
///
/// A program data account is laid out as a `u32` state tag (3), the `u64`
/// slot of the last deployment, then an optional upgrade authority: a `0` or
/// `1` byte followed by 32 bytes.
fn assert_upgrade_authority(program_data: &AccountView, signer: &Address) -> ProgramResult {
	let Some((expected, _)) = try_find_program_address(&[ID.as_ref()], &BPF_LOADER_UPGRADEABLE_ID)
	else {
		return Err(ProgramError::InvalidSeeds);
	};

	program_data
		.assert_address(&expected)?
		.assert_owner(&BPF_LOADER_UPGRADEABLE_ID)?;

	let data = program_data.try_borrow()?;

	if data.len() < 45 || data[..4] != [3, 0, 0, 0] || data[12] != 1 {
		return Err(ReapersError::NotUpgradeAuthority.into());
	}
	if &data[13..45] != signer.as_ref() {
		return Err(ReapersError::NotUpgradeAuthority.into());
	}

	Ok(())
}

// ---------------------------------------------------------------------------
// Administration
// ---------------------------------------------------------------------------

/// Replaces the keeper. Receipts the old keeper signed stop working at once.
#[derive(Accounts, Debug)]
pub struct SetKeeperAccounts<'a> {
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

impl<'a> ProcessAccountInfos<'a> for SetKeeperAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SetKeeperInstruction::try_from_bytes(data)?;

		self.authority.assert_signer()?;

		let mut config = Config::load_pda_mut(self.config, &ID)?;

		if config.authority != *self.authority.address() {
			return Err(ReapersError::NotAuthority.into());
		}

		config.keeper = args.keeper;

		Ok(())
	}
}

/// Hands the administration to another key. Both keys sign, so a mistyped
/// address cannot lock the program's owner out.
#[derive(Accounts, Debug)]
pub struct SetAuthorityAccounts<'a> {
	pub authority: &'a AccountView,
	pub new_authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

impl<'a> ProcessAccountInfos<'a> for SetAuthorityAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = SetAuthorityInstruction::try_from_bytes(data)?;

		self.authority.assert_signer()?;
		self.new_authority.assert_signer()?;

		let mut config = Config::load_pda_mut(self.config, &ID)?;

		if config.authority != *self.authority.address() {
			return Err(ReapersError::NotAuthority.into());
		}

		config.authority = *self.new_authority.address();

		Ok(())
	}
}

/// Pauses or resumes releasing and absorbing.
#[derive(Accounts, Debug)]
pub struct SetPausedAccounts<'a> {
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

impl<'a> ProcessAccountInfos<'a> for SetPausedAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SetPausedInstruction::try_from_bytes(data)?;

		self.authority.assert_signer()?;

		let mut config = Config::load_pda_mut(self.config, &ID)?;

		if config.authority != *self.authority.address() {
			return Err(ReapersError::NotAuthority.into());
		}

		config.paused = args.paused;

		Ok(())
	}
}

/// The keeper, once the configuration is checked and not paused.
fn running_keeper(config: &AccountView) -> Result<Address, ProgramError> {
	let config = Config::load_pda(config, &ID)?;

	if bool::from(config.paused) {
		return Err(ReapersError::Paused.into());
	}

	Ok(config.keeper)
}

// ---------------------------------------------------------------------------
// Release
// ---------------------------------------------------------------------------

/// Moves a Metaplex Core asset the player owns into escrow.
#[derive(Accounts, Debug)]
pub struct ReleaseCoreAccounts<'a> {
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub escrow: &'a mut AccountView,
	pub asset: &'a mut AccountView,
	/// The asset's collection, or the Core program when it has none.
	pub collection: &'a AccountView,
	pub core_program: &'a AccountView,
	pub system_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for ReleaseCoreAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = ReleaseCoreInstruction::try_from_bytes(data)?;

		self.player.assert_signer()?;
		self.core_program
			.assert_address(&mpl_core::CORE_PROGRAM_ID)?;
		self.system_program.assert_address(&system::ID)?;
		running_keeper(self.config)?;

		// The owner, not merely someone Core would let move it: a transfer
		// delegate could otherwise release an asset that is not theirs.
		if mpl_core::asset_owner(self.asset)? != *self.player.address() {
			return Err(ReapersError::NotOwner.into());
		}

		let asset = *self.asset.address();
		create_escrow(self.escrow, self.player, &asset, HuskKind::Core)?;

		mpl_core::transfer(
			self.asset,
			self.collection,
			self.player,
			self.escrow,
			self.core_program,
		)?;

		if mpl_core::asset_owner(self.asset)? != *self.escrow.address() {
			return Err(ReapersError::EscrowEmpty.into());
		}

		emit_released(self.player.address(), &asset, HuskKind::Core)
	}
}

/// Moves a classic NFT the player owns, from any of their token accounts,
/// into a token account its escrow owns.
#[derive(Accounts, Debug)]
pub struct ReleaseTokenAccounts<'a> {
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub escrow: &'a mut AccountView,
	pub mint: &'a AccountView,
	/// The player's token account that holds the NFT.
	pub player_token: &'a mut AccountView,
	/// The escrow's associated token account; created here if it is missing.
	pub vault: &'a mut AccountView,
	pub token_program: &'a AccountView,
	pub associated_token_program: &'a AccountView,
	pub system_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for ReleaseTokenAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = ReleaseTokenInstruction::try_from_bytes(data)?;

		self.player.assert_signer()?;
		self.token_program.assert_addresses(&TOKEN_PROGRAM_IDS)?;
		self.associated_token_program
			.assert_address(&associated_token_account::ID)?;
		self.system_program.assert_address(&system::ID)?;
		running_keeper(self.config)?;

		let token_program = *self.token_program.address();
		let kind = if token_program == token::ID {
			HuskKind::Token
		} else {
			HuskKind::Token2022
		};

		// An NFT: one token that cannot be divided, under a policy that
		// cannot reach into the escrow.
		let mint = self
			.mint
			.as_token_mint_for_program(&token_program)?
			.assert_extensions_allowed(&ALLOWED_MINT_EXTENSIONS)
			.map_err(|_| ProgramError::from(ReapersError::UnsupportedExtension))?;
		if mint.supply() != 1 || mint.decimals() != 0 {
			return Err(ReapersError::NotAnNft.into());
		}
		// Fixed for good, or a burned husk could be minted again, released
		// again and absorbed again with its old receipt.
		assert_fixed_supply(
			self.mint.address(),
			mint.mint_authority(),
			mint.freeze_authority(),
		)?;
		drop(mint);

		// Owned by the player, not merely delegated to them.
		let holding = self
			.player_token
			.as_token_account_for_program(&token_program)?;
		if holding.mint() != self.mint.address() || holding.amount() != 1 {
			return Err(ReapersError::NotAnNft.into());
		}
		if holding.owner() != self.player.address() {
			return Err(ReapersError::NotOwner.into());
		}
		drop(holding);

		let mint_address = *self.mint.address();
		create_escrow(self.escrow, self.player, &mint_address, kind)?;

		// Anyone can create an associated token account for any wallet, so it
		// may already exist. If it does it is still the escrow's: only the
		// escrow could have set a delegate or a close authority on it.
		associated_token_account::instructions::CreateIdempotent {
			funding_account: self.player,
			account: self.vault,
			wallet: self.escrow,
			mint: self.mint,
			system_program: self.system_program,
			token_program: self.token_program,
		}
		.invoke()?;
		let before = self
			.vault
			.as_token_account_for_program(&token_program)?
			.amount();

		token::instructions::TransferChecked::new(
			self.player_token,
			self.mint,
			self.vault,
			self.player,
			1,
			0,
		)
		.invoke_with_program(&token_program)?;

		// The vault received the token, and holds it alone.
		let vault = self.vault.as_token_account_for_program(&token_program)?;
		let received = vault
			.amount()
			.checked_sub(before)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		if vault.owner() != self.escrow.address() || received != 1 || vault.amount() != 1 {
			return Err(ReapersError::EscrowEmpty.into());
		}
		drop(vault);

		emit_released(self.player.address(), &mint_address, kind)
	}
}

/// Checks that nobody can mint more of `mint` or freeze its tokens: each
/// authority is absent, or is the mint's Token Metadata master edition.
fn assert_fixed_supply(
	mint: &Address,
	mint_authority: Option<&Address>,
	freeze_authority: Option<&Address>,
) -> ProgramResult {
	if mint_authority.is_none() && freeze_authority.is_none() {
		return Ok(());
	}

	let Some((edition, _)) = try_find_program_address(
		&[
			b"metadata",
			TOKEN_METADATA_ID.as_ref(),
			mint.as_ref(),
			b"edition",
		],
		&TOKEN_METADATA_ID,
	) else {
		return Err(ReapersError::SupplyNotFixed.into());
	};

	for authority in [mint_authority, freeze_authority].into_iter().flatten() {
		if authority != &edition {
			return Err(ReapersError::SupplyNotFixed.into());
		}
	}

	Ok(())
}

/// Creates the escrow for `mint`, released by `player`. Fails if the husk
/// already has one.
fn create_escrow(
	escrow: &mut AccountView,
	player: &AccountView,
	mint: &Address,
	kind: HuskKind,
) -> ProgramResult {
	let player_address = *player.address();

	CreateProgramAccount {
		account: escrow,
		payer: player,
		owner: &ID,
		seeds: &Escrow::seeds(mint).as_slices(),
	}
	.invoke_with_bump::<Escrow>(|escrow, bump| {
		escrow.player = player_address;
		escrow.mint = *mint;
		escrow.kind = kind as u8;
		escrow.bump = bump;
		Ok(())
	})?;

	Ok(())
}

fn emit_released(player: &Address, mint: &Address, kind: HuskKind) -> ProgramResult {
	HuskReleased::emit(|event| {
		event.player = *player;
		event.mint = *mint;
		event.kind = kind as u8;
		Ok(())
	})
}

// ---------------------------------------------------------------------------
// Absorb
// ---------------------------------------------------------------------------

/// Burns an escrowed Core asset on the strength of the keeper's receipt.
///
/// The instruction immediately before this one must be the Ed25519
/// verification of that receipt: see [`receipt`].
#[derive(Accounts, Debug)]
pub struct AbsorbCoreAccounts<'a> {
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub escrow: &'a mut AccountView,
	pub asset: &'a mut AccountView,
	/// The asset's collection, or the Core program when it has none.
	pub collection: &'a AccountView,
	pub core_program: &'a AccountView,
	pub instructions_sysvar: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for AbsorbCoreAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = AbsorbCoreInstruction::try_from_bytes(data)?;

		self.player.assert_signer()?;
		self.core_program
			.assert_address(&mpl_core::CORE_PROGRAM_ID)?;

		let asset = *self.asset.address();
		let bump = checked_absorption(
			self.config,
			self.escrow,
			self.instructions_sysvar,
			self.player.address(),
			&asset,
			HuskKind::Core,
		)?;

		let bump = [bump];
		let seeds = [
			Seed::from(ESCROW_SEED),
			Seed::from(asset.as_ref()),
			Seed::from(&bump),
		];
		let signer = Signer::from(&seeds);

		mpl_core::burn(
			self.asset,
			self.collection,
			self.player,
			self.escrow,
			self.core_program,
			&[signer],
		)?;

		self.escrow.close_account_zeroed(&ID, self.player)
	}
}

/// Burns an escrowed classic NFT on the strength of the keeper's receipt, and
/// closes the escrow's token account.
///
/// The instruction immediately before this one must be the Ed25519
/// verification of that receipt: see [`receipt`].
#[derive(Accounts, Debug)]
pub struct AbsorbTokenAccounts<'a> {
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub escrow: &'a mut AccountView,
	pub mint: &'a mut AccountView,
	pub vault: &'a mut AccountView,
	pub token_program: &'a AccountView,
	pub instructions_sysvar: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for AbsorbTokenAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = AbsorbTokenInstruction::try_from_bytes(data)?;

		self.player.assert_signer()?;
		self.token_program.assert_addresses(&TOKEN_PROGRAM_IDS)?;

		let token_program = *self.token_program.address();
		let kind = if token_program == token::ID {
			HuskKind::Token
		} else {
			HuskKind::Token2022
		};
		let mint = *self.mint.address();
		let bump = checked_absorption(
			self.config,
			self.escrow,
			self.instructions_sysvar,
			self.player.address(),
			&mint,
			kind,
		)?;

		let vault = self.vault.as_token_account_for_program(&token_program)?;
		if vault.owner() != self.escrow.address() || vault.mint() != &mint || vault.amount() != 1 {
			return Err(ReapersError::EscrowEmpty.into());
		}
		drop(vault);

		let bump = [bump];
		let seeds = [
			Seed::from(ESCROW_SEED),
			Seed::from(mint.as_ref()),
			Seed::from(&bump),
		];
		let signers = [Signer::from(&seeds)];

		token::instructions::Burn::new(self.vault, self.mint, self.escrow, 1)
			.invoke_signed_with_program(&signers, &token_program)?;
		token::instructions::CloseAccount::new(self.vault, self.player, self.escrow)
			.invoke_signed_with_program(&signers, &token_program)?;

		self.escrow.close_account_zeroed(&ID, self.player)
	}
}

/// Everything `Absorb` must know before it burns, for either kind of husk:
/// the program is running, the escrow is this husk's and this player's and
/// holds this kind, and the keeper signed a receipt naming both. Emits the
/// absorption and returns the escrow's bump.
fn checked_absorption(
	config: &AccountView,
	escrow: &AccountView,
	instructions_sysvar: &AccountView,
	player: &Address,
	mint: &Address,
	kind: HuskKind,
) -> Result<u8, ProgramError> {
	let keeper = running_keeper(config)?;

	let bump = {
		let escrow = Escrow::load_pda(escrow, mint, &ID)?;

		if escrow.player != *player {
			return Err(ReapersError::WrongPlayer.into());
		}
		if escrow.kind != kind as u8 {
			return Err(ReapersError::WrongKind.into());
		}

		escrow.bump
	};

	let receipt = receipt::verified_receipt(instructions_sysvar, &keeper)?;

	if receipt.player != player.to_bytes() {
		return Err(ReapersError::WrongPlayer.into());
	}
	if receipt.mint != mint.to_bytes() {
		return Err(ReapersError::WrongHusk.into());
	}

	HuskAbsorbed::emit(|event| {
		event.player = *player;
		event.mint = *mint;
		event.grade = receipt.grade as u8;
		event.feeding = receipt.feeding as u8;
		event.xp = receipt.xp.into();
		event.anima = receipt.anima.into();
		event.at = receipt.at.into();
		event.nonce = receipt.nonce.into();
		Ok(())
	})?;

	Ok(bump)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Discriminators are the wire format. Changing one breaks every client.
	#[test]
	fn discriminators_are_stable() {
		assert_eq!(ReapersInstruction::Initialize as u8, 0);
		assert_eq!(ReapersInstruction::SetKeeper as u8, 1);
		assert_eq!(ReapersInstruction::SetAuthority as u8, 2);
		assert_eq!(ReapersInstruction::SetPaused as u8, 3);
		assert_eq!(ReapersInstruction::ReleaseCore as u8, 4);
		assert_eq!(ReapersInstruction::ReleaseToken as u8, 5);
		assert_eq!(ReapersInstruction::AbsorbCore as u8, 6);
		assert_eq!(ReapersInstruction::AbsorbToken as u8, 7);
		assert_eq!(ReapersAccount::Config as u8, 1);
		assert_eq!(ReapersAccount::Escrow as u8, 2);
		assert_eq!(HuskKind::Core as u8, 1);
		assert_eq!(HuskKind::Token as u8, 2);
		assert_eq!(HuskKind::Token2022 as u8, 3);
	}

	#[test]
	fn names_the_programs_it_trusts() {
		assert_eq!(ID, address!("HuskN1zeKqNaWUZbxwymyoW3S8CxMNneycZWDoXc98XP"));
		assert_eq!(
			receipt::ED25519_PROGRAM_ID,
			address!("Ed25519SigVerify111111111111111111111111111")
		);
	}

	#[test]
	fn rejects_wrong_program_id() {
		let wrong_id: Address = [9u8; 32].into();
		let data = [ReapersInstruction::Initialize as u8];
		let result = parse_instruction::<ReapersInstruction>(&wrong_id, &ID, &data);

		assert!(matches!(result, Err(ProgramError::IncorrectProgramId)));
	}

	mod ed25519_data {
		use reapers_receipt::Feeding;
		use reapers_receipt::Grade;
		use reapers_receipt::Receipt;
		use reapers_receipt::ed25519_instruction_data;

		use super::super::receipt::signed_message;
		use super::super::*;

		const KEEPER: Address = Address::new_from_array([7; 32]);

		fn receipt() -> [u8; 98] {
			Receipt {
				player: [1; 32],
				mint: [2; 32],
				grade: Grade::Great,
				feeding: Feeding::Fair,
				xp: 50,
				anima: 20,
				at: 1,
				nonce: 2,
			}
			.encode()
		}

		fn data() -> [u8; 210] {
			ed25519_instruction_data(&KEEPER.to_bytes(), &[3; 64], &receipt())
		}

		fn error(code: ReapersError) -> ProgramError {
			code.into()
		}

		#[test]
		fn reads_the_message_of_a_self_contained_instruction() {
			assert_eq!(signed_message(&data(), 0, &KEEPER), Ok(&receipt()[..]));
		}

		#[test]
		fn accepts_its_own_index_in_place_of_this_instruction() {
			let mut data = data();
			for at in [4, 8, 14] {
				data[at..at + 2].copy_from_slice(&3u16.to_le_bytes());
			}
			assert_eq!(signed_message(&data, 3, &KEEPER), Ok(&receipt()[..]));
		}

		#[test]
		fn refuses_offsets_into_another_instruction() {
			for at in [4, 8, 14] {
				let mut data = data();
				data[at..at + 2].copy_from_slice(&1u16.to_le_bytes());
				assert_eq!(
					signed_message(&data, 0, &KEEPER),
					Err(error(ReapersError::UnsupportedReceiptInstruction)),
					"index field at byte {at}"
				);
			}
		}

		#[test]
		fn refuses_any_count_but_one() {
			for count in [0, 2] {
				let mut data = data();
				data[0] = count;
				assert_eq!(
					signed_message(&data, 0, &KEEPER),
					Err(error(ReapersError::UnsupportedReceiptInstruction))
				);
			}
		}

		#[test]
		fn refuses_offsets_past_the_end() {
			for (at, value) in [(2, 200u16), (6, 190), (10, 120)] {
				let mut data = data();
				data[at..at + 2].copy_from_slice(&value.to_le_bytes());
				assert_eq!(
					signed_message(&data, 0, &KEEPER),
					Err(error(ReapersError::UnsupportedReceiptInstruction)),
					"offset field at byte {at}"
				);
			}
			assert_eq!(
				signed_message(&data()[..15], 0, &KEEPER),
				Err(error(ReapersError::UnsupportedReceiptInstruction))
			);
		}

		#[test]
		fn refuses_another_key() {
			let other = Address::new_from_array([8; 32]);
			assert_eq!(
				signed_message(&data(), 0, &other),
				Err(error(ReapersError::WrongKeeper))
			);
		}

		#[test]
		fn refuses_a_message_of_another_length() {
			let mut data = data();
			data[12..14].copy_from_slice(&97u16.to_le_bytes());
			assert_eq!(
				signed_message(&data, 0, &KEEPER),
				Err(error(ReapersError::MalformedReceipt))
			);
		}
	}
}
