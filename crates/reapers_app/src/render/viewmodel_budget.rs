//! The first-person budget, read from the built models. No moment of any
//! action draws more than six meshes or six thousand triangles of arms and
//! weapons. The world's effects stay near ten draws. Every part and morph
//! target the game moves is in the models, and every pivot is where the
//! code's forward kinematics put it.

use reapers_rules::Archetype;
use reapers_rules::WeaponKind;
use serde_json::Value;

use super::arms;
use super::arms::Swing;
use super::arms::ViewAction;
use super::arms::manner;
use super::model::gltf;
use super::viewmodel;
use super::viewmodel::visibility;

/// The most meshes the arms may draw at once.
const DRAWS: usize = 6;
/// The most triangles the arms may show at once.
const TRIANGLES: usize = 6000;
/// The most meshes the orb and every world effect draw, all shown at once.
const WORLD_DRAWS: usize = 11;

/// What each slot's weapon does: what every weapon does, and its own.
fn actions(archetype: Archetype) -> Vec<ViewAction> {
	let mut actions = vec![
		ViewAction::Idle,
		ViewAction::Draw,
		ViewAction::Swing { dived: false },
		ViewAction::Swing { dived: true },
		ViewAction::Strike,
		ViewAction::Parry,
		ViewAction::Struck,
		ViewAction::Raise,
		ViewAction::Throw,
	];
	actions.extend_from_slice(match archetype {
		Archetype::Gun => {
			&[
				ViewAction::Fire,
				ViewAction::Empty,
				ViewAction::Reload,
				ViewAction::ActiveReload,
				ViewAction::Jam,
			][..]
		}
		Archetype::Blade => {
			&[
				ViewAction::DiveIn,
				ViewAction::Stab {
					nth: 0,
					dived: true,
				},
				ViewAction::Stab {
					nth: 1,
					dived: true,
				},
				ViewAction::Stab {
					nth: 2,
					dived: false,
				},
				ViewAction::DiveOut,
			]
		}
		Archetype::Heavy | Archetype::Art => &[],
	});
	actions
}

/// Every slot, with the manner its weapon swings in: the heavy slot once for
/// each class weapon.
fn held() -> Vec<(Archetype, Swing)> {
	let mut held = vec![
		(Archetype::Gun, manner(Archetype::Gun, WeaponKind::Scythe)),
		(
			Archetype::Blade,
			manner(Archetype::Blade, WeaponKind::Scythe),
		),
		(Archetype::Art, manner(Archetype::Art, WeaponKind::Scythe)),
	];
	held.extend(WeaponKind::ALL.map(|heavy| (Archetype::Heavy, manner(Archetype::Heavy, heavy))));
	held
}

fn node(gltf: &Value, name: &str) -> usize {
	gltf["nodes"]
		.as_array()
		.unwrap()
		.iter()
		.position(|node| node["name"] == name)
		.unwrap_or_else(|| panic!("no node {name}"))
}

/// A mesh's draws (one for each primitive) and triangles.
fn cost(gltf: &Value, mesh: u64) -> (usize, usize) {
	let primitives = gltf["meshes"][mesh as usize]["primitives"]
		.as_array()
		.unwrap();
	let triangles = primitives
		.iter()
		.map(|primitive| {
			let accessor = primitive["indices"]
				.as_u64()
				.or_else(|| primitive["attributes"]["POSITION"].as_u64())
				.unwrap();
			gltf["accessors"][accessor as usize]["count"]
				.as_u64()
				.unwrap() as usize
				/ 3
		})
		.sum();
	(primitives.len(), triangles)
}

/// What is drawn under `index`, less what `hidden` hides: draws, triangles
/// and the meshes' names. gltfpack hangs a mesh on an unnamed node under
/// its part, so a mesh goes by the nearest name above it.
fn drawn(
	gltf: &Value,
	index: usize,
	named: &str,
	hidden: &[&str],
	into: &mut (usize, usize, Vec<String>),
) {
	let node = &gltf["nodes"][index];
	let name = node["name"]
		.as_str()
		.filter(|name| !name.is_empty())
		.unwrap_or(named);
	if hidden.contains(&name) {
		return;
	}
	if let Some(mesh) = node["mesh"].as_u64() {
		let (draws, triangles) = cost(gltf, mesh);
		into.0 += draws;
		into.1 += triangles;
		into.2.push(format!("{name} ({triangles})"));
	}
	for child in node["children"].as_array().into_iter().flatten() {
		drawn(gltf, child.as_u64().unwrap() as usize, name, hidden, into);
	}
}

