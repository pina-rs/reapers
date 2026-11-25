//! The bundled reapers use the named surfaces of the Flutter renderer,
//! rather than the art build's placeholder grey material.

use std::path::Path;

use serde_json::Value;

struct Glb {
	json: Value,
	bin: Vec<u8>,
}

fn read(path: &Path) -> Glb {
	let bytes = std::fs::read(path).unwrap();
	let size = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
	let bin_start = 20 + size;
	let bin_len = u32::from_le_bytes(bytes[bin_start..bin_start + 4].try_into().unwrap()) as usize;
	Glb {
		json: serde_json::from_slice(&bytes[20..20 + size]).unwrap(),
		bin: bytes[bin_start + 8..bin_start + 8 + bin_len].to_vec(),
	}
}

fn buffer_view_bytes(glb: &Glb, bv_idx: usize) -> &[u8] {
	let bv = &glb.json["bufferViews"][bv_idx];
	let (off, len) = if let Some(ext) = bv
		.get("extensions")
		.and_then(|e| e.get("EXT_meshopt_compression"))
	{
		(
			ext.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize,
			ext["byteLength"].as_u64().unwrap() as usize,
		)
	} else {
		(
			bv.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize,
			bv["byteLength"].as_u64().unwrap() as usize,
		)
	};
	&glb.bin[off..off + len]
}

fn assert_accessor_matches(
	name: &str,
	label: &str,
	src: &Glb,
	src_idx: usize,
	dst: &Glb,
	dst_idx: usize,
) {
	let sa = &src.json["accessors"][src_idx];
	let da = &dst.json["accessors"][dst_idx];
	assert_eq!(sa["componentType"], da["componentType"], "{name}/{label}");
	assert_eq!(sa["type"], da["type"], "{name}/{label}");
	assert_eq!(sa["count"], da["count"], "{name}/{label}");
	assert_eq!(sa["byteOffset"], da["byteOffset"], "{name}/{label}");
	let sbv = sa["bufferView"].as_u64().unwrap() as usize;
	let dbv = da["bufferView"].as_u64().unwrap() as usize;
	assert_eq!(
		buffer_view_bytes(src, sbv),
		buffer_view_bytes(dst, dbv),
		"{name}/{label} binary bytes diverged during compaction"
	);
}

fn assert_animations_preserved(name: &str, source: &Glb, bundled: &Glb) {
	let Some(sanims) = source.json.get("animations").and_then(Value::as_array) else {
		return;
	};
	let danims = bundled.json["animations"].as_array().unwrap();
	assert_eq!(sanims.len(), danims.len(), "{name} animation count");
	for (sa, da) in sanims.iter().zip(danims) {
		assert_eq!(sa["name"], da["name"], "{name} animation name");
		assert_eq!(sa["channels"], da["channels"], "{name} animation channels");
		for (ss, ds) in sa["samplers"]
			.as_array()
			.unwrap()
			.iter()
			.zip(da["samplers"].as_array().unwrap())
		{
			assert_eq!(ss["interpolation"], ds["interpolation"]);
			assert_accessor_matches(
				name,
				"anim.input",
				source,
				ss["input"].as_u64().unwrap() as usize,
				bundled,
				ds["input"].as_u64().unwrap() as usize,
			);
			assert_accessor_matches(
				name,
				"anim.output",
				source,
				ss["output"].as_u64().unwrap() as usize,
				bundled,
				ds["output"].as_u64().unwrap() as usize,
			);
		}
	}
}

