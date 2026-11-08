//! A seat in a rift: four reapers round one husk at a real place.

use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;
use reapers_rules::RiftSite;

/// Everything the rift's view is given: the Flutter `RiftScreen`'s
/// constructor, with the session held by the route.
///
/// The route enters the rift with the keeper, reads the session's frames
/// into `frame`, and passes every report on. The view runs this reaper's
/// own `HuntSim` in rift mode (`HuntSim::rift`) against the frames' shared
/// ward and shriek timetable, and calls `report` with whole standings
/// (`RiftReport`), never changes. `on_leave` walks away; a rift left before
/// it ends pays nothing.
#[derive(Clone)]
pub struct RiftViewProps {
	pub site: RiftSite,
	/// The latest frame, or `None` before the first.
	pub frame: Option<RiftFrame>,
	pub report: Arc<dyn Fn(RiftReport) + Send + Sync>,
	pub on_leave: Arc<dyn Fn() + Send + Sync>,
	/// Whether a simulated player fights (`REAPERS_AUTOPLAY`).
	pub autoplay: bool,
}

/// The rift: the circle gathering over the husk at rest, then this
/// reaper's own fight in the hunt's arena, in rift mode, with the circle
/// under the ward, and how it ended. Shades fill empty seats as the room
/// decides; with `autoplay`, the rules' simulated player fights this seat.
pub fn rift_view(props: &RiftViewProps) -> Element {
	crate::rift::view::rift_view(props)
}
