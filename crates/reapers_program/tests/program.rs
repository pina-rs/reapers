//! The program as it is deployed: the SBF artifact running in Mollusk beside
//! the real Metaplex Core, SPL Token, Token-2022 and associated token account
//! programs, with the Ed25519 precompile verifying receipts.
//!
//! Build the artifact with `pina build` first. The Metaplex Core program is
//! not in the repository: fetch it once with
//!
//! ```sh
//! solana program dump -u m CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d target/fixtures/mpl_core.so
//! ```
//!
//! or run `cargo run -p reapers_ops -- fixtures`.

#![allow(clippy::too_many_lines)]

#[path = "../src/lib.rs"]
#[allow(dead_code, unused_imports, unused_attributes)]
mod program;

// The program's modules name its root as `crate::`.
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use ed25519_dalek::Signer as _;
use ed25519_dalek::SigningKey;
use mollusk_svm::Mollusk;
use mollusk_svm::program::create_program_account_loader_v3;
use mollusk_svm::program::loader_keys::LOADER_V3;
use mollusk_svm::result::types::TransactionProgramResult;
use mollusk_svm::result::types::TransactionResult;
use program::ReapersError;
#[allow(unused_imports)]
use program::*;
use reapers_receipt::Feeding;
use reapers_receipt::Grade;
use reapers_receipt::Receipt;
use reapers_receipt::ed25519_instruction_data;
use solana_account::Account;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_instruction_error::InstructionError;
use solana_pubkey::Pubkey;

