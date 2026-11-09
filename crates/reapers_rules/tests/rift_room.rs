//! Rift rooms: the circle, the ward, the phases and the verdict.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use reapers_rules::*;
use support::*;

/// A person who fights well but never lets the orb go.
fn no_cut(sim: &HuntSim, _person: usize) -> HuntBot {
	HuntBot::new(
		sim,
		HuntBotStyle {
			reaction_ticks: 16,
			miss_every: 7,
			orb_hold_ticks: 1 << 30,
			..HuntBotStyle::CAREFUL
		},
	)
}

/// A person who never strikes, but makes the cut when the ward breaks.
fn watcher(sim: &HuntSim) -> HuntBot {
	HuntBot::new(
		sim,
		HuntBotStyle {
			fires: false,
			..HuntBotStyle::CAREFUL
		},
	)
}

/// The ticks at which `seat`'s husk began to shriek while `bot` played it.
fn shriek_ticks(seat: &mut RiftSeat, bot: &mut HuntBot, ticks: i64) -> Vec<i64> {
	let mut shrieks = Vec::new();

	for tick in 0..ticks {
		let input = bot.decide(&seat.sim);
		let events = seat.step(&input);

		if events
			.iter()
			.any(|event| matches!(event, HuntEvent::ShriekBegan))
		{
			shrieks.push(tick);
		}
	}

	shrieks
}

fn named(name: &str) -> RiftEntrant {
	entrant(name, ReaperClass::Harvester, 1)
}

fn at_level(level: i64) -> RiftEntrant {
	entrant("Vesper", ReaperClass::Harvester, level)
}

fn of_class(reaper_class: ReaperClass) -> RiftEntrant {
	entrant("Vesper", reaper_class, 1)
}

fn ready() -> RiftReport {
	RiftReport::readiness(true)
}

/// A report from a reaper who is ready and has dealt `dealt`.
fn dealt(dealt: f64, outcome: Option<HuntOutcome>) -> RiftReport {
	RiftReport {
		dealt,
		outcome,
		..ready()
	}
}

fn london() -> RiftRoom {
	RiftRoom::new(RiftSite::LondonBridge, 3)
}

fn frame(room: &RiftRoom, seat: usize) -> RiftFrame {
	room.frame_for(seat).unwrap()
}

fn you(frame: &RiftFrame) -> &RiftMember {
	frame.you().unwrap()
}

fn close_to(actual: f64, expected: f64, delta: f64) -> bool {
	(actual - expected).abs() <= delta
}

/// A value written to JSON and read back.
fn sent<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
	serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}

mod gathering {
	use super::*;

	#[test]
	fn the_count_starts_once_everyone_present_stands_ready() {
		let mut room = london();
		let first = room.join(named("Vesper")).unwrap();
		let second = room.join(named("Morrow")).unwrap();

		room.report(first, &ready());
		room.advance(30);

		assert!(!frame(&room, first).counting_down);

		room.report(second, &ready());
		room.advance(1);

		let frame = frame(&room, first);

		assert!(frame.counting_down);
		assert_eq!(frame.ticks_left, RiftRules::COUNTDOWN_TICKS);
		assert_eq!(frame.phase, RiftPhase::Gathering);
	}

	#[test]
	fn a_reaper_leaving_stops_the_count() {
		let mut room = london();
		let first = room.join(default_entrant()).unwrap();
		let second = room.join(default_entrant()).unwrap();

		room.report(first, &ready());
		room.report(second, &ready());
		room.advance(10);
		room.leave(second);

		assert!(!frame(&room, first).counting_down);
		assert_eq!(frame(&room, first).members.len(), 1);
	}

	#[test]
	fn a_rift_does_not_wait_forever_on_a_reaper_who_is_not_ready() {
		let mut room = london();

		room.join(default_entrant()).unwrap();
		room.advance(RiftRules::GATHER_LIMIT_TICKS + RiftRules::COUNTDOWN_TICKS + 1);

		assert_eq!(room.phase(), RiftPhase::Fight);
	}

	#[test]
	fn a_full_circle_and_a_rift_already_begun_take_nobody_else() {
		let mut full = london();

		for _ in 0..RiftRules::CIRCLE {
			full.join(default_entrant()).unwrap();
		}

		assert!(!full.is_open());
		assert_eq!(
			full.join(default_entrant()),
			Err(RiftError::NotTakingReapers)
		);

		let mut begun = RiftTable::new(vec![default_entrant()]);

		begun.begin();

		assert!(!begun.room.is_open());
	}

