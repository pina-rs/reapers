//! The deployed program on an isolated, offline Surfnet, through `pina test`.
//!
//! Offline Surfpool has no Metaplex Core, so this tier covers what needs only
//! the program itself: bootstrapping and administration, as the deployed
//! binary runs them. Releasing and absorbing run against the real Core, Token
//! and Token-2022 programs in `tests/program.rs` (Mollusk), and end to end on
//! Surfpool in `cargo run -p reapers_ops -- rehearse`.

#![cfg(test)]

use pina_test::AccountMeta;
use pina_test::Keypair;
use pina_test::ProgramTest;
use pina_test::Pubkey;
use pina_test::Signer;

#[path = "../../../src/lib.rs"]
#[allow(dead_code, unused_imports, unused_attributes)]
mod program;

// The program's modules name its root as `crate::`.
#[allow(unused_imports)]
use program::*;

const SYSTEM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const LOADER_V3: Pubkey = Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

fn program_id() -> Pubkey {
	Pubkey::new_from_array(program::ID.to_bytes())
}

fn config() -> Pubkey {
	Pubkey::find_program_address(&[program::CONFIG_SEED], &program_id()).0
}

fn initialize_data(authority: Pubkey, keeper: Pubkey) -> Vec<u8> {
	let mut data = [0u8; program::InitializeInstruction::SIZE];
	program::InitializeInstruction::initialize(&mut data, |args| {
		args.authority = authority.to_bytes().into();
		args.keeper = keeper.to_bytes().into();
		Ok(())
	})
	.expect("encode Initialize");
	data.to_vec()
}

fn initialize_accounts(payer: Pubkey) -> Vec<AccountMeta> {
	let program_data = Pubkey::find_program_address(&[program_id().as_ref()], &LOADER_V3).0;
	vec![
		AccountMeta::new(payer, true),
		AccountMeta::new(config(), false),
		AccountMeta::new_readonly(program_data, false),
		AccountMeta::new_readonly(SYSTEM, false),
	]
}

/// Initializing takes the upgrade authority, and administration takes the
/// configured authority.
#[test]
#[ignore = "run with `pina test`"]
fn bootstraps_and_administers() {
	pina_test::run(async {
		let mut program = ProgramTest::start(program_id())
			.await
			.expect("start isolated program test");
		let payer = program.payer();
		let keeper = Pubkey::new_from_array([7; 32]);

		// Before the payer is the upgrade authority, it cannot initialize.
		program
			.set_upgrade_authority(Some(Pubkey::new_from_array([9; 32])))
			.expect("set another upgrade authority");
		let error = program
			.send(&initialize_data(payer, keeper), initialize_accounts(payer))
			.expect_err("a stranger cannot initialize");
		pina_test::assert_custom_error(
			&error,
			u32::from(program::ReapersError::NotUpgradeAuthority as u8),
		);

		program
			.set_upgrade_authority(Some(payer))
			.expect("make the payer the upgrade authority");
		program
			.send(&initialize_data(payer, keeper), initialize_accounts(payer))
			.expect("initialize");
		let config_account = program.account(&config()).expect("read config");
		assert_eq!(&config_account.data[2..34], payer.as_ref());
		assert_eq!(&config_account.data[34..66], keeper.as_ref());

		// A stranger cannot pause.
		let stranger = Keypair::new_from_array([11; 32]);
		program
			.fund(&stranger.pubkey(), 1_000_000_000)
			.expect("fund stranger");
		let pause = |authority: Pubkey| {
			program.instruction(
				&[program::ReapersInstruction::SetPaused as u8, 1],
				vec![
					AccountMeta::new_readonly(authority, true),
					AccountMeta::new(config(), false),
				],
			)
		};
		let error = program
			.send_with_signers(pause(stranger.pubkey()), &[&stranger])
			.expect_err("a stranger cannot pause");
		pina_test::assert_custom_error(
			&error,
			u32::from(program::ReapersError::NotAuthority as u8),
		);

		// The authority can.
		program
			.send_instruction(pause(payer))
			.expect("the authority pauses");
		assert_eq!(program.account(&config()).expect("read config").data[66], 1);

		// And can rotate the keeper.
		let mut data = [0u8; program::SetKeeperInstruction::SIZE];
		program::SetKeeperInstruction::initialize(&mut data, |args| {
			args.keeper = [8; 32].into();
			Ok(())
		})
		.expect("encode SetKeeper");
		program
			.send(
				&data,
				vec![
					AccountMeta::new_readonly(payer, true),
					AccountMeta::new(config(), false),
				],
			)
			.expect("rotate the keeper");
		assert_eq!(
			&program.account(&config()).expect("read config").data[34..66],
			&[8; 32]
		);

		program.stop().expect("stop isolated program test");
	});
}
