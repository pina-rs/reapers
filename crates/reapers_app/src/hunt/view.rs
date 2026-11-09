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
use reapers_world::RoomGridBuilder;
use reapers_world::RoomSurvey;
use reapers_world::sigil_plan;

use super::arena::Arena;
use super::controller::Fight;
use super::controller::HuntController;
use super::hud::Banner;
use super::hud::banner_for;
use super::hud::knock_for;
use super::hud::status;
use super::hud::status_view;
use super::lessons::Learned;
use super::lessons::Teacher;
use super::overlay::Lens;
use super::overlay::Sketch;
use super::overlay::fight;
use super::overlay::frame_class;
use super::overlay::sigil;
use super::overlay::veil;
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
		}
	}

	/// The threshold is crossed: the platform is asked for (this is a
	/// press, which browsers ask for), and the sigil hangs round wherever
	/// the player faces. A hunt that plays itself has no sigil to trace.
	fn begin(&mut self) {
		follow(&self.shared);
		let vessel = self.brief.nft.collection.vessel();
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
		let centre = self.shared.live.borrow().pose().forward.yaw();
		self.stage = Stage::Tracing(SigilTrace::new(self.sigil(), centre));
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
		let occlusion = self
			.shared
			.survey
			.borrow_mut()
			.take()
			.map(|survey| survey.occlusion().clone());
		let controller = if self.autoplay {
			HuntController::autoplay(Fight::Hunt(sim))
		} else {
			HuntController::new(Fight::Hunt(sim))
		};
		self.arena
			.engage(controller, occlusion, self.brief.loadout.weapon.kind);
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
			self.arena.anchor(position);
		}
		let mut closed = None;
		if let Stage::Tracing(trace) = &mut self.stage {
			for event in trace.advance(live_pose.forward, live_pose.turn_speed, delta) {
				match event {
					SigilEvent::LightCaught(_) => platform::thud(),
					SigilEvent::LightLost(_) => platform::tap(),
					SigilEvent::SigilClosed => closed = Some(trace.clone()),
				}
			}
		}
		match self.stage {
			Stage::Briefing | Stage::Tracing(_) => self.arena.idle(delta, live_pose),
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
		let events = self.arena.advance(delta, live_pose);
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
		}
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
	let mut marks = Sketch::default();
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
				);
			}
		}
		Stage::Briefing | Stage::Finished => {}
	}
	if !marks.is_empty() {
		root = root.child(marks.into_view(state.size.0, state.size.1));
	}
	root = match &state.stage {
		Stage::Briefing => root.child(briefing(&state, props)),
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
	let controls = super::controls::controls(&*words, &thumbs);
	View()
		.class("absolute inset-0")
		.child(status_view(&*words, &shown, state.light, None))
		.child(controls)
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
	let mut column = View()
		.class("absolute left-0 right-0 top-0 px-6 pt-4 pt-safe items-center")
		.test_id("hunt-scan")
		.child(label(words.scan_label(), 11, ASH))
		.child(gap(6))
		.child(display(words.scan_title(), 26, BONE))
		.child(gap(6))
		.child(figure(
			format!("{} / {}", trace.caught(), trace.sigil.lights.len()),
			13,
			state.light,
		));
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
	let mut view = View().class("absolute inset-0").child(column);
	if trace.seconds() > SKIP_AFTER {
		view = view.child(
			View()
				.class("absolute left-6 right-6 bottom-8 pb-safe")
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
