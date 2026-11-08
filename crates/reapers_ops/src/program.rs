//! The Reapers program's instructions, built by its generated client, and its
//! configuration as the cluster holds it.

use reapers_program_client::REAPERS_PROGRAM_ID;
use reapers_program_client::accounts::Config;
use reapers_program_client::instructions::Initialize;
use reapers_program_client::instructions::InitializeInstructionData;
use reapers_program_client::instructions::ReleaseCore;
use reapers_program_client::instructions::ReleaseCoreInstructionData;
use reapers_program_client::instructions::SetAuthority;
use reapers_program_client::instructions::SetAuthorityInstructionData;
use reapers_program_client::instructions::SetKeeper;
use reapers_program_client::instructions::SetKeeperInstructionData;
use reapers_program_client::instructions::SetPaused;
use reapers_program_client::instructions::SetPausedInstructionData;
use solana_kit_core::Address;
use solana_kit_core::Instruction;
use solana_pubkey::Pubkey;

use crate::chain::Chain;
use crate::chain::Error;
use crate::chain::address;
use crate::chain::kit;
use crate::chain::pubkey;

/// The BPF upgradeable loader.
const LOADER_V3: Pubkey = Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

/// Compute unit limits requested for each instruction: what the Mollusk suite
/// measured (`docs/program.md`), with room to spare.
pub mod units {
	pub const INITIALIZE: u32 = 20_000;
	pub const ADMIN: u32 = 5_000;
	pub const RELEASE_CORE: u32 = 40_000;
	pub const ABSORB_CORE: u32 = 40_000;
	pub const CORE_MINT: u32 = 60_000;
}

pub fn program_id() -> Address {
	address(&REAPERS_PROGRAM_ID)
}

/// The program's executable data account, which names its upgrade authority.
pub fn program_data() -> Pubkey {
	Pubkey::find_program_address(&[REAPERS_PROGRAM_ID.as_ref()], &LOADER_V3).0
}

pub fn config() -> Address {
	address(&Config::find_pda().0)
}

pub fn initialize(
	upgrade_authority: &Address,
	authority: &Address,
	keeper: &Address,
) -> Instruction {
	let authority = pubkey(authority);
	let keeper = pubkey(keeper);
	let data = InitializeInstructionData::new(|args| {
		args.authority = authority;
		args.keeper = keeper;
	})
	.unwrap_or_else(|error| panic!("Initialize encodes: {error}"));

	kit(Initialize::new(pubkey(upgrade_authority), program_data()).instruction(data))
}

pub fn set_keeper(authority: &Address, keeper: &Address) -> Instruction {
	let keeper = pubkey(keeper);
	let data = SetKeeperInstructionData::new(|args| args.keeper = keeper)
		.unwrap_or_else(|error| panic!("SetKeeper encodes: {error}"));

	kit(SetKeeper::new(pubkey(authority)).instruction(data))
}

pub fn set_paused(authority: &Address, paused: bool) -> Instruction {
	let data = SetPausedInstructionData::new(|args| args.paused = paused.into())
		.unwrap_or_else(|error| panic!("SetPaused encodes: {error}"));

	kit(SetPaused::new(pubkey(authority)).instruction(data))
}

pub fn set_authority(authority: &Address, new_authority: &Address) -> Instruction {
	let data = SetAuthorityInstructionData::new(|_| {})
		.unwrap_or_else(|error| panic!("SetAuthority encodes: {error}"));

	kit(SetAuthority::new(pubkey(authority), pubkey(new_authority)).instruction(data))
}

pub fn release_core(
	player: &Address,
	asset: &Address,
	collection: Option<&Address>,
) -> Instruction {
	let asset = pubkey(asset);
	let collection = collection.map_or(reapers_program_client::absorb::CORE_PROGRAM_ID, pubkey);
	let data = ReleaseCoreInstructionData::new(|_| {})
		.unwrap_or_else(|error| panic!("ReleaseCore encodes: {error}"));
	let mut instruction = ReleaseCore::new(
		pubkey(player),
		reapers_program_client::absorb::escrow(&asset),
		asset,
		collection,
		reapers_program_client::absorb::CORE_PROGRAM_ID,
	)
	.instruction(data);
	// The program reads the collection; Core writes it only on burn.
	instruction.accounts[4].is_writable = false;

	kit(instruction)
}

/// The configuration as the cluster holds it.
pub struct Configuration {
	pub authority: Address,
	pub keeper: Address,
	pub paused: bool,
}

pub async fn configuration(chain: &Chain) -> Result<Option<Configuration>, Error> {
	let Some(account) = chain.account(config()).await? else {
		return Ok(None);
	};

	if account.owner != program_id() {
		return Err(format!(
			"{} is owned by {}, not the program",
			config(),
			account.owner
		)
		.into());
	}

	let config = Config::from_bytes(&account.data)
		.map_err(|error| format!("the config account does not decode: {error}"))?;

	Ok(Some(Configuration {
		authority: address(&config.authority),
		keeper: address(&config.keeper),
		paused: bool::from(config.paused),
	}))
}
