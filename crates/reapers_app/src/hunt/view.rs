//! One hunt, from the threshold to the reap: the Flutter `HuntScreen`, less
//! the books (the route settles what it reports).
//!
//! The hunt lives between renders in [`HuntState`]: the arena and its husk,
//! the sigil being traced, the room being read, and the platform's pose.
//! [`crate::render::ticker`] moves it on about thirty times a second while it
//! is on screen, and the screen is drawn again only when something on it
//! changed.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::ClassPerk;
use reapers_rules::Difficulty;
use reapers_rules::Gait;
use reapers_rules::HuntLog;
use reapers_rules::HuntOutcome;
use reapers_rules::HuntPlan;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::RoomGrid;
use reapers_rules::Sigil;
use reapers_rules::SigilEvent;
use reapers_rules::SigilTrace;
use reapers_rules::wrap_angle;
use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;
use reapers_world::sigil_plan;

use super::arena::Arena;
use super::controller::Fight;
use super::controller::HuntController;
use super::controller::PauseStatus;
use super::controls::SCRIM;
use super::hud::Banner;
use super::hud::banner_for;
use super::hud::knock_for;
use super::hud::status;
use super::hud::status_view;
use super::lessons::Learned;
use super::lessons::Teacher;
use super::overlay::Lens;
use super::overlay::NextLight;
use super::overlay::Sketch;
use super::overlay::fight;
use super::overlay::frame_class;
use super::overlay::sigil;
use super::overlay::veil;
use super::pose::Sensor;
use super::pose::Tracking;
use super::xr::Shared;
use super::xr::follow;
use crate::dev::room_overlay::RoomOverlay;
use crate::l10n::AppTranslations;
use crate::l10n::Messages;
use crate::l10n::use_l10n;
use crate::lookups::difficulty_name;
use crate::platform;
use crate::platform::Prefs;
use crate::render::pointer::Gesture;
use crate::render::pointer::on_gesture;
use crate::render::scene::HUNT;
use crate::seams::HuntBrief;
use crate::seams::HuntViewProps;
use crate::seams::viewmodel_view;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::roman_numeral;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::ledger_row;

/// How long a sigil is given before the player is offered the hunt without
/// it: for anyone who cannot turn far enough, or whose room will not let
/// them.
const SKIP_AFTER: f64 = 60.0;
/// How long the platform is waited on for its first pose before the drag is
/// taken as the source in use. The player is told the phone is finding its
/// bearings while this runs, so it is never a silent wait.
const SIGIL_WAIT_SECONDS: f64 = 3.0;
/// How long a banner stays.
const BANNER_SECONDS: f64 = 1.3;
/// How wide the arc a hunt that plays itself in a drawn room is fought in,
/// either side of straight ahead.
const AUTOPLAY_ARC: f64 = reapers_rules::radians(80.0);
/// How long a finished hunt waits, at most, for its husk to come apart
/// before it is handed to the books.
const LINGER_SECONDS: f64 = 2.4;

/// Where a hunt has got to.
#[derive(Clone, Debug, PartialEq)]
pub enum Stage {
	/// At the threshold: what the husk is, and the button to begin.
	Briefing,
	/// The phone is finding the pose the sigil will hang around. This is
	/// short, and says what it is waiting for.
	Aligning,
	/// Tracing the husk's sigil, while the room is read.
	Tracing(SigilTrace),
	/// The fight.
	Fighting,
	/// Over, and handed to the books.
	Finished,
}

/// Everything a hunt keeps between renders.
pub struct HuntState {
	pub brief: HuntBrief,
	pub difficulty: Difficulty,
	pub autoplay: bool,
	pub stage: Stage,
	pub arena: Arena,
	pub shared: Shared,
	on_finished: Arc<dyn Fn(HuntLog, HuntOutcome) + Send + Sync>,
	banner: Option<(Banner, f64)>,
	/// Seconds since the hunt opened.
	seconds: f64,
	/// Seconds the view has been waiting for the pose the sigil hangs
	/// around.
	aligning_for: f64,
	/// Whether the sigil was hung from the drag because the platform had
	/// not answered. If it answers before a light has begun to fill, the
	/// first light is hung again where the player now faces.
	sigil_on_drag: bool,
	over_for: f64,
	mounted: u32,
	ticking: bool,
	/// Bumped whenever something on screen changed.
	drawn: RwSignal<u64>,
	/// What was last drawn, so a still screen is not drawn again.
	fingerprint: u64,
	/// The hunt's light: its husk's eyes.
	pub light: Rgb,
	/// The view's size in layout units.
	pub size: (f64, f64),
	/// The room drawn by hand the hunt is played in, with
	/// `REAPERS_DRAWN_ROOM`.
	room: Option<reapers_world::DrawnRoom>,
	/// The room's grid drawn over the fight, with `REAPERS_ROOM_OVERLAY`.
	overlay: Option<RoomOverlay>,
	/// What a practice teaches through the caption, until it is learned.
	teacher: Option<Teacher>,
	/// Where what was learned is kept.
	prefs: Option<Rc<dyn Prefs>>,
}

thread_local! {
	static CURRENT: RefCell<Option<Rc<RefCell<HuntState>>>> = const { RefCell::new(None) };
}

