//! Merges the current branch into `main`, locally, once it passes the checks.
//!
//! This repository has no pull requests. This is the whole flow:
//!
//! ```sh
//! git switch -c feat/your-change   # work and commit on a branch
//! cargo xtask merge                # check it, then land it on main
//! cargo xtask merge --push         # the same, then push main
//! ```
//!
//! It runs every check in `cargo xtask check` on the commit being merged,
//! records that the commit passed, and fast-forwards `main` to it. The hooks
//! installed by `cargo xtask hooks install` refuse to move `main` to a
//! commit without that record, so this is the only way in.
//!
//! `main` only ever fast-forwards, which keeps its history a straight line.
//! If `main` has moved since the branch was cut, rebase the branch first.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use reapers_rules::time::Timestamp;

use crate::check;
use crate::check::Report;
use crate::git::Git;
use crate::hooks;
use crate::run::Failure;
use crate::run::Outcome;

pub fn main(root: &Path, push: bool) -> Outcome {
	let git = Git::at(root);

	land(&git, || check::run(root, false))?;

	if !push {
		println!("Push it when you are ready:\n    git push origin main");

		return Ok(());
	}

	match git.command(&["push", "origin", "main"]).status()?.code() {
		Some(0) => Ok(()),
		code => {
			Err(Failure::with_code(
				"",
				crate::run::exit_code(code.unwrap_or(1)),
			))
		}
	}
}

/// Checks the branch `git` has out with `checks`, and fast-forwards `main`
/// to it if they pass.
pub fn land(git: &Git, checks: impl FnOnce() -> Report) -> Outcome {
	let branch = git.read(&["rev-parse", "--abbrev-ref", "HEAD"])?;

	if branch == "main" || branch == "HEAD" {
		return Err(Failure::new(
			"There is nothing to merge from here. Switch to the branch that holds the work, then run this again.",
		));
	}

	let uncommitted = git.read(&["status", "--porcelain"])?;

	if !uncommitted.is_empty() {
		return Err(Failure::new(format!(
			"The working tree has uncommitted changes. Only committed work can be checked and merged, so commit them first:\n{uncommitted}"
		)));
	}

	if !hooks::status(git) {
		return Err(Failure::new(
			"Install the hooks first, so main cannot be moved around them.",
		));
	}

	let head = git.read(&["rev-parse", "HEAD"])?;
	let base = git.read(&["rev-parse", "main"])?;

	if head == base {
		println!("{branch} is already on main. Nothing to merge.");

		return Ok(());
	}

	// If main has commits the branch lacks, merging would put a tree on main
	// that the checks never saw.
	if !git.succeeds(&["merge-base", "--is-ancestor", &base, &head]) {
		return Err(Failure::new(format!(
			"main has moved since {branch} was cut. Rebase onto it, then run this again:\n    git rebase main"
		)));
	}

	println!(
		"Checking {branch} at {} before it reaches main.",
		short(&head)
	);

	let report = checks();

	if let Some(failure) = report.failure {
		return Err(Failure::with_code(
			format!(
				"\nThe checks failed, so main was not moved.\n{}",
				failure.message
			),
			failure.code,
		));
	}

	// The checks may not leave anything behind, or what was checked is not
	// what is committed.
	if !git.read(&["status", "--porcelain"])?.is_empty()
		|| git.read(&["rev-parse", "HEAD"])? != head
	{
		return Err(Failure::new(
			"The tree changed while the checks ran. Commit the change and retry.",
		));
	}

	let record = Record::write(git, &branch, &head, &report)?;
	// A record is good for one landing. Leaving it would let that commit be
	// put back on main later without being checked again; dropping the
	// record deletes it whether or not the fast-forward worked.
	let landed = fast_forward_main(git, &branch);

	drop(record);
	landed?;

	let count = git.read(&["rev-list", "--count", &format!("{base}..{head}")])?;

	println!(
		"✓ main is now at {} ({count} commit(s) from {branch}).",
		short(&head)
	);

	Ok(())
}

/// Moves `main` up to `branch` without anyone having to leave the branch.
fn fast_forward_main(git: &Git, branch: &str) -> Outcome {
	// A checkout that has main out must be the one to move it, or its files
	// would be left behind. With main checked out nowhere, the ref alone
	// moves.
	let output = match git.worktree_on("main")? {
		None => git.output(&["fetch", "--quiet", ".", &format!("{branch}:main")])?,
		Some(checkout) => Git::at(&checkout).output(&["merge", "--ff-only", "--quiet", branch])?,
	};

	if output.status.success() {
		Ok(())
	} else {
		Err(Failure::new(format!(
			"main could not be fast-forwarded:\n{}",
			String::from_utf8_lossy(&output.stderr).trim_end()
		)))
	}
}

/// The verification record for one commit, deleted when dropped.
struct Record(PathBuf);

