//! Repeatable micro- and macro-benchmarks for `reapers_rules`.
//!
//! Run with: `cargo run --release -p reapers_rules --example perf_bench`

use std::hint::black_box;
use std::time::Instant;

use reapers_rules::Difficulty;
use reapers_rules::Eye;
use reapers_rules::HuntBot;
use reapers_rules::HuntInput;
use reapers_rules::HuntLog;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::PlayerState;
use reapers_rules::Rarity;
use reapers_rules::ReaperGenome;
use reapers_rules::ReaperState;
use reapers_rules::RiftEntrant;
use reapers_rules::RiftRoom;
use reapers_rules::RiftSite;
use reapers_rules::RoomCell;
use reapers_rules::RoomGrid;
use reapers_rules::RoomNav;
use reapers_rules::Sigil;
use reapers_rules::Timestamp;
use reapers_rules::Vec3;
use reapers_rules::Vessel;
use reapers_rules::market_listings;

fn median_us(samples: &mut [f64]) -> f64 {
	samples.sort_by(f64::total_cmp);
	samples[samples.len() / 2]
}

fn furnished_grid() -> RoomGrid {
	let width: u8 = 80;
	let depth: u8 = 60;
	let levels = vec![0_u8, 45, 75, 90];
	let mut cells = vec![RoomCell::walkable(0).byte(); usize::from(width) * usize::from(depth)];

	let mut fill = |x0: usize, z0: usize, w: usize, d: usize, cell: u8| {
		for z in z0..(z0 + d).min(usize::from(depth)) {
			for x in x0..(x0 + w).min(usize::from(width)) {
				cells[z * usize::from(width) + x] = cell;
			}
		}
	};

	// Walls on far edges.
	fill(0, 0, 80, 2, RoomCell::WALL.byte());
	fill(0, 58, 80, 2, RoomCell::WALL.byte());
	fill(0, 0, 2, 60, RoomCell::WALL.byte());
	fill(78, 0, 2, 60, RoomCell::WALL.byte());
	// Desk at 75 cm + hidden floor behind it.
	fill(32, 42, 16, 8, RoomCell::walkable(2).byte());
	fill(32, 50, 16, 8, RoomCell::hidden(0).byte());
	// Chair beside desk at 45 cm.
	fill(25, 42, 6, 6, RoomCell::walkable(1).byte());
	// Counter on right at 90 cm.
	fill(56, 30, 14, 18, RoomCell::walkable(3).byte());
	fill(70, 30, 8, 18, RoomCell::hidden(0).byte());
	// Sofa on left: seat 45 cm + obstacle back 88 cm.
	fill(8, 28, 8, 18, RoomCell::walkable(1).byte());
	fill(6, 28, 2, 18, RoomCell::obstacle(88).byte());
	// Unknown patch in corner.
	fill(58, 4, 18, 12, RoomCell::UNKNOWN.byte());

	RoomGrid::new(145, 0, 0, width, depth, 40, 20, levels, cells)
}

fn record_hunt(plan: HuntPlan, vessel: Vessel, rarity: Rarity, seed: i64) -> HuntLog {
	let state = PlayerState::initial().take_oath(42).expect("oath");
	let stats = HuntStats::derive(&state.loadout(), 1.0).expect("stats");
	let mut sim = HuntSim::new(
		seed,
		vessel,
		rarity,
		rarity.ward(),
		&stats,
		plan.clone(),
		Difficulty::Normal,
	)
	.expect("sim");
	let mut bot = HuntBot::careful(&sim);
	let _ = bot.play(&mut sim);

	HuntLog {
		inputs: bot.recorded,
		plan,
		difficulty: Difficulty::Normal,
	}
}

fn rich_player_state() -> PlayerState {
	let mut state = PlayerState::initial().take_oath(1001).expect("oath");
	let now = Timestamp::from_millis(1_790_000_000_000);
	let listings = market_listings();

	for (i, &nft) in listings.iter().take(4).enumerate() {
		if let Ok(next) = state.acquire(nft) {
			state = next;
		}
		if i < 2
			&& let Ok(next) = state.release(nft, 100 + i as i64, now)
		{
			state = next;
		}
	}

	let mut json: serde_json::Value = serde_json::to_value(&state).expect("to_value");
	let mut absorptions = Vec::new();
	for (i, &nft) in listings.iter().enumerate() {
		absorptions.push(serde_json::json!({
			"nft": nft.id(),
			"grade": "clean",
			"feeding": "rich",
			"xp": 180,
			"anima": 45,
			"nonce": (i + 1) as u64,
			"at": Timestamp::from_millis(1_790_000_000_000 + (i as i64) * 60_000).to_iso8601(),
			"signature": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
		}));
	}
	if let Some(reapers) = json["reapers"].as_array_mut() {
		if let Some(starter) = reapers.first_mut() {
			starter["codex"] = serde_json::Value::Array(absorptions.clone());
			starter["xp"] = serde_json::json!(18_000);
		}
		for i in 0..7_i64 {
			let genome = ReaperGenome::shuffle(2000 + i);
			let reaper = ReaperState::bound(format!("AssetAddr{i:036}"), genome);
			let mut val = serde_json::to_value(&reaper).expect("reaper val");
			val["codex"] = serde_json::Value::Array(absorptions[..10].to_vec());
			reapers.push(val);
		}
	}
	serde_json::from_value(json).expect("rich player")
}

