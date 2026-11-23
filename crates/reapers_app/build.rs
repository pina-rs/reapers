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

use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

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

/// Adapts material metadata to the renderer, strips `COLOR_0` attributes, and
/// compacts now-unreferenced `COLOR_0` accessors and binary slices out of the
/// GLB BIN chunk while preserving every retained geometry and animation byte.
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

	let bin_chunk_start = 20 + json_length;
	let compacted_bin = compact_bin(&mut json, &glb[bin_chunk_start..]);

	let Ok(mut chunk) = serde_json::to_vec(&json) else {
		return glb;
	};
	// Chunks are padded to four bytes, the JSON with spaces.
	while chunk.len() % 4 != 0 {
		chunk.push(b' ');
	}
	let rest: Vec<u8> = match compacted_bin {
		Some(mut bin) => {
			while bin.len() % 4 != 0 {
				bin.push(0);
			}
			let mut bin_chunk = Vec::with_capacity(8 + bin.len());
			bin_chunk.extend_from_slice(&(bin.len() as u32).to_le_bytes());
			bin_chunk.extend_from_slice(&0x004E_4942_u32.to_le_bytes());
			bin_chunk.extend_from_slice(&bin);
			bin_chunk
		}
		None => glb[bin_chunk_start..].to_vec(),
	};
	let total = 20 + chunk.len() + rest.len();
	let mut out = Vec::with_capacity(total);
	out.extend_from_slice(&glb[..8]);
	out.extend_from_slice(&(total as u32).to_le_bytes());
	out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
	out.extend_from_slice(&glb[16..20]);
	out.extend_from_slice(&chunk);
	out.extend_from_slice(&rest);
	out
}

fn collect_live_accessors(json: &serde_json::Value) -> HashSet<usize> {
	let mut live = HashSet::new();
	for mesh in json["meshes"].as_array().into_iter().flatten() {
		for prim in mesh["primitives"].as_array().into_iter().flatten() {
			for val in prim["attributes"]
				.as_object()
				.into_iter()
				.flat_map(serde_json::Map::values)
			{
				if let Some(idx) = val.as_u64() {
					live.insert(idx as usize);
				}
			}
			if let Some(idx) = prim["indices"].as_u64() {
				live.insert(idx as usize);
			}
			for target in prim["targets"].as_array().into_iter().flatten() {
				for val in target
					.as_object()
					.into_iter()
					.flat_map(serde_json::Map::values)
				{
					if let Some(idx) = val.as_u64() {
						live.insert(idx as usize);
					}
				}
			}
		}
	}
	for anim in json["animations"].as_array().into_iter().flatten() {
		for sampler in anim["samplers"].as_array().into_iter().flatten() {
			if let Some(idx) = sampler["input"].as_u64() {
				live.insert(idx as usize);
			}
			if let Some(idx) = sampler["output"].as_u64() {
				live.insert(idx as usize);
			}
		}
	}
	for skin in json["skins"].as_array().into_iter().flatten() {
		if let Some(idx) = skin["inverseBindMatrices"].as_u64() {
			live.insert(idx as usize);
		}
	}
	live
}

fn remap_references(
	json: &mut serde_json::Value,
	acc_remap: &HashMap<usize, usize>,
	bv_remap: &HashMap<usize, usize>,
) -> Option<()> {
	use serde_json::Value;

	for mesh in json["meshes"].as_array_mut().into_iter().flatten() {
		for prim in mesh["primitives"].as_array_mut().into_iter().flatten() {
			for val in prim["attributes"]
				.as_object_mut()
				.into_iter()
				.flat_map(serde_json::Map::values_mut)
			{
				*val = serde_json::json!(*acc_remap.get(&(val.as_u64()? as usize))?);
			}
			if let Some(old) = prim.get("indices").and_then(Value::as_u64) {
				prim["indices"] = serde_json::json!(*acc_remap.get(&(old as usize))?);
			}
			for target in prim
				.get_mut("targets")
				.and_then(Value::as_array_mut)
				.into_iter()
				.flatten()
			{
				for val in target
					.as_object_mut()
					.into_iter()
					.flat_map(serde_json::Map::values_mut)
				{
					*val = serde_json::json!(*acc_remap.get(&(val.as_u64()? as usize))?);
				}
			}
		}
	}
	for anim in json
		.get_mut("animations")
		.and_then(Value::as_array_mut)
		.into_iter()
		.flatten()
	{
		for sampler in anim
			.get_mut("samplers")
			.and_then(Value::as_array_mut)
			.into_iter()
			.flatten()
		{
			if let Some(old) = sampler.get("input").and_then(Value::as_u64) {
				sampler["input"] = serde_json::json!(*acc_remap.get(&(old as usize))?);
			}
			if let Some(old) = sampler.get("output").and_then(Value::as_u64) {
				sampler["output"] = serde_json::json!(*acc_remap.get(&(old as usize))?);
			}
		}
	}
	for skin in json
		.get_mut("skins")
		.and_then(Value::as_array_mut)
		.into_iter()
		.flatten()
	{
		if let Some(old) = skin.get("inverseBindMatrices").and_then(Value::as_u64) {
			skin["inverseBindMatrices"] = serde_json::json!(*acc_remap.get(&(old as usize))?);
		}
	}
	for image in json
		.get_mut("images")
		.and_then(Value::as_array_mut)
		.into_iter()
		.flatten()
	{
		if let Some(old) = image.get("bufferView").and_then(Value::as_u64) {
			image["bufferView"] = serde_json::json!(*bv_remap.get(&(old as usize))?);
		}
	}
	Some(())
}

