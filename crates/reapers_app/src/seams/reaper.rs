//! The reaper, standing in the dark.

use leptoskit::prelude::*;
use reapers_rules::Outfit;
use reapers_rules::ReaperGenome;
use reapers_rules::Weapon;

use crate::render::scene::Key;
use crate::render::stage::plain_show;
use crate::render::stage::stage_view;

/// Everything the stage needs to draw one reaper: the Flutter
/// `ReaperStage`'s constructor.
///
/// The stage fills the box it is given (lay it `absolute inset-0` or in a
/// sized parent) and draws on transparent ground over the page's ink. The
/// reaper stands centred, looked at from about chest height (the Flutter
/// camera's target is 1.42 m up), lit with [`reaper_light`].
#[derive(Clone, Debug, PartialEq)]
pub struct ReaperStageProps {
	/// The reaper's birth traits: calling, build, colourway, omen.
	pub genome: ReaperGenome,
	/// What it holds. The armory passes what the player is trying on.
	pub weapon: Weapon,
	/// What it wears: the same model, dyed, charred or hollowed.
	pub outfit: Outfit,
	/// Whether it turns slowly by itself (the oath's reveal). A stage also
	/// turns under a drag, and holds still under reduced motion.
	pub turns_on_its_own: bool,
	/// Whether the camera moves in on the face (the studio's expressions).
	pub focus_face: bool,
}

impl ReaperStageProps {
	pub fn new(genome: ReaperGenome, weapon: Weapon, outfit: Outfit) -> Self {
		Self {
			genome,
			weapon,
			outfit,
			turns_on_its_own: false,
			focus_face: false,
		}
	}
}

/// The reaper on its stage: the class's model, dressed and lit, drawn by
/// `leptoskit_3d` on the host. Its silhouette stands in while the model
/// loads, and for good where 3D cannot be drawn.
///
/// Each line that shows a stage keeps its own scene, so the sanctum, the
/// armory and the oath each hold their reaper between renders and send only
/// what changes.
#[track_caller]
pub fn reaper_stage(props: &ReaperStageProps) -> Element {
	let key = Key::caller();
	stage_view(
		key,
		plain_show(
			props.genome,
			props.weapon,
			props.outfit,
			props.turns_on_its_own,
			props.focus_face,
		),
	)
	.1
}
