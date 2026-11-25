//! What every shipped model costs, and the budgets it must stay inside.
//!
//! ```sh
//! cargo xtask build-art --report
//! ```
//!
//! Reads each packed model in `assets/models` (and, when a build has left
//! it, its full-size original in `art/build/models`) and prints its
//! triangles, vertices, draw calls, materials, morph targets, clips and
//! bytes. It fails when a model is over its budget, and so does every build.
//!
//! The counts are of the model as if every node were shown: a reaper class
//! file holds every variant, of which the game shows one at a time. A mesh
//! drawn by two nodes counts twice.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde_json::Value;

use super::MODELS;
use super::RAW_MODELS;
use super::glb_json;
use super::matching;
use crate::run::Failure;
use crate::run::Outcome;

/// What one weapon may hold: the guideline in art/blender/README.md.
const WEAPON_TRIANGLES: u64 = 6_000;
/// What a creature may hold, which `lib/creature.py` also enforces.
const HUSK_TRIANGLES: u64 = 14_000;
/// The capture orb and the first-person lights, which `viewmodel.py` also
/// enforces.
const ORB_TRIANGLES: u64 = 3_000;
const FX_TRIANGLES: u64 = 1_500;
const KIB: u64 = 1024;

/// What one shipped model may cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
	/// The model's file name in `assets/models`.
	pub model: &'static str,
	/// Triangles, with every node shown.
	pub triangles: u64,
	/// Bytes of the packed file.
	pub bytes: u64,
}

/// Every shipped model's budget.
///
/// Triangles are the README's guidelines where today's models meet them.
/// Where a model is already over, the ceiling is what it measured when the
/// budgets were set (2026-11), so it can only get better: lower it when a
/// model is slimmed, and never raise it to let one grow.
///
/// No size guideline was ever written down, so every byte ceiling is the
/// packed size measured then, rounded up to the next 8 KiB: gltfpack's
/// output moves by about a hundred bytes between builds of the same model.
pub const BUDGETS: [Budget; 16] = [
	// The README asks for about 110,000 triangles a class file, every
	// variant included. Every class is over it, so these are the measured
	// ceilings: bring them down toward 110,000, never up.
	Budget {
		model: "reaper_chooser.glb",
		triangles: 138_158,
		bytes: 2_344 * KIB,
	},
	Budget {
		model: "reaper_ferryman.glb",
		triangles: 133_358,
		bytes: 2_264 * KIB,
	},
	Budget {
		model: "reaper_harvester.glb",
		triangles: 142_204,
		bytes: 2_384 * KIB,
	},
	Budget {
		model: "reaper_hollow.glb",
		triangles: 137_378,
		bytes: 2_376 * KIB,
	},
	Budget {
		model: "reaper_mourner.glb",
		triangles: 134_796,
		bytes: 2_288 * KIB,
	},
	Budget {
		model: "reaper_weigher.glb",
		triangles: 129_450,
		bytes: 2_256 * KIB,
	},
	Budget {
		model: "weapons.glb",
		triangles: 8 * WEAPON_TRIANGLES,
		bytes: 784 * KIB,
	},
	Budget {
		model: "husk_ape.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 304 * KIB,
	},
	Budget {
		model: "husk_bird.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 256 * KIB,
	},
	Budget {
		model: "husk_llama.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 224 * KIB,
	},
	Budget {
		model: "husk_penguin.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 232 * KIB,
	},
	Budget {
		model: "husk_reliquaryStack.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 112 * KIB,
	},
	Budget {
		model: "husk_robot.glb",
		triangles: HUSK_TRIANGLES,
		bytes: 280 * KIB,
	},
	// No guideline for the whole first-person file: `viewmodel.py` holds
	// each moment to 6,000. Measured ceiling.
	Budget {
		model: "viewmodel.glb",
		triangles: 10_170,
		bytes: 56 * KIB,
	},
	Budget {
		model: "viewmodel_fx.glb",
		triangles: FX_TRIANGLES,
		bytes: 16 * KIB,
	},
	Budget {
		model: "viewmodel_orb.glb",
		triangles: ORB_TRIANGLES,
		bytes: 32 * KIB,
	},
];

/// What each weapon may hold, by its node: the README's guideline, but for
/// the lantern pole, which is already over it and has its measured ceiling.
pub fn weapon_budget(node: &str) -> u64 {
	if node == "weapon_lanternPole" {
		6_328
	} else {
		WEAPON_TRIANGLES
	}
}

