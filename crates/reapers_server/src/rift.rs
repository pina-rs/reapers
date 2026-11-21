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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use futures::future::BoxFuture;
use leptoskit_auth::Account;
use leptoskit_db::Id;
use leptoskit_rpc as rpc;
use reapers_api::refused;
use reapers_rules::HuntRules;
use reapers_rules::Refusal;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftError;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftReward;
use reapers_rules::RiftRoom;
use reapers_rules::RiftSite;
use tokio::sync::watch;

/// A rift that fell this far behind catches up no further in one beat.
const MAX_TICKS_PER_BEAT: i64 = 60;

/// Maximum concurrent hosted rift rooms on a single keeper process so
/// unbounded joining cannot exhaust memory.
const MAX_ACTIVE_ROOMS: usize = 256;

/// Server-owned settlement callback invoked by a rift's clock when the room
/// seals, independent of whether any player's WebSocket is still open.
pub type RiftSettler = Arc<
	dyn Fn(Id<Account>, Option<String>, RiftReward) -> BoxFuture<'static, rpc::Result<()>>
		+ Send
		+ Sync,
>;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Gathering circles keyed by `(site, demo)`.
type GatheringMap = Arc<Mutex<HashMap<(RiftSite, bool), Arc<HostedRift>>>>;

/// The rifts this keeper is hosting, and who is sitting in which.
pub struct RiftHall {
	beat: Duration,
	ticks_per_beat: Option<i64>,
	gathering: GatheringMap,
	seats: Arc<Mutex<HashMap<Id<Account>, Seat>>>,
	tickets: AtomicU64,
	active_rooms: Arc<AtomicUsize>,
}

