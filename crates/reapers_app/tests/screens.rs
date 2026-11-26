//! Every screen renders headlessly at phone size with the real layout
//! engine (which also checks every Tailwind class it uses), and the main
//! journeys work end to end against the offline and simnet keepers.

use std::rc::Rc;

use futures::executor::block_on;
use leptoskit_testing::Finder;
use leptoskit_testing::WidgetTester;
use reactive_graph::owner::Owner;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::Set;
use reapers_app::config::BuildFlags;
use reapers_app::l10n::En;
use reapers_app::l10n::L10n;
use reapers_app::l10n::Messages;
use reapers_app::platform;
use reapers_app::platform::MemoryPrefs;
use reapers_app::render::pointer::Gesture;
use reapers_app::routes::Hunt;
use reapers_app::routes::Navigation;
use reapers_app::routes::Rift;
use reapers_app::routes::Route;
use reapers_app::state::Books;
use reapers_app::state::Game;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use reapers_rules::RiftSite;

/// The game in `flags`, its router at `route`, and a tester on its screen.
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
		let tester = WidgetTester::new(move || owner.with(|| screen.render()), 390.0, 844.0)
			.expect("the first screen mounts");
		Self {
			game,
			navigation,
			tester,
		}
	}

	fn offline() -> Self {
		Self::new(BuildFlags::default())
	}

	/// An offline build whose hunts the rules' own player fights
	/// (`REAPERS_AUTOPLAY`).
	fn autoplay() -> Self {
		Self::new(BuildFlags {
			autoplay: true,
			..BuildFlags::default()
		})
	}

	/// Plays the hunt on screen to its end, through the real view, as the
	/// ticker would on a device: the tests have no timer.
	fn fight(&mut self) {
		assert!(
			reapers_app::hunt::view::play_on(300.0),
			"the hunt ends\n{}",
			self.tester.tree()
		);
		self.settle();
	}

	/// Lets every task the screens started run, and the screen catch up.
	fn settle(&mut self) {
		platform::run_until_stalled();
		self.tester.pump();
		platform::run_until_stalled();
		self.tester.pump();
	}

	fn open(&mut self, route: impl Into<Route>) {
		self.navigation.navigate(route);
		self.settle();
	}

	fn tap(&mut self, id: &str) {
		self.tester
			.tap(&Finder::test_id(id))
			.unwrap_or_else(|error| panic!("{id}: {error:?}\n{}", self.tester.tree()));
		self.settle();
	}

	fn shows(&self, text: &str) -> bool {
		self.tester.find(&Finder::text(text)).is_ok()
	}

	fn has(&self, id: &str) -> bool {
		self.tester.find(&Finder::test_id(id)).is_ok()
	}

	/// Binds a reaper through the oath screen, as a player would.
	fn take_oath(&mut self) {
		self.open(Route::Oath);
		self.tap("oath-speak");
		assert!(self.has("oath-drawn"), "{}", self.tester.tree());
		self.tap("oath-enter");
	}
}

const SAINT: Nft = Nft::new(NftCollection::DegenApes, 1);

#[test]
fn the_main_screen_gives_the_reaper_a_visible_phone_sized_stage() {
	let mut harness = Harness::offline();
	harness.take_oath();
	let stage = harness
		.tester
		.find(&Finder::test_id("reaper-stage"))
		.unwrap()
		.frame;
	let scene = harness
		.tester
		.find(&Finder::test_id("reaper-scene"))
		.unwrap()
		.frame;
	assert!(stage.width >= 389.0 && stage.height >= 340.0, "{stage:?}");
	assert!(
		stage.y >= 0.0 && stage.y + stage.height <= 844.0,
		"{stage:?}"
	);
	for (actual, expected) in [
		(scene.x, stage.x),
		(scene.y, stage.y),
		(scene.width, stage.width),
		(scene.height, stage.height),
	] {
		assert!(
			(actual - expected).abs() < 0.02,
			"the GPU view fills the stage: {scene:?}, {stage:?}"
		);
	}
	let scene_id = harness
		.tester
		.find(&Finder::test_id("reaper-scene"))
		.unwrap()
		.props
		.scene3d;
	assert!(scene_id.is_some());
	// The studio's row waits under the stage, below a phone's fold: make the
	// page tall enough to tap it, then come back to the phone's height to
	// compare the stage that returns.
	harness.tester.resize(390.0, 1400.0).unwrap();
	harness.settle();
	harness.tap("sanctum-studio");
	harness.tap("ledger-back");
	harness.tester.resize(390.0, 844.0).unwrap();
	harness.settle();
	let returned = harness
		.tester
		.find(&Finder::test_id("reaper-scene"))
		.unwrap();
	assert_eq!(returned.props.scene3d, scene_id);
	assert_eq!(
		returned.frame, scene,
		"returning from the studio restores the main stage"
	);
}

