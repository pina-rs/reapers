//! The arms bench (`REAPERS_ARMS_BENCH`): the reaper's first-person arms,
//! its weapons, their blows and the capture orb, played on their own over
//! a husk, against a dark, a bright or a grey background, at full speed or
//! a quarter of it.
//!
//! It keeps its own clock in the hunt's ticks and drives the viewmodel seam
//! as a hunt would: an action and the tick it began, each blow's juice
//! landed on the tick it strikes, and the orb as the rules would report it,
//! on their own ticks. It runs no hunt: it stands in for one.

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use leptoskit::native::animation::Easing;
use leptoskit::native::animation::Transition;
use leptoskit::prelude::*;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use reapers_rules::Archetype;
use reapers_rules::CaptureGrade;
use reapers_rules::CaptureRules;
use reapers_rules::HuskPose;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use reapers_rules::OrbStage;
use reapers_rules::OrbView;
use reapers_rules::Vec3;
use reapers_rules::WeaponKind;

use crate::l10n::use_l10n;
use crate::render::arms::CONTACT;
use crate::render::arms::THROW_RELEASE;
use crate::render::arms::TICKS_PER_SECOND;
use crate::render::husk::HuskMood;
use crate::render::husk::HuskRig;
use crate::render::juice::dive_zoom;
use crate::render::juice::zoomed_fov;
use crate::render::orb::stage_began;
use crate::render::reaction::HuskPlight;
use crate::render::scene::Key;
use crate::render::scene::light_hunt;
use crate::render::scene::scene;
use crate::render::turn::multiply;
use crate::render::turn::view_turn;
use crate::seams::HitView;
use crate::seams::Jolt;
use crate::seams::Juice;
use crate::seams::ViewAction;
use crate::seams::ViewmodelProps;
use crate::seams::WorldView;
use crate::seams::dive;
use crate::seams::shake_turn;
use crate::seams::viewmodel_show;
use crate::seams::viewmodel_view;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::INK;
use crate::ui::body;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_back;
use crate::ui::ledger_row;
use crate::ui::margin;

/// The bench's world: the husk, and the orb and effects around it.
pub const WORLD: Key = Key::Fixed("arms-bench");
/// The view's size, in CSS pixels: a phone held upright.
pub const VIEW_WIDTH: f32 = 360.0;
pub const VIEW_HEIGHT: f32 = 480.0;
/// The world's lens: the hunt's.
const FOV: f32 = 62.0 * std::f32::consts::PI / 180.0;
/// Where the husk stands, in the hunt's space: metres from the eye, z
/// ahead.
const HUSK_AT: [f64; 3] = [0.0, -0.5, 3.2];
/// How far below the eye the floor is.
const FLOOR: f32 = -1.45;
/// How high above the ground the orb's middle rests.
const ORB_REST: f32 = 0.126;
/// Where a throw that misses comes down, in the hunt's space: on the floor,
/// past the husk and to its right.
const MISS_AT: [f64; 3] = [1.1, -1.32, 4.6];
/// Every fourth blow is a critical one.
const CRIT_EVERY: u32 = 4;
/// The class weapon in the bench's heavy slot.
const HEAVY: WeaponKind = WeaponKind::Scythe;

/// How a throw on the bench ends. In a hunt, the rules decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
	/// It holds, and the husk is captured.
	Holds,
	/// The soul breaks out on pulse 1, 2 or 3.
	BreaksOn(i64),
	/// It misses the husk.
	Misses,
}

/// What the arms are drawn over.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ground {
	#[default]
	Dark,
	/// A bright room, as a camera might see one by day.
	Bright,
	Grey,
}

/// The bench between renders.
pub struct Bench {
	/// The bench's clock, in the hunt's ticks.
	pub tick: f64,
	pub quarter: bool,
	pub looking: bool,
	look_clock: f64,
	pub ground: Ground,
	/// What the selected slot holds.
	pub archetype: Archetype,
	pub action: ViewAction,
	action_tick: i64,
	stabs: u8,
	blows: u32,
	juice: Juice,
	hit: Option<HitView>,
	/// A blow the husk will feel: the tick, and how hard.
	flash: Option<(i64, f64)>,
	/// The orb: the tick it leaves the hand, and how it ends.
	pub orb: Option<(i64, Ending)>,
	/// Whether the orb's blows have landed.
	orb_jolts: (bool, bool),
	husk: Option<HuskRig>,
	/// How much of the husk the orb has drawn in, from 0 to 1.
	taken: f64,
	/// The world's camera, and its lens, as last set.
	camera: (Transform, f32),
	sent: Option<ViewmodelProps>,
	/// Bumped by every press, to draw the page again.
	pub version: RwSignal<u64>,
	/// What the orb is doing, for the page to say: its stage, and the pulse
	/// it is on or broke on.
	pub stage: RwSignal<Option<(OrbStage, i64)>>,
	ticking: bool,
}

