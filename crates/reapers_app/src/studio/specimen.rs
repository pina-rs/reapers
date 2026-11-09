//! The studio's specimen: an unsaved reaper, its growth and its motion, as
//! plain data. Every figure comes from `reapers_rules`; this only keeps the
//! choices the Flutter `ReaperStudio` kept in its hooks.

use reapers_rules::CaptureGrade;
use reapers_rules::Feeding;
use reapers_rules::GenomeChanges;
use reapers_rules::HuntDeeds;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntOutcome;
use reapers_rules::HuntStats;
use reapers_rules::MotionGroup;
use reapers_rules::Outfit;
use reapers_rules::Progression;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperGift;
use reapers_rules::ReaperMotion;
use reapers_rules::ReaperSoul;
use reapers_rules::SoulMemory;
use reapers_rules::SoulPath;
use reapers_rules::SoulRules;
use reapers_rules::Temper;
use reapers_rules::Weapon;
use reapers_rules::WeaponKind;

use crate::render::appearance::ReaperAppearance;
use crate::render::camera::STUDIO_DISTANCE;
use crate::render::playback::ReaperPlayback;
use crate::render::playback::expression;
use crate::render::stage::StageShow;

/// The seed the Flutter studio drew its first specimen from, and counted
/// up from for each shuffle.
const FIRST_SEED: i64 = 11;

/// The speeds the studio offers. The host plays a clip on its own clock,
/// from a quarter speed to full.
pub const SPEEDS: [f64; 3] = [0.25, 0.5, 1.0];

/// The three sections under the stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Section {
	#[default]
	Appearance,
	Growth,
	Soul,
}

/// How many entries the gallery bench steps through.
const GALLERY_SIZE: usize = ReaperClass::ALL.len() * Outfit::ALL.len();

/// What the gallery bench steps through: every calling, in every outfit.
pub const GALLERY: [(ReaperClass, Outfit); GALLERY_SIZE] = {
	let mut entries = [(ReaperClass::Harvester, Outfit::Shrouded); GALLERY_SIZE];
	let mut index = 0;
	while index < GALLERY_SIZE {
		entries[index] = (
			ReaperClass::ALL[index / Outfit::ALL.len()],
			Outfit::ALL[index % Outfit::ALL.len()],
		);
		index += 1;
	}
	entries
};

/// One unsaved specimen and everything the studio shows of it.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(
	clippy::struct_excessive_bools,
	reason = "each is one switch in the studio"
)]
pub struct Specimen {
	/// The reaper the studio was opened on. Nothing here changes it.
	pub opened: ReaperGenome,
	pub genome: ReaperGenome,
	pub xp: i64,
	pub memory: SoulMemory,
	pub outfit: Outfit,
	pub temper: Temper,
	pub section: Section,
	seed: i64,
	pub captures: i64,
	/// The XP the last absorption earned, until the level is moved.
	pub reward: Option<i64>,
	pub spinning: bool,
	/// Counts each start of the rotation, so a stop timer can tell whether
	/// its rotation is still the one turning.
	pub spin: u32,
	pub focus_face: bool,
	pub appearance: ReaperAppearance,
	/// The object tried in the hands, or `None` for the calling's own.
	pub held: Option<WeaponKind>,
	/// Whether the animations are open.
	pub motions_open: bool,
	pub group: MotionGroup,
	/// What the stage's clock was last told.
	pub clock: ReaperPlayback,
	/// Counts each change the stage's clock must be told of.
	pub cue: u32,
	/// Whether the face is held in the clip's expression.
	pub inspecting: bool,
	/// Whether the genome's code is shown, for copying.
	pub code_shown: bool,
	/// Whether the code went on the clipboard as well.
	pub code_copied: bool,
	/// Whether the Book of Becoming is open over the studio.
	pub book: bool,
	/// Where the gallery bench stands, when the studio is one.
	pub gallery: Option<usize>,
}

impl Specimen {
	/// A specimen of `genome`, as the studio opens on it.
	pub fn new(genome: ReaperGenome) -> Self {
		Self {
			opened: genome,
			genome,
			xp: 0,
			memory: SoulMemory::default(),
			outfit: Outfit::Shrouded,
			temper: Temper::Shrouded,
			section: Section::Appearance,
			seed: FIRST_SEED,
			captures: 0,
			reward: None,
			spinning: false,
			spin: 0,
			focus_face: false,
			appearance: ReaperAppearance::default(),
			held: None,
			motions_open: false,
			group: MotionGroup::Movement,
			clock: ReaperPlayback::default(),
			cue: 0,
			inspecting: false,
			code_shown: false,
			code_copied: false,
			book: false,
			gallery: None,
		}
	}

