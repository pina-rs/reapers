//! The thumbs' half of a hunt: FIRE under the right thumb, with the weapon
//! rail and what the weapon in hand has left above it; ORB under the left,
//! ringed by the soul it has; and a caption under the crosshair that says
//! what the ring means, what the orb is doing, or, in a first practice, the
//! one thing to learn next. The Flutter `HuntControls`.
//!
//! The phone aims; these only say when. They are read over a camera's
//! picture of any brightness, so every word sits on a dark scrim, and every
//! state is said in words as well as in colour.

use std::f64::consts::FRAC_PI_2;
use std::f64::consts::TAU;

use leptoskit::prelude::*;
use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::ArtSkill;
use reapers_rules::BladeRules;
use reapers_rules::CaptureGrade;
use reapers_rules::CaptureRules;
use reapers_rules::GunRules;
use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;
use reapers_rules::HuskRing;
use reapers_rules::OrbBlock;
use reapers_rules::OrbStage;
use reapers_rules::OrbView;
use reapers_rules::Vec3;
use reapers_rules::WeaponKind;

use super::controller::HuntController;
use super::lessons::Prompt;
use super::overlay::Sight;
use super::overlay::Sketch;
use super::overlay::colour;
use super::pose::LivePose;
use crate::l10n::Messages;
use crate::lookups::art_name;
use crate::lookups::weapon_name;
use crate::render::pointer::Gesture;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::INK;
use crate::theme::Rgb;
use crate::theme::WOUND;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;

/// The smallest a thumb button is drawn, across, in points.
pub const BUTTON: f64 = 92.0;

/// The gold of a husk weak enough for an orb, and of an excellent throw.
pub const ORB_GOLD: Rgb = Rgb::from_hex(0x00E9_C25C);

/// The side of the square a thumb button and its dial are drawn in.
const DIAL: f64 = BUTTON + 24.0;
/// The radius of the dial's ring, just outside the button.
const DIAL_RADIUS: f64 = BUTTON / 2.0 + 7.0;
/// How thick the dial's ring is.
const DIAL_WIDTH: f64 = 5.0;
/// The smallest a touch target is, either way, in points.
const TARGET: f64 = 44.0;
/// How opaque the ground under every word over the room is: enough that
/// the words hold over the brightest room, or the reaper's bone-white arms.
const GROUND_ALPHA: f64 = 0.9;
/// The ink every word over the room sits on, at `GROUND_ALPHA`.
pub const SCRIM: &str = "bg-[#07080be6]";
/// How much of a state's colour a lit button's ground carries.
const TINT: f64 = 0.25;
/// The contrast every word over the room holds at the worst, with white
/// behind its ground: WCAG's for body text.
const LEGIBLE: f64 = 4.5;
/// The amber of a husk drawing breath.
const AMBER: Rgb = Rgb::from_hex(0x00E3_A23B);
/// The blue of a stunned husk.
const STUN_BLUE: Rgb = Rgb::from_hex(0x006F_A9E6);
/// How long the caption keeps saying how an orb ended, in ticks.
const ORB_AFTERWORD_TICKS: i64 = 60;

/// Whether a point, from 0 to 1 across and down the view, falls on ORB.
pub fn orb_area(x: f64, y: f64) -> bool {
	x <= 0.36 && y >= 0.5
}

/// What a finger on a fight's view does: a drag turns the view by
/// `per_unit` radians a pixel, and a press on ORB's side of the screen holds
/// ORB until the finger comes up.
///
/// It is handed what it changes, never the screen: the browser hands a
/// gesture to a screen it has already borrowed, and a gesture that reached
/// for the screen again would borrow it twice and abort the game.
pub fn steer(
	gesture: Gesture,
	look: &mut LivePose,
	per_unit: f64,
	controller: Option<&mut HuntController>,
) {
	match gesture {
		Gesture::Drag { dx, dy } => {
			look.drag(f64::from(dx) * per_unit, -f64::from(dy) * per_unit);
		}
		Gesture::Down { x, y } => {
			if let Some(controller) = controller
				&& orb_area(f64::from(x), f64::from(y))
			{
				controller.hold_orb(true);
			}
		}
		Gesture::Up => {
			if let Some(controller) = controller {
				controller.hold_orb(false);
			}
		}
		Gesture::Tap { .. } => {}
	}
}

