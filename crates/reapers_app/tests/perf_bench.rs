//! Headless UI, 30Hz hunt-frame, and client deserialization benchmarks for `reapers_app`.
//!
//! Run with:
//! `cargo test --release -p reapers_app --test perf_bench -- --ignored --nocapture`

use std::cell::Cell;
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
fn app_frame_and_screen_benchmark() {
	let owner = Owner::new();
	let navigation = Navigation::new(Route::Index);
	let render_count = Rc::new(Cell::new(0_u64));
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
	let count = render_count.clone();
	let test_owner = owner.clone();
	let mut tester = WidgetTester::new(
		move || {
			count.set(count.get() + 1);
			test_owner.with(|| screen.render())
		},
		390.0,
		844.0,
	)
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

	// Measure M13 & M14: 60 ticks (2 seconds of 30Hz combat)
	let renders_before = render_count.get();
	let mut frame_us = Vec::with_capacity(60);
	for _ in 0..60 {
		let t0 = Instant::now();
		reapers_app::hunt::view::play_on(1.0 / 30.0);
		tester.pump();
		frame_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let renders_after = render_count.get();
	let rebuilds_per_sec = (renders_after - renders_before) as f64 / 2.0;

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
		let val: serde_json::Value = serde_json::from_slice(&json_bytes).expect("val");
		let parsed: PlayerState = serde_json::from_value(val).expect("player");
		black_box(parsed);
		deser_us.push(t0.elapsed().as_secs_f64() * 1e6);
	}

	println!("=== reapers_app perf_bench (release) ===");
	println!(
		"M13_hunt_frame_us:         {:.1} us/frame (median over 60 ticks)",
		median(&mut frame_us)
	);
	println!(
		"M14_hunt_rebuilds_per_sec: {:.1} full-tree rebuilds/sec (at 30Hz)",
		rebuilds_per_sec
	);
	println!(
		"M15_vault_render_us:       {:.1} us/render (40 NFTs)",
		median(&mut vault_us)
	);
	println!(
		"M23_client_api_deser_us:   {:.1} us (Value->PlayerState baseline)",
		median(&mut deser_us)
	);
}
