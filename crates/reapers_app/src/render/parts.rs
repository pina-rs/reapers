//! Which parts of each model are drawn in which material.
//!
//! `leptoskit_3d` changes how a part draws by its node's name, not by its
//! material's, so a look reaches a material through the parts drawn in it.
//! The models are built, never edited; `tests::every_model_s_parts_are_listed`
//! reads each one and fails, printing the table afresh, when a rebuild
//! changes them. Parents come before their children, so a part's own
//! material wins over its parent's.

/// One model's parts, by the material each is drawn in.
pub type Parts = &'static [(&'static str, &'static [&'static str])];

/// `models/husk_ape.glb`.
pub const HUSK_APE: Parts = &[
	(
		"trim",
		&[
			"chest",
			"ear_-1",
			"ear_1",
			"face",
			"foot_l",
			"foot_r",
			"fist_l",
			"fist_r",
			"jaw_lower",
			"cap_button",
		],
	),
	("glow", &["seal_0", "seal_1", "seal_2", "seal_3", "seal_4"]),
	("shadow", &["shadow"]),
	(
		"body",
		&[
			"torso",
			"skull",
			"haunch_l",
			"haunch_r",
			"forelimb_l",
			"forelimb_r",
			"brow_1",
			"brow_-1",
		],
	),
	(
		"dark",
		&[
			"mouth",
			"nostril_-1",
			"nostril_1",
			"socket_-1",
			"socket_1",
			"mouth_floor",
		],
	),
	(
		"detail",
		&[
			"teeth_upper",
			"teeth_lower",
			"cap",
			"cap_brim",
			"crown",
			"crown_band",
		],
	),
	("accent", &["eye_1", "eye_-1"]),
];

/// `models/husk_bird.glb`.
pub const HUSK_BIRD: Parts = &[
	(
		"detail",
		&[
			"feet",
			"beak_upper",
			"chain",
			"beak_lower",
			"cap",
			"cap_bill",
		],
	),
	("trim", &["front"]),
	("glow", &["seal_0", "seal_1", "seal_2", "seal_3", "seal_4"]),
	(
		"body",
		&["torso", "skull", "tail_fan", "feathers_l", "feathers_r"],
	),
	("dark", &["visor", "brow_1", "brow_-1"]),
	("accent", &["eyes.001"]),
];

/// `models/husk_llama.glb`.
pub const HUSK_LLAMA: Parts = &[
	("trim", &["blanket", "hat_band"]),
	(
		"detail",
		&[
			"blanket_stripe_0",
			"blanket_stripe_1",
			"blanket_stripe_2",
			"hoof_bl",
			"hoof_br",
			"hoof_fl",
			"hoof_fr",
			"neckerchief",
			"neckerchief_knot",
			"hat",
		],
	),
	("glow", &["seal_0", "seal_1", "seal_2", "seal_3", "seal_4"]),
	("shadow", &["shadow"]),
	(
		"body",
		&[
			"torso",
			"leg_wool_bl",
			"shin_bl",
			"leg_wool_br",
			"shin_br",
			"leg_wool_fl",
			"shin_fl",
			"leg_wool_fr",
			"shin_fr",
			"neck_wool",
			"tail_tuft",
			"ear_-1",
			"ear_1",
			"skull",
			"jaw_lower",
		],
	),
	(
		"dark",
		&[
			"ear_hollow_-1",
			"ear_hollow_1",
			"mouth",
			"nostril_-1",
			"nostril_1",
			"socket_-1",
			"socket_1",
			"brow_1",
			"brow_-1",
		],
	),
	("accent", &["eye_1", "eye_-1"]),
];

/// `models/husk_penguin.glb`.
pub const HUSK_PENGUIN: Parts = &[
	("trim", &["belly", "face", "beanie_band", "beanie_bobble"]),
	("glow", &["seal_0", "seal_1", "seal_2", "seal_3", "seal_4"]),
	("shadow", &["shadow"]),
	(
		"body",
		&[
			"torso",
			"flipper_l",
			"flipper_r",
			"skull",
			"tail_fan",
			"brow_1",
			"brow_-1",
		],
	),
	(
		"detail",
		&[
			"beak_upper",
			"foot_l",
			"foot_r",
			"beak_lower",
			"beanie",
			"crown",
			"crown_band",
		],
	),
	("dark", &["socket_-1", "socket_1"]),
	("accent", &["eye_1", "eye_-1"]),
];

