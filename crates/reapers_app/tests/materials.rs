//! The bundled reapers use the named surfaces of the Flutter renderer,
//! rather than the art build's placeholder grey material.

use std::path::Path;

use serde_json::Value;

fn read(path: &Path) -> (Value, Vec<u8>) {
	let bytes = std::fs::read(path).unwrap();
	let size = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
	(
		serde_json::from_slice(&bytes[20..20 + size]).unwrap(),
		bytes[20 + size..].to_vec(),
	)
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
		let (source, geometry) = read(&root.join(format!("../../assets/models/{name}.glb")));
		let (bundled, bundled_geometry) = read(&root.join(format!("assets/models/{name}.glb")));
		assert!(
			geometry == bundled_geometry,
			"{name}'s binary geometry changes during bundling"
		);
		assert_eq!(source["animations"], bundled["animations"]);
		for material in bundled["materials"].as_array().unwrap() {
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
		let (source, geometry) = read(&source_path);
		let (bundled, bundled_geometry) = read(&root.join("assets/models").join(name));
		assert_eq!(source["materials"], bundled["materials"], "{name}");
		assert!(geometry == bundled_geometry, "{name}");
		checked += 1;
	}
	assert!(checked >= 5, "the test must read the shipped husks");
}