fn assert_geometry_and_animations_preserved(name: &str, source: &Glb, bundled: &Glb) {
	// Only the mask colours are stripped: a model without them, one that
	// carries nothing but ink weights in `COLOR_1` say, keeps every byte.
	let masked = source.json["meshes"]
		.as_array()
		.unwrap()
		.iter()
		.flat_map(|mesh| mesh["primitives"].as_array().unwrap())
		.any(|primitive| primitive["attributes"].get("COLOR_0").is_some());
	if masked {
		assert!(
			bundled.bin.len() < source.bin.len(),
			"{name}: expected COLOR_0 compaction to shrink BIN ({} vs {})",
			bundled.bin.len(),
			source.bin.len()
		);
	} else {
		assert_eq!(
			bundled.bin.len(),
			source.bin.len(),
			"{name}: nothing to strip, so nothing may shrink"
		);
	}
	for (mi, (sm, dm)) in source.json["meshes"]
		.as_array()
		.unwrap()
		.iter()
		.zip(bundled.json["meshes"].as_array().unwrap())
		.enumerate()
	{
		for (pi, (sp, dp)) in sm["primitives"]
			.as_array()
			.unwrap()
			.iter()
			.zip(dm["primitives"].as_array().unwrap())
			.enumerate()
		{
			let dattrs = dp["attributes"].as_object().unwrap();
			assert!(
				!dattrs.contains_key("COLOR_0"),
				"{name} mesh {mi} prim {pi} still has COLOR_0"
			);
			for (attr, sval) in sp["attributes"].as_object().unwrap() {
				if attr == "COLOR_0" {
					continue;
				}
				let dval = dattrs
					.get(attr)
					.unwrap_or_else(|| panic!("{name} missing attribute {attr}"));
				assert_accessor_matches(
					name,
					attr,
					source,
					sval.as_u64().unwrap() as usize,
					bundled,
					dval.as_u64().unwrap() as usize,
				);
			}
			if let Some(si) = sp.get("indices").and_then(Value::as_u64) {
				let di = dp["indices"].as_u64().unwrap();
				assert_accessor_matches(name, "indices", source, si as usize, bundled, di as usize);
			}
			if let Some(stargets) = sp.get("targets").and_then(Value::as_array) {
				let dtargets = dp["targets"].as_array().unwrap();
				assert_eq!(stargets.len(), dtargets.len());
				for (st, dt) in stargets.iter().zip(dtargets) {
					for (attr, sval) in st.as_object().unwrap() {
						assert_accessor_matches(
							name,
							attr,
							source,
							sval.as_u64().unwrap() as usize,
							bundled,
							dt[attr].as_u64().unwrap() as usize,
						);
					}
				}
			}
		}
	}
	assert_animations_preserved(name, source, bundled);
}

#[test]
fn bundled_reapers_replace_placeholder_colours_and_keep_authored_geometry() {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));
	for name in [
		"reaper_harvester",
		"reaper_ferryman",
		"reaper_weigher",
		"reaper_chooser",
		"reaper_hollow",
		"reaper_mourner",
		"weapons",
	] {
		let source = read(&root.join(format!("../../assets/models/{name}.glb")));
		let bundled = read(&root.join(format!("assets/models/{name}.glb")));
		assert_geometry_and_animations_preserved(name, &source, &bundled);
		for material in bundled.json["materials"].as_array().unwrap() {
			let surface = material["name"]
				.as_str()
				.unwrap()
				.split('.')
				.next()
				.unwrap();
			let pbr = &material["pbrMetallicRoughness"];
			assert_eq!(
				pbr["baseColorFactor"],
				serde_json::json!([1.0, 1.0, 1.0, 1.0]),
				"{name}/{surface} darkens the game's tint"
			);
			let metal = matches!(surface, "metal" | "gold");
			assert_eq!(
				pbr["metallicFactor"].as_f64().unwrap(),
				if metal { 1.0 } else { 0.0 }
			);
			if metal {
				assert!(
					pbr["roughnessFactor"].as_f64().unwrap() <= 0.5,
					"{name}/{surface} lost its metal finish"
				);
			}
			if surface == "cloth" {
				assert_eq!(pbr["roughnessFactor"].as_f64().unwrap(), 0.95);
			}
		}
	}
}

#[test]
fn husks_keep_their_authored_materials() {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));
	let mut checked = 0;
	for entry in std::fs::read_dir(root.join("../../assets/models")).unwrap() {
		let source_path = entry.unwrap().path();
		let name = source_path.file_name().unwrap().to_str().unwrap();
		if !name.starts_with("husk_")
			|| !source_path
				.extension()
				.is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
		{
			continue;
		}
		let source = read(&source_path);
		let bundled = read(&root.join("assets/models").join(name));
		assert_eq!(
			source.json["materials"], bundled.json["materials"],
			"{name}"
		);
		assert_geometry_and_animations_preserved(name, &source, &bundled);
		checked += 1;
	}
	assert!(checked >= 5, "the test must read the shipped husks");
}
