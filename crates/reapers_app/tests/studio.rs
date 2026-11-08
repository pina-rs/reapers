//! The reaper studio at phone size, through its controls: each changes the
//! stage, and none changes the player's books. The intent of the Flutter
//! `test/studio/`.

use std::rc::Rc;

use futures::executor::block_on;
use leptoskit_testing::Finder;
use leptoskit_testing::WidgetTester;
use reactive_graph::owner::Owner;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::Set;
use reapers_app::config::BuildFlags;
use reapers_app::l10n::L10n;
use reapers_app::platform;
use reapers_app::platform::MemoryPrefs;
use reapers_app::render::appearance::ReaperClothing;
use reapers_app::render::appearance::ReaperForm;
use reapers_app::render::scene::STUDIO;
use reapers_app::render::stage::StageShow;
use reapers_app::render::stage::stage;
use reapers_app::routes::Navigation;
use reapers_app::routes::Route;
use reapers_app::state::Books;
use reapers_app::state::Game;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGift;
use reapers_rules::ReaperMotion;
use reapers_rules::ReaperSoul;
use reapers_rules::Temper;
use reapers_rules::WeaponKind;

struct Harness {
	game: Game,
	navigation: Navigation,
	tester: WidgetTester,
}

impl Harness {
	fn new(flags: BuildFlags) -> Self {
		let owner = Owner::new();
		let navigation = Navigation::new(Route::Index);
		let game = owner.with(|| {
			L10n::new(&[]).provide();
			reapers_app::screens::install_navigation(navigation.clone());
			provide_context(navigation.clone());
			let game = Game::new(flags, Rc::new(MemoryPrefs::new()));
			provide_context(game);
			block_on(game.open());
			game.arrived.set(true);
			game
		});
		let screen = navigation.clone();
		// Phone width, and tall enough that the whole studio is on screen:
		// the tester does not scroll a scroll view's content.
		let tester = WidgetTester::new(move || owner.with(|| screen.render()), 390.0, 16000.0)
			.expect("the first screen mounts");
		Self {
			game,
			navigation,
			tester,
		}
	}

	/// A bound player, in the studio.
	fn studio() -> Self {
		let mut harness = Self::new(BuildFlags::default());
		harness.navigation.navigate(Route::Oath);
		harness.settle();
		harness.tap("oath-speak");
		harness.tap("oath-enter");
		harness.navigation.navigate(Route::Studio);
		harness.settle();
		assert!(harness.has("studio-view"), "{}", harness.tester.tree());
		harness
	}

	fn settle(&mut self) {
		platform::run_until_stalled();
		self.tester.pump();
		platform::run_until_stalled();
		self.tester.pump();
	}

	fn tap(&mut self, id: &str) {
		self.tester
			.tap(&Finder::test_id(id))
			.unwrap_or_else(|error| panic!("{id}: {error:?}\n{}", self.tester.tree()));
		self.settle();
	}

	fn toggle(&mut self, id: &str, checked: bool) {
		self.tester
			.toggle(&Finder::test_id(id), checked)
			.unwrap_or_else(|error| panic!("{id}: {error:?}\n{}", self.tester.tree()));
		self.settle();
	}

	fn slide(&mut self, id: &str, value: f64) {
		self.tester
			.set_slider_value(&Finder::test_id(id), value)
			.unwrap_or_else(|error| panic!("{id}: {error:?}\n{}", self.tester.tree()));
		self.settle();
	}

	fn shows(&self, text: &str) -> bool {
		self.tester.find(&Finder::text(text)).is_ok()
	}

	fn has(&self, id: &str) -> bool {
		self.tester.find(&Finder::test_id(id)).is_ok()
	}

	fn player(&self) -> PlayerState {
		let Books::Open(player) = self.game.books.get_untracked() else {
			panic!("the books are open");
		};
		player
	}
}

/// What the studio's stage shows.
fn shown() -> StageShow {
	stage(STUDIO)
		.borrow()
		.shown()
		.expect("the stage shows the specimen")
}