/// The name of what is in `slot`.
pub fn slot_name(
	words: &dyn Messages,
	slot: usize,
	weapon: WeaponKind,
	art: Option<ArtSkill>,
) -> String {
	match Arsenal::archetype_of(slot) {
		Some(Archetype::Gun) => words.hunt_slot_pistol().to_owned(),
		Some(Archetype::Blade) => words.hunt_slot_knife().to_owned(),
		Some(Archetype::Heavy) => weapon_name(words, weapon).to_owned(),
		Some(Archetype::Art) => {
			art.map_or_else(
				|| words.hunt_slot_no_art().to_owned(),
				|art| art_name(words, art.art).to_owned(),
			)
		}
		None => String::new(),
	}
}

/// The colour of the ring under the husk: the same signal for every reaper.
const fn ring_colour(ring: HuskRing) -> Rgb {
	match ring {
		HuskRing::Open => BONE,
		HuskRing::DrawingBreath => AMBER,
		HuskRing::Charging => WOUND,
		HuskRing::Stunned => STUN_BLUE,
		HuskRing::Weakened => ORB_GOLD,
	}
}

/// The colour of an orb thrown at `grade`.
pub fn grade_colour(grade: CaptureGrade) -> Rgb {
	match grade {
		CaptureGrade::Plain => ASH,
		CaptureGrade::Great => BONE.lerp(ORB_GOLD, 0.5),
		CaptureGrade::Excellent => ORB_GOLD,
	}
}

/// Whether `orb`'s story is still being told: in the air, holding the husk,
/// or just ended. The rules keep the last orb for the rest of the hunt.
pub fn orb_told(orb: &OrbView) -> bool {
	let ended = match orb.stage {
		OrbStage::BrokeFree => {
			let pulse = orb.broke_free_on.unwrap_or(1).clamp(1, 3);
			CaptureRules::PULSE_TICKS[pulse as usize - 1]
		}
		OrbStage::Missed => CaptureRules::IMPACT_TICK,
		_ => return true,
	};
	orb.ticks < ended + ORB_AFTERWORD_TICKS
}

/// One line under the crosshair.
#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
	pub text: String,
	pub colour: Rgb,
	/// Whether it teaches, rather than reports.
	pub teaching: bool,
}

/// What the caption under the crosshair says: the orb's story while one is
/// being told; the ring's meaning while the husk is fought, unless the
/// player has `lesson` to learn and the husk is not about to strike.
pub fn caption(words: &dyn Messages, sim: &HuntSim, lesson: Option<Prompt>) -> Option<Caption> {
	if let Some(orb) = sim.orb().filter(orb_told) {
		let text = match orb.stage {
			OrbStage::Flight => words.hunt_orb_flight().to_owned(),
			OrbStage::Drawing => words.hunt_orb_drawing().to_owned(),
			OrbStage::Falling => words.hunt_orb_falling().to_owned(),
			OrbStage::Pulsing => words.hunt_orb_pulse(orb.pulses.max(1)),
			OrbStage::Breath => words.hunt_orb_breath().to_owned(),
			OrbStage::Captured => {
				match orb.grade {
					CaptureGrade::Excellent => words.hunt_throw_excellent().to_owned(),
					CaptureGrade::Great => words.hunt_throw_great().to_owned(),
					CaptureGrade::Plain => words.hunt_throw_plain().to_owned(),
				}
			}
			OrbStage::BrokeFree => words.hunt_orb_broke_free().to_owned(),
			OrbStage::Missed => words.hunt_orb_missed().to_owned(),
		};
		let colour = match orb.stage {
			OrbStage::BrokeFree => WOUND,
			OrbStage::Missed => ASH,
			_ => grade_colour(orb.grade),
		};
		return Some(Caption {
			text,
			colour,
			teaching: false,
		});
	}

	if !matches!(
		sim.phase(),
		HuntPhase::Fight
			| HuntPhase::Windup
			| HuntPhase::Shriek
			| HuntPhase::Charge
			| HuntPhase::Broken
	) {
		return None;
	}

	let ring = sim.ring();
	let danger = matches!(ring, HuskRing::DrawingBreath | HuskRing::Charging);
	if let Some(lesson) = lesson.filter(|_| !danger) {
		return Some(Caption {
			text: lesson.text(words).to_owned(),
			colour: BONE,
			teaching: true,
		});
	}
	let text = match ring {
		HuskRing::Open => words.hunt_ring_open(),
		HuskRing::DrawingBreath => words.hunt_ring_drawing_breath(),
		HuskRing::Charging => words.hunt_ring_charging(),
		HuskRing::Stunned => words.hunt_ring_stunned(),
		HuskRing::Weakened => words.hunt_ring_weakened(),
	};

	Some(Caption {
		text: text.to_owned(),
		colour: ring_colour(ring),
		teaching: false,
	})
}

