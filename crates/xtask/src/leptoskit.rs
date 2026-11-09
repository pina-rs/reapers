//! Moves every leptoskit pin, and the CLI's, to one revision.
//!
//! ```sh
//! cargo xtask leptoskit              # the head of leptoskit's main
//! cargo xtask leptoskit --rev <sha>  # a revision of your choosing
//! ```
//!
//! leptoskit reaches this workspace from git, and its revision is written in
//! more than one place: every `leptoskit*` line under `[patch.crates-io]` in
//! the root `Cargo.toml`, `LEPTOSKIT_REV` in CI, and the revisions the
//! READMEs tell a person to install the CLI at. This rewrites all of them
//! together, moves the lock's leptoskit entries with `cargo update`, and then
//! says what to do next: install the CLI at the new revision and run
//! `cargo xtask check`. See AGENTS.md, "leptoskit and Solana Kit are pinned
//! in one place".
//!
//! Solana Kit is pinned the same way and is deliberately left alone: it has
//! its own revision and nothing here matches it.
//!
//! Without `--rev` the head of leptoskit's main is asked of the remote with
//! `git ls-remote`, which sends the repository's address and the ref asked
//! for, and nothing else. The files it rewrites have to be committed first:
//! moving half a pin leaves a workspace that does not build.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::git::Git;
use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::USAGE;

/// Where leptoskit lives.
pub const ORIGIN: &str = "https://github.com/leptoskit/leptoskit";

/// The root's manifest: the one place every leptoskit crate is pinned.
const MANIFEST: &str = "Cargo.toml";
/// The lock that moves with the manifest.
const LOCK: &str = "Cargo.lock";
/// CI installs the CLI at the revision the workspace pins.
const CI: &str = ".github/workflows/ci.yml";
/// The files that tell a person how to install the CLI.
const DOCUMENTS: [&str; 3] = ["README.md", "AGENTS.md", "crates/reapers_app/README.md"];
/// The documentation, which may name the revision too.
const DOCS: &str = "docs";
/// The app's manifest: it pins `wasm-bindgen` to the version the CLI embeds.
const APP_MANIFEST: &str = "crates/reapers_app/Cargo.toml";

#[derive(clap::Args)]
pub struct Options {
	/// The revision to pin. Defaults to the head of leptoskit's main.
	#[arg(long)]
	rev: Option<String>,
}

pub fn main(root: &Path, options: &Options) -> Outcome {
	let git = Git::at(root);
	let manifest = fs::read_to_string(root.join(MANIFEST))?;
	let revisions = pinned_revisions(&manifest);
	let [old] = revisions.as_slice() else {
		return Err(Failure::new(if revisions.is_empty() {
			format!("{MANIFEST} pins no leptoskit revision to move.")
		} else {
			format!(
				"{MANIFEST} pins leptoskit to more than one revision: {}. Move them by hand first.",
				revisions.join(", ")
			)
		}));
	};
	let old = old.clone();

	let new = match &options.rev {
		Some(revision) => canonical(revision).ok_or_else(|| {
			Failure::with_code(
				format!(
					"--rev takes the full 40-character commit SHA, not {}. Leave it out for the head of {ORIGIN}'s main.",
					revision.trim()
				),
				USAGE,
			)
		})?,
		None => head(&git)?,
	};

	let files = pinned_files(root)?;
	require_clean(&git, &files)?;

	for path in &files {
		let text = fs::read_to_string(root.join(path))?;
		let rewritten = replace_revision(&text, &old, &new);

		if rewritten == text {
			println!("same   {}", path.display());
		} else {
			fs::write(root.join(path), rewritten)?;
			println!("wrote  {}", path.display());
		}
	}

	let lock = fs::read_to_string(root.join(LOCK))?;
	move_the_lock(root, &lock)?;
	let moved = fs::read_to_string(root.join(LOCK))?;
	println!("{}   {LOCK}", if moved == lock { "same" } else { "wrote" });

	check_wasm_bindgen(root, &new)?;

	println!();
	println!("leptoskit {old} -> {new}");
	println!();
	println!("Next:");
	println!("  cargo install --locked --git {ORIGIN} --rev {new} leptoskit_cli --force");
	println!("  cargo xtask check");

	Ok(())
}

