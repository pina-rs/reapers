//! Running other programs, and failing with a message and an exit code.

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Why a tool stopped: what to tell the person, and the exit code.
#[derive(Debug)]
pub struct Failure {
	pub message: String,
	pub code: u8,
}

impl Failure {
	/// A failure that exits with 1.
	pub fn new(message: impl Into<String>) -> Self {
		Self::with_code(message, 1)
	}

	pub fn with_code(message: impl Into<String>, code: u8) -> Self {
		Self {
			message: message.into(),
			code,
		}
	}
}

impl From<std::io::Error> for Failure {
	fn from(error: std::io::Error) -> Self {
		Self::new(error.to_string())
	}
}

pub type Outcome<T = ()> = Result<T, Failure>;

/// `EX_USAGE`: the command was called wrongly.
pub const USAGE: u8 = 64;
/// `EX_UNAVAILABLE`: a program this needs is missing.
pub const UNAVAILABLE: u8 = 69;

/// The repository's root: two levels above this crate. The tools are only
/// ever built from the tree they run on, so this is that tree.
pub fn root() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.ancestors()
		.nth(2)
		.expect("crates/xtask has two parents")
		.to_path_buf()
}

/// Runs `program` in `directory` with the terminal's input and output, and
/// returns its exit code. A program that cannot be started at all fails
/// with the reason.
pub fn status<I, S>(program: &str, arguments: I, directory: &Path) -> Outcome<i32>
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	let status = Command::new(program)
		.args(arguments)
		.current_dir(directory)
		.status()
		.map_err(|error| {
			Failure::with_code(format!("Could not run {program}: {error}."), UNAVAILABLE)
		})?;

	// A program killed by a signal has no code; report it as a failure.
	Ok(status.code().unwrap_or(1))
}

/// Runs `program` and fails, naming it, unless it exits with 0.
pub fn require<I, S>(program: &str, arguments: I, directory: &Path) -> Outcome
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	match status(program, arguments, directory)? {
		0 => Ok(()),
		code => {
			Err(Failure::with_code(
				format!("{program} failed (exit {code})."),
				exit_code(code),
			))
		}
	}
}

/// Whether `program` is an executable on `PATH`.
pub fn on_path(program: &str) -> bool {
	let Some(path) = std::env::var_os("PATH") else {
		return false;
	};

	std::env::split_paths(&path).any(|directory| is_executable(&directory.join(program)))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
	use std::os::unix::fs::PermissionsExt;

	path.metadata()
		.is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
	path.is_file() || path.with_extension("exe").is_file()
}

/// A child's exit code as this process's: the same, or 1 where it does not
/// fit or would read as success.
pub fn exit_code(code: i32) -> u8 {
	u8::try_from(code)
		.ok()
		.filter(|code| *code != 0)
		.unwrap_or(1)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn root_is_the_repository() {
		let root = root();

		assert!(root.join("Cargo.toml").is_file());
		assert!(root.join("crates/xtask/Cargo.toml").is_file());
	}

	#[test]
	fn exit_codes_never_read_as_success() {
		assert_eq!(exit_code(3), 3);
		assert_eq!(exit_code(0), 1);
		assert_eq!(exit_code(-1), 1);
		assert_eq!(exit_code(300), 1);
	}

	#[test]
	fn a_missing_program_is_unavailable() {
		let failure = status("reapers-no-such-program", ["--version"], &root()).unwrap_err();

		assert_eq!(failure.code, UNAVAILABLE);
		assert!(!on_path("reapers-no-such-program"));
	}
}