#[test]
fn customization_and_progression_change_the_stage_and_never_the_books() {
	let mut harness = Harness::studio();
	let before = serde_json::to_string(&harness.player()).unwrap();
	let genome = harness.player().genome();
	assert_eq!(shown().genome, genome, "it opens on the player's reaper");
	assert!(harness.shows(&genome.name()));

	harness.tap("studio-class-chooser");
	assert_eq!(shown().genome.reaper_class, ReaperClass::Chooser);
	harness.tap("studio-form-humanoid");
	assert_eq!(shown().appearance.form, ReaperForm::Humanoid);
	assert_eq!(shown().appearance.clothing, ReaperClothing::Robe);
	harness.tap("studio-clothing-unrobed");
	assert_eq!(shown().appearance.clothing, ReaperClothing::Unrobed);
	harness.tap("studio-held-scythe");
	assert_eq!(shown().weapon.kind, WeaponKind::Scythe);
	harness.toggle("studio-holding", false);
	assert!(!shown().appearance.holding);
	assert!(
		!harness.has("studio-held-scythe"),
		"no held object to choose"
	);
	harness.toggle("studio-mantle", false);
	assert!(!shown().appearance.mantle);
	harness.tap("studio-head-1");
	assert_eq!(shown().genome.head, 1);
	harness.tap("studio-omen-gilded");
	assert_eq!(shown().genome.omen, reapers_rules::Omen::Gilded);
	harness.slide("studio-trait-stature", 1000.0);
	assert_eq!(shown().genome.stature, 1000);
	harness.tap("studio-copy");
	assert!(harness.has("studio-code"), "the code is shown to copy");

	harness.tap("studio-tab-growth");
	harness.tap("studio-absorb");
	assert!(harness.has("studio-reward"));
	assert!(harness.shows("1 absorbed"));
	assert!(
		harness
			.tester
			.find(&Finder::test_id("studio-temper-hollowed"))
			.is_ok()
	);
	harness.tap("studio-potential");
	assert_eq!(shown().weapon.temper, Temper::Hollowed);
	assert_eq!(shown().outfit, Outfit::Hollowed);
	assert!(harness.shows("30 / 30"));
	harness.slide("studio-level", 4.0);
	assert_eq!(
		shown().weapon.temper,
		Temper::Shrouded,
		"level 4 gives it up"
	);
	harness.tap("studio-temper-sworn");
	assert_eq!(shown().weapon.temper, Temper::Sworn);
	harness.tap("studio-potential");

	harness.tap("studio-tab-soul");
	harness.tap("studio-gift-reprisal");
	assert_eq!(
		ReaperSoul::from_genome(&shown().genome).gift,
		ReaperGift::Reprisal
	);
	assert!(harness.shows("Awakening 3 / 3"));
	harness.tap("studio-memory-reset");
	assert!(harness.shows("Awakening 0 / 3"));
	harness.tap("studio-memory-stillness");
	assert!(harness.shows("Awakening 0 / 3"));

	assert_eq!(
		serde_json::to_string(&harness.player()).unwrap(),
		before,
		"the books are untouched"
	);
}