	/// The gallery bench, at its first calling and outfit.
	pub fn gallery(genome: ReaperGenome) -> Self {
		let mut specimen = Self::new(genome);
		specimen.gallery = Some(0);
		specimen.show_gallery();
		specimen
	}

	pub fn level(&self) -> i64 {
		Progression::level_for_xp(self.xp)
	}

	pub fn weapon(&self) -> Weapon {
		Weapon::new(
			self.held.unwrap_or(self.genome.reaper_class.weapon()),
			self.temper,
		)
	}

	/// What the hands hold, or `None` for empty hands.
	pub fn visible_weapon(&self) -> Option<WeaponKind> {
		self.appearance.holding.then(|| self.weapon().kind)
	}

	pub fn soul(&self) -> ReaperSoul {
		ReaperSoul::from_genome(&self.genome)
	}

	pub fn soul_rank(&self) -> i64 {
		SoulRules::rank(self.level(), &self.memory, self.soul().gift)
	}

	/// The strike damage of this specimen, and of a new reaper of its
	/// calling and soul carrying its starting weapon.
	pub fn strikes(&self) -> (f64, f64) {
		let calling = self.genome.reaper_class;
		let soul = Some(self.soul());
		let now = HuntLoadout {
			soul,
			soul_rank: self.soul_rank(),
			..HuntLoadout::new(calling, self.level(), self.weapon())
		};
		let start = HuntLoadout {
			soul,
			..HuntLoadout::new(calling, 1, Weapon::new(calling.weapon(), Temper::Shrouded))
		};
		let damage =
			|loadout: &HuntLoadout| HuntStats::derive(loadout, 1.0).map_or(0.0, |stats| stats.hit);
		(damage(&now), damage(&start))
	}

	/// How much harder it strikes than a new reaper, in percent.
	pub fn strength_percent(&self) -> i64 {
		let (now, start) = self.strikes();
		if start <= 0.0 {
			return 0;
		}
		((now / start - 1.0) * 100.0).round() as i64
	}

	/// What the stage shows.
	pub fn show(&self) -> StageShow {
		StageShow {
			genome: self.genome,
			weapon: self.weapon(),
			outfit: self.outfit,
			appearance: self.appearance,
			turns_on_its_own: self.spinning,
			focus_face: self.focus_face,
			distance: STUDIO_DISTANCE,
		}
	}

	/// The expression the face is held in.
	pub fn face(&self) -> Option<(&'static str, f32)> {
		if self.inspecting {
			expression(self.clock.motion())
		} else {
			None
		}
	}

	/// Tries birth traits; a change the genome's bounds refuse is ignored.
	pub fn change(&mut self, changes: GenomeChanges) {
		if let Ok(genome) = self.genome.with(changes) {
			self.genome = genome;
			self.settle_motion();
		}
	}

	pub fn choose_class(&mut self, reaper_class: ReaperClass) {
		self.change(GenomeChanges {
			reaper_class: Some(reaper_class),
			..GenomeChanges::default()
		});
	}

	pub fn dress(&mut self, appearance: ReaperAppearance) {
		self.appearance = appearance;
		self.settle_motion();
	}

	pub fn hold(&mut self, kind: WeaponKind) {
		self.held = Some(kind);
		self.settle_motion();
	}

	/// Draws the next specimen.
	pub fn shuffle(&mut self) {
		self.seed += 1;
		self.genome = ReaperGenome::shuffle(self.seed);
		self.code_shown = false;
		self.code_copied = false;
		self.settle_motion();
	}

	/// Moves to `next`, giving up equipment the level no longer reaches.
	pub fn set_level(&mut self, next: i64) {
		let next = next.clamp(1, Progression::LEVEL_CAP);
		self.xp = Progression::xp_for_level(next);
		self.reward = None;
		if self.temper.level_required() > next {
			self.temper = Temper::Shrouded;
		}
		if self.outfit.level_required() > next {
			self.outfit = Outfit::Shrouded;
		}
	}

