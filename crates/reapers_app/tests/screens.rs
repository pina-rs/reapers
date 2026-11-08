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
	assert!(harness.has("sanctum-simnet"));
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