const SYSTEM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const CORE: Pubkey = Pubkey::from_str_const("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
const TOKEN: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022: Pubkey = Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ATA: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const ED25519: Pubkey = Pubkey::from_str_const("Ed25519SigVerify111111111111111111111111111");
const INSTRUCTIONS: Pubkey = Pubkey::from_str_const("Sysvar1nstructions1111111111111111111111111");

fn pk(n: u8) -> Pubkey {
	Pubkey::new_from_array([n; 32])
}

fn workspace() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_elf(path: &Path, how: &str) -> Vec<u8> {
	std::fs::read(path)
		.unwrap_or_else(|error| panic!("cannot read {}: {error}. {how}", path.display()))
}

/// The world a test plays in: Mollusk, and every account it has touched.
struct World {
	mollusk: Mollusk,
	accounts: HashMap<Pubkey, Account>,
	admin: Pubkey,
	keeper: SigningKey,
	next: u8,
}

impl World {
	fn new() -> Self {
		let mut mollusk = Mollusk::default();
		let program_id = Pubkey::new_from_array(program::ID.to_bytes());
		let elf = read_elf(
			&std::env::var_os("REAPERS_PROGRAM_SO").map_or_else(
				|| workspace().join("target/deploy/reapers_program.so"),
				PathBuf::from,
			),
			"Run `pina build` in crates/reapers_program first.",
		);
		mollusk.add_program_with_loader_and_elf(&program_id, &LOADER_V3, &elf);
		let core = read_elf(
			&workspace().join("target/fixtures/mpl_core.so"),
			"Run `cargo run -p reapers_ops -- fixtures`.",
		);
		mollusk.add_program_with_loader_and_elf(&CORE, &LOADER_V3, &core);
		mollusk_svm_programs_token::token::add_program(&mut mollusk);
		mollusk_svm_programs_token::token2022::add_program(&mut mollusk);
		mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

		let mut accounts = HashMap::new();
		accounts.insert(CORE, create_program_account_loader_v3(&CORE));
		accounts.insert(TOKEN, mollusk_svm_programs_token::token::account());
		accounts.insert(TOKEN_2022, mollusk_svm_programs_token::token2022::account());
		accounts.insert(ATA, mollusk_svm_programs_token::associated_token::account());
		accounts.insert(
			SYSTEM,
			mollusk_svm::program::keyed_account_for_system_program().1,
		);

		let mut world = Self {
			mollusk,
			accounts,
			admin: pk(200),
			keeper: SigningKey::from_bytes(&[42; 32]),
			next: 1,
		};
		world.fund(world.admin);
		world.program_data(world.admin);
		world
	}

	/// A new address, funded with 10 SOL.
	fn wallet(&mut self) -> Pubkey {
		let address = self.fresh();
		self.fund(address);
		address
	}

	fn fresh(&mut self) -> Pubkey {
		self.next += 1;
		Pubkey::new_from_array([self.next; 32])
	}

	fn fund(&mut self, address: Pubkey) {
		self.accounts.insert(
			address,
			Account::new_with_data(10_000_000_000, vec![], &SYSTEM),
		);
	}

	fn rent(&self, len: usize) -> u64 {
		self.mollusk.sysvars.rent.minimum_balance(len)
	}

	/// Writes this program's executable data account, naming `authority` as
	/// its upgrade authority.
	fn program_data(&mut self, authority: Pubkey) {
		let mut data = vec![3, 0, 0, 0];
		data.extend_from_slice(&0u64.to_le_bytes());
		data.push(1);
		data.extend_from_slice(authority.as_ref());
		let account = Account {
			lamports: self.rent(data.len()),
			data,
			owner: LOADER_V3,
			executable: false,
			rent_epoch: 0,
		};
		self.accounts.insert(program_data(), account);
	}

	/// Runs `instructions` as one transaction paid by the first signer, and
	/// keeps the result if it succeeds.
	fn send(&mut self, instructions: &[Instruction]) -> TransactionResult {
		let mut keys: Vec<Pubkey> = Vec::new();
		for instruction in instructions {
			for meta in &instruction.accounts {
				if !keys.contains(&meta.pubkey) && meta.pubkey != INSTRUCTIONS {
					keys.push(meta.pubkey);
				}
			}
		}
		let accounts: Vec<(Pubkey, Account)> = keys
			.iter()
			.map(|key| (*key, self.accounts.get(key).cloned().unwrap_or_default()))
			.collect();
		let payer = instructions
			.iter()
			.flat_map(|instruction| &instruction.accounts)
			.find(|meta| meta.is_signer)
			.map(|meta| meta.pubkey);
		let result =
			self.mollusk
				.process_transaction_instructions(instructions, &accounts, payer.as_ref());

		if result.program_result.is_ok() {
			for (key, account) in &result.resulting_accounts {
				self.accounts.insert(*key, account.clone());
			}
		}

		result
	}

	fn ok(&mut self, instructions: &[Instruction]) -> u64 {
		let result = self.send(instructions);
		assert!(
			result.program_result.is_ok(),
			"expected success, got {:?}",
			result.program_result
		);
		result.compute_units_consumed
	}

	fn account(&self, key: &Pubkey) -> Account {
		self.accounts.get(key).cloned().unwrap_or_default()
	}

	// -- The program ----------------------------------------------------------

	fn initialize(&mut self) {
		let ix = initialize(self.admin, self.admin, self.keeper_key());
		self.ok(&[ix]);
	}

	fn keeper_key(&self) -> Pubkey {
		Pubkey::new_from_array(self.keeper.verifying_key().to_bytes())
	}

	// -- Metaplex Core --------------------------------------------------------

	fn core_collection(&mut self) -> Pubkey {
		let collection = self.fresh();
		let mut data = vec![21];
		borsh_string(&mut data, "Thugbirdz (demo)");
		borsh_string(&mut data, "https://example.invalid/c.json");
		data.extend_from_slice(&[0, 0]);
		let ix = Instruction {
			program_id: CORE,
			accounts: vec![
				AccountMeta::new(collection, true),
				AccountMeta::new_readonly(CORE, false),
				AccountMeta::new(self.admin, true),
				AccountMeta::new_readonly(SYSTEM, false),
			],
			data,
		};
		self.ok(&[ix]);
		collection
	}

	/// A new Core asset owned by `owner`, in `collection` if there is one.
	fn core_asset(&mut self, owner: Pubkey, collection: Option<Pubkey>) -> Pubkey {
		let asset = self.fresh();
		let mut data = vec![20, 0];
		borsh_string(&mut data, "Husk #1");
		borsh_string(&mut data, "https://example.invalid/1.json");
		data.extend_from_slice(&[0, 0]);
		let ix = Instruction {
			program_id: CORE,
			accounts: vec![
				AccountMeta::new(asset, true),
				collection.map_or(AccountMeta::new_readonly(CORE, false), |c| {
					AccountMeta::new(c, false)
				}),
				AccountMeta::new_readonly(CORE, false),
				AccountMeta::new(self.admin, true),
				AccountMeta::new_readonly(owner, false),
				AccountMeta::new_readonly(CORE, false),
				AccountMeta::new_readonly(SYSTEM, false),
				AccountMeta::new_readonly(CORE, false),
			],
			data,
		};
		self.ok(&[ix]);
		asset
	}

	fn core_owner(&self, asset: &Pubkey) -> Pubkey {
		let data = self.account(asset).data;
		Pubkey::try_from(&data[1..33]).unwrap()
	}

	// -- SPL Token ------------------------------------------------------------

	/// A mint under `token_program` and a token account of `owner` holding
	/// `amount` of it. Token-2022 mints carry `extensions`, as (type, value).
	fn token_nft(
		&mut self,
		owner: Pubkey,
		token_program: Pubkey,
		supply: u64,
		decimals: u8,
		extensions: &[(u16, Vec<u8>)],
	) -> (Pubkey, Pubkey) {
		let mint = self.fresh();
		let mut data = vec![0; 82];
		data[0..4].copy_from_slice(&1u32.to_le_bytes());
		data[4..36].copy_from_slice(self.admin.as_ref());
		data[36..44].copy_from_slice(&supply.to_le_bytes());
		data[44] = decimals;
		data[45] = 1;
		if !extensions.is_empty() {
			data.resize(165, 0);
			data.push(1);
			for (kind, value) in extensions {
				data.extend_from_slice(&kind.to_le_bytes());
				data.extend_from_slice(&(value.len() as u16).to_le_bytes());
				data.extend_from_slice(value);
			}
		}
		let account = Account {
			lamports: self.rent(data.len()),
			data,
			owner: token_program,
			executable: false,
			rent_epoch: 0,
		};
		self.accounts.insert(mint, account);

		let holding = self.fresh();
		let data = token_account(mint, owner, supply);
		let account = Account {
			lamports: self.rent(data.len()),
			data,
			owner: token_program,
			executable: false,
			rent_epoch: 0,
		};
		self.accounts.insert(holding, account);

		(mint, holding)
	}
}

/// The Ed25519 instruction carrying `keeper`'s signature over `receipt`.
fn signed(keeper: &SigningKey, receipt: &Receipt) -> Instruction {
	let message = receipt.encode();
	let signature = keeper.sign(&message).to_bytes();
	ed25519(&ed25519_instruction_data(
		&keeper.verifying_key().to_bytes(),
		&signature,
		&message,
	))
}

/// A receipt the keeper might sign for `player` absorbing `mint`.
fn receipt(player: Pubkey, mint: Pubkey) -> Receipt {
	Receipt {
		player: player.to_bytes(),
		mint: mint.to_bytes(),
		grade: Grade::Clean,
		feeding: Feeding::Rich,
		xp: 63,
		anima: 20,
		at: 1_793_836_800,
		nonce: 7,
	}
}

fn borsh_string(data: &mut Vec<u8>, value: &str) {
	data.extend_from_slice(&(value.len() as u32).to_le_bytes());
	data.extend_from_slice(value.as_bytes());
}

fn token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> Vec<u8> {
	let mut data = vec![0; 165];
	data[0..32].copy_from_slice(mint.as_ref());
	data[32..64].copy_from_slice(owner.as_ref());
	data[64..72].copy_from_slice(&amount.to_le_bytes());
	data[108] = 1;
	data
}

fn token_amount(account: &Account) -> u64 {
	u64::from_le_bytes(account.data[64..72].try_into().unwrap())
}

fn program_id() -> Pubkey {
	Pubkey::new_from_array(program::ID.to_bytes())
}

fn program_data() -> Pubkey {
	Pubkey::find_program_address(&[program_id().as_ref()], &LOADER_V3).0
}

fn config() -> Pubkey {
	Pubkey::find_program_address(&[program::CONFIG_SEED], &program_id()).0
}

fn escrow(mint: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(&[program::ESCROW_SEED, mint.as_ref()], &program_id()).0
}

fn ata(wallet: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
		&ATA,
	)
	.0
}

