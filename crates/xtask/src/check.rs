//! Runs every check a change has to pass before it reaches main.
//!
//! ```sh
//! cargo xtask check            # everything
//! cargo xtask check --fast     # skip the server's tests, which start PostgreSQL
//! ```
//!
//! In order: formatting, clippy over every target, the Wasm builds of the
//! crates the game and the server share, the program's SBF build and its
//! size, every test, and, when the leptoskit CLI is installed, the check of
//! `leptoskit.workspace.toml` and the web builds of the app and the landing.
//! It stops at the first step that fails and exits with that step's code.
//! `cargo xtask merge` runs the same checks, in full, before it lets a branch
//! into main, and CI runs them on every pull request.

use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::UNAVAILABLE;

/// The largest the deployed program may be, in bytes. It is 46,792 today
/// (`docs/program.md`); the ceiling leaves room to grow and still fails a
/// build that lost its fat LTO (20-35% larger) or kept `core::fmt` for
/// `verbose-logs`. Rent and the deploy's write buffer are paid by the byte.
pub const PROGRAM_SIZE_LIMIT: u64 = 56 * 1024;

/// Where `pina build` puts the program, from the repository's root.
const PROGRAM: &str = "target/deploy/reapers_program.so";

/// Where the program's tests look for Metaplex Core, from the root.
const CORE_FIXTURE: &str = "target/fixtures/mpl_core.so";

/// The crates the game builds for the browser and the native apps' Wasm
/// guest, which must build there on their own.
const WASM_CRATES: [&str; 4] = [
	"reapers_rules",
	"reapers_receipt",
	"reapers_api",
	"reapers_program_client",
];

/// The app's Wasm guest, checked for one platform's feature set. The app's
/// camera and room code is compiled only for `wasm32` with the `web` or
/// `native` feature, so the host-side clippy and tests never see it and the
/// optional leptoskit web build is the only step that does — until it is
/// not installed and a field renamed on one path (but not the other) ships
/// unnoticed. These checks need no leptoskit CLI and no platform toolchain
/// beyond the `wasm32` target the `wasm` step already requires.
fn guest(platform: &'static str) -> Action {
	cargo(&[
		"check",
		"--locked",
		"--target",
		"wasm32-unknown-unknown",
		"--package",
		"reapers_app",
		"--no-default-features",
		"--features",
		platform,
	])
}

/// The crate whose tests start a PostgreSQL cluster.
const SERVER: &str = "reapers_server";

/// One thing to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
	/// Run a program, in a directory relative to the root.
	Run {
		program: &'static str,
		arguments: Vec<String>,
		directory: &'static str,
	},
	/// Hold the built program to [`PROGRAM_SIZE_LIMIT`].
	ProgramSize,
}

#[derive(Debug)]
pub struct Step {
	pub name: String,
	pub action: Action,
}

/// What the checks depend on that differs from one machine to the next.
#[derive(Debug)]
pub struct Plan {
	/// Skip the server's tests.
	pub fast: bool,
	/// The leptoskit CLI is installed, so the workspace check and the app's
	/// and the landing's web builds can run.
	pub leptoskit: bool,
	/// Metaplex Core still has to be fetched for the program's tests.
	pub fetch_core: bool,
	/// Every crate `cargo test` covers by default, by name.
	pub packages: Vec<String>,
}

/// How a run of the checks went: the steps that passed, those that were
/// skipped and why, and the failure that stopped it, if one did.
#[derive(Debug)]
pub struct Report {
	pub passed: Vec<String>,
	pub skipped: Vec<String>,
	pub failure: Option<Failure>,
}

fn cargo(arguments: &[&str]) -> Action {
	Action::Run {
		program: "cargo",
		arguments: arguments.iter().map(ToString::to_string).collect(),
		directory: ".",
	}
}

fn leptoskit(arguments: &[&str], directory: &'static str) -> Action {
	Action::Run {
		program: "leptoskit",
		arguments: arguments.iter().map(ToString::to_string).collect(),
		directory,
	}
}

fn step(name: impl Into<String>, action: Action) -> Step {
	Step {
		name: name.into(),
		action,
	}
}

