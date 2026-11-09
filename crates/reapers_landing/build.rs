//! Brings the few game assets the landing page shows into the project
//! before leptoskit bundles it.
//!
//! leptoskit bundles the project's `assets/` directory, every file in it,
//! so this mirrors only what the page uses from the repository's own
//! `assets/`: the two typefaces and their licences, the mark, and the husk
//! portraits it shows. A file is copied when it differs and never rewritten
//! when it does not, so `leptoskit dev`'s watcher does not chase its own
//! tail. The mirrors are git-ignored; the sources stay the only copies in
//! git. A file that cannot be mirrored fails the build: the page would ship
//! without its type or its pictures.

use std::error::Error;
use std::fs;
use std::path::Path;

/// Each file the page uses, relative to the repository's `assets/`.
const MIRRORED: [&str; 10] = [
	"fonts/Fraunces.ttf",
	"fonts/Fraunces-LICENSE.txt",
	"fonts/DMSans.ttf",
	"fonts/DMSans-LICENSE.txt",
	"branding/mark.png",
	"nfts/degenApes_1.webp",
	"nfts/peskyPenguins_2.webp",
	"nfts/sollamas_4.webp",
	"nfts/solarians_3.webp",
	"nfts/thugbirdz_1.webp",
];

fn mirror(from: &Path, to: &Path) -> std::io::Result<()> {
	let bytes = fs::read(from)?;
	if fs::read(to).ok().as_deref() != Some(bytes.as_slice()) {
		if let Some(directory) = to.parent() {
			fs::create_dir_all(directory)?;
		}
		fs::write(to, bytes)?;
	}
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let source = Path::new("../../assets");
	for file in MIRRORED {
		let from = source.join(file);
		println!("cargo::rerun-if-changed={}", from.display());
		mirror(&from, &Path::new("assets").join(file))
			.map_err(|error| format!("could not mirror {}: {error}", from.display()))?;
	}
	// Where the page's "play" links go, read by `src/config.rs`.
	println!("cargo::rerun-if-env-changed=REAPERS_APP_URL");
	leptoskit_build::platforms()?;
	Ok(())
}