fn ed25519(data: &[u8]) -> Instruction {
	Instruction {
		program_id: ED25519,
		accounts: vec![],
		data: data.to_vec(),
	}
}

// -- Instruction builders, from the program's own instruction types ----------

fn initialize(payer: Pubkey, authority: Pubkey, keeper: Pubkey) -> Instruction {
	let mut data = [0; program::InitializeInstruction::SIZE];
	program::InitializeInstruction::initialize(&mut data, |args| {
		args.authority = authority.to_bytes().into();
		args.keeper = keeper.to_bytes().into();
		Ok(())
	})
	.unwrap();
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new(payer, true),
			AccountMeta::new(config(), false),
			AccountMeta::new_readonly(program_data(), false),
			AccountMeta::new_readonly(SYSTEM, false),
		],
		data: data.to_vec(),
	}
}

fn set_keeper(authority: Pubkey, keeper: Pubkey) -> Instruction {
	let mut data = [0; program::SetKeeperInstruction::SIZE];
	program::SetKeeperInstruction::initialize(&mut data, |args| {
		args.keeper = keeper.to_bytes().into();
		Ok(())
	})
	.unwrap();
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new_readonly(authority, true),
			AccountMeta::new(config(), false),
		],
		data: data.to_vec(),
	}
}

fn set_authority(authority: Pubkey, new_authority: Pubkey, both_sign: bool) -> Instruction {
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new_readonly(authority, true),
			AccountMeta::new_readonly(new_authority, both_sign),
			AccountMeta::new(config(), false),
		],
		data: vec![program::ReapersInstruction::SetAuthority as u8],
	}
}

fn set_paused(authority: Pubkey, paused: bool) -> Instruction {
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new_readonly(authority, true),
			AccountMeta::new(config(), false),
		],
		data: vec![
			program::ReapersInstruction::SetPaused as u8,
			u8::from(paused),
		],
	}
}

fn collection_meta(collection: Option<Pubkey>) -> AccountMeta {
	collection.map_or(AccountMeta::new_readonly(CORE, false), |c| {
		AccountMeta::new(c, false)
	})
}

fn release_core(player: Pubkey, asset: Pubkey, collection: Option<Pubkey>) -> Instruction {
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new(player, true),
			AccountMeta::new_readonly(config(), false),
			AccountMeta::new(escrow(&asset), false),
			AccountMeta::new(asset, false),
			collection_meta(collection),
			AccountMeta::new_readonly(CORE, false),
			AccountMeta::new_readonly(SYSTEM, false),
		],
		data: vec![program::ReapersInstruction::ReleaseCore as u8],
	}
}