thread_local! {
	static BENCH: RefCell<Option<Rc<RefCell<Bench>>>> = const { RefCell::new(None) };
}

pub fn bench() -> Rc<RefCell<Bench>> {
	BENCH.with_borrow_mut(|bench| {
		bench
			.get_or_insert_with(|| {
				Rc::new(RefCell::new(Bench {
					tick: 0.0,
					quarter: false,
					looking: false,
					look_clock: 0.0,
					ground: Ground::Dark,
					archetype: Archetype::Gun,
					action: ViewAction::Draw,
					action_tick: 0,
					stabs: 0,
					blows: 0,
					juice: Juice::default(),
					hit: None,
					flash: None,
					orb: None,
					orb_jolts: (false, false),
					husk: None,
					taken: 0.0,
					camera: (Transform::IDENTITY, FOV),
					sent: None,
					version: crate::ui::lasting(|| RwSignal::new(0)),
					stage: crate::ui::lasting(|| RwSignal::new(None)),
					ticking: false,
				}))
			})
			.clone()
	})
}

/// Starts the bench's clock and puts the husk in its world, once.
pub fn start(bench: &Rc<RefCell<Bench>>) {
	let mut state = bench.borrow_mut();
	if state.ticking {
		return;
	}
	state.ticking = true;
	let world = scene(WORLD);
	let ops = world.ops();
	light_hunt(ops, false);
	let nft = Nft::new(NftCollection::DegenApes, 1);
	let mut husk = HuskRig::spawn(ops, nft.collection.vessel(), &nft);
	let [x, y, z] = HUSK_AT;
	husk.place(&HuskPose::new(
		nft.collection.vessel(),
		Vec3::new(x, y, z),
		PI,
	));
	state.husk = Some(husk);
	world.flush();
	drop(state);
	let ticking = Rc::downgrade(bench);
	crate::render::ticker::every_frame(move |delta| {
		let Some(bench) = ticking.upgrade() else {
			return false;
		};
		bench.borrow_mut().advance(delta);
		true
	});
}

impl Bench {
	/// The bench's clock as the hunt's tick.
	pub fn now(&self) -> i64 {
		self.tick.floor() as i64
	}

	/// How many of the hunt's ticks pass each second.
	fn rate(&self) -> f32 {
		if self.quarter {
			TICKS_PER_SECOND / 4.0
		} else {
			TICKS_PER_SECOND
		}
	}

	/// The husk's middle, and the ground under it, in the scene.
	fn husk_at() -> ([f32; 3], [f32; 3]) {
		let [x, y, z] = HUSK_AT.map(|value| value as f32);
		([x, y, -z], [x, FLOOR + ORB_REST, -z])
	}

	/// Plays `action` from now.
	pub fn play(&mut self, action: ViewAction) {
		self.action = action;
		self.action_tick = self.now();
		let now = self.action_tick;
		match action {
			ViewAction::Fire => self.blow(now, 14, Jolt::Shot),
			ViewAction::Stab { .. } => self.blow(now + 5, 9, Jolt::Stab),
			ViewAction::Swing => {
				let contact = now + (CONTACT * TICKS_PER_SECOND).round() as i64;
				self.blow(contact, 38, Jolt::Heavy(HEAVY));
			}
			ViewAction::Parry => self.juice.land(Jolt::Parry, now + 5),
			ViewAction::Struck => self.juice.land(Jolt::Struck, now),
			_ => {}
		}
		self.version.update(|version| *version += 1);
	}

	/// A blow that strikes the husk at `tick`.
	fn blow(&mut self, tick: i64, damage: i64, jolt: Jolt) {
		self.blows += 1;
		let crit = self.blows.is_multiple_of(CRIT_EVERY);
		let jolt = match jolt {
			Jolt::Shot if crit => Jolt::SealShot,
			jolt => jolt,
		};
		self.juice.land(jolt, tick);
		let (husk, _) = Self::husk_at();
		// Each blow lands a little apart from the last, on the husk's near
		// side.
		let spread = (f64::from(self.blows) * 2.4).sin() as f32;
		let color = self
			.husk
			.as_ref()
			.map_or([0.8, 0.3, 0.9], |husk| husk.light);
		self.hit = Some(HitView {
			tick,
			at: [
				husk[0] + 0.12 * spread,
				husk[1] + 0.08 * spread.abs(),
				husk[2] + 0.32,
			],
			damage: if crit { damage * 5 / 2 } else { damage },
			crit,
			color,
		});
		self.flash = Some((tick, if crit { 1.0 } else { 0.6 }));
	}

