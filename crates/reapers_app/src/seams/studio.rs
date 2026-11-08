//! The reaper studio: every reaper, outfit and motion, previewed.

use leptoskit::prelude::*;
use reapers_rules::ReaperGenome;

/// Everything the studio is given: the Flutter `ReaperStudio`'s
/// constructor. It opens no save, server, wallet or camera, and nothing done
/// in it changes an owned reaper.
#[derive(Clone, Debug, PartialEq)]
pub struct StudioViewProps {
	/// The reaper it opens on: the player's, from the sanctum or roster.
	pub genome: ReaperGenome,
}

/// The studio, on `leptoskit_3d`: see [`crate::studio`].
pub fn studio_view(props: &StudioViewProps) -> Element {
	crate::studio::studio(props.genome)
}