fn release_token(
	player: Pubkey,
	mint: Pubkey,
	holding: Pubkey,
	token_program: Pubkey,
) -> Instruction {
	let escrow = escrow(&mint);
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new(player, true),
			AccountMeta::new_readonly(config(), false),
			AccountMeta::new(escrow, false),
			AccountMeta::new_readonly(mint, false),
			AccountMeta::new(holding, false),
			AccountMeta::new(ata(&escrow, &mint, &token_program), false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(ATA, false),
			AccountMeta::new_readonly(SYSTEM, false),
		],
		data: vec![program::ReapersInstruction::ReleaseToken as u8],
	}
}

fn absorb_core(player: Pubkey, asset: Pubkey, collection: Option<Pubkey>) -> Instruction {
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new(player, true),
			AccountMeta::new_readonly(config(), false),
			AccountMeta::new(escrow(&asset), false),
			AccountMeta::new(asset, false),
			collection_meta(collection),
			AccountMeta::new_readonly(CORE, false),
			AccountMeta::new_readonly(INSTRUCTIONS, false),
		],
		data: vec![program::ReapersInstruction::AbsorbCore as u8],
	}
}

fn absorb_token(player: Pubkey, mint: Pubkey, token_program: Pubkey) -> Instruction {
	let escrow = escrow(&mint);
	Instruction {
		program_id: program_id(),
		accounts: vec![
			AccountMeta::new(player, true),
			AccountMeta::new_readonly(config(), false),
			AccountMeta::new(escrow, false),
			AccountMeta::new(mint, false),
			AccountMeta::new(ata(&escrow, &mint, &token_program), false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(INSTRUCTIONS, false),
		],
		data: vec![program::ReapersInstruction::AbsorbToken as u8],
	}
}

// -- Assertions --------------------------------------------------------------

fn custom(error: ReapersError) -> u32 {
	match pina::ProgramError::from(error) {
		pina::ProgramError::Custom(code) => code,
		other => panic!("{error:?} is not a custom error: {other:?}"),
	}
}

#[track_caller]
fn assert_fails_with(result: &TransactionResult, error: ReapersError) {
	let expected = custom(error);
	match &result.program_result {
		TransactionProgramResult::Failure(_, pina::ProgramError::Custom(code))
		| TransactionProgramResult::UnknownError(_, InstructionError::Custom(code))
			if code == &expected => {}
		other => panic!("expected {error:?} ({expected}), got {other:?}"),
	}
}

#[track_caller]
fn assert_fails(result: &TransactionResult) {
	assert!(
		!result.program_result.is_ok(),
		"expected a failure, got success"
	);
}

/// A world with the program initialized, a player, and a Core husk they own
/// in a collection.
fn core_world() -> (World, Pubkey, Pubkey, Option<Pubkey>) {
	let mut world = World::new();
	world.initialize();
	let collection = Some(world.core_collection());
	let player = world.wallet();
	let asset = world.core_asset(player, collection);
	(world, player, asset, collection)
}

// ===========================================================================
// Initialize and administration
// ===========================================================================

#[test]
fn only_the_upgrade_authority_initializes() {
	let mut world = World::new();
	let stranger = world.wallet();
	let keeper = world.keeper_key();

	let result = world.send(&[initialize(stranger, stranger, keeper)]);
	assert_fails_with(&result, ReapersError::NotUpgradeAuthority);

	world.ok(&[initialize(world.admin, world.admin, keeper)]);
	let data = world.account(&config()).data;
	assert_eq!(&data[2..34], world.admin.as_ref());
	assert_eq!(&data[34..66], keeper.as_ref());
	assert_eq!(data[66], 0, "not paused");

	let result = world.send(&[initialize(world.admin, world.admin, keeper)]);
	assert_fails(&result);
}

#[test]
fn initialize_refuses_a_program_data_account_that_is_not_ours() {
	let mut world = World::new();
	let keeper = world.keeper_key();
	let mut ix = initialize(world.admin, world.admin, keeper);
	let fake = world.fresh();
	world.accounts.insert(fake, world.account(&program_data()));
	ix.accounts[2] = AccountMeta::new_readonly(fake, false);

	assert_fails(&world.send(&[ix]));
}

#[test]
fn only_the_authority_administers() {
	let mut world = World::new();
	world.initialize();
	let stranger = world.wallet();
	let other_keeper = pk(77);

	assert_fails_with(
		&world.send(&[set_keeper(stranger, other_keeper)]),
		ReapersError::NotAuthority,
	);
	assert_fails_with(
		&world.send(&[set_paused(stranger, true)]),
		ReapersError::NotAuthority,
	);
	assert_fails_with(
		&world.send(&[set_authority(stranger, stranger, true)]),
		ReapersError::NotAuthority,
	);
}

#[test]
fn handing_on_the_authority_needs_both_signatures() {
	let mut world = World::new();
	world.initialize();
	let successor = world.wallet();

	let result = world.send(&[set_authority(world.admin, successor, false)]);
	assert_fails(&result);

	world.ok(&[set_authority(world.admin, successor, true)]);
	assert_eq!(&world.account(&config()).data[2..34], successor.as_ref());

	assert_fails_with(
		&world.send(&[set_paused(world.admin, true)]),
		ReapersError::NotAuthority,
	);
	world.ok(&[set_paused(successor, true)]);
}

// ===========================================================================
// Release
// ===========================================================================

#[test]
fn a_core_husk_is_released_into_escrow() {
	let (mut world, player, asset, collection) = core_world();

	world.ok(&[release_core(player, asset, collection)]);

	assert_eq!(world.core_owner(&asset), escrow(&asset));
	let data = world.account(&escrow(&asset)).data;
	assert_eq!(data[0], program::ReapersAccount::Escrow as u8);
	assert_eq!(&data[2..34], player.as_ref());
	assert_eq!(&data[34..66], asset.as_ref());
	assert_eq!(data[66], program::HuskKind::Core as u8);
}

#[test]
fn a_core_husk_outside_any_collection_is_released() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let asset = world.core_asset(player, None);

	world.ok(&[release_core(player, asset, None)]);

	assert_eq!(world.core_owner(&asset), escrow(&asset));
}