	/// Absorbs an illustrative husk: a clean cut of a rare one, fairly fed,
	/// with the deed that teaches `path`, by the same rules as a real hunt.
	/// It never submits a claim or touches a save.
	pub fn absorb(&mut self, path: Option<SoulPath>) -> i64 {
		let outcome = HuntOutcome::absorbed_with(
			CaptureGrade::Great,
			path == Some(SoulPath::Unseen),
			0,
			HuntDeeds {
				holds: i64::from(path == Some(SoulPath::Stillness)),
				parries: 0,
				max_cadence: if path == Some(SoulPath::Rhythm) {
					SoulRules::CADENCE_FOR_MARK
				} else {
					0
				},
			},
		);
		let earned = outcome.xp_for(Rarity::Rare, Feeding::Fair);
		self.xp += earned;
		self.captures += 1;
		self.reward = Some(earned);
		self.memory = self.memory.remember(&outcome);
		earned
	}

	/// Moves the gaze just far enough that the soul is born with `gift`.
	pub fn try_gift(&mut self, gift: ReaperGift) {
		let per_class = SoulRules::GIFTS_PER_CLASS as i64;
		let delta = (gift.index() as i64 - self.soul().gift.index() as i64).rem_euclid(per_class);
		let mut gaze = i64::from(self.genome.gaze) + delta;
		if gaze > i64::from(ReaperGenome::TRAIT_SCALE) {
			gaze -= per_class;
		}
		self.change(GenomeChanges {
			gaze: Some(gaze as u32),
			..GenomeChanges::default()
		});
	}

	/// Level thirty, hollowed equipment and every awakening.
	pub fn see_potential(&mut self) {
		self.set_level(Progression::LEVEL_CAP);
		self.temper = Temper::Hollowed;
		self.outfit = Outfit::Hollowed;
		let last = SoulRules::RANK_MARKS[SoulRules::RANK_MARKS.len() - 1];
		self.memory = SoulMemory {
			unseen: last,
			stillness: last,
			rhythm: last,
		};
	}

	/// Equips `temper` if the level reaches it.
	pub fn choose_temper(&mut self, temper: Temper) {
		if temper.level_required() <= self.level() {
			self.temper = temper;
		}
	}

	/// Wears `outfit` if the level reaches it.
	pub fn choose_outfit(&mut self, outfit: Outfit) {
		if outfit.level_required() <= self.level() {
			self.outfit = outfit;
		}
	}

	/// Starts or stops the rotation; the face close-up never turns.
	pub fn toggle_spin(&mut self) {
		self.spinning = !self.spinning;
		if self.spinning {
			self.spin = self.spin.wrapping_add(1);
		}
	}

	pub fn toggle_face(&mut self) {
		self.focus_face = !self.focus_face;
		self.spinning = false;
		if !self.focus_face && self.inspecting {
			self.inspecting = false;
			self.cue += 1;
		}
	}

	/// Plays `motion` from its start, or holds it there under reduced
	/// motion. A finisher the hands cannot perform is refused.
	pub fn select_motion(&mut self, motion: ReaperMotion, reduced: bool) {
		if !motion.supports(self.visible_weapon()) {
			return;
		}
		self.clock.select(motion, !reduced);
		self.inspecting = false;
		self.cue += 1;
	}

	/// Pauses or plays. The host cannot hold a frame mid-clip, so a pause
	/// returns the clip to its first frame and play starts it again.
	pub fn press_play(&mut self, reduced: bool) {
		if reduced {
			return;
		}
		if self.clock.playing() {
			self.clock.toggle();
		} else {
			self.clock.replay();
		}
		self.inspecting = false;
		self.cue += 1;
	}

	pub fn replay(&mut self, reduced: bool) {
		self.clock.select(self.clock.motion(), !reduced);
		self.inspecting = false;
		self.cue += 1;
	}

	pub fn set_speed(&mut self, speed: f64) {
		if self.clock.set_speed(speed).is_ok() {
			self.cue += 1;
		}
	}

	/// Holds the clip and its face, and moves the camera onto the face.
	pub fn inspect_expression(&mut self) {
		if expression(self.clock.motion()).is_none() {
			return;
		}
		self.clock.select(self.clock.motion(), false);
		self.inspecting = true;
		self.focus_face = true;
		self.spinning = false;
		self.cue += 1;
	}