	/// The next stab, from the other side.
	pub fn stab(&mut self) {
		let nth = self.stabs;
		self.stabs = (self.stabs + 1) % 3;
		self.play(ViewAction::Stab { nth });
	}

	/// Selects the slot of `archetype` and draws its weapon.
	pub fn take(&mut self, archetype: Archetype) {
		self.archetype = archetype;
		self.play(ViewAction::Draw);
	}

	/// Throws the orb, to end as `ending`.
	pub fn throw(&mut self, ending: Ending) {
		self.play(ViewAction::Throw);
		let release = self.action_tick + (THROW_RELEASE * TICKS_PER_SECOND).round() as i64;
		self.orb = Some((release, ending));
		self.orb_jolts = (false, false);
	}

	/// The orb at `tick`, once it has left the hand, as the rules report one
	/// (`HuntSim::orb`) that ends as it was told to.
	fn orb_view(&self, tick: i64) -> Option<OrbView> {
		let (release, ending) = self.orb?;
		let ticks = tick - release;
		if ticks < 0 {
			return None;
		}
		let on_target = ending != Ending::Misses;
		let broke_free_on = match ending {
			Ending::BreaksOn(pulse) => Some(pulse.clamp(1, 3)),
			Ending::Holds | Ending::Misses => None,
		}
		.filter(|&pulse| ticks >= CaptureRules::PULSE_TICKS[pulse as usize - 1]);
		let pulses = CaptureRules::PULSE_TICKS
			.iter()
			.filter(|&&at| ticks >= at)
			.count() as i64;
		let stage = if !on_target {
			if ticks < CaptureRules::IMPACT_TICK {
				OrbStage::Flight
			} else {
				OrbStage::Missed
			}
		} else if broke_free_on.is_some() {
			OrbStage::BrokeFree
		} else if ticks >= CaptureRules::RESULT_TICK {
			OrbStage::Captured
		} else if ticks < CaptureRules::IMPACT_TICK {
			OrbStage::Flight
		} else if ticks < CaptureRules::DRAWN_TICK {
			OrbStage::Drawing
		} else if ticks < CaptureRules::LANDED_TICK {
			OrbStage::Falling
		} else if ticks < CaptureRules::PULSE_TICKS[2] {
			OrbStage::Pulsing
		} else {
			OrbStage::Breath
		};
		let [x, y, z] = if on_target { HUSK_AT } else { MISS_AT };
		Some(OrbView {
			thrown_at: release,
			ticks,
			stage,
			grade: CaptureGrade::Plain,
			on_target,
			pulses: broke_free_on.map_or(pulses, |pulse| pulses.min(pulse - 1)),
			target: Vec3::new(x, y, z),
			// The view never reads the chance.
			chance: None,
			broke_free_on,
		})
	}
}

/// How long a sealed orb lies there before the bench clears it, in ticks.
const SEALED_FOR: i64 = 180;
/// How long a husk takes to gather itself again, in ticks.
const REFORM: f64 = 30.0;

impl Bench {
	/// Moves the bench's clock on by `delta` seconds and shows the moment.
	fn advance(&mut self, delta: f64) {
		let speed = if self.quarter { 0.25 } else { 1.0 };
		self.tick += delta * f64::from(TICKS_PER_SECOND) * speed;
		let now = self.now();
		if let Some((at, strength)) = self.flash
			&& now >= at
		{
			self.flash = None;
			if let Some(husk) = &mut self.husk {
				husk.flash(strength);
			}
		}
		self.land_orb_jolts(now);
		let orb = self.orb_view(now);
		if let Some(view) = orb
			&& matches!(
				view.stage,
				OrbStage::Captured | OrbStage::BrokeFree | OrbStage::Missed
			)
			&& view.ticks - stage_began(&view) > SEALED_FOR
		{
			self.orb = None;
		}
		let stage = orb.map(|view| (view.stage, view.broke_free_on.unwrap_or(view.pulses)));
		if self.stage.get_untracked() != stage {
			self.stage.set(stage);
		}
		let world = scene(WORLD);
		let ops = world.ops();
		let (camera, fov) = self.aim(delta * speed);
		self.camera = (camera, fov);
		ops.push(Op::SetCamera {
			transform: camera,
			fov_y: fov,
			near: 0.05,
			far: 60.0,
		});
		if let Some(husk) = &mut self.husk {
			// A soul let go gathers itself again; it is never there at once.
			let taken = soul_taken(orb, self.tick);
			let gathered = delta * speed * f64::from(TICKS_PER_SECOND) / REFORM;
			self.taken = taken.max(self.taken - gathered);
			husk.dissolve(self.taken);
			husk.animate(
				ops,
				delta * speed,
				HuskMood::default(),
				HuskPlight::default(),
			);
		}
		world.flush();
		let props = self.props();
		if self.sent.is_none_or(|sent| still(&sent) != still(&props)) {
			viewmodel_show(&props);
			self.sent = Some(props);
		}
	}

