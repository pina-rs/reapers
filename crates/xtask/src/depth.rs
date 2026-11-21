//! Fetches the depth model the phone apps read rooms with when the phone
//! has no depth of its own.
//!
//! ```sh
//! cargo xtask fetch-depth-model
//! ```
//!
//! The model is Depth Anything V2 Small, the metric indoor fine-tune,
//! exported to ONNX with 8-bit weights: the file the Flutter game shipped
//! and `leptoskit_depth` pins. It is Apache-2.0, and its licence travels
//! with it. At 27 MB it is not committed: this fetches it, with its licence,
//! from the repository's `depth-model-v1` release into `assets/depth`, checks
//! both against known checksums, and `crates/reapers_app/build.rs` mirrors
//! them into the app's bundle. A build without them still runs; phones
//! without platform depth then read no room. See docs/environment.md.

use std::fs;
use std::path::Path;

use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::UNAVAILABLE;
use crate::run::sha256;

/// One file of the release.
struct File {
	name: &'static str,
	sha256: &'static str,
}

const RELEASE: &str = "https://github.com/pina-rs/reapers/releases/download/depth-model-v1";
const DIRECTORY: &str = "assets/depth";

/// The model, at the digest `leptoskit_depth::MODEL` pins, and its licence.
const FILES: [File; 2] = [
	File {
		name: "depth_anything_v2_metric_indoor_small.onnx",
		sha256: "b81c4fae8985bd73d5026ffc1ebf545ff01d73fbf4d534a06c811abfcadb6dbd",
	},
	File {
		name: "LICENSE.txt",
		sha256: "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4",
	},
];

pub fn main(root: &Path) -> Outcome {
	if !run::on_path("curl") {
		return Err(Failure::with_code(
			"Fetching the depth model needs curl, which is not on the PATH.",
			UNAVAILABLE,
		));
	}

	fs::create_dir_all(root.join(DIRECTORY))?;

	for file in &FILES {
		let path = Path::new(DIRECTORY).join(file.name);
		let full = root.join(&path);

		if full.is_file() && sha256(&fs::read(&full)?) == file.sha256 {
			println!("have   {}", path.display());
			continue;
		}

		println!("fetching {}", file.name);
		let part = Path::new(DIRECTORY).join(format!("{}.part", file.name));
		let full_part = root.join(&part);
		run::require("curl", fetch_arguments(file, &part), root)?;
		let checksum = sha256(&fs::read(&full_part)?);

		if checksum != file.sha256 {
			let _ = fs::remove_file(&full_part);
			return Err(Failure::new(format!(
				"{} is not the file the game pins: its checksum is {checksum}. It has been deleted.",
				path.display()
			)));
		}

		fs::rename(&full_part, &full)?;
		println!("wrote  {}", path.display());
	}

	Ok(())
}

fn fetch_arguments(file: &File, path: &Path) -> Vec<String> {
	vec![
		"--fail".to_owned(),
		"--location".to_owned(),
		"--silent".to_owned(),
		"--show-error".to_owned(),
		"--proto".to_owned(),
		"=https".to_owned(),
		"--output".to_owned(),
		path.display().to_string(),
		format!("{RELEASE}/{}", file.name),
	]
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn every_file_comes_from_the_release_over_https() {
		for file in &FILES {
			let arguments = fetch_arguments(file, Path::new("out"));
			assert!(arguments.contains(&"=https".to_owned()));
			assert!(arguments.last().is_some_and(|url| {
				url.starts_with("https://github.com/pina-rs/reapers/releases/download/")
			}));
			assert_eq!(file.sha256.len(), 64, "{}", file.name);
		}
	}
}