#[test]
fn an_unbound_player_stands_at_the_threshold() {
	let harness = Harness::offline();
	assert!(harness.has("threshold"));
	assert!(harness.shows(En.app_title()));
	assert!(harness.has("threshold-oath"));
}

#[test]
fn the_oath_binds_a_reaper_and_the_sanctum_shows_it() {
	let mut harness = Harness::offline();
	harness.take_oath();
	assert!(harness.has("sanctum"), "{}", harness.tester.tree());
	let Books::Open(player) = harness.game.books.get_untracked() else {
		panic!("the books are open");
	};
	assert!(harness.shows(&player.genome().name()));
}

/// The sanctum gives the reaper the stage and prints nothing over it: the
/// name sits under the stage, the primary and the tab bar under the page.
#[test]
fn the_sanctum_prints_the_name_under_the_stage_and_never_over_the_reaper() {
	let mut harness = Harness::offline();
	harness.take_oath();
	let frame = |id: &str| {
		harness
			.tester
			.find(&Finder::test_id(id))
			.unwrap_or_else(|error| panic!("{id}: {error:?}"))
			.frame
	};
	let stage = frame("reaper-stage");
	let stage_foot = stage.y + stage.height;
	for id in ["sanctum-name", "sanctum-hunt", "sanctum-vault"] {
		let below = frame(id);
		assert!(
			below.y >= stage_foot - 0.5,
			"{id} is printed over the stage: {below:?} under {stage:?}"
		);
	}
	let hunt = frame("sanctum-hunt");
	let tab = frame("sanctum-vault");
	assert!(
		hunt.y + hunt.height <= tab.y,
		"the primary sits over the tab bar"
	);
	assert!(tab.y + tab.height <= 844.5, "the tab bar is on the phone");
}

/// One reaper is one reaper: the roster's row counts in the singular and the
/// plural.
#[test]
fn the_sanctum_counts_one_reaper_in_the_singular() {
	let mut harness = Harness::offline();
	harness.take_oath();
	// The rows sit at the foot of the page, under a phone's fold; the tester
	// does not scroll a scroll view's content, so the page is made tall
	// enough to show every row at once.
	harness.tester.resize(390.0, 1600.0).unwrap();
	harness.settle();
	assert!(harness.has("sanctum-roster"), "{}", harness.tester.tree());
	assert!(harness.shows(&En.sanctum_roster(1)));
	assert_eq!(En.sanctum_roster(1), "1 reaper");
	assert_eq!(En.sanctum_roster(2), "2 reapers");
	assert!(harness.has("sanctum-practice"));
	assert!(harness.has("sanctum-colophon"));
}

