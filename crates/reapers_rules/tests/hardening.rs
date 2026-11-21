//! Regression tests for rules hardening, validation bounds, and authoritative rift seats.

use reapers_rules::*;

#[test]
fn room_grid_accepts_all_32_five_bit_levels_and_rejects_malformed_json() {
	let levels: Vec<u8> = (0..RoomCell::MAX_LEVELS as u8).map(|i| i * 7).collect();
	let mut cells = vec![RoomCell::walkable(0).byte(); 16];
	cells[15] = RoomCell::walkable(RoomCell::MAX_LEVEL).byte();

	let grid = RoomGrid::new(145, 0, 0, 4, 4, 2, 2, levels, cells);
	assert!(grid.is_well_formed());
	assert_eq!(grid.top_centimetres_at(3, 3), 31 * 7);

	let encoded = grid.encode();
	let decoded = RoomGrid::decode(&encoded).expect("32-level grid must round-trip");
	assert_eq!(decoded, grid);

	// Malformed JSON (wrong cell count or out-of-range level index) must fail deserialization.
	let bad_cells = serde_json::json!({
		"eye_height_centimetres": 145,
		"pivot_arm_centimetres": 0,
		"grid_yaw": 0,
		"width": 4,
		"depth": 4,
		"player_x": 2,
		"player_z": 2,
		"levels_centimetres": [0],
		"cells": [0, 0],
	});
	assert!(serde_json::from_value::<RoomGrid>(bad_cells).is_err());

	let bad_level = serde_json::json!({
		"eye_height_centimetres": 145,
		"pivot_arm_centimetres": 0,
		"grid_yaw": 0,
		"width": 2,
		"depth": 2,
		"player_x": 0,
		"player_z": 0,
		"levels_centimetres": [0],
		"cells": [
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(0).byte(),
			RoomCell::walkable(5).byte(),
		],
	});
	assert!(serde_json::from_value::<RoomGrid>(bad_level).is_err());
}

#[test]
fn rift_room_ignores_nan_negative_and_oversized_reports_and_invalid_standing() {
	let entrant = RiftEntrant {
		name: "Vesper".to_owned(),
		loadout: HuntLoadout::new(
			ReaperClass::Harvester,
			4,
			Weapon::new(WeaponKind::Scythe, Temper::Sworn),
		),
	};
	let mut room = RiftRoom::new(RiftSite::LondonBridge, 42);
	let seat = room.join(entrant).unwrap();

	// Non-finite GeoStanding must not poison the room's anchor.
	room.report(
		seat,
		&RiftReport {
			ready: true,
			dealt: 0.0,
			outcome: None,
			standing: Some(GeoStanding {
				at: GeoPoint::new(f64::NAN, 0.0),
				facing: f64::NAN,
				accuracy: 5.0,
				observed_at: None,
			}),
		},
	);
	assert!(room.anchor().is_none());

	room.advance(RiftRules::COUNTDOWN_TICKS + 1);
	assert_eq!(room.phase(), RiftPhase::Fight);
	let full_ward = room.ward_max();

	// NaN and negative damage must never poison ward().
	room.report(
		seat,
		&RiftReport {
			ready: true,
			dealt: f64::NAN,
			outcome: None,
			standing: None,
		},
	);
	room.report(
		seat,
		&RiftReport {
			ready: true,
			dealt: -500.0,
			outcome: None,
			standing: None,
		},
	);
	assert!(room.ward().is_finite());
	assert!(room.ward() <= full_ward);

	// Infinite damage is ignored; finite damage is clamped at ward_max.
	room.report(
		seat,
		&RiftReport {
			ready: true,
			dealt: f64::INFINITY,
			outcome: None,
			standing: None,
		},
	);
	assert!(room.ward().is_finite());
	assert!(room.ward() > 0.0);
}

#[test]
fn authoritative_rift_room_steps_human_seat_and_ignores_client_damage_claims() {
	let entrant = RiftEntrant {
		name: "Vesper".to_owned(),
		loadout: HuntLoadout::new(
			ReaperClass::Harvester,
			6,
			Weapon::new(WeaponKind::Scythe, Temper::Sworn),
		),
	};
	let mut room = RiftRoom::authoritative(RiftSite::LondonBridge, 7);
	assert!(room.is_authoritative());
	let seat = room.join(entrant.clone()).unwrap();
	room.report(seat, &RiftReport::readiness(true));
	room.advance(RiftRules::COUNTDOWN_TICKS + 1);
	assert_eq!(room.phase(), RiftPhase::Fight);

	// Untrusted dealt/outcome reports are completely ignored in authoritative mode.
	room.report(
		seat,
		&RiftReport {
			ready: true,
			dealt: 999_999.0,
			outcome: Some(HuntOutcome::absorbed_with(
				CaptureGrade::Excellent,
				false,
				1,
				HuntDeeds::default(),
			)),
			standing: None,
		},
	);
	let frame = room.frame_for(seat).unwrap();
	assert_eq!(frame.you().unwrap().dealt, 0.0);
	assert!(frame.you().unwrap().cut.is_none());

	// Driving the authoritative human seat with real bot inputs seals the rift deterministically.
	let mut local = RiftSeat::new(
		room.seed(),
		room.site(),
		room.ward_max(),
		&entrant.loadout,
		None,
	)
	.unwrap();
	let mut bot = HuntBot::new(&local.sim, HuntBotStyle::CAREFUL);

	while !room.phase().is_over() {
		let frame = room.frame_for(seat).unwrap();
		local.sync(frame.ward, frame.you().unwrap().dealt);
		let input = bot.decide(&local.sim);
		local.step(&input);
		room.queue_inputs(seat, &[input]);
		room.advance(1);
	}

	assert_eq!(room.phase(), RiftPhase::Sealed);
	assert!(room.reward_for(seat).xp > 0);
	assert!(room.reward_for(seat).anima > 0);
}

