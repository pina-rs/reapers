//! What a reaper is shown wearing: model presentation only. None of it
//! alters a genome or a save.

use reapers_rules::Outfit;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperMotion;

/// The two performances every state has: floating, or grounded on legs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReaperForm {
	#[default]
	Spirit,
	Humanoid,
}

/// The cut of the outer garment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReaperClothing {
	#[default]
	Robe,
	Tunic,
	Unrobed,
}

/// How a reaper is dressed for the stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(
	clippy::struct_excessive_bools,
	reason = "each is one switch in the studio"
)]
pub struct ReaperAppearance {
	pub form: ReaperForm,
	pub clothing: ReaperClothing,
	/// Whether the shoulder piece is worn.
	pub mantle: bool,
	/// Whether anything is held.
	pub holding: bool,
}

impl Default for ReaperAppearance {
	fn default() -> Self {
		Self {
			form: ReaperForm::Spirit,
			clothing: ReaperClothing::Robe,
			mantle: true,
			holding: true,
		}
	}
}

impl ReaperAppearance {
	/// The clip that performs `motion` in this form: each state has a
	/// floating performance and a grounded humanoid one.
	pub fn clip_name(&self, motion: ReaperMotion) -> String {
		match self.form {
			ReaperForm::Spirit => motion.name().to_owned(),
			ReaperForm::Humanoid => format!("humanoid_{}", motion.name()),
		}
	}
}

/// Whether the model's node `name` shows on `genome` wearing `outfit`, or
/// `None` to leave it alone: unrelated nodes, animation targets beneath
/// hidden variant parents among them.
pub fn part_visible(
	name: &str,
	genome: &ReaperGenome,
	outfit: Outfit,
	appearance: &ReaperAppearance,
) -> Option<bool> {
	if name == "humanoid_form" {
		return Some(appearance.form == ReaperForm::Humanoid);
	}
	if name == "spirit_form" {
		return Some(appearance.form == ReaperForm::Spirit);
	}
	for (prefix, chosen, enabled) in [
		("head_", genome.head, true),
		("mantle_", genome.mantle, appearance.mantle),
		(
			"robe_",
			genome.robe,
			appearance.clothing != ReaperClothing::Unrobed,
		),
	] {
		let Some(index) = name.strip_prefix(prefix) else {
			continue;
		};
		if let Ok(index) = index.parse::<u32>() {
			return Some(enabled && index == chosen);
		}
	}
	if name.contains("_sleeve") || name.ends_with("_cuff") {
		return Some(appearance.clothing != ReaperClothing::Unrobed);
	}
	if name.starts_with("void_") || name.starts_with("wrap_") {
		return Some(outfit != Outfit::Hollowed);
	}
	None
}

/// Every node of the six reaper models whose visibility [`part_visible`]
/// decides. The models are built, never edited, so the list is fixed; a
/// test reads the models to keep it whole.
pub const VARIANT_PARTS: [&str; 24] = [
	"humanoid_form",
	"spirit_form",
	"head_0",
	"head_1",
	"head_2",
	"mantle_0",
	"mantle_1",
	"mantle_2",
	"robe_0",
	"robe_1",
	"robe_2",
	"robe_3",
	"arm_l_upper_sleeve",
	"arm_l_lower_sleeve",
	"arm_l_cuff",
	"arm_r_upper_sleeve",
	"arm_r_lower_sleeve",
	"arm_r_cuff",
	"void_0",
	"void_1",
	"void_2",
	"wrap_0",
	"wrap_1",
	"wrap_2",
];