/// Moves the hunt on screen on by `seconds`, a tick at a time, as the
/// ticker would: for a host with no timer, and the tests. Answers whether
/// it has been handed to the books.
pub fn play_on(seconds: f64) -> bool {
	let Some(hunt) = current() else {
		return false;
	};
	let mut left = seconds;
	while left > 0.0 && hunt.borrow().stage != Stage::Finished {
		hunt.borrow_mut().tick(HuntController::TICK_SECONDS);
		left -= HuntController::TICK_SECONDS;
	}
	hunt.borrow().stage == Stage::Finished
}

/// Hands `gesture` to the hunt on screen, as the browser's pointer does,
/// with the hunt borrowed: for the tests, which have no pointer.
pub fn feel(gesture: Gesture) {
	if let Some(hunt) = current() {
		hunt.borrow_mut().gesture(gesture);
	}
}

/// The hunt on screen, if any.
fn current() -> Option<Rc<RefCell<HuntState>>> {
	CURRENT.with_borrow(Clone::clone)
}

fn with_hunt(action: impl FnOnce(&mut HuntState)) {
	if let Some(hunt) = current() {
		action(&mut hunt.borrow_mut());
	}
}

/// The game's words, for what the hunt says between renders, where there
/// is no component to ask: the game speaks English only.
fn words() -> &'static dyn Messages {
	use leptoskit_l10n::Translations as _;
	AppTranslations::messages(&AppTranslations::supported_locales()[0])
}

fn light_of(brief: &HuntBrief) -> Rgb {
	let accent = reapers_rules::NftLook::of(&brief.nft).accent;
	Rgb((accent >> 16) as u8, (accent >> 8) as u8, accent as u8)
}

impl HuntState {
	fn new(props: &HuntViewProps) -> Self {
		let size = platform::viewport();
		// The native apps ask for the camera's picture behind the hunt; with
		// no session to give it, it draws as the browser's does.
		let mut arena = Arena::new(
			HUNT,
			props.brief.nft.collection.vessel(),
			&props.brief.nft,
			size.0 / size.1,
			cfg!(feature = "native"),
		);
		arena.height = size.1;
		let shared = Shared {
			environment: arena.environment.clone(),
			scene: Some(arena.scene.id().0),
			..Shared::default()
		};
		let prefs =
			reactive_graph::owner::use_context::<crate::state::Game>().map(|game| game.prefs());
		let teacher = props.brief.is_practice().then(Teacher::default);
		if teacher.is_some()
			&& let Some(prefs) = prefs.clone()
		{
			platform::spawn(async move {
				let Ok(Some(value)) = prefs.get(Learned::KEY).await else {
					return;
				};
				let Some(bits) = value.as_u64().and_then(|bits| u8::try_from(bits).ok()) else {
					return;
				};
				with_hunt(|hunt| {
					if let Some(teacher) = hunt.teacher.as_mut() {
						teacher.recall(Learned::from_bits(bits));
					}
				});
			});
		}
		Self {
			brief: props.brief,
			difficulty: props.difficulty,
			autoplay: props.autoplay,
			stage: Stage::Briefing,
			arena,
			shared,
			on_finished: props.on_finished.clone(),
			banner: None,
			seconds: 0.0,
			aligning_for: 0.0,
			sigil_on_drag: false,
			over_for: 0.0,
			mounted: 0,
			ticking: false,
			drawn: crate::ui::lasting(|| RwSignal::new(0)),
			fingerprint: 0,
			light: light_of(&props.brief),
			size,
			room: crate::dev::drawn_room::room(),
			overlay: None,
			teacher,
			prefs,
		}
	}

	/// Keeps what this practice has taught, unless it played itself.
	fn remember(&self) {
		let (Some(prefs), Some(teacher)) = (self.prefs.clone(), self.teacher.as_ref()) else {
			return;
		};
		if self.autoplay {
			return;
		}
		let bits = teacher.learned().bits();
		platform::spawn(async move {
			let _ = prefs.set(Learned::KEY, serde_json::Value::from(bits)).await;
		});
	}

	/// The fight for this husk, inside `plan`.
	fn sim(&self, plan: HuntPlan) -> Option<HuntSim> {
		let stats = HuntStats::derive(&self.brief.loadout, 1.0).ok()?;
		HuntSim::new(
			self.brief.seed,
			self.brief.nft.collection.vessel(),
			self.brief.nft.rarity(),
			self.brief.nft.rarity().ward(),
			&stats,
			plan,
			self.difficulty,
		)
		.ok()
	}

	/// The sigil that draws this husk out, as forgiving as the difficulty.
	fn sigil(&self) -> Sigil {
		Sigil::draw(
			self.brief.seed,
			&self.brief.nft.rarity().sigil(),
			self.brief.nft.collection.vessel().gait(),
			self.difficulty,
		)
	}

	fn lens(&self) -> Lens {
		let live = self.shared.live.borrow();
		Lens {
			width: self.size.0,
			height: self.size.1,
			pose: self.arena.pose(live.pose()),
			fov: self.arena.view_fov(),
			eye: self.arena.eye(),
		}
	}