#[test]
fn a_core_husk_the_player_does_not_own_is_refused() {
	let (mut world, _, asset, collection) = core_world();
	let thief = world.wallet();

	assert_fails_with(
		&world.send(&[release_core(thief, asset, collection)]),
		ReapersError::NotOwner,
	);
}

#[test]
fn a_released_husk_cannot_be_released_again() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);

	assert_fails(&world.send(&[release_core(player, asset, collection)]));
}

#[test]
fn an_account_that_is_not_a_core_asset_is_refused() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let impostor = world.fresh();
	let mut data = vec![1];
	data.extend_from_slice(player.as_ref());
	let account = Account::new_with_data(world.rent(33), data, &program_id());
	world.accounts.insert(impostor, account);

	assert_fails_with(
		&world.send(&[release_core(player, impostor, None)]),
		ReapersError::NotAnNft,
	);
}

#[test]
fn a_classic_nft_is_released_into_escrow() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let (mint, holding) = world.token_nft(player, TOKEN, 1, 0, &[]);

	world.ok(&[release_token(player, mint, holding, TOKEN)]);

	let vault = world.account(&ata(&escrow(&mint), &mint, &TOKEN));
	assert_eq!(token_amount(&vault), 1);
	assert_eq!(&vault.data[32..64], escrow(&mint).as_ref());
	assert_eq!(token_amount(&world.account(&holding)), 0);
	assert_eq!(
		world.account(&escrow(&mint)).data[66],
		program::HuskKind::Token as u8
	);
}

#[test]
fn a_fungible_token_is_refused() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();

	let (mint, holding) = world.token_nft(player, TOKEN, 1_000, 6, &[]);
	assert_fails_with(
		&world.send(&[release_token(player, mint, holding, TOKEN)]),
		ReapersError::NotAnNft,
	);

	// One token, but divisible: still not an NFT.
	let (mint, holding) = world.token_nft(player, TOKEN, 1, 2, &[]);
	assert_fails_with(
		&world.send(&[release_token(player, mint, holding, TOKEN)]),
		ReapersError::NotAnNft,
	);
}

#[test]
fn a_token_account_the_player_only_holds_a_delegation_on_is_refused() {
	let mut world = World::new();
	world.initialize();
	let owner = world.wallet();
	let delegate = world.wallet();
	let (mint, holding) = world.token_nft(owner, TOKEN, 1, 0, &[]);
	let account = world.accounts.get_mut(&holding).unwrap();
	account.data[72..76].copy_from_slice(&1u32.to_le_bytes());
	account.data[76..108].copy_from_slice(delegate.as_ref());
	account.data[121..129].copy_from_slice(&1u64.to_le_bytes());

	assert_fails_with(
		&world.send(&[release_token(delegate, mint, holding, TOKEN)]),
		ReapersError::NotOwner,
	);
}

/// Token-2022 extension type numbers.
const METADATA_POINTER: u16 = 18;
const PERMANENT_DELEGATE: u16 = 12;

#[test]
fn a_token_2022_nft_with_a_metadata_pointer_is_released() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let (mint, holding) =
		world.token_nft(player, TOKEN_2022, 1, 0, &[(METADATA_POINTER, vec![0; 64])]);

	world.ok(&[release_token(player, mint, holding, TOKEN_2022)]);

	let vault = world.account(&ata(&escrow(&mint), &mint, &TOKEN_2022));
	assert_eq!(token_amount(&vault), 1);
	assert_eq!(
		world.account(&escrow(&mint)).data[66],
		program::HuskKind::Token2022 as u8
	);
}

#[test]
fn a_token_2022_mint_with_a_permanent_delegate_is_refused() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let (mint, holding) = world.token_nft(
		player,
		TOKEN_2022,
		1,
		0,
		&[(PERMANENT_DELEGATE, pk(9).to_bytes().to_vec())],
	);

	assert_fails_with(
		&world.send(&[release_token(player, mint, holding, TOKEN_2022)]),
		ReapersError::UnsupportedExtension,
	);
}

