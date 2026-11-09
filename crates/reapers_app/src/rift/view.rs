//! A seat in a rift: four reapers round one husk at a real place. The
//! Flutter `RiftScreen`, less the session, which the route holds.
//!
//! Each reaper fights the husk through their own phone, exactly as in a
//! lone hunt, in the same [`Arena`]. What the circle shares is the ward, the
//! moments it has to hold still, and whether enough of it cut in the end.
//! This reaper's own `HuntSim` runs in rift mode as a `RiftSeat`, which is
//! brought up to date from every frame, and its standing goes up as whole
//! reports.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use leptoskit::prelude::*;
use reapers_rules::ClassPerk;
use reapers_rules::EscapeCause;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::RiftFrame;
use reapers_rules::RiftPhase;
use reapers_rules::RiftReport;
use reapers_rules::RiftRules;
use reapers_rules::RiftSeat;
use reapers_rules::RiftSite;
use reapers_world::RiftPlacer;

use crate::hunt::arena::Arena;
use crate::hunt::controller::Fight;
use crate::hunt::controller::HuntController;
use crate::hunt::hud::Banner;
use crate::hunt::hud::banner_for;
use crate::hunt::hud::knock_for;
use crate::hunt::hud::status;
use crate::hunt::hud::status_view;
use crate::hunt::overlay::Lens;
use crate::hunt::overlay::Sketch;
use crate::hunt::overlay::fight;
use crate::hunt::overlay::frame_class;
use crate::hunt::overlay::veil;
use crate::hunt::pose::Tracking;
use crate::hunt::xr::Shared;
use crate::hunt::xr::follow;
use crate::l10n::AppTranslations;
use crate::l10n::Messages;
use crate::l10n::use_l10n;
use crate::platform;
use crate::render::pointer::Gesture;
use crate::render::pointer::on_gesture;
use crate::render::scene::RIFT;
use crate::seams::RiftViewProps;
use crate::seams::viewmodel_view;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::WOUND;
use crate::ui::LedgerButton;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_heading;
use crate::ui::ledger_meter;
use crate::ui::ledger_row;
use crate::ui::light;
use crate::ui::margin;

/// A reaper who joins a fight already under way plays at most this much of
/// it standing still, to fall into step with the circle.
const MAX_CATCH_UP_TICKS: i64 = 2 * HuntRules::TICK_RATE;
/// Ticks between one report of this reaper's standing and the next.
const REPORT_EVERY_TICKS: i64 = 6;
/// How long a banner stays.
const BANNER_SECONDS: f64 = 1.3;

fn words() -> &'static dyn Messages {
	use leptoskit_l10n::Translations as _;
	AppTranslations::messages(&AppTranslations::supported_locales()[0])
}

/// Everything a rift keeps between renders.
#[allow(
	clippy::struct_excessive_bools,
	reason = "each is one thing the seat has done"
)]
pub struct RiftState {
	pub site: RiftSite,
	arena: Arena,
	shared: Shared,
	/// Who the reaper was on the way in: sealing the rift changes the
	/// player, and the fight must not change under them when it does.
	loadout: HuntLoadout,
	autoplay: bool,
	report: Arc<dyn Fn(RiftReport) + Send + Sync>,
	/// Where the rift stands for this reaper.
	placer: RiftPlacer,
	frame: Option<RiftFrame>,
	rested: bool,
	following: bool,
	last_report_tick: i64,
	reported_end: bool,
	banner: Option<(Banner, f64)>,
	seconds: f64,
	mounted: u32,
	ticking: bool,
	drawn: RwSignal<u64>,
	fingerprint: u64,
	light: Rgb,
	size: (f64, f64),
}

thread_local! {
	static CURRENT: RefCell<Option<Rc<RefCell<RiftState>>>> = const { RefCell::new(None) };
}

fn current() -> Option<Rc<RefCell<RiftState>>> {
	CURRENT.with_borrow(Clone::clone)
}

