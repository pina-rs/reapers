//! Builds every 3D model and NFT portrait the game ships from its Blender
//! source.
//!
//! ```sh
//! cargo xtask build-art                # build everything
//! cargo xtask build-art reapers        # build one script
//! cargo xtask build-art --preview      # also render review sheets
//! cargo xtask build-art --specs-only   # write art/build/vessels.json and motions.json only
//! ```
//!
//! The sources are the scripts in `art/blender`. Blender's authoring API is
//! Python, so those scripts are Python; this is what runs them, and it is the
//! only way models reach `assets/models`. Nothing in that folder is edited by
//! hand.
//!
//! Blender writes full-size models into `art/build/models`. Each is then
//! compressed by gltfpack into `assets/models`, with meshopt compression and
//! quantized attributes, keeping every named node, material and extra the
//! game looks things up by (`-cc -kn -km -ke`). Building the husks also
//! renders every NFT's portrait into `art/build/portraits`, which become the
//! WebP pictures in `assets/nfts`.
//!
//! Review sheets land in `art/build/previews`. Nothing under `art/build` is
//! committed.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use reapers_rules::ReaperMotion;
use reapers_rules::Vessel;
use reapers_rules::market_listings;
use serde_json::Value;
use serde_json::json;

use crate::run;
use crate::run::Failure;
use crate::run::Outcome;
use crate::run::UNAVAILABLE;
use crate::run::USAGE;

/// The Blender release the committed models were built with. A different
/// release may triangulate or export differently, so upgrade it on purpose.
pub const BLENDER_RELEASE: &str = "5.1";

/// The gltfpack the committed models were compressed with, the version its
/// models name as their generator. Take the matching release of
/// meshoptimizer (github.com/zeux/meshoptimizer/releases/tag/v0.25, the
/// `gltfpack-<platform>.zip` asset) and put `gltfpack` on PATH. Another
/// version may pack differently, so upgrade it on purpose and rebuild every
/// model.
pub const GLTFPACK_VERSION: &str = "0.25";

/// What gltfpack is asked to do: meshopt compression at its higher level,
/// keeping named nodes, materials and extras.
pub const GLTFPACK_FLAGS: [&str; 4] = ["-cc", "-kn", "-km", "-ke"];

/// The scripts, and the start of the name of every model each writes.
const SCRIPTS: [(&str, &str); 4] = [
	("reapers", "reaper_"),
	("weapons", "weapons"),
	("husks", "husk_"),
	("viewmodel", "viewmodel"),
];

const RAW_MODELS: &str = "art/build/models";
const MODELS: &str = "assets/models";
const RAW_PORTRAITS: &str = "art/build/portraits";
const PORTRAITS: &str = "assets/nfts";
const PREVIEWS: &str = "art/build/previews";
const VESSELS: &str = "art/build/vessels.json";
const MOTIONS: &str = "art/build/motions.json";

#[derive(clap::Args)]
pub struct Options {
	/// Build only these scripts: reapers, weapons, husks.
	scripts: Vec<String>,
	/// Also render review sheets into art/build/previews.
	#[arg(long)]
	preview: bool,
	/// Write the specs the scripts read, and stop.
	#[arg(long)]
	specs_only: bool,
}

pub fn main(root: &Path, options: &Options) -> Outcome {
	let scripts = chosen(&options.scripts)?;

	write_specs(root)?;

	// Writing the specs alone is enough to build one model by hand while
	// working on it: see art/blender/README.md.
	if options.specs_only {
		println!("wrote  {VESSELS}\nwrote  {MOTIONS}");

		return Ok(());
	}

	require_blender()?;
	require_gltfpack()?;

	for (script, prefix) in scripts {
		let source = root.join("art/blender").join(format!("{script}.py"));

		if !source.is_file() {
			println!("skip   {script} (no art/blender/{script}.py yet)");
			continue;
		}

		// Only what this run writes is compressed, so a model whose script
		// no longer makes it is not brought back from an old run.
		remove_matching(&root.join(RAW_MODELS), prefix, "glb")?;
		if script == "husks" {
			remove_matching(&root.join(RAW_PORTRAITS), "", "png")?;
		}

		println!("build  {script}");
		run_blender(root, &blender_arguments(script, options.preview))?;

		for model in matching(&root.join(RAW_MODELS), prefix, "glb")? {
			pack(root, &model)?;
		}
		if script == "husks" {
			for portrait in matching(&root.join(RAW_PORTRAITS), "", "png")? {
				to_webp(root, &portrait)?;
			}
		}
	}

	Ok(())
}