	#[test]
	fn a_rift_everyone_walked_away_from_is_abandoned() {
		let mut room = london();
		let seat = room.join(default_entrant()).unwrap();

		assert!(!room.is_abandoned());

		room.leave(seat);

		assert!(room.is_abandoned());
	}
}

mod the_circle {
	use super::*;

	#[test]
	fn shades_fill_the_seats_nobody_took() {
		let mut table = RiftTable::new(vec![at_level(6)]);

		table.begin();

		assert_eq!(table.seats.len(), 1);

		let frame = frame(&table.room, table.seats[0]);
		let shades: Vec<&RiftMember> = frame.members.iter().filter(|member| member.shade).collect();

		assert_eq!(frame.members.len(), RiftRules::CIRCLE);
		assert!(!you(&frame).shade);
		assert_eq!(shades.len(), RiftRules::CIRCLE - 1);
		assert!(
			shades
				.iter()
				.all(|member| member.level == 6 - RiftRules::SHADE_LEVEL_GAP)
		);
	}

	#[test]
	fn the_same_seed_always_sends_the_same_shades() {
		let shades = || {
			let mut table =
				RiftTable::with(vec![default_entrant()], RiftSite::LondonBridge, 21, careful);

			table.begin();

			frame(&table.room, 0)
				.members
				.iter()
				.map(|member| format!("{}/{}", member.name, member.reaper_class.code()))
				.collect::<Vec<_>>()
		};

		assert_eq!(shades(), shades());
	}

	#[test]
	fn the_ward_is_sized_to_the_circle_not_to_a_fixed_number() {
		let mut novices = RiftTable::new(vec![default_entrant(); 4]);
		let mut veterans = RiftTable::new(vec![at_level(20); 4]);

		novices.begin();
		veterans.begin();

		assert!(veterans.room.ward_max() > novices.room.ward_max() * 2.0);
	}
}

mod one_fight_shared {
	use super::*;

	#[test]
	fn there_is_nothing_to_stalk_the_husk_is_ahead_and_the_fight_is_on() {
		let seat = RiftSeat::new(
			5,
			RiftSite::LondonBridge,
			1000.0,
			&default_entrant().loadout,
			None,
		)
		.unwrap();

		assert_eq!(seat.sim.phase(), HuntPhase::Fight);
		assert!(seat.sim.pose().yaw().abs() < radians(1.0));
	}

	#[test]
	fn everyone_hears_the_shriek_at_the_same_moment() {
		let seat = |reaper_class| {
			RiftSeat::new(
				5,
				RiftSite::LondonBridge,
				1e9,
				&of_class(reaper_class).loadout,
				None,
			)
			.unwrap()
		};
		let mut striker = seat(ReaperClass::Harvester);
		let mut watching = seat(ReaperClass::Mourner);
		let ticks = 40 * HuntRules::TICK_RATE;

		let mut striker_bot = HuntBot::human(&striker.sim);
		let struck = shriek_ticks(&mut striker, &mut striker_bot, ticks);
		let mut watcher_bot = watcher(&watching.sim);
		let watched = shriek_ticks(&mut watching, &mut watcher_bot, ticks);

		assert!(struck.len() >= 3, "{struck:?}");
		assert!(striker.dealt() > 0.0);
		assert_eq!(watching.dealt(), 0.0);
		assert_eq!(struck, watched);
	}

	#[test]
	fn a_strike_anyone_lands_comes_off_the_ward_everyone_sees() {
		let mut table = RiftTable::new(vec![default_entrant(), default_entrant()]);

		table.begin();

		for _ in 0..12 * HuntRules::TICK_RATE {
			table.step();
		}

		let frame = frame(&table.room, table.seats[0]);
		let mut dealt = 0.0;

		for member in &frame.members {
			dealt += member.dealt;
		}

		assert!(dealt > 0.0);
		assert!(close_to(frame.ward, frame.ward_max - dealt, 1e-6));
	}

