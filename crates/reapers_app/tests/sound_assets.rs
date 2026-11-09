//! The small effects bank is complete, licensed, and matches its build manifest.

use reapers_app::sound::Effect;
use sha2::Digest;
use sha2::Sha256;

#[test]
fn every_cue_has_the_recorded_asset_it_names_within_the_phone_budget() {
	let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
	let manifest: serde_json::Value =
		serde_json::from_slice(&std::fs::read(root.join("sfx/manifest.json")).unwrap()).unwrap();
	let clips = manifest["clips"].as_array().unwrap();
	assert_eq!(clips.len(), Effect::ALL.len());
	let mut bytes = 0;
	for effect in Effect::ALL {
		let file = std::fs::read(root.join(effect.asset())).unwrap();
		assert!(file.starts_with(b"OggS"));
		let name = effect.asset().strip_prefix("sfx/").unwrap();
		let clip = clips.iter().find(|clip| clip["file"] == name).unwrap();
		assert_eq!(clip["sha256"], format!("{:x}", Sha256::digest(&file)));
		assert_eq!(clip["bytes"], file.len());
		bytes += file.len();
	}
	assert!(bytes <= 256 * 1024, "effects bank grew to {bytes} bytes");
	let credits = std::fs::read_to_string(root.join("sfx/CREDITS.md")).unwrap();
	for author in [
		"Ben Jaszczak",
		"Kenney.nl",
		"rubberduck",
		"SpringySpringo",
		"CC0",
	] {
		assert!(credits.contains(author));
	}
}