/// The steps, in order, for `plan`.
pub fn steps(plan: &Plan) -> Vec<Step> {
	let mut steps = vec![
		step("format", cargo(&["fmt", "--all", "--", "--check"])),
		step(
			"clippy",
			cargo(&[
				"clippy",
				"--locked",
				"--all-targets",
				"--",
				"--deny",
				"warnings",
			]),
		),
	];

	let mut wasm = vec!["build", "--locked", "--target", "wasm32-unknown-unknown"];
	for package in WASM_CRATES {
		wasm.extend(["--package", package]);
	}
	steps.push(step("wasm", cargo(&wasm)));
	steps.push(step("app native guest", guest("native")));
	steps.push(step("app web guest", guest("web")));

	steps.push(step(
		"program",
		Action::Run {
			program: "pina",
			arguments: vec!["build".to_owned()],
			directory: "crates/reapers_program",
		},
	));
	steps.push(step("program size", Action::ProgramSize));

	if plan.fetch_core {
		steps.push(step(
			"fetch Metaplex Core",
			cargo(&[
				"run",
				"--locked",
				"--quiet",
				"--package",
				"reapers_ops",
				"--",
				"fixtures",
			]),
		));
	}

	let mut test = vec!["test".to_owned(), "--locked".to_owned()];
	for package in &plan.packages {
		if !(plan.fast && package == SERVER) {
			test.extend(["--package".to_owned(), package.clone()]);
		}
	}
	steps.push(step(
		"test",
		Action::Run {
			program: "cargo",
			arguments: test,
			directory: ".",
		},
	));

	if plan.leptoskit {
		steps.push(step(
			"leptoskit workspace",
			leptoskit(&["workspace", "check"], "."),
		));
		steps.push(step(
			"app web build",
			leptoskit(&["build", "--platform", "web"], "crates/reapers_app"),
		));
		steps.push(step(
			"landing web build",
			leptoskit(&["build", "--platform", "web"], "crates/reapers_landing"),
		));
	}

	steps
}

/// The steps `plan` leaves out, and why.
pub fn skipped(plan: &Plan) -> Vec<String> {
	let mut skipped = Vec::new();

	if plan.fast {
		skipped.push(format!("the {SERVER} tests (--fast)"));
	}
	if !plan.leptoskit {
		skipped.push(
			"the leptoskit workspace check and the app's and the landing's web builds (no \
			 leptoskit CLI on PATH)"
				.to_owned(),
		);
	}

	skipped
}

/// Whether a program of `bytes` may be deployed.
pub fn program_size(bytes: u64) -> Result<String, String> {
	if bytes <= PROGRAM_SIZE_LIMIT {
		Ok(format!(
			"{PROGRAM} is {bytes} bytes, within {PROGRAM_SIZE_LIMIT}."
		))
	} else {
		Err(format!(
			"{PROGRAM} is {bytes} bytes, over the {PROGRAM_SIZE_LIMIT}-byte ceiling. Build it with `pina build` (fat LTO, no verbose-logs), or raise PROGRAM_SIZE_LIMIT in crates/xtask/src/check.rs and say why in docs/program.md."
		))
	}
}

/// Runs the checks from the repository's root and reports how they went.
pub fn run(root: &Path, fast: bool) -> Report {
	let failed = |failure| {
		Report {
			passed: Vec::new(),
			skipped: Vec::new(),
			failure: Some(failure),
		}
	};
	let plan = match plan(root, fast) {
		Ok(plan) => plan,
		Err(failure) => return failed(failure),
	};
	let mut report = Report {
		passed: Vec::new(),
		skipped: skipped(&plan),
		failure: None,
	};

	for step in steps(&plan) {
		println!("── {}", step.name);

		let outcome = match &step.action {
			Action::Run {
				program,
				arguments,
				directory,
			} => {
				run::status(program, arguments, &root.join(directory)).and_then(|code| {
					match code {
						0 => Ok(()),
						code => {
							Err(Failure::with_code(
								format!("✗ {} failed (exit {code})", step.name),
								run::exit_code(code),
							))
						}
					}
				})
			}
			Action::ProgramSize => {
				fs::metadata(root.join(PROGRAM))
					.map_err(|error| Failure::new(format!("✗ {PROGRAM}: {error}")))
					.and_then(|metadata| {
						program_size(metadata.len())
							.map(|said| println!("{said}"))
							.map_err(|said| Failure::new(format!("✗ {said}")))
					})
			}
		};

		if let Err(failure) = outcome {
			report.failure = Some(failure);
			return report;
		}

		report.passed.push(step.name);
	}

	report
}