	/// A change of calling or hands can make a finisher impossible: the
	/// specimen goes back to idling.
	fn settle_motion(&mut self) {
		if !self.clock.motion().supports(self.visible_weapon()) {
			let playing = self.clock.playing();
			self.clock.select(ReaperMotion::Idle, playing);
			self.inspecting = false;
			self.cue += 1;
		}
	}

	/// Steps the gallery bench by `step` entries, wrapping round.
	pub fn step_gallery(&mut self, step: i64) {
		let Some(index) = self.gallery else {
			return;
		};
		let count = GALLERY.len() as i64;
		self.gallery = Some((index as i64 + step).rem_euclid(count) as usize);
		self.show_gallery();
	}

	fn show_gallery(&mut self) {
		let Some(index) = self.gallery else {
			return;
		};
		let (calling, outfit) = GALLERY[index];
		self.choose_class(calling);
		self.set_level(Progression::LEVEL_CAP);
		self.outfit = outfit;
	}
}

#[cfg(test)]
mod tests {
	use reapers_rules::PlayerState;

	use super::*;
	use crate::render::appearance::ReaperClothing;
	use crate::render::appearance::ReaperForm;

	fn saved() -> ReaperGenome {
		PlayerState::initial()
			.take_oath(11)
			.expect("an oath")
			.genome()
	}

	#[test]
	fn customization_and_progression_never_touch_the_reaper_it_opened_on() {
		let opened = saved();
		let mut specimen = Specimen::new(opened);
		specimen.choose_class(ReaperClass::Chooser);
		assert_eq!(specimen.genome.reaper_class, ReaperClass::Chooser);
		specimen.dress(ReaperAppearance {
			form: ReaperForm::Humanoid,
			..specimen.appearance
		});
		assert_eq!(specimen.appearance.clothing, ReaperClothing::Robe);
		specimen.dress(ReaperAppearance {
			clothing: ReaperClothing::Unrobed,
			..specimen.appearance
		});
		specimen.hold(WeaponKind::Scythe);
		assert_eq!(specimen.weapon().kind, WeaponKind::Scythe);
		specimen.dress(ReaperAppearance {
			holding: false,
			..specimen.appearance
		});
		assert_eq!(specimen.visible_weapon(), None);
		assert!(!specimen.show().appearance.holding);
		specimen.change(GenomeChanges {
			head: Some(1),
			..GenomeChanges::default()
		});
		assert_eq!(specimen.genome.head, 1);

		let earned = specimen.absorb(None);
		let fair = HuntOutcome::absorbed_with(CaptureGrade::Great, false, 0, HuntDeeds::default())
			.xp_for(Rarity::Rare, Feeding::Fair);
		assert_eq!(earned, fair);
		assert_eq!(specimen.reward, Some(fair));
		assert_eq!(specimen.captures, 1);

		specimen.see_potential();
		assert_eq!(specimen.weapon().temper, Temper::Hollowed);
		assert_eq!(specimen.level(), Progression::LEVEL_CAP);
		assert_eq!(specimen.soul_rank(), SoulRules::MAX_RANK);

		specimen.try_gift(ReaperGift::Reprisal);
		assert_eq!(specimen.soul().gift, ReaperGift::Reprisal);
		assert_eq!(specimen.soul_rank(), SoulRules::MAX_RANK);
		specimen.memory = SoulMemory::default();
		assert_eq!(specimen.soul_rank(), 0);
		specimen.absorb(Some(SoulPath::Stillness));
		assert_eq!(specimen.memory.stillness, 1);
		assert_eq!(specimen.soul_rank(), 0);
		assert_eq!(
			specimen.opened, opened,
			"the reaper it opened on is unchanged"
		);
	}

	#[test]
	fn trying_each_gift_of_every_calling_lands_on_it() {
		for calling in ReaperClass::ALL {
			for gaze in [0, 1, 2, 500, 998, 999, 1000] {
				let genome = ReaperGenome {
					reaper_class: calling,
					gaze,
					..ReaperGenome::BORROWED
				};
				for gift in ReaperGift::ALL
					.into_iter()
					.filter(|gift| gift.reaper_class() == calling)
				{
					let mut specimen = Specimen::new(genome);
					specimen.try_gift(gift);
					assert_eq!(specimen.soul().gift, gift, "{calling:?} at gaze {gaze}");
				}
			}
		}
	}