/// What a model holds, counted from its glTF JSON.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts {
	pub triangles: u64,
	pub vertices: u64,
	/// One per primitive per node that draws it.
	pub draws: u64,
	pub materials: usize,
	/// Every mesh's morph targets, added up.
	pub morph_targets: usize,
	pub clips: usize,
	/// The triangles under each node named `weapon_<kind>`.
	pub weapons: Vec<(String, u64)>,
}

/// Counts what a glTF document holds, walking its scene from the root nodes
/// so a mesh drawn twice counts twice.
pub fn count(gltf: &Value) -> Counts {
	let nodes = gltf["nodes"]
		.as_array()
		.map(Vec::as_slice)
		.unwrap_or_default();
	let scene = gltf["scene"].as_u64().unwrap_or(0) as usize;
	let roots: Vec<usize> = gltf["scenes"][scene]["nodes"]
		.as_array()
		.map(|roots| roots.iter().filter_map(index).collect())
		.unwrap_or_default();
	let mut counts = Counts {
		materials: gltf["materials"].as_array().map_or(0, Vec::len),
		clips: gltf["animations"].as_array().map_or(0, Vec::len),
		morph_targets: gltf["meshes"]
			.as_array()
			.map(|meshes| {
				meshes
					.iter()
					.map(|mesh| {
						mesh["primitives"][0]["targets"]
							.as_array()
							.map_or(0, Vec::len)
					})
					.sum()
			})
			.unwrap_or_default(),
		..Counts::default()
	};

	for root in roots {
		walk(gltf, nodes, root, &mut counts);
	}

	counts
}

/// Adds what node `at` and everything under it draws to `counts`, and
/// returns the triangles under it.
fn walk(gltf: &Value, nodes: &[Value], at: usize, counts: &mut Counts) -> u64 {
	let Some(node) = nodes.get(at) else {
		return 0;
	};
	let mut triangles = 0;

	if let Some(mesh) = node["mesh"].as_u64() {
		for primitive in gltf["meshes"][mesh as usize]["primitives"]
			.as_array()
			.map(Vec::as_slice)
			.unwrap_or_default()
		{
			let vertices = accessor_count(gltf, &primitive["attributes"]["POSITION"]);
			let corners = match primitive.get("indices") {
				Some(indices) => accessor_count(gltf, indices),
				None => vertices,
			};

			triangles += match primitive["mode"].as_u64().unwrap_or(4) {
				4 => corners / 3,
				5 | 6 => corners.saturating_sub(2),
				_ => 0,
			};
			counts.vertices += vertices;
			counts.draws += 1;
		}
	}

	counts.triangles += triangles;

	// Children add their own share to `counts`; only the total is needed
	// here, for a weapon's subtree.
	for child in node["children"]
		.as_array()
		.map(Vec::as_slice)
		.unwrap_or_default()
		.iter()
		.filter_map(index)
	{
		triangles += walk(gltf, nodes, child, counts);
	}

	if let Some(name) = node["name"]
		.as_str()
		.filter(|name| name.starts_with("weapon_"))
	{
		counts.weapons.push((name.to_owned(), triangles));
	}

	triangles
}

fn index(value: &Value) -> Option<usize> {
	value.as_u64().map(|index| index as usize)
}

fn accessor_count(gltf: &Value, accessor: &Value) -> u64 {
	accessor
		.as_u64()
		.and_then(|accessor| gltf["accessors"][accessor as usize]["count"].as_u64())
		.unwrap_or(0)
}

/// One shipped model, measured.
pub struct Measured {
	pub model: String,
	pub counts: Counts,
	pub packed_bytes: u64,
	/// The full-size model a build left in `art/build/models`, if any.
	pub raw_bytes: Option<u64>,
}

/// Measures every shipped model.
pub fn measure(root: &Path) -> Outcome<Vec<Measured>> {
	let mut measured = Vec::new();

	for path in matching(&root.join(MODELS), "", "glb")? {
		let model = path
			.file_name()
			.and_then(|name| name.to_str())
			.expect("a model's name is UTF-8")
			.to_owned();
		let bytes = fs::read(&path)?;
		let gltf = glb_json(&bytes)
			.ok_or_else(|| Failure::new(format!("{} is not a binary glTF.", path.display())))?;
		let raw_bytes = fs::metadata(root.join(RAW_MODELS).join(&model))
			.ok()
			.map(|raw| raw.len());

		measured.push(Measured {
			counts: count(&gltf),
			packed_bytes: bytes.len() as u64,
			raw_bytes,
			model,
		});
	}

	Ok(measured)
}

