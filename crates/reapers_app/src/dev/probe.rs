//! The depth bench (`REAPERS_DEPTH_PROBE`): what the platform's AR makes of
//! the room, where the Flutter bench showed what its depth model did. It
//! follows the platform as a hunt does and, once a second, casts the
//! hunt's grid of rays against the platform's depth, drawing each ray's
//! distance as a shade, nearest brightest, with how long the look took.

use std::cell::RefCell;
use std::rc::Rc;

use leptoskit::prelude::*;

use crate::hunt::pose::Tracking;
use crate::hunt::xr::Shared;
use crate::hunt::xr::follow;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::ledger_back;
use crate::ui::ledger_row;

/// The farthest distance drawn, in metres; anything farther is as dark.
const FAR: f64 = 5.0;

struct Bench {
	shared: Shared,
	started: bool,
	drawn: RwSignal<u64>,
	ticking: bool,
}

thread_local! {
	static BENCH: RefCell<Option<Rc<RefCell<Bench>>>> = const { RefCell::new(None) };
}

fn bench() -> Rc<RefCell<Bench>> {
	BENCH.with_borrow_mut(|bench| {
		bench
			.get_or_insert_with(|| {
				let shared = Shared::default();
				shared.probing.set(true);
				Rc::new(RefCell::new(Bench {
					shared,
					started: false,
					drawn: crate::ui::lasting(|| RwSignal::new(0)),
					ticking: false,
				}))
			})
			.clone()
	})
}

fn start() {
	let bench = bench();
	let mut state = bench.borrow_mut();
	if state.started {
		return;
	}
	state.started = true;
	follow(&state.shared);
	if !state.ticking {
		state.ticking = true;
		let drawn = state.drawn;
		let shared = state.shared.clone();
		crate::render::ticker::every_frame(move |delta| {
			shared.live.borrow_mut().advance(delta);
			drawn.update(|drawn| *drawn = drawn.wrapping_add(1));
			true
		});
	}
}

/// The bench.
pub fn page() -> Element {
	let bench = bench();
	let state = bench.borrow();
	let _ = state.drawn.get();
	let live = state.shared.live.borrow();
	let pose = live.pose();
	let tracking = match live.tracking() {
		Tracking::Drag => "nothing: a finger would steer",
		Tracking::Turning => "the rotation sensor (turns, no position)",
		Tracking::World => "the platform's AR (camera and position)",
	};
	let mut column = View()
		.class("w-full max-w-[520px] self-center px-7 pt-2 pb-10")
		.test_id("depth-probe")
		.child(ledger_back(None))
		.child(gap(14))
		.child(display("Depth bench", 30, BONE))
		.child(gap(8))
		.child(body(
			"What the platform's AR measures of the room: the hunt's grid of rays, cast against its depth once a second.",
			14,
			ASH,
		))
		.child(gap(16))
		.child(ledger_row("Following", tracking, false))
		.child(ledger_row(
			"Facing",
			format!(
				"{:.0}° right, {:.0}° up, turning {:.1} rad/s",
				pose.forward.yaw().to_degrees(),
				pose.forward.pitch().to_degrees(),
				pose.turn_speed
			),
			false,
		))
		.child(ledger_row(
			"Lens",
			live.fov.map_or_else(|| "not reported".to_owned(), |fov| format!("{:.1}° tall", fov.to_degrees())),
			false,
		))
		.child(ledger_row(
			"Floor below",
			live.eye_height.map_or_else(|| "no floor plane yet".to_owned(), |height| format!("{height:.2} m")),
			false,
		))
		.child(ledger_row("Depth", if live.depth { "measured" } else { "none" }, live.depth));
	let look = state.shared.last_look.borrow().clone();
	match look {
		Some(look) => {
			let hits = look
				.distances
				.iter()
				.filter(|distance| distance.is_some())
				.count();
			column = column
				.child(ledger_row(
					"Last look",
					format!(
						"{hits} of {} rays hit, in {} ms",
						look.distances.len(),
						look.millis
					),
					false,
				))
				.child(gap(12))
				.child(picture(&look));
		}
		None => {
			column = column.child(gap(12)).child(body(
				if state.started {
					"No look yet: it needs a phone whose AR measures depth (ARKit on LiDAR iPhones, ARCore's Depth API)."
				} else {
					"Start to follow the platform."
				},
				14,
				ASH,
			));
		}
	}
	if !state.started {
		column = column.child(gap(18)).child(
			LedgerButton::new("Start")
				.test_id("probe-start")
				.on_press(start),
		);
	}
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column)
}

/// The look as a picture: one cell for each ray, nearest brightest.
fn picture(look: &crate::hunt::xr::Look) -> Element {
	let (columns, rows) = (look.columns as f32, look.rows as f32);
	let Ok(view_box) = ViewBox::new(0.0, 0.0, columns, rows) else {
		return View();
	};
	let mut drawing = Vector(view_box);
	for (index, distance) in look.distances.iter().enumerate() {
		let (column, row) = ((index % look.columns) as f32, (index / look.columns) as f32);
		let shade = distance.map_or(0, |metres| {
			(255.0 * (1.0 - (metres / FAR).clamp(0.0, 1.0))) as u8
		});
		if let Ok(cell) = PathData::rect(column, row, 1.0, 1.0) {
			drawing =
				drawing.path(VectorPath::new(cell).fill(Color::rgba(shade, shade, shade, 0xFF)));
		}
	}
	drawing
		.class(format!("w-full h-[{}px]", (360.0 * rows / columns) as u32))
		.accessibility_label(format!("{} by {} rays", look.columns, look.rows))
}
