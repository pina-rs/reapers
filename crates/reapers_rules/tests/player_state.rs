//! The Rust side of `reapers_shared/test/player_state_test.dart`.

mod support;

use reapers_rules::*;
use serde_json::Value;
use serde_json::json;

fn now() -> Timestamp {
	Timestamp::utc(2026, 10, 1, 12, 0, 0)
}

fn later() -> Timestamp {
	now().plus_seconds(60)
}

const SAINT: Nft = Nft::new(NftCollection::DegenApes, 1);

const ABSORBED: HuntOutcome = HuntOutcome::absorbed_with(
	ReapGrade::Clean,
	false,
	1800,
	HuntDeeds {
		holds: 0,
		parries: 0,
		max_cadence: 0,
	},
);

/// Asserts that `result` was refused for `refusal`.
#[track_caller]
fn refuses<T: std::fmt::Debug>(result: RuleResult<T>, refusal: Refusal) {
	assert_eq!(result.unwrap_err().refusal, refusal);
}

/// The first seed that rolls `feeding`, so a test can ask for the luck it
/// is about.
fn seed_for(feeding: Feeding) -> i64 {
	(0..10000)
		.find(|&seed| Feeding::roll(seed) == feeding)
		.unwrap()
}

fn fair() -> i64 {
	seed_for(Feeding::Fair)
}

fn oathbound() -> PlayerState {
	PlayerState::initial().take_oath(11).unwrap()
}

/// `state` as a save, with `changes` written over its top-level fields.
fn save_with(state: &PlayerState, changes: Value) -> PlayerState {
	let mut json = serde_json::to_value(state).unwrap();
	let fields = json.as_object_mut().unwrap();
	let Value::Object(changes) = changes else {
		panic!("changes are an object");
	};

	fields.extend(changes);

	serde_json::from_value(json).unwrap()
}

/// `reaper` with `xp` experience, as Dart's `copyWith(xp:)`.
fn with_xp(reaper: &ReaperState, xp: i64) -> ReaperState {
	ReaperState::new(
		reaper.id().to_owned(),
		*reaper.genome(),
		xp,
		reaper.weapons().to_vec(),
		reaper.outfits().to_vec(),
		reaper.weapon(),
		reaper.outfit(),
		reaper.codex().to_vec(),
		reaper.hunts_lost(),
		reaper.rifts_sealed(),
		*reaper.memory(),
		reaper.art(),
	)
	.unwrap()
}

/// The record of the shared bot hunting `state`'s one loose husk.
fn played(state: &PlayerState) -> HuntLog {
	let [sighting] = state.sightings() else {
		panic!("expected one sighting");
	};
	let mut sim = HuntSim::open(
		sighting.seed,
		sighting.nft.collection.vessel(),
		sighting.nft.rarity().ward(),
		&HuntStats::derive(&state.loadout(), 1.0).unwrap(),
	);
	let mut bot = HuntBot::careful(&sim);

	bot.play(&mut sim);

	HuntLog::new(bot.recorded)
}

/// The oathbound player with the saint released into the veil.
fn released() -> PlayerState {
	oathbound()
		.acquire(SAINT)
		.unwrap()
		.release(SAINT, 5, now())
		.unwrap()
}

mod the_oath {
	use super::*;

	#[test]
	fn a_new_player_has_no_reaper_and_plays_with_the_borrowed_one() {
		let state = PlayerState::initial();

		assert!(!state.oath_taken());
		assert_eq!(state.genome(), ReaperGenome::BORROWED);
		assert_eq!(state.loadout(), HuntLoadout::BORROWED);
		assert_eq!(state.level(), 1);
	}

	#[test]
	fn binds_the_reaper_the_seed_draws_and_arms_it_for_its_class() {
		let state = oathbound();
		let genome = ReaperGenome::shuffle(11);

		assert_eq!(state.genome(), genome);
		assert_eq!(
			state.weapon(),
			Some(Weapon::new(genome.reaper_class.weapon(), Temper::Shrouded))
		);
		assert_eq!(state.weapons(), [state.weapon().unwrap()]);
		assert_eq!(state.loadout().reaper_class, genome.reaper_class);
	}

	#[test]
	fn is_taken_once() {
		refuses(oathbound().take_oath(12), Refusal::OathAlreadyTaken);
	}

	#[test]
	fn is_needed_before_the_market_opens() {
		refuses(PlayerState::initial().acquire(SAINT), Refusal::OathRequired);
	}
}