/// What is wrong with the measured models: each one over a budget, and
/// each one with no budget at all.
pub fn problems(measured: &[Measured]) -> Vec<String> {
	let mut problems = Vec::new();

	for model in measured {
		let Some(budget) = BUDGETS.iter().find(|budget| budget.model == model.model) else {
			problems.push(format!(
				"{} has no budget: add one to BUDGETS in crates/xtask/src/art/report.rs.",
				model.model
			));
			continue;
		};

		if model.counts.triangles > budget.triangles {
			problems.push(format!(
				"{} has {} triangles; its budget is {}.",
				model.model,
				thousands(model.counts.triangles),
				thousands(budget.triangles)
			));
		}
		if model.packed_bytes > budget.bytes {
			problems.push(format!(
				"{} is {} bytes packed; its budget is {}.",
				model.model,
				thousands(model.packed_bytes),
				thousands(budget.bytes)
			));
		}
		for (weapon, triangles) in &model.counts.weapons {
			let allowed = weapon_budget(weapon);

			if *triangles > allowed {
				problems.push(format!(
					"{weapon} in {} has {} triangles; its budget is {}.",
					model.model,
					thousands(*triangles),
					thousands(allowed)
				));
			}
		}
	}

	problems
}

/// The report, as a table.
pub fn table(measured: &[Measured]) -> String {
	let mut table = format!(
		"{:<26}{:>10}{:>10}{:>7}{:>6}{:>8}{:>7}{:>11}{:>11}{:>11}{:>11}\n",
		"model",
		"triangles",
		"vertices",
		"draws",
		"mats",
		"morphs",
		"clips",
		"raw KB",
		"packed KB",
		"tri budget",
		"KB budget"
	);

	for model in measured {
		let budget = BUDGETS.iter().find(|budget| budget.model == model.model);
		let counts = &model.counts;
		let _ = writeln!(
			table,
			"{:<26}{:>10}{:>10}{:>7}{:>6}{:>8}{:>7}{:>11}{:>11}{:>11}{:>11}",
			model.model,
			thousands(counts.triangles),
			thousands(counts.vertices),
			counts.draws,
			counts.materials,
			counts.morph_targets,
			counts.clips,
			model
				.raw_bytes
				.map_or_else(|| "-".to_owned(), |bytes| thousands(bytes / 1024)),
			thousands(model.packed_bytes / 1024),
			budget.map_or_else(|| "none".to_owned(), |budget| thousands(budget.triangles)),
			budget.map_or_else(
				|| "none".to_owned(),
				|budget| thousands(budget.bytes / 1024)
			),
		);

		for (weapon, triangles) in &counts.weapons {
			let _ = writeln!(
				table,
				"  {:<24}{:>10}{:>74}",
				weapon,
				thousands(*triangles),
				thousands(weapon_budget(weapon))
			);
		}
	}

	let packed: u64 = measured.iter().map(|model| model.packed_bytes).sum();
	let raw: Option<u64> = measured.iter().map(|model| model.raw_bytes).sum();
	let _ = writeln!(
		table,
		"{:<26}{:>59}{:>11}",
		"all",
		raw.map_or_else(|| "-".to_owned(), |bytes| thousands(bytes / 1024)),
		thousands(packed / 1024)
	);

	table
}

/// `12345678` as `12,345,678`.
fn thousands(value: u64) -> String {
	let digits = value.to_string();
	let mut grouped = String::new();

	for (index, digit) in digits.chars().enumerate() {
		if index > 0 && (digits.len() - index) % 3 == 0 {
			grouped.push(',');
		}
		grouped.push(digit);
	}

	grouped
}

/// Prints the report, and fails when a model is over its budget.
pub fn main(root: &Path) -> Outcome {
	let measured = measure(root)?;

	print!("{}", table(&measured));

	check(&measured)
}

