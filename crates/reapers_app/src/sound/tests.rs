//! The Flutter game's `test/sound/`: how the music is heard, and what the
//! soundtrack tells its engine.

use std::cell::RefCell;
use std::rc::Rc;

use reapers_rules::Difficulty;
use reapers_rules::Gait;
use reapers_rules::HuntBot;
use reapers_rules::HuntInput;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use reapers_rules::Sigil;
use reapers_rules::Vessel;

use super::*;

fn assert_close(actual: f64, expected: f64, within: f64) {
	assert!(
		(actual - expected).abs() <= within,
		"{actual} is not within {within} of {expected}"
	);
}

fn hunt_of(rarity: Rarity) -> HuntSim {
	let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).expect("the borrowed reaper");
	HuntSim::open(4, Vessel::Ape, rarity.ward(), &stats)
}

fn hunt() -> HuntSim {
	hunt_of(Rarity::Epic)
}

fn still(sim: &HuntSim) -> HuntInput {
	HuntInput::measured(sim.pose().direction(), 0.0, &[], false)
}

/// Looks at the husk and swings at a charge, so the hunt runs on.
fn wary(sim: &HuntSim) -> HuntInput {
	let turn = if sim.charge_progress() > 0.3 {
		sim.stats().parry_speed + 0.5
	} else {
		0.0
	};
	HuntInput::measured(sim.pose().direction(), turn, &[], false)
}

fn until(sim: &mut HuntSim, stop: impl Fn(&HuntSim) -> bool) {
	while !stop(sim) && !sim.is_over() {
		let input = wary(sim);
		sim.step(&input);
	}
}

mod the_veil {
	use super::*;

	#[test]
	fn clear_lets_everything_through_and_full_lets_only_the_lows() {
		assert_close(
			Mix {
				volume: 1.0,
				veil: 0.0,
			}
			.cutoff(),
			16_000.0,
			1.0,
		);
		assert_close(
			Mix {
				volume: 1.0,
				veil: 1.0,
			}
			.cutoff(),
			320.0,
			1.0,
		);
	}

	#[test]
	fn each_step_takes_off_the_same_share_of_what_is_left() {
		let at = |veil| Mix { volume: 1.0, veil }.cutoff();
		assert_close(at(0.5) / at(0.25), at(0.75) / at(0.5), 1e-9);
	}

	#[test]
	fn a_change_too_small_to_hear_is_no_change() {
		let mix = Mix {
			volume: 0.5,
			veil: 0.5,
		};
		let other = |volume, veil| Mix { volume, veil };
		assert!(!other(0.505, 0.5).differs_from(mix));
		assert!(other(0.55, 0.5).differs_from(mix));
		assert!(other(0.5, 0.6).differs_from(mix));
	}
}

mod the_music_of_a_sigil {
	use reapers_rules::SigilTrace;

	use super::*;

	#[test]
	fn draws_nearer_with_every_light_to_where_the_stalk_takes_it_up() {
		let sigil = Sigil::draw(3, &Rarity::Rare.sigil(), Gait::Ground, Difficulty::Normal);
		let mut trace = SigilTrace::new(sigil, 0.0);
		let mut heard = vec![sigil_mix(&trace)];

		while !trace.is_closed() {
			let aim = trace.direction_of(trace.caught());
			trace.advance(aim, 0.0, 1.0 / 60.0);

			if trace.fill() == 0.0 {
				heard.push(sigil_mix(&trace));
			}
		}

		assert_eq!(heard[0].volume, Mix::WAITING.volume);
		assert_eq!(heard[0].veil, Mix::WAITING.veil);
		for step in heard.windows(2) {
			assert!(step[1].volume > step[0].volume);
			assert!(step[1].veil < step[0].veil);
		}
		let last = heard[heard.len() - 1];
		assert_close(last.volume, hunt_mix(&hunt()).volume, 1e-9);
		assert_close(last.veil, hunt_mix(&hunt()).veil, 1e-9);
	}
}

mod the_music_of_a_hunt {
	use super::*;

	#[test]
	fn is_far_off_while_the_husk_has_not_been_found() {
		let sim = hunt();
		let mix = hunt_mix(&sim);
		assert_eq!(sim.phase(), HuntPhase::Stalk);
		assert!(mix.volume < 0.4);
		assert!(mix.veil > 0.7);
	}

	#[test]
	fn draws_nearer_as_the_reaper_steadies_on_the_husk() {
		let mut sim = hunt();
		let unseen = hunt_mix(&sim);

		while sim.phase() == HuntPhase::Stalk && sim.steadiness() < 0.8 {
			let input = still(&sim);
			sim.step(&input);
		}

		let steadied = hunt_mix(&sim);
		assert!(steadied.volume > unseen.volume);
		assert!(steadied.veil < unseen.veil);
	}

	#[test]
	fn comes_clear_when_the_fight_is_on() {
		let mut sim = hunt();
		until(&mut sim, |sim| sim.phase() == HuntPhase::Fight);
		assert_eq!(hunt_mix(&sim).veil, 0.0);
		assert!(hunt_mix(&sim).volume > 0.7);
	}