#[allow(clippy::too_many_lines)]
fn main() {
	let state = PlayerState::initial().take_oath(42).expect("oath");
	let stats = HuntStats::derive(&state.loadout(), 1.0).expect("stats");

	// M1: Open-room full hunt replay
	let open_log = record_hunt(HuntPlan::OPEN, Vessel::Ape, Rarity::Rare, 777);
	let open_bytes = open_log.encode();
	let open_ticks = open_log.inputs.len();
	let mut m1_samples = Vec::with_capacity(15);
	for _ in 0..15 {
		let t0 = Instant::now();
		let decoded = HuntLog::decode(&open_bytes).expect("decode");
		let outcome = HuntSim::replay(
			777,
			Vessel::Ape,
			Rarity::Rare,
			Rarity::Rare.ward(),
			&stats,
			&decoded,
		)
		.expect("replay");
		black_box(outcome);
		m1_samples.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let m1 = median_us(&mut m1_samples);

	// M2: Room-grid full hunt replay (walker on 80x60 furnished grid)
	let grid = furnished_grid();
	let room_plan = HuntPlan {
		room: Some(grid.clone()),
		..HuntPlan::new(0, 6000)
	};
	let room_log = record_hunt(room_plan, Vessel::Ape, Rarity::Rare, 888);
	let room_bytes = room_log.encode();
	let room_ticks = room_log.inputs.len();
	let mut m2_samples = Vec::with_capacity(9);
	for _ in 0..9 {
		let t0 = Instant::now();
		let decoded = HuntLog::decode(&room_bytes).expect("decode");
		let outcome = HuntSim::replay(
			888,
			Vessel::Ape,
			Rarity::Rare,
			Rarity::Rare.ward(),
			&stats,
			&decoded,
		)
		.expect("replay");
		black_box(outcome);
		m2_samples.push(t0.elapsed().as_secs_f64() * 1e3);
	}
	let m2 = median_us(&mut m2_samples);

	// M3: RoomNav::for_hunt startup latency
	let profile = Vessel::Ape.locomotion();
	let mut m3_samples = Vec::with_capacity(15);
	for _ in 0..15 {
		let t0 = Instant::now();
		let nav = RoomNav::for_hunt(
			grid.clone(),
			profile,
			HuntRules::STAGING_FAR_METRES,
			HuntRules::NEAREST_METRES,
		);
		black_box(nav);
		m3_samples.push(t0.elapsed().as_secs_f64() * 1e3);
	}
	let m3 = median_us(&mut m3_samples);

	// M4: RoomNav::path multi-region query latency
	let nav = RoomNav::for_hunt(
		grid,
		profile,
		HuntRules::STAGING_FAR_METRES,
		HuntRules::NEAREST_METRES,
	);
	let queries = [
		(Vec3::new(-1.5, 0.0, 1.5), Vec3::new(0.0, 0.75, 2.5)),
		(Vec3::new(0.0, 0.75, 2.5), Vec3::new(2.0, 0.90, 1.8)),
		(Vec3::new(2.0, 0.90, 1.8), Vec3::new(-2.8, 0.45, 1.5)),
		(Vec3::new(-2.8, 0.45, 1.5), Vec3::new(0.0, 0.0, 3.4)),
	];
	let mut m4_samples = Vec::with_capacity(50);
	for _ in 0..50 {
		let t0 = Instant::now();
		for &(from, to) in &queries {
			black_box(nav.path(from, to));
		}
		m4_samples.push(t0.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
	}
	let m4 = median_us(&mut m4_samples);

	// M5: Sigil::draw (Epic 6-light 3-crossing sigil)
	let shape = Rarity::Epic.sigil();
	let mut m5_samples = Vec::with_capacity(20);
	for seed in 1..=20_i64 {
		let t0 = Instant::now();
		let sigil = Sigil::draw(seed * 97, &shape, Vessel::Ape.gait(), Difficulty::Normal);
		black_box(sigil);
		m5_samples.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let m5 = median_us(&mut m5_samples);

	// M6 & M7: Rich PlayerState serialize / deserialize
	let rich = rich_player_state();
	let json_bytes = serde_json::to_vec(&rich).expect("ser");
	let mut m6_samples = Vec::with_capacity(50);
	let mut m7_samples = Vec::with_capacity(50);
	for _ in 0..50 {
		let t0 = Instant::now();
		let out = serde_json::to_vec(&rich).expect("ser");
		black_box(&out);
		m6_samples.push(t0.elapsed().as_secs_f64() * 1e6);

		let t1 = Instant::now();
		let back: PlayerState = serde_json::from_slice(&json_bytes).expect("de");
		black_box(back);
		m7_samples.push(t1.elapsed().as_secs_f64() * 1e6);
	}
	let m6 = median_us(&mut m6_samples);
	let m7 = median_us(&mut m7_samples);

	// M8: RiftRoom 1 second of fight (1 human + 3 shades, 60 ticks + 10 frame_for calls)
	let mut m8_samples = Vec::with_capacity(15);
	for _ in 0..15 {
		let mut room = RiftRoom::new(RiftSite::LondonBridge, 4242);
		let seat = room
			.join(RiftEntrant {
				name: "Vesper".into(),
				loadout: state.loadout(),
			})
			.expect("join");
		room.report(seat, &reapers_rules::RiftReport::readiness(true));
		// Advance past gathering countdown into Fight.
		while room.phase() == reapers_rules::RiftPhase::Gathering {
			room.advance(6);
		}
		assert_eq!(room.phase(), reapers_rules::RiftPhase::Fight);
		let t0 = Instant::now();
		for _ in 0..10 {
			room.advance(6);
			black_box(room.frame_for(seat));
		}
		m8_samples.push(t0.elapsed().as_secs_f64() * 1e6);
	}
	let m8 = median_us(&mut m8_samples);

	// M29: 90-second moving-eye hunt log wire size (5400 ticks)
	let long_inputs: Vec<HuntInput> = (0..5400)
		.map(|t| {
			let a = f64::from(t) * 0.015;
			let eye = Eye::rounded(Vec3::new(
				0.2 * libm::sin(a * 0.3),
				0.04 * libm::cos(a * 0.5),
				0.15 * libm::cos(a * 0.2),
			))
			.expect("eye");
			HuntInput::measured(
				Vec3::from_angles(0.6 * libm::sin(a), 0.2 * libm::cos(a * 0.7)),
				0.4 * libm::cos(a).abs(),
				t % 18 == 0,
				0,
				t > 4800,
			)
			.with_eye(eye)
		})
		.collect();
	let long_log = HuntLog {
		inputs: long_inputs,
		plan: HuntPlan {
			room: Some(furnished_grid()),
			..HuntPlan::new(0, 6000)
		},
		difficulty: Difficulty::Normal,
	};
	let long_raw = long_log.encode();
	let long_b64_len = long_raw.len().div_ceil(3) * 4;
	let long_compact = long_log.encode_compact();
	let long_compact_b64_len = long_compact.len().div_ceil(3) * 4;
	assert_eq!(
		HuntLog::decode(&long_compact).expect("compact decode"),
		long_log
	);

	println!("=== reapers_rules perf_bench (release) ===");
	println!(
		"M1_hunt_open_replay_us:   {m1:.1} us ({open_ticks} ticks, {:.1} ns/tick)",
		m1 * 1000.0 / open_ticks as f64
	);
	println!("M2_hunt_room_replay_ms:   {m2:.3} ms ({room_ticks} ticks)");
	println!("M3_room_nav_derive_ms:    {m3:.3} ms");
	println!("M4_room_path_astar_us:    {m4:.1} us/query");
	println!("M5_sigil_draw_us:         {m5:.1} us");
	println!(
		"M6_player_state_ser_us:   {m6:.1} us ({} bytes JSON)",
		json_bytes.len()
	);
	println!("M7_player_state_de_us:    {m7:.1} us");
	println!("M8_rift_room_1s_us:       {m8:.1} us (60 ticks + 10 frames)");
	println!(
		"M29_hunt_log_90s_raw:     {} bytes (base64: {} bytes) | compact v13: {} bytes (base64: {} bytes)",
		long_raw.len(),
		long_b64_len,
		long_compact.len(),
		long_compact_b64_len
	);
}
