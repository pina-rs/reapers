//! A seat in a rift: four reapers round one husk at a real place.

use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftSite;

use crate::l10n::use_l10n;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::WOUND;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::ledger_heading;
use crate::ui::ledger_meter;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::ui::margin;

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

/// The rift.
///
/// Until the arena is ported, this is the circle as the frames tell it: the
/// ward, who stands in which seat, and the way to stand ready. The shades
/// fight on; the rift ends, and pays, as the room decides.
pub fn rift_view(props: &RiftViewProps) -> Element {
	let words = use_l10n();
	let light = light();
	let leave = props.on_leave.clone();
	let mut column = View()
		.class("flex-1 w-full max-w-[520px] self-center px-7 pt-6 pb-7")
		.test_id("rift-view")
		.child(margin(words.rift_label()))
		.child(gap(10))
		.child(display(props.site.title(), 36, BONE));

	match &props.frame {
		None => {
			column = column
				.child(gap(14))
				.child(body(words.rift_entering(), 15, ASH));
		}
		Some(frame) => {
			let (title, colour) = match frame.phase {
				RiftPhase::Sealed => (words.rift_sealed_title().to_owned(), light),
				RiftPhase::Failed => (words.rift_failed_title().to_owned(), WOUND),
				RiftPhase::Gathering if frame.counting_down => {
					(
						words.rift_begins(
							frame.ticks_left / reapers_rules::HuntRules::TICK_RATE + 1,
						),
						BONE,
					)
				}
				RiftPhase::Gathering => (words.rift_waiting().to_owned(), BONE),
				RiftPhase::Fight | RiftPhase::Reap => (words.rift_circle().to_owned(), BONE),
			};
			column = column
				.child(gap(14))
				.child(body(title, 15, colour))
				.child(gap(18))
				.child(ledger_meter(
					words.hunt_ward(),
					if frame.ward_max > 0.0 {
						frame.ward / frame.ward_max
					} else {
						0.0
					},
					light,
				))
				.child(ledger_heading(
					words.rift_circle(),
					Some(format!("{}/{}", frame.cuts(), frame.cuts_needed())),
				));
			for member in &frame.members {
				let state = if member.shade {
					words.rift_seat_shade()
				} else if member.ready {
					words.rift_seat_ready()
				} else {
					words.rift_seat_waiting()
				};
				column = column.child(ledger_row(&member.name, state, member.ready));
			}
			if frame.phase == RiftPhase::Gathering && !frame.you().is_some_and(|you| you.ready) {
				let report = props.report.clone();
				column = column.child(gap(18)).child(
					LedgerButton::new(words.rift_stand_ready())
						.test_id("rift-ready")
						.on_press(move || report(RiftReport::readiness(true))),
				);
			}
			if frame.phase.is_over() {
				column = column
					.child(gap(14))
					.child(body(
						if frame.phase == RiftPhase::Sealed {
							words.rift_sealed_body(props.site.title())
						} else {
							words.rift_failed_body().to_owned()
						},
						15,
						ASH,
					))
					.child(ledger_row(
						words.common_experience(),
						format!("+{}", frame.xp),
						false,
					))
					.child(ledger_row(
						words.common_anima(),
						format!("+{}", frame.anima),
						false,
					));
			}
		}
	}

	column = column.child(gap(24)).child(
		LedgerButton::new(words.rift_leave())
			.quiet()
			.test_id("rift-leave")
			.on_press(move || leave()),
	);
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column)
}
