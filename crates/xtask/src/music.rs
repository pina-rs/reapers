//! Builds the game's music from the recordings it is made of.
//!
//! ```sh
//! cargo xtask build-music
//! ```
//!
//! Nothing here is composed or generated. Each piece is a recording somebody
//! made and released for anyone to use, fetched from where its maker put it,
//! checked against a known checksum, and re-encoded at a size a phone can
//! carry. Who made each one, and on what terms, is in
//! `assets/music/CREDITS.md`, and the game's colophon says the same to the
//! player: all three ask for that credit.
//!
//! The originals are kept in `art/build/music`, which is not committed. The
//! encoded pieces are, so nobody needs to run this to build the game. Run it
//! to change a piece or how it is encoded. It needs `curl` and `ffmpeg`.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use sha2::Digest;
use sha2::Sha256;

use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::UNAVAILABLE;

/// One piece of music: where the original is, and what it becomes.
struct Piece {
	/// The file it becomes, `<name>.ogg`.
	name: &'static str,
	/// Its title, as the credits give it.
	title: &'static str,
	url: &'static str,
	original: &'static str,
	sha256: &'static str,
	gain_decibels: i32,
}

const ORIGINALS: &str = "art/build/music";
const ENCODED: &str = "assets/music";

/// Vorbis's quality scale: 2 is about 96 kbit/s at 44.1 kHz stereo, and
/// in practice 80 for these quiet, slow pieces. Enough for a phone's speaker
/// or a pair of earphones, at a size the web build can fetch.
const VORBIS_QUALITY: &str = "2";

const PIECES: [Piece; 3] = [
	// "At the intersection of relaxation and horror": a slow drone, uneasy
	// but something to come back to.
	Piece {
		name: "sanctum",
		title: "Ossuary 1 - A Beginning",
		url: "https://incompetech.com/music/royalty-free/mp3-royaltyfree/Ossuary%201%20-%20A%20Beginning.mp3",
		original: "macleod_ossuary_1.mp3",
		sha256: "2c3140481edd0c6ab5ff21b46198a97755822e74e5fd0c78f2e7b456cece3cf8",
		gain_decibels: -1,
	},
	// Something in the shadows that cannot quite be heard. Brought down to
	// sit with the others.
	Piece {
		name: "hunt",
		title: "Penumbra",
		url: "https://incompetech.com/music/royalty-free/mp3-royaltyfree/Penumbra.mp3",
		original: "macleod_penumbra.mp3",
		sha256: "55bcea6dbd758694862a8c3de03c4f293cce2c13bd752fa570886d8849f43cfb",
		gain_decibels: -4,
	},
	// Almost a death march.
	Piece {
		name: "rift",
		title: "Oppressive Gloom",
		url: "https://incompetech.com/music/royalty-free/mp3-royaltyfree/Oppressive%20Gloom.mp3",
		original: "macleod_oppressive_gloom.mp3",
		sha256: "d630686bde6c92f832fb0cb16f6ce9009bf502c7db701139329af2a7683b1020",
		gain_decibels: -3,
	},
];

pub fn main(root: &Path) -> Outcome {
	for program in ["curl", "ffmpeg"] {
		if !run::on_path(program) {
			return Err(Failure::with_code(
				format!("The music build needs {program}, which is not on the PATH."),
				UNAVAILABLE,
			));
		}
	}

	fs::create_dir_all(root.join(ORIGINALS))?;
	fs::create_dir_all(root.join(ENCODED))?;

	for piece in &PIECES {
		let original = Path::new(ORIGINALS).join(piece.original);

		if !root.join(&original).is_file() {
			println!("fetching {}", piece.original);
			run::require("curl", fetch_arguments(piece, &original), root)?;
		}

		let checksum = sha256(&fs::read(root.join(&original))?);

		if checksum != piece.sha256 {
			return Err(Failure::new(format!(
				"{} is not the recording this piece was chosen as: its checksum is {checksum}. Delete it to fetch it again.",
				original.display()
			)));
		}

		let encoded = Path::new(ENCODED).join(format!("{}.ogg", piece.name));

		run::require("ffmpeg", encode_arguments(piece, &original, &encoded), root)?;

		let megabytes = fs::metadata(root.join(&encoded))?.len() as f64 / (1024.0 * 1024.0);

		println!(
			"wrote  {} ({}, {megabytes:.1} MB)",
			encoded.display(),
			piece.title
		);
	}

	Ok(())
}

