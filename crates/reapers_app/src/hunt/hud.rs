//! The strip of the ledger that is always in view during a fight: what is
//! being fought, what is left of its ward and of the reaper's resolve, and
//! one line of instruction. The Flutter `HuntStatus` and its banners.
//!
//! What it shows is read off the rules into a [`Status`], a plain value, so
//! that the screen is drawn again only when something on it changed.

use leptoskit::prelude::*;
use reapers_rules::Art;
use reapers_rules::Footing;
use reapers_rules::HuntEvent;
use reapers_rules::HuntPhase;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::OrbBlock;

use crate::l10n::Messages;
use crate::lookups::gift_name;
use crate::platform;
use crate::theme::BONE;
use crate::theme::Rgb;
use crate::theme::WOUND;
use crate::ui::body;
use crate::ui::display;
use crate::ui::figure;
use crate::ui::gap;
use crate::ui::label;
use crate::ui::ledger_meter;
use crate::ui::ledger_pip;

/// Fresh feedback with its own lifetime. Routine hits are quieter than
/// changes of phase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banner {
	pub text: String,
	pub strike: bool,
}

/// Feedback for an event the rules have already decided, or `None` when
/// the event needs no words.
pub fn banner_for(words: &dyn Messages, event: &HuntEvent) -> Option<Banner> {
	let text = match event {
		HuntEvent::GiftStirred(gift) => gift_name(words, *gift).to_owned(),
		HuntEvent::StalkEnded { ambush } => {
			if *ambush {
				words.hunt_ambush()
			} else {
				words.hunt_noticed()
			}
			.to_owned()
		}
		HuntEvent::Flinched { forgiven, .. } => {
			if *forgiven {
				words.hunt_steadfast()
			} else {
				words.hunt_flinched()
			}
			.to_owned()
		}
		HuntEvent::HeldFirm => words.hunt_held_firm().to_owned(),
		HuntEvent::Parried => words.hunt_parried().to_owned(),
		HuntEvent::Struck { .. } => words.hunt_struck().to_owned(),
		HuntEvent::Broken { .. } => words.hunt_broken().to_owned(),
		HuntEvent::Recovered { .. } => words.hunt_recovered().to_owned(),
		HuntEvent::ArtLanded { hit: false, .. } => words.hunt_art_missed().to_owned(),
		HuntEvent::ArtLanded {
			art: Art::Cinder, ..
		} => words.hunt_burning().to_owned(),
		HuntEvent::ArtLanded { art: Art::Rime, .. } => words.hunt_frozen().to_owned(),
		HuntEvent::ArtLanded { art: Art::Gale, .. } => words.hunt_gusted().to_owned(),
		HuntEvent::IceShattered => words.hunt_shattered().to_owned(),
		HuntEvent::HitLanded { damage, seal, .. } => {
			let damage = damage.round() as i64;

			if seal.is_some() {
				words.hunt_hit_seal(damage)
			} else {
				words.hunt_hit(damage)
			}
		}
		HuntEvent::Missed(_) => words.hunt_missed().to_owned(),
		HuntEvent::OrbBrokeFree(_) => words.hunt_orb_broke_free().to_owned(),
		HuntEvent::OrbMissed => words.hunt_orb_missed().to_owned(),
		HuntEvent::HuskCaptured(_) => words.hunt_captured().to_owned(),
		HuntEvent::OrbThrown { .. }
		| HuntEvent::OrbImpact { .. }
		| HuntEvent::OrbAbsorbed
		| HuntEvent::OrbLanded
		| HuntEvent::OrbPulse(_)
		| HuntEvent::ArtThrown(_)
		| HuntEvent::WeaponDrawn(_)
		| HuntEvent::ShotFired { .. }
		| HuntEvent::ReloadStarted
		| HuntEvent::ReloadJammed
		| HuntEvent::ReloadFinished { .. }
		| HuntEvent::DiveStarted(_)
		| HuntEvent::DiveEnded
		| HuntEvent::Leapt(_)
		| HuntEvent::SealOpened(_)
		| HuntEvent::ShriekWindup
		| HuntEvent::ShriekBegan
		| HuntEvent::ChargeBegan
		| HuntEvent::Relocating
		| HuntEvent::HuskEscaped(_) => return None,
	};
	Some(Banner {
		text,
		strike: matches!(event, HuntEvent::HitLanded { .. } | HuntEvent::Missed(_)),
	})
}

/// How hard `event` knocks in the player's hand: the phone is the weapon,
/// so what the weapon meets should be felt.
pub fn knock_for(event: &HuntEvent) {
	match event {
		HuntEvent::HitLanded {
			empowered: true, ..
		}
		| HuntEvent::ShriekBegan
		| HuntEvent::Parried
		| HuntEvent::Broken { .. }
		| HuntEvent::HuskCaptured(_)
		| HuntEvent::HuskEscaped(_)
		| HuntEvent::ArtLanded { hit: true, .. }
		| HuntEvent::IceShattered
		| HuntEvent::Flinched { .. }
		| HuntEvent::Struck { .. } => platform::thud(),
		HuntEvent::StalkEnded { .. }
		| HuntEvent::HeldFirm
		| HuntEvent::ChargeBegan
		| HuntEvent::ArtThrown(_)
		| HuntEvent::HitLanded { .. }
		| HuntEvent::Missed(_)
		| HuntEvent::OrbBrokeFree(_) => platform::tap(),
		_ => {}
	}
}