	#[test]
	fn the_ward_breaks_for_a_reaper_who_never_landed_a_blow() {
		let mut table = RiftTable::with(
			vec![default_entrant(); 4],
			RiftSite::LondonBridge,
			11,
			|sim, person| {
				if person == 0 {
					watcher(sim)
				} else {
					HuntBot::human(sim)
				}
			},
		);

		table.play();

		let frame = frame(&table.room, 0);

		assert_eq!(table.room.phase(), RiftPhase::Sealed);
		assert_eq!(you(&frame).dealt, 0.0);
		assert!(you(&frame).cut.is_some());
	}

	#[test]
	fn a_report_can_be_repeated_or_arrive_late_without_being_recounted() {
		let mut table = RiftTable::new(vec![default_entrant()]);

		table.begin();

		assert_eq!(table.seats.len(), 1);

		let seat = table.seats[0];

		table.room.report(seat, &dealt(40.0, None));
		table.room.report(seat, &dealt(40.0, None));
		table.room.report(seat, &dealt(25.0, None));

		assert_eq!(you(&frame(&table.room, seat)).dealt, 40.0);
	}

	#[test]
	fn a_seat_shows_its_own_strikes_before_the_room_has_heard_of_them() {
		let mut seat = RiftSeat::new(
			5,
			RiftSite::LondonBridge,
			1000.0,
			&default_entrant().loadout,
			None,
		)
		.unwrap();

		seat.sim.set_dealt(60.0);

		// The room has counted 20 of this seat's 60 and others dealt 100.
		seat.sync(1000.0 - 100.0 - 20.0, 20.0);

		assert_eq!(seat.sim.ward(), 1000.0 - 100.0 - 60.0);
	}

	/// The rules once refused this outright. They leave a lone hunt's ward
	/// alone instead, so this checks that sharing changes nothing.
	#[test]
	fn a_lone_hunt_has_no_shared_ward() {
		let mut sim = default_sim();
		let before = sim.ward();

		sim.share_ward(10.0);

		assert_eq!(sim.ward(), before);
	}
}

mod sealing {
	use super::*;

	#[test]
	fn one_careful_reaper_and_three_shades_seal_it_with_time_to_spare() {
		let mut table = RiftTable::new(vec![at_level(4)]);

		table.play();

		let frame = frame(&table.room, 0);

		assert_eq!(table.room.phase(), RiftPhase::Sealed);
		assert!(table.room.tick() < RiftRules::FIGHT_LIMIT_TICKS);
		assert!(frame.cuts() >= frame.cuts_needed());
		assert!(frame.xp > 0);
		assert!(frame.anima > 0);
	}

	#[test]
	fn four_reapers_break_the_ward_sooner_than_one_with_shades() {
		let mut alone = RiftTable::new(vec![at_level(4)]);
		let mut together = RiftTable::new(
			ReaperClass::ALL
				.iter()
				.take(4)
				.map(|&reaper_class| entrant("Vesper", reaper_class, 4))
				.collect(),
		);

		alone.play();
		together.play();

		assert_eq!(together.room.phase(), RiftPhase::Sealed);
		assert!(
			together.room.tick() < alone.room.tick() - 10 * HuntRules::TICK_RATE,
			"{} against {}",
			together.room.tick(),
			alone.room.tick()
		);
	}

	#[test]
	fn shades_cannot_shut_a_rift_without_a_cut_from_the_living() {
		let mut table = RiftTable::with(vec![at_level(4)], RiftSite::LondonBridge, 11, no_cut);

		table.play();

		let frame = frame(&table.room, 0);

		assert!(frame.cuts() >= frame.cuts_needed());
		assert_eq!(table.room.phase(), RiftPhase::Failed);
		assert_eq!(frame.xp, 0);
	}

	#[test]
	fn half_the_circle_rounded_up_has_to_cut() {
		assert_eq!(RiftFrame::cuts_needed_for(4), 2);
		assert_eq!(RiftFrame::cuts_needed_for(3), 2);
		assert_eq!(RiftFrame::cuts_needed_for(2), 1);
		assert_eq!(RiftFrame::cuts_needed_for(1), 1);
		assert_eq!(RiftFrame::cuts_needed_for(0), 1);
	}

	#[test]
	fn a_ward_nobody_breaks_in_time_leaves_the_rift_open() {
		let mut room = london();

		for seat in 0..RiftRules::CIRCLE {
			room.join(default_entrant()).unwrap();
			room.report(seat, &ready());
		}

		room.advance(RiftRules::COUNTDOWN_TICKS + RiftRules::FIGHT_LIMIT_TICKS + 2);

		assert_eq!(room.phase(), RiftPhase::Failed);
	}

