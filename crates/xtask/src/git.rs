//! Asking git, in one checkout.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use crate::run::Failure;
use crate::run::Outcome;

/// git, run in one checkout of the repository.
pub struct Git {
	directory: PathBuf,
	/// Variables to set on every git this runs, for tests that need a git
	/// untouched by the person's own configuration.
	environment: Vec<(String, String)>,
}

impl Git {
	pub fn at(directory: &Path) -> Self {
		Self {
			directory: directory.to_path_buf(),
			environment: Vec::new(),
		}
	}

	/// A git that reads no global or system configuration, for tests in a
	/// repository of their own.
	#[cfg(test)]
	pub fn isolated(directory: &Path) -> Self {
		Self {
			directory: directory.to_path_buf(),
			environment: [
				("GIT_CONFIG_GLOBAL", "/dev/null"),
				("GIT_CONFIG_NOSYSTEM", "1"),
				("GIT_AUTHOR_NAME", "Reaper"),
				("GIT_AUTHOR_EMAIL", "reaper@reapers.invalid"),
				("GIT_COMMITTER_NAME", "Reaper"),
				("GIT_COMMITTER_EMAIL", "reaper@reapers.invalid"),
			]
			.into_iter()
			.map(|(name, value)| (name.to_owned(), value.to_owned()))
			.collect(),
		}
	}

	#[cfg(test)]
	pub fn directory(&self) -> &Path {
		&self.directory
	}

	/// A git command in this checkout, ready to run.
	pub fn command(&self, arguments: &[&str]) -> Command {
		let mut command = Command::new("git");

		command.args(arguments).current_dir(&self.directory);
		// Run from inside another git (a hook, a rebase), these would point
		// this one at that repository instead.
		for inherited in [
			"GIT_DIR",
			"GIT_WORK_TREE",
			"GIT_INDEX_FILE",
			"GIT_COMMON_DIR",
		] {
			command.env_remove(inherited);
		}
		for (name, value) in &self.environment {
			command.env(name, value);
		}

		command
	}

	pub fn output(&self, arguments: &[&str]) -> Outcome<Output> {
		self.command(arguments)
			.output()
			.map_err(|error| Failure::new(format!("Could not run git: {error}.")))
	}

	/// Runs git with its own output on the terminal, and fails when it does
	/// not succeed. For the commands a person watches run: a push, whose
	/// progress is worth seeing and whose error git explains itself.
	pub fn show(&self, arguments: &[&str]) -> Outcome {
		let status = self
			.command(arguments)
			.status()
			.map_err(|error| Failure::new(format!("Could not run git: {error}.")))?;

		if status.success() {
			Ok(())
		} else {
			Err(Failure::with_code(
				format!("git {} failed.", arguments.join(" ")),
				crate::run::exit_code(status.code().unwrap_or(1)),
			))
		}
	}

	/// What git prints, trimmed. Fails with git's own complaint.
	pub fn read(&self, arguments: &[&str]) -> Outcome<String> {
		let output = self.output(arguments)?;

		if !output.status.success() {
			return Err(Failure::new(format!(
				"git {} failed:\n{}",
				arguments.join(" "),
				String::from_utf8_lossy(&output.stderr).trim_end()
			)));
		}

		Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
	}

	/// Whether git exits with 0.
	pub fn succeeds(&self, arguments: &[&str]) -> bool {
		self.output(arguments)
			.is_ok_and(|output| output.status.success())
	}

	/// The git directory every worktree of this repository shares.
	pub fn common_directory(&self) -> Outcome<PathBuf> {
		let raw = self
			.read(&["rev-parse", "--git-common-dir"])
			.map_err(|_| Failure::new("Not inside a git repository."))?;
		let path = PathBuf::from(raw);

		Ok(if path.is_absolute() {
			path
		} else {
			self.directory.join(path)
		})
	}

	/// The checkout that has `branch` out, if any.
	pub fn worktree_on(&self, branch: &str) -> Outcome<Option<PathBuf>> {
		Ok(worktree_on(
			&self.read(&["worktree", "list", "--porcelain"])?,
			branch,
		))
	}
}

/// The checkout `git worktree list --porcelain` says has `branch` out.
pub fn worktree_on(porcelain: &str, branch: &str) -> Option<PathBuf> {
	let wanted = format!("branch refs/heads/{branch}");
	let mut path = None;

	for line in porcelain.lines() {
		if let Some(worktree) = line.strip_prefix("worktree ") {
			path = Some(PathBuf::from(worktree.trim()));
		} else if line.trim() == wanted {
			return path;
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn finds_the_checkout_a_branch_is_out_in() {
		let porcelain = "\
worktree /work/reapers
HEAD 1111111111111111111111111111111111111111
branch refs/heads/main

worktree /work/feature
HEAD 2222222222222222222222222222222222222222
branch refs/heads/feat/main-menu

worktree /work/detached
HEAD 3333333333333333333333333333333333333333
detached
";

		assert_eq!(
			worktree_on(porcelain, "main"),
			Some(PathBuf::from("/work/reapers"))
		);
		assert_eq!(
			worktree_on(porcelain, "feat/main-menu"),
			Some(PathBuf::from("/work/feature"))
		);
		// A branch whose name merely ends the same is a different branch.
		assert_eq!(worktree_on(porcelain, "menu"), None);
		assert_eq!(worktree_on(porcelain, "elsewhere"), None);
	}
}