	/// The threshold is crossed: the platform is asked for (this is a
	/// press, which browsers ask for), and the sigil waits for the pose of
	/// the source actually in use. A hunt that plays itself has no sigil to
	/// trace.
	fn begin(&mut self) {
		follow(&self.shared);
		let vessel = self.brief.nft.collection.vessel();
		// A new reading starts its own frame: the first picture it accepts
		// places the room.
		self.shared.room_frame.set(None);
		// A room drawn by hand is read whole at once, as if the platform had
		// seen all of it.
		*self.shared.survey.borrow_mut() = Some(
			self.room
				.as_ref()
				.map_or_else(|| RoomSurvey::new(vessel), |room| room.survey(vessel)),
		);
		// A husk that walks reads the room cell by cell as well, to walk
		// across its furniture. One that flies keeps to its own air.
		*self.shared.grid.borrow_mut() = (vessel.gait() == Gait::Ground).then(|| {
			let mut grid = RoomGridBuilder::new();
			if let Some(room) = &self.room {
				crate::dev::drawn_room::read_grid(room, &mut grid);
			}
			grid
		});
		if self.autoplay {
			// A hunt that plays itself has no sigil to trace. A drawn room
			// has its furniture straight ahead, so that is where its hunt is.
			let plan = if self.room.is_some() {
				self.shared
					.survey
					.borrow()
					.as_ref()
					.map_or(HuntPlan::OPEN, |survey| {
						survey.plan_within(0.0, AUTOPLAY_ARC)
					})
			} else {
				HuntPlan::OPEN
			};
			self.engage(plan);
			return;
		}
		self.stage = Stage::Aligning;
		self.aligning_for = 0.0;
	}

	/// Hangs the sigil around the first pose of the source in use, with its
	/// first light where the reaper faces. The whole shape stays around the
	/// player: turning to find the first light is never a turn to somewhere
	/// they were not looking.
	fn hang_sigil(&mut self) -> bool {
		let (pose, follows) = {
			let live = self.shared.live.borrow();
			(live.ready_pose(), live.follows_device())
		};
		let Some(pose) = pose else {
			return false;
		};

		let sigil = self.sigil();
		let first = sigil.lights.first().map_or(0.0, |light| light.yaw);

		self.sigil_on_drag = !follows;
		self.stage = Stage::Tracing(SigilTrace::new(
			sigil,
			wrap_angle(pose.forward.yaw() - first),
		));
		true
	}

	/// Starts the fight inside `plan`. The room stops being read; what it
	/// showed stays, to hide the husk behind.
	fn engage(&mut self, mut plan: HuntPlan) {
		if plan.room.is_none() {
			plan.room = self.room_grid();
		}
		let without_room = HuntPlan {
			room: None,
			..plan.clone()
		};
		let Some(sim) = self
			.sim(plan)
			.or_else(|| self.sim(without_room))
			.or_else(|| self.sim(HuntPlan::OPEN))
		else {
			return;
		};
		self.overlay = sim
			.plan()
			.room
			.clone()
			.filter(|_| crate::config::BuildFlags::current().room_overlay)
			.map(|grid| RoomOverlay::new(grid, self.brief.nft.collection.vessel()));
		self.shared.survey.borrow_mut().take();
		// The room's pictures were charted in the frame of the first one. The
		// fight is drawn there too, so the husk and the furniture it hides
		// behind stay in one room.
		if let Some(frame) = self.shared.room_frame.get() {
			self.arena.ground_in_room(frame);
		}
		let controller = if self.autoplay {
			HuntController::autoplay(Fight::Hunt(sim))
		} else {
			HuntController::new(Fight::Hunt(sim))
		};
		self.arena
			.engage(controller, self.brief.loadout.weapon.kind);
		self.stage = Stage::Fighting;
		self.banner = Some((
			Banner {
				text: words().scan_found().to_owned(),
				strike: false,
			},
			self.seconds,
		));
		platform::thud();
	}

	/// The room as read cell by cell, for a husk that walks, once it has
	/// taken in a picture. The room stops being read.
	fn room_grid(&self) -> Option<RoomGrid> {
		let grid = self.shared.grid.borrow_mut().take()?;
		(grid.frames() > 0).then(|| grid.finish())
	}

	/// The plan the traced sigil and the room read so far come to.
	fn plan_of(&self, trace: &SigilTrace) -> HuntPlan {
		let survey = self.shared.survey.borrow();
		sigil_plan(
			trace,
			survey.as_ref().filter(|survey| survey.pictures() > 0),
		)
	}

	fn skip(&mut self) {
		if let Stage::Tracing(trace) = &self.stage {
			let plan = self.plan_of(trace);
			self.engage(plan);
		}
	}

