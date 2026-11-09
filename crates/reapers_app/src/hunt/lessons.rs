//! What a first practice hunt teaches, through the caption line under the
//! crosshair: one thing at a time, as each becomes relevant, and each gone
//! for good once it is done. There is no tutorial to read and nothing to
//! dismiss.
//!
//! - **Fire**: centre the husk and press FIRE, until a blow lands.
//! - **Reload**: an empty pistol reloads on FIRE, and FIRE again in the
//!   bright band finishes it early. Learned by a perfect reload, or after
//!   two reloads either way, so that it never nags.
//! - **Orb**: once the husk is weak, hold ORB and let go as the ring closes.
//!   Learned by the first throw.
//!
//! What has been learned is kept in the device's preferences, so the
//! lessons are taught once, not every practice.

use reapers_rules::Archetype;
use reapers_rules::Arsenal;
use reapers_rules::HuntEvent;
use reapers_rules::HuntPhase;
use reapers_rules::HuntSim;

use crate::l10n::Messages;

/// One thing a first practice teaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lesson {
	Fire,
	Reload,
	Orb,
}

impl Lesson {
	const fn bit(self) -> u8 {
		1 << self as u8
	}
}

/// The lessons a player has learned, as the preferences keep them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Learned(u8);

impl Learned {
	/// Where the preferences keep them.
	pub const KEY: &'static str = "reapers.lessons";

	/// What the preferences held, ignoring lessons this build does not know.
	pub const fn from_bits(bits: u8) -> Self {
		Self(bits & 0b111)
	}

	pub const fn bits(self) -> u8 {
		self.0
	}

	pub const fn knows(self, lesson: Lesson) -> bool {
		self.0 & lesson.bit() != 0
	}

	/// Whether every lesson is learned, and the caption has nothing to teach.
	pub const fn all(self) -> bool {
		self.0 == 0b111
	}

	/// Adds what `other` knows.
	#[must_use]
	pub const fn with(self, other: Self) -> Self {
		Self(self.0 | other.0)
	}

	fn learn(&mut self, lesson: Lesson) -> bool {
		let new = !self.knows(lesson);
		self.0 |= lesson.bit();
		new
	}
}

/// What the caption line teaches right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
	/// Centre it and press FIRE.
	Fire,
	/// The pistol is empty: FIRE reloads it.
	Empty,
	/// Reloading: FIRE in the bright band finishes it.
	Band,
	/// FIRE came before the band and the pistol jammed.
	Jammed,
	/// It is weak enough: hold ORB.
	Orb,
	/// ORB is held: let go as the ring closes on it.
	LetGo,
}

impl Prompt {
	/// The lesson this prompt is part of.
	pub const fn lesson(self) -> Lesson {
		match self {
			Self::Fire => Lesson::Fire,
			Self::Empty | Self::Band | Self::Jammed => Lesson::Reload,
			Self::Orb | Self::LetGo => Lesson::Orb,
		}
	}

	/// What the caption says.
	pub fn text(self, words: &dyn Messages) -> &str {
		match self {
			Self::Fire => words.hunt_teach_fire(),
			Self::Empty => words.hunt_teach_empty(),
			Self::Band => words.hunt_teach_band(),
			Self::Jammed => words.hunt_teach_jammed(),
			Self::Orb => words.hunt_teach_orb(),
			Self::LetGo => words.hunt_teach_let_go(),
		}
	}
}

/// The teacher for one practice hunt.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Teacher {
	learned: Learned,
	/// Reloads finished without a perfect one.
	reloads: u8,
}

impl Teacher {
	/// How many ordinary reloads teach the reload as well as a perfect one.
	const RELOADS: u8 = 2;

	/// A teacher for a player who knows `learned`.
	pub const fn new(learned: Learned) -> Self {
		Self {
			learned,
			reloads: 0,
		}
	}

	pub const fn learned(&self) -> Learned {
		self.learned
	}

	/// Adds what the preferences say was learned before.
	pub const fn recall(&mut self, before: Learned) {
		self.learned = self.learned.with(before);
	}

	/// Hears what the rules decided. Answers whether a lesson was learned.
	pub fn hear(&mut self, event: &HuntEvent) -> bool {
		match event {
			HuntEvent::HitLanded { .. } => self.learned.learn(Lesson::Fire),
			HuntEvent::ReloadFinished { perfect: true } => self.learned.learn(Lesson::Reload),
			HuntEvent::ReloadFinished { perfect: false } => {
				self.reloads += 1;
				self.reloads >= Self::RELOADS && self.learned.learn(Lesson::Reload)
			}
			HuntEvent::OrbThrown { .. } => self.learned.learn(Lesson::Orb),
			_ => false,
		}
	}