#[test]
fn a_mint_under_the_wrong_token_program_is_refused() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let (mint, holding) = world.token_nft(player, TOKEN, 1, 0, &[]);

	assert_fails(&world.send(&[release_token(player, mint, holding, TOKEN_2022)]));
}

// ===========================================================================
// Absorb
// ===========================================================================

#[test]
fn a_core_husk_is_burned_by_a_receipt() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let before = world.account(&player).lamports;
	let escrow_rent = world.account(&escrow(&asset)).lamports;

	let husk_receipt = receipt(player, asset);
	let units = world.ok(&[
		signed(&world.keeper, &husk_receipt),
		absorb_core(player, asset, collection),
	]);
	println!("absorb core (with Ed25519): {units} CU");

	let burned = world.account(&asset);
	println!(
		"burned asset: {} bytes, {} lamports, owner {}; player gained {}; escrow rent {}",
		burned.data.len(),
		burned.lamports,
		burned.owner,
		world.account(&player).lamports - before,
		escrow_rent
	);
	// Core leaves a one-byte tombstone, `Key::Uninitialized`, so the address
	// can never hold an asset again.
	assert_eq!(burned.data, [0], "the asset is gone");
	assert_eq!(world.account(&escrow(&asset)).lamports, 0);
	assert!(world.account(&player).lamports >= before + escrow_rent);
}

#[test]
fn a_core_husk_outside_any_collection_is_burned() {
	let mut world = World::new();
	world.initialize();
	let player = world.wallet();
	let asset = world.core_asset(player, None);
	world.ok(&[release_core(player, asset, None)]);

	let husk_receipt = receipt(player, asset);
	world.ok(&[
		signed(&world.keeper, &husk_receipt),
		absorb_core(player, asset, None),
	]);

	assert_eq!(world.account(&asset).data, [0]);
}

#[test]
fn a_classic_nft_is_burned_by_a_receipt() {
	for token_program in [TOKEN, TOKEN_2022] {
		let mut world = World::new();
		world.initialize();
		let player = world.wallet();
		let (mint, holding) = world.token_nft(player, token_program, 1, 0, &[]);
		world.ok(&[release_token(player, mint, holding, token_program)]);
		let vault = ata(&escrow(&mint), &mint, &token_program);

		let husk_receipt = receipt(player, mint);
		let units = world.ok(&[
			signed(&world.keeper, &husk_receipt),
			absorb_token(player, mint, token_program),
		]);
		println!("absorb token {token_program} (with Ed25519): {units} CU");

		let supply = u64::from_le_bytes(world.account(&mint).data[36..44].try_into().unwrap());
		assert_eq!(supply, 0, "the token is burned");
		assert_eq!(world.account(&vault).lamports, 0, "the vault is closed");
		assert_eq!(world.account(&escrow(&mint)).lamports, 0);
	}
}

#[test]
fn a_receipt_cannot_be_used_twice() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset);
	let ed = signed(&world.keeper, &husk_receipt);
	world.ok(&[ed.clone(), absorb_core(player, asset, collection)]);

	assert_fails(&world.send(&[ed.clone(), absorb_core(player, asset, collection)]));

	// Nor within one transaction: the second `Absorb` is not preceded by the
	// Ed25519 instruction, and the escrow is gone anyway.
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let ed = signed(&world.keeper, &receipt(player, asset));
	assert_fails(&world.send(&[
		ed.clone(),
		absorb_core(player, asset, collection),
		absorb_core(player, asset, collection),
	]));
	assert_fails(&world.send(&[
		ed.clone(),
		absorb_core(player, asset, collection),
		ed,
		absorb_core(player, asset, collection),
	]));
}

#[test]
fn a_burned_husk_cannot_be_released_again() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset);
	world.ok(&[
		signed(&world.keeper, &husk_receipt),
		absorb_core(player, asset, collection),
	]);

	assert_fails(&world.send(&[release_core(player, asset, collection)]));
}

#[test]
fn a_receipt_from_another_keeper_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let impostor = SigningKey::from_bytes(&[13; 32]);
	let husk_receipt = receipt(player, asset);

	assert_fails_with(
		&world.send(&[
			signed(&impostor, &husk_receipt),
			absorb_core(player, asset, collection),
		]),
		ReapersError::WrongKeeper,
	);
}

#[test]
fn a_rotated_keeper_retires_the_old_one() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let old = world.keeper.clone();
	let new = SigningKey::from_bytes(&[43; 32]);
	world.ok(&[set_keeper(
		world.admin,
		Pubkey::new_from_array(new.verifying_key().to_bytes()),
	)]);
	let husk_receipt = receipt(player, asset);

	assert_fails_with(
		&world.send(&[
			signed(&old, &husk_receipt),
			absorb_core(player, asset, collection),
		]),
		ReapersError::WrongKeeper,
	);
	world.ok(&[
		signed(&new, &husk_receipt),
		absorb_core(player, asset, collection),
	]);
}