mod the_vault {
	use super::*;

	#[test]
	fn buying_an_nft_spends_sol_and_holds_it() {
		let state = oathbound().acquire(SAINT).unwrap();

		assert_eq!(state.vault(), [SAINT]);
		assert_eq!(
			state.lamports(),
			PlayerState::STARTING_LAMPORTS - SAINT.rarity().lamports()
		);
	}

	#[test]
	fn an_nft_is_held_once() {
		refuses(
			oathbound().acquire(SAINT).unwrap().acquire(SAINT),
			Refusal::AlreadyHeld,
		);
	}

	#[test]
	fn a_wallet_that_cannot_cover_the_price_buys_nothing() {
		let short = save_with(
			&oathbound(),
			json!({"lamports": SAINT.rarity().lamports() - 1}),
		);

		refuses(short.acquire(SAINT), Refusal::InsufficientFunds);
	}

	#[test]
	fn the_starting_wallet_covers_the_whole_stand_in_market() {
		let mut state = oathbound();

		for nft in market_listings() {
			state = state.acquire(nft).unwrap();
		}

		assert_eq!(state.vault().len(), market_listings().len());
		assert!(state.lamports() >= 0);
	}

	#[test]
	fn every_listing_has_a_price_and_an_image() {
		assert_eq!(market_listings().len(), NftCollection::ALL.len() * 8);

		for nft in market_listings() {
			assert!(nft.rarity().lamports() > 0, "{}", nft.id());
			assert_eq!(Nft::parse(&nft.id()).unwrap(), nft);
			assert_eq!(nft.mock_mint().len(), 44);
		}
	}
}

mod releasing {
	use std::collections::HashSet;

	use super::*;

	#[test]
	fn moves_an_nft_from_the_vault_into_the_veil() {
		let state = released();
		let [sighting] = state.sightings() else {
			panic!("expected one sighting");
		};

		assert_eq!(state.vault(), []);
		assert_eq!(sighting.nft, SAINT);
		assert!(!sighting.has_surfaced(now()));
		assert!(
			(PlayerState::SURFACE_MIN_SECONDS..=PlayerState::SURFACE_MAX_SECONDS)
				.contains(&(sighting.surfaces_at.micros_since(now()) / 1_000_000))
		);
	}

	#[test]
	fn needs_the_nft_to_be_in_the_vault() {
		refuses(oathbound().release(SAINT, 5, now()), Refusal::NotInVault);
	}

	#[test]
	fn the_veil_holds_only_so_many_husks() {
		let mut state = oathbound();

		for serial in 1..=PlayerState::MAX_SIGHTINGS + 1 {
			state = state
				.acquire(Nft::new(NftCollection::DegenApes, serial))
				.unwrap();
		}

		for serial in 1..=PlayerState::MAX_SIGHTINGS {
			state = state
				.release(
					Nft::new(NftCollection::DegenApes, serial),
					serial as i64,
					now(),
				)
				.unwrap();
		}

		refuses(
			state.release(Nft::new(NftCollection::DegenApes, 4), 9, now()),
			Refusal::VeilCrowded,
		);
		assert_eq!(
			state
				.sightings()
				.iter()
				.map(|sighting| sighting.id)
				.collect::<HashSet<_>>(),
			HashSet::from([1, 2, 3])
		);
	}

	#[test]
	fn a_husk_cannot_be_hunted_before_it_surfaces() {
		let state = released();

		refuses(state.huntable(1, now()), Refusal::NotSurfaced);
		assert_eq!(state.huntable(1, later()).unwrap().nft, SAINT);
		refuses(state.huntable(2, later()), Refusal::UnknownSighting);
	}
}

mod settling_a_hunt {
	use super::*;

	/// `x` rounded half away from zero, as Dart's `round`.
	fn round(x: f64) -> i64 {
		x.round() as i64
	}

	#[test]
	fn an_absorbed_husk_pays_out_and_enters_the_codex() {
		let state = released()
			.settle_hunt(1, &ABSORBED, later(), fair())
			.unwrap();
		let [entry] = state.codex() else {
			panic!("expected one absorption");
		};

		assert_eq!(state.sightings(), []);
		assert_eq!(entry.nft, SAINT);
		assert_eq!(entry.grade, ReapGrade::Clean);
		assert_eq!(state.xp(), round(Rarity::Common.xp() as f64 * 1.25));
		assert_eq!(state.anima(), round(Rarity::Common.anima() as f64 * 1.25));
		assert!(state.holds(SAINT));
	}