/// The words beside the crosshair, for the two states a shape alone does
/// not explain.
pub fn sight_words(words: &dyn Messages, sight: Sight, light: Rgb) -> Option<(&str, Rgb)> {
	match sight {
		Sight::Seal => Some((words.hunt_sight_seal(), light)),
		Sight::TooFar => Some((words.hunt_sight_too_far(), ASH)),
		Sight::Off | Sight::On => None,
	}
}

/// Why an orb cannot be thrown right now, in words.
fn orb_reason(words: &dyn Messages, block: OrbBlock) -> String {
	match block {
		OrbBlock::NotFighting => words.hunt_orb_not_fighting().to_owned(),
		OrbBlock::NotWeakened => words.hunt_orb_not_weakened().to_owned(),
		OrbBlock::LowSoul => words.hunt_orb_low_soul(CaptureRules::COST.round() as i64),
		OrbBlock::InFlight => words.hunt_orb_in_flight().to_owned(),
	}
}

/// What the weapon in hand has left, and whether it is ready.
fn readout(words: &dyn Messages, sim: &HuntSim, light: Rgb, aim: Vec3) -> (String, Rgb) {
	if sim.draw_ticks_left() > 0 {
		return (words.hunt_drawing().to_owned(), ASH);
	}
	let reach = sim.in_reach(aim);

	match Arsenal::archetype_of(sim.selected_slot()) {
		Some(Archetype::Gun) if sim.reload_jammed() => (words.hunt_jammed().to_owned(), WOUND),
		Some(Archetype::Gun) if sim.in_active_reload() => {
			(words.hunt_reload_now().to_owned(), ORB_GOLD)
		}
		Some(Archetype::Gun) if sim.reloading() => (words.hunt_reloading().to_owned(), BONE),
		Some(Archetype::Gun) if sim.rounds() == 0 => (words.hunt_empty().to_owned(), WOUND),
		Some(Archetype::Gun) if sim.magazine_boosted() => {
			(
				words.hunt_rounds_boosted(sim.rounds(), GunRules::ROUNDS),
				ORB_GOLD,
			)
		}
		Some(Archetype::Gun) => (words.hunt_rounds(sim.rounds(), GunRules::ROUNDS), BONE),
		Some(Archetype::Blade) => {
			(
				words.hunt_dives(sim.dive_charges()),
				if reach { light } else { BONE },
			)
		}
		Some(Archetype::Heavy) if reach => (words.hunt_in_reach().to_owned(), light),
		Some(Archetype::Heavy) => (words.hunt_out_of_reach().to_owned(), ASH),
		Some(Archetype::Art) => (words.hunt_throws(sim.art_charges()), BONE),
		None => (String::new(), ASH),
	}
}

/// A word over the room: small capitals on a dark scrim.
fn tag(text: &str, size: u32, colour: Rgb) -> Element {
	View()
		.class(format!("px-2 py-[3px] rounded-[4px] {SCRIM}"))
		.child(label(text, size, legible(colour, INK)))
}

/// A lit button's ground: `colour` mixed into the ink, so that the words
/// on it hold whatever is behind.
fn tinted(colour: Rgb) -> Rgb {
	INK.lerp(colour, TINT)
}

/// The class for a ground of `colour`, at `GROUND_ALPHA`.
fn ground(colour: Rgb) -> String {
	format!("bg-[{}]", colour.hex_alpha(GROUND_ALPHA))
}

/// `colour`, lifted toward bone just far enough that words in it hold
/// `LEGIBLE` on a ground of `under` with white behind: a state keeps its
/// hue and is still read in a bright room.
pub fn legible(colour: Rgb, under: Rgb) -> Rgb {
	let worst = under.lerp(Rgb(255, 255, 255), 1.0 - GROUND_ALPHA);
	(0..=20)
		.map(|step| colour.lerp(BONE, f64::from(step) / 20.0))
		.find(|lifted| contrast(*lifted, worst) >= LEGIBLE)
		.unwrap_or(BONE)
}