#[test]
fn player_state_hardening_guards_rift_nft_negative_rewards_and_oath_ordering() {
	let base = PlayerState::initial();
	assert_eq!(
		base.wear(Outfit::Shrouded).unwrap_err().refusal,
		Refusal::OathRequired
	);

	// Binding an NFT reaper before taking the oath must still permit taking the oath
	// without wiping the bound NFT reaper.
	let nft_reaper = ReaperState::bound("asset111", ReaperGenome::shuffle(9));
	let with_nft = base.bind_reaper(nft_reaper.clone()).unwrap();
	let with_both = with_nft.take_oath(3).unwrap();
	assert_eq!(with_both.reapers().len(), 2);
	assert!(with_both.reapers().iter().any(|r| r.id() == "asset111"));
	assert!(with_both.reapers().iter().any(|r| !r.is_nft()));

	// settle_rift rejects negative rewards; entrant_for rejects unverified NFT rift entry.
	assert_eq!(
		with_both.settle_rift(-10, 5, None).unwrap_err().refusal,
		Refusal::RiftUnproven
	);
	assert_eq!(
		with_both.settle_rift(10, -5, None).unwrap_err().refusal,
		Refusal::RiftUnproven
	);
	assert_eq!(
		with_both
			.select_reaper("asset111")
			.unwrap()
			.entrant_for(RiftSite::LondonBridge)
			.unwrap_err()
			.refusal,
		Refusal::RiftUnproven
	);

	// reconcile_reapers rejects non-NFT entries in `owned`.
	let starter = ReaperState::bound(ReaperState::OATH_ID, ReaperGenome::shuffle(5));
	assert_eq!(
		with_both
			.reconcile_reapers(vec![starter])
			.unwrap_err()
			.refusal,
		Refusal::NotOwned
	);
}

#[test]
fn hunt_log_decode_and_stats_reject_broken_plans_and_bad_parameters() {
	// Zero eye height makes the plan ill-formed; HuntLog::decode must reject it immediately.
	let mut bytes = HuntLog::new(Vec::new()).encode();
	bytes[5] = 0; // eye_centimetres = 0 (< 30)
	assert_eq!(HuntLog::decode(&bytes), Err(HuntLogError::BrokenPlan));

	let valid = HuntLoadout::new(
		ReaperClass::Harvester,
		1,
		Weapon::new(WeaponKind::Scythe, Temper::Shrouded),
	);
	assert!(HuntStats::derive(&valid, 0.0).is_err());
	assert!(HuntStats::derive(&valid, -1.0).is_err());
	assert!(HuntStats::derive(&valid, f64::NAN).is_err());

	let neg_sets = HuntLoadout {
		completed_sets: -1,
		..valid
	};
	assert!(HuntStats::derive(&neg_sets, 1.0).is_err());

	// HuntOutcome deserialization refuses simultaneous grade + escape and negative ticks.
	let both = serde_json::json!({
		"grade": "excellent",
		"escape": "time",
		"ambushed": false,
		"ticks": 100
	});
	assert!(serde_json::from_value::<HuntOutcome>(both).is_err());

	let negative_ticks = serde_json::json!({
		"grade": "excellent",
		"escape": null,
		"ambushed": false,
		"ticks": -1
	});
	assert!(serde_json::from_value::<HuntOutcome>(negative_ticks).is_err());
}

#[test]
fn timestamp_rejects_impossible_calendar_dates_and_offsets() {
	assert!(Timestamp::parse("2026-02-31T12:00:00Z").is_err());
	assert!(Timestamp::parse("2026-04-31T12:00:00Z").is_err());
	assert!(Timestamp::parse("2026-10-01T24:59:59Z").is_err());
	assert!(Timestamp::parse("2026-10-01T12:00:00+99:00").is_err());
	assert!(Timestamp::parse("2026-10-01T12:00:00+01:").is_err());
	// Leap day in leap year passes; non-leap year fails.
	assert!(Timestamp::parse("2024-02-29T12:00:00Z").is_ok());
	assert!(Timestamp::parse("2026-02-29T12:00:00Z").is_err());
}

#[test]
fn halted_flying_husk_does_not_teleport_to_perch() {
	let mut cells = vec![RoomCell::walkable(0).byte(); 60 * 60];
	// Small elevated perch at (30, 45).
	for z in 44..48 {
		for x in 28..33 {
			cells[z * 60 + x] = RoomCell::walkable(1).byte();
		}
	}
	let grid = RoomGrid::new(145, 0, 0, 60, 60, 30, 30, vec![0, 80], cells);
	let nav = RoomNav::new(grid, Vessel::Bird.locomotion(), 5.0, 1.5);
	let start = Vec3::new(0.0, 0.2, 2.5);
	let mut motion = HuskMotion::new(&nav, SeededRandom::new(1), Some(start), 0.0).unwrap();

	let perches = nav.spots_of(RoomSpotKind::Perch);
	assert!(!perches.is_empty(), "expected elevated perch spot");
	let perch = *perches[0];
	motion.go_to_spot(perch, MotionPace::Run);
	// Fly into cruise altitude, partway toward the perch.
	for _ in 0..25 {
		motion.tick();
	}
	assert_eq!(motion.state(), HuskMotionState::Air);
	let pos_before_halt = motion.position();

	motion.halt();
	motion.tick();

	let jump = (motion.position() - pos_before_halt).length();
	assert!(
		jump < 0.05,
		"halted flier must stay where it halted instead of teleporting to perch (jumped {jump} m)"
	);
	assert_eq!(motion.state(), HuskMotionState::Idle);
}