	#[test]
	fn an_ambush_and_a_perfect_reap_both_pay_more() {
		let best = HuntOutcome::absorbed_with(ReapGrade::Perfect, true, 1800, HuntDeeds::default());

		assert_eq!(
			best.xp_for(Rarity::Rare, Feeding::Fair),
			round(Rarity::Rare.xp() as f64 * 1.5 * HuntRules::AMBUSH_REWARD)
		);
		assert!(
			best.xp_for(Rarity::Rare, Feeding::Fair) > ABSORBED.xp_for(Rarity::Rare, Feeding::Fair)
		);
	}

	#[test]
	fn how_well_the_reaper_fed_scales_the_experience_and_nothing_else() {
		for feeding in Feeding::ALL {
			let state = released()
				.settle_hunt(1, &ABSORBED, later(), seed_for(feeding))
				.unwrap();
			let [entry] = state.codex() else {
				panic!("expected one absorption");
			};

			assert_eq!(entry.feeding, feeding);
			assert_eq!(
				state.xp(),
				round(Rarity::Common.xp() as f64 * ReapGrade::Clean.reward() * feeding.share())
			);
			assert_eq!(state.anima(), round(Rarity::Common.anima() as f64 * 1.25));
			assert_eq!(entry.xp, state.xp());
			assert_eq!(entry.anima, state.anima());
			assert_eq!(entry.nonce, seed_for(feeding));
			assert_eq!(entry.signature, None);
		}
	}

	#[test]
	fn the_same_seed_always_feeds_the_same() {
		for seed in 0..200 {
			assert_eq!(Feeding::roll(seed), Feeding::roll(seed));
		}
	}

	#[test]
	fn feedings_come_up_about_as_often_as_their_weights_say() {
		const ROLLS: i64 = 20000;
		let mut counts = [0_i64; Feeding::ALL.len()];

		for seed in 0..ROLLS {
			counts[Feeding::roll(seed).index()] += 1;
		}

		for feeding in Feeding::ALL {
			let share = counts[feeding.index()] as f64 / ROLLS as f64;

			assert!(
				(share - f64::from(feeding.weight()) / 100.0).abs() <= 0.02,
				"{}: {share}",
				feeding.name()
			);
		}
	}

	#[test]
	fn on_average_the_roll_is_a_small_gift_never_a_tax() {
		assert!(Feeding::expected_share() > 1.0);
		assert!(Feeding::expected_share() < 1.1);
		assert_eq!(
			Feeding::ALL.into_iter().map(Feeding::weight).sum::<u32>(),
			100
		);
	}

	#[test]
	fn a_judged_hunt_reports_how_well_the_reaper_fed() {
		let state = released();
		let rich = seed_for(Feeding::Rich);
		let settlement = state
			.judge_hunt(1, &played(&state), later(), rich, &Judging::default())
			.unwrap();

		assert_eq!(settlement.feeding, Some(Feeding::Rich));
		let [entry] = settlement.state.codex() else {
			panic!("expected one absorption");
		};
		assert_eq!(settlement.xp, entry.xp);
		let restored: HuntSettlement =
			serde_json::from_value(serde_json::to_value(&settlement).unwrap()).unwrap();
		assert_eq!(restored.feeding, Some(Feeding::Rich));
	}

	#[test]
	fn a_signature_is_attached_to_the_absorption_it_signs() {
		let other = Nft::new(NftCollection::DegenApes, 2);
		let mut state = released()
			.settle_hunt(1, &ABSORBED, later(), fair())
			.unwrap()
			.acquire(other)
			.unwrap()
			.release(other, 6, now())
			.unwrap();

		state = state
			.settle_hunt(2, &ABSORBED, later(), fair())
			.unwrap()
			.sign_absorption(other, "c2lnbmVk");

		assert_eq!(state.codex().first().unwrap().signature, None);
		assert_eq!(
			state.codex().last().unwrap().signature.as_deref(),
			Some("c2lnbmVk")
		);
		let restored: PlayerState =
			serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
		assert_eq!(
			restored.codex().last().unwrap().signature.as_deref(),
			Some("c2lnbmVk")
		);
	}