fn with_rift(action: impl FnOnce(&mut RiftState)) {
	if let Some(rift) = current() {
		action(&mut rift.borrow_mut());
	}
}

/// Presses FIRE once for the rift on screen.
fn fire() {
	with_rift(|rift| {
		if let Some(controller) = rift.arena.controller.as_mut() {
			controller.press();
		}
	});
}

/// Draws the weapon in `slot`.
fn select_slot(slot: usize) {
	with_rift(|rift| {
		if let Some(controller) = rift.arena.controller.as_mut() {
			controller.select(slot);
		}
	});
}

impl RiftState {
	fn new(props: &RiftViewProps, loadout: HuntLoadout) -> Self {
		let size = platform::viewport();
		let relics = props.site.relics();
		let first = relics[0];
		let accent = reapers_rules::NftLook::of(&first).accent;
		// As the hunt's: the camera's picture behind it in the native apps.
		let mut arena = Arena::new(
			RIFT,
			props.site.vessel(),
			&first,
			size.0 / size.1,
			cfg!(feature = "native"),
		);
		arena.height = size.1;
		let shared = Shared {
			scene: Some(arena.scene.id().0),
			..Shared::default()
		};
		Self {
			site: props.site,
			arena,
			shared,
			loadout,
			autoplay: props.autoplay,
			report: props.report.clone(),
			placer: RiftPlacer::new(),
			frame: None,
			rested: false,
			following: false,
			last_report_tick: -REPORT_EVERY_TICKS,
			reported_end: false,
			banner: None,
			seconds: 0.0,
			mounted: 0,
			ticking: false,
			drawn: crate::ui::lasting(|| RwSignal::new(0)),
			fingerprint: 0,
			light: Rgb((accent >> 16) as u8, (accent >> 8) as u8, accent as u8),
			size,
		}
	}

	/// Starts following the platform's pose: from a press, which browsers
	/// ask for.
	fn follow(&mut self) {
		if !self.following {
			self.following = true;
			follow(&self.shared);
		}
	}

	/// Takes in the rift's latest frame.
	fn on_frame(&mut self, next: &RiftFrame) {
		let pose = self.shared.live.borrow().pose();
		let fighting = matches!(next.phase, RiftPhase::Fight | RiftPhase::Reap);
		let engaged = self.arena.controller.is_some();
		if !engaged {
			// What the platform knows of north; the rotation sensor's
			// heading is not one, so without a compass the rift stands dead
			// ahead.
			let standing = self
				.placer
				.observe(next, pose.forward.yaw(), None, platform::now());
			if let Some(standing) = standing {
				(self.report)(RiftReport {
					standing: Some(standing),
					..RiftReport::readiness(next.you().is_some_and(|you| you.ready))
				});
			}
		}
		if fighting && !engaged {
			// Once the fight is on the husk stays where it was last seen:
			// moving it would turn the reaper's own blade away from it.
			if let Ok(mut seat) = RiftSeat::new(
				next.seed,
				self.site,
				next.ward_max,
				&self.loadout,
				self.placer.sighting,
			) {
				seat.catch_up(next.tick.min(MAX_CATCH_UP_TICKS));
				let controller = if self.autoplay {
					HuntController::autoplay(Fight::Rift(seat))
				} else {
					HuntController::new(Fight::Rift(seat))
				};
				self.arena
					.engage(controller, None, self.loadout.weapon.kind);
			}
		} else if !engaged && !self.rested {
			// Nothing is being fought yet, but the husk already stands where
			// it will be fought.
			if let Ok(stats) = HuntStats::derive(&self.loadout, 1.0) {
				self.rested = true;
				let sim = HuntSim::rift(
					next.seed,
					self.site.vessel(),
					1.0,
					&stats,
					self.placer.sighting,
				);
				self.arena.rest(sim.pose());
			}
		}
		if let Some(seat) = self
			.arena
			.controller
			.as_mut()
			.and_then(HuntController::seat_mut)
		{
			let counted = next.you().map_or(0.0, |you| you.dealt);
			seat.sync(next.ward, counted);
		}
		self.frame = Some(next.clone());
	}