#[test]
fn a_tampered_receipt_fails_the_signature() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset);
	let mut ed = signed(&world.keeper, &husk_receipt);
	// Change the feeding after signing.
	ed.data[reapers_receipt::ed25519::MESSAGE + reapers_receipt::offset::FEEDING] = 3;

	let result = world.send(&[ed, absorb_core(player, asset, collection)]);
	assert_fails(&result);
	assert!(
		matches!(
			result.program_result,
			TransactionProgramResult::UnknownError(0, _)
				| TransactionProgramResult::Failure(0, _)
				| TransactionProgramResult::TransactionError(_)
		),
		"the Ed25519 instruction fails: {:?}",
		result.program_result
	);
}

#[test]
fn a_receipt_for_another_player_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let someone = world.wallet();
	let husk_receipt = receipt(someone, asset);

	assert_fails_with(
		&world.send(&[
			signed(&world.keeper, &husk_receipt),
			absorb_core(player, asset, collection),
		]),
		ReapersError::WrongPlayer,
	);
}

#[test]
fn another_player_cannot_absorb_a_husk_they_did_not_release() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let someone = world.wallet();
	// Even holding a receipt that names them.
	let husk_receipt = receipt(someone, asset);

	assert_fails_with(
		&world.send(&[
			signed(&world.keeper, &husk_receipt),
			absorb_core(someone, asset, collection),
		]),
		ReapersError::WrongPlayer,
	);
}

#[test]
fn a_receipt_for_another_husk_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	let other = world.core_asset(player, collection);
	world.ok(&[release_core(player, asset, collection)]);
	world.ok(&[release_core(player, other, collection)]);
	let husk_receipt = receipt(player, other);

	assert_fails_with(
		&world.send(&[
			signed(&world.keeper, &husk_receipt),
			absorb_core(player, asset, collection),
		]),
		ReapersError::WrongHusk,
	);
}

#[test]
fn absorbing_needs_the_ed25519_instruction_right_before() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let ed = signed(&world.keeper, &receipt(player, asset));

	// Nothing before it.
	assert_fails_with(
		&world.send(&[absorb_core(player, asset, collection)]),
		ReapersError::MissingReceipt,
	);
	// Something else before it.
	assert_fails_with(
		&world.send(&[
			ed.clone(),
			set_paused(world.admin, false),
			absorb_core(player, asset, collection),
		]),
		ReapersError::MissingReceipt,
	);
	// After it.
	assert_fails_with(
		&world.send(&[absorb_core(player, asset, collection), ed]),
		ReapersError::MissingReceipt,
	);
}

/// The known attack on Ed25519 introspection: the instruction before
/// `Absorb` points the precompile at a genuine signature elsewhere in the
/// transaction, while its own bytes, which a careless program reads, hold a
/// forged receipt.
#[test]
fn offsets_into_another_instruction_are_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let attacker = world.wallet();
	let decoy = world.core_asset(attacker, collection);
	world.ok(&[release_core(attacker, decoy, collection)]);

	// Instruction 0 is a genuine, self-contained receipt for the attacker's
	// own husk. Instruction 1 verifies those same bytes in instruction 0,
	// but carries a forged receipt for the victim's husk at the same offsets.
	let genuine = signed(&world.keeper, &receipt(attacker, decoy));
	let mut forged = genuine.data.clone();
	let husk_receipt = receipt(attacker, asset).encode();
	forged[reapers_receipt::ed25519::MESSAGE..].copy_from_slice(&husk_receipt);
	for at in [4, 8, 14] {
		forged[at..at + 2].copy_from_slice(&0u16.to_le_bytes());
	}

	// The victim's escrow names the victim, so make the attacker's attempt
	// as strong as it can be: absorb their own escrow with a receipt forged
	// for a better feeding.
	let mut better = receipt(attacker, decoy);
	better.feeding = Feeding::Feast;
	better.xp = 10_000;
	let mut forged_own = genuine.data.clone();
	forged_own[reapers_receipt::ed25519::MESSAGE..].copy_from_slice(&better.encode());
	for at in [4, 8, 14] {
		forged_own[at..at + 2].copy_from_slice(&0u16.to_le_bytes());
	}

	assert_fails_with(
		&world.send(&[
			genuine.clone(),
			ed25519(&forged_own),
			absorb_core(attacker, decoy, collection),
		]),
		ReapersError::UnsupportedReceiptInstruction,
	);
	assert_fails_with(
		&world.send(&[
			genuine,
			ed25519(&forged),
			absorb_core(attacker, asset, collection),
		]),
		ReapersError::WrongPlayer,
	);
}

#[test]
fn an_ed25519_instruction_with_two_signatures_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset).encode();
	let keeper = world.keeper.clone();
	let signature = keeper.sign(&husk_receipt).to_bytes();
	let key = keeper.verifying_key().to_bytes();

	// Two signatures, both genuine, both over the same receipt.
	let header = 2 + 14 * 2;
	let (key_at, sig_at, msg_at) = (header, header + 32, header + 96);
	let mut data = vec![2, 0];
	for _ in 0..2 {
		for value in [
			sig_at as u16,
			u16::MAX,
			key_at as u16,
			u16::MAX,
			msg_at as u16,
			98,
			u16::MAX,
		] {
			data.extend_from_slice(&value.to_le_bytes());
		}
	}
	data.extend_from_slice(&key);
	data.extend_from_slice(&signature);
	data.extend_from_slice(&husk_receipt);

	assert_fails_with(
		&world.send(&[ed25519(&data), absorb_core(player, asset, collection)]),
		ReapersError::UnsupportedReceiptInstruction,
	);
}