/// The head of leptoskit's main, as the remote reports it.
fn head(git: &Git) -> Outcome<String> {
	let listing = git
		.read(&["ls-remote", ORIGIN, "refs/heads/main"])
		.map_err(|failure| {
			Failure::new(format!(
				"Could not read the head of {ORIGIN}'s main: {}",
				failure.message
			))
		})?;

	revision_from_ls_remote(&listing).ok_or_else(|| {
		Failure::new(format!(
			"{ORIGIN} reported no head for main. Pass --rev to pin a revision by hand."
		))
	})
}

/// The revision `git ls-remote <url> refs/heads/main` answered with.
fn revision_from_ls_remote(listing: &str) -> Option<String> {
	listing.lines().find_map(|line| {
		let mut fields = line.split_whitespace();
		let revision = fields.next()?;
		let name = fields.next()?;

		if name != "refs/heads/main" {
			return None;
		}

		canonical(revision)
	})
}

/// A revision as the pins write it: 40 lowercase hexadecimal digits.
fn canonical(revision: &str) -> Option<String> {
	let revision = revision.trim().to_ascii_lowercase();
	let whole = revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit());

	whole.then_some(revision)
}

/// Every distinct revision the manifest pins a leptoskit crate at, in the
/// order they are written.
fn pinned_revisions(manifest: &str) -> Vec<String> {
	let mut revisions: Vec<String> = Vec::new();

	for revision in manifest
		.lines()
		.filter(|line| line.trim_start().starts_with("leptoskit"))
		.filter(|line| line.contains(ORIGIN))
		.filter_map(revision_in)
	{
		if !revisions.contains(&revision) {
			revisions.push(revision);
		}
	}

	revisions
}

/// The `rev` a manifest line names.
fn revision_in(line: &str) -> Option<String> {
	let rest = line.split_once("rev = \"")?.1;
	let revision = rest.split('"').next()?;

	canonical(revision)
}

/// `text` with every mention of `old` a whole revision, and not part of a
/// longer hash, replaced by `new`. Nothing else is touched: another project's
/// revision is another revision.
fn replace_revision(text: &str, old: &str, new: &str) -> String {
	let mut rewritten = String::with_capacity(text.len());
	let mut last = 0;
	let mut search = 0;

	while let Some(offset) = text[search..].find(old) {
		let start = search + offset;
		let end = start + old.len();
		search = end;

		let before = start
			.checked_sub(1)
			.and_then(|index| text.as_bytes().get(index));
		let after = text.as_bytes().get(end);
		let whole =
			!before.is_some_and(u8::is_ascii_hexdigit) && !after.is_some_and(u8::is_ascii_hexdigit);

		if !whole {
			continue;
		}

		rewritten.push_str(&text[last..start]);
		rewritten.push_str(new);
		last = end;
	}

	rewritten.push_str(&text[last..]);
	rewritten
}

/// The files a move rewrites: the manifest, CI, and every document that may
/// name the revision. Paths are from the root.
fn pinned_files(root: &Path) -> Outcome<Vec<PathBuf>> {
	let mut files: Vec<PathBuf> = [MANIFEST, CI].into_iter().map(PathBuf::from).collect();
	files.extend(DOCUMENTS.into_iter().map(PathBuf::from));
	markdown(root, Path::new(DOCS), &mut files)?;
	files.sort_unstable();

	Ok(files)
}