	#[test]
	fn moving_the_level_gives_up_equipment_it_no_longer_reaches() {
		let mut specimen = Specimen::new(saved());
		specimen.choose_temper(Temper::Hollowed);
		assert_eq!(specimen.temper, Temper::Shrouded, "a locked temper refuses");
		specimen.see_potential();
		assert_eq!(specimen.outfit, Outfit::Hollowed);
		specimen.set_level(Temper::Sworn.level_required());
		assert_eq!(specimen.temper, Temper::Shrouded);
		assert_eq!(specimen.outfit, Outfit::Shrouded);
		specimen.choose_temper(Temper::Sworn);
		assert_eq!(specimen.temper, Temper::Sworn);
		assert!(specimen.strength_percent() > 0);
		specimen.set_level(1);
		assert_eq!(specimen.strength_percent(), 0);
		assert_eq!(specimen.reward, None);
	}

	#[test]
	fn a_finisher_the_hands_cannot_perform_returns_to_idle() {
		let mut specimen = Specimen::new(ReaperGenome {
			reaper_class: ReaperClass::Chooser,
			..ReaperGenome::BORROWED
		});
		specimen.hold(WeaponKind::Spear);
		specimen.select_motion(ReaperMotion::FinisherImpale, false);
		assert_eq!(specimen.clock.motion(), ReaperMotion::FinisherImpale);
		assert!(specimen.clock.playing());
		let cue = specimen.cue;
		specimen.hold(WeaponKind::Bell);
		assert_eq!(specimen.clock.motion(), ReaperMotion::Idle);
		assert!(specimen.cue > cue, "the stage is told");
		specimen.select_motion(ReaperMotion::FinisherImpale, false);
		assert_eq!(
			specimen.clock.motion(),
			ReaperMotion::Idle,
			"refused while a bell is held"
		);
	}

	#[test]
	fn play_pause_replay_and_speed_drive_the_clock() {
		let mut specimen = Specimen::new(saved());
		specimen.select_motion(ReaperMotion::DashLeft, false);
		assert_eq!(specimen.clock.motion(), ReaperMotion::DashLeft);
		assert!(specimen.clock.playing());
		specimen.press_play(false);
		assert!(!specimen.clock.playing(), "paused");
		specimen.press_play(false);
		assert!(specimen.clock.playing());
		assert_eq!(specimen.clock.seconds(), 0.0, "play starts it again");
		specimen.set_speed(0.25);
		assert_eq!(specimen.clock.speed(), 0.25);
		specimen.replay(false);
		assert!(specimen.clock.playing());

		// Reduced motion holds every clip and refuses play.
		specimen.select_motion(ReaperMotion::DashRight, true);
		assert!(!specimen.clock.playing());
		specimen.press_play(true);
		assert!(!specimen.clock.playing());
		specimen.replay(true);
		assert!(!specimen.clock.playing());
	}

	#[test]
	fn inspecting_an_expression_holds_the_face_and_frames_it() {
		let mut specimen = Specimen::new(saved());
		specimen.select_motion(ReaperMotion::Slash, false);
		specimen.inspect_expression();
		assert_eq!(
			specimen.face(),
			None,
			"a strike has no expression to inspect"
		);
		specimen.select_motion(ReaperMotion::Laugh, false);
		specimen.toggle_spin();
		specimen.inspect_expression();
		assert_eq!(specimen.face(), Some(("joy", 1.0)));
		assert!(specimen.focus_face);
		assert!(!specimen.spinning);
		assert!(!specimen.clock.playing());
		assert!(specimen.show().focus_face);
		specimen.toggle_face();
		assert!(!specimen.focus_face);
		assert_eq!(specimen.face(), None, "the full figure lets the face go");
	}

	#[test]
	fn the_gallery_steps_through_every_calling_in_every_outfit() {
		let mut specimen = Specimen::gallery(saved());
		let mut seen = Vec::new();
		for _ in 0..GALLERY.len() {
			seen.push((specimen.genome.reaper_class, specimen.outfit));
			specimen.step_gallery(1);
		}
		assert_eq!(specimen.gallery, Some(0), "it wraps round");
		for calling in ReaperClass::ALL {
			for outfit in Outfit::ALL {
				assert!(seen.contains(&(calling, outfit)), "{calling:?} {outfit:?}");
			}
		}
		specimen.step_gallery(-1);
		assert_eq!(specimen.gallery, Some(GALLERY.len() - 1));
		assert_eq!(specimen.genome.reaper_class, ReaperClass::Mourner);
	}
}
