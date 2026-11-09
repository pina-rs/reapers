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
//! The art build paints each model's
//! vertex colours as masks for the Flutter game's own shader (grime, char,
//! wear and hollowing, in the four channels), and `leptoskit_3d`, drawing
//! the model's own materials, would multiply them into the colour as
//! colours: a reaper came out blotched in red and orange. So the copy of a
//! `.glb` under `assets/models` leaves `COLOR_0` out of every primitive. The
//! JSON is all that changes; the binary chunk is copied as it is. Reapers
//! and weapons also replace the art build's placeholder surfaces with the
//! roughness and metalness of Flutter's `ReaperLook`. Their base colour is
//! white: the game's material tint supplies the whole colour, as Flutter's
//! replacement shader did. Husks keep their authored materials.

use std::fs;
use std::path::Path;

/// Each mirror: where it comes from, relative to this crate, and where it
/// goes under `assets/`.
const MIRRORS: [(&str, &str); 7] = [
	("../../assets/nfts", "nfts"),
	("../../assets/music", "music"),
	("../../assets/sfx", "sfx"),
	("../../assets/models", "models"),
	("../../assets/fonts", "fonts"),
	("../../assets/branding", "branding"),
	// Fetched by `cargo xtask fetch-depth-model`, not committed: the xr
	// plugin reads rooms with it on phones without platform depth.
	("../../assets/depth", "depth"),
];

/// Adapts material metadata to the renderer, preserving every binary byte.
fn for_renderer(glb: Vec<u8>, reaper: bool) -> Vec<u8> {
	let word = |at: usize| -> Option<usize> {
		Some(u32::from_le_bytes(glb.get(at..at + 4)?.try_into().ok()?) as usize)
	};
	let (Some(json_length), Some(json_kind)) = (word(12), word(16)) else {
		return glb;
	};
	if glb.get(..4) != Some(b"glTF") || json_kind != 0x4E4F_534A || glb.len() < 20 + json_length {
		return glb;
	}
	let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(&glb[20..20 + json_length])
	else {
		return glb;
	};
	let Some(meshes) = json["meshes"].as_array_mut() else {
		return glb;
	};
	for mesh in meshes {
		if let Some(primitives) = mesh["primitives"].as_array_mut() {
			for primitive in primitives {
				if let Some(attributes) = primitive["attributes"].as_object_mut() {
					attributes.remove("COLOR_0");
				}
			}
		}
	}
	if reaper && let Some(materials) = json["materials"].as_array_mut() {
		for material in materials {
			let name = material["name"].as_str().unwrap_or_default();
			let name = name.split('.').next().unwrap_or_default();
			let (roughness, metallic) = reaper_surface(name);
			let pbr = &mut material["pbrMetallicRoughness"];
			let alpha = pbr["baseColorFactor"][3].as_f64().unwrap_or(1.0);
			pbr["baseColorFactor"] = serde_json::json!([1.0, 1.0, 1.0, alpha]);
			pbr["roughnessFactor"] = serde_json::json!(roughness);
			pbr["metallicFactor"] = serde_json::json!(metallic);
		}
	}
	let Ok(mut chunk) = serde_json::to_vec(&json) else {
		return glb;
	};
	// Chunks are padded to four bytes, the JSON with spaces.
	while chunk.len() % 4 != 0 {
		chunk.push(b' ');
	}
	let rest = &glb[20 + json_length..];
	let total = 20 + chunk.len() + rest.len();
	let mut out = Vec::with_capacity(total);
	out.extend_from_slice(&glb[..8]);
	out.extend_from_slice(&(total as u32).to_le_bytes());
	out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
	out.extend_from_slice(&glb[16..20]);
	out.extend_from_slice(&chunk);
	out.extend_from_slice(rest);
	out
}

/// The named surfaces of Flutter's `ReaperLook`; only material metadata,
/// never the geometry, animation, morphs or the husks' own colours.
fn reaper_surface(name: &str) -> (f64, f64) {
	match name {
		"cloth" | "rope" => (0.95, 0.0),
		"cloth_alt" => (0.92, 0.0),
		"feather" => (0.55, 0.0),
		"bone" => (0.7, 0.0),
		"metal" => (0.5, 1.0),
		"gold" => (0.4, 1.0),
		"spirit" => (0.65, 0.0),
		"void" => (1.0, 0.0),
		"paper" | "wood" => (0.8, 0.0),
		_ => (0.9, 0.0),
	}
}

fn mirror(from: &Path, to: &Path) -> std::io::Result<()> {
	fs::create_dir_all(to)?;
	for entry in fs::read_dir(from)? {
		let entry = entry?;
		let source = entry.path();
		let target = to.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			mirror(&source, &target)?;
		} else {
			let mut bytes = fs::read(&source)?;
			if source
				.extension()
				.is_some_and(|extension| extension == "glb")
			{
				let name = source
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or_default();
				bytes = for_renderer(bytes, name.starts_with("reaper_") || name == "weapons.glb");
			}
			if fs::read(&target).ok() != Some(bytes.clone()) {
				fs::write(&target, bytes)?;
			}
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