	fn tick(&mut self, delta: f64) {
		self.size = platform::viewport();
		self.arena.aspect = self.size.0 / self.size.1;
		self.arena.height = self.size.1;
		self.seconds += delta;
		let live_pose = {
			let mut live = self.shared.live.borrow_mut();
			live.advance(delta);
			live.pose()
		};
		if let Some(fov) = self.shared.live.borrow().fov {
			self.arena.fov = fov;
		}
		if let Some(position) = self.shared.live.borrow().position {
			if self.shared.live.borrow().world_tracking() == Some(true) {
				self.arena.anchor_world(position);
			} else {
				self.arena.anchor(position);
			}
		}
		let mut hung = false;
		if self.stage == Stage::Aligning {
			self.aligning_for += delta;
			let waiting = self.shared.live.borrow().sensor() == Sensor::Waiting;

			if waiting && self.aligning_for >= SIGIL_WAIT_SECONDS {
				// The platform has had its say and said nothing: the drag is
				// the source in use, and the player is told so.
				self.shared.live.borrow_mut().no_sensor();
			}

			hung = self.hang_sigil();
		}
		if !hung && self.sigil_on_drag {
			// The platform answered after the drag had to stand in. Nothing
			// has caught yet, so the first light is hung again where the
			// player now faces; after the first catch the lights stay fixed.
			let untouched = match &self.stage {
				Stage::Tracing(trace) => trace.caught() == 0 && trace.fill() == 0.0,
				_ => false,
			};
			let follows = self.shared.live.borrow().follows_device();

			if untouched && follows {
				hung = self.hang_sigil();
			}
		}
		let mut closed = None;
		if !hung && let Stage::Tracing(trace) = &mut self.stage {
			for event in trace.advance(live_pose.forward, live_pose.turn_speed, delta) {
				match event {
					SigilEvent::LightCaught(_) => platform::thud(),
					SigilEvent::LightLost(_) => platform::tap(),
					SigilEvent::SigilClosed => closed = Some(trace.clone()),
				}
			}
		}
		match self.stage {
			Stage::Briefing | Stage::Aligning | Stage::Tracing(_) => {
				self.arena.idle(delta, live_pose);
			}
			Stage::Fighting => self.fight(delta, live_pose),
			Stage::Finished => {}
		}
		if let Some(trace) = closed {
			let plan = self.plan_of(&trace);
			self.engage(plan);
		}
		if self
			.banner
			.as_ref()
			.is_some_and(|(_, since)| self.seconds - since > BANNER_SECONDS)
		{
			self.banner = None;
		}
		let fingerprint = self.fingerprint();
		if fingerprint != self.fingerprint {
			self.fingerprint = fingerprint;
			self.drawn.update(|drawn| *drawn += 1);
		}
	}

	fn fight(&mut self, delta: f64, live_pose: reapers_world::ArPose) {
		// A phone has no press over a scene to hear: a finger resting on the
		// screen is the hold. The browser hears ORB itself, through the
		// view's gestures.
		if !crate::render::pointer::follows_fingers()
			&& let Some(controller) = self.arena.controller.as_mut()
			&& !controller.plays_itself()
		{
			let holding = self.shared.live.borrow().holding;

			controller.hold_orb(holding);
		}
		let world = self.shared.live.borrow().world_tracking();
		let events = self.arena.advance(delta, live_pose, world);
		let mut learned = false;
		for (_, event) in &events {
			knock_for(event);
			if let Some(banner) = banner_for(words(), event) {
				self.banner = Some((banner, self.seconds));
			}
			if let Some(teacher) = self.teacher.as_mut() {
				learned |= teacher.hear(event);
			}
		}
		if learned {
			self.remember();
		}
		let Some(controller) = self.arena.controller.as_ref() else {
			return;
		};
		if !controller.sim().is_over() {
			return;
		}
		// The quiet after the reap is part of the hunt: the books hear of it
		// once the husk has come apart.
		self.over_for += delta;
		if self.arena.gone() || self.over_for > LINGER_SECONDS {
			let Some(outcome) = controller.sim().outcome() else {
				return;
			};
			let log = controller.log();
			self.stage = Stage::Finished;
			self.shared.done.set(true);
			(self.on_finished)(log, outcome);
		}
	}

	/// A cheap summary of what is on screen.
	fn fingerprint(&self) -> u64 {
		use std::hash::Hash;
		use std::hash::Hasher;
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		let pose = self.lens().pose;
		let quantum = |value: f64| (value * 500.0).round() as i64;
		(
			quantum(pose.forward.x),
			quantum(pose.forward.y),
			quantum(pose.forward.z),
			quantum(pose.up.x),
		)
			.hash(&mut hasher);
		std::mem::discriminant(&self.stage).hash(&mut hasher);
		if let Stage::Tracing(trace) = &self.stage {
			(
				trace.caught(),
				quantum(trace.fill()),
				(self.seconds * 4.0) as i64,
			)
				.hash(&mut hasher);
		}
		if let Some(controller) = &self.arena.controller {
			controller.ticks().hash(&mut hasher);
			std::mem::discriminant(&controller.pause_status()).hash(&mut hasher);
		}
		let eye = self.arena.eye();
		(quantum(eye.x), quantum(eye.y), quantum(eye.z)).hash(&mut hasher);
		self.banner
			.as_ref()
			.map(|(banner, _)| banner.text.clone())
			.hash(&mut hasher);
		hasher.finish()
	}

	fn gesture(&mut self, gesture: Gesture) {
		// Three view-heights of drag turn the view through its own field of
		// view: the Flutter `_LookAround`.
		let per_unit = self.arena.fov / self.size.1 * 3.0;
		super::controls::steer(
			gesture,
			&mut self.shared.live.borrow_mut(),
			per_unit,
			self.arena.controller.as_mut(),
		);
	}
}

/// Keeps `hunt` ticking while it is on screen: from this render until the
/// one that replaces it, or the screen goes.
fn mount(hunt: &Rc<RefCell<HuntState>>) {
	let mut state = hunt.borrow_mut();
	state.mounted += 1;
	if !state.ticking {
		state.ticking = true;
		let ticking = Rc::downgrade(hunt);
		crate::render::ticker::every_frame(move |delta| {
			let Some(hunt) = ticking.upgrade() else {
				return false;
			};
			let mut hunt = hunt.borrow_mut();
			if hunt.mounted == 0 || hunt.stage == Stage::Finished {
				hunt.ticking = false;
				hunt.shared
					.done
					.set(hunt.mounted == 0 || hunt.stage == Stage::Finished);
				return false;
			}
			hunt.tick(delta);
			true
		});
		let listener = Rc::downgrade(hunt);
		on_gesture(state.arena.scene.id(), move |gesture| {
			if let Some(hunt) = listener.upgrade() {
				hunt.borrow_mut().gesture(gesture);
			}
		});
	}
	drop(state);
	let unmount = send_wrapper::SendWrapper::new(Rc::downgrade(hunt));
	reactive_graph::owner::on_cleanup(move || {
		if let Some(hunt) = unmount.take().upgrade() {
			let mut hunt = hunt.borrow_mut();
			hunt.mounted = hunt.mounted.saturating_sub(1);
		}
	});
}