/// The scripts to build, from what was asked for.
fn chosen(wanted: &[String]) -> Outcome<Vec<(&'static str, &'static str)>> {
	if wanted.is_empty() {
		return Ok(SCRIPTS.to_vec());
	}

	let unknown: Vec<&str> = wanted
		.iter()
		.map(String::as_str)
		.filter(|name| !SCRIPTS.iter().any(|(script, _)| script == name))
		.collect();

	if !unknown.is_empty() {
		return Err(Failure::with_code(
			format!(
				"Unknown art script: {}\nKnown scripts: {}",
				unknown.join(", "),
				SCRIPTS.map(|(script, _)| script).join(", ")
			),
			USAGE,
		));
	}

	Ok(SCRIPTS
		.into_iter()
		.filter(|(script, _)| wanted.iter().any(|name| name == script))
		.collect())
}

/// What Blender is told to run, from the repository's root.
fn blender_arguments(script: &str, preview: bool) -> Vec<String> {
	let mut arguments: Vec<String> = [
		"--background",
		"--python-exit-code",
		"1",
		"--python",
		&format!("art/blender/{script}.py"),
		"--",
		"--out",
		RAW_MODELS,
		"--specs",
		VESSELS,
	]
	.map(ToOwned::to_owned)
	.to_vec();

	if preview {
		arguments.extend(["--preview".to_owned(), PREVIEWS.to_owned()]);
	}
	match script {
		"husks" => arguments.extend(["--portraits".to_owned(), RAW_PORTRAITS.to_owned()]),
		"reapers" => arguments.extend(["--motions".to_owned(), MOTIONS.to_owned()]),
		_ => {}
	}

	arguments
}

/// Writes the numbers the scripts shape the models to.
fn write_specs(root: &Path) -> Outcome {
	let build = root.join("art/build");

	fs::create_dir_all(&build)?;
	fs::write(root.join(MOTIONS), motions().to_string())?;
	fs::write(root.join(VESSELS), vessel_specs().to_string())?;

	Ok(())
}

/// Every reaper clip's name and timing, for the reaper script's animation.
fn motions() -> Value {
	ReaperMotion::ALL
		.iter()
		.map(|motion| {
			json!({
				"name": motion.name(),
				"seconds": motion.seconds(),
				"loops": motion.loops(),
				"unarmed": motion.unarmed(),
				"impact": motion.impact(),
			})
		})
		.collect()
}

/// The body sizes and seal positions the rules use, with the look of every
/// NFT that wears each body.
///
/// The models are shaped to these numbers rather than the other way round,
/// so a seal is always drawn exactly where the rules say a strike lands, and
/// a portrait always shows the colours the game dresses that NFT in.
fn vessel_specs() -> Value {
	let listings = market_listings();

	Vessel::ALL
		.iter()
		.map(|vessel| {
			let seals: Vec<Value> = vessel
				.seals()
				.iter()
				.map(|seal| json!({ "offset": [seal.offset.x, seal.offset.y, seal.offset.z] }))
				.collect();
			let looks: Vec<Value> = listings
				.iter()
				.filter(|nft| nft.collection.vessel() == *vessel)
				.map(|nft| {
					let mut look = serde_json::to_value(nft.look()).expect("a look is JSON");

					look["key"] = json!(nft.image_key());
					look
				})
				.collect();

			(
				vessel.name().to_owned(),
				json!({
					"gait": vessel.gait().name(),
					"halfWidth": vessel.half_width(),
					"halfHeight": vessel.half_height(),
					"seals": seals,
					"looks": looks,
				}),
			)
		})
		.collect::<serde_json::Map<_, _>>()
		.into()
}