/// WCAG's contrast ratio between two colours.
fn contrast(a: Rgb, b: Rgb) -> f64 {
	fn luminance(colour: Rgb) -> f64 {
		let channel = |value: u8| {
			let value = f64::from(value) / 255.0;
			if value <= 0.040_45 {
				value / 12.92
			} else {
				((value + 0.055) / 1.055).powf(2.4)
			}
		};
		0.2126 * channel(colour.0) + 0.7152 * channel(colour.1) + 0.0722 * channel(colour.2)
	}
	let (a, b) = (luminance(a) + 0.05, luminance(b) + 0.05);
	a.max(b) / a.min(b)
}

/// `count` arcs round a dial with a little air between them, the first
/// `lit` of them in `paint`, and a share of the next one when `part` is
/// more than nothing.
fn segments(sketch: &mut Sketch, count: i64, lit: i64, part: f64, paint: Rgb) {
	let centre = (DIAL / 2.0, DIAL / 2.0);
	let span = TAU / count as f64;
	let air = 0.12;
	for index in 0..count {
		let start = -FRAC_PI_2 + span * index as f64 + air / 2.0;
		let sweep = span - air;
		sketch.arc(
			centre,
			DIAL_RADIUS,
			start,
			sweep,
			colour(BONE, 0.18),
			DIAL_WIDTH,
		);
		let fill = match index.cmp(&lit) {
			std::cmp::Ordering::Less => 1.0,
			std::cmp::Ordering::Equal => part.clamp(0.0, 1.0),
			std::cmp::Ordering::Greater => 0.0,
		};
		if fill > 0.0 {
			sketch.arc(
				centre,
				DIAL_RADIUS,
				start,
				sweep * fill,
				colour(paint, if index < lit { 1.0 } else { 0.5 }),
				DIAL_WIDTH,
			);
		}
	}
}

/// A dial that fills clockwise from twelve o'clock.
fn sweep(sketch: &mut Sketch, fraction: f64, paint: Rgb) {
	let centre = (DIAL / 2.0, DIAL / 2.0);
	sketch.ring(centre, DIAL_RADIUS, colour(BONE, 0.18), DIAL_WIDTH);
	sketch.arc(
		centre,
		DIAL_RADIUS,
		-FRAC_PI_2,
		TAU * fraction.clamp(0.0, 1.0),
		colour(paint, 1.0),
		DIAL_WIDTH,
	);
}

/// The ring round FIRE: the rounds left, a reload and its bright band, the
/// knife's dives, the class weapon's reach, or the art's next throw.
fn fire_dial(sim: &HuntSim, light: Rgb, aim: Vec3) -> Sketch {
	let mut sketch = Sketch::default();
	let centre = (DIAL / 2.0, DIAL / 2.0);
	if sim.draw_ticks_left() > 0 {
		let drawn = 1.0 - sim.draw_ticks_left() as f64 / Arsenal::SWITCH_TICKS as f64;
		sweep(&mut sketch, drawn, ASH);
		return sketch;
	}
	match Arsenal::archetype_of(sim.selected_slot()) {
		Some(Archetype::Gun) if sim.reloading() => {
			// The whole reload once round, and the band where FIRE finishes it.
			let share = |tick: i64| tick as f64 / GunRules::RELOAD_TICKS as f64;
			let band_from = share(GunRules::ACTIVE_FROM_TICK);
			let band_to = share(GunRules::ACTIVE_UNTIL_TICK + 1);
			let jammed = sim.reload_jammed();
			sketch.ring(centre, DIAL_RADIUS, colour(BONE, 0.18), DIAL_WIDTH);
			sketch.arc(
				centre,
				DIAL_RADIUS,
				-FRAC_PI_2 + TAU * band_from,
				TAU * (band_to - band_from),
				colour(ORB_GOLD, if jammed { 0.3 } else { 1.0 }),
				DIAL_WIDTH + 6.0,
			);
			let progress = sim.reload_progress().clamp(0.0, 1.0);
			let paint = if jammed { WOUND } else { BONE };
			sketch.arc(
				centre,
				DIAL_RADIUS,
				-FRAC_PI_2,
				TAU * progress,
				colour(paint, 0.9),
				DIAL_WIDTH - 2.0,
			);
			let angle = -FRAC_PI_2 + TAU * progress;
			let head = (
				centre.0 + DIAL_RADIUS * angle.cos(),
				centre.1 + DIAL_RADIUS * angle.sin(),
			);
			sketch.dot(head, 6.0, colour(Rgb(0x02, 0x02, 0x03), 0.8));
			sketch.dot(head, 4.5, colour(paint, 1.0));
		}
		Some(Archetype::Gun) => {
			let paint = if sim.magazine_boosted() {
				ORB_GOLD
			} else {
				BONE
			};
			segments(&mut sketch, GunRules::ROUNDS, sim.rounds(), 0.0, paint);
		}
		Some(Archetype::Blade) => {
			segments(
				&mut sketch,
				BladeRules::DIVE_CHARGES,
				sim.dive_charges(),
				sim.dive_recharge(),
				light,
			);
		}
		Some(Archetype::Heavy) => {
			let reach = sim.in_reach(aim);
			sweep(
				&mut sketch,
				if sim.diving() {
					1.0 - sim.dive_progress()
				} else {
					1.0
				},
				if reach { light } else { ASH },
			);
		}
		Some(Archetype::Art) => {
			let charges = sim.stats().art.map_or(0, |art| art.max_charges());
			if charges > 0 {
				segments(
					&mut sketch,
					charges,
					sim.art_charges(),
					1.0 - sim.art_recharge(),
					BONE,
				);
			}
		}
		None => {}
	}
	sketch
}