	#[test]
	fn an_escaped_husk_stays_loose_and_pays_nothing() {
		let escaped = HuntOutcome::escaped_by(EscapeCause::Time, false, 4000, HuntDeeds::default());
		let state = released()
			.settle_hunt(1, &escaped, later(), fair())
			.unwrap();

		assert_eq!(state.sightings().len(), 1);
		assert_eq!(state.codex(), []);
		assert_eq!(state.xp(), 0);
		assert_eq!(state.hunts_lost(), 1);
	}

	#[test]
	fn an_absorbed_nft_cannot_be_bought_again() {
		let state = released()
			.settle_hunt(1, &ABSORBED, later(), fair())
			.unwrap();

		refuses(state.acquire(SAINT), Refusal::AlreadyHeld);
	}

	#[test]
	fn three_of_a_collection_complete_its_set_and_strengthen_strikes() {
		let mut state = oathbound();

		for serial in 1..=NftCollection::SET_SIZE {
			let nft = Nft::new(NftCollection::DegenApes, serial);

			state = state
				.acquire(nft)
				.unwrap()
				.release(nft, serial as i64, now())
				.unwrap()
				.settle_hunt(state.next_sighting_id(), &ABSORBED, later(), fair())
				.unwrap();
		}

		assert_eq!(state.completed_sets(), 1);
		assert_eq!(state.loadout().completed_sets, 1);
		assert!(
			HuntStats::derive(&state.loadout(), 1.0)
				.unwrap()
				.strike_damage
				> HuntStats::derive(
					&HuntLoadout::new(
						state.genome().reaper_class,
						state.level(),
						state.weapon().unwrap(),
					),
					1.0,
				)
				.unwrap()
				.strike_damage
		);
	}
}

mod judging_a_hunt_from_its_log {
	use super::*;

	#[test]
	fn a_played_hunt_is_replayed_and_settled() {
		let state = released();
		let settlement = state
			.judge_hunt(1, &played(&state), later(), fair(), &Judging::default())
			.unwrap();
		let [entry] = settlement.state.codex() else {
			panic!("expected one absorption");
		};

		assert!(settlement.outcome.absorbed());
		assert_eq!(entry.nft, SAINT);
		assert_eq!(settlement.state.xp(), settlement.xp);
		assert!(settlement.xp > 0);
	}

	#[test]
	fn the_lenient_replay_the_server_runs_agrees_with_the_phone() {
		let state = released();
		let log = played(&state);
		let strict = state
			.judge_hunt(1, &log, later(), fair(), &Judging::default())
			.unwrap();
		let lenient = state
			.judge_hunt(
				1,
				&log,
				later(),
				fair(),
				&Judging {
					leniency: 1.03,
					..Judging::default()
				},
			)
			.unwrap();

		assert_eq!(lenient.outcome.absorbed(), strict.outcome.absorbed());
	}

	#[test]
	fn a_log_that_stops_before_the_hunt_ends_is_refused() {
		let state = released();
		let log = played(&state);
		let cut = HuntLog::new(log.inputs[..log.inputs.len() / 3].to_vec());

		refuses(
			state.judge_hunt(1, &cut, later(), fair(), &Judging::default()),
			Refusal::HuntUnproven,
		);
	}

	#[test]
	fn an_empty_log_wins_nothing() {
		refuses(
			released().judge_hunt(
				1,
				&HuntLog::new(Vec::new()),
				later(),
				fair(),
				&Judging::default(),
			),
			Refusal::HuntUnproven,
		);
	}

	#[test]
	fn a_log_too_long_to_be_one_hunt_is_refused_before_it_is_replayed() {
		let still = HuntInput::new(Aim::AHEAD, 0);

		refuses(
			released().judge_hunt(
				1,
				&HuntLog::new(vec![still; HuntLog::MAX_TICKS + 1]),
				later(),
				fair(),
				&Judging::default(),
			),
			Refusal::HuntUnproven,
		);
	}
}

mod the_armory {
	use super::*;

	/// An oathbound player with plenty of anima and a reaper at `level`.
	fn rich_at(level: i64) -> PlayerState {
		let reaper = with_xp(
			oathbound().active_reaper().unwrap(),
			Progression::xp_for_level(level),
		);

		save_with(
			&oathbound(),
			json!({"anima": 100_000, "reapers": [serde_json::to_value(&reaper).unwrap()]}),
		)
	}

	fn rich() -> PlayerState {
		rich_at(30)
	}