fn require_blender() -> Outcome {
	let found = version_line("blender", &["--version"]).ok_or_else(|| {
		Failure::with_code(
			format!("Blender is not on the PATH. Install Blender {BLENDER_RELEASE}."),
			UNAVAILABLE,
		)
	})?;

	if found.contains(&format!(" {BLENDER_RELEASE}.")) {
		Ok(())
	} else {
		Err(Failure::with_code(
			format!(
				"Found \"{found}\" but the models are built with Blender {BLENDER_RELEASE}. Use that release, or update BLENDER_RELEASE in crates/xtask/src/art.rs and rebuild every model."
			),
			UNAVAILABLE,
		))
	}
}

fn require_gltfpack() -> Outcome {
	// gltfpack has no version flag: run bare, it prints its version above
	// its usage, and exits non-zero.
	let found = version_line("gltfpack", &[]).ok_or_else(|| {
		Failure::with_code(
			format!(
				"gltfpack is not on the PATH. Get gltfpack {GLTFPACK_VERSION} from meshoptimizer's release: https://github.com/zeux/meshoptimizer/releases/tag/v{GLTFPACK_VERSION}"
			),
			UNAVAILABLE,
		)
	})?;

	if is_gltfpack(&found, GLTFPACK_VERSION) {
		Ok(())
	} else {
		Err(Failure::with_code(
			format!(
				"Found \"{found}\" but the models are packed with gltfpack {GLTFPACK_VERSION}. Use that release, or update GLTFPACK_VERSION in crates/xtask/src/art.rs and rebuild every model."
			),
			UNAVAILABLE,
		))
	}
}

/// The first line a program prints, from either stream, or `None` when it
/// cannot be run at all.
fn version_line(program: &str, arguments: &[&str]) -> Option<String> {
	let output = Command::new(program).args(arguments).output().ok()?;
	let text = format!(
		"{}\n{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr)
	);

	text.lines()
		.map(str::trim)
		.find(|line| !line.is_empty())
		.map(ToOwned::to_owned)
}

/// Whether gltfpack's banner, or a model's generator, names `version`.
fn is_gltfpack(banner: &str, version: &str) -> bool {
	banner
		.strip_prefix("gltfpack ")
		.is_some_and(|rest| rest.split_whitespace().next() == Some(version))
}

/// Runs Blender, showing only the lines worth reading.
fn run_blender(root: &Path, arguments: &[String]) -> Outcome {
	let mut child = Command::new("blender")
		.args(arguments)
		.current_dir(root)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;
	let stderr = child.stderr.take().expect("piped");
	let relay = std::thread::spawn(move || {
		for line in BufReader::new(stderr).lines().map_while(Result::ok) {
			if worth_showing(&line) {
				eprintln!("{line}");
			}
		}
	});

	for line in BufReader::new(child.stdout.take().expect("piped"))
		.lines()
		.map_while(Result::ok)
	{
		if worth_showing(&line) {
			println!("{line}");
		}
	}
	let _ = relay.join();

	match child.wait()?.code() {
		Some(0) => Ok(()),
		code => {
			let code = code.unwrap_or(1);

			Err(Failure::with_code(
				format!("Blender failed on {} (exit {code}).", arguments[4]),
				run::exit_code(code),
			))
		}
	}
}

/// Blender says a great deal. These are the lines that matter: what was
/// built, and what went wrong.
fn worth_showing(line: &str) -> bool {
	["BUILT", "Error", "Traceback", "File \""]
		.iter()
		.any(|mark| line.contains(mark))
}