/// The ring round ORB: the soul gathered, with a notch at what an orb
/// costs, all gold once one can be thrown.
fn orb_dial(sim: &HuntSim, ready: bool) -> Sketch {
	let mut sketch = Sketch::default();
	let centre = (DIAL / 2.0, DIAL / 2.0);
	let soul = sim.soul() / CaptureRules::SOUL_MAX;
	sweep(&mut sketch, soul, if ready { ORB_GOLD } else { ASH });
	let cost = -FRAC_PI_2 + TAU * CaptureRules::COST / CaptureRules::SOUL_MAX;
	let (dx, dy) = (cost.cos(), cost.sin());
	sketch.line(
		(
			centre.0 + dx * (DIAL_RADIUS - 6.0),
			centre.1 + dy * (DIAL_RADIUS - 6.0),
		),
		(
			centre.0 + dx * (DIAL_RADIUS + 6.0),
			centre.1 + dy * (DIAL_RADIUS + 6.0),
		),
		colour(BONE, 0.9),
		2.0,
	);
	sketch
}

/// A thumb button inside its dial.
fn dialled(dial: Sketch, button: Element) -> Element {
	View()
		.class(format!(
			"relative w-[{DIAL}px] h-[{DIAL}px] items-center justify-center"
		))
		.child(dial.into_view(DIAL, DIAL))
		.child(button)
}

/// Everything the thumbs' controls are drawn from.
pub struct Thumbs<'a> {
	pub sim: &'a HuntSim,
	/// The slot in hand.
	pub slot: usize,
	/// Whether the reaper carries each slot into this fight.
	pub carried: [bool; Arsenal::SLOTS],
	/// The class weapon.
	pub weapon: WeaponKind,
	/// The husk's light.
	pub light: Rgb,
	/// Where the crosshair points.
	pub aim: Vec3,
	/// What the caption teaches, in a first practice.
	pub lesson: Option<Prompt>,
	/// False for a hunt that plays itself: the controls show and cannot be
	/// pressed.
	pub touchable: bool,
	pub on_fire: fn(),
	pub on_slot: fn(usize),
}

/// The controls over a fight, for a thumb.
pub fn controls(words: &dyn Messages, thumbs: &Thumbs) -> Element {
	let mut view = View().class("absolute inset-0");
	let sight = Sight::of(thumbs.sim, thumbs.aim);
	if thumbs.sim.phase() != HuntPhase::Stalk
		&& let Some((text, paint)) = sight_words(words, sight, thumbs.light)
	{
		// Beside the crosshair, on the right of the middle of the view.
		view = view.child(
			View()
				.class("absolute inset-0 flex-row items-center")
				.aria_hidden(true)
				.child(View().class("flex-1"))
				.child(
					View()
						.class("flex-1 items-start pl-[30px]")
						.test_id("hunt-sight")
						.child(tag(text, 11, paint)),
				),
		);
	}
	if let Some(line) = caption(words, thumbs.sim, thumbs.lesson) {
		// The caption sits under the crosshair, where the eye already is.
		let words_of_it = if line.teaching {
			View()
				.class(format!(
					"max-w-[300px] px-4 py-2 rounded-[6px] border border-[{}] {SCRIM} items-center",
					BONE.hex_alpha(0.6)
				))
				.child(
					body(line.text, 15, legible(line.colour, INK))
						.class("text-center font-semibold"),
				)
		} else {
			View()
				.class(format!(
					"flex-row items-center gap-2 px-3 py-[5px] rounded-full {SCRIM}"
				))
				.child(View().class(format!("w-2 h-2 rounded-full bg-[{}]", line.colour.hex())))
				.child(label(&line.text, 12, legible(line.colour, INK)))
		};
		view = view.child(
			View()
				.class("absolute left-6 right-6 top-[55%] items-center")
				.test_id(if line.teaching {
					"hunt-lesson"
				} else {
					"hunt-caption"
				})
				.aria_live(leptoskit::native::Live::Polite)
				.child(words_of_it),
		);
	}
	view.child(orb(words, thumbs.sim))
		.child(fire(words, thumbs))
}