impl std::fmt::Debug for RiftHall {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RiftHall")
			.field("beat", &self.beat)
			.field("ticks_per_beat", &self.ticks_per_beat)
			.field("active_rooms", &self.active_rooms.load(Ordering::Relaxed))
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
			gathering: Arc::new(Mutex::new(HashMap::new())),
			seats: Arc::new(Mutex::new(HashMap::new())),
			tickets: AtomicU64::new(0),
			active_rooms: Arc::new(AtomicUsize::new(0)),
		}
	}

	/// Seats `entrant`, for `account`, in the rift gathering at `site`,
	/// opening a new one when none is gathering or the one that is has
	/// filled.
	///
	/// `reaper` names the reaper that entered and `settler` (when supplied)
	/// pays each surviving human seat from the rift's own clock as soon as the
	/// rift seals, so a late disconnect or token refresh never forfeits earned
	/// rewards.
	///
	/// A `demo` circle is kept apart from the real one at the same site, so
	/// a development demo can never be mixed into a circle of ordinary
	/// reapers. An account sits in one rift at a time: its reports go to
	/// that seat.
	#[expect(clippy::too_many_arguments, reason = "internal hall join contract")]
	pub fn join(
		&self,
		account: Id<Account>,
		reaper: Option<String>,
		site: RiftSite,
		entrant: RiftEntrant,
		demo: bool,
		seed: i64,
		settler: Option<RiftSettler>,
	) -> rpc::Result<Chair> {
		let mut seats = locked(&self.seats);

		if seats.contains_key(&account) {
			return Err(rpc::Error::conflict("this reaper is already in a rift"));
		}

		let mut gathering = locked(&self.gathering);
		let key = (site, demo);

		// Try to join an existing gathering circle, falling back seamlessly to
		// a new room if the existing one filled or started on the last tick.
		let (rift, seat) = {
			let mut joined = None;
			if let Some(existing) = gathering.get(&key).cloned() {
				let mut room = locked(&existing.room);
				if room.is_open() && !room.is_abandoned() {
					match room.join(entrant.clone()) {
						Ok(seat) => {
							let closed = !room.is_open();
							drop(room);
							if closed {
								gathering.remove(&key);
							}
							joined = Some((existing, seat));
						}
						Err(RiftError::NotTakingReapers) => {
							drop(room);
							gathering.remove(&key);
						}
						Err(RiftError::InvalidLoadout(_)) => {
							return Err(rpc::Error::server("the reaper's loadout is invalid"));
						}
					}
				} else {
					drop(room);
					gathering.remove(&key);
				}
			}

			if let Some(pair) = joined {
				pair
			} else {
				if self.active_rooms.load(Ordering::Relaxed) >= MAX_ACTIVE_ROOMS {
					return Err(rpc::Error::retry(
						"too many rifts are open right now; try again shortly",
					));
				}

				let rift = HostedRift::new(
					RiftRoom::new(site, seed),
					key,
					Arc::clone(&self.gathering),
					Arc::clone(&self.active_rooms),
					settler,
				);
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

				gathering.insert(key, Arc::clone(&rift));
				(rift, seat)
			}
		};

		drop(gathering);

		let ticket = self.tickets.fetch_add(1, Ordering::Relaxed);
		let failure = Arc::new(Mutex::new(None));
		let settled_error = Arc::new(Mutex::new(None));

		locked(&rift.participants).push(Participant {
			seat,
			account,
			reaper,
			failure: Arc::clone(&failure),
			settled_error: Arc::clone(&settled_error),
		});

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
			settled_error,
			beats,
			seats: Arc::clone(&self.seats),
		})
	}

	/// Takes in `report` from `account`'s seat, rejecting non-finite,
	/// implausibly inflated damage claims, premature reap outcomes during
	/// `Gathering`/`Fight`, or reports for a seat that has already failed.
	///
	/// # Errors
	///
	/// Not found when the account is in no rift, or a refusal when the report
	/// violates rift bounds.
	pub fn report(&self, account: Id<Account>, report: &RiftReport) -> rpc::Result<()> {
		let seats = locked(&self.seats);
		let Some(seat) = seats.get(&account) else {
			return Err(rpc::Error::not_found("this reaper is in no rift"));
		};

		if let Some(existing) = locked(&seat.failure).clone() {
			return Err(existing);
		}

		let mut room = locked(&seat.rift.room);

		if !report.dealt.is_finite() || report.dealt < 0.0 {
			return Err(refused(Refusal::RiftUnproven));
		}

		let phase = room.phase();

		if !phase.is_over() {
			if matches!(phase, RiftPhase::Gathering | RiftPhase::Fight)
				&& report.outcome.is_some_and(|outcome| outcome.absorbed())
			{
				return Err(refused(Refusal::RiftUnproven));
			}

			if phase != RiftPhase::Gathering {
				let limit = room.max_plausible_dealt(seat.index);
				if report.dealt > limit + 1e-6 {
					return Err(refused(Refusal::RiftUnproven));
				}
			}
		}

		room.report(seat.index, report);

		Ok(())
	}

	/// Ends `account`'s rift with `failure` and wakes their stream immediately.
	pub fn fail(&self, account: Id<Account>, failure: rpc::Error) {
		if let Some(seat) = locked(&self.seats).get(&account) {
			locked(&seat.failure).get_or_insert(failure);
			seat.rift.notify_beat();
		}
	}

	/// Gracefully drains every active rift during server shutdown, notifying
	/// all connected streams so they close promptly.
	pub fn drain(&self) {
		locked(&self.gathering).clear();
		let seats = locked(&self.seats);
		for seat in seats.values() {
			locked(&seat.failure).get_or_insert_with(|| {
				rpc::Error::retry("the keeper is shutting down; rejoin shortly")
			});
			seat.rift.notify_beat();
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

/// Human participant metadata tracked by a [`HostedRift`] so the rift's own
/// clock can settle rewards when the rift seals even if a client disconnects.
struct Participant {
	seat: usize,
	account: Id<Account>,
	reaper: Option<String>,
	failure: Arc<Mutex<Option<rpc::Error>>>,
	settled_error: Arc<Mutex<Option<rpc::Error>>>,
}

/// One rift being kept in time by this keeper.
pub struct HostedRift {
	key: (RiftSite, bool),
	gathering: GatheringMap,
	active_rooms: Arc<AtomicUsize>,
	room: Mutex<RiftRoom>,
	participants: Mutex<Vec<Participant>>,
	settler: Option<RiftSettler>,
	beats: watch::Sender<u64>,
	finished: AtomicBool,
	clock: Mutex<Option<Clock>>,
}

/// A rift's clock, once its first reaper sits down.
struct Clock {
	task: tokio::task::JoinHandle<()>,
}

impl HostedRift {
	fn new(
		room: RiftRoom,
		key: (RiftSite, bool),
		gathering: GatheringMap,
		active_rooms: Arc<AtomicUsize>,
		settler: Option<RiftSettler>,
	) -> Arc<Self> {
		active_rooms.fetch_add(1, Ordering::Relaxed);
		Arc::new(Self {
			key,
			gathering,
			active_rooms,
			room: Mutex::new(room),
			participants: Mutex::new(Vec::new()),
			settler,
			beats: watch::channel(0_u64).0,
			finished: AtomicBool::new(false),
			clock: Mutex::new(None),
		})
	}

	fn notify_beat(&self) {
		self.beats.send_modify(|seq| {
			*seq = seq.wrapping_add(1);
		});
	}

	fn evict_from_gathering(self: &Arc<Self>) {
		let mut gathering = locked(&self.gathering);
		if gathering
			.get(&self.key)
			.is_some_and(|entry| Arc::ptr_eq(entry, self))
		{
			gathering.remove(&self.key);
		}
	}

	/// Pays every human participant whose seat did not fail before the rift
	/// sealed, independent of stream liveness.
	async fn settle_all(&self) {
		let Some(settler) = self.settler.clone() else {
			return;
		};

		let jobs: Vec<_> = {
			let room = locked(&self.room);
			let participants = locked(&self.participants);

			participants
				.iter()
				.filter_map(|participant| {
					if locked(&participant.failure).is_some() {
						return None;
					}
					let reward = room.reward_for(participant.seat);
					Some((
						participant.account,
						participant.reaper.clone(),
						reward,
						Arc::clone(&participant.settled_error),
					))
				})
				.collect()
		};

		let futures = jobs
			.into_iter()
			.map(|(account, reaper, reward, settled_error)| {
				let settler = Arc::clone(&settler);
				async move {
					if let Err(error) = settler(account, reaper, reward).await {
						eprintln!("reapers_server: rift settlement failed for {account}: {error}");
						*locked(&settled_error) = Some(error);
					}
				}
			});

		futures::future::join_all(futures).await;
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
				let (open, sealed, stopped) = {
					let mut room = locked(&rift.room);

					room.advance(ticks);
					(
						room.is_open(),
						room.phase() == RiftPhase::Sealed,
						room.phase().is_over() || room.is_abandoned(),
					)
				};

				played = due;

				if !open || stopped {
					rift.evict_from_gathering();
				}

				// Settle rewards before broadcasting the terminal Sealed beat
				// so every client (and any post-stream Load) observes the
				// committed reward, and late disconnects never forfeit payouts.
				if sealed {
					rift.settle_all().await;
				}

				if stopped {
					rift.finished.store(true, Ordering::Release);
				}

				rift.notify_beat();

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
		self.active_rooms.fetch_sub(1, Ordering::Relaxed);
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
	settled_error: Arc<Mutex<Option<rpc::Error>>>,
	beats: watch::Receiver<u64>,
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

	/// Any settlement error recorded by the rift's server-owned settlement
	/// when it sealed.
	pub fn settlement_error(&self) -> Option<rpc::Error> {
		locked(&self.settled_error).take()
	}

	/// Waits until the rift has moved on. `false` once it never will again.
	///
	/// Backed by a `watch` channel so a slow client never deadlocks on
	/// `Lagged` and always wakes for the terminal frame.
	pub async fn beat(&mut self) -> bool {
		if self.beats.has_changed().unwrap_or(false) {
			self.beats.mark_unchanged();
			return true;
		}

		if self.rift.finished.load(Ordering::Acquire) {
			return false;
		}

		self.beats.changed().await.is_ok()
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
