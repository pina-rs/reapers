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
use crate::l10n::use_l10n;
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
	mounted: usize,
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
					mounted: 0,
				}))
			})
			.clone()
	})
}

fn mount(bench: &Rc<RefCell<Bench>>) {
	bench.borrow_mut().mounted += 1;
	let unmount = send_wrapper::SendWrapper::new(Rc::downgrade(bench));
	reactive_graph::owner::on_cleanup(move || {
		if let Some(bench) = unmount.take().upgrade() {
			let mut state = bench.borrow_mut();
			state.mounted = state.mounted.saturating_sub(1);
			if state.mounted == 0 {
				state.shared.done.set(true);
				state.started = false;
				state.ticking = false;
			}
		}
	});
}

fn start() {
	let bench = bench();
	let mut state = bench.borrow_mut();
	if state.started {
		return;
	}
	state.shared = Shared::default();
	state.shared.probing.set(true);
	state.started = true;
	follow(&state.shared);
	if !state.ticking {
		state.ticking = true;
		let ticking = Rc::downgrade(&bench);
		crate::render::ticker::every_frame(move |delta| {
			let Some(bench) = ticking.upgrade() else {
				return false;
			};
			let mut state = bench.borrow_mut();
			if state.mounted == 0 || !state.started {
				state.ticking = false;
				state.shared.done.set(true);
				return false;
			}
			state.shared.live.borrow_mut().advance(delta);
			state.drawn.update(|drawn| *drawn = drawn.wrapping_add(1));
			true
		});
	}
}

/// The bench.
pub fn page() -> Element {
	let words = use_l10n();
	let bench = bench();
	mount(&bench);
	let state = bench.borrow();
	let _ = state.drawn.get();
	let live = state.shared.live.borrow();
	let pose = live.pose();
	let tracking = match live.tracking() {
		Tracking::Drag => words.probe_track_drag(),
		Tracking::Turning => words.probe_track_turning(),
		Tracking::World => words.probe_track_world(),
	};
	let mut column = View()
		.class("w-full max-w-[520px] self-center px-7 pt-2 pb-10")
		.test_id("depth-probe")
		.child(ledger_back(None))
		.child(gap(14))
		.child(display(words.probe_title(), 30, BONE))
		.child(gap(8))
		.child(body(words.probe_intro(), 14, ASH))
		.child(gap(16))
		.child(ledger_row(words.probe_following(), tracking, false))
		.child(ledger_row(
			words.probe_facing(),
			words.probe_facing_value(
				&format!("{:.0}", pose.forward.yaw().to_degrees()),
				&format!("{:.0}", pose.forward.pitch().to_degrees()),
				&format!("{:.1}", pose.turn_speed),
			),
			false,
		))
		.child(ledger_row(
			words.probe_lens(),
			live.fov.map_or_else(
				|| words.probe_lens_none().to_owned(),
				|fov| words.probe_lens_value(&format!("{:.1}", fov.to_degrees())),
			),
			false,
		))
		.child(ledger_row(
			words.probe_floor(),
			live.eye_height.map_or_else(
				|| words.probe_floor_none().to_owned(),
				|height| words.probe_floor_value(&format!("{height:.2}")),
			),
			false,
		))
		.child(ledger_row(
			words.probe_depth(),
			if live.depth {
				words.probe_depth_measured()
			} else {
				words.probe_depth_none()
			},
			live.depth,
		));
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
					words.probe_last_look(),
					words.probe_last_look_value(
						hits as i64,
						look.distances.len() as i64,
						look.millis,
					),
					false,
				))
				.child(gap(12))
				.child(picture(&*words, &look));
		}
		None => {
			column = column.child(gap(12)).child(body(
				if state.started {
					words.probe_waiting()
				} else {
					words.probe_idle()
				},
				14,
				ASH,
			));
		}
	}
	if !state.started {
		column = column.child(gap(18)).child(
			LedgerButton::new(words.probe_start())
				.test_id("probe-start")
				.on_press(start),
		);
	}
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column)
}

/// The look as a picture: one cell for each ray, nearest brightest.
fn picture(words: &dyn crate::l10n::Messages, look: &crate::hunt::xr::Look) -> Element {
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
		.accessibility_label(words.probe_rays_label(look.columns as i64, look.rows as i64))
}
