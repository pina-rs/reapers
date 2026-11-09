//! The thumbs' half of a hunt: FIRE under the right thumb, with the weapon
//! rail and what the weapon in hand has left above it; ORB under the left;
//! and a caption under the crosshair that says what the ring means or what
//! the orb is doing. The Flutter `HuntControls`.
//!
//! The phone aims; these only say when.

use leptoskit::prelude::*;
use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::ArtSkill;
use reapers_rules::CaptureGrade;
use reapers_rules::CaptureRules;
use reapers_rules::GunRules;
use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;
use reapers_rules::HuskRing;
use reapers_rules::OrbBlock;
use reapers_rules::OrbStage;
use reapers_rules::WeaponKind;

use crate::l10n::Messages;
use crate::lookups::art_name;
use crate::lookups::weapon_name;
use crate::theme::ASH;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::WOUND;
use crate::ui::display;
use crate::ui::gap;
use crate::ui::label;

/// The smallest a thumb button is drawn, across, in points.
pub const BUTTON: f64 = 92.0;

/// The gold of a husk weak enough for an orb, and of an excellent throw.
pub const ORB_GOLD: Rgb = Rgb::from_hex(0x00E9_C25C);

/// Whether a point, from 0 to 1 across and down the view, falls on ORB.
pub fn orb_area(x: f64, y: f64) -> bool {
	x <= 0.36 && y >= 0.5
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
fn ring_colour(ring: HuskRing) -> Rgb {
	match ring {
		HuskRing::Open => BONE,
		HuskRing::DrawingBreath => Rgb::from_hex(0x00E3_A23B),
		HuskRing::Charging => WOUND,
		HuskRing::Stunned => Rgb::from_hex(0x006F_A9E6),
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

/// What the caption under the crosshair says: the orb's stages while one is
/// in the air, and the ring's meaning while the husk is fought.
pub fn caption(words: &dyn Messages, sim: &HuntSim) -> Option<(String, Rgb)> {
	if let Some(orb) = sim.orb() {
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

		return Some((text, colour));
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
	let text = match ring {
		HuskRing::Open => words.hunt_ring_open(),
		HuskRing::DrawingBreath => words.hunt_ring_drawing_breath(),
		HuskRing::Charging => words.hunt_ring_charging(),
		HuskRing::Stunned => words.hunt_ring_stunned(),
		HuskRing::Weakened => words.hunt_ring_weakened(),
	};

	Some((text.to_owned(), ring_colour(ring)))
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
fn readout(words: &dyn Messages, sim: &HuntSim, light: Rgb) -> (String, Rgb) {
	let archetype = Arsenal::archetype_of(sim.selected_slot());

	if sim.draw_ticks_left() > 0 {
		return (words.hunt_drawing().to_owned(), ASH);
	}

	match archetype {
		Some(Archetype::Gun) if sim.reload_jammed() => (words.hunt_jammed().to_owned(), WOUND),
		Some(Archetype::Gun) if sim.in_active_reload() => {
			(words.hunt_reload_now().to_owned(), ORB_GOLD)
		}
		Some(Archetype::Gun) if sim.reloading() => (words.hunt_reloading().to_owned(), ASH),
		Some(Archetype::Gun) => {
			(
				words.hunt_rounds(sim.rounds(), GunRules::ROUNDS),
				if sim.magazine_boosted() {
					ORB_GOLD
				} else {
					BONE
				},
			)
		}
		Some(Archetype::Blade) => {
			(
				words.hunt_dives(sim.dive_charges()),
				if sim.in_reach(sim.pose().direction()) {
					light
				} else {
					ASH
				},
			)
		}
		Some(Archetype::Heavy) => {
			(
				if sim.in_reach(sim.pose().direction()) {
					words.hunt_in_reach().to_owned()
				} else {
					words.hunt_out_of_reach().to_owned()
				},
				if sim.in_reach(sim.pose().direction()) {
					light
				} else {
					ASH
				},
			)
		}
		Some(Archetype::Art) => (words.hunt_throws(sim.art_charges()), BONE),
		None => (String::new(), ASH),
	}
}

/// How far through its readying the weapon in hand is, from 0 to 1.
fn ready_fraction(sim: &HuntSim) -> f64 {
	if sim.draw_ticks_left() > 0 {
		return 1.0 - sim.draw_ticks_left() as f64 / Arsenal::SWITCH_TICKS as f64;
	}

	match Arsenal::archetype_of(sim.selected_slot()) {
		Some(Archetype::Gun) if sim.reloading() => sim.reload_progress(),
		Some(Archetype::Gun) => sim.rounds() as f64 / GunRules::ROUNDS as f64,
		Some(Archetype::Blade | Archetype::Heavy) if sim.diving() => 1.0 - sim.dive_progress(),
		Some(Archetype::Blade) => {
			(sim.dive_charges() as f64 + sim.dive_recharge())
				/ reapers_rules::BladeRules::DIVE_CHARGES as f64
		}
		Some(Archetype::Art) => 1.0 - sim.art_recharge(),
		_ => 1.0,
	}
	.clamp(0.0, 1.0)
}

fn meter(fraction: f64, colour: Rgb, width: &str) -> Element {
	View()
		.class(format!("{width} h-[3px] bg-[#e9e4d42e]"))
		.child(View().class(format!(
			"h-full w-[{:.0}%] bg-[{}]",
			fraction.clamp(0.0, 1.0) * 100.0,
			colour.hex()
		)))
}

/// The controls over a fight, for a thumb. When `touchable` is false (a hunt
/// that plays itself) they show and cannot be pressed.
#[allow(
	clippy::too_many_arguments,
	reason = "one thumb's half of the screen, and what the other sees"
)]
pub fn controls(
	words: &dyn Messages,
	sim: &HuntSim,
	slot: usize,
	weapon: WeaponKind,
	light: Rgb,
	touchable: bool,
	on_fire: fn(),
	on_slot: fn(usize),
) -> Element {
	let mut view = View().class("absolute inset-0");
	if let Some((text, colour)) = caption(words, sim) {
		// The caption sits under the crosshair, where the eye already is.
		view = view.child(
			View()
				.class("absolute left-0 right-0 top-[54%] items-center")
				.test_id("hunt-caption")
				.child(label(&text.to_uppercase(), 12, colour)),
		);
	}
	view.child(orb(words, sim, touchable)).child(fire(
		words, sim, slot, weapon, light, touchable, on_fire, on_slot,
	))
}

fn orb(words: &dyn Messages, sim: &HuntSim, touchable: bool) -> Element {
	let block = sim.orb_block();
	let soul = sim.soul().floor() as i64;
	let colour = if block.is_none() { ORB_GOLD } else { ASH };
	let name = if block.is_none() {
		words.hunt_soul(soul)
	} else {
		block.map_or_else(String::new, |block| orb_reason(words, block))
	};
	let mut button = View()
		.class(format!(
			"w-[{BUTTON}px] h-[{BUTTON}px] rounded-full border items-center justify-center bg-[{}] border-[{}]",
			colour.hex_alpha(0.12),
			colour.hex()
		))
		.test_id("hunt-orb")
		.accessibility_label(words.hunt_orb_semantics(soul))
		.child(display(words.hunt_orb(), 20, colour));
	if let Some(block) = block {
		button = button.aria_description(orb_reason(words, block));
	}
	let _ = touchable;
	View()
		.class("absolute left-4 bottom-5 pb-safe items-start")
		.child(label(&name.to_uppercase(), 11, colour))
		.child(gap(8))
		.child(button)
		.child(gap(6))
		.child(meter(
			sim.soul() / CaptureRules::SOUL_MAX,
			colour,
			&format!("w-[{BUTTON}px]"),
		))
}

#[allow(clippy::too_many_arguments, reason = "one thumb's half of the screen")]
fn fire(
	words: &dyn Messages,
	sim: &HuntSim,
	slot: usize,
	weapon: WeaponKind,
	light: Rgb,
	touchable: bool,
	on_fire: fn(),
	on_slot: fn(usize),
) -> Element {
	let art = sim.stats().art;
	let mut rail = View().class("items-end");
	for index in 0..Arsenal::SLOTS {
		let name = slot_name(words, index, weapon, art);
		let selected = index == slot;
		let enabled = Arsenal::archetype_of(index) != Some(Archetype::Art) || art.is_some();
		let colour = if !enabled {
			ASH
		} else if selected {
			light
		} else {
			ASH
		};
		let mut chip = View()
			.class("flex-row items-center gap-2 min-h-[44px]")
			.test_id(format!("hunt-slot-{index}"))
			.accessibility_label(words.hunt_slot_semantics(&name, index as i64 + 1))
			.child(label(
				&name.to_uppercase(),
				if selected { 12 } else { 11 },
				colour,
			))
			.child(
				View()
					.class("w-[3px]")
					.class(if selected { "h-[22px]" } else { "h-[12px]" })
					.class(format!("bg-[{}]", colour.hex())),
			);
		if enabled && touchable {
			chip = chip.on_press(move || on_slot(index));
		}
		rail = rail.child(chip);
	}
	let (text, colour) = readout(words, sim, light);
	let ready = sim.cooldown_ticks() == 0 && !sim.reloading();
	let mut button = View()
		.class(format!(
			"w-[{BUTTON}px] h-[{BUTTON}px] rounded-full border items-center justify-center bg-[{}] border-[{}]",
			light.hex_alpha(if ready { 0.18 } else { 0.08 }),
			if ready { light.hex() } else { ASH.hex() }
		))
		.test_id("hunt-fire")
		.accessibility_label(
			words.hunt_fire_semantics(&slot_name(words, slot, weapon, art)),
		)
		.child(display(
			words.hunt_fire(),
			20,
			if sim.in_active_reload() { ORB_GOLD } else { BONE },
		));
	if touchable {
		button = button.on_press(on_fire);
	}
	View()
		.class("absolute right-4 bottom-5 pb-safe items-end")
		.child(rail)
		.child(gap(8))
		.child(label(&text.to_uppercase(), 12, colour))
		.child(gap(8))
		.child(button)
		.child(gap(6))
		.child(meter(
			ready_fraction(sim),
			light,
			&format!("w-[{BUTTON}px]"),
		))
		.into_view()
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
		assert!(caption(&words, &sim).is_none());
		while sim.phase() == HuntPhase::Stalk {
			let input = reapers_rules::HuntInput::still(sim.pose().direction(), 0.0);
			sim.step(&input);
		}
		assert_eq!(
			caption(&words, &sim).map(|(text, _)| text),
			Some(words.hunt_ring_open().to_owned())
		);
	}
}
