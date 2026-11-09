//! Lands the current branch on `main` through a pull request.
//!
//! Every change lands as a pull request, and the checks that let it land run
//! here rather than on GitHub. This is the whole flow:
//!
//! ```sh
//! git switch -c feat/your-change   # work and commit on a branch
//! cargo xtask merge                # push, open the pull request, check, land
//! ```
//!
//! It pushes the branch, opens a pull request from it into `main` (or reuses
//! the open one), runs every check in `cargo xtask check` on the commit being
//! merged, and records that the commit passed. The hooks installed by
//! `cargo xtask hooks install` refuse to move `main` to a commit without that
//! record, so this is the only way in.
//!
//! The pull request is merged by pushing `main`: GitHub marks it merged as
//! soon as its head commit is on the branch it targets. `gh pr merge` is never
//! used, because it rewrites or re-signs the commits it lands, and these
//! commits carry the owner's signature. The run then comments its check
//! report on the pull request, confirms GitHub marked it merged, and deletes
//! the branch on origin.
//!
//! `main` only ever fast-forwards, which keeps its history a straight line.
//! If `main` has moved since the branch was cut, rebase the branch first.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use reapers_rules::time::Timestamp;

use crate::check;
use crate::check::Report;
use crate::git::Git;
use crate::hooks;
use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::UNAVAILABLE;

/// How many times to ask `gh` whether the push merged the pull request, and
/// how long to wait between asks. GitHub reads a push in a moment; this waits
/// that moment out without waiting forever on a GitHub that never answers.
const MERGED_ATTEMPTS: usize = 10;
const MERGED_INTERVAL: Duration = Duration::from_secs(1);

pub fn main(root: &Path) -> Outcome {
	let git = Git::at(root);

	let Some(target) = Target::resolve(&git)? else {
		return Ok(());
	};

	// Everything from here talks to GitHub, so say now, before the checks
	// take ten minutes, when it cannot.
	preflight(root)?;

	let pull = open_pull_request(root, &git, &target)?;

	println!(
		"Checking {} at {} before it reaches main.",
		target.branch,
		short(&target.commit)
	);

	let started = Instant::now();
	let report = check::run(root, false);
	let seconds = started.elapsed().as_secs();

	land(&git, &target, &report)?;
	finish(root, &git, &target, &pull, &report, seconds)
}

/// The branch to land: what it is called, the commit it holds, and the commit
/// of `main` it fast-forwards.
#[derive(Debug)]
struct Target {
	branch: String,
	commit: String,
	base: String,
}

impl Target {
	/// Refuses the checkouts that cannot be landed, in the words they have
	/// always been refused in, and answers `None` when the branch is already
	/// on `main`.
	fn resolve(git: &Git) -> Outcome<Option<Self>> {
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

		let commit = git.read(&["rev-parse", "HEAD"])?;
		let base = git.read(&["rev-parse", "main"])?;

		if commit == base {
			println!("{branch} is already on main. Nothing to merge.");

			return Ok(None);
		}

		// If main has commits the branch lacks, merging would put a tree on main
		// that the checks never saw.
		if !git.succeeds(&["merge-base", "--is-ancestor", &base, &commit]) {
			return Err(Failure::new(format!(
				"main has moved since {branch} was cut. Rebase onto it, then run this again:\n    git rebase main"
			)));
		}

		Ok(Some(Self {
			branch,
			commit,
			base,
		}))
	}
}

/// Pushes `main`, which is what merges the pull request on GitHub, then says
/// on the pull request what the checks did, waits for GitHub to agree it
/// merged, and deletes the branch on origin.
fn finish(
	root: &Path,
	git: &Git,
	target: &Target,
	pull: &PullRequest,
	report: &Report,
	seconds: u64,
) -> Outcome {
	// Not `gh pr merge`: that rewrites or re-signs the commits it lands.
	git.show(&["push", "origin", "main"]).map_err(|failure| {
		Failure::with_code(
			format!(
				"{}\nmain moved here but not on origin, so the pull request is still open. Push it to merge:\n    git push origin main",
				failure.message
			),
			failure.code,
		)
	})?;
	comment(root, pull, target, report, seconds)?;
	confirm_merged(root, pull)?;

	if let Err(left) = delete_remote_branch(git, &target.branch) {
		println!("The branch was left on origin: {}", left.message);
	}

	println!("✓ {} is merged: {}", target.branch, pull.url);

	Ok(())
}

