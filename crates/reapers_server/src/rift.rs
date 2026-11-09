//! The rifts open on this keeper right now.
//!
//! A rift lives in the memory of the process that hosts it, so every reaper
//! in one circle has to be connected to the same process. That is true of a
//! single keeper and is the first thing to change when there are several:
//! see docs/architecture.md.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use leptoskit_auth::Account;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use reapers_rules::HuntRules;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftError;
use reapers_rules::RiftFrame;
use reapers_rules::RiftReport;
use reapers_rules::RiftReward;
use reapers_rules::RiftRoom;
use reapers_rules::RiftSite;
use tokio::sync::broadcast;

/// A rift that fell this far behind catches up no further in one beat.
const MAX_TICKS_PER_BEAT: i64 = 60;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The rifts this keeper is hosting, and who is sitting in which.
pub struct RiftHall {
	beat: Duration,
	ticks_per_beat: Option<i64>,
	gathering: Mutex<HashMap<(RiftSite, bool), Arc<HostedRift>>>,
	seats: Arc<Mutex<HashMap<Id<Account>, Seat>>>,
	tickets: AtomicU64,
}

impl std::fmt::Debug for RiftHall {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RiftHall")
			.field("beat", &self.beat)
			.field("ticks_per_beat", &self.ticks_per_beat)
			.finish_non_exhaustive()
	}
}

impl Default for RiftHall {
	fn default() -> Self {
		Self::new(Duration::from_millis(100), None)
	}
}

impl RiftHall {
	/// A hall that moves each rift on every `beat`, by `ticks_per_beat`
	/// ticks, or by as many as real time has passed when that is `None`,
	/// which is what a real rift wants. A test passes a number to play a
	/// rift through faster than it could be fought.
	pub fn new(beat: Duration, ticks_per_beat: Option<i64>) -> Self {
		Self {
			beat,
			ticks_per_beat,
			gathering: Mutex::new(HashMap::new()),
			seats: Arc::new(Mutex::new(HashMap::new())),
			tickets: AtomicU64::new(0),
		}
	}

	/// Seats `entrant`, for `account`, in the rift gathering at `site`,
	/// opening a new one when none is gathering or the one that is has
	/// filled.
	///
	/// A `demo` circle is kept apart from the real one at the same site, so
	/// a development demo can never be mixed into a circle of ordinary
	/// reapers. An account sits in one rift at a time: its reports go to
	/// that seat.
	pub fn join(
		&self,
		account: Id<Account>,
		site: RiftSite,
		entrant: RiftEntrant,
		demo: bool,
		seed: i64,
	) -> rpc::Result<Chair> {
		let mut seats = locked(&self.seats);

		if seats.contains_key(&account) {
			return Err(rpc::Error::conflict("this reaper is already in a rift"));
		}

		let mut gathering = locked(&self.gathering);
		// A circle everyone left while it gathered is still open, but its
		// clock has stopped: a new reaper starts a new one.
		let open = gathering
			.get(&(site, demo))
			.filter(|rift| {
				let room = locked(&rift.room);

				room.is_open() && !room.is_abandoned()
			})
			.cloned();
		let rift = open.unwrap_or_else(|| {
			let rift = HostedRift::new(RiftRoom::new(site, seed));

			gathering.insert((site, demo), Arc::clone(&rift));
			rift
		});
		let seat = match locked(&rift.room).join(entrant) {
			Ok(seat) => seat,
			Err(RiftError::NotTakingReapers) => {
				return Err(rpc::Error::conflict(
					"the rift began as you arrived; enter again",
				));
			}
			Err(RiftError::InvalidLoadout(_)) => {
				return Err(rpc::Error::server("the reaper's loadout is invalid"));
			}
		};

		drop(gathering);

		let ticket = self.tickets.fetch_add(1, Ordering::Relaxed);
		let failure = Arc::new(Mutex::new(None));

		seats.insert(
			account,
			Seat {
				rift: Arc::clone(&rift),
				index: seat,
				ticket,
				failure: Arc::clone(&failure),
			},
		);
		drop(seats);

		let beats = rift.beats.subscribe();

		rift.start(self.beat, self.ticks_per_beat);

		Ok(Chair {
			rift,
			seat,
			account,
			ticket,
			failure,
			beats,
			seats: Arc::clone(&self.seats),
		})
	}

	/// Takes in `report` from `account`'s seat.
	///
	/// # Errors
	///
	/// Not found when the account is in no rift.
	pub fn report(&self, account: Id<Account>, report: &RiftReport) -> rpc::Result<()> {
		let seats = locked(&self.seats);
		let Some(seat) = seats.get(&account) else {
			return Err(rpc::Error::not_found("this reaper is in no rift"));
		};

		locked(&seat.rift.room).report(seat.index, report);

		Ok(())
	}