/// Compresses one model Blender wrote into `assets/models`, and checks the
/// result says which gltfpack made it.
fn pack(root: &Path, raw: &Path) -> Outcome {
	let name = raw.file_name().expect("a model has a name");
	let packed = root.join(MODELS).join(name);

	fs::create_dir_all(root.join(MODELS))?;

	let mut arguments = vec![
		"-i".into(),
		raw.as_os_str().to_owned(),
		"-o".into(),
		packed.as_os_str().to_owned(),
	];
	arguments.extend(GLTFPACK_FLAGS.map(Into::into));
	run::require("gltfpack", &arguments, root)?;

	let generator = glb_generator(&fs::read(&packed)?).unwrap_or_default();

	if !is_gltfpack(&generator, GLTFPACK_VERSION) {
		return Err(Failure::new(format!(
			"{} was written by \"{generator}\", not gltfpack {GLTFPACK_VERSION}.",
			packed.display()
		)));
	}

	println!(
		"pack   {} ({} KB, from {} KB)",
		Path::new(MODELS).join(name).display(),
		fs::metadata(&packed)?.len() / 1024,
		fs::metadata(raw)?.len() / 1024
	);

	Ok(())
}

/// The `asset.generator` of a binary glTF: its JSON chunk, read from the
/// header (magic, version, length, then the chunk's length and type).
fn glb_generator(glb: &[u8]) -> Option<String> {
	let word = |at: usize| {
		glb.get(at..at + 4)
			.map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
	};

	if glb.get(..4)? != b"glTF" || glb.get(16..20)? != b"JSON" {
		return None;
	}

	let length = word(12)? as usize;
	let json: Value = serde_json::from_slice(glb.get(20..20 + length)?).ok()?;

	json["asset"]["generator"].as_str().map(ToOwned::to_owned)
}

/// Turns one rendered portrait into the WebP the game shows.
fn to_webp(root: &Path, png: &Path) -> Outcome {
	let stem = png.file_stem().expect("a portrait has a name");
	let webp = root.join(PORTRAITS).join(stem).with_extension("webp");

	fs::create_dir_all(root.join(PORTRAITS))?;
	if !run::on_path("magick") {
		return Err(Failure::with_code(
			"The portraits are converted to WebP by ImageMagick 7 (`magick`), which is not on the PATH.",
			UNAVAILABLE,
		));
	}

	// Lossy at 80 with the alpha kept, and nothing but the picture: a
	// portrait is a few kilobytes and reads at thumbnail size.
	let arguments = [
		png.as_os_str(),
		"-strip".as_ref(),
		"-quality".as_ref(),
		"80".as_ref(),
		"-define".as_ref(),
		"webp:method=6".as_ref(),
		webp.as_os_str(),
	];
	run::require("magick", arguments, root)?;
	println!(
		"wrote  {}",
		Path::new(PORTRAITS)
			.join(stem)
			.with_extension("webp")
			.display()
	);

	Ok(())
}

/// The files in `directory` whose names start with `prefix` and end in
/// `.extension`, in order.
fn matching(directory: &Path, prefix: &str, extension: &str) -> Outcome<Vec<std::path::PathBuf>> {
	let Ok(entries) = fs::read_dir(directory) else {
		return Ok(Vec::new());
	};
	let mut found = Vec::new();

	for entry in entries {
		let path = entry?.path();
		let named = path
			.file_name()
			.and_then(|name| name.to_str())
			.is_some_and(|name| name.starts_with(prefix));

		if named && path.extension().is_some_and(|found| found == extension) {
			found.push(path);
		}
	}
	found.sort();

	Ok(found)
}