/// The hunt for `props`, kept from render to render.
fn hunt_for(props: &HuntViewProps) -> Rc<RefCell<HuntState>> {
	if let Some(hunt) = current() {
		let same = {
			let state = hunt.borrow();
			state.brief == props.brief && state.stage != Stage::Finished
		};
		if same {
			let mut state = hunt.borrow_mut();
			// The difficulty may change at the threshold.
			if state.stage == Stage::Briefing {
				state.difficulty = props.difficulty;
			}
			state.on_finished = props.on_finished.clone();
			drop(state);
			return hunt;
		}
		hunt.borrow().shared.done.set(true);
	}
	let hunt = Rc::new(RefCell::new(HuntState::new(props)));
	CURRENT.with_borrow_mut(|current| *current = Some(hunt.clone()));
	hunt
}

/// The hunt: the threshold, then the sigil and the fight over the room.
pub fn hunt_view(props: &HuntViewProps) -> Element {
	let hunt = hunt_for(props);
	mount(&hunt);
	let state = hunt.borrow();
	// Drawn again whenever the hunt says something changed.
	let _ = state.drawn.get();
	let lens = state.lens();
	// Where the platform draws its camera's picture behind the scene, the
	// room shows through: no veil, and nothing opaque under the scene.
	let camera = state.shared.live.borrow().camera;
	let mut sketch = Sketch::default();
	if !camera {
		veil(&mut sketch, &lens);
	}
	if let Some(room) = &state.room {
		crate::dev::drawn_room::draw(&mut sketch, &lens, room);
	}
	if state.stage == Stage::Fighting
		&& let Some(overlay) = &state.overlay
	{
		overlay.draw(&mut sketch, &lens);
	}
	let mut root = View()
		.class(frame_class(camera))
		.test_id("hunt-view")
		.child(sketch.into_view(state.size.0, state.size.1))
		.child(
			Scene3d(state.arena.scene.id())
				.class("absolute inset-0 w-full h-full")
				.aria_hidden(true),
		);
	if state.stage == Stage::Fighting
		&& let Some(arms) = state.arena.viewmodel()
	{
		// The reaper's own arms, over the room and under the marks and the
		// thumbs' controls.
		root = root.child(viewmodel_view(&arms));
	}
	let mut marks = state
		.arena
		.room_marks(&lens, state.shared.live.borrow().position);
	match &state.stage {
		Stage::Tracing(trace) => sigil(&mut marks, &lens, trace, state.light),
		Stage::Fighting => {
			if let Some(controller) = &state.arena.controller {
				fight(
					&mut marks,
					&lens,
					controller.sim(),
					state.light,
					state.arena.wound,
					state.brief.loadout.reaper_class.perk() == ClassPerk::Measure,
					controller.pause_status() != PauseStatus::Playing,
				);
			}
		}
		Stage::Briefing | Stage::Aligning | Stage::Finished => {}
	}
	// A light can project into the instruction panel. Keep the tracing
	// targets above it so the very thing the player must find stays visible.
	// Combat marks stay below the weapon controls.
	if let Stage::Tracing(trace) = &state.stage {
		return root
			.child(scan(&state, trace))
			.child(marks.into_view(state.size.0, state.size.1));
	}
	if !marks.is_empty() {
		root = root.child(marks.into_view(state.size.0, state.size.1));
	}
	root = match &state.stage {
		Stage::Briefing => root.child(briefing(&state, props)),
		Stage::Aligning => root.child(aligning()),
		Stage::Tracing(trace) => root.child(scan(&state, trace)),
		Stage::Fighting => root.child(fighting(&state)),
		Stage::Finished => root,
	};
	root
}

fn fighting(state: &HuntState) -> Element {
	let words = use_l10n();
	let Some(controller) = &state.arena.controller else {
		return View();
	};
	let live = state.shared.live.borrow();
	let pose = state.arena.pose(live.pose());
	let mut shown = status(
		&*words,
		&state.brief.nft.title(),
		controller.sim(),
		pose.turn_speed,
		live.tracking() != Tracking::Drag,
		state.banner.as_ref().map(|(banner, _)| banner.clone()),
		"",
	);
	let pause = pause_words(&*words, controller.pause_status());
	if pause.is_some() {
		shown.prompt.clear();
	}
	let lesson = state
		.teacher
		.as_ref()
		.and_then(|teacher| teacher.prompt(controller.sim()));
	// One thing to read at a time: while the caption teaches, the ledger's
	// hint, which says much the same in other words, keeps quiet.
	if super::controls::caption(&*words, controller.sim(), lesson).is_some_and(|line| line.teaching)
	{
		shown.prompt.clear();
	}
	let thumbs = super::controls::Thumbs {
		sim: controller.sim(),
		slot: controller.slot(),
		carried: std::array::from_fn(|slot| controller.carries(slot)),
		weapon: state.brief.loadout.weapon.kind,
		light: state.light,
		aim: pose.forward,
		lesson,
		touchable: !controller.plays_itself(),
		on_fire: fire,
		on_slot: select_slot,
	};
	let mut screen =
		View()
			.class("absolute inset-0")
			.child(status_view(&*words, &shown, state.light, None));
	if pause.is_none() {
		screen = screen.child(super::controls::controls(&*words, &thumbs));
	} else {
		screen = screen.child(
			View()
				.class("absolute left-3 bottom-4 pb-safe px-3 py-2 rounded-[8px] bg-black")
				.child(label(words.hunt_orb(), 16, BONE)),
		);
	}
	if let Some(message) = pause {
		screen = screen.child(
			View()
				.class("absolute left-3 right-3 top-0 pt-safe pt-3")
				.child(
					View()
						.class(format!("px-3 py-2 rounded-[8px] {SCRIM}"))
						.test_id("hunt-pause-guidance")
						.child(body(message, 14, BONE).aria_live(leptoskit::native::Live::Polite)),
				),
		);
	}
	screen
}