/// Looks at the machine, and fails before anything runs, with the fix,
/// when it lacks something a step needs.
fn plan(root: &Path, fast: bool) -> Outcome<Plan> {
	for (program, fix) in [
		("pina", "cargo install pina_cli --version 0.24.0 --locked"),
		(
			"cargo-build-sbf",
			"install Solana's CLI (Agave), which brings the platform tools: https://docs.anza.xyz/cli/install",
		),
	] {
		if !run::on_path(program) {
			return Err(Failure::with_code(
				format!(
					"The program's build needs {program}, which is not on PATH. To get it:\n    {fix}"
				),
				UNAVAILABLE,
			));
		}
	}

	let fetch_core = !root.join(CORE_FIXTURE).is_file();

	if fetch_core && !run::on_path("solana") {
		return Err(Failure::with_code(
			format!(
				"The program's tests need Metaplex Core at {CORE_FIXTURE}, fetched with the solana CLI, which is not on PATH."
			),
			UNAVAILABLE,
		));
	}

	if !fast && !postgres_available() {
		return Err(Failure::with_code(
			"The server's tests start their own PostgreSQL and need initdb, pg_ctl and createdb on PATH, or in the directory REAPERS_PG_BIN names. Install PostgreSQL's binaries, or skip those tests:\n    cargo xtask check --fast",
			UNAVAILABLE,
		));
	}

	Ok(Plan {
		fast,
		leptoskit: run::on_path("leptoskit"),
		fetch_core,
		packages: default_members(root)?,
	})
}

fn postgres_available() -> bool {
	let tools = ["initdb", "pg_ctl", "createdb"];

	match std::env::var_os("REAPERS_PG_BIN") {
		Some(directory) => {
			tools
				.iter()
				.all(|tool| Path::new(&directory).join(tool).is_file())
		}
		None => tools.iter().all(|tool| run::on_path(tool)),
	}
}

/// The crates `cargo test` covers with no `--package`: the workspace's
/// default members. The app's native runners are members too, but build only
/// after `leptoskit build` has generated their registrants.
fn default_members(root: &Path) -> Outcome<Vec<String>> {
	let output = std::process::Command::new("cargo")
		.args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
		.current_dir(root)
		.output()?;

	if !output.status.success() {
		return Err(Failure::new(format!(
			"cargo metadata failed:\n{}",
			String::from_utf8_lossy(&output.stderr)
		)));
	}

	let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|error| {
		Failure::new(format!(
			"cargo metadata wrote something unreadable: {error}"
		))
	})?;

	default_member_names(&metadata)
		.ok_or_else(|| Failure::new("cargo metadata named no default members."))
}

fn default_member_names(metadata: &serde_json::Value) -> Option<Vec<String>> {
	let members = metadata["workspace_default_members"].as_array()?;
	let mut names: Vec<String> = metadata["packages"]
		.as_array()?
		.iter()
		.filter(|package| members.contains(&package["id"]))
		.filter_map(|package| package["name"].as_str().map(ToOwned::to_owned))
		.collect();

	names.sort();

	(!names.is_empty()).then_some(names)
}