	/// Ends `account`'s rift with `failure`: their stream says so at its
	/// next beat.
	pub fn fail(&self, account: Id<Account>, failure: rpc::Error) {
		if let Some(seat) = locked(&self.seats).get(&account) {
			locked(&seat.failure).get_or_insert(failure);
		}
	}
}

/// Who is in a seat, so their reports reach it.
struct Seat {
	rift: Arc<HostedRift>,
	index: usize,
	ticket: u64,
	failure: Arc<Mutex<Option<rpc::Error>>>,
}

/// One rift being kept in time by this keeper.
pub struct HostedRift {
	room: Mutex<RiftRoom>,
	beats: broadcast::Sender<()>,
	clock: Mutex<Option<Clock>>,
}

/// A rift's clock, once its first reaper sits down.
struct Clock {
	task: tokio::task::JoinHandle<()>,
}

impl HostedRift {
	fn new(room: RiftRoom) -> Arc<Self> {
		Arc::new(Self {
			room: Mutex::new(room),
			beats: broadcast::channel(64).0,
			clock: Mutex::new(None),
		})
	}

	/// Starts the rift's clock, if it has not started yet.
	fn start(self: &Arc<Self>, beat: Duration, ticks_per_beat: Option<i64>) {
		let mut clock = locked(&self.clock);

		if clock.is_some() {
			return;
		}

		let rift = Arc::downgrade(self);
		let task = tokio::spawn(async move {
			let started = Instant::now();
			let mut played = 0;
			let mut interval = tokio::time::interval(beat);

			// A beat the host was too busy to take is not owed a replay: run
			// back to back, they would move the rift through a whole reap
			// window before a reaper's report could come back.
			interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

			interval.tick().await;

			loop {
				interval.tick().await;

				let Some(rift) = rift.upgrade() else {
					return;
				};
				let elapsed = i64::try_from(started.elapsed().as_micros()).unwrap_or(i64::MAX);
				let due = elapsed.saturating_mul(HuntRules::TICK_RATE) / 1_000_000;
				let ticks =
					ticks_per_beat.unwrap_or_else(|| (due - played).clamp(0, MAX_TICKS_PER_BEAT));
				let stopped = {
					let mut room = locked(&rift.room);

					room.advance(ticks);
					room.phase().is_over() || room.is_abandoned()
				};

				played = due;
				let _ = rift.beats.send(());

				if stopped {
					return;
				}
			}
		});

		*clock = Some(Clock { task });
	}
}

impl Drop for HostedRift {
	fn drop(&mut self) {
		if let Some(clock) = locked(&self.clock).take() {
			clock.task.abort();
		}
	}
}

/// One reaper's seat in a hosted rift. Dropping it leaves the rift.
pub struct Chair {
	rift: Arc<HostedRift>,
	seat: usize,
	account: Id<Account>,
	ticket: u64,
	failure: Arc<Mutex<Option<rpc::Error>>>,
	beats: broadcast::Receiver<()>,
	seats: Arc<Mutex<HashMap<Id<Account>, Seat>>>,
}

impl std::fmt::Debug for Chair {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Chair")
			.field("seat", &self.seat)
			.finish_non_exhaustive()
	}
}

impl Chair {
	/// The rift as this reaper sees it now.
	pub fn frame(&self) -> Option<RiftFrame> {
		locked(&self.rift.room).frame_for(self.seat)
	}

	/// What the rift paid this reaper.
	pub fn reward(&self) -> RiftReward {
		locked(&self.rift.room).reward_for(self.seat)
	}

	/// Why this reaper's rift must end early, if something said so.
	pub fn failure(&self) -> Option<rpc::Error> {
		locked(&self.failure).take()
	}

	/// Waits until the rift has moved on. `false` once it never will again.
	pub async fn beat(&mut self) -> bool {
		loop {
			match self.beats.recv().await {
				Ok(()) => return true,
				Err(broadcast::error::RecvError::Lagged(_)) => {}
				Err(broadcast::error::RecvError::Closed) => return false,
			}
		}
	}
}

impl Drop for Chair {
	fn drop(&mut self) {
		let mut seats = locked(&self.seats);

		if seats
			.get(&self.account)
			.is_some_and(|seat| seat.ticket == self.ticket)
		{
			seats.remove(&self.account);
		}

		drop(seats);

		// Nobody is left to fight or to be told how it ended once the last
		// reaper goes; the clock stops at its next beat.
		locked(&self.rift.room).leave(self.seat);
	}
}
