//! A seat in a rift, as the books see it: the keeper seats the player, the
//! session's frames are read as they come, and the rift's view is given the
//! latest one.

use std::cell::Cell;
use std::sync::Arc;

use futures::StreamExt as _;
use leptoskit::prelude::*;
use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::StoredValue;
use reactive_graph::traits::GetValue;
use reactive_graph::traits::SetValue;
use reactive_graph::traits::WithValue;
use reapers_rules::RiftFrame;
use reapers_rules::RiftSite;

use super::back;
use super::back_or_home;
use crate::backend::RiftSession;
use crate::l10n::use_l10n;
use crate::platform;
use crate::seams::RiftViewProps;
use crate::seams::rift_view;
use crate::sound::Score;
use crate::state::Game;
use crate::ui::ledger_notice;

/// The session, shared between the screen and its reader of frames.
type Shared = std::rc::Rc<std::cell::RefCell<Box<dyn RiftSession>>>;

/// The rift this screen is seated in.
#[derive(Clone, Copy)]
struct Seat {
	site: RwSignal<Option<RiftSite>>,
	frame: RwSignal<Option<RiftFrame>>,
	session: StoredValue<Option<Shared>, LocalStorage>,
	/// Incremented on every enter and leave so stale async replies or frame
	/// streams cannot overwrite a newer seat or navigate away after leaving.
	generation: StoredValue<Cell<u64>, LocalStorage>,
}

impl Seat {
	fn next_generation(self) -> u64 {
		self.generation.with_value(|generation| {
			let next = generation.get().wrapping_add(1);
			generation.set(next);
			next
		})
	}

	fn is_current(self, run: u64) -> bool {
		self.generation
			.with_value(|generation| generation.get() == run)
	}
}

fn seat() -> Seat {
	thread_local! {
		static SEAT: Seat = crate::ui::lasting(|| Seat {
			site: RwSignal::new(None),
			frame: RwSignal::new(None),
			session: StoredValue::new_local(None),
			generation: StoredValue::new_local(Cell::new(0)),
		});
	}
	SEAT.with(|seat| *seat)
}

/// Seats the player at `site` with the keeper, and reads its frames.
fn enter(game: Game, seat: Seat, site: RiftSite) {
	if let Some(previous) = seat.session.get_value() {
		previous.borrow().leave();
	}
	seat.session.set_value(None);
	let run = seat.next_generation();
	seat.site.set(Some(site));
	seat.frame.set(None);
	let demo = game.flags.demo_rift;
	game.attempt(
		move |keeper| {
			async move {
				if demo {
					keeper.enter_demo_rift(site).await
				} else {
					keeper.enter_rift(site).await
				}
			}
		},
		move |session| {
			if !seat.is_current(run) {
				if let Some(session) = session {
					session.leave();
				}
				return;
			}
			let Some(mut session) = session else {
				seat.site.set(None);
				back_or_home();
				return;
			};
			let frames = session.frames();
			seat.session
				.set_value(Some(std::rc::Rc::new(std::cell::RefCell::new(session))));
			let Some(mut frames) = frames else {
				return;
			};
			platform::spawn(async move {
				while let Some(frame) = frames.next().await {
					if !seat.is_current(run) {
						break;
					}
					match frame {
						Ok(frame) => {
							let over = frame.phase.is_over();
							seat.frame.set(Some(frame));
							if over {
								// The keeper paid the rift before its last
								// frame; read the books again to show it.
								game.reload().await;
							}
						}
						Err(error) => game.explain(&error),
					}
				}
			});
		},
	);
}

/// Walks away from the rift.
fn leave(game: Game, seat: Seat) {
	let _ = seat.next_generation();
	if let Some(session) = seat.session.get_value() {
		session.borrow().leave();
	}
	seat.session.set_value(None);
	seat.site.set(None);
	seat.frame.set(None);
	crate::rift::view::clear();
	game.play(Score::Sanctum);
	back_or_home();
}

/// Leaves whichever rift is open on screen (for system Back).
pub fn leave_current(game: Game) {
	leave(game, seat());
}

/// The rift at the site named `name`.
pub fn page(name: &str) -> Element {
	let game = Game::get();
	let words = use_l10n();
	let Some(site) = RiftSite::from_name(name) else {
		return ledger_notice(
			words.rift_lost_title(),
			&words.rift_lost_body(name),
			words.common_back(),
			back,
		);
	};
	let seat = seat();
	if seat.site.get_untracked() != Some(site) {
		enter(game, seat, site);
	}
	game.play(Score::Rift);
	rift_view(&RiftViewProps {
		site,
		frame: seat.frame.get(),
		report: Arc::new(move |report| {
			if let Some(session) = seat.session.get_value() {
				session.borrow().report(&report);
			}
		}),
		on_leave: Arc::new(move || leave(game, seat)),
		autoplay: game.flags.autoplay,
	})
}