/// The parts to show and hide for `genome` in `outfit` and `appearance`.
pub fn dressing(
	genome: &ReaperGenome,
	outfit: Outfit,
	appearance: &ReaperAppearance,
) -> Vec<(&'static str, bool)> {
	VARIANT_PARTS
		.iter()
		.filter_map(|&name| {
			part_visible(name, genome, outfit, appearance).map(|shown| (name, shown))
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn genome() -> ReaperGenome {
		ReaperGenome {
			head: 2,
			mantle: 1,
			robe: 3,
			..ReaperGenome::shuffle(11)
		}
	}

	#[test]
	fn spirit_and_humanoid_forms_are_exclusive_and_clothing_never_hides_a_hand() {
		let genome = genome();
		for form in [ReaperForm::Spirit, ReaperForm::Humanoid] {
			for clothing in [
				ReaperClothing::Robe,
				ReaperClothing::Tunic,
				ReaperClothing::Unrobed,
			] {
				let appearance = ReaperAppearance {
					form,
					clothing,
					..ReaperAppearance::default()
				};
				let visible = |name| part_visible(name, &genome, Outfit::Shrouded, &appearance);
				assert_eq!(visible("humanoid_form"), Some(form == ReaperForm::Humanoid));
				assert_eq!(visible("spirit_form"), Some(form == ReaperForm::Spirit));
				assert_eq!(visible("robe_3"), Some(clothing != ReaperClothing::Unrobed));
				assert_eq!(visible("robe_0"), Some(false));
				assert_eq!(
					visible("arm_l_lower_sleeve"),
					Some(clothing != ReaperClothing::Unrobed)
				);
				assert_eq!(visible("arm_l_hand"), None);
				assert_eq!(visible("head_2"), Some(true));
				assert_eq!(visible("head_0"), Some(false));
				assert_eq!(visible("face_2"), None);
			}
		}
	}

	#[test]
	fn removing_shoulder_pieces_does_not_remove_heads_or_carried_skulls() {
		let genome = genome();
		let appearance = ReaperAppearance {
			mantle: false,
			holding: false,
			..ReaperAppearance::default()
		};
		assert_eq!(
			part_visible("mantle_1", &genome, Outfit::Shrouded, &appearance),
			Some(false)
		);
		assert_eq!(
			part_visible("head_2", &genome, Outfit::Shrouded, &appearance),
			Some(true)
		);
		assert_eq!(
			part_visible("lantern_head", &genome, Outfit::Shrouded, &appearance),
			None
		);
	}

	#[test]
	fn the_hollowed_outfit_takes_the_void_and_the_wraps() {
		let genome = genome();
		let appearance = ReaperAppearance::default();
		assert_eq!(
			part_visible("void_1", &genome, Outfit::Hollowed, &appearance),
			Some(false)
		);
		assert_eq!(
			part_visible("wrap_0", &genome, Outfit::Sworn, &appearance),
			Some(true)
		);
	}

	#[test]
	fn the_list_of_variant_parts_is_every_model_s_whole() {
		for calling in reapers_rules::ReaperClass::ALL {
			let names =
				crate::render::model::node_names(&crate::render::rig::reaper_model(calling));
			let decided: Vec<_> = names
				.iter()
				.filter(|name| {
					part_visible(
						name,
						&genome(),
						Outfit::Shrouded,
						&ReaperAppearance::default(),
					)
					.is_some()
				})
				.collect();
			assert_ne!(decided.len(), 0);
			for name in decided {
				assert!(
					VARIANT_PARTS.contains(&name.as_str()),
					"{calling:?} has {name}"
				);
			}
		}
	}

	#[test]
	fn clips_are_named_by_form() {
		let humanoid = ReaperAppearance {
			form: ReaperForm::Humanoid,
			..ReaperAppearance::default()
		};
		assert_eq!(
			ReaperAppearance::default().clip_name(ReaperMotion::Walk),
			"walk"
		);
		assert_eq!(
			humanoid.clip_name(ReaperMotion::DodgeLeft),
			"humanoid_dodgeLeft"
		);
	}

	#[test]
	fn empty_hands_exclude_weapon_attacks_but_retain_reactions_and_hand_finishers() {
		for motion in [
			ReaperMotion::Slash,
			ReaperMotion::Thrust,
			ReaperMotion::Parry,
			ReaperMotion::FinisherImpale,
			ReaperMotion::FinisherReap,
		] {
			assert!(!motion.supports(None));
		}
		for motion in [
			ReaperMotion::Laugh,
			ReaperMotion::Sigma,
			ReaperMotion::FinisherPalm,
			ReaperMotion::FinisherUppercut,
			ReaperMotion::FinisherSoulCrush,
		] {
			assert!(motion.supports(None));
		}
	}
}