	/// What to teach now, if anything: the lesson not yet learned whose
	/// moment this is. A reload is taught first, as its window is short;
	/// then the orb, as a weak husk does not wait; then FIRE.
	pub fn prompt(&self, sim: &HuntSim) -> Option<Prompt> {
		if !matches!(
			sim.phase(),
			HuntPhase::Fight | HuntPhase::Windup | HuntPhase::Charge | HuntPhase::Broken
		) || sim.orb().as_ref().is_some_and(super::controls::orb_told)
		{
			return None;
		}
		let gun = Arsenal::archetype_of(sim.selected_slot()) == Some(Archetype::Gun)
			&& sim.draw_ticks_left() == 0;
		let knows = |lesson| self.learned.knows(lesson);
		if gun && !knows(Lesson::Reload) {
			if sim.reload_jammed() {
				return Some(Prompt::Jammed);
			}
			if sim.reloading() {
				return Some(Prompt::Band);
			}
			if sim.rounds() == 0 {
				return Some(Prompt::Empty);
			}
		}
		if !knows(Lesson::Orb) {
			if sim.orb_ring().is_some() {
				return Some(Prompt::LetGo);
			}
			if sim.orb_block().is_none() {
				return Some(Prompt::Orb);
			}
		}
		let firing =
			sim.phase() == HuntPhase::Fight && !(gun && (sim.reloading() || sim.rounds() == 0));
		(!knows(Lesson::Fire) && firing).then_some(Prompt::Fire)
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::Difficulty;
	use reapers_rules::HuntInput;
	use reapers_rules::HuntLoadout;
	use reapers_rules::HuntPlan;
	use reapers_rules::HuntStats;
	use reapers_rules::market_listings;

	use super::*;

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
			sim.step(&HuntInput::still(sim.pose().direction(), 0.0));
		}
	}

	#[test]
	fn the_first_lesson_waits_for_the_fight_and_goes_once_a_blow_lands() {
		let mut sim = sim();
		let mut teacher = Teacher::default();
		assert_eq!(
			teacher.prompt(&sim),
			None,
			"nothing to teach while stalking"
		);
		found(&mut sim);
		assert_eq!(teacher.prompt(&sim), Some(Prompt::Fire));
		let mut learned = false;
		for _ in 0..600 {
			let aim = sim.pose().direction();
			for event in sim.step(&HuntInput::measured(aim, 0.0, true, 0, false)) {
				learned |= teacher.hear(&event);
			}
			if learned {
				break;
			}
		}
		assert!(learned, "a shot at the husk lands");
		assert!(teacher.learned().knows(Lesson::Fire));
		assert_ne!(teacher.prompt(&sim), Some(Prompt::Fire));
	}

	#[test]
	fn an_empty_pistol_teaches_the_reload_one_step_at_a_time() {
		let mut sim = sim();
		let mut teacher = Teacher::default();
		found(&mut sim);
		let mut seen = Vec::new();
		for _ in 0..900 {
			let prompt = teacher.prompt(&sim);
			if seen.last() != Some(&prompt) {
				seen.push(prompt);
			}
			// Pressed as early as it can be, which jams the first reload.
			let aim = sim.pose().direction();
			for event in sim.step(&HuntInput::measured(aim, 0.0, true, 0, false)) {
				teacher.hear(&event);
			}
			if teacher.learned().knows(Lesson::Reload) {
				break;
			}
		}
		assert!(seen.contains(&Some(Prompt::Empty)), "{seen:?}");
		assert!(seen.contains(&Some(Prompt::Jammed)), "{seen:?}");
		assert!(teacher.learned().knows(Lesson::Reload), "{seen:?}");
	}

	#[test]
	fn a_perfect_reload_is_learned_at_once_and_ordinary_ones_after_two() {
		let mut teacher = Teacher::default();
		assert!(teacher.hear(&HuntEvent::ReloadFinished { perfect: true }));
		assert!(!teacher.hear(&HuntEvent::ReloadFinished { perfect: true }));
		let mut teacher = Teacher::default();
		assert!(!teacher.hear(&HuntEvent::ReloadFinished { perfect: false }));
		assert!(teacher.hear(&HuntEvent::ReloadFinished { perfect: false }));
	}

	#[test]
	fn what_was_learned_before_is_not_taught_again() {
		let mut sim = sim();
		found(&mut sim);
		let mut teacher = Teacher::default();
		teacher.recall(Learned::from_bits(0xFF));
		assert!(teacher.learned().all());
		assert_eq!(teacher.prompt(&sim), None);
		assert_eq!(Learned::from_bits(0xFF).bits(), 0b111);
	}
}
