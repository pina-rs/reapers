//! Brings the game's assets into the project before leptoskit bundles it.
//!
//! The models, music and NFT portraits are built into the repository's own
//! `assets/` (by `tool/build_art.dart` and `tool/build_music.dart`), and the
//! typefaces and the mark live with the Flutter game. leptoskit bundles the
//! project's `assets/` directory, and only real files in it, so each of
//! those is mirrored there: copied when it differs, never rewritten when it
//! does not, so `leptoskit dev`'s watcher does not chase its own tail. The
//! mirrors are git-ignored; the sources stay the only copies in git.

use std::fs;
use std::path::Path;

/// Each mirror: where it comes from, relative to this crate, and where it
/// goes under `assets/`.
const MIRRORS: [(&str, &str); 5] = [
	("../../assets/nfts", "nfts"),
	("../../assets/music", "music"),
	("../../assets/models", "models"),
	("../../reapers_flutter/assets/fonts", "fonts"),
	("../../reapers_flutter/assets/branding", "branding"),
];

fn mirror(from: &Path, to: &Path) -> std::io::Result<()> {
	fs::create_dir_all(to)?;
	for entry in fs::read_dir(from)? {
		let entry = entry?;
		let source = entry.path();
		let target = to.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			mirror(&source, &target)?;
		} else if fs::read(&target).ok() != Some(fs::read(&source)?) {
			fs::copy(&source, &target)?;
		}
	}
	Ok(())
}

fn main() -> Result<(), leptoskit_build::Error> {
	for (from, to) in MIRRORS {
		println!("cargo::rerun-if-changed={from}");
		let from = Path::new(from);
		if from.is_dir()
			&& let Err(error) = mirror(from, &Path::new("assets").join(to))
		{
			println!(
				"cargo::warning=could not mirror {}: {error}",
				from.display()
			);
		}
	}
	leptoskit_build::platforms()
}
