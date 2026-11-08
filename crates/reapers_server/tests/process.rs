//! The keeper as a process, and its migrations.

use std::path::Path;
use std::process::Command;

#[test]
fn the_migrations_leave_exactly_the_tables_in_the_code() {
	let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../reapers_db/migrations");

	if let Err(error) = reapers_server::tables::migrations::check(&directory) {
		panic!("{error}");
	}
}

#[test]
fn a_deployed_keeper_refuses_to_start_without_its_secrets() {
	for mode in ["staging", "production"] {
		let output = Command::new(env!("CARGO_BIN_EXE_reapers_server"))
			.env_clear()
			.env("REAPERS_RUN_MODE", mode)
			.output()
			.expect("the keeper runs");
		let stderr = String::from_utf8_lossy(&output.stderr);

		assert_eq!(output.status.code(), Some(78), "{stderr}");
		assert!(
			stderr.contains("database, authKeys, receiptSigningKey"),
			"names every missing secret: {stderr}"
		);
		assert!(stderr.contains("REAPERS_SECRET_"), "{stderr}");
	}
}

#[test]
fn an_unknown_run_mode_is_refused() {
	let output = Command::new(env!("CARGO_BIN_EXE_reapers_server"))
		.env_clear()
		.env("REAPERS_RUN_MODE", "prod")
		.output()
		.expect("the keeper runs");

	assert!(!output.status.success());
	assert!(String::from_utf8_lossy(&output.stderr).contains("REAPERS_RUN_MODE"));
}