	#[test]
	fn a_circle_everyone_walked_out_of_is_left_to_its_shades() {
		let mut table = RiftTable::new(vec![default_entrant(), default_entrant()]);

		table.begin();

		for &seat in &table.seats {
			table.room.leave(seat);
		}

		table.room.advance(1);

		// The shades fight on for nobody, so the room is free to be dropped.
		assert!(table.room.is_abandoned());
	}
}

mod what_it_pays {
	use super::*;

	/// A rift sealed by a full circle, in which seat 0 dealt
	/// `report.dealt` shares of the ward and ended with `report.outcome`.
	fn sealed_with(report: &RiftReport) -> RiftRoom {
		let mut room = london();

		for seat in 0..RiftRules::CIRCLE {
			room.join(default_entrant()).unwrap();
			room.report(seat, &ready());
		}

		room.advance(RiftRules::COUNTDOWN_TICKS + 1);

		let share = room.ward_max() / RiftRules::CIRCLE as f64;
		let cut = Some(clean());

		room.report(0, &dealt(report.dealt * share, report.outcome));

		for seat in 1..RiftRules::CIRCLE {
			room.report(seat, &dealt(share, cut));
		}

		// The first seat may have dealt less than its share; the rest make up
		// the difference so the ward still breaks.
		room.report(1, &dealt(share * 2.0, cut));
		room.advance(RiftRules::REAP_WINDOW_TICKS + 120);

		assert_eq!(room.phase(), RiftPhase::Sealed);

		room
	}

	fn clean() -> HuntOutcome {
		HuntOutcome::absorbed_with(CaptureGrade::Great, false, 1, HuntDeeds::default())
	}

	fn perfect() -> HuntOutcome {
		HuntOutcome::absorbed_with(CaptureGrade::Excellent, false, 1, HuntDeeds::default())
	}

	fn cast_out() -> HuntOutcome {
		HuntOutcome::escaped_by(EscapeCause::Resolve, false, 1, HuntDeeds::default())
	}

	fn missed() -> HuntOutcome {
		HuntOutcome::escaped_by(EscapeCause::Lapsed, false, 1, HuntDeeds::default())
	}

	fn round(value: f64) -> i64 {
		value.round() as i64
	}

	#[test]
	fn a_full_share_and_a_clean_cut_earn_the_stated_reward() {
		let room = sealed_with(&dealt(1.0, Some(clean())));

		assert_eq!(
			room.reward_for(0),
			RiftReward {
				xp: round(RiftRules::XP as f64 * ReapGrade::Clean.reward()),
				anima: round(RiftRules::ANIMA as f64 * ReapGrade::Clean.reward()),
			}
		);
	}

	#[test]
	fn a_better_cut_earns_more() {
		let better = sealed_with(&dealt(1.0, Some(perfect())));
		let worse = sealed_with(&dealt(1.0, Some(clean())));

		assert!(better.reward_for(0).xp > worse.reward_for(0).xp);
	}

	#[test]
	fn pulling_less_than_a_fair_share_earns_less_but_never_nothing() {
		let idle = sealed_with(&dealt(0.0, Some(clean())));

		assert_eq!(
			idle.reward_for(0).xp,
			round(RiftRules::XP as f64 * RiftRules::IDLE_SHARE * ReapGrade::Clean.reward())
		);
	}

	#[test]
	fn pulling_more_than_a_fair_share_earns_no_more_than_the_whole() {
		let eager = sealed_with(&dealt(1.5, Some(clean())));

		assert_eq!(
			eager.reward_for(0).xp,
			round(RiftRules::XP as f64 * ReapGrade::Clean.reward())
		);
	}

	#[test]
	fn missing_the_cut_and_being_cast_out_each_cost_a_share() {
		let uncut = sealed_with(&dealt(1.0, Some(missed())));
		let out = sealed_with(&dealt(1.0, Some(cast_out())));

		assert_eq!(
			uncut.reward_for(0).xp,
			round(RiftRules::XP as f64 * RiftRules::UNCUT_SHARE)
		);
		assert_eq!(
			out.reward_for(0).xp,
			round(RiftRules::XP as f64 * RiftRules::CAST_OUT_SHARE)
		);
		assert!(!you(&frame(&out, 0)).standing);
	}

