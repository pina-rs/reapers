//! The robe's secondary motion, and the genome's traits on it.

use reapers_rules::ReaperGenome;
use reapers_rules::ReaperMotion;

use super::appearance::ReaperAppearance;
use super::appearance::ReaperClothing;
use super::appearance::ReaperForm;

/// How the robe stirs at one moment of a motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClothStir {
	pub sway_x: f64,
	pub sway_y: f64,
	pub billow: f64,
}

/// Periodic secondary motion, sampled from the same clock as the clip.
/// Genome morphs are separate, so posing never erases a trait.
pub fn cloth_at(motion: ReaperMotion, seconds: f64) -> ClothStir {
	let phase = seconds / motion.seconds() * std::f64::consts::TAU;
	let trailing = match motion {
		ReaperMotion::Sprint => 0.7,
		ReaperMotion::Run => 0.45,
		_ => 0.2,
	};
	ClothStir {
		sway_x: 0.35 * phase.sin(),
		sway_y: trailing + 0.2 * phase.sin(),
		billow: 0.3 + 0.2 * phase.sin(),
	}
}

/// The robe's morph weights, by morph target name.
///
/// Garment targets are mutually exclusive: fitted tunics already include
/// shortening, so adding the spirit's short-robe target would shorten twice.
pub fn cloth_weights(
	genome: &ReaperGenome,
	appearance: &ReaperAppearance,
	motion: ReaperMotion,
	seconds: f64,
) -> [(&'static str, f64); 8] {
	let cloth = cloth_at(motion, seconds);
	let humanoid = appearance.form == ReaperForm::Humanoid;
	let tunic = appearance.clothing == ReaperClothing::Tunic;
	// A grounded garment cannot drift sideways through the hips like an
	// empty spirit robe. Keep the secondary sway inside its fitted allowance.
	let drift = if humanoid { 0.35 } else { 1.0 };
	let on = |yes: bool| if yes { 1.0 } else { 0.0 };
	[
		("sway_x", cloth.sway_x * drift),
		("sway_y", cloth.sway_y * drift),
		("billow", cloth.billow),
		("gaunt", ReaperGenome::fraction(genome.gauntness)),
		("long_hem", ReaperGenome::fraction(genome.tatter)),
		("short_robe", on(!humanoid && tunic)),
		("humanoid_robe", on(humanoid && !tunic)),
		("humanoid_tunic", on(humanoid && tunic)),
	]
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn switching_form_and_clothing_replaces_the_fit_without_double_shortening() {
		let genome = ReaperGenome::shuffle(11);
		for form in [ReaperForm::Spirit, ReaperForm::Humanoid] {
			for clothing in [ReaperClothing::Robe, ReaperClothing::Tunic] {
				let weights = cloth_weights(
					&genome,
					&ReaperAppearance {
						form,
						clothing,
						..ReaperAppearance::default()
					},
					ReaperMotion::Walk,
					0.3,
				);
				let active: Vec<_> = weights
					.iter()
					.filter(|(name, weight)| {
						["short_robe", "humanoid_robe", "humanoid_tunic"].contains(name)
							&& (*weight - 1.0).abs() < f64::EPSILON
					})
					.map(|(name, _)| *name)
					.collect();
				let expected: &[&str] = match (form, clothing) {
					(ReaperForm::Spirit, ReaperClothing::Robe) => &[],
					(ReaperForm::Spirit, _) => &["short_robe"],
					(ReaperForm::Humanoid, ReaperClothing::Robe) => &["humanoid_robe"],
					_ => &["humanoid_tunic"],
				};
				assert_eq!(active, expected);
				assert!(weights.contains(&("gaunt", ReaperGenome::fraction(genome.gauntness))));
				assert!(weights.iter().all(|(_, weight)| weight.is_finite()));
			}
		}
	}
}