fn orb(words: &dyn Messages, sim: &HuntSim) -> Element {
	let block = sim.orb_block();
	let ready = block.is_none();
	let soul = sim.soul().floor() as i64;
	let paint = if ready { ORB_GOLD } else { ASH };
	let state = block.map_or_else(
		|| words.hunt_orb_ready().to_owned(),
		|block| orb_reason(words, block),
	);
	let under = if ready { tinted(ORB_GOLD) } else { INK };
	let mut button = View()
		.class(format!(
			"w-[{BUTTON}px] h-[{BUTTON}px] rounded-full border-2 items-center justify-center {} border-[{}]",
			ground(under),
			paint.hex()
		))
		.test_id("hunt-orb")
		.accessibility_label(words.hunt_orb_semantics(soul))
		.child(display(
			words.hunt_orb(),
			20,
			legible(if ready { ORB_GOLD } else { BONE }, under),
		))
		.child(figure(soul.to_string(), 12, legible(paint, under)));
	if let Some(block) = block {
		button = button.aria_description(orb_reason(words, block));
	}
	// ORB is heard by the view's gestures: the whole lower left is its hold.
	View()
		.class("absolute left-3 bottom-4 pb-safe items-center")
		.child(
			View()
				.test_id("hunt-orb-state")
				.child(tag(&state, 11, paint)),
		)
		.child(gap(4))
		.child(dialled(orb_dial(sim, ready), button))
}

fn fire(words: &dyn Messages, thumbs: &Thumbs) -> Element {
	let sim = thumbs.sim;
	let art = sim.stats().art;
	// Two to a row, so the rail stays below the caption and above ORB.
	let mut rail = View().class("flex-row flex-wrap justify-end gap-1 max-w-[300px]");
	for index in (0..Arsenal::SLOTS).filter(|&index| thumbs.carried[index]) {
		rail = rail.child(chip(words, thumbs, index, art));
	}
	let (text, paint) = readout(words, sim, thumbs.light, thumbs.aim);
	let window = sim.in_active_reload();
	let ready = sim.cooldown_ticks() == 0 && !sim.reloading() && sim.draw_ticks_left() == 0;
	let edge = if window {
		ORB_GOLD
	} else if ready {
		thumbs.light
	} else {
		ASH
	};
	let under = if window {
		tinted(ORB_GOLD)
	} else if ready {
		tinted(thumbs.light)
	} else {
		INK
	};
	let mut button = Pressable(words.hunt_fire())
		.class(format!(
			"w-[{BUTTON}px] h-[{BUTTON}px] rounded-full border-2 items-center justify-center {} border-[{}]",
			ground(under),
			edge.hex()
		))
		.test_id("hunt-fire")
		.accessibility_label(words.hunt_fire_semantics(&slot_name(
			words,
			thumbs.slot,
			thumbs.weapon,
			art,
		)))
		.aria_description(text.clone())
		.child(display(
			words.hunt_fire(),
			20,
			legible(if window { ORB_GOLD } else { BONE }, under),
		));
	if thumbs.touchable {
		button = button.on_press(thumbs.on_fire);
	}
	View()
		.class("absolute right-3 bottom-4 pb-safe items-end")
		.child(rail)
		.child(gap(8))
		.child(
			View()
				.class("items-center")
				.child(View().test_id("hunt-readout").child(tag(&text, 12, paint)))
				.child(gap(4))
				.child(dialled(fire_dial(sim, thumbs.light, thumbs.aim), button)),
		)
		.into_view()
}

