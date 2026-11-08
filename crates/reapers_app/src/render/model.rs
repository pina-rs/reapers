//! Reading the committed models, for the tests that keep the tables in step
//! with them. The game never reads a model itself: the host loads it.

use serde_json::Value;

/// The model at `path`, relative to the repository's `assets/`, as its glTF
/// JSON chunk.
pub fn gltf(path: &str) -> Value {
	let file = format!("{}/../../assets/{path}", env!("CARGO_MANIFEST_DIR"));
	let bytes = std::fs::read(&file).unwrap_or_else(|error| panic!("{file}: {error}"));
	assert_eq!(&bytes[..4], b"glTF", "{file} is a binary glTF");
	let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
	serde_json::from_slice(&bytes[20..20 + length]).unwrap()
}

fn name(node: &Value) -> String {
	node["name"].as_str().unwrap_or_default().to_owned()
}

/// Every node's name.
pub fn node_names(path: &str) -> Vec<String> {
	gltf(path)["nodes"]
		.as_array()
		.unwrap()
		.iter()
		.map(name)
		.collect()
}

/// Every animation's name.
pub fn clip_names(path: &str) -> Vec<String> {
	gltf(path)["animations"]
		.as_array()
		.map(|clips| clips.iter().map(name).collect())
		.unwrap_or_default()
}

/// Every morph target's name, from the meshes' `extras.targetNames`.
pub fn morph_names(path: &str) -> Vec<String> {
	let mut names: Vec<String> = gltf(path)["meshes"]
		.as_array()
		.unwrap()
		.iter()
		.filter_map(|mesh| mesh["extras"]["targetNames"].as_array())
		.flatten()
		.filter_map(|target| target.as_str().map(str::to_owned))
		.collect();
	names.sort();
	names.dedup();
	names
}

/// The parts drawn in each material, parents first: what
/// [`super::parts`] lists.
pub fn parts_by_material(path: &str) -> Vec<(String, Vec<String>)> {
	let gltf = gltf(path);
	let nodes = gltf["nodes"].as_array().unwrap();
	let materials = gltf["materials"].as_array().unwrap();
	let mut queue: std::collections::VecDeque<usize> = gltf["scenes"][0]["nodes"]
		.as_array()
		.unwrap()
		.iter()
		.map(|index| index.as_u64().unwrap() as usize)
		.collect();
	let mut groups: Vec<(String, Vec<String>)> = Vec::new();
	while let Some(index) = queue.pop_front() {
		let node = &nodes[index];
		if let Some(children) = node["children"].as_array() {
			queue.extend(
				children
					.iter()
					.map(|child| child.as_u64().unwrap() as usize),
			);
		}
		let Some(mesh) = node["mesh"].as_u64() else {
			continue;
		};
		let first = gltf["meshes"][mesh as usize]["primitives"]
			.as_array()
			.unwrap()
			.iter()
			.find_map(|primitive| primitive["material"].as_u64());
		let Some(material) = first else {
			continue;
		};
		let material = name(&materials[material as usize]);
		let material = material.split('.').next().unwrap_or_default().to_owned();
		match groups
			.iter_mut()
			.find(|(existing, _)| *existing == material)
		{
			Some((_, parts)) => parts.push(name(node)),
			None => groups.push((material, vec![name(node)])),
		}
	}
	groups
}
