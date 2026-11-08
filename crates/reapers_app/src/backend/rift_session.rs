//! One reaper's place in a rift, as the game talks to it.
//!
//! Whoever hosts the rift, the device alone or the server for many, the
//! game sees the same thing: a stream of [`RiftFrame`]s coming one way and
//! this reaper's [`RiftReport`]s going the other.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc;
use futures::future::LocalBoxFuture;
use futures::stream::LocalBoxStream;
use reapers_rules::HuntRules;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftReward;
use reapers_rules::RiftRoom;
use reapers_rules::Timestamp;

use super::KeeperResult;
use crate::platform;

/// The frames of a rift, as this reaper sees them. The last frame says how
/// the rift ended, and then the stream ends.
pub type RiftFrames = LocalBoxStream<'static, KeeperResult<RiftFrame>>;

/// One reaper's seat in a rift.
pub trait RiftSession {
	/// The rift's frames. There is one stream per session: the first call
	/// takes it, and later calls return `None`.
	fn frames(&mut self) -> Option<RiftFrames>;

	/// Tells the rift how this reaper's own fight stands.
	fn report(&self, report: &RiftReport);

	/// Walks away. A rift left before it ends pays this reaper nothing.
	fn leave(&self);
}

/// Settles the books once the rift is sealed.
pub type OnSealed = Rc<dyn Fn(RiftReward) -> LocalBoxFuture<'static, KeeperResult<()>>>;

#[allow(
	clippy::struct_field_names,
	reason = "the room in the room is the rules' one"
)]
struct Room {
	room: RiftRoom,
	seat: usize,
	opened: Timestamp,
	ticks_played: i64,
	closing: bool,
	frames: mpsc::UnboundedSender<KeeperResult<RiftFrame>>,
}

/// A rift hosted on the device itself: one person, and shades in every
/// other seat.
///
/// The room and its rules are the shared ones, so this plays exactly as a
/// rift on the server does with nobody else in it.
pub struct LocalRiftSession {
	room: Rc<RefCell<Room>>,
	frames: Option<mpsc::UnboundedReceiver<KeeperResult<RiftFrame>>>,
}

impl LocalRiftSession {
	/// How often the room is moved on and a frame is sent.
	pub const BEAT: Duration = Duration::from_millis(50);
	/// The room catches up at most this many ticks at once, so a device that
	/// was put to sleep does not wake to a rift that ran without it.
	pub const MAX_TICKS_PER_BEAT: i64 = 30;

	/// Seats `seat` of `room`, opened at `opened`, and starts its beat on the
	/// host's timer. A host without one (the tests) moves it with [`Self::beat`].
	pub fn open(
		room: RiftRoom,
		seat: usize,
		opened: Timestamp,
		clock: Rc<dyn Fn() -> Timestamp>,
		on_sealed: OnSealed,
	) -> Self {
		let session = Self::seated(room, seat, opened);
		let room = Rc::downgrade(&session.room);

		platform::spawn(async move {
			while platform::sleep(Self::BEAT).await {
				let Some(room) = room.upgrade() else {
					return;
				};
				if !Self::advance(&room, clock(), &on_sealed).await {
					return;
				}
			}
		});

		session
	}

	/// A session over `room` with no beat of its own.
	pub fn seated(room: RiftRoom, seat: usize, opened: Timestamp) -> Self {
		let (sender, receiver) = mpsc::unbounded();
		if let Some(frame) = room.frame_for(seat) {
			let _ = sender.unbounded_send(Ok(frame));
		}
		Self {
			room: Rc::new(RefCell::new(Room {
				room,
				seat,
				opened,
				ticks_played: 0,
				closing: false,
				frames: sender,
			})),
			frames: Some(receiver),
		}
	}

	/// Moves the room on to `now` and sends its frame. Returns whether the
	/// rift goes on.
	pub async fn beat(&self, now: Timestamp, on_sealed: &OnSealed) -> bool {
		Self::advance(&self.room, now, on_sealed).await
	}