fn pause_words(words: &dyn Messages, pause: PauseStatus) -> Option<&'static str> {
	match pause {
		PauseStatus::Playing => None,
		PauseStatus::Tracking => Some(words.hunt_tracking_pause()),
		PauseStatus::ReturnToStance => Some(words.hunt_stance_pause()),
		PauseStatus::ResumeHeldOrb => Some(words.hunt_orb_resume()),
	}
}

/// Presses FIRE once for the fight on screen.
fn fire() {
	with_hunt(|hunt| {
		if let Some(controller) = hunt.arena.controller.as_mut() {
			controller.press();
		}
	});
}

/// Draws the weapon in `slot`.
fn select_slot(slot: usize) {
	with_hunt(|hunt| {
		if let Some(controller) = hunt.arena.controller.as_mut() {
			controller.select(slot);
		}
	});
}

fn scan(state: &HuntState, trace: &SigilTrace) -> Element {
	let words = use_l10n();
	let drag = state.shared.live.borrow().tracking() == Tracking::Drag;
	let guidance = next_light_words(&*words, NextLight::of(&state.lens(), trace), drag);
	let mut column = View()
		.class(format!("px-3 pt-2 pb-3 rounded-[8px] {SCRIM}"))
		.test_id("hunt-scan-panel")
		.child(label(words.scan_label(), 11, ASH))
		.child(gap(6))
		.child(display(words.scan_title(), 26, BONE))
		.child(gap(6))
		.child(figure(
			format!("{} / {}", trace.caught(), trace.sigil.lights.len()),
			13,
			BONE,
		))
		.child(gap(8))
		.child(
			body(guidance, 14, BONE)
				.test_id("hunt-scan-guidance")
				.aria_live(leptoskit::native::Live::Polite),
		);
	let floor = state
		.shared
		.survey
		.borrow()
		.as_ref()
		.and_then(|survey| survey.map().floor_depth());
	if let Some(floor) = floor {
		column = column.child(gap(6)).child(body(
			words.scan_floor_found(&format!("{floor:.1}")),
			13,
			ASH,
		));
	}
	let mut view = View().class("absolute inset-0").child(
		View()
			.class("absolute left-3 right-3 top-0 pt-2 pt-safe")
			.child(column),
	);
	if trace.seconds() > SKIP_AFTER {
		view = view.child(
			View()
				.class("absolute left-3 right-3 bottom-8 pb-safe")
				.child(
					LedgerButton::new(words.scan_skip())
						.quiet()
						.test_id("hunt-skip")
						.on_press(|| with_hunt(HuntState::skip)),
				),
		);
	}
	view
}

/// What to do about the next light, in words that match the arrow and the
/// reticle. A drag says drag; a phone says turn.
fn next_light_words(words: &dyn Messages, next: NextLight, drag: bool) -> &'static str {
	match (next, drag) {
		(NextLight::Hold, _) => words.scan_hold(),
		(NextLight::Slow, _) => words.sigil_steady(),
		(NextLight::Aim, _) => words.scan_aim(),
		(NextLight::Left, false) => words.scan_turn_left(),
		(NextLight::Right, false) => words.scan_turn_right(),
		(NextLight::Up, false) => words.scan_look_up(),
		(NextLight::Down, false) => words.scan_look_down(),
		(NextLight::Left, true) => words.scan_drag_left(),
		(NextLight::Right, true) => words.scan_drag_right(),
		(NextLight::Up, true) => words.scan_drag_up(),
		(NextLight::Down, true) => words.scan_drag_down(),
	}
}

/// The moment between raising the blade and the sigil: the phone is finding
/// where it faces, and the player is told it is being waited on rather than
/// left at a still 0 / 3.
fn aligning() -> Element {
	let words = use_l10n();

	View().class("absolute inset-0").child(
		View()
			.class("absolute left-3 right-3 top-0 pt-2 pt-safe")
			.child(
				View()
					.class(format!("px-3 pt-2 pb-3 rounded-[8px] {SCRIM}"))
					.test_id("hunt-aligning")
					.child(label(words.scan_label(), 11, ASH))
					.child(gap(6))
					.child(display(words.scan_title(), 26, BONE))
					.child(gap(8))
					.child(
						body(words.scan_starting(), 14, BONE)
							.aria_live(leptoskit::native::Live::Polite),
					),
			),
	)
}

