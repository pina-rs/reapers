//! Helpers the rules' tests share: the Rust side of `reapers_shared/test/support`.

#![allow(dead_code, reason = "each test file uses some of these")]

use reapers_rules::Aim;
use reapers_rules::HuntBot;
use reapers_rules::HuntInput;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntPlan;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftReport;
use reapers_rules::RiftRoom;
use reapers_rules::RiftRules;
use reapers_rules::RiftSeat;
use reapers_rules::RiftSite;
use reapers_rules::SigilEvent;
use reapers_rules::SigilTrace;
use reapers_rules::Temper;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::Weapon;
use reapers_rules::degrees;
use reapers_rules::radians;

// hunt_bot.dart ---------------------------------------------------------

/// What a hunt for one husk is played with, for [`sim_for`].
#[derive(Clone, Copy, Debug)]
pub struct SimFor {
	pub seed: i64,
	pub vessel: Vessel,
	pub rarity: Rarity,
	pub reaper_class: ReaperClass,
	pub level: i64,
	pub temper: Temper,
	pub leniency: f64,
}

impl Default for SimFor {
	/// A hunt for a common penguin with a level-one harvester.
	fn default() -> Self {
		Self {
			seed: 7,
			vessel: Vessel::Penguin,
			rarity: Rarity::Common,
			reaper_class: ReaperClass::Harvester,
			level: 1,
			temper: Temper::Shrouded,
			leniency: 1.0,
		}
	}
}

impl SimFor {
	pub fn stats(&self) -> HuntStats {
		HuntStats::derive(
			&HuntLoadout::new(
				self.reaper_class,
				self.level,
				Weapon::new(self.reaper_class.weapon(), self.temper),
			),
			self.leniency,
		)
		.unwrap()
	}

	pub fn sim(&self) -> HuntSim {
		HuntSim::open(self.seed, self.vessel, self.rarity.ward(), &self.stats())
	}
}

/// A hunt for a common husk with a level-one harvester.
pub fn sim_for(options: SimFor) -> HuntSim {
	options.sim()
}

/// The default hunt: seed 7, a common penguin, a level-one harvester.
pub fn default_sim() -> HuntSim {
	SimFor::default().sim()
}

/// An input that does nothing: aim straight ahead, perfectly still.
pub fn idle() -> HuntInput {
	HuntInput::new(Aim::AHEAD, 0)
}

/// Looks at the husk and holds still, but swings to turn aside a charge: a
/// reaper who does nothing to the husk and lets nothing be done to them.
pub fn wary_on_husk(sim: &HuntSim) -> HuntInput {
	HuntInput::still(
		sim.pose().direction(),
		if sim.charge_progress() > 0.3 {
			sim.stats().parry_speed + 0.5
		} else {
			0.0
		},
	)
}

// rooms.dart ------------------------------------------------------------

/// A kitchen: a counter 92 cm high runs across the room ahead, from 35
/// degrees left to 35 degrees right, with its front 2.2 m away and room to
/// hide behind it 3.2 m out. Its top is level. Everywhere else the floor is
/// clear for 4.5 m.
pub fn kitchen() -> HuntPlan {
	kitchen_with(92, true)
}

pub fn kitchen_with(counter_centimetres: i32, ledge: bool) -> HuntPlan {
	let counter = |slice: usize| degrees(HuntPlan::bearing_of(slice)).abs() < 35.0;
	let slices = 0..HuntPlan::SLICES;

	HuntPlan {
		reach_decimetres: slices
			.clone()
			.map(|slice| if counter(slice) { 22 } else { 45 })
			.collect(),
		cover_centimetres: slices
			.clone()
			.map(|slice| {
				if counter(slice) {
					counter_centimetres
				} else {
					0
				}
			})
			.collect(),
		hide_decimetres: slices
			.clone()
			.map(|slice| if counter(slice) { 32 } else { 0 })
			.collect(),
		ledges: slices.map(|slice| ledge && counter(slice)).collect(),
		..HuntPlan::new(0, 8000)
	}
}

// rift_table.dart -------------------------------------------------------

/// A reaper of `reaper_class` at `level`, with their class's weapon at the
/// best temper that level allows.
pub fn entrant(name: &str, reaper_class: ReaperClass, level: i64) -> RiftEntrant {
	RiftEntrant {
		name: name.to_owned(),
		loadout: HuntLoadout::new(
			reaper_class,
			level,
			Weapon::new(reaper_class.weapon(), Temper::best_for(level)),
		),
	}
}

/// A level-one harvester called Vesper.
pub fn default_entrant() -> RiftEntrant {
	entrant("Vesper", ReaperClass::Harvester, 1)
}

/// A person who plays as carefully as [`HuntBot::human`] does.
pub fn careful(sim: &HuntSim, _person: usize) -> HuntBot {
	HuntBot::human(sim)
}

/// A rift played out in one place: the room, and a seat for every person in
/// it, each played by the bot `play` gives them.
///
/// This is what a host does, the game alone or the server for many, with the
/// network taken out: every tick each person's seat is synced, stepped and
/// reported, and the room is moved on.
pub struct RiftTable {
	pub room: RiftRoom,
	reapers: Vec<RiftEntrant>,
	play: fn(&HuntSim, usize) -> HuntBot,