fn remove_matching(directory: &Path, prefix: &str, extension: &str) -> Outcome {
	for path in matching(directory, prefix, extension)? {
		fs::remove_file(path)?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn every_model_the_game_ships_comes_from_a_script() {
		let models = matching(&run::root().join(MODELS), "", "glb").unwrap();

		assert_ne!(models.len(), 0);
		for model in models {
			let name = model.file_name().unwrap().to_str().unwrap();

			assert!(
				SCRIPTS.iter().any(|(_, prefix)| name.starts_with(prefix)),
				"{name} has no script"
			);
		}
	}

	#[test]
	fn the_shipped_models_are_packed_by_the_pinned_gltfpack() {
		for model in matching(&run::root().join(MODELS), "", "glb").unwrap() {
			let generator = glb_generator(&fs::read(&model).unwrap()).unwrap();

			assert!(
				is_gltfpack(&generator, GLTFPACK_VERSION),
				"{}: {generator}",
				model.display()
			);
		}
	}

	#[test]
	fn every_nft_has_a_portrait() {
		for nft in market_listings() {
			let portrait = run::root()
				.join(PORTRAITS)
				.join(format!("{}.webp", nft.image_key()));

			assert!(portrait.is_file(), "{} has no portrait", nft.image_key());
		}
	}

	#[test]
	fn reads_gltfpack_versions() {
		assert!(is_gltfpack("gltfpack 0.25", "0.25"));
		assert!(is_gltfpack(
			"gltfpack 0.25 (built with meshoptimizer)",
			"0.25"
		));
		assert!(!is_gltfpack("gltfpack 0.24", "0.25"));
		assert!(!is_gltfpack("gltfpack 0.250", "0.25"));
		assert!(!is_gltfpack("Khronos glTF Blender I/O v5.1.0", "0.25"));
	}

	#[test]
	fn rejects_what_is_not_a_binary_gltf() {
		assert_eq!(glb_generator(b"not a model at all"), None);
		assert_eq!(glb_generator(b""), None);
	}

	#[test]
	fn chooses_scripts() {
		assert_eq!(chosen(&[]).unwrap().len(), 4);
		assert_eq!(chosen(&["husks".to_owned()]).unwrap(), [("husks", "husk_")]);
		assert_eq!(chosen(&["sculpture".to_owned()]).unwrap_err().code, USAGE);
	}

	#[test]
	fn hands_each_script_what_it_reads() {
		let husks = blender_arguments("husks", true);
		let reapers = blender_arguments("reapers", false);
		let weapons = blender_arguments("weapons", false);

		assert!(
			husks
				.windows(2)
				.any(|pair| pair == ["--portraits", RAW_PORTRAITS])
		);
		assert!(husks.windows(2).any(|pair| pair == ["--preview", PREVIEWS]));
		assert!(
			reapers
				.windows(2)
				.any(|pair| pair == ["--motions", MOTIONS])
		);
		assert!(!weapons.iter().any(|argument| argument == "--preview"));
		for arguments in [husks, reapers, weapons] {
			assert!(
				arguments
					.windows(2)
					.any(|pair| pair == ["--out", RAW_MODELS])
			);
			assert!(
				arguments
					.windows(2)
					.any(|pair| pair == ["--python-exit-code", "1"])
			);
		}
	}

	#[test]
	fn shows_what_was_built_and_what_failed() {
		assert!(worth_showing("BUILT reaper_harvester (104,211 triangles)"));
		assert!(worth_showing("Traceback (most recent call last):"));
		assert!(worth_showing("  File \"art/blender/husks.py\", line 12"));
		assert!(worth_showing("ValueError: seal 3 is buried"));
		assert!(!worth_showing("Read prefs: /home/blender/userpref.blend"));
	}

	#[test]
	fn specs_carry_every_vessel_seal_and_look() {
		let specs = vessel_specs();

		for vessel in Vessel::ALL {
			let spec = &specs[vessel.name()];

			assert_eq!(spec["halfWidth"], json!(vessel.half_width()));
			assert_eq!(
				spec["seals"].as_array().unwrap().len(),
				vessel.seals().len()
			);
		}

		let looks: usize = Vessel::ALL
			.iter()
			.map(|vessel| specs[vessel.name()]["looks"].as_array().unwrap().len())
			.sum();
		assert_eq!(looks, market_listings().len());

		let look = &specs["penguin"]["looks"][0];
		for field in [
			"key", "body", "trim", "detail", "accent", "backdrop", "trait",
		] {
			assert!(!look[field].is_null(), "a look has no {field}");
		}
	}

	#[test]
	fn motions_carry_every_clip() {
		let motions = motions();
		let motions = motions.as_array().unwrap();

		assert_eq!(motions.len(), ReaperMotion::ALL.len());
		assert_eq!(motions[0]["name"], "idle");
		assert!(
			motions
				.iter()
				.all(|motion| motion["seconds"].as_f64().unwrap() > 0.0)
		);
	}
}
