//! Brings the game's assets into the project before leptoskit bundles it.
//!
//! The models, music and NFT portraits are built into the repository's own
//! `assets/` (by `cargo xtask build-art` and `cargo xtask build-music`),
//! beside the typefaces and the mark, and the depth model is fetched there
//! by `cargo xtask fetch-depth-model`. leptoskit bundles the project's
//! `assets/` directory, and only real files in it, so each of those is
//! mirrored there: copied when it differs, never rewritten when it
//! does not, so `leptoskit dev`'s watcher does not chase its own tail. The
//! mirrors are git-ignored; the sources stay the only copies in git.
//!
//! The copy of a `.glb` under `assets/models` is the model as the game's
//! renderer draws it (`build/glb.rs`): the art build's mask colours,
//! `COLOR_0`, are left out, and every other attribute, `COLOR_1`'s ink
//! weights among them, is kept. Reapers and weapons have their materials
//! whitened; husks keep their authored materials.

use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

#[path = "build/glb.rs"]
mod glb;

/// Each mirror: where it comes from relative to this crate, where it goes under
/// `assets/`, and whether missing or unreadable sources should fail the build.
const MIRRORS: [(&str, &str, bool); 7] = [
	("../../assets/nfts", "nfts", true),
	("../../assets/music", "music", true),
	("../../assets/sfx", "sfx", true),
	("../../assets/models", "models", true),
	("../../assets/fonts", "fonts", true),
	("../../assets/branding", "branding", true),
	// Fetched by `cargo xtask fetch-depth-model`, not committed: the xr
	// plugin reads rooms with it on phones without platform depth.
	("../../assets/depth", "depth", false),
];

/// Returns `true` if `file_name` under mirrored category `group` is needed at
/// runtime and should be bundled into `crates/reapers_app/assets/`.
fn should_mirror(group: &str, file_name: &str) -> bool {
	let has_ext = |target: &str| {
		Path::new(file_name)
			.extension()
			.is_some_and(|ext| ext.eq_ignore_ascii_case(target))
	};
	match group {
		// Runtime screens use the WebP marks; PNGs stay in git for native
		// launcher icon generation (`leptoskit.toml`) and OpenGraph meta tags.
		"branding" => has_ext("webp"),
		// Ship only WOFF2 web fonts and their required OFL license files.
		"fonts" => has_ext("woff2") || file_name.ends_with("-LICENSE.txt"),
		// Credits and manifest metadata are compiled in or checked by tests
		// from the repository root; do not ship them in the runtime bundle.
		"music" | "sfx" => has_ext("ogg"),
		_ => true,
	}
}

fn mirror(group: &str, from: &Path, to: &Path) -> std::io::Result<()> {
	println!("cargo::rerun-if-changed={}", from.display());
	fs::create_dir_all(to)?;

	let mut expected: HashSet<OsString> = HashSet::new();
	for entry in fs::read_dir(from)? {
		let entry = entry?;
		let file_name = entry.file_name();
		let source = entry.path();
		let target = to.join(&file_name);

		if entry.file_type()?.is_dir() {
			expected.insert(file_name);
			mirror(group, &source, &target)?;
		} else {
			if let Some(name_str) = file_name.to_str()
				&& !should_mirror(group, name_str)
			{
				continue;
			}
			expected.insert(file_name);
			println!("cargo::rerun-if-changed={}", source.display());
			let mut bytes = fs::read(&source)?;
			if source
				.extension()
				.is_some_and(|extension| extension == "glb")
			{
				let name = source
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or_default();
				bytes =
					glb::for_renderer(bytes, name.starts_with("reaper_") || name == "weapons.glb");
			}
			if fs::read(&target).ok().as_deref() != Some(bytes.as_slice()) {
				fs::write(&target, bytes)?;
			}
		}
	}

	// Prune mirrored files or directories that were removed from the source
	// or filtered out of the runtime bundle.
	for entry in fs::read_dir(to)? {
		let entry = entry?;
		if !expected.contains(&entry.file_name()) {
			let stale = entry.path();
			if entry.file_type()?.is_dir() {
				fs::remove_dir_all(&stale)?;
			} else {
				fs::remove_file(&stale)?;
			}
		}
	}

	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	for (from, to, required) in MIRRORS {
		let from = Path::new(from);
		let target = Path::new("assets").join(to);
		println!("cargo::rerun-if-changed={}", from.display());

		if !from.is_dir() {
			if required {
				return Err(
					format!("required asset directory {} is missing", from.display()).into(),
				);
			}
			if target.is_dir() {
				let _ = fs::remove_dir_all(&target);
			}
			continue;
		}

		if let Err(error) = mirror(to, from, &target) {
			if required {
				return Err(format!("could not mirror {}: {error}", from.display()).into());
			}
			println!(
				"cargo::warning=could not mirror {}: {error}",
				from.display()
			);
		}
	}
	leptoskit_build::platforms()?;
	Ok(())
}