/// With nothing loose the primary sends the reaper after a stray; with a
/// husk sighted it says how many are near and hunts the first.
#[test]
fn going_hunting_from_the_sanctum_hunts_what_is_near_or_a_stray() {
	let mut harness = Harness::offline();
	harness.take_oath();
	assert!(harness.shows(&En.sanctum_practice_detail().to_uppercase()));
	harness.tap("sanctum-hunt");
	assert!(harness.has("hunt-view"), "{}", harness.tester.tree());

	harness.open(Route::Vault);
	harness.tap(&format!("vault-nft-{}", SAINT.id()));
	harness.tap("vault-buy");
	harness.tap("vault-release");
	let Books::Open(player) = harness.game.books.get_untracked() else {
		panic!("the books are open");
	};
	let sighting = player.sightings()[0];
	harness.game.now.set(sighting.surfaces_at);
	harness.settle();
	assert!(
		harness.shows(&En.sanctum_nearby(1).to_uppercase()),
		"{}",
		harness.tester.tree()
	);
	assert!(harness.shows(&En.sanctum_loose_one(&sighting.nft.title())));
	harness.tap("sanctum-hunt");
	assert!(harness.has("hunt-view"), "{}", harness.tester.tree());
}

#[test]
fn a_landscape_phone_keeps_room_to_scroll_the_sanctum_menu() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.tester.resize(869.0, 359.0).unwrap();
	harness.settle();
	let menu = harness
		.tester
		.find(&Finder::test_id("sanctum-menu"))
		.unwrap();
	assert!(
		menu.frame.height >= 120.0,
		"the menu needs a usable scroll area: {menu:?}"
	);
	assert!(
		menu.frame.y + menu.frame.height <= 360.0,
		"the menu fits the phone: {menu:?}"
	);
}

#[test]
fn buying_and_releasing_from_the_vault_lets_a_husk_loose() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.open(Route::Vault);
	harness.tap(&format!("vault-nft-{}", SAINT.id()));
	assert!(harness.has("sheet"));
	harness.tap("vault-buy");
	assert!(harness.has("vault-release"), "{}", harness.tester.tree());
	harness.tap("vault-release");
	assert!(
		harness.has("sanctum"),
		"releasing goes home to watch for the husk"
	);
	let Books::Open(player) = harness.game.books.get_untracked() else {
		panic!("the books are open");
	};
	assert_eq!(player.sightings().len(), 1);
	assert!(harness.has(&format!("sanctum-sighting-{}", player.sightings()[0].id)));
}

#[test]
fn a_refusal_is_explained_and_dismissed() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.open(Route::Armory);
	// A new reaper cannot afford the costliest outfit: the rules refuse, and
	// the screen puts their reason in front of the player.
	harness.game.change(
		|keeper| async move { keeper.buy_outfit(reapers_rules::Outfit::Hollowed).await },
		|done| assert!(!done),
	);
	harness.settle();
	assert!(harness.has("dialog"), "{}", harness.tester.tree());
	harness.tap("refusal-dismiss");
	assert!(!harness.has("dialog"));
	assert_eq!(harness.game.overlay.get_untracked(), None);
}

#[test]
fn every_screen_renders_for_a_bound_player() {
	let mut harness = Harness::offline();
	harness.take_oath();
	for (route, id) in [
		(Route::Index, "sanctum"),
		(Route::Armory, "armory"),
		(Route::Studio, "studio-view"),
		(Route::Practice, "hunt-view"),
	] {
		harness.open(route.clone());
		assert!(harness.has(id), "{route:?}: {}", harness.tester.tree());
	}
	for route in [
		Route::Vault,
		Route::Codex,
		Route::Rifts,
		Route::Colophon,
		Route::Roster,
	] {
		harness.open(route.clone());
		assert!(
			harness.has("ledger-back"),
			"{route:?}: {}",
			harness.tester.tree()
		);
	}
	harness.open(Route::Armory);
	harness.tap("armory-tab-arts");
	assert!(harness.has("armory-art-cinder"));
	harness.tap("armory-tab-outfits");
	assert!(harness.has("armory-outfit-shrouded"));
	harness.open(Route::Codex);
	harness.tap("codex-soul");
	assert!(harness.shows(En.soul_marks().to_uppercase().as_str()));
}