	#[test]
	fn a_better_temper_costs_anima_and_can_be_wielded() {
		let start = rich();
		let weapon = Weapon::new(start.weapon().unwrap().kind, Temper::Sworn);
		let state = start.buy_weapon(weapon).unwrap().wield(weapon).unwrap();

		assert_eq!(state.weapon(), Some(weapon));
		assert_eq!(state.anima(), start.anima() - Temper::Sworn.price());
		assert!(
			HuntStats::derive(&state.loadout(), 1.0)
				.unwrap()
				.strike_damage
				> HuntStats::derive(&start.loadout(), 1.0)
					.unwrap()
					.strike_damage
		);
	}

	#[test]
	fn another_classs_weapon_can_be_bought_at_a_price() {
		let start = rich();
		let other = WeaponKind::ALL
			.into_iter()
			.find(|&kind| kind != start.genome().reaper_class.weapon())
			.unwrap();
		let state = start
			.buy_weapon(Weapon::new(other, Temper::Shrouded))
			.unwrap();

		assert_eq!(state.anima(), start.anima() - Weapon::CROSS_CLASS_PRICE);
	}

	#[test]
	fn refuses_what_the_reaper_cannot_afford_reach_or_already_has() {
		let poor = oathbound();
		let sworn = Weapon::new(poor.weapon().unwrap().kind, Temper::Sworn);

		refuses(poor.buy_weapon(sworn), Refusal::LevelTooLow);
		let reaper = with_xp(
			poor.active_reaper().unwrap(),
			Progression::xp_for_level(Temper::Sworn.level_required()),
		);
		refuses(
			save_with(
				&poor,
				json!({"reapers": [serde_json::to_value(&reaper).unwrap()]}),
			)
			.buy_weapon(sworn),
			Refusal::InsufficientAnima,
		);
		refuses(
			rich().buy_weapon(poor.weapon().unwrap()),
			Refusal::AlreadyOwned,
		);
		refuses(poor.wield(sworn), Refusal::NotOwned);
	}

	#[test]
	fn outfits_are_bought_in_the_same_way_and_only_change_the_look() {
		let start = rich();
		let state = start
			.buy_outfit(Outfit::Scorched)
			.unwrap()
			.wear(Outfit::Scorched)
			.unwrap();

		assert_eq!(state.outfit(), Outfit::Scorched);
		assert_eq!(state.anima(), start.anima() - Outfit::Scorched.price());
		assert_eq!(
			HuntStats::derive(&state.loadout(), 1.0)
				.unwrap()
				.strike_damage
				.to_bits(),
			HuntStats::derive(&start.loadout(), 1.0)
				.unwrap()
				.strike_damage
				.to_bits()
		);
		refuses(start.wear(Outfit::Hollowed), Refusal::NotOwned);
		refuses(
			rich_at(2).buy_outfit(Outfit::Hollowed),
			Refusal::LevelTooLow,
		);
	}

	#[test]
	fn every_temper_and_outfit_costs_more_than_the_one_before() {
		for values in [
			Temper::ALL.map(Temper::price).to_vec(),
			Temper::ALL.map(Temper::level_required).to_vec(),
			Outfit::ALL.map(Outfit::price).to_vec(),
			Outfit::ALL.map(Outfit::level_required).to_vec(),
		] {
			let mut sorted = values.clone();
			sorted.sort_unstable();

			assert_eq!(values, sorted);
		}
	}
}

mod json {
	use super::*;

	#[test]
	fn a_played_state_survives_a_round_trip_through_text() {
		let state = released()
			.settle_hunt(1, &ABSORBED, later(), fair())
			.unwrap()
			.acquire(Nft::new(NftCollection::Solarians, 8))
			.unwrap();
		let text = serde_json::to_string(&state).unwrap();
		let restored: PlayerState = serde_json::from_str(&text).unwrap();

		assert_eq!(serde_json::to_string(&restored).unwrap(), text);
		assert_eq!(restored.genome(), state.genome());
		let [entry] = restored.codex() else {
			panic!("expected one absorption");
		};
		assert_eq!(entry.at, later());
	}

	#[test]
	fn an_unknown_schema_is_refused_rather_than_guessed_at() {
		let mut json = serde_json::to_value(PlayerState::initial()).unwrap();
		json["schema"] = json!(99);

		assert!(serde_json::from_value::<PlayerState>(json).is_err());
	}
}
