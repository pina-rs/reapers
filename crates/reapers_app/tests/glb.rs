//! What `build.rs` keeps of a model as it mirrors it for the renderer: the
//! art build's mask colours (`COLOR_0`) go, and everything else stays byte
//! for byte, the ink weights the art build writes as `COLOR_1` above all.

#[path = "../build/glb.rs"]
mod glb;

use serde_json::Value;
use serde_json::json;

/// Three vertices, as each attribute's bytes.
const POSITIONS: [u8; 36] = {
	let mut bytes = [0; 36];
	// (0, 0, 0), (1, 0, 0), (0, 1, 0) as little-endian f32.
	bytes[14] = 0x80;
	bytes[15] = 0x3F;
	bytes[30] = 0x80;
	bytes[31] = 0x3F;
	bytes
};
const MASKS: [u8; 12] = [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
const INK_WEIGHTS: [u8; 12] = [128, 0, 0, 255, 0, 0, 0, 255, 255, 0, 0, 255];
const INDICES: [u8; 8] = [0, 0, 1, 0, 2, 0, 0, 0];

/// A one-triangle model whose primitive carries `colours`, each a set's
/// name and bytes, in a material whose base colour is not white.
fn model(colours: &[(&str, &[u8])]) -> Vec<u8> {
	let mut bin = Vec::new();
	let mut views = Vec::new();
	let mut accessors = Vec::new();
	let mut attributes = serde_json::Map::new();
	let mut slice = |bytes: &[u8], accessor: Value| {
		views.push(json!({ "buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len() }));
		bin.extend_from_slice(bytes);
		let mut accessor = accessor;
		accessor["bufferView"] = json!(views.len() - 1);
		accessors.push(accessor);
		accessors.len() - 1
	};
	let position = slice(
		&POSITIONS,
		json!({ "componentType": 5126, "count": 3, "type": "VEC3", "min": [0, 0, 0], "max": [1, 1, 0] }),
	);
	attributes.insert("POSITION".to_owned(), json!(position));
	for (name, bytes) in colours {
		let colour = slice(
			bytes,
			json!({ "componentType": 5121, "normalized": true, "count": 3, "type": "VEC4" }),
		);
		attributes.insert((*name).to_owned(), json!(colour));
	}
	let indices = slice(
		&INDICES[..6],
		json!({ "componentType": 5123, "count": 3, "type": "SCALAR" }),
	);
	while bin.len() % 4 != 0 {
		bin.push(0);
	}
	let gltf = json!({
		"asset": { "version": "2.0" },
		"buffers": [{ "byteLength": bin.len() }],
		"bufferViews": views,
		"accessors": accessors,
		"materials": [{
			"name": "cloth",
			"pbrMetallicRoughness": { "baseColorFactor": [0.2, 0.3, 0.4, 1.0] }
		}],
		"meshes": [{
			"primitives": [{ "attributes": attributes, "indices": indices, "material": 0 }]
		}],
		"nodes": [{ "mesh": 0 }],
		"scenes": [{ "nodes": [0] }],
	});
	let mut text = serde_json::to_vec(&gltf).unwrap();
	while text.len() % 4 != 0 {
		text.push(b' ');
	}
	let total = 12 + 8 + text.len() + 8 + bin.len();
	let mut glb = Vec::with_capacity(total);
	glb.extend_from_slice(b"glTF");
	glb.extend_from_slice(&2_u32.to_le_bytes());
	glb.extend_from_slice(&u32::try_from(total).unwrap().to_le_bytes());
	glb.extend_from_slice(&u32::try_from(text.len()).unwrap().to_le_bytes());
	glb.extend_from_slice(&0x4E4F_534A_u32.to_le_bytes());
	glb.extend_from_slice(&text);
	glb.extend_from_slice(&u32::try_from(bin.len()).unwrap().to_le_bytes());
	glb.extend_from_slice(&0x004E_4942_u32.to_le_bytes());
	glb.extend_from_slice(&bin);
	glb
}

/// A model's JSON and binary chunk.
fn chunks(glb: &[u8]) -> (Value, Vec<u8>) {
	let word = |at: usize| u32::from_le_bytes(glb[at..at + 4].try_into().unwrap()) as usize;
	let text = word(12);
	let bin = 20 + text;
	(
		serde_json::from_slice(&glb[20..bin]).unwrap(),
		glb[bin + 8..bin + 8 + word(bin)].to_vec(),
	)
}

/// The bytes of the attribute `name` of the model's one primitive.
fn attribute(glb: &[u8], name: &str) -> Option<Vec<u8>> {
	let (json, bin) = chunks(glb);
	let accessor = json["meshes"][0]["primitives"][0]["attributes"].get(name)?;
	let view = &json["bufferViews"]
		[json["accessors"][accessor.as_u64()? as usize]["bufferView"].as_u64()? as usize];
	let start = view["byteOffset"].as_u64()? as usize;
	Some(bin[start..start + view["byteLength"].as_u64()? as usize].to_vec())
}

#[test]
fn the_mask_colours_go_and_the_ink_weights_stay() {
	let source = model(&[("COLOR_0", &MASKS), ("COLOR_1", &INK_WEIGHTS)]);
	let drawn = glb::for_renderer(source.clone(), true);
	assert_eq!(attribute(&drawn, "COLOR_0"), None);
	assert_eq!(
		attribute(&drawn, "COLOR_1").as_deref(),
		Some(&INK_WEIGHTS[..])
	);
	assert_eq!(
		attribute(&drawn, "POSITION").as_deref(),
		Some(&POSITIONS[..])
	);
	let (json, bin) = chunks(&drawn);
	assert_eq!(bin.len() + MASKS.len(), chunks(&source).1.len());
	assert_eq!(json["accessors"].as_array().unwrap().len(), 3);
	assert_eq!(
		json["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"],
		json!([1.0, 1.0, 1.0, 1.0]),
		"a reaper's material is whitened for the print"
	);
	let indices = json["meshes"][0]["primitives"][0]["indices"]
		.as_u64()
		.unwrap();
	assert_eq!(json["accessors"][indices as usize]["count"], 3);
}

#[test]
fn a_model_without_ink_weights_loses_only_its_mask_colours() {
	let drawn = glb::for_renderer(model(&[("COLOR_0", &MASKS)]), false);
	let (json, _) = chunks(&drawn);
	let attributes = json["meshes"][0]["primitives"][0]["attributes"]
		.as_object()
		.unwrap();
	assert_eq!(attributes.keys().collect::<Vec<_>>(), ["POSITION"]);
	assert_eq!(
		attribute(&drawn, "POSITION").as_deref(),
		Some(&POSITIONS[..])
	);
	assert_eq!(
		json["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"],
		json!([0.2, 0.3, 0.4, 1.0]),
		"a husk keeps its authored material"
	);
}

#[test]
fn a_model_with_only_ink_weights_keeps_every_byte() {
	let source = model(&[("COLOR_1", &INK_WEIGHTS)]);
	let drawn = glb::for_renderer(source.clone(), false);
	assert_eq!(chunks(&drawn).1, chunks(&source).1);
	assert_eq!(
		attribute(&drawn, "COLOR_1").as_deref(),
		Some(&INK_WEIGHTS[..])
	);
}