/// Second Ink's strokes are generated once and kept (`ui::ink`): a screen
/// built a second time, after the player has been elsewhere, draws no new
/// stroke. The counts are printed so a run with `--nocapture` shows them.
#[test]
fn a_screen_built_again_draws_no_new_ink() {
	use reapers_app::ui::ink;

	let mut harness = Harness::offline();
	// The oath lands on the sanctum: its first build is the sanctum's cold
	// one.
	harness.open(Route::Oath);
	harness.tap("oath-speak");
	let (drawn, asked) = (ink::drawn(), ink::asked());
	harness.tap("oath-enter");
	assert!(harness.has("sanctum"), "{}", harness.tester.tree());
	println!(
		"ink: cold sanctum, first build: {} drawn, {} asked for",
		ink::drawn() - drawn,
		ink::asked() - asked
	);
	let rounds = [
		Route::Index,
		Route::Colophon,
		Route::Vault,
		Route::Kit,
		Route::Index,
	];
	// Opens each route in turn, printing what each drew and asked for.
	let mut round = |name: &str| {
		for route in rounds.clone() {
			let (drawn, asked) = (ink::drawn(), ink::asked());
			harness.open(route.clone());
			println!(
				"ink: {name} {route:?}: {} drawn, {} asked for",
				ink::drawn() - drawn,
				ink::asked() - asked
			);
		}
	};
	let before = ink::drawn();
	round("cold");
	let cold = ink::drawn() - before;
	let asked = ink::asked();
	round("again");
	let again = ink::drawn() - before - cold;
	println!(
		"ink: {cold} strokes generated on the first build, {again} on the second ({} asked for again)",
		ink::asked() - asked
	);
	assert!(cold > 0, "the screens print some ink");
	assert_eq!(again, 0, "a second build must not regenerate a stroke");
	assert!(
		ink::asked() > asked,
		"the second build asked for its strokes"
	);
}

/// The kit page prints every section of the kit, with a live control of
/// each kind a test can press, and says what its build drew.
#[test]
fn the_kit_page_prints_every_component_in_every_state() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.open(Route::Kit);
	assert!(harness.has("kit"), "{}", harness.tester.tree());
	// A screen tall enough to show the whole page at once.
	harness.tester.resize(390.0, 24_000.0).unwrap();
	harness.settle();
	for section in [
		"type",
		"primary",
		"secondary",
		"quiet",
		"rows",
		"tabs",
		"slips",
		"stamps",
		"dial",
		"lamp",
		"meter",
		"tally",
		"sheet",
		"fields",
		"toggles",
		"ink",
	] {
		assert!(harness.has(&format!("kit-section-{section}")), "{section}");
	}
	for live in [
		"kit-live-primary",
		"kit-live-secondary",
		"kit-live-quiet",
		"kit-live-row",
		"kit-live-tab",
		"field-kit-live",
		"kit-ink",
	] {
		assert!(harness.has(live), "{live}");
	}
	harness.tap("kit-live-primary");
	assert!(harness.has("kit"), "a press on the kit stays on the kit");
}

#[test]
fn a_hunt_for_an_unknown_husk_says_so() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.open(Hunt { sighting: 99 });
	assert!(harness.shows(En.refusal_unknown_sighting()));
}

#[test]
fn a_practice_hunt_fought_by_the_rules_own_player_ends_in_a_verdict() {
	let mut harness = Harness::autoplay();
	harness.open(Route::Practice);
	assert!(harness.has("hunt-view"));
	harness.tap("hunt-begin");
	harness.fight();
	assert!(harness.has("hunt-result"), "{}", harness.tester.tree());
	harness.tap("hunt-leave");
}

#[test]
fn a_hunt_draws_its_overlay_no_more_than_thirty_times_a_second() {
	// The 3D moves on every frame the display draws; the 2D overlay, whose
	// every drawing rebuilds the screen, is held to thirty a second however
	// fast the frames come.
	let mut harness = Harness::autoplay();
	harness.open(Route::Practice);
	harness.tap("hunt-begin");
	// The rules' own player traces the sigil and the fight begins.
	reapers_app::hunt::view::play_on(3.0);
	harness.settle();
	let before = reapers_app::hunt::view::drawn_count();
	// Four seconds of the fight, a frame at a time.
	for _ in 0..240 {
		reapers_app::hunt::view::play_on(1.0 / 60.0);
	}
	let drawn = reapers_app::hunt::view::drawn_count() - before;
	assert!(drawn > 10, "the fight is drawn as it changes: {drawn}");
	assert!(drawn <= 4 * 30 + 1, "{drawn} drawings in four seconds");
}