#[test]
fn no_moment_of_any_action_breaks_the_budget() {
	let model = gltf("models/viewmodel.glb");
	let root = node(&model, "viewmodel");
	let mut busiest = (0, 0, Vec::new());
	for (weapon, swing) in held() {
		for action in actions(weapon) {
			let end = arms::duration(swing, action).max(0.5) + 0.2;
			for frame in 0..=(end * 60.0) as usize {
				let pose = arms::pose(swing, action, frame as f32 / 60.0, 0.0);
				let hidden: Vec<&str> = visibility(weapon, &pose)
					.into_iter()
					.filter_map(|(part, shown)| (!shown).then_some(part))
					.collect();
				let mut shown = (0, 0, Vec::new());
				drawn(&model, root, "viewmodel", &hidden, &mut shown);
				assert!(
					shown.0 <= DRAWS && shown.1 <= TRIANGLES,
					"{weapon:?} ({swing:?}) {action:?} at frame {frame} draws {} meshes, {} triangles: {:?}",
					shown.0,
					shown.1,
					shown.2
				);
				if (shown.0, shown.1) > (busiest.0, busiest.1) {
					busiest = shown;
				}
			}
		}
	}
	// The busiest moment is a reload or a throw: both arms, a weapon and
	// what the left hand carries.
	assert!(
		busiest.0 >= 5,
		"the budget is checked against something: {busiest:?}"
	);
	println!(
		"busiest: {} draws, {} triangles: {:?}",
		busiest.0, busiest.1, busiest.2
	);
}

#[test]
fn the_world_effects_stay_near_ten_draws() {
	let mut draws = 0;
	for path in ["models/viewmodel_orb.glb", "models/viewmodel_fx.glb"] {
		let model = gltf(path);
		for node in model["nodes"].as_array().unwrap() {
			if let Some(mesh) = node["mesh"].as_u64() {
				draws += cost(&model, mesh).0;
			}
		}
	}
	assert!(
		draws <= WORLD_DRAWS,
		"the orb and the effects draw {draws} meshes"
	);
}

#[test]
fn every_part_the_game_moves_is_in_the_models() {
	let arms = super::model::node_names("models/viewmodel.glb");
	let moved = [
		"viewmodel",
		"vm_right",
		"vm_wrist_r",
		"vm_spin",
		"pistol_cylinder",
		"vm_left",
	];
	let toggled = visibility(Archetype::Gun, &arms::Pose::default()).map(|(part, _)| part);
	for part in moved.into_iter().chain(toggled) {
		assert!(
			arms.iter().any(|name| name == part),
			"viewmodel.glb has no {part}"
		);
	}
	let orb = super::model::node_names("models/viewmodel_orb.glb");
	for part in ["capture_orb", "orb_body", "orb_core", "orb_sigil"] {
		assert!(
			orb.iter().any(|name| name == part),
			"viewmodel_orb.glb has no {part}"
		);
	}
	assert_eq!(
		super::model::morph_names("models/viewmodel_orb.glb"),
		["burst", "open"]
	);
	let fx = super::model::node_names("models/viewmodel_fx.glb");
	for part in [
		"fx_shade",
		"fx_halo",
		"fx_flare",
		"fx_sparks_0",
		"fx_sparks_1",
		"fx_ring",
		"fx_stream",
		"fx_tracer",
	] {
		assert!(
			fx.iter().any(|name| name == part),
			"viewmodel_fx.glb has no {part}"
		);
	}
	assert_eq!(
		super::model::morph_names("models/viewmodel_fx.glb"),
		["flow", "gather"]
	);
}

#[test]
fn every_pivot_is_where_the_arms_are_turned_about() {
	let model = gltf("models/viewmodel.glb");
	let at = |name: &str| -> [f32; 3] {
		let translation = &model["nodes"][node(&model, name)]["translation"];
		[0, 1, 2].map(|axis| translation[axis].as_f64().unwrap_or(0.0) as f32)
	};
	let pivots = [
		("vm_wrist_r", viewmodel::WRIST),
		("socket_weapon", super::turn::scale(viewmodel::WRIST, -1.0)),
		("vm_spin", viewmodel::TRIGGER),
		("pistol_cylinder", viewmodel::DRUM),
		("muzzle", viewmodel::MUZZLE),
		("socket_palm", viewmodel::PALM),
	];
	for (name, expected) in pivots {
		let found = at(name);
		assert!(
			found
				.iter()
				.zip(expected)
				.all(|(found, expected)| (found - expected).abs() < 2.0e-3),
			"{name} is at {found:?}, the code turns it about {expected:?}"
		);
	}
}