/// Every markdown file under `relative`, a directory of `root`.
fn markdown(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Outcome {
	let entries = match fs::read_dir(root.join(relative)) {
		Ok(entries) => entries,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
		Err(error) => return Err(Failure::new(format!("{}: {error}", relative.display()))),
	};

	for entry in entries {
		let entry = entry?;
		let path = relative.join(entry.file_name());

		if entry.file_type()?.is_dir() {
			markdown(root, &path, files)?;
		} else if path.extension().is_some_and(|extension| extension == "md") {
			files.push(path);
		}
	}

	Ok(())
}

/// Whether every file this would rewrite has its changes committed.
fn require_clean(git: &Git, files: &[PathBuf]) -> Outcome {
	let mut arguments: Vec<String> = ["status", "--porcelain", "--"]
		.into_iter()
		.map(ToOwned::to_owned)
		.collect();
	arguments.extend(files.iter().map(|path| path.display().to_string()));
	let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
	let dirty = git.read(&arguments)?;

	if dirty.is_empty() {
		return Ok(());
	}

	Err(Failure::new(format!(
		"These files have changes that are not committed:\n{dirty}\nCommit them first: a move rewrites every pin and the lock together, and half of one does not build."
	)))
}

/// Moves only the lock's leptoskit entries, and nothing else's.
fn move_the_lock(root: &Path, lock: &str) -> Outcome {
	let packages = locked_packages(lock);

	if packages.is_empty() {
		println!("skip   {LOCK} names no leptoskit package");

		return Ok(());
	}

	let mut arguments = vec!["update".to_owned()];
	for package in &packages {
		arguments.extend(["-p".to_owned(), package.clone()]);
	}

	run::require("cargo", &arguments, root)
}

/// The packages the lock takes from leptoskit's repository.
fn locked_packages(lock: &str) -> Vec<String> {
	let mut packages = Vec::new();
	let mut name: Option<String> = None;

	for line in lock.lines().map(str::trim) {
		if line == "[[package]]" {
			name = None;
		} else if let Some(rest) = line.strip_prefix("name = \"") {
			name = rest.split('"').next().map(ToOwned::to_owned);
		} else if line.starts_with("source = ") {
			let from_leptoskit = line.contains(&format!("git+{ORIGIN}"));

			if from_leptoskit && let Some(name) = name.take() {
				packages.push(name);
			}

			name = None;
		}
	}

	packages
}

/// Compares the app's `wasm-bindgen` pin with the version the leptoskit CLI
/// at `revision` embeds; they have to be the same, and the CLI's is
/// leptoskit's own `Cargo.lock` that cargo checks out for the build.
fn check_wasm_bindgen(root: &Path, revision: &str) -> Outcome {
	let manifest = fs::read_to_string(root.join(APP_MANIFEST))?;
	let Some(pinned) = wasm_bindgen_pin(&manifest) else {
		println!("check  {APP_MANIFEST} pins no wasm-bindgen version");

		return Ok(());
	};
	let Some(embedded) = embedded_wasm_bindgen(revision) else {
		println!(
			"check  install the CLI, then compare the wasm-bindgen it embeds with ={pinned} in {APP_MANIFEST}"
		);
		println!(
			"       (leptoskit's Cargo.lock at {revision} is not in cargo's git checkouts yet)"
		);

		return Ok(());
	};

	if embedded == pinned {
		println!("have   wasm-bindgen {pinned}, the version the CLI embeds");
	} else {
		println!(
			"warn   the CLI at {revision} embeds wasm-bindgen {embedded}, but {APP_MANIFEST} pins ={pinned}. Change that pin to ={embedded}, or the web build will not link."
		);
	}

	Ok(())
}

/// The version `wasm-bindgen` is pinned to in a manifest: the CLI's own
/// wasm-bindgen tooling accepts exactly its matching library.
fn wasm_bindgen_pin(manifest: &str) -> Option<String> {
	let line = manifest
		.lines()
		.find(|line| line.trim_start().starts_with("wasm-bindgen ="))?;
	let rest = line.split_once("version = \"=")?.1;
	let version = rest.split('"').next()?;

	Some(version.to_owned())
}

/// The `wasm-bindgen` in leptoskit's `Cargo.lock`, from cargo's checkout of
/// `revision`.
fn embedded_wasm_bindgen(revision: &str) -> Option<String> {
	let cargo_home = std::env::var_os("CARGO_HOME")
		.map(PathBuf::from)
		.or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo")))?;
	let checkouts = fs::read_dir(cargo_home.join("git/checkouts")).ok()?;

	for checkout in checkouts.flatten() {
		let directory = checkout.path().join(&revision[..7]);

		if !directory.join("crates/leptoskit_cli").is_dir() {
			continue;
		}

		let lock = fs::read_to_string(directory.join("Cargo.lock")).ok()?;

		return locked_version(&lock, "wasm-bindgen");
	}

	None
}

/// The version the lock gives `package`.
fn locked_version(lock: &str, package: &str) -> Option<String> {
	let name = format!("name = \"{package}\"");
	let mut inside = false;

	for line in lock.lines().map(str::trim) {
		if line == "[[package]]" {
			inside = false;
		} else if line == name {
			inside = true;
		} else if inside && let Some(rest) = line.strip_prefix("version = \"") {
			return Some(rest.split('"').next()?.to_owned());
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The revision the pins are moved from in these tests.
	const OLD: &str = "b878753721d8c81819b8b332cc0a7b9455097c86";
	/// The revision they are moved to.
	const NEW: &str = "365e2a2341ba4c675048e2a05e4caf946d4c623b";
	/// Solana Kit's, which a move never touches.
	const SOLANA_KIT: &str = "17912ee644d2898c9de3ecded026274ada8761e7";

	#[test]
	fn moves_every_pin_of_one_revision() {
		let manifest = format!(
			"[patch.crates-io]\n\
			 leptoskit = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n\
			 leptoskit_3d = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n"
		);
		let ci = format!("  LEPTOSKIT_REV: {OLD}\n");
		let readme = format!("  --rev {OLD} leptoskit_cli\n");

		for text in [&manifest, &ci, &readme] {
			let moved = replace_revision(text, OLD, NEW);

			assert!(!moved.contains(OLD));
			assert_eq!(moved.matches(NEW).count(), text.matches(OLD).count());
		}
	}

	#[test]
	fn leaves_another_projects_revision_and_longer_hashes_alone() {
		let checksum = format!("{OLD}{}", "0".repeat(24));
		let text = format!(
			"solana_kit_core = {{ git = \"https://github.com/pina-rs/solana_kit\", rev = \"{SOLANA_KIT}\" }}\n\
			 sha256 = \"{checksum}\"\n\
			 leptoskit = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n"
		);
		let moved = replace_revision(&text, OLD, NEW);

		assert!(moved.contains(SOLANA_KIT), "Solana Kit's pin moved");
		assert!(moved.contains(&checksum), "a longer hash was rewritten");
		assert!(
			moved.contains(&format!("rev = \"{NEW}\"")),
			"leptoskit's pin did not move"
		);
	}

	#[test]
	fn moving_a_pin_twice_is_moving_it_once() {
		let text = format!(
			"leptoskit = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n\
			 LEPTOSKIT_REV: {OLD}\n\
			 --rev {OLD} leptoskit_cli\n"
		);
		let once = replace_revision(&text, OLD, NEW);

		assert_eq!(replace_revision(&once, OLD, NEW), once);
		assert!(!once.contains(OLD));
	}

	#[test]
	fn reads_the_leptoskit_pin_not_the_solana_kit_one() {
		let manifest = format!(
			"[workspace.dependencies]\n\
			 solana_kit_core = {{ git = \"https://github.com/pina-rs/solana_kit\", rev = \"{SOLANA_KIT}\" }}\n\
			 \n\
			 [patch.crates-io]\n\
			 leptoskit = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n\
			 leptoskit_3d = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n"
		);

		assert_eq!(pinned_revisions(&manifest), [OLD.to_owned()]);
		assert_eq!(
			pinned_revisions("[workspace]\nmembers = []\n"),
			Vec::<String>::new()
		);
	}

	#[test]
	fn reads_a_divergent_pin_as_more_than_one() {
		let manifest = format!(
			"leptoskit = {{ git = \"{ORIGIN}\", rev = \"{OLD}\" }}\n\
			 leptoskit_3d = {{ git = \"{ORIGIN}\", rev = \"{NEW}\" }}\n"
		);

		assert_eq!(
			pinned_revisions(&manifest),
			[OLD.to_owned(), NEW.to_owned()]
		);
	}

	#[test]
	fn reads_the_head_of_main_from_ls_remote() {
		let listing = format!("{NEW}\trefs/heads/main\n{OLD}\trefs/heads/next\n");

		assert_eq!(revision_from_ls_remote(&listing), Some(NEW.to_owned()));
		assert_eq!(revision_from_ls_remote(""), None);
		assert_eq!(
			revision_from_ls_remote(&format!("{OLD}\trefs/heads/next\n")),
			None
		);
	}

	#[test]
	fn takes_only_a_full_revision_for_one_to_pin() {
		assert_eq!(canonical(NEW).as_deref(), Some(NEW));
		assert_eq!(
			canonical(&format!(" {} ", NEW.to_uppercase())).as_deref(),
			Some(NEW)
		);
		assert_eq!(canonical("365e2a2"), None);
		assert_eq!(canonical("main"), None);
		assert_eq!(canonical(&format!("{NEW}0")), None);
	}

	#[test]
	fn compares_the_apps_wasm_bindgen_pin_with_the_clis() {
		let manifest = "\
			# The embedded wasm-bindgen CLI accepts exactly its matching library version.\n\
			wasm-bindgen = { version = \"=0.2.129\", optional = true }\n\
			wasm-bindgen-futures = { version = \"0.4\", optional = true }\n\
		";
		let lock = "\
			[[package]]\n\
			name = \"wasm-bindgen\"\n\
			version = \"0.2.129\"\n\
			source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\
			\n\
			[[package]]\n\
			name = \"wasm-bindgen-futures\"\n\
			version = \"0.4.79\"\n\
		";

		assert_eq!(wasm_bindgen_pin(manifest), Some("0.2.129".to_owned()));
		assert_eq!(wasm_bindgen_pin("[dependencies]\n"), None);
		assert_eq!(
			locked_version(lock, "wasm-bindgen"),
			Some("0.2.129".to_owned())
		);
		assert_eq!(
			locked_version(lock, "wasm-bindgen-futures"),
			Some("0.4.79".to_owned())
		);
		assert_eq!(locked_version(lock, "leptoskit"), None);
	}

	#[test]
	fn moves_every_leptoskit_entry_in_the_lock() {
		let lock = format!(
			"[[package]]\n\
			 name = \"leptoskit\"\n\
			 version = \"0.3.0\"\n\
			 source = \"git+{ORIGIN}?rev={OLD}#{OLD}\"\n\
			 \n\
			 [[package]]\n\
			 name = \"leptoskit_3d\"\n\
			 version = \"0.3.0\"\n\
			 source = \"git+{ORIGIN}?rev={OLD}#{OLD}\"\n\
			 \n\
			 [[package]]\n\
			 name = \"solana_kit_core\"\n\
			 version = \"0.1.0\"\n\
			 source = \"git+https://github.com/pina-rs/solana_kit?rev={SOLANA_KIT}#{SOLANA_KIT}\"\n\
			 \n\
			 [[package]]\n\
			 name = \"serde\"\n\
			 version = \"1.0.0\"\n\
			 source = \"registry+https://github.com/rust-lang/crates.io-index\"\n"
		);

		assert_eq!(locked_packages(&lock), ["leptoskit", "leptoskit_3d"]);
		assert_eq!(
			locked_packages("[package]\nname = \"x\"\n"),
			Vec::<String>::new()
		);
	}

	/// A revision that moved in one file and not another leaves a workspace
	/// that does not build.
	#[test]
	fn every_file_pins_the_same_leptoskit_revision() {
		let root = run::root();
		let manifest = fs::read_to_string(root.join(MANIFEST)).unwrap();
		let revisions = pinned_revisions(&manifest);

		assert_eq!(revisions.len(), 1, "{MANIFEST} pins {revisions:?}");
		let pinned = &revisions[0];

		for path in pinned_files(&root).unwrap() {
			let text = fs::read_to_string(root.join(&path)).unwrap();

			for revision in revisions_after_rev(&text) {
				assert_eq!(&revision, pinned, "{} names {revision}", path.display());
			}
		}

		let lock = fs::read_to_string(root.join(LOCK)).unwrap();

		for revision in locked_sources(&lock) {
			assert_eq!(&revision, pinned, "{LOCK} names {revision}");
		}
		assert!(
			!locked_packages(&lock).is_empty(),
			"{LOCK} names no leptoskit package"
		);
	}

	/// Every revision a document tells a person to install the CLI at.
	fn revisions_after_rev(text: &str) -> Vec<String> {
		text.split("--rev ")
			.skip(1)
			.filter_map(|rest| rest.split_whitespace().next().and_then(canonical))
			.collect()
	}

	/// Every revision the lock takes a leptoskit package from.
	fn locked_sources(lock: &str) -> Vec<String> {
		let marker = format!("git+{ORIGIN}?rev=");

		lock.lines()
			.filter_map(|line| line.split_once(&marker))
			.filter_map(|(_, rest)| rest.split('#').next().and_then(canonical))
			.collect()
	}
}