	#[test]
	fn drains_away_as_the_husk_draws_breath_to_near_silence() {
		let mut sim = hunt();
		until(&mut sim, |sim| sim.phase() == HuntPhase::Windup);
		let early = hunt_mix(&sim);
		until(&mut sim, |sim| sim.shriek_progress() > 0.8);
		let late = hunt_mix(&sim);
		until(&mut sim, |sim| sim.phase() == HuntPhase::Shriek);
		let shriek = hunt_mix(&sim);

		assert!(late.volume < early.volume);
		assert!(late.veil > early.veil);
		assert!(shriek.volume < 0.1);
		assert_eq!(shriek.veil, 1.0);
	}

	#[test]
	fn swells_when_the_husk_charges() {
		let mut sim = hunt();
		until(&mut sim, |sim| sim.phase() == HuntPhase::Charge);
		assert_eq!(hunt_mix(&sim).volume, 1.0);
		assert_eq!(hunt_mix(&sim).veil, 0.0);
	}

	#[test]
	fn is_gone_once_the_husk_is_reaped() {
		let mut sim = hunt_of(Rarity::Common);
		HuntBot::careful(&sim).play(&mut sim);
		assert_eq!(sim.phase(), HuntPhase::Absorbed);
		assert_eq!(hunt_mix(&sim).volume, 0.0);
	}

	#[test]
	fn never_asks_for_more_than_the_engine_can_give() {
		let mut sim = hunt();
		let mut bot = HuntBot::human(&sim);

		while !sim.is_over() {
			bot.step_once(&mut sim);
			let mix = hunt_mix(&sim);
			assert!((0.0..=1.0).contains(&mix.volume), "{mix:?}");
			assert!((0.0..=1.0).contains(&mix.veil), "{mix:?}");
		}
	}
}

/// An engine that writes down what it is told.
#[derive(Default)]
struct Listening(Rc<RefCell<Vec<String>>>);

impl ScoreEngine for Listening {
	fn play(&self, piece: Score, mix: Mix, _fade: Duration) {
		self.0
			.borrow_mut()
			.push(format!("play {} at {:?}", piece.name(), mix.volume));
	}

	fn shape(&self, mix: Mix) {
		self.0
			.borrow_mut()
			.push(format!("shape {:?}/{:?}", mix.volume, mix.veil));
	}

	fn silence(&self, _fade: Duration) {
		self.0.borrow_mut().push("silence".into());
	}

	fn hold(&self, held: bool) {
		self.0
			.borrow_mut()
			.push(if held { "hold" } else { "carry on" }.into());
	}
}

fn soundtrack(enabled: bool) -> (Soundtrack, Rc<RefCell<Vec<String>>>) {
	let told = Rc::new(RefCell::new(Vec::new()));
	(
		Soundtrack::new(Box::new(Listening(told.clone())), enabled),
		told,
	)
}

fn mix(volume: f64, veil: f64) -> Mix {
	Mix { volume, veil }
}

#[test]
fn a_piece_is_brought_in_once_however_often_it_is_asked_for() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.play(Score::Sanctum, Mix::OPEN);
	soundtrack.play(Score::Sanctum, Mix::OPEN);
	assert_eq!(*told.borrow(), ["play sanctum at 0.55"]);
	assert_eq!(soundtrack.piece(), Some(Score::Sanctum));
}

#[test]
fn another_piece_takes_over_from_the_one_playing() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.play(Score::Sanctum, Mix::OPEN);
	soundtrack.play(Score::Hunt, Mix::WAITING);
	assert_eq!(
		*told.borrow(),
		["play sanctum at 0.55", "play hunt at 0.22"]
	);
}

#[test]
fn the_engine_is_only_told_of_changes_that_would_be_heard() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.play(Score::Hunt, mix(0.5, 0.0));
	soundtrack.shape(mix(0.503, 0.0));
	soundtrack.shape(mix(0.8, 0.0));
	soundtrack.shape(mix(0.8, 0.0));
	assert_eq!(*told.borrow(), ["play hunt at 0.5", "shape 0.8/0.0"]);
}

#[test]
fn asking_for_the_piece_already_playing_reshapes_it() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.play(Score::Hunt, Mix::WAITING);
	soundtrack.play(Score::Hunt, mix(0.8, 0.0));
	assert_eq!(*told.borrow(), ["play hunt at 0.22", "shape 0.8/0.0"]);
}

#[test]
fn with_music_off_the_engine_hears_nothing() {
	let (soundtrack, told) = soundtrack(false);
	soundtrack.play(Score::Hunt, Mix::OPEN);
	soundtrack.shape(mix(1.0, 0.0));
	assert!(told.borrow().is_empty());
	assert_eq!(
		soundtrack.piece(),
		Some(Score::Hunt),
		"it still knows what belongs"
	);
}

#[test]
fn turned_back_on_the_music_comes_back_as_it_should_be_now() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.play(Score::Sanctum, Mix::OPEN);
	soundtrack.set_enabled(false);
	soundtrack.play(Score::Hunt, mix(0.8, 0.0));
	soundtrack.set_enabled(true);
	assert_eq!(
		*told.borrow(),
		["play sanctum at 0.55", "silence", "play hunt at 0.8",]
	);
}

#[test]
fn off_the_screen_the_music_is_held_and_carries_on_after() {
	let (soundtrack, told) = soundtrack(true);
	soundtrack.hold(true);
	soundtrack.hold(false);
	assert_eq!(*told.borrow(), ["hold", "carry on"]);
}
