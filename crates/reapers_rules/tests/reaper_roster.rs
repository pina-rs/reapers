//! A player's roster of reapers.

mod support;

use reapers_rules::*;
use serde_json::Value;

fn now() -> Timestamp {
	Timestamp::utc(2026, 10, 5, 0, 0, 0)
}

fn husk() -> Nft {
	Nft::parse("degenApes:1").unwrap()
}

const CUT: HuntOutcome = HuntOutcome::absorbed_with(
	ReapGrade::Clean,
	false,
	100,
	HuntDeeds {
		holds: 0,
		parries: 0,
		max_cadence: 0,
	},
);

fn pair() -> PlayerState {
	PlayerState::initial()
		.take_oath(11)
		.unwrap()
		.bind_reaper(ReaperState::bound(
			"verified-asset",
			ReaperGenome::shuffle(22),
		))
		.unwrap()
}

/// Asserts that `result` was refused for `refusal`.
#[track_caller]
fn refused<T: std::fmt::Debug>(result: RuleResult<T>, refusal: Refusal) {
	assert_eq!(result.unwrap_err().refusal, refusal);
}

/// `reaper` with `xp` and `outfit` in place of its own, refused when it is a
/// record no reaper could have.
fn copy_with(reaper: &ReaperState, xp: i64, outfit: Outfit) -> Result<ReaperState, FormatError> {
	ReaperState::new(
		reaper.id().to_owned(),
		*reaper.genome(),
		xp,
		reaper.weapons().to_vec(),
		reaper.outfits().to_vec(),
		reaper.weapon(),
		outfit,
		reaper.codex().to_vec(),
		reaper.hunts_lost(),
		reaper.rifts_sealed(),
		*reaper.memory(),
		reaper.art(),
	)
}

/// `json` with `changes` written over its top-level fields.
fn merged(json: &Value, changes: &Value) -> Value {
	let mut json = json.clone();
	let fields = json.as_object_mut().unwrap();

	for (key, value) in changes.as_object().unwrap() {
		fields.insert(key.clone(), value.clone());
	}

	json
}

#[test]
fn hunt_experience_and_codex_belong_only_to_the_reaper_that_hunted() {
	let original = pair();
	let hunted = original
		.select_reaper("verified-asset")
		.unwrap()
		.acquire(husk())
		.unwrap()
		.release(husk(), 8, now())
		.unwrap()
		.settle_hunt(1, &CUT, now().plus_seconds(60), 3)
		.unwrap();
	let starter = hunted.select_reaper(ReaperState::OATH_ID).unwrap();
	assert!(hunted.xp() > 0);
	let [entry] = hunted.codex() else {
		panic!("expected one absorption");
	};
	assert_eq!(entry.nft, husk());
	assert_eq!(starter.xp(), 0);
	assert_eq!(starter.codex(), []);
	assert_eq!(starter.anima(), hunted.anima());
	assert!(starter.holds(husk()));
	refused(starter.acquire(husk()), Refusal::AlreadyHeld);
}

#[test]
fn cannot_switch_away_from_loose_hunts_or_select_an_unowned_reaper() {
	let loose = pair()
		.acquire(husk())
		.unwrap()
		.release(husk(), 1, now())
		.unwrap();
	refused(loose.select_reaper("verified-asset"), Refusal::ReaperBusy);
	assert_eq!(
		loose
			.select_reaper(ReaperState::OATH_ID)
			.unwrap()
			.active_reaper_id(),
		Some(ReaperState::OATH_ID)
	);
	refused(pair().select_reaper("not-mine"), Refusal::NotOwned);
}

#[test]
fn tradable_reapers_cannot_earn_growth_from_unverified_rifts() {
	refused(
		pair()
			.select_reaper("verified-asset")
			.unwrap()
			.entrant_for(RiftSite::ALL[0]),
		Refusal::RiftUnproven,
	);
}

#[test]
fn roster_capacity_is_enforced_by_the_shared_rule() {
	let mut player = PlayerState::initial().take_oath(11).unwrap();
	for index in 0..PlayerState::MAX_NFT_REAPERS {
		player = player
			.bind_reaper(ReaperState::bound(
				format!("asset-{index}"),
				ReaperGenome::shuffle(index as i64),
			))
			.unwrap();
	}
	refused(
		player.bind_reaper(ReaperState::bound("overflow", ReaperGenome::shuffle(80))),
		Refusal::RosterFull,
	);
}

#[test]
fn reaper_progression_is_immutable_and_equipped_gear_must_be_owned() {
	let pair = pair();
	let reaper = pair.reapers().first().unwrap();
	// The collections cannot be emptied from outside: they are lent out only
	// as shared slices, which cannot be changed at all.
	let _: &[Weapon] = reaper.weapons();
	let _: &[Outfit] = reaper.outfits();
	let _: &[Absorption] = reaper.codex();
	assert!(copy_with(reaper, -1, reaper.outfit()).is_err());
	assert!(copy_with(reaper, reaper.xp(), Outfit::Hollowed).is_err());
}