	/// The orb striking the husk, and the soul breaking out, on their ticks.
	fn land_orb_jolts(&mut self, now: i64) {
		let Some((release, ending)) = self.orb else {
			return;
		};
		let impact = release + CaptureRules::IMPACT_TICK;
		if !self.orb_jolts.0 && ending != Ending::Misses && now >= impact {
			self.orb_jolts.0 = true;
			self.juice.land(Jolt::OrbImpact, impact);
			self.flash = Some((impact, 0.8));
		}
		if let Ending::BreaksOn(pulse) = ending {
			let broke = release + CaptureRules::PULSE_TICKS[pulse.clamp(1, 3) as usize - 1];
			if !self.orb_jolts.1 && now >= broke {
				self.orb_jolts.1 = true;
				self.juice.land(Jolt::OrbBreak, broke);
			}
		}
	}

	/// The world's camera at the eye: turned by a slow look around, if on,
	/// narrowed by the dive lens and shaken. The background zooms with the
	/// lens on the page; the shake never moves it.
	fn aim(&mut self, delta: f64) -> (Transform, f32) {
		if self.looking {
			self.look_clock += delta;
		}
		let yaw = (0.22 * (self.look_clock * 1.1).sin()) as f32;
		let fov = zoomed_fov(FOV, self.zoom());
		let shake = shake_turn(&self.juice, self.tick, fov, VIEW_HEIGHT);
		let camera = Transform {
			translation: [0.0; 3],
			rotation: multiply(view_turn(yaw, 0.0, 0.0), shake),
			scale: [1.0; 3],
		};
		(camera, fov)
	}

	/// How far the dive lens has zoomed in.
	pub fn zoom(&self) -> f32 {
		let since = self.props().action_seconds(self.tick);
		dive_zoom(dive(self.action, since), HUSK_AT[2] as f32)
	}

	/// How far the dive lens will have zoomed once the dive is done.
	pub fn zoom_target(&self) -> f32 {
		let dived = matches!(self.action, ViewAction::DiveIn | ViewAction::Stab { .. });
		dive_zoom(if dived { 1.0 } else { 0.0 }, HUSK_AT[2] as f32)
	}

	/// What the seam is shown.
	pub fn props(&self) -> ViewmodelProps {
		let (husk, ground) = Self::husk_at();
		let looking = if self.looking { 1.0 } else { 0.0 };
		ViewmodelProps {
			archetype: self.archetype,
			heavy: HEAVY,
			action: self.action,
			action_tick: self.action_tick,
			now: self.now(),
			rate: self.rate(),
			turn_rate: [
				(0.22 * 1.1 * (self.look_clock * 1.1).cos()) as f32 * looking,
				0.0,
			],
			juice: self.juice,
			orb: self.orb_view(self.now()),
			world: Some(WorldView {
				scene: WORLD,
				camera: self.camera.0,
				fov_y: self.camera.1,
				aspect: VIEW_WIDTH / VIEW_HEIGHT,
				husk,
				ground,
			}),
			hit: self.hit,
		}
	}
}

/// `props` without what moves every tick by itself: the clock, which the
/// seam keeps, and the camera's shake.
fn still(props: &ViewmodelProps) -> ViewmodelProps {
	let mut props = *props;
	props.now = 0;
	if let Some(orb) = &mut props.orb {
		orb.ticks = 0;
	}
	if let Some(world) = &mut props.world {
		world.camera = Transform::IDENTITY;
	}
	props
}