	/// The room seat of each person, in the order they were given.
	pub seats: Vec<usize>,

	pub fights: Vec<RiftSeat>,
	gone: Vec<usize>,
	bots: Vec<HuntBot>,
}

impl RiftTable {
	/// A table at London Bridge with seed 11, every person careful.
	pub fn new(reapers: Vec<RiftEntrant>) -> Self {
		Self::with(reapers, RiftSite::LondonBridge, 11, careful)
	}

	pub fn with(
		reapers: Vec<RiftEntrant>,
		site: RiftSite,
		seed: i64,
		play: fn(&HuntSim, usize) -> HuntBot,
	) -> Self {
		let mut room = RiftRoom::new(site, seed);
		let seats = reapers
			.iter()
			.map(|reaper| room.join(reaper.clone()).unwrap())
			.collect();

		Self {
			room,
			reapers,
			play,
			seats,
			fights: Vec::new(),
			gone: Vec::new(),
			bots: Vec::new(),
		}
	}

	/// Has everyone stand ready and waits out the count.
	pub fn begin(&mut self) {
		for &seat in &self.seats {
			self.room.report(seat, &RiftReport::readiness(true));
		}

		self.room.advance(RiftRules::COUNTDOWN_TICKS + 1);

		for reaper in &self.reapers {
			let fight = RiftSeat::new(
				self.room.seed(),
				self.room.site(),
				self.room.ward_max(),
				&reaper.loadout,
				None,
			)
			.unwrap();

			self.bots.push((self.play)(&fight.sim, self.bots.len()));
			self.fights.push(fight);
		}
	}

	/// Has person `index` walk away from the rift.
	pub fn walk_away(&mut self, index: usize) {
		self.gone.push(index);
		self.room.leave(self.seats[index]);
	}

	/// Plays one tick for everyone still here.
	pub fn step(&mut self) {
		for index in 0..self.seats.len() {
			if self.gone.contains(&index) {
				continue;
			}

			let frame = self.room.frame_for(self.seats[index]).unwrap();
			let counted = frame.you().map_or(0.0, |you| you.dealt);
			let fight = &mut self.fights[index];

			fight.sync(frame.ward, counted);
			let input = self.bots[index].decide(&fight.sim);
			fight.step(&input);
			self.room.report(self.seats[index], &fight.report());
		}

		self.room.advance(1);
	}

	/// Plays until the rift is over.
	pub fn play(&mut self) {
		self.begin();

		while !self.room.phase().is_over() {
			self.step();
		}
	}
}

// sigil_tracer.dart -----------------------------------------------------

/// What came of a simulated reaper tracing a sigil.
#[derive(Clone, Copy, Debug)]
pub struct TraceResult {
	pub closed: bool,
	pub seconds: f64,
	pub lost: usize,
}

/// How a simulated reaper traces a sigil, for [`trace_sigil`].
#[derive(Clone, Copy, Debug)]
pub struct Tracer {
	pub turn_speed: f64,
	pub wobble: f64,
	pub from: Option<Vec3>,
	pub limit_seconds: f64,
}

impl Default for Tracer {
	fn default() -> Self {
		Self {
			turn_speed: 1.0,
			wobble: 0.15,
			from: None,
			limit_seconds: 60.0,
		}
	}
}

/// Traces `trace` the way a person would: turning toward the next light at
/// `turn_speed` radians a second until the view is well inside it, then
/// holding the phone still, with `wobble` radians a second of the hand's own
/// tremor, until it catches.
///
/// Gives up after `limit_seconds`.
pub fn trace_sigil(trace: &mut SigilTrace, tracer: Tracer) -> TraceResult {
	let frame = 1.0 / 60.0;
	let mut aim = tracer
		.from
		.unwrap_or_else(|| Vec3::from_angles(trace.centre, radians(-25.0)));
	let mut seconds = 0.0;
	let mut lost = 0;

	while !trace.is_closed() && seconds < tracer.limit_seconds {
		let target = trace.direction_of(trace.caught());
		let away = aim.angle_to(target);
		let settled = away <= trace.sigil.radius * 0.4;
		let speed = if settled {
			tracer.wobble
		} else {
			tracer.turn_speed
		};

		if !settled {
			aim = turn_toward(aim, target, away.min(tracer.turn_speed * frame));
		}

		for event in trace.advance(aim, speed, frame) {
			if matches!(event, SigilEvent::LightLost(_)) {
				lost += 1;
			}
		}

		seconds += frame;
	}

	TraceResult {
		closed: trace.is_closed(),
		seconds,
		lost,
	}
}

/// `from` turned `angle` radians toward `to`, along the shortest way round.
fn turn_toward(from: Vec3, to: Vec3, angle: f64) -> Vec3 {
	let between = from.angle_to(to);

	if between <= angle || between == 0.0 {
		return to;
	}

	let share = angle / between;
	let sine = between.sin();

	(from * (((1.0 - share) * between).sin() / sine) + to * ((share * between).sin() / sine))
		.normalized()
}