fn fetch_arguments(piece: &Piece, original: &Path) -> Vec<String> {
	vec![
		"--fail".to_owned(),
		"--location".to_owned(),
		"--silent".to_owned(),
		"--show-error".to_owned(),
		"--proto".to_owned(),
		"=https".to_owned(),
		"--output".to_owned(),
		original.display().to_string(),
		piece.url.to_owned(),
	]
}

/// Stereo at 44.1 kHz in Vorbis. Nothing is done to the sound itself but a
/// plain change of level, and nothing but the sound is kept.
fn encode_arguments(piece: &Piece, original: &Path, encoded: &Path) -> Vec<String> {
	[
		"-loglevel",
		"error",
		"-y",
		"-i",
		&original.display().to_string(),
		"-map",
		"0:a:0",
		"-map_metadata",
		"-1",
		"-af",
		&format!("volume={}dB", piece.gain_decibels),
		"-ar",
		"44100",
		"-ac",
		"2",
		"-c:a",
		"libvorbis",
		"-q:a",
		VORBIS_QUALITY,
		&encoded.display().to_string(),
	]
	.map(ToOwned::to_owned)
	.to_vec()
}

fn sha256(bytes: &[u8]) -> String {
	Sha256::digest(bytes)
		.iter()
		.fold(String::with_capacity(64), |mut hex, byte| {
			let _ = write!(hex, "{byte:02x}");
			hex
		})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn checksums_are_sha256_in_hex() {
		assert_eq!(
			sha256(b"abc"),
			"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
		);
		for piece in &PIECES {
			assert_eq!(piece.sha256.len(), 64, "{}", piece.name);
			assert!(
				piece
					.sha256
					.chars()
					.all(|digit| digit.is_ascii_hexdigit() && !digit.is_ascii_uppercase())
			);
		}
	}

	#[test]
	fn every_piece_comes_from_its_maker_over_https() {
		for piece in &PIECES {
			assert!(
				piece.url.starts_with("https://incompetech.com/"),
				"{}",
				piece.name
			);
			let arguments = fetch_arguments(piece, Path::new("original.mp3"));
			assert!(
				arguments
					.windows(2)
					.any(|pair| pair == ["--proto", "=https"])
			);
			assert!(arguments.contains(&"--fail".to_owned()));
		}
	}

	#[test]
	fn encodes_vorbis_at_quality_two_changing_only_the_level() {
		let arguments = encode_arguments(&PIECES[1], Path::new("in.mp3"), Path::new("out.ogg"));

		assert!(
			arguments
				.windows(2)
				.any(|pair| pair == ["-c:a", "libvorbis"])
		);
		assert!(arguments.windows(2).any(|pair| pair == ["-q:a", "2"]));
		assert!(
			arguments
				.windows(2)
				.any(|pair| pair == ["-af", "volume=-4dB"])
		);
		assert!(
			arguments
				.windows(2)
				.any(|pair| pair == ["-map_metadata", "-1"])
		);
		assert_eq!(arguments.last().unwrap(), "out.ogg");
	}

	/// A piece changed here changes the credits, and the other way round.
	#[test]
	fn every_piece_is_credited_and_shipped() {
		let root = run::root();
		let credits = fs::read_to_string(root.join(ENCODED).join("CREDITS.md")).unwrap();

		for piece in &PIECES {
			assert!(
				credits.contains(&format!("`{}.ogg`", piece.name)),
				"{} is not in the credits",
				piece.name
			);
			assert!(
				credits.contains(&format!(
					"\"{}\" Kevin MacLeod (incompetech.com)",
					piece.title
				)),
				"{} has no credit line",
				piece.title
			);
			assert!(
				root.join(ENCODED)
					.join(format!("{}.ogg", piece.name))
					.is_file()
			);
		}

		let shipped = fs::read_dir(root.join(ENCODED))
			.unwrap()
			.filter(|entry| {
				entry
					.as_ref()
					.unwrap()
					.path()
					.extension()
					.is_some_and(|extension| extension == "ogg")
			})
			.count();
		assert_eq!(
			shipped,
			PIECES.len(),
			"a piece is shipped that is not built here"
		);
	}
}