/// `models/husk_reliquaryStack.glb`.
pub const HUSK_RELIQUARYSTACK: Parts = &[
	("glow", &["seam_0", "seam_1", "seam_2", "seam_3"]),
	(
		"frame",
		&["block_0", "block_1", "block_2", "block_3", "block_4"],
	),
	(
		"canvas",
		&[
			"panel_0_0",
			"panel_0_1",
			"panel_0_2",
			"panel_0_3",
			"panel_1_0",
			"panel_1_1",
			"panel_1_2",
			"panel_1_3",
			"panel_2_0",
			"panel_2_1",
			"panel_2_2",
			"panel_2_3",
			"panel_3_0",
			"panel_3_1",
			"panel_3_2",
			"panel_3_3",
			"panel_4_0",
			"panel_4_1",
			"panel_4_2",
			"panel_4_3",
		],
	),
];

/// `models/husk_robot.glb`.
pub const HUSK_ROBOT: Parts = &[
	("glow", &["seal_0", "seal_1", "seal_2", "seal_3", "seal_4"]),
	("shadow", &["shadow"]),
	(
		"body",
		&[
			"torso_body",
			"claw_arm_body",
			"clamp_arm_body",
			"skull_body",
			"piston_leg_body",
			"drum_leg_body",
			"brow_plate_1_body",
			"brow_plate_-1_body",
		],
	),
	("dark", &["torso_dark", "skull_dark", "mouth_dark"]),
	(
		"detail",
		&[
			"torso_detail",
			"claw_arm_detail",
			"clamp_arm_detail",
			"skull_detail",
			"piston_leg_detail",
			"drum_leg_detail",
			"mouth_detail",
			"antenna_detail",
			"crest_detail",
		],
	),
	(
		"trim",
		&[
			"torso_trim",
			"claw_arm_trim",
			"clamp_arm_trim",
			"skull_trim",
			"piston_leg_trim",
			"drum_leg_trim",
			"mouth_trim",
			"antenna_trim",
			"crest_trim",
		],
	),
	("accent", &["eye_light_1_accent", "eye_light_-1_accent"]),
];