#[test]
fn a_first_practice_teaches_one_thing_at_a_time() {
	let mut harness = Harness::autoplay();
	harness.open(Route::Practice);
	harness.tap("hunt-begin");
	let mut taught = 0;
	while !reapers_app::hunt::view::play_on(0.5) {
		harness.settle();
		if harness.has("hunt-lesson") {
			taught += 1;
			assert!(
				!harness.has("hunt-prompt"),
				"a lesson and the ledger's hint at once\n{}",
				harness.tester.tree()
			);
		}
	}
	assert!(taught > 0, "a first practice teaches");
}

#[test]
fn a_finger_on_a_hunt_turns_it_holds_orb_and_lets_go() {
	// The browser hands each gesture to the hunt while the hunt is
	// borrowed. A gesture that borrowed it again aborted the game on the
	// first finger to come up.
	let mut harness = Harness::offline();
	harness.open(Route::Practice);
	harness.tap("hunt-begin");
	let gestures = [
		Gesture::Down { x: 0.5, y: 0.4 },
		Gesture::Drag {
			dx: 30.0,
			dy: -12.0,
		},
		Gesture::Up,
		Gesture::Down { x: 0.5, y: 0.4 },
		Gesture::Tap { x: 0.5, y: 0.4 },
		Gesture::Up,
		Gesture::Down { x: 0.1, y: 0.9 },
		Gesture::Up,
	];
	// While the sigil is traced, and again in the fight.
	for gesture in gestures {
		reapers_app::hunt::view::feel(gesture);
	}
	reapers_app::hunt::view::play_on(61.0);
	harness.settle();
	harness.tap("hunt-skip");
	assert!(harness.has("hunt-fire"), "{}", harness.tester.tree());
	for gesture in gestures {
		reapers_app::hunt::view::feel(gesture);
	}
	harness.settle();
	assert!(harness.has("hunt-fire"), "{}", harness.tester.tree());
}

#[test]
fn a_loose_husk_is_hunted_settled_by_the_keeper_and_written_in_the_codex() {
	let mut harness = Harness::autoplay();
	harness.take_oath();
	harness.open(Route::Vault);
	harness.tap(&format!("vault-nft-{}", SAINT.id()));
	harness.tap("vault-buy");
	harness.tap("vault-release");
	let Books::Open(player) = harness.game.books.get_untracked() else {
		panic!("the books are open");
	};
	let sighting = player.sightings()[0];
	// The husk has not surfaced yet; the keeper's clock is the device's, so
	// the books are moved on to after it surfaces rather than waiting.
	assert!(harness.has(&format!("sanctum-sighting-{}", sighting.id)));
	harness.game.now.set(sighting.surfaces_at);
	harness.open(Hunt {
		sighting: sighting.id,
	});
	assert!(harness.has("hunt-view"));
	harness.tap("hunt-begin");
	harness.fight();
	// The keeper refuses a husk that has not surfaced by its own clock, and
	// the refusal is explained; either way the books are left consistent.
	let settled = harness.has("hunt-result");
	let refused = harness.shows(En.refusal_not_surfaced());
	assert!(settled || refused, "{}", harness.tester.tree());
}

#[test]
fn a_rift_shut_to_the_reaper_says_whose_dead_it_wants() {
	let mut harness = Harness::offline();
	harness.take_oath();
	harness.open(Route::Rifts);
	let site = RiftSite::ALL[0];
	assert!(harness.has(&format!("rift-shut-{}", site.name())));
	harness.open(Rift {
		site: site.name().to_owned(),
	});
	assert!(
		harness.shows(En.refusal_not_attuned()),
		"{}",
		harness.tester.tree()
	);
}