/// What stands between the player and the hunt: which husk it is, and the
/// one thing to remember before raising the phone.
fn briefing(state: &HuntState, props: &HuntViewProps) -> Element {
	let words = use_l10n();
	let brief = state.brief;
	let mut column = View()
		.class("w-full max-w-[520px] self-center px-7 pt-2 pb-7")
		.child(ledger_back(None))
		.child(gap(18))
		.child(label(
			if brief.is_practice() {
				words.hunt_practice_label()
			} else {
				words.hunt_sighted_label()
			},
			12,
			ASH,
		))
		.child(gap(10))
		.child(display(brief.nft.title(), 40, BONE))
		.child(gap(18));
	// The whole of a hunt, in the order it happens.
	for (step, line) in [
		words.hunt_how_sigil(),
		words.hunt_how_stalk(),
		words.hunt_how_fire(),
		words.hunt_how_weapons(),
		words.hunt_how_hold(),
		words.hunt_how_charge(),
		words.hunt_how_capture(),
	]
	.into_iter()
	.enumerate()
	{
		column = column.child(
			View()
				.class("flex-row pb-2")
				.child(View().class("w-[26px]").child(figure(
					roman_numeral(step as i64 + 1),
					13,
					ASH,
				)))
				.child(View().class("flex-1").child(body(line, 15, BONE))),
		);
	}
	column = column
		.child(gap(10))
		.child(body(words.hunt_safety(), 14, ASH))
		.child(gap(16));
	column = match props.on_difficulty.clone() {
		Some(cycle) => {
			column.child(
				LedgerButton::new(words.hunt_difficulty())
					.detail(difficulty_name(&*words, state.difficulty))
					.quiet()
					.test_id("hunt-difficulty")
					.on_press(move || cycle()),
			)
		}
		None => {
			column.child(ledger_row(
				words.hunt_difficulty(),
				difficulty_name(&*words, state.difficulty),
				false,
			))
		}
	};
	column = column.child(gap(18)).child(
		LedgerButton::new(words.hunt_begin())
			.test_id("hunt-begin")
			.on_press(|| with_hunt(HuntState::begin)),
	);
	ScrollView()
		.class("absolute inset-0 pt-safe pb-safe bg-[#07080bd1]")
		.child(column)
}

#[cfg(test)]
mod tests {
	use reactive_graph::owner::Owner;
	use reapers_rules::Vec3;
	use reapers_world::ArPose;
	use reapers_world::to_platform;

	use crate::render::camera::look_at;
	#[test]
	fn pause_guidance_uses_the_localized_action_for_each_state() {
		use super::PauseStatus;
		use super::pause_words;
		use super::words;
		let words = words();
		assert_eq!(pause_words(words, PauseStatus::Playing), None);
		assert_eq!(
			pause_words(words, PauseStatus::Tracking),
			Some(words.hunt_tracking_pause())
		);
		assert_eq!(
			pause_words(words, PauseStatus::ReturnToStance),
			Some(words.hunt_stance_pause())
		);
		assert_eq!(
			pause_words(words, PauseStatus::ResumeHeldOrb),
			Some(words.hunt_orb_resume())
		);
	}
	use std::sync::Arc;

	use reapers_rules::HuntLoadout;
	use reapers_rules::market_listings;

	use super::*;
	use crate::seams::HuntBrief;

	fn hunt() -> HuntState {
		HuntState::new(&HuntViewProps {
			brief: HuntBrief {
				nft: market_listings()[0],
				seed: 7,
				loadout: HuntLoadout::BORROWED,
				sighting_id: None,
			},
			difficulty: Difficulty::default(),
			autoplay: false,
			on_finished: Arc::new(|_, _| {}),
			on_difficulty: None,
		})
	}

	/// A turn of `angle` about the platform's vertical, as a quaternion.
	fn turned(angle: f64) -> [f64; 4] {
		[0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()]
	}

	#[test]
	fn the_sigil_waits_for_the_source_in_use_and_hangs_its_first_light_ahead() {
		let mut hunt = hunt();
		hunt.begin();

		assert!(
			matches!(hunt.stage, Stage::Aligning),
			"no trace is hung before a source has answered"
		);

		// A platform has been asked and has not answered yet; the sigil
		// waits rather than hanging from the drag's arbitrary origin.
		hunt.shared.live.borrow_mut().expect_platform();
		assert!(hunt.shared.live.borrow().ready_pose().is_none());
		hunt.tick(HuntController::TICK_SECONDS);
		assert!(matches!(hunt.stage, Stage::Aligning));

		// Its first frame faces far from the drag's origin. The sigil is
		// hung on that frame, so its first light is where the phone now
		// points, not at the drag's old origin.
		hunt.shared
			.live
			.borrow_mut()
			.track(turned(-2.4), 0.0, Tracking::Turning);
		hunt.tick(HuntController::TICK_SECONDS);

		let view = hunt.shared.live.borrow().pose().forward;
		let Stage::Tracing(trace) = &hunt.stage else {
			panic!(
				"the sigil hangs once the platform answers\n{:?}",
				hunt.stage
			);
		};
		let first = trace.direction_of(0);
		assert!(
			wrap_angle(first.yaw() - view.yaw()).abs() < 1e-9,
			"the first light is where the player faces, not behind them"
		);
	}