pub fn main(root: &Path, fast: bool) -> Outcome {
	let started = Instant::now();
	let report = run(root, fast);

	if let Some(failure) = report.failure {
		return Err(failure);
	}

	for skipped in &report.skipped {
		println!("skipped: {skipped}");
	}
	println!(
		"✓ all checks passed in {}s{}",
		started.elapsed().as_secs(),
		if report.skipped.is_empty() {
			""
		} else {
			", with the steps above skipped"
		}
	);

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn plan(fast: bool, leptoskit: bool, fetch_core: bool) -> Plan {
		Plan {
			fast,
			leptoskit,
			fetch_core,
			packages: vec![
				"reapers_rules".to_owned(),
				SERVER.to_owned(),
				"xtask".to_owned(),
			],
		}
	}

	fn names(plan: &Plan) -> Vec<String> {
		steps(plan).into_iter().map(|step| step.name).collect()
	}

	fn arguments(plan: &Plan, name: &str) -> Vec<String> {
		match steps(plan)
			.into_iter()
			.find(|step| step.name == name)
			.map(|step| step.action)
		{
			Some(Action::Run { arguments, .. }) => arguments,
			other => panic!("{name}: {other:?}"),
		}
	}

	#[test]
	fn a_full_run_does_everything_in_order() {
		assert_eq!(
			names(&plan(false, true, true)),
			[
				"format",
				"clippy",
				"wasm",
				"app native guest",
				"app web guest",
				"program",
				"program size",
				"fetch Metaplex Core",
				"test",
				"leptoskit workspace",
				"app web build",
				"landing web build",
			]
		);
		assert_eq!(skipped(&plan(false, true, false)), Vec::<String>::new());
	}

	#[test]
	fn the_program_is_built_and_measured_before_its_tests() {
		let names = names(&plan(true, false, true));
		let at = |name: &str| names.iter().position(|step| step == name).unwrap();

		assert!(at("program") < at("program size"));
		assert!(at("program size") < at("test"));
		assert!(at("fetch Metaplex Core") < at("test"));
	}

	#[test]
	fn fast_leaves_out_only_the_server_tests() {
		let full = arguments(&plan(false, true, false), "test");
		let fast = arguments(&plan(true, true, false), "test");

		assert!(full.contains(&SERVER.to_owned()));
		assert!(!fast.contains(&SERVER.to_owned()));
		assert!(fast.contains(&"reapers_rules".to_owned()));
		assert_eq!(
			names(&plan(true, true, false)),
			names(&plan(false, true, false))
		);
		assert_eq!(
			skipped(&plan(true, true, false)),
			["the reapers_server tests (--fast)"]
		);
	}

	#[test]
	fn the_web_builds_run_only_with_the_cli() {
		let names = names(&plan(false, false, false));
		for name in ["leptoskit workspace", "app web build", "landing web build"] {
			assert!(!names.contains(&name.to_owned()), "{name}");
		}
		assert_eq!(
			skipped(&plan(false, false, false)),
			[
				"the leptoskit workspace check and the app's and the landing's web builds (no \
				 leptoskit CLI on PATH)"
			]
		);
	}

	#[test]
	fn the_apps_guest_is_checked_on_both_platforms_without_the_cli() {
		let plan = plan(false, false, false);

		for (name, platform) in [("app native guest", "native"), ("app web guest", "web")] {
			let arguments = arguments(&plan, name);

			assert!(
				arguments.contains(&"--locked".to_owned()),
				"{name} is locked"
			);
			assert_eq!(arguments[3], "wasm32-unknown-unknown".to_owned(), "{name}");
			assert!(
				arguments.contains(&"reapers_app".to_owned())
					&& arguments.contains(&"native".to_owned()) == (platform == "native")
					&& arguments.contains(&"web".to_owned()) == (platform == "web"),
				"{name}: {arguments:?}"
			);
		}
	}

	#[test]
	fn each_frontend_is_built_in_its_own_directory() {
		for (name, directory) in [
			("app web build", "crates/reapers_app"),
			("landing web build", "crates/reapers_landing"),
		] {
			let action = steps(&plan(false, true, false))
				.into_iter()
				.find(|step| step.name == name)
				.map(|step| step.action);
			match action {
				Some(Action::Run {
					directory: built_in,
					..
				}) => assert_eq!(built_in, directory, "{name}"),
				other => panic!("{name}: {other:?}"),
			}
		}
	}

	#[test]
	fn lints_deny_warnings_and_every_build_is_locked() {
		let plan = plan(false, true, true);

		assert!(
			arguments(&plan, "clippy").ends_with(&["--deny".to_owned(), "warnings".to_owned()])
		);
		for name in [
			"clippy",
			"wasm",
			"app native guest",
			"app web guest",
			"fetch Metaplex Core",
			"test",
		] {
			assert!(
				arguments(&plan, name).contains(&"--locked".to_owned()),
				"{name}"
			);
		}
		for package in WASM_CRATES {
			assert!(arguments(&plan, "wasm").contains(&package.to_owned()));
		}
	}

	#[test]
	fn holds_the_program_to_its_ceiling() {
		assert!(program_size(49_352).is_ok());
		assert!(program_size(PROGRAM_SIZE_LIMIT).is_ok());
		// The same program without fat LTO, at a fifth larger.
		assert!(program_size(49_352 * 6 / 5).is_err());
	}

	#[test]
	fn reads_the_default_members() {
		let metadata = serde_json::json!({
			"packages": [
				{ "id": "path+file:///r/crates/a#0.1.0", "name": "a" },
				{ "id": "path+file:///r/crates/app/ios#0.1.0", "name": "app_ios" },
				{ "id": "path+file:///r/crates/b#0.1.0", "name": "b" },
			],
			"workspace_default_members": [
				"path+file:///r/crates/b#0.1.0",
				"path+file:///r/crates/a#0.1.0",
			],
		});

		assert_eq!(default_member_names(&metadata).unwrap(), ["a", "b"]);
		assert_eq!(default_member_names(&serde_json::json!({})), None);
	}

	#[test]
	fn this_workspace_tests_its_crates_and_not_the_native_runners() {
		let names = default_members(&run::root()).unwrap();

		for wanted in [
			"reapers_rules",
			"reapers_server",
			"reapers_program",
			"xtask",
		] {
			assert!(names.contains(&wanted.to_owned()), "{wanted}");
		}
		assert!(!names.iter().any(|name| name.starts_with("reapers_app_")));
	}
}
