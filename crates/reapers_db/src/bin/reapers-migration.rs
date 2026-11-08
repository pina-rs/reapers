//! Writes the next migration: `cargo run -p reapers_db --bin
//! reapers-migration -- <name>`. `--check` instead fails when a table
//! changed without one.
//!
//! Run it on its own, as above: built together with the keeper, `serde_json`
//! reads numbers with arbitrary precision and the latest snapshot cannot be
//! read back to diff against.

use std::path::PathBuf;
use std::process::ExitCode;

use reapers_db::migrations;

fn main() -> ExitCode {
	let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
	let argument = std::env::args().nth(1).unwrap_or_default();
	let result = if argument == "--check" {
		migrations::check(&directory).map(|()| "the migrations are up to date".to_owned())
	} else {
		let now = chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string();

		migrations::create(&directory, &argument, &now).map(|name| {
			format!("wrote migrations/{name}; review it, then commit it with registry.txt")
		})
	};

	match result {
		Ok(message) => {
			println!("{message}");
			ExitCode::SUCCESS
		}
		Err(error) => {
			eprintln!("{error}");
			ExitCode::FAILURE
		}
	}
}