/// The meter under the banner, when the phase has one.
#[derive(Clone, Debug, PartialEq)]
pub enum Gauge {
	/// How steady the phone is while the husk shrieks.
	Steadiness { fraction: f64, steady: bool },
	/// What is left of the window to land an orb.
	Broken { fraction: f64 },
}

/// Everything the fight's ledger shows, read off the rules.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
	pub title: String,
	/// Seconds left of the fight, once it has begun.
	pub seconds: Option<i64>,
	pub ward: f64,
	pub resolve: i64,
	pub resolve_max: i64,
	pub cadence: i64,
	/// How much of the cadence's window is left, from 1 to 0.
	pub cadence_left: f64,
	pub banner: Option<Banner>,
	pub gauge: Option<Gauge>,
	pub prompt: String,
	/// Whether the prompt is a shout: hold still, or cut.
	pub loud: bool,
}

/// Rounds a fraction to a hundredth, so that the ledger is drawn again
/// only when a meter moves by one.
fn step(fraction: f64) -> f64 {
	(fraction.clamp(0.0, 1.0) * 100.0).round() / 100.0
}

/// What the ledger shows for `sim`, with the phone turning at `turn_speed`.
/// `follows_device` says whether the view follows the phone or a drag, and
/// `afterword` is what to say once this reaper's own part is over.
pub fn status(
	words: &dyn Messages,
	title: &str,
	sim: &HuntSim,
	turn_speed: f64,
	follows_device: bool,
	banner: Option<Banner>,
	afterword: &str,
) -> Status {
	let phase = sim.phase();
	let stats = sim.stats();
	let any_seal_open = (0..sim.vessel().seals().len()).any(|seal| sim.seal_open(seal));
	let prompt = match phase {
		HuntPhase::Stalk if turn_speed > HuntRules::STARTLE_SPEED => words.hunt_prompt_too_fast(),
		HuntPhase::Stalk if sim.steadiness() > 0.05 => words.hunt_prompt_steady(),
		HuntPhase::Stalk if follows_device => words.hunt_prompt_stalk(),
		HuntPhase::Stalk => words.hunt_prompt_stalk_drag(),
		HuntPhase::Fight if sim.footing() == Footing::Hidden && !sim.leaping() => {
			words.hunt_prompt_hidden()
		}
		HuntPhase::Fight if sim.footing() == Footing::Ledge && !sim.leaping() => {
			words.hunt_prompt_perched()
		}
		HuntPhase::Fight if sim.stunned() => words.hunt_prompt_stunned(),
		HuntPhase::Fight if sim.orb_block().is_none() => words.hunt_prompt_throw(),
		HuntPhase::Fight if sim.orb_block() == Some(OrbBlock::LowSoul) => {
			words.hunt_prompt_low_soul()
		}
		HuntPhase::Fight if any_seal_open => words.hunt_prompt_seals(),
		HuntPhase::Fight => words.hunt_prompt_fire(),
		HuntPhase::Windup => words.hunt_prompt_windup(),
		HuntPhase::Shriek => words.hunt_prompt_hold(),
		HuntPhase::Charge => words.hunt_prompt_charge(),
		HuntPhase::Broken => words.hunt_prompt_broken(),
		HuntPhase::Capture => words.hunt_prompt_capture(),
		HuntPhase::Absorbed | HuntPhase::Escaped => afterword,
	};
	let gauge = match phase {
		HuntPhase::Shriek => {
			Some(Gauge::Steadiness {
				fraction: step(turn_speed / stats.flinch_speed),
				steady: turn_speed <= stats.flinch_speed,
			})
		}
		HuntPhase::Broken if sim.broken_ticks() > 0 => {
			Some(Gauge::Broken {
				fraction: step(sim.broken_ticks_left() as f64 / sim.broken_ticks() as f64),
			})
		}
		_ => None,
	};
	// A routine hit is said only while there is something to hit.
	let banner = banner
		.filter(|banner| !banner.strike || phase == HuntPhase::Fight || phase == HuntPhase::Charge);
	Status {
		title: title.to_owned(),
		seconds: (phase != HuntPhase::Stalk && !sim.is_over())
			.then(|| (sim.fight_ticks_left() + HuntRules::TICK_RATE - 1) / HuntRules::TICK_RATE),
		ward: step(if sim.ward_max() > 0.0 {
			sim.ward() / sim.ward_max()
		} else {
			0.0
		}),
		resolve: sim.resolve(),
		resolve_max: stats.resolve,
		cadence: sim.cadence(),
		cadence_left: step(
			sim.cadence_ticks_left() as f64 / HuntRules::CADENCE_WINDOW_TICKS as f64,
		),
		banner,
		gauge,
		prompt: prompt.to_owned(),
		loud: matches!(phase, HuntPhase::Shriek | HuntPhase::Charge),
	}
}

