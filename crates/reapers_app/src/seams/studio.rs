//! The reaper studio: every reaper, outfit and motion, previewed.

use leptoskit::prelude::*;
use reapers_rules::ReaperGenome;

use super::ReaperStageProps;
use super::reaper_stage;
use crate::l10n::use_l10n;
use crate::lookups::class_name;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::light;

/// Everything the studio is given: the Flutter `ReaperStudio`'s
/// constructor. It opens no save, server, wallet or camera, and nothing done
/// in it changes an owned reaper.
#[derive(Clone, Debug, PartialEq)]
pub struct StudioViewProps {
	/// The reaper it opens on: the player's, from the sanctum or roster.
	pub genome: ReaperGenome,
}

/// The studio.
///
/// Until `leptoskit_3d` draws the models it previews, this is the reaper it
/// opens on, standing on the stage.
pub fn studio_view(props: &StudioViewProps) -> Element {
	let words = use_l10n();
	let genome = props.genome.clone();
	let weapon = reapers_rules::Weapon::new(
		genome.reaper_class.weapon(),
		reapers_rules::Temper::Shrouded,
	);
	View()
		.class("flex-1 w-full min-h-screen")
		.test_id("studio-view")
		.child(
			View()
				.class("relative h-[58vh] w-full")
				.child(reaper_stage(&ReaperStageProps::new(
					genome.clone(),
					weapon,
					reapers_rules::Outfit::Shrouded,
				)))
				.child(
					View()
						.class("absolute left-6 top-2 pt-safe")
						.child(ledger_back(None)),
				),
		)
		.child(
			View()
				.class("w-full max-w-[520px] self-center px-6 pb-10")
				.child(label(class_name(&*words, genome.reaper_class), 12, light()))
				.child(gap(6))
				.child(display(genome.name(), 30, BONE))
				.child(gap(12))
				.child(body(words.studio_intro(), 15, ASH)),
		)
}