	#[test]
	fn with_no_platform_the_drag_is_the_source_and_the_sigil_hangs_at_once() {
		let mut hunt = hunt();
		hunt.begin();
		assert_eq!(hunt.shared.live.borrow().sensor(), Sensor::Missing);

		hunt.tick(HuntController::TICK_SECONDS);

		assert!(
			matches!(hunt.stage, Stage::Tracing(_)),
			"a browser or simulator traces at once"
		);
		assert_eq!(hunt.shared.live.borrow().tracking(), Tracking::Drag);
	}

	#[test]
	fn a_platform_that_stays_silent_is_let_go_after_the_wait_is_told() {
		let mut hunt = hunt();
		hunt.begin();
		hunt.shared.live.borrow_mut().expect_platform();

		hunt.tick(SIGIL_WAIT_SECONDS + HuntController::TICK_SECONDS);

		assert!(
			matches!(hunt.stage, Stage::Tracing(_)),
			"the drag takes over rather than waiting for ever"
		);
		assert_eq!(hunt.shared.live.borrow().sensor(), Sensor::Missing);
		assert_eq!(hunt.shared.live.borrow().tracking(), Tracking::Drag);
	}

	#[test]
	fn a_platform_that_answers_after_the_drag_hangs_the_first_light_again() {
		let mut hunt = hunt();
		hunt.begin();
		hunt.shared.live.borrow_mut().expect_platform();
		hunt.tick(SIGIL_WAIT_SECONDS + HuntController::TICK_SECONDS);
		assert!(hunt.sigil_on_drag, "the drag stood in for the platform");

		hunt.shared
			.live
			.borrow_mut()
			.track(turned(-2.0), 0.0, Tracking::Turning);
		hunt.tick(HuntController::TICK_SECONDS);

		let view = hunt.shared.live.borrow().pose().forward;
		let Stage::Tracing(trace) = &hunt.stage else {
			panic!("the sigil is still traced");
		};
		assert!(
			wrap_angle(trace.direction_of(0).yaw() - view.yaw()).abs() < 1e-9,
			"nothing had caught, so the first light moves to where the platform now points"
		);
		assert!(!hunt.sigil_on_drag);
	}

	#[test]
	fn a_fixed_sigil_is_not_re_anchored_when_tracking_pauses_and_returns() {
		let mut hunt = hunt();
		hunt.begin();
		hunt.shared.live.borrow_mut().expect_platform();
		hunt.shared
			.live
			.borrow_mut()
			.track(turned(1.0), 0.0, Tracking::Turning);
		hunt.tick(HuntController::TICK_SECONDS);

		let Stage::Tracing(trace) = &hunt.stage else {
			panic!("the sigil hangs");
		};
		let before: Vec<_> = (0..trace.sigil.lights.len())
			.map(|light| trace.direction_of(light))
			.collect();

		hunt.shared.live.borrow_mut().advance(2.0);
		hunt.tick(HuntController::TICK_SECONDS);
		hunt.shared
			.live
			.borrow_mut()
			.track(turned(4.2), 3.0, Tracking::Turning);
		hunt.tick(HuntController::TICK_SECONDS);

		let Stage::Tracing(trace) = &hunt.stage else {
			panic!("the sigil is still traced");
		};
		for (light, expected) in before.iter().enumerate() {
			assert_eq!(trace.direction_of(light), *expected);
		}
	}

	/// Feeds a real phone pose through `LivePose`, with the phone rolled a
	/// quarter turn. This exercises the same coordinates as Android's XR
	/// frames, instead of the autoplay controller or the skip button.
	fn point(hunt: &HuntState, target: Vec3, time: f64) {
		let pose = ArPose::looking(target, 0.0);
		let rotation = look_at(
			[0.0; 3],
			to_platform(target).map(|x| x as f32),
			to_platform(pose.right * -1.0).map(|x| x as f32),
		)
		.rotation
		.map(f64::from);
		hunt.shared
			.live
			.borrow_mut()
			.world_frame(rotation, [0.0; 3], time, true);
	}

	#[test]
	fn a_phone_can_find_the_sigil_then_face_the_husk_without_skipping() {
		let owner = Owner::new();
		owner.with(|| {
			for (seed, nft) in market_listings().into_iter().take(5).enumerate() {
				let props = HuntViewProps {
					brief: HuntBrief {
						nft,
						seed: seed as i64,
						loadout: HuntLoadout::BORROWED,
						sighting_id: None,
					},
					difficulty: Difficulty::default(),
					autoplay: false,
					on_finished: Arc::new(|_, _| {}),
					on_difficulty: None,
				};
				let mut hunt = HuntState::new(&props);
				point(&hunt, Vec3::from_angles(2.8, 0.0), 0.0);
				hunt.begin();
				hunt.tick(1.0 / 30.0);
				let mut ticks = 0;
				while let Stage::Tracing(trace) = &hunt.stage {
					assert!(ticks < 4000, "seed {seed} never found its sigil");
					let target = trace.direction_of(trace.caught());
					point(&hunt, target, f64::from(ticks) / 30.0);
					hunt.tick(1.0 / 30.0);
					ticks += 1;
				}
				assert_eq!(hunt.stage, Stage::Fighting);
				let controller = hunt.arena.controller.as_ref().unwrap();
				assert!(!controller.plays_itself());
				let target = controller.sim().pose().direction();
				point(&hunt, target, f64::from(ticks) / 30.0);
				let lens = hunt.lens();
				let (x, y) = lens
					.place(target)
					.expect("the husk is in front of the phone");
				assert!((x - lens.width / 2.0).abs() < 0.01);
				assert!((y - lens.height / 2.0).abs() < 0.01);
			}
		});
	}
}