/// `models/reaper_chooser.glb`.
pub const REAPER_CHOOSER: Parts = &[
	(
		"cloth",
		&[
			"robe_0",
			"robe_1",
			"robe_2",
			"robe_3",
			"arm_l_upper_sleeve",
			"arm_r_upper_sleeve",
			"arm_l_lower_sleeve",
			"arm_r_lower_sleeve",
			"boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	("feather", &["wing_0", "wing_1", "wing_2", "crest"]),
	(
		"cloth_alt",
		&[
			"wing_0_threads",
			"wing_1_threads",
			"wing_2_threads",
			"wrap_0",
			"wrap_1",
			"wrap_2",
		],
	),
	("accent", &["face_0", "face_1", "face_2"]),
	("metal", &["helm_0", "helm_1", "helm_2"]),
	("void", &["void_0", "void_1", "void_2"]),
	("bone", &["arm_l_hand", "arm_r_hand"]),
];

/// `models/reaper_ferryman.glb`.
pub const REAPER_FERRYMAN: Parts = &[
	(
		"cloth",
		&[
			"robe_0",
			"robe_1",
			"robe_2",
			"robe_3",
			"arm_l_upper_sleeve",
			"arm_r_upper_sleeve",
			"arm_l_lower_sleeve",
			"arm_r_lower_sleeve",
			"boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	("rope", &["rope_coil", "half_cape_line", "net", "fares"]),
	("cloth_alt", &["half_cape", "cowl_0", "cowl_1", "cowl_2"]),
	("wood", &["net_floats"]),
	("accent", &["face_0", "face_1", "face_2"]),
	("void", &["void_0", "void_1", "void_2"]),
	("bone", &["arm_l_hand", "arm_r_hand"]),
];

/// `models/reaper_harvester.glb`.
pub const REAPER_HARVESTER: Parts = &[
	(
		"cloth",
		&[
			"robe_0",
			"robe_1",
			"robe_2",
			"robe_3",
			"arm_l_upper_sleeve",
			"arm_r_upper_sleeve",
			"arm_l_lower_sleeve",
			"arm_r_lower_sleeve",
			"boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	(
		"cloth_alt",
		&[
			"capelet",
			"collar",
			"collar_tail_0",
			"collar_tail_1",
			"hat_0",
			"wrap_0",
			"hat_1",
			"wrap_1",
			"hat_2",
			"wrap_2",
		],
	),
	("rope", &["sheaf", "strap", "band_0", "band_1"]),
	("accent", &["face_0", "face_1", "face_2"]),
	("void", &["void_0", "void_1", "void_2"]),
	("wood", &["ribs"]),
	("bone", &["arm_l_hand", "arm_r_hand"]),
];

/// `models/reaper_hollow.glb`.
pub const REAPER_HOLLOW: Parts = &[
	(
		"cloth",
		&[
			"robe_0",
			"robe_1",
			"robe_2",
			"robe_3",
			"arm_l_upper_sleeve",
			"arm_r_upper_sleeve",
			"arm_l_lower_sleeve",
			"arm_r_lower_sleeve",
			"boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	(
		"metal",
		&[
			"arm_l_spaulder",
			"arm_r_spaulder",
			"pauldron",
			"strap_buckle",
			"cape_clasp",
			"chains",
			"neck_ring",
			"lantern_chain",
			"lantern_stirrup",
			"lantern_harness",
		],
	),
	("rope", &["strap"]),
	(
		"cloth_alt",
		&[
			"cape",
			"arm_l_cuff",
			"arm_r_cuff",
			"collar_0",
			"collar_1",
			"collar_2",
		],
	),
	("accent", &["eyes_0", "eyes_1", "eyes_2", "face_0"]),
	("void", &["void_0", "void_1", "void_2"]),
	("bone", &["arm_l_hand", "arm_r_hand", "lantern_skull"]),
];

/// `models/reaper_mourner.glb`.
pub const REAPER_MOURNER: Parts = &[
	(
		"cloth",
		&[
			"robe_0", "robe_1", "robe_2", "robe_3", "veil_0", "veil_1", "veil_2", "boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	(
		"bone",
		&[
			"arm_l_upper",
			"arm_r_upper",
			"arm_l_forearm",
			"arm_r_forearm",
			"arm_l_hand",
			"arm_r_hand",
		],
	),
	(
		"cloth_alt",
		&[
			"veil_over",
			"shawl",
			"shawl_knot",
			"shawl_tail_0",
			"shawl_tail_1",
		],
	),
	("metal", &["cord_weights", "circlet_0"]),
	("rope", &["cords"]),
	("accent", &["face_0", "face_1", "face_2"]),
	("void", &["void_0", "void_1", "void_2"]),
];

/// `models/reaper_weigher.glb`.
pub const REAPER_WEIGHER: Parts = &[
	(
		"cloth",
		&[
			"robe_0",
			"robe_1",
			"robe_2",
			"robe_3",
			"arm_l_upper_sleeve",
			"arm_r_upper_sleeve",
			"arm_l_lower_sleeve",
			"arm_r_lower_sleeve",
			"wrap_0",
			"wrap_1",
			"wrap_2",
			"boot_l",
			"boot_r",
		],
	),
	(
		"spirit",
		&[
			"pelvis_body",
			"spirit_body",
			"spirit_wisp_0",
			"spirit_wisp_1",
			"spirit_wisp_2",
			"thigh_l",
			"thigh_r",
			"spirit_arm_l_upper",
			"spirit_arm_r_upper",
			"patella_l",
			"shin_l",
			"patella_r",
			"shin_r",
			"spirit_arm_l_lower",
			"spirit_arm_r_lower",
		],
	),
	("cloth_alt", &["stole", "winding", "winding_tail"]),
	(
		"gold",
		&[
			"stole_weights",
			"collar",
			"collar_counterweight",
			"crest_0",
			"mask_0",
			"crest_1",
			"mask_1",
			"crest_2",
			"mask_2",
		],
	),
	("accent", &["face_0", "face_1", "face_2"]),
	("void", &["void_0", "void_1", "void_2"]),
	("bone", &["arm_l_hand", "arm_r_hand"]),
];

/// `models/weapons.glb`.
pub const WEAPONS: Parts = &[
	(
		"metal",
		&[
			"axe_butt",
			"axe_eye",
			"axe_head",
			"axe_spike",
			"flail_chains_risen",
			"flail_chains_sunk",
			"bell_body",
			"bell_yoke",
			"whip_barb",
			"whip_pommel",
			"whip_socket",
			"whip_spine",
			"greatsword_blade",
			"greatsword_guard",
			"greatsword_pommel",
			"lantern_pole_collar",
			"lantern_pole_crook",
			"lantern_pole_hook",
			"lantern_pole_lantern",
			"lantern_pole_shoe",
			"scythe_blade",
			"scythe_collar",
			"scythe_hook",
			"spear_blade",
			"spear_butt",
			"spear_socket",
		],
	),
	(
		"accent",
		&[
			"axe_edge",
			"flail_pin",
			"bell_light",
			"whip_embers",
			"greatsword_edge_l",
			"greatsword_edge_r",
			"greatsword_fuller_b",
			"greatsword_fuller_f",
			"lantern_pole_flame",
			"scythe_edge",
			"spear_edge",
		],
	),
	(
		"rope",
		&[
			"axe_grip",
			"axe_lashing",
			"flail_grip",
			"whip_grip",
			"greatsword_wrap",
			"lantern_pole_grip",
			"lantern_pole_lashing",
			"scythe_grip",
			"scythe_lashing",
			"spear_collar",
			"spear_grip",
		],
	),
	(
		"wood",
		&[
			"axe_haft",
			"flail_haft",
			"bell_clapper",
			"bell_handle",
			"whip_handle",
			"greatsword_grip",
			"lantern_pole_haft",
			"scythe_haft",
			"spear_haft",
		],
	),
	(
		"gold",
		&[
			"flail_beam",
			"flail_foot",
			"flail_pan_risen",
			"flail_pan_sunk",
			"flail_pivot",
			"flail_socket",
		],
	),
	("feather", &["spear_feathers"]),
	("cloth_alt", &["spear_threads"]),
];

/// The parts of the model at `path`, under `assets/`.
pub fn parts(path: &str) -> Parts {
	match path {
		"models/husk_ape.glb" => HUSK_APE,
		"models/husk_bird.glb" => HUSK_BIRD,
		"models/husk_llama.glb" => HUSK_LLAMA,
		"models/husk_penguin.glb" => HUSK_PENGUIN,
		"models/husk_reliquaryStack.glb" => HUSK_RELIQUARYSTACK,
		"models/husk_robot.glb" => HUSK_ROBOT,
		"models/reaper_chooser.glb" => REAPER_CHOOSER,
		"models/reaper_ferryman.glb" => REAPER_FERRYMAN,
		"models/reaper_harvester.glb" => REAPER_HARVESTER,
		"models/reaper_hollow.glb" => REAPER_HOLLOW,
		"models/reaper_mourner.glb" => REAPER_MOURNER,
		"models/reaper_weigher.glb" => REAPER_WEIGHER,
		"models/weapons.glb" => WEAPONS,
		_ => &[],
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::render::model::parts_by_material;

	const MODELS: [&str; 13] = [
		"models/husk_ape.glb",
		"models/husk_bird.glb",
		"models/husk_llama.glb",
		"models/husk_penguin.glb",
		"models/husk_reliquaryStack.glb",
		"models/husk_robot.glb",
		"models/reaper_chooser.glb",
		"models/reaper_ferryman.glb",
		"models/reaper_harvester.glb",
		"models/reaper_hollow.glb",
		"models/reaper_mourner.glb",
		"models/reaper_weigher.glb",
		"models/weapons.glb",
	];

	#[test]
	fn every_model_s_parts_are_listed() {
		for path in MODELS {
			let listed: Vec<(String, Vec<String>)> = parts(path)
				.iter()
				.map(|(material, parts)| {
					(
						(*material).to_owned(),
						parts.iter().map(|part| (*part).to_owned()).collect(),
					)
				})
				.collect();
			assert_eq!(
				listed,
				parts_by_material(path),
				"{path} changed: list its parts again from the test's reading"
			);
		}
	}
}