/// How much of the husk the orb has drawn in at `tick`, from 0 to 1.
fn soul_taken(orb: Option<OrbView>, tick: f64) -> f64 {
	let Some(view) = orb else {
		return 0.0;
	};
	let since = tick - (view.thrown_at + CaptureRules::IMPACT_TICK) as f64;
	match view.stage {
		OrbStage::Flight | OrbStage::Missed | OrbStage::BrokeFree => 0.0,
		OrbStage::Drawing => {
			(since / (CaptureRules::DRAWN_TICK - CaptureRules::IMPACT_TICK) as f64).clamp(0.0, 1.0)
		}
		OrbStage::Falling | OrbStage::Pulsing | OrbStage::Breath | OrbStage::Captured => 1.0,
	}
}

/// A room by day, as a camera might see it: a window, a sunlit floor, a
/// table and a lamp, in the view's own pixels, painted in order. Bright and
/// busy, to see the arms and the light hold up against it.
const BRIGHT_ROOM: &[(Shape, u32)] = &[
	(Shape::Path("M0 0h360v300h-360z"), 0x00E8_E1D2),
	(Shape::Path("M250 0h110v300h-110z"), 0x00DD_D4C2),
	(Shape::Path("M34 34h162v182h-162z"), 0x00CF_C6B5),
	(Shape::Path("M40 40h150v170h-150z"), 0x00EE_F6FF),
	(Shape::Ellipse(92.0, 86.0, 38.0, 14.0), 0x00FF_FFFF),
	(Shape::Path("M112 40h6v170h-6z"), 0x00CF_C6B5),
	(Shape::Path("M40 122h150v6h-150z"), 0x00CF_C6B5),
	(Shape::Path("M230 60h90v70h-90z"), 0x003A_4A5C),
	(Shape::Path("M236 66h78v58h-78z"), 0x008F_B3C9),
	(Shape::Path("M0 300h360v180h-360z"), 0x00C9_A77C),
	(Shape::Path("M0 342h360v2h-360z"), 0x00B8_956A),
	(Shape::Path("M0 392h360v2h-360z"), 0x00B8_956A),
	(Shape::Path("M0 452h360v2h-360z"), 0x00B8_956A),
	(Shape::Path("M60 304L200 304L262 480L14 480z"), 0x00F3_E2C2),
	(Shape::Path("M0 296h360v8h-360z"), 0x00F4_EFE4),
	(Shape::Path("M230 250h112v10h-112z"), 0x006E_4F33),
	(Shape::Path("M240 260h8v84h-8z"), 0x005A_3F28),
	(Shape::Path("M324 260h8v84h-8z"), 0x005A_3F28),
	(
		Shape::Path("M270 170L300 170L312 200L258 200z"),
		0x00FF_F4D6,
	),
	(Shape::Path("M283 200h4v50h-4z"), 0x0044_4444),
	(Shape::Path("M150 266h40v40h-40z"), 0x00B5_653E),
	(Shape::Ellipse(160.0, 248.0, 12.0, 26.0), 0x005C_8A4A),
	(Shape::Ellipse(180.0, 244.0, 12.0, 30.0), 0x006F_9A57),
	(Shape::Ellipse(170.0, 236.0, 10.0, 30.0), 0x004F_7A3F),
];