	#[test]
	fn a_reaper_who_walked_away_before_the_end_is_paid_nothing() {
		let mut table = RiftTable::new(vec![default_entrant(); 4]);

		table.begin();

		for _ in 0..10 * HuntRules::TICK_RATE {
			table.step();
		}

		table.walk_away(3);

		while !table.room.phase().is_over() {
			table.step();
		}

		assert_eq!(table.room.phase(), RiftPhase::Sealed);
		assert_eq!(
			table.room.reward_for(table.seats[3]),
			RiftReward { xp: 0, anima: 0 }
		);
		assert!(table.room.reward_for(table.seats[0]).xp > 0);
	}

	#[test]
	fn leaving_once_it_is_over_changes_nothing() {
		let mut room = sealed_with(&dealt(1.0, Some(clean())));
		let paid = room.reward_for(0);

		room.leave(0);

		assert_eq!(room.reward_for(0), paid);
		assert_eq!(frame(&room, 0).members.len(), RiftRules::CIRCLE);
	}
}

mod frames_and_reports {
	use super::*;

	#[test]
	fn a_frame_survives_being_sent() {
		let mut table = RiftTable::new(vec![default_entrant()]);

		table.play();

		let frame = frame(&table.room, 0);
		let copy = sent(&frame);

		assert_eq!(
			serde_json::to_value(&copy).unwrap(),
			serde_json::to_value(&frame).unwrap()
		);
		assert_eq!(copy.phase, RiftPhase::Sealed);
		assert_eq!(you(&copy).cut, you(&frame).cut);
	}

	#[test]
	fn a_report_survives_being_sent_with_and_without_an_outcome() {
		let fighting = dealt(12.5, None);
		let done = dealt(
			80.0,
			Some(HuntOutcome::absorbed_with(
				CaptureGrade::Excellent,
				false,
				900,
				HuntDeeds::default(),
			)),
		);

		assert_eq!(
			serde_json::to_value(sent(&fighting)).unwrap(),
			serde_json::to_value(&fighting).unwrap()
		);
		assert_eq!(
			sent(&done).outcome.unwrap().grade(),
			Some(CaptureGrade::Excellent)
		);
	}
}

mod attunement {
	use super::*;

	fn now() -> Timestamp {
		Timestamp::utc(2026, 10, 1, 0, 0, 0)
	}

	fn bound() -> PlayerState {
		PlayerState::initial().take_oath(4).unwrap()
	}

	fn absorbed(player: &PlayerState, nft: Nft) -> PlayerState {
		let released = player.acquire(nft).unwrap().release(nft, 9, now()).unwrap();

		assert_eq!(released.sightings().len(), 1);

		let sighting = released.sightings()[0];

		released
			.settle_hunt(
				sighting.id,
				&HuntOutcome::absorbed_with(CaptureGrade::Great, false, 600, HuntDeeds::default()),
				sighting.surfaces_at,
				1,
			)
			.unwrap()
	}

	#[test]
	fn a_rift_stays_shut_to_a_reaper_who_carries_none_of_its_dead() {
		let player = bound();

		assert!(!player.attuned_to(RiftSite::LondonBridge));
		assert_eq!(
			player
				.entrant_for(RiftSite::LondonBridge)
				.unwrap_err()
				.refusal,
			Refusal::NotAttuned
		);
	}

	#[test]
	fn absorbing_one_of_its_collection_opens_it_and_only_it() {
		let player = absorbed(&bound(), Nft::new(NftCollection::DegenApes, 1));

		assert!(player.attuned_to(RiftSite::LondonBridge));
		assert!(!player.attuned_to(RiftSite::ShibuyaCrossing));
		assert_eq!(
			player.entrant_for(RiftSite::LondonBridge).unwrap().name,
			player.genome().name()
		);
	}

	#[test]
	fn nobody_enters_a_rift_before_the_oath() {
		assert_eq!(
			PlayerState::initial()
				.entrant_for(RiftSite::LondonBridge)
				.unwrap_err()
				.refusal,
			Refusal::OathRequired
		);
	}

	#[test]
	fn sealing_a_rift_is_paid_into_the_ledger() {
		let player = bound().settle_rift(320, 140, None).unwrap();

		assert_eq!(player.xp(), 320);
		assert_eq!(player.anima(), 140);
		assert_eq!(player.rifts_sealed(), 1);
	}
}