/// One slot on the rail: its name, and whether it is in hand, in words and
/// in light.
fn chip(words: &dyn Messages, thumbs: &Thumbs, index: usize, art: Option<ArtSkill>) -> Element {
	let name = slot_name(words, index, thumbs.weapon, art);
	let selected = index == thumbs.slot;
	let enabled = Arsenal::archetype_of(index) != Some(Archetype::Art) || art.is_some();
	let (text, under, edge) = if !enabled {
		(ASH, INK, "border-transparent".to_owned())
	} else if selected {
		(
			BONE,
			tinted(thumbs.light),
			format!("border-[{}]", thumbs.light.hex()),
		)
	} else {
		(BONE, INK, "border-transparent".to_owned())
	};
	let back = ground(under);
	let mut chip = Pressable(words.hunt_slot_semantics(&name, index as i64 + 1))
		.class(format!(
			"flex-row items-center gap-2 min-h-[{TARGET}px] min-w-[{TARGET}px] px-3 rounded-full border {back} {edge}"
		))
		.test_id(format!("hunt-slot-{index}"))
		.accessibility_label(words.hunt_slot_semantics(&name, index as i64 + 1))
		.aria_selected(selected)
		.enabled(enabled && thumbs.touchable)
		.child(figure((index + 1).to_string(), 11, legible(ASH, under)))
		.child(label(
			&name,
			if selected { 13 } else { 12 },
			legible(text, under),
		));
	if enabled && thumbs.touchable {
		let on_slot = thumbs.on_slot;
		chip = chip.on_press(move || on_slot(index));
	}
	chip
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntPlan;
	use reapers_rules::HuntStats;
	use reapers_rules::market_listings;

	use super::*;
	use crate::l10n::En;

	fn sim() -> HuntSim {
		let nft = market_listings()[0];
		HuntSim::new(
			7,
			nft.collection.vessel(),
			nft.rarity(),
			nft.rarity().ward(),
			&HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap(),
			HuntPlan::OPEN,
			Difficulty::default(),
		)
		.unwrap()
	}

	fn found(sim: &mut HuntSim) {
		while sim.phase() == HuntPhase::Stalk {
			let input = reapers_rules::HuntInput::still(sim.pose().direction(), 0.0);
			sim.step(&input);
		}
	}

	#[test]
	fn fire_and_the_weapon_rail_are_controls_the_phone_can_press() {
		use leptoskit_testing::Finder;
		use leptoskit_testing::WidgetTester;
		let tester = WidgetTester::new(
			|| {
				let sim = sim();
				View().class("relative w-full h-screen").child(controls(
					&En,
					&Thumbs {
						sim: &sim,
						slot: 0,
						carried: [true; 4],
						weapon: WeaponKind::Scythe,
						light: BONE,
						aim: -sim.pose().direction(),
						lesson: None,
						touchable: true,
						on_fire: || {},
						on_slot: |_| {},
					},
				))
			},
			390.0,
			844.0,
		)
		.unwrap();
		for id in ["hunt-fire", "hunt-slot-0", "hunt-slot-1", "hunt-slot-2"] {
			assert_eq!(
				tester.find(&Finder::test_id(id)).unwrap().kind,
				leptoskit::native::NativeKind::Pressable,
				"{id} must generate a native press even when the crosshair misses"
			);
		}
	}

	#[test]
	fn every_word_over_the_room_holds_over_the_brightest_room() {
		let white = Rgb(255, 255, 255);
		// Every colour the HUD paints words in, and the extremes a reaper's
		// light could reach.
		let colours = [
			BONE,
			ASH,
			WOUND,
			ORB_GOLD,
			AMBER,
			STUN_BLUE,
			white,
			Rgb(0, 0, 0),
			Rgb(0, 0, 255),
			Rgb(255, 0, 0),
			Rgb(0x7A, 0x3C, 0xFF),
			Rgb(0xC8, 0xFF, 0x6A),
		];
		let grounds = std::iter::once(INK).chain(colours.iter().map(|&colour| tinted(colour)));
		for under in grounds {
			let worst = under.lerp(white, 1.0 - GROUND_ALPHA);
			for colour in colours {
				let words = legible(colour, under);
				let ratio = contrast(words, worst);
				assert!(ratio >= LEGIBLE, "{colour:?} on {under:?}: {ratio:.2}");
			}
		}
		assert_eq!(SCRIM, ground(INK), "the scrim is the ink's ground");
		assert_eq!(legible(BONE, INK), BONE, "bone needs no lifting");
		assert_eq!(legible(ORB_GOLD, INK), ORB_GOLD, "nor does gold on ink");
		assert!((contrast(BONE, INK) - contrast(INK, BONE)).abs() < 1e-12);
	}

	#[test]
	fn a_finger_turns_the_view_and_holds_orb_only_on_its_side() {
		let mut controller = HuntController::new(super::super::controller::Fight::Hunt(sim()));
		let mut look = LivePose::default();
		let ahead = look.pose().forward;
		steer(
			Gesture::Down { x: 0.8, y: 0.9 },
			&mut look,
			0.01,
			Some(&mut controller),
		);
		assert!(!controller.orb_held(), "FIRE's side is not ORB's");
		steer(
			Gesture::Down { x: 0.1, y: 0.9 },
			&mut look,
			0.01,
			Some(&mut controller),
		);
		assert!(controller.orb_held());
		steer(
			Gesture::Drag { dx: 40.0, dy: 0.0 },
			&mut look,
			0.01,
			Some(&mut controller),
		);
		let turned = look.pose().forward.angle_to(ahead);
		assert!((turned - 0.4).abs() < 1e-9, "{turned}");
		steer(Gesture::Up, &mut look, 0.01, Some(&mut controller));
		assert!(!controller.orb_held());
		// With no fight yet there is nothing to hold, and nothing breaks.
		steer(Gesture::Down { x: 0.1, y: 0.9 }, &mut look, 0.01, None);
		steer(Gesture::Up, &mut look, 0.01, None);
	}

	#[test]
	fn the_rail_names_the_four_slots_and_falls_back_to_no_art() {
		let words = En;
		let names: Vec<String> = (0..Arsenal::SLOTS)
			.map(|slot| slot_name(&words, slot, WeaponKind::Spear, None))
			.collect();
		assert_eq!(names[0], words.hunt_slot_pistol());
		assert_eq!(names[1], words.hunt_slot_knife());
		assert_eq!(names[2], words.weapon_spear());
		assert_eq!(names[3], words.hunt_slot_no_art());
		assert!(orb_area(0.1, 0.9));
		assert!(!orb_area(0.9, 0.9));
	}

	#[test]
	fn the_caption_says_what_the_ring_means() {
		let words = En;
		let mut sim = sim();
		// A husk that has not been found says nothing.
		assert!(caption(&words, &sim, None).is_none());
		found(&mut sim);
		assert_eq!(
			caption(&words, &sim, None).map(|line| line.text),
			Some(words.hunt_ring_open().to_owned())
		);
	}

	#[test]
	fn a_lesson_takes_the_caption_unless_the_husk_is_about_to_strike() {
		let words = En;
		let mut sim = sim();
		found(&mut sim);
		let line = caption(&words, &sim, Some(Prompt::Fire)).unwrap();
		assert!(line.teaching);
		assert_eq!(line.text, words.hunt_teach_fire());
		while !matches!(sim.ring(), HuskRing::DrawingBreath | HuskRing::Charging) {
			let input = reapers_rules::HuntInput::still(sim.pose().direction(), 0.0);
			sim.step(&input);
			assert!(sim.tick() < 60 * 120, "the husk draws breath in time");
		}
		let line = caption(&words, &sim, Some(Prompt::Fire)).unwrap();
		assert!(!line.teaching, "a warning is never hidden by a lesson");
	}

	#[test]
	fn an_ended_orb_is_told_for_a_moment_then_the_ring_again() {
		let mut orb = OrbView {
			thrown_at: 0,
			ticks: CaptureRules::IMPACT_TICK + 10,
			stage: OrbStage::Missed,
			grade: CaptureGrade::Plain,
			on_target: false,
			pulses: 0,
			target: Vec3::FORWARD,
			chance: None,
			broke_free_on: None,
		};
		assert!(orb_told(&orb));
		orb.ticks = CaptureRules::IMPACT_TICK + ORB_AFTERWORD_TICKS;
		assert!(!orb_told(&orb));
		orb.stage = OrbStage::BrokeFree;
		orb.broke_free_on = Some(2);
		orb.ticks = CaptureRules::PULSE_TICKS[1] + 1;
		assert!(orb_told(&orb));
		orb.stage = OrbStage::Pulsing;
		orb.ticks = 10_000;
		assert!(orb_told(&orb), "a story under way is always told");
	}

	#[test]
	fn every_state_of_the_sight_that_a_shape_cannot_explain_has_words() {
		let words = En;
		assert!(sight_words(&words, Sight::Seal, BONE).is_some());
		assert!(sight_words(&words, Sight::TooFar, BONE).is_some());
		assert!(sight_words(&words, Sight::On, BONE).is_none());
		assert!(sight_words(&words, Sight::Off, BONE).is_none());
	}
}