/// Fails, listing every problem, when any model is over its budget.
pub fn check(measured: &[Measured]) -> Outcome {
	let problems = problems(measured);

	if problems.is_empty() {
		Ok(())
	} else {
		Err(Failure::new(format!(
			"Over budget:\n  {}\nSlim the model, or, if it must grow, change its budget in crates/xtask/src/art/report.rs on purpose.",
			problems.join("\n  ")
		)))
	}
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;
	use crate::run;

	#[test]
	fn every_shipped_model_is_within_its_budget() {
		let measured = measure(&run::root()).unwrap();

		assert_ne!(measured.len(), 0);
		assert_eq!(problems(&measured), Vec::<String>::new());
	}

	#[test]
	fn every_budget_is_for_a_shipped_model() {
		let measured = measure(&run::root()).unwrap();

		for budget in BUDGETS {
			assert!(
				measured.iter().any(|model| model.model == budget.model),
				"{} has a budget but is not in {MODELS}",
				budget.model
			);
		}
	}

	#[test]
	fn the_weapons_file_holds_all_eight_weapons_each_within_budget() {
		let measured = measure(&run::root()).unwrap();
		let weapons = measured
			.iter()
			.find(|model| model.model == "weapons.glb")
			.unwrap();

		assert_eq!(weapons.counts.weapons.len(), 8);
		for (weapon, triangles) in &weapons.counts.weapons {
			assert!(*triangles > 0, "{weapon} draws nothing");
			assert!(*triangles <= weapon_budget(weapon), "{weapon}");
		}
	}

	/// A mesh of one indexed triangle list (two triangles) drawn by two
	/// nodes, one inside a weapon, and a mesh of one unindexed strip.
	fn handmade() -> Value {
		json!({
			"scene": 0,
			"scenes": [{ "nodes": [0, 3] }],
			"nodes": [
				{ "name": "weapon_scythe", "children": [1, 2] },
				{ "name": "blade", "mesh": 0 },
				{ "name": "haft" },
				{ "name": "body", "mesh": 0, "children": [4] },
				{ "name": "ribbon", "mesh": 1 },
				{ "name": "orphan", "mesh": 1 },
			],
			"meshes": [
				{ "primitives": [{ "attributes": { "POSITION": 0 }, "indices": 1, "targets": [{}, {}] }] },
				{ "primitives": [{ "attributes": { "POSITION": 2 }, "mode": 5 }] },
			],
			"accessors": [{ "count": 4 }, { "count": 6 }, { "count": 5 }],
			"materials": [{}, {}, {}],
			"animations": [{}],
		})
	}

	#[test]
	fn counts_what_every_node_draws() {
		let counts = count(&handmade());

		// Two nodes draw the two-triangle mesh, one the five-vertex strip;
		// the node outside the scene draws nothing.
		assert_eq!(counts.triangles, 2 + 2 + 3);
		assert_eq!(counts.vertices, 4 + 4 + 5);
		assert_eq!(counts.draws, 3);
		assert_eq!(counts.materials, 3);
		assert_eq!(counts.morph_targets, 2);
		assert_eq!(counts.clips, 1);
		assert_eq!(counts.weapons, [("weapon_scythe".to_owned(), 2)]);
	}

	fn fx(triangles: u64, packed_bytes: u64) -> Measured {
		Measured {
			model: "viewmodel_fx.glb".to_owned(),
			counts: Counts {
				triangles,
				..Counts::default()
			},
			packed_bytes,
			raw_bytes: None,
		}
	}

	#[test]
	fn reports_a_model_over_or_without_a_budget() {
		let mut over = fx(FX_TRIANGLES + 1, 16 * KIB + 1);
		over.counts.weapons = vec![("weapon_axe".to_owned(), WEAPON_TRIANGLES + 1)];
		let unknown = Measured {
			model: "statue.glb".to_owned(),
			..fx(0, 1)
		};

		assert_eq!(
			problems(&[fx(FX_TRIANGLES, 16 * KIB)]),
			Vec::<String>::new()
		);
		assert!(check(&[fx(FX_TRIANGLES, 16 * KIB)]).is_ok());
		assert_eq!(problems(&[over]).len(), 3);
		assert!(problems(&[unknown])[0].contains("statue.glb has no budget"));
	}

	#[test]
	fn the_table_names_every_model_and_its_weapons() {
		let measured = vec![Measured {
			model: "weapons.glb".to_owned(),
			counts: count(&handmade()),
			packed_bytes: 3 * KIB,
			raw_bytes: Some(9 * KIB),
		}];
		let table = table(&measured);

		assert!(table.contains("weapons.glb"));
		assert!(table.contains("  weapon_scythe"));
		assert!(table.lines().last().unwrap().starts_with("all"));
	}

	#[test]
	fn groups_thousands() {
		assert_eq!(thousands(0), "0");
		assert_eq!(thousands(999), "999");
		assert_eq!(thousands(1_000), "1,000");
		assert_eq!(thousands(142_204), "142,204");
		assert_eq!(thousands(12_345_678), "12,345,678");
	}
}
