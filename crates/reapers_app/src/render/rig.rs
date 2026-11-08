//! One reaper, assembled in a scene from its genome: the class's model with
//! the genome's parts shown, the weapon in its hand, and its look on every
//! surface. The Flutter `ReaperRig`.
//!
//! A class's model carries every part the shuffle can pick; the rig shows
//! the ones this genome names and hides the rest. Weapons share one model,
//! whose instance follows the reaper's `socket_weapon` with every weapon but
//! the held one hidden.

use leptoskit::scene3d::AssetId;
use leptoskit::scene3d::NodeId;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use leptoskit::scene3d::client::Scene;
use reapers_rules::Outfit;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperMotion;
use reapers_rules::Temper;
use reapers_rules::Weapon;
use reapers_rules::WeaponKind;

use super::appearance::ReaperAppearance;
use super::appearance::dressing;
use super::cloth::cloth_weights;
use super::look::ReaperLook;
use super::parts::parts;
use super::playback::EXPRESSIONS;
use super::playback::ReaperPlayback;

/// The model of one class, holding every head, mantle and robe it can wear.
pub fn reaper_model(calling: ReaperClass) -> String {
	format!("models/reaper_{}.glb", calling.name())
}

/// The model that holds every weapon kind.
pub const WEAPONS_MODEL: &str = "models/weapons.glb";

/// The node in the weapons model that holds one kind.
pub fn weapon_part(kind: WeaponKind) -> String {
	format!("weapon_{}", kind.name())
}

/// Shortest and tallest a reaper stands, as a scale on the model.
const MIN_STATURE: f64 = 0.95;
const MAX_STATURE: f64 = 1.08;

/// How tall `genome` stands, as a scale on its model.
pub fn stature(genome: &ReaperGenome) -> f32 {
	(MIN_STATURE + (MAX_STATURE - MIN_STATURE) * ReaperGenome::fraction(genome.stature)) as f32
}

/// The outfit a weapon's temper is drawn as: the same model, more burned and
/// more hollow.
pub const fn temper_look(temper: Temper) -> Outfit {
	match temper {
		Temper::Shrouded => Outfit::Shrouded,
		Temper::Sworn => Outfit::Sworn,
		Temper::Scorched => Outfit::Scorched,
		Temper::Hollowed => Outfit::Hollowed,
	}
}

/// One reaper standing in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct ReaperRig {
	/// The group the reaper stands on: place and turn the reaper with it.
	pub root: NodeId,
	/// The class's model.
	pub body: NodeId,
	/// The weapons model, following the hand.
	pub weapons: NodeId,
	pub genome: ReaperGenome,
	weapon: Weapon,
	outfit: Outfit,
	appearance: ReaperAppearance,
	motion: ReaperMotion,
}

impl ReaperRig {
	/// Queues the ops that stand `genome` in `scene`, under `parent` or at
	/// the root, holding `weapon` and wearing `outfit`, at rest.
	pub fn spawn(
		scene: &Scene,
		parent: Option<NodeId>,
		genome: ReaperGenome,
		weapon: Weapon,
		outfit: Outfit,
		appearance: ReaperAppearance,
	) -> Self {
		let model: AssetId = scene.load(reaper_model(genome.reaper_class));
		let weapons_model = scene.load(WEAPONS_MODEL);
		let root = scene.spawn_in(None, parent);
		let body = scene.spawn_in(Some(model), Some(root));
		let weapons = scene.spawn_in(Some(weapons_model), None);
		scene.attach(weapons, body, "socket_weapon");
		scene.set_transform(body, Transform::IDENTITY.with_scale(stature(&genome)));
		let mut rig = Self {
			root,
			body,
			weapons,
			genome,
			weapon,
			outfit,
			appearance,
			motion: ReaperMotion::Idle,
		};
		// Wearing the outfit dresses the reaper, which puts the weapon in its
		// hand.
		rig.wear(scene, outfit);
		rig.temper(scene);
		let mut idle = ReaperPlayback::default();
		idle.select(ReaperMotion::Idle, true);
		rig.perform(scene, &idle);
		rig
	}

	pub const fn weapon(&self) -> Weapon {
		self.weapon
	}