	fn tick(&mut self, delta: f64) {
		self.seconds += delta;
		let live_pose = {
			let mut live = self.shared.live.borrow_mut();
			live.advance(delta);
			live.pose()
		};
		if let Some(position) = self.shared.live.borrow().position {
			self.arena.anchor(position);
		}
		if self.arena.controller.is_none() {
			self.arena.idle(delta, live_pose);
		} else {
			if !crate::render::pointer::follows_fingers()
				&& let Some(controller) = self.arena.controller.as_mut()
				&& !controller.plays_itself()
			{
				let holding = self.shared.live.borrow().holding;

				controller.hold_orb(holding);
			}
			let events = self.arena.advance(delta, live_pose);
			for (_, event) in &events {
				knock_for(event);
				if let Some(banner) = banner_for(words(), event) {
					self.banner = Some((banner, self.seconds));
				}
			}
			self.send_report();
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

	/// Sends this seat's whole standing, every few ticks and once when its
	/// part is over: never a change, so a lost or repeated one miscounts
	/// nothing.
	fn send_report(&mut self) {
		let Some(controller) = &self.arena.controller else {
			return;
		};
		let Some(seat) = controller.seat() else {
			return;
		};
		let over = seat.sim.is_over();
		let due = seat.sim.tick() - self.last_report_tick >= REPORT_EVERY_TICKS || over;
		if !due || self.reported_end {
			return;
		}
		self.last_report_tick = seat.sim.tick();
		self.reported_end = over;
		(self.report)(seat.report());
	}

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
		)
			.hash(&mut hasher);
		if let Some(controller) = &self.arena.controller {
			controller.ticks().hash(&mut hasher);
		}
		self.banner
			.as_ref()
			.map(|(banner, _)| banner.text.clone())
			.hash(&mut hasher);
		hasher.finish()
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

	fn gesture(&mut self, gesture: Gesture) {
		let per_unit = self.arena.fov / self.size.1 * 3.0;
		super::super::hunt::controls::steer(
			gesture,
			&mut self.shared.live.borrow_mut(),
			per_unit,
			self.arena.controller.as_mut(),
		);
	}
}

fn mount(rift: &Rc<RefCell<RiftState>>) {
	let mut state = rift.borrow_mut();
	state.mounted += 1;
	if !state.ticking {
		state.ticking = true;
		let ticking = Rc::downgrade(rift);
		crate::render::ticker::every_frame(move |delta| {
			let Some(rift) = ticking.upgrade() else {
				return false;
			};
			let mut rift = rift.borrow_mut();
			if rift.mounted == 0 {
				rift.ticking = false;
				rift.shared.done.set(true);
				return false;
			}
			rift.tick(delta);
			true
		});
		let listener = Rc::downgrade(rift);
		on_gesture(state.arena.scene.id(), move |gesture| {
			if let Some(rift) = listener.upgrade() {
				rift.borrow_mut().gesture(gesture);
			}
		});
	}
	drop(state);
	let unmount = send_wrapper::SendWrapper::new(Rc::downgrade(rift));
	reactive_graph::owner::on_cleanup(move || {
		if let Some(rift) = unmount.take().upgrade() {
			let mut rift = rift.borrow_mut();
			rift.mounted = rift.mounted.saturating_sub(1);
		}
	});
}

fn rift_for(props: &RiftViewProps) -> Rc<RefCell<RiftState>> {
	if let Some(rift) = current() {
		if rift.borrow().site == props.site {
			rift.borrow_mut().report = props.report.clone();
			return rift;
		}
		rift.borrow().shared.done.set(true);
	}
	let loadout = reactive_graph::owner::use_context::<crate::state::Game>()
		.and_then(|game| game.player())
		.map_or(HuntLoadout::BORROWED, |player| player.loadout());
	let rift = Rc::new(RefCell::new(RiftState::new(props, loadout)));
	CURRENT.with_borrow_mut(|current| *current = Some(rift.clone()));
	rift
}

/// The thumbs' controls for the circle's fight: the pistol and the art
/// alone, and nothing to teach.
fn thumbs(
	words: &dyn crate::l10n::Messages,
	state: &RiftState,
	controller: &HuntController,
	live: &crate::hunt::pose::LivePose,
) -> Element {
	let thumbs = super::super::hunt::controls::Thumbs {
		sim: controller.sim(),
		slot: controller.slot(),
		carried: std::array::from_fn(|slot| controller.carries(slot)),
		weapon: state.loadout.weapon.kind,
		light: state.light,
		aim: state.arena.pose(live.pose()).forward,
		lesson: None,
		touchable: !controller.plays_itself(),
		on_fire: fire,
		on_slot: select_slot,
	};
	super::super::hunt::controls::controls(words, &thumbs)
}

/// The rift: the circle gathering over the husk at rest, the fight in the
/// same arena as a hunt with the circle under the ward, and how it ended.
pub fn rift_view(props: &RiftViewProps) -> Element {
	let rift = rift_for(props);
	mount(&rift);
	{
		let mut state = rift.borrow_mut();
		let fresh = match (&props.frame, &state.frame) {
			(Some(next), Some(seen)) => {
				next.tick != seen.tick
					|| next.phase != seen.phase
					|| next.ward.to_bits() != seen.ward.to_bits()
			}
			(Some(_), None) => true,
			_ => false,
		};
		if fresh && let Some(next) = &props.frame {
			state.on_frame(next);
		}
		if state.autoplay && state.arena.controller.is_none() {
			state.follow();
		}
	}
	let state = rift.borrow();
	let _ = state.drawn.get();
	let words = use_l10n();
	let lens = state.lens();
	// Where the platform draws its camera's picture behind the scene, the
	// room shows through: no veil, and nothing opaque under the scene.
	let camera = state.shared.live.borrow().camera;
	let mut sketch = Sketch::default();
	if !camera {
		veil(&mut sketch, &lens);
	}
	let mut root = View()
		.class(frame_class(camera))
		.test_id("rift-view")
		.child(sketch.into_view(state.size.0, state.size.1))
		.child(
			Scene3d(state.arena.scene.id())
				.class("absolute inset-0 w-full h-full")
				.aria_hidden(true),
		);
	let over = props
		.frame
		.as_ref()
		.is_some_and(|frame| frame.phase.is_over());
	match (&state.arena.controller, &props.frame) {
		(Some(controller), Some(frame)) if !over => {
			if let Some(arms) = state.arena.viewmodel() {
				root = root.child(viewmodel_view(&arms));
			}
			let mut marks = Sketch::default();
			fight(
				&mut marks,
				&lens,
				controller.sim(),
				state.light,
				state.arena.wound,
				state.loadout.reaper_class.perk() == ClassPerk::Measure,
			);
			root = root.child(marks.into_view(state.size.0, state.size.1));
			let live = state.shared.live.borrow();
			let afterword = match controller.sim().outcome() {
				Some(outcome) if outcome.grade().is_some() => words.rift_cut_made(),
				Some(outcome) if outcome.escape() == Some(EscapeCause::Resolve) => {
					words.rift_cast_out()
				}
				Some(outcome) if outcome.escape() == Some(EscapeCause::Lapsed) => {
					words.rift_cut_missed()
				}
				_ => "",
			};
			let shown = status(
				&*words,
				props.site.title(),
				controller.sim(),
				state.arena.pose(live.pose()).turn_speed,
				live.tracking() != Tracking::Drag,
				state.banner.as_ref().map(|(banner, _)| banner.clone()),
				afterword,
			);
			let controls = thumbs(&*words, &state, controller, &live);
			root = root.child(
				View()
					.class("absolute inset-0")
					.child(status_view(
						&*words,
						&shown,
						state.light,
						Some(circle(&*words, frame, state.light)),
					))
					.child(controls),
			);
		}
		_ => root = root.child(lobby(props, &state)),
	}
	root
}

/// The circle during the fight: who is in it, how much of the ward each
/// has taken, and who has made their cut.
fn circle(words: &dyn Messages, frame: &RiftFrame, light: Rgb) -> Element {
	let fair_share = frame.ward_max / RiftRules::CIRCLE as f64;
	let mut row = View().class("flex-row gap-[10px]");
	for (index, member) in frame.members.iter().enumerate() {
		let you = index as i64 == frame.seat;
		let name = if you {
			words.rift_seat_you().to_owned()
		} else {
			member.name.clone()
		};
		let colour = if !member.standing {
			WOUND
		} else if you {
			BONE
		} else {
			ASH
		};
		let share = if member.cut.is_some() {
			1.0
		} else if fair_share > 0.0 {
			(member.dealt / fair_share).clamp(0.0, 1.0)
		} else {
			0.0
		};
		row = row.child(
			View()
				.class("flex-1")
				.accessibility_label(format!("{name}, {:.0}%", share * 100.0))
				.child(label(&name, 9, colour))
				.child(gap(4))
				.child(
					View()
						.class("w-full h-[2px] bg-[#e9e4d42e]")
						.child(View().class(format!(
							"h-full w-[{:.0}%] bg-[{}]",
							share * 100.0,
							if member.cut.is_some() {
								BONE.hex()
							} else {
								light.hex()
							}
						))),
				),
		);
	}
	let mut column = View().child(row);
	if frame.phase == RiftPhase::Reap {
		column = column.child(gap(8)).child(label(
			&words.rift_cuts(frame.cuts() as i64, frame.cuts_needed() as i64),
			11,
			BONE,
		));
	}
	column
}

/// Before the fight, and after it: where this is, who has come, standing
/// ready, and how it ended.
#[allow(clippy::too_many_lines, reason = "one ledger, read top to bottom")]
fn lobby(props: &RiftViewProps, state: &RiftState) -> Element {
	let words = use_l10n();
	let light = light();
	let leave = props.on_leave.clone();
	let mut column = View()
		.class("w-full max-w-[520px] self-center px-7 pt-6 pb-7")
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
						words.rift_begins(frame.ticks_left / HuntRules::TICK_RATE + 1),
						BONE,
					)
				}
				RiftPhase::Gathering => (words.rift_waiting().to_owned(), BONE),
				RiftPhase::Fight | RiftPhase::Reap => (words.rift_circle().to_owned(), BONE),
			};
			let placing = if state.placer.fix.is_none() {
				if state.placer.still_looking() {
					words.rift_finding()
				} else {
					words.rift_unplaced()
				}
			} else {
				words.rift_no_compass()
			};
			column = column.child(gap(14)).child(body(title, 15, colour));
			if frame.phase == RiftPhase::Gathering {
				column = column.child(gap(6)).child(body(placing, 13, ASH));
			}
			column = column
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
				let seat_state = if member.shade {
					words.rift_seat_shade()
				} else if member.ready {
					words.rift_seat_ready()
				} else {
					words.rift_seat_waiting()
				};
				column = column.child(ledger_row(&member.name, seat_state, member.ready));
			}
			if frame.phase == RiftPhase::Gathering && !frame.you().is_some_and(|you| you.ready) {
				let report = props.report.clone();
				column = column.child(gap(18)).child(
					LedgerButton::new(words.rift_stand_ready())
						.test_id("rift-ready")
						.on_press(move || {
							// Standing ready is the press the browser needs to
							// follow the phone's turning.
							with_rift(RiftState::follow);
							report(RiftReport::readiness(true));
						}),
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
		.class("absolute inset-0 pt-safe pb-safe bg-[#07080bc7]")
		.child(column)
}