impl Record {
	fn write(git: &Git, branch: &str, commit: &str, report: &Report) -> Outcome<Self> {
		let path = git
			.common_directory()?
			.join(hooks::VERIFIED_DIRECTORY)
			.join(record_name(branch));
		let verified_at = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.map_or(0, |elapsed| elapsed.as_secs().cast_signed());
		let contents = serde_json::json!({
			"branch": branch,
			"commit": commit,
			"checks": report.passed,
			"skipped": report.skipped,
			"verifiedAt": Timestamp::from_unix_seconds(verified_at).to_iso8601(),
		});

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(
			&path,
			format!(
				"{}\n",
				serde_json::to_string_pretty(&contents).expect("a record is JSON")
			),
		)?;

		Ok(Self(path))
	}
}

impl Drop for Record {
	fn drop(&mut self) {
		let _ = fs::remove_file(&self.0);
	}
}

/// The file a branch's record is kept in.
fn record_name(branch: &str) -> String {
	format!("{}.json", branch.replace('/', "-"))
}

fn short(commit: &str) -> &str {
	commit.get(..8).unwrap_or(commit)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::hooks::tests::Scratch;

	fn passing() -> Report {
		Report {
			passed: vec!["format".to_owned(), "test".to_owned()],
			skipped: Vec::new(),
			failure: None,
		}
	}

	fn failing() -> Report {
		Report {
			passed: vec!["format".to_owned()],
			skipped: Vec::new(),
			failure: Some(Failure::with_code("✗ test failed (exit 101)", 101)),
		}
	}

	fn guarded_branch() -> (Scratch, String) {
		let scratch = Scratch::new();

		hooks::install(&scratch.git).unwrap();
		scratch
			.git
			.read(&["switch", "--quiet", "--create", "feat/thing"])
			.unwrap();
		let feature = scratch.commit("feature");

		(scratch, feature)
	}

	fn records(scratch: &Scratch) -> usize {
		fs::read_dir(
			scratch
				.git
				.common_directory()
				.unwrap()
				.join(hooks::VERIFIED_DIRECTORY),
		)
		.map_or(0, Iterator::count)
	}

	#[test]
	fn lands_a_checked_branch_and_forgets_the_record() {
		let (scratch, feature) = guarded_branch();
		let mut checked = false;

		land(&scratch.git, || {
			checked = true;
			passing()
		})
		.unwrap();

		assert!(checked);
		assert_eq!(scratch.head_of("main"), feature);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn moves_the_checkout_that_has_main_out() {
		let (scratch, feature) = guarded_branch();
		let other = scratch.path().with_extension("main");
		let _ = fs::remove_dir_all(&other);

		// main is out in a second worktree; its files must move with it.
		scratch
			.git
			.read(&["switch", "--quiet", "--detach"])
			.unwrap();
		scratch
			.git
			.read(&[
				"worktree",
				"add",
				"--quiet",
				other.to_str().unwrap(),
				"main",
			])
			.unwrap();
		scratch
			.git
			.read(&["switch", "--quiet", "feat/thing"])
			.unwrap();
		land(&scratch.git, passing).unwrap();

		assert_eq!(scratch.head_of("main"), feature);
		assert!(other.join("feature.txt").is_file());
		fs::remove_dir_all(&other).unwrap();
	}

	#[test]
	fn failed_checks_leave_main_where_it_was() {
		let (scratch, _) = guarded_branch();
		let before = scratch.head_of("main");
		let failure = land(&scratch.git, failing).unwrap_err();

		assert_eq!(failure.code, 101);
		assert_eq!(scratch.head_of("main"), before);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn refuses_without_running_the_checks() {
		let refusals = [
			// On main itself.
			|scratch: &Scratch| {
				scratch.git.read(&["switch", "--quiet", "main"]).unwrap();
			},
			// With uncommitted work.
			|scratch: &Scratch| {
				fs::write(scratch.path().join("loose.txt"), "loose").unwrap();
			},
			// Without the guard.
			|scratch: &Scratch| {
				hooks::uninstall(&scratch.git).unwrap();
			},
			// With main moved on since the branch was cut.
			|scratch: &Scratch| {
				let feature = scratch.head_of("HEAD");
				scratch
					.git
					.read(&["switch", "--quiet", "--create", "side", "main"])
					.unwrap();
				let side = scratch.commit("side");
				scratch.verify(&side);
				scratch
					.git
					.read(&["branch", "--force", "main", "side"])
					.unwrap();
				scratch
					.git
					.read(&["switch", "--quiet", "feat/thing"])
					.unwrap();
				assert_eq!(scratch.head_of("HEAD"), feature);
			},
		];

		for refuse in refusals {
			let (scratch, _) = guarded_branch();

			refuse(&scratch);
			let before = scratch.head_of("main");

			land(&scratch.git, || panic!("the checks ran")).unwrap_err();
			assert_eq!(scratch.head_of("main"), before);
		}
	}

	#[test]
	fn refuses_when_the_checks_change_the_tree() {
		let (scratch, _) = guarded_branch();
		let before = scratch.head_of("main");

		land(&scratch.git, || {
			fs::write(scratch.path().join("generated.txt"), "left behind").unwrap();
			passing()
		})
		.unwrap_err();

		assert_eq!(scratch.head_of("main"), before);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn names_records_after_branches() {
		assert_eq!(record_name("feat/main-menu"), "feat-main-menu.json");
		assert_eq!(short("0123456789abcdef"), "01234567");
	}
}