#[test]
fn the_animations_play_pause_replay_and_hold_a_face() {
	let mut harness = Harness::studio();
	harness.tap("studio-motions");
	harness.tap("motion-dashLeft");
	let clock = || stage(STUDIO).borrow().playback;
	assert_eq!(clock().motion(), ReaperMotion::DashLeft);
	assert!(clock().playing());
	harness.tap("motion-dashRight");
	assert_eq!(clock().motion(), ReaperMotion::DashRight);

	harness.tap("motion-group-finisher");
	harness.tap("studio-held-bell");
	// A disabled chip refuses the tap, or ignores it.
	let _ = harness
		.tester
		.tap(&Finder::test_id("motion-finisherImpale"));
	harness.settle();
	assert_ne!(
		clock().motion(),
		ReaperMotion::FinisherImpale,
		"impale needs a spear"
	);
	harness.tap("studio-held-spear");
	harness.tap("motion-finisherImpale");
	assert_eq!(clock().motion(), ReaperMotion::FinisherImpale);
	assert!(harness.has("motion-current"));
	harness.tap("motion-play");
	assert!(!clock().playing(), "paused");
	harness.tap("motion-play");
	assert!(clock().playing());
	assert_eq!(clock().seconds(), 0.0);
	harness.tap("motion-speed-0.25");
	assert_eq!(clock().speed(), 0.25);
	harness.tap("motion-replay");
	assert!(clock().playing());
	harness.tap("studio-held-bell");
	assert_eq!(clock().motion(), ReaperMotion::Idle, "a bell cannot impale");

	harness.tap("motion-group-reaction");
	harness.tap("motion-laugh");
	harness.tap("motion-inspect");
	assert_eq!(stage(STUDIO).borrow().held_face(), Some(("joy", 1.0)));
	assert!(shown().focus_face);
	assert!(harness.shows("FULL FIGURE"));
	assert!(harness.shows("PLAY ANIMATION"));
	harness.tap("studio-face");
	assert!(!shown().focus_face);
	assert_eq!(stage(STUDIO).borrow().held_face(), None);
}

#[test]
fn the_book_of_becoming_exercises_the_rules_it_explains() {
	let mut harness = Harness::studio();
	harness.tap("studio-book-open");
	assert!(harness.has("studio-book"));
	harness.tap("ledger-calling-ferryman");
	harness.tap("ledger-gift-anchor");
	harness.slide("ledger-level", 30.0);
	assert!(harness.shows("Awakening 0 / 3"));
	harness.tap("ledger-marks-30");
	assert!(harness.shows("Awakening 3 / 3"));

	harness.tap("ledger-chapter-1");
	assert!(harness.shows("25% · ×0.85 XP · 43 XP"));
	assert!(harness.shows("5% · ×2.0 XP · 100 XP"));
	harness.tap("ledger-draw-birth");
	assert!(harness.has("ledger-birth"));
	harness.tap("ledger-roll-feeding");
	assert!(harness.has("ledger-rolled"));
	harness.tap("ledger-rarity-rare");
	assert!(
		!harness.has("ledger-rolled"),
		"a new scenario forgets its roll"
	);
	assert!(harness.shows("25% · ×0.85 XP · 128 XP"));
	assert!(harness.shows(
		"Expected XP for this scenario: 159.60. Anima: 60. Expectation uses each rounded reward and its weight."
	));
	for chapter in ["ledger-chapter-2", "ledger-chapter-3"] {
		harness.tap(chapter);
	}
	harness.tap("book-back");
	assert!(harness.has("studio-view"));
	assert!(!harness.has("studio-book"));
}

#[test]
fn leaving_the_studio_forgets_the_specimen() {
	let mut harness = Harness::studio();
	harness.tap("studio-class-mourner");
	assert_eq!(shown().genome.reaper_class, ReaperClass::Mourner);
	harness.tap("ledger-back");
	assert!(reapers_app::studio::specimen().get_untracked().is_none());
	harness.navigation.navigate(Route::Studio);
	harness.settle();
	assert_eq!(shown().genome, harness.player().genome());
}

#[test]
fn the_gallery_bench_steps_through_every_reaper_and_outfit() {
	let mut harness = Harness::new(BuildFlags {
		gallery: true,
		..BuildFlags::default()
	});
	assert!(harness.has("studio-gallery"), "{}", harness.tester.tree());
	assert_eq!(shown().genome.reaper_class, ReaperClass::Harvester);
	assert_eq!(shown().outfit, Outfit::Shrouded);
	assert!(harness.shows("Harvester · Shrouded · 1 of 24"));
	harness.tap("gallery-next");
	assert_eq!(shown().outfit, Outfit::Sworn);
	harness.tap("gallery-previous");
	harness.tap("gallery-previous");
	assert_eq!(shown().genome.reaper_class, ReaperClass::Mourner);
	assert_eq!(shown().outfit, Outfit::Hollowed);
	assert!(
		!harness.has("ledger-back"),
		"a bench is home: there is no way back"
	);
}