	pub const fn outfit(&self) -> Outfit {
		self.outfit
	}

	pub const fn appearance(&self) -> ReaperAppearance {
		self.appearance
	}

	/// Shows the parts `appearance` and the outfit choose; the cloth's fit
	/// follows.
	pub fn dress(&mut self, scene: &Scene, appearance: ReaperAppearance) {
		self.appearance = appearance;
		for (part, shown) in dressing(&self.genome, self.outfit, &appearance) {
			scene.set_part_visible(self.body, part, shown);
		}
		self.stir(scene, 0.0);
		self.show_weapon(scene);
	}

	/// Dresses the reaper in `outfit`: the same model drawn with different
	/// numbers, so this is cheap enough to call as the player browses.
	pub fn wear(&mut self, scene: &Scene, outfit: Outfit) {
		self.outfit = outfit;
		let look = ReaperLook::of(&self.genome, outfit);
		for (material, nodes) in parts(&reaper_model(self.genome.reaper_class)) {
			let change = look.material(material);
			for node in *nodes {
				scene.set_material(self.body, Some(node), change);
			}
		}
		self.dress(scene, self.appearance);
	}

	/// Puts `weapon` in the reaper's hand, replacing whatever it held.
	pub fn wield(&mut self, scene: &Scene, weapon: Weapon) {
		self.weapon = weapon;
		self.show_weapon(scene);
		self.temper(scene);
	}

	fn show_weapon(&self, scene: &Scene) {
		let holding = self.appearance.holding && !self.motion.unarmed();
		for kind in WeaponKind::ALL {
			scene.set_part_visible(
				self.weapons,
				weapon_part(kind),
				holding && kind == self.weapon.kind,
			);
		}
	}

	/// A weapon's temper is drawn the way the outfit of the same name is.
	fn temper(&self, scene: &Scene) {
		let look = ReaperLook::of(&self.genome, temper_look(self.weapon.temper));
		for (material, nodes) in parts(super::rig::WEAPONS_MODEL) {
			let change = look.material(material);
			for node in *nodes {
				scene.set_material(self.weapons, Some(node), change);
			}
		}
	}

	/// Plays `playback`'s motion, in this reaper's form. Weapon-free
	/// performances hide the weapon while they play.
	pub fn perform(&mut self, scene: &Scene, playback: &ReaperPlayback) {
		let unarmed_before = self.motion.unarmed();
		self.motion = playback.motion();
		scene.play(self.body, playback.play(&self.appearance));
		if unarmed_before != self.motion.unarmed() {
			self.show_weapon(scene);
		}
	}

	/// Sets the robe's morphs for `seconds` into the motion: its fit, the
	/// genome's gauntness and tatter, and the sway of the cloth. Only the
	/// robe the genome wears takes them: the spirit's body under it carries
	/// targets of the same names, and shrinking it with the robe would push
	/// it out through the cloth.
	pub fn stir(&self, scene: &Scene, seconds: f64) {
		let robe = format!("robe_{}", self.genome.robe);
		for (target, weight) in cloth_weights(&self.genome, &self.appearance, self.motion, seconds)
		{
			scene.push(Op::SetMorphWeight {
				node: self.body,
				part: Some(robe.clone()),
				target: target.to_owned(),
				weight: weight as f32,
			});
		}
	}

	/// Holds the face in `expression` (a morph target name and weight), or
	/// hands it back to the clips.
	pub fn hold_face(&self, scene: &Scene, expression: Option<(&str, f32)>) {
		for target in EXPRESSIONS {
			let weight = match expression {
				Some((held, weight)) if held == target => weight,
				_ => 0.0,
			};
			scene.set_morph_weight(self.body, target, weight);
		}
	}

	/// Removes the reaper and its weapons from the scene.
	pub fn remove(&self, scene: &Scene) {
		scene.remove(self.weapons);
		scene.remove(self.root);
	}
}

#[cfg(test)]
mod tests {
	use leptoskit::scene3d::SceneId;

	use super::*;
	use crate::render::model;