fn collect_live_buffer_views(
	json: &serde_json::Value,
	old_accs: &[serde_json::Value],
	live_accs: &HashSet<usize>,
) -> Option<HashSet<usize>> {
	let mut live = HashSet::new();
	for &acc_idx in live_accs {
		let acc = old_accs.get(acc_idx)?;
		if let Some(bv) = acc["bufferView"].as_u64() {
			live.insert(bv as usize);
		}
		if let Some(sparse) = acc.get("sparse") {
			for key in ["indices", "values"] {
				if let Some(bv) = sparse[key]["bufferView"].as_u64() {
					live.insert(bv as usize);
				}
			}
		}
	}
	for image in json["images"].as_array().into_iter().flatten() {
		if let Some(bv) = image["bufferView"].as_u64() {
			live.insert(bv as usize);
		}
	}
	Some(live)
}

/// Removes unreferenced accessors and bufferViews left behind after stripping
/// `COLOR_0` and repacks the remaining `buffer 0` slices into a compact BIN
/// buffer (handling both plain bufferViews and `EXT_meshopt_compression`).
fn compact_bin(json: &mut serde_json::Value, rest: &[u8]) -> Option<Vec<u8>> {
	use serde_json::Value;

	let bin_len = u32::from_le_bytes(rest.get(0..4)?.try_into().ok()?) as usize;
	let bin_kind = u32::from_le_bytes(rest.get(4..8)?.try_into().ok()?);
	if bin_kind != 0x004E_4942 || rest.len() < 8 + bin_len {
		return None;
	}
	let bin_data = &rest[8..8 + bin_len];

	let live_accs = collect_live_accessors(json);
	let old_accs = json["accessors"].as_array()?;
	let old_bvs = json["bufferViews"].as_array()?;
	let live_bvs = collect_live_buffer_views(json, old_accs, &live_accs)?;

	if live_bvs.len() == old_bvs.len() && live_accs.len() == old_accs.len() {
		return None;
	}

	let mut sorted_bvs: Vec<usize> = live_bvs.into_iter().collect();
	sorted_bvs.sort_unstable();
	let bv_remap: HashMap<usize, usize> = sorted_bvs
		.iter()
		.enumerate()
		.map(|(new, &old)| (old, new))
		.collect();

	let mut sorted_accs: Vec<usize> = live_accs.into_iter().collect();
	sorted_accs.sort_unstable();
	let acc_remap: HashMap<usize, usize> = sorted_accs
		.iter()
		.enumerate()
		.map(|(new, &old)| (old, new))
		.collect();

	let mut new_bin = Vec::with_capacity(bin_data.len());
	let mut new_bvs = Vec::with_capacity(sorted_bvs.len());
	for &old_bv_idx in &sorted_bvs {
		let mut bv = old_bvs.get(old_bv_idx)?.clone();
		let target = if let Some(ext) = bv
			.get_mut("extensions")
			.and_then(|e| e.get_mut("EXT_meshopt_compression"))
			&& ext.get("buffer").and_then(Value::as_u64).unwrap_or(0) == 0
		{
			Some(ext)
		} else if bv.get("buffer").and_then(Value::as_u64) == Some(0) {
			Some(&mut bv)
		} else {
			None
		};
		if let Some(entry) = target {
			let old_off = entry.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
			let len = entry.get("byteLength")?.as_u64()? as usize;
			let slice = bin_data.get(old_off..old_off.checked_add(len)?)?;
			while new_bin.len() % 4 != 0 {
				new_bin.push(0);
			}
			entry["byteOffset"] = serde_json::json!(new_bin.len());
			new_bin.extend_from_slice(slice);
		}
		new_bvs.push(bv);
	}
	while new_bin.len() % 4 != 0 {
		new_bin.push(0);
	}

	let mut new_accs = Vec::with_capacity(sorted_accs.len());
	for &old_acc_idx in &sorted_accs {
		let mut acc = old_accs.get(old_acc_idx)?.clone();
		if let Some(bv) = acc.get("bufferView").and_then(Value::as_u64) {
			acc["bufferView"] = serde_json::json!(*bv_remap.get(&(bv as usize))?);
		}
		if let Some(sparse) = acc.get_mut("sparse") {
			for key in ["indices", "values"] {
				if let Some(bv) = sparse
					.get(key)
					.and_then(|s| s.get("bufferView"))
					.and_then(Value::as_u64)
				{
					sparse[key]["bufferView"] = serde_json::json!(*bv_remap.get(&(bv as usize))?);
				}
			}
		}
		new_accs.push(acc);
	}

	remap_references(json, &acc_remap, &bv_remap)?;
	json["bufferViews"] = Value::Array(new_bvs);
	json["accessors"] = Value::Array(new_accs);
	if let Some(buffer0) = json.get_mut("buffers").and_then(|b| b.get_mut(0)) {
		buffer0["byteLength"] = serde_json::json!(new_bin.len());
	}

	Some(new_bin)
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
				bytes = for_renderer(bytes, name.starts_with("reaper_") || name == "weapons.glb");
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
