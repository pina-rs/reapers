//! Headless UI, 30Hz hunt-frame, and client deserialization benchmarks for `reapers_app`.
//!
//! Run with:
//! `cargo test --release -p reapers_app --test perf_bench -- --ignored --nocapture`

use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

use futures::executor::block_on;
use leptoskit_testing::WidgetTester;
use reactive_graph::owner::Owner;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::Set;
use reapers_app::config::BuildFlags;
use reapers_app::l10n::L10n;
use reapers_app::platform;
use reapers_app::platform::MemoryPrefs;
use reapers_app::routes::Hunt;
use reapers_app::routes::Navigation;
use reapers_app::routes::Route;
use reapers_app::state::Books;
use reapers_app::state::Game;
use reapers_rules::Nft;
use reapers_rules::PlayerState;
use reapers_rules::Timestamp;
use reapers_rules::market_listings;

fn median(samples: &mut [f64]) -> f64 {
	samples.sort_by(f64::total_cmp);
	samples[samples.len() / 2]
}

#[test]
#[ignore = "performance benchmark, run explicitly in --release"]
#[allow(clippy::too_many_lines, reason = "end-to-end benchmark driver")]
fn app_frame_and_screen_benchmark() {
	let owner = Owner::new();
	let navigation = Navigation::new(Route::Index);
	let game = owner.with(|| {
		L10n::new(&[]).provide();
		reapers_app::screens::install_navigation(navigation.clone());
		provide_context(navigation.clone());
		let game = Game::new(
			BuildFlags {
				autoplay: true,
				..BuildFlags::default()
			},
			Rc::new(MemoryPrefs::new()),
		);
		provide_context(game);
		block_on(game.open());
		game.arrived.set(true);
		game
	});

	let screen = navigation.clone();
	let test_owner = owner.clone();
	let mut tester = WidgetTester::new(move || test_owner.with(|| screen.render()), 390.0, 844.0)
		.expect("mount");

	// Take oath + acquire + release to enter a real hunt and populate vault
	{
		let keeper = game.keeper().expect("keeper");
		block_on(async {
			keeper.take_oath().await.expect("oath");
		});
		block_on(game.reload());
	}
	platform::run_until_stalled();
	tester.pump();

	// Measure M15: Vault screen render latency (40 NFTs)
	navigation.navigate(Route::Vault);
	platform::run_until_stalled();
	tester.pump();

	let mut vault_us = Vec::with_capacity(25);
	for _ in 0..25 {
		let t0 = Instant::now();
		let _ = black_box(owner.with(|| navigation.render()));
		vault_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}

	// Acquire and release a husk, then enter Hunt screen
	let nft = Nft::parse("degenApes:1").expect("nft");
	{
		let keeper = game.keeper().expect("keeper");
		block_on(async {
			keeper.acquire(nft).await.expect("acquire");
			keeper.release(nft).await.expect("release");
		});
		block_on(game.reload());
	}
	let sighting_id = match game.books.get_untracked() {
		Books::Open(player) => player.sightings()[0].id,
		_ => panic!("expected open books"),
	};

	navigation.navigate(Hunt {
		sighting: sighting_id,
	});
	platform::run_until_stalled();
	tester.pump();
	// Tap begin button if still at briefing
	let _ = tester.tap(&leptoskit_testing::Finder::test_id("hunt-begin"));
	platform::run_until_stalled();
	tester.pump();

	// Warm up a few combat ticks
	for _ in 0..10 {
		reapers_app::hunt::view::play_on(1.0 / 30.0);
		tester.pump();
	}

	// Measure M13 & M14: 60 ticks (2 seconds of 30Hz active combat)
	let drawn_before = reapers_app::hunt::view::drawn_count();
	let mut frame_us = Vec::with_capacity(60);
	let mut last_drawn = drawn_before;
	for _ in 0..60 {
		let t0 = Instant::now();
		reapers_app::hunt::view::play_on(1.0 / 30.0);
		let now_drawn = reapers_app::hunt::view::drawn_count();
		if now_drawn != last_drawn {
			last_drawn = now_drawn;
			tester.pump();
		}
		frame_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let rebuilds_per_sec = (last_drawn - drawn_before) as f64 / 2.0;

	// Measure M23: Client JSON response decode cost (rich PlayerState)
	let mut state = PlayerState::initial().take_oath(55).expect("oath");
	let now = Timestamp::from_millis(1_790_000_000_000);
	for &nft in market_listings().iter().take(4) {
		if let Ok(next) = state.acquire(nft) {
			state = next;
		}
	}
	let _ = now;
	let json_bytes = serde_json::to_vec(&state).expect("ser");
	let mut deser_us = Vec::with_capacity(100);
	for _ in 0..100 {
		let t0 = Instant::now();
		let parsed: PlayerState = serde_json::from_slice(&json_bytes).expect("player");
		black_box(parsed);
		deser_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}

	println!("=== reapers_app perf_bench (release) ===");
	println!(
		"M13_hunt_frame_us:         {:.1} us/frame (median over 60 active ticks)",
		median(&mut frame_us)
	);
	println!(
		"M14_hunt_rebuilds_per_sec: {rebuilds_per_sec:.1} full-tree rebuilds/sec (active 30Hz)"
	);
	println!(
		"M15_vault_render_us:       {:.1} us/render (40 NFTs)",
		median(&mut vault_us)
	);
	println!(
		"M23_client_api_deser_us:   {:.1} us (direct from_slice->PlayerState)",
		median(&mut deser_us)
	);
}

#[test]
#[ignore = "performance benchmark, run explicitly in --release"]
fn app_steady_aim_benchmark() {
	let nav = Navigation::new(Route::Index);
	let owner = Owner::new();
	let game = owner.with(|| {
		L10n::new(&[]).provide();
		reapers_app::screens::install_navigation(nav.clone());
		provide_context(nav.clone());
		let game = Game::new(BuildFlags::default(), Rc::new(MemoryPrefs::new()));
		provide_context(game);
		block_on(game.open());
		game.arrived.set(true);
		game
	});
	let screen = nav.clone();
	let owner_render = owner.clone();
	let mut tester = WidgetTester::new(move || owner_render.with(|| screen.render()), 390.0, 844.0)
		.expect("mount steady");
	let nft = Nft::parse("degenApes:1").expect("nft");
	{
		let keeper = game.keeper().expect("keeper");
		block_on(async {
			keeper.take_oath().await.expect("oath");
			keeper.acquire(nft).await.expect("acquire");
			keeper.release(nft).await.expect("release");
		});
		block_on(game.reload());
	}
	let sighting = match game.books.get_untracked() {
		Books::Open(player) => player.sightings()[0].id,
		_ => panic!("expected open books"),
	};
	nav.navigate(Hunt { sighting });
	platform::run_until_stalled();
	tester.pump();
	let _ = tester.tap(&leptoskit_testing::Finder::test_id("hunt-begin"));
	platform::run_until_stalled();
	tester.pump();
	for _ in 0..10 {
		reapers_app::hunt::view::play_on(1.0 / 30.0);
		tester.pump();
	}
	let drawn_before = reapers_app::hunt::view::drawn_count();
	let mut last_drawn = drawn_before;
	let mut frame_us = Vec::with_capacity(60);
	for _ in 0..60 {
		let t0 = Instant::now();
		reapers_app::hunt::view::play_on(1.0 / 30.0);
		let now_drawn = reapers_app::hunt::view::drawn_count();
		if now_drawn != last_drawn {
			last_drawn = now_drawn;
			tester.pump();
		}
		frame_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let steady_rebuilds_per_sec = (last_drawn - drawn_before) as f64 / 2.0;
	println!("=== reapers_app steady-aim perf_bench (release) ===");
	println!(
		"M13b_hunt_steady_frame_us: {:.1} us/frame (median over 60 steady-aim ticks)",
		median(&mut frame_us)
	);
	println!(
		"M14b_hunt_steady_rebuilds: {steady_rebuilds_per_sec:.1} full-tree rebuilds/sec (steady aim)"
	);
}