	fn rig() -> (Scene, ReaperRig) {
		let scene = Scene::new(SceneId(9));
		let genome = ReaperGenome::shuffle(4);
		let rig = ReaperRig::spawn(
			&scene,
			None,
			genome,
			Weapon::new(WeaponKind::Spear, Temper::Sworn),
			Outfit::Sworn,
			ReaperAppearance::default(),
		);
		(scene, rig)
	}

	#[test]
	fn every_model_and_weapon_the_rig_names_exists() {
		for calling in ReaperClass::ALL {
			let path = reaper_model(calling);
			assert!(model::node_names(&path).contains(&"socket_weapon".to_owned()));
			let clips = model::clip_names(&path);
			for motion in ReaperMotion::ALL {
				for appearance in [
					ReaperAppearance::default(),
					ReaperAppearance {
						form: super::super::appearance::ReaperForm::Humanoid,
						..ReaperAppearance::default()
					},
				] {
					let clip = appearance.clip_name(motion);
					assert!(clips.contains(&clip), "{path} has no {clip}");
				}
			}
			let morphs = model::morph_names(&path);
			for target in EXPRESSIONS {
				assert!(
					morphs.contains(&target.to_owned()),
					"{path} has no {target}"
				);
			}
			for (target, _) in cloth_weights(
				&ReaperGenome::BORROWED,
				&ReaperAppearance::default(),
				ReaperMotion::Idle,
				0.0,
			) {
				assert!(
					morphs.contains(&target.to_owned()),
					"{path} has no {target}"
				);
			}
		}
		let weapons = model::node_names(WEAPONS_MODEL);
		for kind in WeaponKind::ALL {
			assert!(
				weapons.contains(&weapon_part(kind)),
				"no {}",
				weapon_part(kind)
			);
		}
	}

	#[test]
	fn a_rig_loads_its_class_and_holds_one_weapon() {
		let (scene, rig) = rig();
		let ops = scene.take_ops();
		assert!(
			matches!(&ops[0], Op::LoadAsset { path, .. } if *path == reaper_model(rig.genome.reaper_class))
		);
		assert!(ops.iter().any(|op| matches!(op, Op::Attach { node, target, bone } if *node == rig.weapons && *target == rig.body && bone == "socket_weapon")));
		let shown: Vec<_> = ops
			.iter()
			.filter_map(|op| {
				match op {
					Op::SetPartVisible {
						node,
						part,
						visible: true,
					} if *node == rig.weapons => Some(part.clone()),
					_ => None,
				}
			})
			.collect();
		assert_eq!(shown, ["weapon_spear"]);
		assert!(
			ops.iter()
				.any(|op| matches!(op, Op::Play { play, .. } if play.clip == "idle"))
		);
	}

	#[test]
	fn unarmed_motions_put_the_weapon_away_and_take_it_back() {
		let (scene, mut rig) = rig();
		scene.take_ops();
		let mut clock = ReaperPlayback::default();
		clock.select(ReaperMotion::FinisherPalm, true);
		rig.perform(&scene, &clock);
		let hidden = scene.take_ops().iter().any(|op| {
			matches!(op, Op::SetPartVisible { node, part, visible: false } if *node == rig.weapons && part == "weapon_spear")
		});
		assert!(hidden);
		clock.select(ReaperMotion::Slash, true);
		rig.perform(&scene, &clock);
		let shown = scene.take_ops().iter().any(|op| {
			matches!(op, Op::SetPartVisible { node, part, visible: true } if *node == rig.weapons && part == "weapon_spear")
		});
		assert!(shown);
	}

	#[test]
	fn an_outfit_restyles_every_part_of_the_model() {
		let (scene, mut rig) = rig();
		scene.take_ops();
		rig.wear(&scene, Outfit::Hollowed);
		let ops = scene.take_ops();
		let styled = ops
			.iter()
			.filter(|op| matches!(op, Op::SetMaterial { node, .. } if *node == rig.body))
			.count();
		let parts: usize = parts(&reaper_model(rig.genome.reaper_class))
			.iter()
			.map(|(_, nodes)| nodes.len())
			.sum();
		assert_eq!(styled, parts);
		assert!(ops.iter().any(
			|op| matches!(op, Op::SetPartVisible { part, visible: false, .. } if part.starts_with("void_"))
		));
	}
}