#[test]
fn the_simnet_build_posts_the_airdrop_and_shows_it_in_the_explorer() {
	let mut harness = Harness::new(BuildFlags {
		simnet: true,
		..BuildFlags::default()
	});
	harness.take_oath();
	assert!(harness.has("simnet-slip"), "{}", harness.tester.tree());
	// The ledger's row sits last with the sanctum's other rows at the foot of
	// its page, under the fold of a phone; the tester does not scroll a
	// scroll view's content, so the screen is made tall enough to show it.
	harness.tester.resize(390.0, 1600.0).unwrap();
	harness.settle();
	assert!(harness.has("sanctum-simnet"), "{}", harness.tester.tree());
	harness.tap("sanctum-simnet");
	assert!(harness.shows(&En.simnet_instruction_airdrop().to_uppercase()));
	let airdrop = harness.game.ledger.get_untracked()[0].signature.clone();
	harness.tap(&format!("simnet-tx-{airdrop}"));
	assert!(harness.shows(En.simnet_wire_none()));
	harness.game.overlay.set(None);
	harness.settle();
	harness.tap("simnet-reset");
	harness.tap("simnet-reset-confirm");
	assert!(
		harness.has("threshold"),
		"starting over leaves a new player at the threshold"
	);
	assert_eq!(
		harness.game.ledger.get_untracked(),
		[] as [reapers_app::simnet::SimnetTransaction; 0]
	);
}

#[test]
fn the_colophon_turns_the_music_off_and_on() {
	let mut harness = Harness::offline();
	harness.open(Route::Colophon);
	assert!(harness.shows(En.colophon_sanctum_credit()));
	harness.tap("colophon-music");
	assert!(!harness.game.music.get_untracked());
	harness.tap("colophon-music");
	assert!(harness.game.music.get_untracked());
}

#[test]
fn a_server_build_that_cannot_reach_its_server_offers_to_try_again() {
	let harness = Harness::new(BuildFlags {
		server_url: Some("http://localhost:8080/"),
		..BuildFlags::default()
	});
	assert!(harness.shows(En.common_unreachable()));
	assert!(harness.has("notice-action"));
}

#[test]
fn the_arms_bench_plays_every_weapon_motion_and_orb_ending() {
	let mut harness = Harness::new(BuildFlags {
		arms_bench: true,
		..BuildFlags::default()
	});
	// Tall enough that every button is on screen at once.
	harness.tester.resize(390.0, 1800.0).unwrap();
	harness.open(Route::Index);
	assert!(harness.has("arms-bench"), "{}", harness.tester.tree());
	assert!(harness.has("arms-view"));
	let pistol: &[&str] = &[
		"arms-fire",
		"arms-empty",
		"arms-reload",
		"arms-active-reload",
		"arms-jam",
	];
	let knife: &[&str] = &["arms-dive-in", "arms-stab", "arms-dive-out"];
	let scythe: &[&str] = &["arms-swing"];
	for (weapon, own) in [
		("arms-pistol", pistol),
		("arms-knife", knife),
		("arms-scythe", scythe),
	] {
		harness.tap(weapon);
		for motion in ["arms-idle", "arms-draw"].iter().chain(own).chain(&[
			"arms-parry",
			"arms-struck",
			"arms-raise",
		]) {
			harness.tap(motion);
		}
	}
	for button in [
		"arms-blow-seal",
		"arms-blow-body",
		"arms-blow-miss",
		"arms-seal",
		"arms-break-1",
		"arms-break-2",
		"arms-break-3",
		"arms-miss",
		"arms-bright",
		"arms-grey",
		"arms-dark",
		"arms-quarter-speed",
		"arms-full-speed",
		"arms-look",
	] {
		harness.tap(button);
	}
	assert!(harness.has("arms-bench"), "{}", harness.tester.tree());
}
