//! The installed host must start even when the game is played offline.
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn both_phone_hosts_connect_xr_to_the_scene_renderer() {
	for (platform, manifest) in [
		("android", include_str!("../android/Cargo.toml")),
		("ios", include_str!("../ios/Cargo.toml")),
	] {
		let manifest: toml::Value = toml::from_str(manifest).unwrap();
		let target = format!("cfg(target_os = \"{platform}\")");
		let features = manifest["target"][&target]["dependencies"]["leptoskit_xr"]["features"]
			.as_array()
			.unwrap();
		assert!(
			features
				.iter()
				.any(|feature| feature.as_str() == Some("scene3d")),
			"{platform} must install the live camera feed, not only the tracking plugin"
		);
	}
}

#[test]
fn the_native_websocket_factory_starts_with_the_shipped_options() {
	let config: toml::Value = toml::from_str(include_str!("../leptoskit.toml")).unwrap();
	let options = &config["native"]["plugin-options"]["websocket"];
	let json = serde_json::to_string(options).unwrap();
	leptoskit_plugin_websocket::from_config(&json)
		.expect("the native host's WebSocket plugin must start before the first screen");
}