/// One shape of the bright room.
#[derive(Clone, Copy)]
enum Shape {
	/// SVG path data.
	Path(&'static str),
	/// An ellipse's middle and radii.
	Ellipse(f32, f32, f32, f32),
}

/// The bright room as a drawing filling the view.
fn bright_room() -> Element {
	let Ok(view_box) = ViewBox::new(0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT) else {
		return View();
	};
	let mut room = Vector(view_box);
	for &(shape, colour) in BRIGHT_ROOM {
		let path = match shape {
			Shape::Path(data) => PathData::parse(data),
			Shape::Ellipse(x, y, across, up) => PathData::ellipse(x, y, across, up),
		};
		if let Ok(path) = path {
			let [_, red, green, blue] = colour.to_be_bytes();
			room = room.path(VectorPath::new(path).fill(Color::rgba(red, green, blue, 0xFF)));
		}
	}
	room.class("absolute inset-0 w-full h-full")
}

/// The bench.
pub fn page() -> Element {
	let bench = bench();
	start(&bench);
	let words = use_l10n();
	let state = bench.borrow();
	let _ = state.version.get();
	let props = state.props();
	let viewport = View()
		.class(format!(
			"relative w-[{VIEW_WIDTH}px] h-[{VIEW_HEIGHT}px] self-center overflow-hidden"
		))
		.test_id("arms-view")
		.child(plate(state.ground, state.zoom_target()))
		.child(
			Scene3d(scene(WORLD).id())
				.class("absolute inset-0 w-full h-full")
				.aria_hidden(true),
		)
		.child(viewmodel_view(&props));
	let column = View()
		.class("w-full max-w-[520px] self-center px-5 pt-2 pb-10")
		.test_id("arms-bench")
		.child(ledger_back(None))
		.child(gap(10))
		.child(display(words.arms_bench(), 30, BONE))
		.child(gap(6))
		.child(body(words.arms_bench_about(), 14, ASH))
		.child(gap(14))
		.child(viewport)
		.child(gap(10))
		.child(ledger_row(
			words.arms_bench_doing(),
			action_name(state.action),
			false,
		))
		.child(orb_row())
		.child(controls(&state));
	ScrollView()
		.class("flex-1 w-full pt-safe pb-safe")
		.child(column)
}

/// What the arms are drawn over, zoomed by the dive lens about its middle.
fn plate(ground: Ground, zoom: f32) -> Element {
	let backing = match ground {
		Ground::Dark => View().class("absolute inset-0 bg-[#07080b]"),
		Ground::Grey => View().class("absolute inset-0 bg-[#7f7f7f]"),
		Ground::Bright => bright_room(),
	};
	let millis = if zoom > 1.0 {
		crate::render::arms::DIVE_IN_SECONDS
	} else {
		crate::render::arms::DIVE_OUT_SECONDS
	} * 1000.0;
	View()
		.class("absolute inset-0")
		.aria_hidden(true)
		.transform(leptoskit::native::Transform::scale(zoom))
		.transition(Transition::timing(millis, Easing::EASE_IN_OUT))
		.child(backing)
}

/// The orb's stage, kept up to date as it changes without drawing the page
/// again.
fn orb_row() -> Element {
	let words = use_l10n();
	let stage = bench().borrow().stage;
	let names: Vec<String> = [
		words.arms_bench_in_hand().to_owned(),
		words.arms_bench_flying().to_owned(),
		words.arms_bench_absorbing().to_owned(),
		words.arms_bench_falling().to_owned(),
		words.arms_bench_breath().to_owned(),
		words.arms_bench_sealed().to_owned(),
		words.arms_bench_missed().to_owned(),
	]
	.into_iter()
	.chain((1..=3).map(|pulse| words.arms_bench_pulse(pulse)))
	.chain((1..=3).map(|pulse| words.arms_bench_broken(pulse)))
	.collect();
	View()
		.class("flex-row items-end py-[7px] flex-wrap")
		.child(margin(words.arms_bench_orb()))
		.child(View().class("flex-1 min-w-[16px]"))
		.child(
			Text(move || {
				let index = match stage.get() {
					None => 0,
					Some((OrbStage::Flight, _)) => 1,
					Some((OrbStage::Drawing, _)) => 2,
					Some((OrbStage::Falling, _)) => 3,
					Some((OrbStage::Breath, _)) => 4,
					Some((OrbStage::Captured, _)) => 5,
					Some((OrbStage::Missed, _)) => 6,
					Some((OrbStage::Pulsing, pulse)) => 6 + pulse.clamp(1, 3) as usize,
					Some((OrbStage::BrokeFree, pulse)) => 9 + pulse.clamp(1, 3) as usize,
				};
				names[index].clone()
			})
			.class(format!("text-[15px] font-medium text-[{}]", BONE.hex())),
		)
}

/// Every button: the weapon, its motions, the orb, the background and the
/// clock.
fn controls(state: &Bench) -> Element {
	let words = use_l10n();
	let weapons = [
		(
			Archetype::Gun,
			"arms-pistol",
			words.arms_bench_grave_pistol(),
		),
		(
			Archetype::Blade,
			"arms-knife",
			words.arms_bench_bone_knife(),
		),
		(Archetype::Heavy, "arms-scythe", words.arms_bench_scythe()),
	]
	.map(|(archetype, id, name)| {
		chip(id, name, state.archetype == archetype, move |bench| {
			bench.take(archetype);
		})
	});
	let own: &[ViewAction] = match state.archetype {
		Archetype::Gun => {
			&[
				ViewAction::Fire,
				ViewAction::Empty,
				ViewAction::Reload,
				ViewAction::ActiveReload,
				ViewAction::Jam,
			]
		}
		Archetype::Blade => {
			&[
				ViewAction::DiveIn,
				ViewAction::Stab { nth: 0 },
				ViewAction::DiveOut,
			]
		}
		Archetype::Heavy => &[ViewAction::Swing],
		Archetype::Art => &[],
	};
	let motions = [ViewAction::Idle, ViewAction::Draw]
		.iter()
		.chain(own)
		.chain(&[ViewAction::Parry, ViewAction::Struck, ViewAction::Raise])
		.map(|&action| {
			let chosen = same_motion(state.action, action);
			let id = format!("arms-{}", action_id(action));
			chip(&id, action_name(action), chosen, move |bench| {
				match action {
					ViewAction::Stab { .. } => bench.stab(),
					action => bench.play(action),
				}
			})
		});
	let throws = throws(state);
	let grounds = [
		(Ground::Dark, "arms-dark", words.arms_bench_dark()),
		(Ground::Bright, "arms-bright", words.arms_bench_bright()),
		(Ground::Grey, "arms-grey", words.arms_bench_grey()),
	]
	.map(|(ground, id, name)| {
		chip(id, name, state.ground == ground, move |bench| {
			bench.ground = ground;
			bench.version.update(|version| *version += 1);
		})
	});
	let clock = [
		chip(
			"arms-full-speed",
			words.arms_bench_full_speed(),
			!state.quarter,
			|bench| {
				bench.set_quarter(false);
			},
		),
		chip(
			"arms-quarter-speed",
			words.arms_bench_quarter_speed(),
			state.quarter,
			|bench| {
				bench.set_quarter(true);
			},
		),
		chip(
			"arms-look",
			words.arms_bench_look_around(),
			state.looking,
			|bench| {
				bench.looking = !bench.looking;
				bench.version.update(|version| *version += 1);
			},
		),
	];
	View()
		.class("w-full gap-[6px] pt-3")
		.child(section(words.arms_bench_weapon(), weapons))
		.child(section(words.arms_bench_motion(), motions))
		.child(section(words.arms_bench_orb(), throws))
		.child(section(words.arms_bench_behind(), grounds))
		.child(section(words.arms_bench_clock(), clock))
}

/// The orb's buttons: a throw to each ending.
fn throws(state: &Bench) -> [Element; 5] {
	let words = use_l10n();
	[
		Ending::Holds,
		Ending::BreaksOn(1),
		Ending::BreaksOn(2),
		Ending::BreaksOn(3),
		Ending::Misses,
	]
	.map(|ending| {
		let (id, name) = match ending {
			Ending::Holds => {
				(
					"arms-seal".to_owned(),
					words.arms_bench_orb_seals().to_owned(),
				)
			}
			Ending::BreaksOn(pulse) => {
				(
					format!("arms-break-{pulse}"),
					words.arms_bench_orb_breaks(pulse),
				)
			}
			Ending::Misses => {
				(
					"arms-miss".to_owned(),
					words.arms_bench_orb_misses().to_owned(),
				)
			}
		};
		let chosen = state.orb.is_some_and(|(_, thrown)| thrown == ending);
		chip(&id, &name, chosen, move |bench| bench.throw(ending))
	})
}

/// A heading over a row of buttons that wraps.
fn section(heading: &str, buttons: impl IntoIterator<Item = Element>) -> Element {
	let mut row = View().class("flex-row flex-wrap gap-2");
	for button in buttons {
		row = row.child(button);
	}
	View()
		.class("w-full gap-2 pt-2")
		.child(margin(heading))
		.child(row)
}

/// A small button, filled in while it is the one chosen, found by tests as
/// `id`.
fn chip(
	id: &str,
	text: &str,
	chosen: bool,
	press: impl Fn(&mut Bench) + Send + Sync + 'static,
) -> Element {
	let (colour, filled) = if chosen {
		(INK, "bg-[#e9e4d4]")
	} else {
		(BONE, "bg-[#0f1116]")
	};
	Pressable(text.to_owned())
		.class(format!(
			"min-h-[44px] min-w-[44px] px-3 justify-center items-center border border-[#e9e4d45c] {filled}"
		))
		.aria_pressed(chosen)
		.test_id(id)
		.on_press(move || {
			crate::platform::tap();
			let bench = bench();
			press(&mut bench.borrow_mut());
		})
		.child(label(text, 12, colour))
}

impl Bench {
	fn set_quarter(&mut self, quarter: bool) {
		self.quarter = quarter;
		self.version.update(|version| *version += 1);
	}
}

/// Whether `shown` is `action`, any stab being any other.
fn same_motion(shown: ViewAction, action: ViewAction) -> bool {
	matches!(
		(shown, action),
		(ViewAction::Stab { .. }, ViewAction::Stab { .. })
	) || shown == action
}

/// What tests find a motion's button by.
const fn action_id(action: ViewAction) -> &'static str {
	match action {
		ViewAction::Idle => "idle",
		ViewAction::Draw => "draw",
		ViewAction::Fire => "fire",
		ViewAction::Empty => "empty",
		ViewAction::Reload => "reload",
		ViewAction::ActiveReload => "active-reload",
		ViewAction::Jam => "jam",
		ViewAction::DiveIn => "dive-in",
		ViewAction::Stab { .. } => "stab",
		ViewAction::DiveOut => "dive-out",
		ViewAction::Swing => "swing",
		ViewAction::Parry => "parry",
		ViewAction::Struck => "struck",
		ViewAction::Raise => "raise",
		ViewAction::Throw => "throw",
	}
}