fn thin_meter(fraction: f64, colour: Rgb) -> Element {
	View()
		.class("w-full h-[3px] bg-[#e9e4d42e]")
		.child(View().class(format!(
			"h-full w-[{:.0}%] bg-[{}]",
			fraction * 100.0,
			colour.hex()
		)))
}

/// The ledger over the fight, in the husk's own `light`. `company` is
/// whoever else is in the fight, shown under the reaper's own standing.
pub fn status_view(
	words: &dyn Messages,
	status: &Status,
	light: Rgb,
	company: Option<Element>,
) -> Element {
	let mut top =
		View()
			.class("flex-row items-end gap-3")
			.child(
				View()
					.class("flex-1")
					.child(display(status.title.clone(), 22, BONE)),
			);
	if let Some(seconds) = status.seconds {
		top = top.child(figure(words.hunt_seconds(seconds), 13, BONE));
	}
	let mut resolve = View()
		.class("flex-row items-center gap-[6px]")
		.child(label(words.hunt_resolve(), 11, BONE))
		.child(View().class("w-1"));
	for pip in 0..status.resolve_max {
		resolve = resolve.child(ledger_pip(pip < status.resolve, BONE));
	}
	resolve = resolve.child(View().class("flex-1"));
	if status.cadence > 1 {
		resolve = resolve.child(label(&words.hunt_cadence(status.cadence), 11, light));
	}
	let mut column = View()
		.class("absolute inset-0 px-5 pt-3 pb-7 pt-safe pb-safe")
		.test_id("hunt-status")
		.child(top)
		.child(gap(8))
		.child(ledger_meter(words.hunt_ward(), status.ward, light))
		.child(gap(10))
		.child(resolve);
	if status.cadence > 0 && status.seconds.is_some() {
		column = column
			.child(gap(6))
			.child(thin_meter(status.cadence_left, light));
	}
	if let Some(company) = company {
		column = column.child(gap(12)).child(company);
	}
	column = column.child(gap(30));
	if let Some(banner) = &status.banner {
		let words_of_it = if banner.strike {
			body(banner.text.clone(), 18, BONE)
		} else {
			display(banner.text.clone(), 40, BONE)
		};
		column = column.child(
			View()
				.class("items-center")
				.test_id("hunt-banner")
				.child(words_of_it),
		);
	}
	column = column.child(View().class("flex-1"));
	if let Some(gauge) = &status.gauge {
		let (name, fraction, colour) = match gauge {
			Gauge::Steadiness { fraction, steady } => {
				(
					if *steady {
						words.hunt_motion_steady()
					} else {
						words.hunt_motion_unsteady()
					},
					*fraction,
					if *steady { light } else { WOUND },
				)
			}
			Gauge::Broken { fraction } => (words.hunt_broken_window(), *fraction, BONE),
		};
		column = column.child(
			View()
				.class("pb-3")
				.child(label(name, 11, BONE))
				.child(gap(6))
				.child(thin_meter(fraction, colour)),
		);
	}
	let prompt = if status.loud {
		display(status.prompt.clone(), 30, BONE)
	} else {
		body(status.prompt.clone(), 16, BONE)
	};
	column.child(
		View()
			.class("items-center")
			.test_id("hunt-prompt")
			.child(prompt),
	)
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::EscapeCause;
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
	fn a_hit_is_quiet_and_a_phase_change_is_loud() {
		let words = En;
		let hit = banner_for(
			&words,
			&HuntEvent::HitLanded {
				weapon: reapers_rules::Archetype::Gun,
				damage: 6.4,
				cadence: 1,
				critical: false,
				empowered: false,
				seal: None,
				cleaved: Vec::new(),
			},
		)
		.unwrap();
		assert!(hit.strike);
		assert_eq!(hit.text, words.hunt_hit(6));
		let broken = banner_for(&words, &HuntEvent::Broken { window_ticks: 300 }).unwrap();
		assert!(!broken.strike);
		assert_eq!(
			banner_for(&words, &HuntEvent::HuskEscaped(EscapeCause::Time)),
			None
		);
	}

	#[test]
	fn a_stalk_asks_for_stillness_and_says_how_to_look() {
		let words = En;
		let sim = sim();
		let still = status(&words, "A husk", &sim, 0.0, true, None, "");
		assert_eq!(still.prompt, words.hunt_prompt_stalk());
		assert_eq!(still.seconds, None, "the clock starts with the fight");
		assert_eq!(still.ward, 1.0);
		let dragged = status(&words, "A husk", &sim, 0.0, false, None, "");
		assert_eq!(dragged.prompt, words.hunt_prompt_stalk_drag());
		let hurried = status(&words, "A husk", &sim, 9.0, true, None, "");
		assert_eq!(hurried.prompt, words.hunt_prompt_too_fast());
	}

	#[test]
	fn a_strike_banner_waits_for_something_to_strike() {
		let words = En;
		let banner = Banner {
			text: "6".to_owned(),
			strike: true,
		};
		let quiet = status(&words, "A husk", &sim(), 0.0, true, Some(banner), "");
		assert_eq!(quiet.banner, None);
	}
}