/// Fast-forwards `main` to `target` once `report` says the checks passed on
/// exactly its commit.
fn land(git: &Git, target: &Target, report: &Report) -> Outcome {
	if let Some(failure) = &report.failure {
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
		|| git.read(&["rev-parse", "HEAD"])? != target.commit
	{
		return Err(Failure::new(
			"The tree changed while the checks ran. Commit the change and retry.",
		));
	}

	let record = Record::write(git, &target.branch, &target.commit, report)?;
	// A record is good for one landing. Leaving it would let that commit be
	// put back on main later without being checked again; dropping the
	// record deletes it whether or not the fast-forward worked.
	let landed = fast_forward_main(git, &target.branch);

	drop(record);
	landed?;

	let count = git.read(&[
		"rev-list",
		"--count",
		&format!("{}..{}", target.base, target.commit),
	])?;

	println!(
		"✓ main is now at {} ({count} commit(s) from {}).",
		short(&target.commit),
		target.branch
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

/// `gh` is how the branch becomes a pull request and how the run reports on
/// it, so its absence, or a signed-out one, is worth knowing before the
/// checks start.
fn preflight(root: &Path) -> Outcome {
	if !run::on_path("gh") {
		return Err(Failure::with_code(
			"Landing a change opens a pull request, which needs the GitHub CLI on PATH. Install it and sign in:\n    brew install gh\n    gh auth login",
			UNAVAILABLE,
		));
	}

	gh(root, &["auth", "status"]).map_err(|unauthorised| {
		Failure::with_code(
			format!(
				"Landing a change opens a pull request, and gh cannot:\n{}\nSign in with:\n    gh auth login",
				unauthorised.message
			),
			UNAVAILABLE,
		)
	})?;

	Ok(())
}

/// Pushes `target` and answers the pull request it lands through: the one the
/// branch already has open, or a new one.
fn open_pull_request(root: &Path, git: &Git, target: &Target) -> Outcome<PullRequest> {
	push_branch(git, &target.branch)?;

	if let Some(pull) = reusable(found(root, &target.branch)) {
		println!("Reusing pull request: {}", pull.url);

		return Ok(pull);
	}

	let url = gh(
		root,
		&[
			"pr",
			"create",
			"--base",
			"main",
			"--head",
			&target.branch,
			"--fill",
		],
	)?;

	println!("Opened pull request: {url}");

	Ok(PullRequest {
		url,
		state: State::Open,
	})
}

/// Replaces origin's copy of the branch. Branches are rebased before they
/// land, so what arrives is a different history, never more of the same one.
fn push_branch(git: &Git, branch: &str) -> Outcome {
	// `--force-with-lease` compares against the copy of the branch this
	// checkout last saw, and refuses a branch it has never seen as `stale
	// info`: every branch first pushed from another checkout. Taking origin's
	// copy first makes the lease the real thing. A branch origin does not have
	// yet is the first push, and there is nothing to compare against.
	git.output(&[
		"fetch",
		"--quiet",
		"origin",
		&format!("+refs/heads/{branch}:refs/remotes/origin/{branch}"),
	])?;

	git.show(&["push", "--force-with-lease", "origin", branch])
}

/// Says on the pull request what the branch passed, since the checks that let
/// it land only ran on the machine that landed it.
fn comment(
	root: &Path,
	pull: &PullRequest,
	target: &Target,
	report: &Report,
	seconds: u64,
) -> Outcome {
	gh(
		root,
		&[
			"pr",
			"comment",
			&pull.url,
			"--body",
			&comment_text(target, report, seconds),
		],
	)
	.map(|_| ())
}

/// The comment `cargo xtask merge` leaves on the pull request: the commit the
/// checks saw, what each of them did, and how long they took.
fn comment_text(target: &Target, report: &Report, seconds: u64) -> String {
	let mut lines = vec![
		format!(
			"`cargo xtask merge` checked **{}** at `{}` on the machine that landed it.",
			target.branch, target.commit
		),
		String::new(),
	];

	for passed in &report.passed {
		lines.push(format!("- ✓ {passed}"));
	}
	for skipped in &report.skipped {
		lines.push(format!("- skipped: {skipped}"));
	}

	lines.push(String::new());
	lines.extend([
		format!("All checks passed in {}.", duration(seconds)),
		"Merged by fast-forwarding `main`; GitHub Actions runs the same checks as a mirror."
			.to_owned(),
	]);

	lines.join("\n")
}

/// Waits for GitHub to notice that the push merged the pull request.
fn confirm_merged(root: &Path, pull: &PullRequest) -> Outcome {
	for attempt in 1..=MERGED_ATTEMPTS {
		if attempt > 1 {
			thread::sleep(MERGED_INTERVAL);
		}

		if found(root, &pull.url).is_some_and(|pull| pull.state == State::Merged) {
			return Ok(());
		}
	}

	Err(Failure::new(format!(
		"main was pushed, but GitHub has not marked {} merged. It will once it reads the push; check it with:\n    gh pr view {}",
		pull.url, pull.url
	)))
}

/// Deletes the branch on origin, now that its pull request is merged. A
/// branch already gone is not a failure: a pull request can be set to delete
/// it.
fn delete_remote_branch(git: &Git, branch: &str) -> Outcome {
	if git
		.read(&["ls-remote", "--heads", "origin", branch])?
		.is_empty()
	{
		return Ok(());
	}

	git.show(&["push", "--delete", "origin", branch])
}

/// A pull request this run lands through. Its URL is its identity, because a
/// pull request keeps its URL after it merges.
#[derive(Debug, PartialEq, Eq)]
struct PullRequest {
	url: String,
	state: State,
}

impl PullRequest {
	/// The pull request, as `gh pr view --json number,url,state` prints one.
	fn parse(printed: &str) -> Option<Self> {
		let printed: serde_json::Value = serde_json::from_str(printed).ok()?;
		let state = match printed["state"].as_str()? {
			"OPEN" => State::Open,
			"CLOSED" => State::Closed,
			"MERGED" => State::Merged,
			_ => return None,
		};

		Some(Self {
			url: printed["url"].as_str()?.to_owned(),
			state,
		})
	}
}

/// Where a pull request stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
	Open,
	Closed,
	Merged,
}

/// The pull request this run can add its report to: only an open one. A
/// merged or closed pull request belongs to the commits the branch used to
/// hold, and the branch needs a new one.
fn reusable(pull: Option<PullRequest>) -> Option<PullRequest> {
	pull.filter(|pull| pull.state == State::Open)
}

/// The pull request `gh pr view` knows at `reference`, a branch name or a
/// URL. Anything else it says, including that there is no pull request, is
/// `None`: the caller's next `gh` call says what went wrong when GitHub
/// cannot be asked at all.
fn found(root: &Path, reference: &str) -> Option<PullRequest> {
	gh(
		root,
		&["pr", "view", reference, "--json", "number,url,state"],
	)
	.ok()
	.and_then(|printed| PullRequest::parse(&printed))
}

/// gh in the repository, answering what it printed. Its output is read
/// rather than shown: every call here is one this decides on.
fn gh(root: &Path, arguments: &[&str]) -> Outcome<String> {
	let output = Command::new("gh")
		.args(arguments)
		.current_dir(root)
		.output()
		.map_err(|error| Failure::with_code(format!("Could not run gh: {error}."), UNAVAILABLE))?;

	if !output.status.success() {
		return Err(Failure::new(format!(
			"gh {} failed:\n{}",
			arguments.join(" "),
			String::from_utf8_lossy(&output.stderr).trim_end()
		)));
	}

	Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// How long the checks took, as a person reads it.
fn duration(seconds: u64) -> String {
	if seconds < 60 {
		format!("{seconds}s")
	} else {
		format!("{}m {}s", seconds / 60, seconds % 60)
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

	fn guarded_branch() -> (Scratch, Target) {
		let scratch = Scratch::new();

		hooks::install(&scratch.git).unwrap();
		scratch
			.git
			.read(&["switch", "--quiet", "--create", "feat/thing"])
			.unwrap();
		scratch.commit("feature");
		let target = Target::resolve(&scratch.git).unwrap().unwrap();

		(scratch, target)
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
		let (scratch, target) = guarded_branch();

		land(&scratch.git, &target, &passing()).unwrap();

		assert_eq!(scratch.head_of("main"), target.commit);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn moves_the_checkout_that_has_main_out() {
		let (scratch, target) = guarded_branch();
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
		land(&scratch.git, &target, &passing()).unwrap();

		assert_eq!(scratch.head_of("main"), target.commit);
		assert!(other.join("feature.txt").is_file());
		fs::remove_dir_all(&other).unwrap();
	}

	#[test]
	fn failed_checks_leave_main_where_it_was() {
		let (scratch, target) = guarded_branch();
		let before = scratch.head_of("main");
		let failure = land(&scratch.git, &target, &failing()).unwrap_err();

		assert_eq!(failure.code, 101);
		assert_eq!(scratch.head_of("main"), before);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn refuses_a_checkout_that_cannot_land() {
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

			Target::resolve(&scratch.git).unwrap_err();
			assert_eq!(scratch.head_of("main"), before);
		}
	}

	#[test]
	fn says_nothing_to_merge_when_the_branch_is_on_main() {
		let scratch = Scratch::new();

		hooks::install(&scratch.git).unwrap();
		scratch
			.git
			.read(&["switch", "--quiet", "--create", "feat/thing"])
			.unwrap();

		assert!(Target::resolve(&scratch.git).unwrap().is_none());
	}

	#[test]
	fn refuses_when_the_checks_change_the_tree() {
		let (scratch, target) = guarded_branch();
		let before = scratch.head_of("main");

		fs::write(scratch.path().join("generated.txt"), "left behind").unwrap();
		land(&scratch.git, &target, &passing()).unwrap_err();

		assert_eq!(scratch.head_of("main"), before);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn lands_only_the_commit_the_checks_saw() {
		let (scratch, target) = guarded_branch();
		let before = scratch.head_of("main");

		scratch.commit("afterwards");
		land(&scratch.git, &target, &passing()).unwrap_err();

		assert_eq!(scratch.head_of("main"), before);
		assert_eq!(records(&scratch), 0);
	}

	#[test]
	fn reuses_only_an_open_pull_request() {
		let printed = |state: &str| {
			format!(
				r#"{{"number": 12, "url": "https://github.com/pina-rs/reapers/pull/12", "state": "{state}"}}"#
			)
		};
		let open = PullRequest {
			url: "https://github.com/pina-rs/reapers/pull/12".to_owned(),
			state: State::Open,
		};

		// An open one is added to; a merged or closed one belongs to commits
		// the branch has moved past, so a new one is opened instead.
		assert_eq!(reusable(PullRequest::parse(&printed("OPEN"))), Some(open));
		assert_eq!(reusable(PullRequest::parse(&printed("MERGED"))), None);
		assert_eq!(reusable(PullRequest::parse(&printed("CLOSED"))), None);
		// gh failed, which is how it says the branch has no pull request.
		assert_eq!(reusable(None), None);
	}

	#[test]
	fn reads_only_what_it_understands() {
		assert_eq!(
			PullRequest::parse(r#"{"state": "MERGED", "url": "u"}"#),
			Some(PullRequest {
				url: "u".to_owned(),
				state: State::Merged
			})
		);
		assert_eq!(PullRequest::parse("not json"), None);
		assert_eq!(PullRequest::parse(r#"{"state": "OPEN"}"#), None);
		assert_eq!(
			PullRequest::parse(r#"{"state": "OPENED", "url": "u"}"#),
			None
		);
	}

	#[test]
	fn comments_every_check_it_ran() {
		let target = Target {
			branch: "ci/merge-through-pull-requests".to_owned(),
			commit: "0123456789abcdef".to_owned(),
			base: "1111111111111111111111111111111111111111".to_owned(),
		};
		let mut report = passing();

		report
			.skipped
			.push("the reapers_server tests (--fast)".to_owned());
		let text = comment_text(&target, &report, 372);

		assert!(text.contains("**ci/merge-through-pull-requests** at `0123456789abcdef`"));
		assert!(text.contains("- ✓ format"));
		assert!(text.contains("- ✓ test"));
		assert!(text.contains("- skipped: the reapers_server tests (--fast)"));
		assert!(text.contains("All checks passed in 6m 12s."));
	}

	#[test]
	fn reads_a_run_as_minutes_and_seconds() {
		assert_eq!(duration(0), "0s");
		assert_eq!(duration(59), "59s");
		assert_eq!(duration(60), "1m 0s");
		assert_eq!(duration(372), "6m 12s");
	}

	#[test]
	fn replaces_the_branch_on_origin_and_removes_it() {
		let (scratch, _) = guarded_branch();
		let remote = scratch.path().with_extension("origin.git");
		let _ = fs::remove_dir_all(&remote);

		scratch
			.git
			.read(&["init", "--quiet", "--bare", remote.to_str().unwrap()])
			.unwrap();
		scratch
			.git
			.read(&["remote", "add", "origin", remote.to_str().unwrap()])
			.unwrap();
		push_branch(&scratch.git, "feat/thing").unwrap();
		assert_eq!(
			Git::at(&remote).read(&["rev-parse", "feat/thing"]).unwrap(),
			scratch.head_of("feat/thing")
		);

		// Rewritten history, from a checkout that has never seen origin's copy:
		// the lease is taken against origin, not against a ref that is not
		// there, which git would call stale.
		scratch
			.git
			.read(&["reset", "--quiet", "--hard", "main"])
			.unwrap();
		let rewritten = scratch.commit("rewritten");

		scratch
			.git
			.read(&["update-ref", "-d", "refs/remotes/origin/feat/thing"])
			.unwrap();
		push_branch(&scratch.git, "feat/thing").unwrap();
		assert_eq!(
			Git::at(&remote).read(&["rev-parse", "feat/thing"]).unwrap(),
			rewritten
		);

		delete_remote_branch(&scratch.git, "feat/thing").unwrap();
		assert!(!Git::at(&remote).succeeds(&["rev-parse", "--verify", "feat/thing"]));
		// A branch already gone is not a failure: a pull request can be set to
		// delete it.
		delete_remote_branch(&scratch.git, "feat/thing").unwrap();

		fs::remove_dir_all(&remote).unwrap();
	}

	#[test]
	fn names_records_after_branches() {
		assert_eq!(record_name("feat/main-menu"), "feat-main-menu.json");
		assert_eq!(short("0123456789abcdef"), "01234567");
	}
}