#[test]
fn rift_settlement_credits_the_entrant_even_after_selection_changes() {
	let result = pair()
		.select_reaper("verified-asset")
		.unwrap()
		.settle_rift(400, 40, Some(ReaperState::OATH_ID))
		.unwrap();
	assert_eq!(result.xp(), 0);
	assert_eq!(result.rifts_sealed(), 0);
	let entrant = result.select_reaper(ReaperState::OATH_ID).unwrap();
	assert_eq!(entrant.xp(), 400);
	assert_eq!(entrant.rifts_sealed(), 1);
	assert_eq!(entrant.anima(), 40);
}

#[test]
fn losing_ownership_returns_unfinished_husks_without_growth() {
	let owned = pair()
		.select_reaper("verified-asset")
		.unwrap()
		.acquire(husk())
		.unwrap()
		.release(husk(), 1, now())
		.unwrap();
	let refreshed = owned.reconcile_reapers(Vec::new()).unwrap();
	assert_eq!(refreshed.active_reaper_id(), Some(ReaperState::OATH_ID));
	assert_eq!(refreshed.vault(), [husk()]);
	assert_eq!(refreshed.sightings(), []);
	assert_eq!(refreshed.xp(), 0);
	assert_eq!(refreshed.codex(), []);
}

#[test]
fn nft_equipment_and_experience_survive_roster_serialization_independently() {
	let original = pair()
		.select_reaper("verified-asset")
		.unwrap()
		.settle_rift(Progression::xp_for_level(20), 10000, None)
		.unwrap()
		.buy_outfit(Outfit::Scorched)
		.unwrap()
		.wear(Outfit::Scorched)
		.unwrap();
	let restored: PlayerState =
		serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
	assert_eq!(
		serde_json::to_value(&restored).unwrap(),
		serde_json::to_value(&original).unwrap()
	);
	assert_eq!(
		restored
			.select_reaper(ReaperState::OATH_ID)
			.unwrap()
			.outfit(),
		Outfit::Shrouded
	);
	assert_eq!(restored.outfit(), Outfit::Scorched);
}

#[test]
fn schema_two_preserves_a_grown_reaper_equipment_currency_and_codex() {
	let grown = pair()
		.acquire(husk())
		.unwrap()
		.release(husk(), 1, now())
		.unwrap()
		.settle_hunt(1, &CUT, now().plus_seconds(60), 5)
		.unwrap();
	let json = serde_json::to_value(&grown).unwrap();
	let mut legacy = merged(
		&merged(
			&json,
			&serde_json::to_value(grown.active_reaper().unwrap()).unwrap(),
		),
		&serde_json::json!({"schema": 2}),
	);
	let fields = legacy.as_object_mut().unwrap();
	fields.remove("reapers");
	fields.remove("activeReaperId");
	fields.remove("id");
	let restored: PlayerState = serde_json::from_value(legacy).unwrap();
	assert_eq!(restored.reapers().len(), 1);
	assert_eq!(restored.genome(), grown.genome());
	assert_eq!(restored.xp(), grown.xp());
	assert_eq!(restored.weapon(), grown.weapon());
	let ([restored_entry], [grown_entry]) = (restored.codex(), grown.codex()) else {
		panic!("expected one absorption each");
	};
	assert_eq!(
		serde_json::to_value(restored_entry).unwrap(),
		serde_json::to_value(grown_entry).unwrap()
	);
	assert_eq!(restored.lamports(), grown.lamports());
	assert_eq!(restored.anima(), grown.anima());
	assert_eq!(serde_json::to_value(&restored).unwrap()["schema"], 3);
}

#[test]
fn unbound_legacy_save_still_opens_without_inventing_a_reaper() {
	let legacy = merged(
		&serde_json::to_value(PlayerState::initial()).unwrap(),
		&serde_json::json!({"schema": 2, "genome": null}),
	);
	assert!(
		!serde_json::from_value::<PlayerState>(legacy)
			.unwrap()
			.oath_taken()
	);
}

#[test]
fn duplicate_seats_and_an_invalid_selection_cannot_be_read() {
	let json = serde_json::to_value(pair()).unwrap();
	assert!(
		serde_json::from_value::<PlayerState>(merged(
			&json,
			&serde_json::json!({"activeReaperId": "absent"})
		))
		.is_err()
	);
	let seat = serde_json::to_value(pair().reapers().first().unwrap()).unwrap();
	assert!(
		serde_json::from_value::<PlayerState>(merged(
			&json,
			&serde_json::json!({"reapers": [seat.clone(), seat]})
		))
		.is_err()
	);
}

#[test]
fn preview_birth_trait_edits_keep_names_valid_and_reject_out_of_range_parts() {
	let original = ReaperGenome::shuffle(11);
	for calling in ReaperClass::ALL {
		let tried = original
			.with(GenomeChanges {
				reaper_class: Some(calling),
				head: Some(2),
				tatter: Some(1000),
				..Default::default()
			})
			.unwrap();
		assert_ne!(tried.name(), "");
		assert_eq!(ReaperGenome::parse(&tried.code()).unwrap(), tried);
	}
	assert!(
		original
			.with(GenomeChanges {
				head: Some(9),
				..Default::default()
			})
			.is_err()
	);
	assert_eq!(original, ReaperGenome::shuffle(11));
}
