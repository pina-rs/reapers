//! The installed host must start even when the game is played offline.
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn the_native_websocket_factory_starts_with_the_shipped_options() {
	let config: toml::Value = toml::from_str(include_str!("../leptoskit.toml")).unwrap();
	let options = &config["native"]["plugin-options"]["websocket"];
	let json = serde_json::to_string(options).unwrap();
	leptoskit_plugin_websocket::from_config(&json)
		.expect("the native host's WebSocket plugin must start before the first screen");
}