	async fn advance(room: &Rc<RefCell<Room>>, now: Timestamp, on_sealed: &OnSealed) -> bool {
		let reward = {
			let mut state = room.borrow_mut();
			if state.closing {
				return false;
			}

			let due = now.micros_since(state.opened) * HuntRules::TICK_RATE / 1_000_000;
			let ticks = due - state.ticks_played;
			state.ticks_played = due;
			state.room.advance(ticks.clamp(0, Self::MAX_TICKS_PER_BEAT));

			if !state.room.phase().is_over() {
				if let Some(frame) = state.room.frame_for(state.seat) {
					let _ = state.frames.unbounded_send(Ok(frame));
				}
				return true;
			}

			state.closing = true;
			(state.room.phase() == RiftPhase::Sealed).then(|| state.room.reward_for(state.seat))
		};

		// The books are settled before the last frame is shown, so whoever
		// reads the player after it sees what the rift paid.
		let settled = match reward {
			Some(reward) => on_sealed(reward).await,
			None => Ok(()),
		};

		let state = room.borrow();
		match settled {
			Ok(()) => {
				if let Some(frame) = state.room.frame_for(state.seat) {
					let _ = state.frames.unbounded_send(Ok(frame));
				}
			}
			Err(error) => {
				let _ = state.frames.unbounded_send(Err(error));
			}
		}
		state.frames.close_channel();
		false
	}
}

impl RiftSession for LocalRiftSession {
	fn frames(&mut self) -> Option<RiftFrames> {
		self.frames.take().map(StreamExt::boxed_local)
	}

	fn report(&self, report: &RiftReport) {
		let mut state = self.room.borrow_mut();
		let seat = state.seat;
		state.room.report(seat, report);
	}

	fn leave(&self) {
		let mut state = self.room.borrow_mut();
		if state.closing {
			return;
		}
		state.closing = true;
		let seat = state.seat;
		state.room.leave(seat);
		state.frames.close_channel();
	}
}

impl Drop for LocalRiftSession {
	fn drop(&mut self) {
		self.leave();
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;

	use futures::executor::block_on;
	use futures::future::FutureExt as _;
	use reapers_rules::HuntLoadout;
	use reapers_rules::RiftEntrant;
	use reapers_rules::RiftSite;

	use super::*;

	fn session() -> LocalRiftSession {
		let mut room = RiftRoom::new(RiftSite::ALL[0], 11);
		let seat = room
			.join(RiftEntrant {
				name: "Vesper".into(),
				loadout: HuntLoadout::BORROWED,
			})
			.unwrap();
		LocalRiftSession::seated(room, seat, Timestamp::from_millis(0))
	}

	fn never_sealed() -> OnSealed {
		Rc::new(|_| async { Ok(()) }.boxed_local())
	}

	#[test]
	fn the_first_frame_is_there_at_once() {
		let mut session = session();
		let mut frames = session.frames().unwrap();
		assert!(block_on(frames.next()).unwrap().is_ok());
		assert!(session.frames().is_none());
	}

	#[test]
	fn each_beat_sends_a_frame() {
		let mut session = session();
		let mut frames = session.frames().unwrap();
		let _ = block_on(frames.next());
		assert!(block_on(
			session.beat(Timestamp::from_millis(500), &never_sealed())
		));
		let frame = block_on(frames.next()).unwrap().unwrap();
		assert_eq!(
			frame.phase,
			RiftPhase::Gathering,
			"nobody has stood ready yet"
		);
		assert_eq!(frame.site, RiftSite::ALL[0]);
	}

	#[test]
	fn a_sleeping_device_catches_up_a_little_at_a_time() {
		let mut session = session();
		let mut frames = session.frames().unwrap();
		let _ = block_on(frames.next());
		session.report(&RiftReport::readiness(true));
		block_on(session.beat(Timestamp::from_millis(600_000), &never_sealed()));
		let frame = block_on(frames.next()).unwrap().unwrap();
		assert!(
			!frame.phase.is_over(),
			"ten minutes asleep move the rift a beat's worth, not to its end"
		);
	}

	#[test]
	fn leaving_ends_the_frames_and_settles_nothing() {
		let mut session = session();
		let mut frames = session.frames().unwrap();
		let _ = block_on(frames.next());
		session.leave();
		assert!(block_on(frames.next()).is_none());

		let settled = Rc::new(Cell::new(false));
		let flag = settled.clone();
		let on_sealed: OnSealed = Rc::new(move |_| {
			flag.set(true);
			async { Ok(()) }.boxed_local()
		});
		assert!(!block_on(
			session.beat(Timestamp::from_millis(1_000), &on_sealed)
		));
		assert!(!settled.get());
	}
}