/// The name a motion goes by on its button.
fn action_name(action: ViewAction) -> &'static str {
	let words = use_l10n();
	match action {
		ViewAction::Idle => words.arms_bench_idle(),
		ViewAction::Draw => words.arms_bench_draw(),
		ViewAction::Fire => words.arms_bench_fire(),
		ViewAction::Empty => words.arms_bench_empty(),
		ViewAction::Reload => words.arms_bench_reload(),
		ViewAction::ActiveReload => words.arms_bench_active_reload(),
		ViewAction::Jam => words.arms_bench_jam(),
		ViewAction::DiveIn => words.arms_bench_dive_in(),
		ViewAction::Stab { .. } => words.arms_bench_stab(),
		ViewAction::DiveOut => words.arms_bench_dive_out(),
		ViewAction::Swing => words.arms_bench_swing(),
		ViewAction::Parry => words.arms_bench_parry(),
		ViewAction::Struck => words.arms_bench_struck(),
		ViewAction::Raise => words.arms_bench_raise(),
		ViewAction::Throw => words.arms_bench_throw(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_orb_follows_the_rules_timeline_from_the_release() {
		let bench = bench();
		let mut bench = bench.borrow_mut();
		bench.tick = 100.0;
		bench.throw(Ending::BreaksOn(2));
		let release = 100 + 18;
		let at = |bench: &Bench, ticks: i64| bench.orb_view(release + ticks).unwrap();
		assert_eq!(bench.orb_view(release - 1), None, "it is still in the hand");
		assert_eq!(at(&bench, 0).stage, OrbStage::Flight);
		assert_eq!(at(&bench, 30).stage, OrbStage::Drawing);
		assert_eq!(at(&bench, 50).stage, OrbStage::Falling);
		let first = at(&bench, 70);
		assert_eq!((first.stage, first.pulses), (OrbStage::Pulsing, 1));
		let held = at(&bench, 113);
		assert_eq!((held.stage, held.broke_free_on), (OrbStage::Pulsing, None));
		let broke = at(&bench, 114);
		assert_eq!(broke.stage, OrbStage::BrokeFree, "it breaks on its pulse");
		assert_eq!((broke.pulses, broke.broke_free_on), (1, Some(2)));
		bench.throw(Ending::Holds);
		let release = bench.orb.unwrap().0;
		let orb = |ticks| bench.orb_view(release + ticks).unwrap();
		assert_eq!((orb(200).stage, orb(200).pulses), (OrbStage::Breath, 3));
		assert_eq!(orb(222).stage, OrbStage::Captured);
		bench.throw(Ending::Misses);
		let release = bench.orb.unwrap().0;
		let missed = bench.orb_view(release + 27).unwrap();
		assert_eq!(missed.stage, OrbStage::Missed);
		assert!(!missed.on_target);
	}

	#[test]
	fn a_blow_lands_when_it_strikes() {
		let bench = bench();
		let mut bench = bench.borrow_mut();
		bench.tick = 10.0;
		bench.take(Archetype::Heavy);
		bench.play(ViewAction::Swing);
		let hit = bench.hit.unwrap();
		assert_eq!(
			hit.tick,
			10 + 13,
			"a heavy swing strikes halfway through its stroke"
		);
		assert!(bench.juice.trauma_at(22.0) < 1.0e-6);
		assert!(bench.juice.trauma_at(23.0) > 0.4);
		assert_ne!(hit.color, [1.0, 1.0, 1.0], "a hit never flashes white");
	}
}