#[test]
fn a_version_1_receipt_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let mut message = receipt(player, asset).encode();
	message[..8].copy_from_slice(b"RPRSABS1");
	let keeper = world.keeper.clone();
	let data = ed25519_instruction_data(
		&keeper.verifying_key().to_bytes(),
		&keeper.sign(&message).to_bytes(),
		&message,
	);

	assert_fails_with(
		&world.send(&[ed25519(&data), absorb_core(player, asset, collection)]),
		ReapersError::MalformedReceipt,
	);
}

#[test]
fn absorbing_the_wrong_kind_is_refused() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset);

	assert_fails_with(
		&world.send(&[
			signed(&world.keeper, &husk_receipt),
			absorb_token(player, asset, TOKEN),
		]),
		ReapersError::WrongKind,
	);
}

#[test]
fn pausing_stops_release_and_absorb() {
	let (mut world, player, asset, collection) = core_world();
	let other = world.core_asset(player, collection);
	world.ok(&[release_core(player, asset, collection)]);
	world.ok(&[set_paused(world.admin, true)]);

	assert_fails_with(
		&world.send(&[release_core(player, other, collection)]),
		ReapersError::Paused,
	);
	let husk_receipt = receipt(player, asset);
	let ed = signed(&world.keeper, &husk_receipt);
	assert_fails_with(
		&world.send(&[ed.clone(), absorb_core(player, asset, collection)]),
		ReapersError::Paused,
	);

	world.ok(&[set_paused(world.admin, false)]);
	world.ok(&[ed, absorb_core(player, asset, collection)]);
}

/// The generated client's absorb helpers build what the program expects.
#[test]
fn the_client_absorbs_both_kinds() {
	let (mut world, player, asset, collection) = core_world();
	world.ok(&[release_core(player, asset, collection)]);
	let husk_receipt = receipt(player, asset);
	let signature = world.keeper.sign(&husk_receipt.encode()).to_bytes();
	world.ok(&reapers_program_client::absorb::absorb_core(
		&world.keeper_key(),
		&signature,
		&husk_receipt,
		collection,
	));
	assert_eq!(world.account(&asset).data, [0]);

	for token_program in [TOKEN, TOKEN_2022] {
		let (mint, holding) = world.token_nft(player, token_program, 1, 0, &[]);
		world.ok(&[release_token(player, mint, holding, token_program)]);
		assert_eq!(
			reapers_program_client::absorb::vault(&mint, &token_program),
			ata(&escrow(&mint), &mint, &token_program)
		);
		let husk_receipt = receipt(player, mint);
		let signature = world.keeper.sign(&husk_receipt.encode()).to_bytes();
		world.ok(&reapers_program_client::absorb::absorb_token(
			&world.keeper_key(),
			&signature,
			&husk_receipt,
			&token_program,
		));
		assert_eq!(world.account(&escrow(&mint)).lamports, 0);
	}
}

/// Compute units of every instruction on its success path, for
/// `docs/program.md`. Run with `--nocapture`.
#[test]
fn compute_units() {
	let mut world = World::new();
	let units = world.ok(&[initialize(world.admin, world.admin, world.keeper_key())]);
	println!("initialize: {units} CU");
	let units = world.ok(&[set_keeper(world.admin, world.keeper_key())]);
	println!("set_keeper: {units} CU");
	let units = world.ok(&[set_paused(world.admin, false)]);
	println!("set_paused: {units} CU");
	let successor = world.wallet();
	let units = world.ok(&[set_authority(world.admin, successor, true)]);
	println!("set_authority: {units} CU");
	world.ok(&[set_authority(successor, world.admin, true)]);

	let collection = Some(world.core_collection());
	let player = world.wallet();
	let asset = world.core_asset(player, collection);
	let units = world.ok(&[release_core(player, asset, collection)]);
	println!("release_core: {units} CU");
	let husk_receipt = receipt(player, asset);
	let ed = signed(&world.keeper, &husk_receipt);
	let units = world.ok(&[ed, absorb_core(player, asset, collection)]);
	println!("absorb_core: {units} CU");

	for token_program in [TOKEN, TOKEN_2022] {
		let (mint, holding) = world.token_nft(player, token_program, 1, 0, &[]);
		let units = world.ok(&[release_token(player, mint, holding, token_program)]);
		println!("release_token ({token_program}): {units} CU");
		let husk_receipt = receipt(player, mint);
		let ed = signed(&world.keeper, &husk_receipt);
		let units = world.ok(&[ed, absorb_token(player, mint, token_program)]);
		println!("absorb_token ({token_program}): {units} CU");
	}
}
