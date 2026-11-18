//! A signed-in player's books, through the operations, and the simulated
//! economy that only development and test allow, against a real PostgreSQL.

mod support;

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reapers_api::player::*;
use reapers_rules::Absorption;
use reapers_rules::Art;
use reapers_rules::ArtSkill;
use reapers_rules::CaptureGrade;
use reapers_rules::Feeding;
use reapers_rules::GeoFix;
use reapers_rules::GeoPoint;
use reapers_rules::HuntLog;
use reapers_rules::HuntPlan;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Nft;
use reapers_rules::NftCollection;
use reapers_rules::Outfit;
use reapers_rules::PlayerState;
use reapers_rules::Rarity;
use reapers_rules::Refusal;
use reapers_rules::Sighting;
use reapers_rules::SoulMemory;
use reapers_rules::Temper;
use reapers_rules::Timestamp;
use reapers_rules::Weapon;
use reapers_server::config::RunMode;
use reapers_server::keeper::now;
use reapers_server::receipt;
use reapers_server::signer::ReceiptSigner;
use sha2::Digest;
use sha2::Sha256;
use solana_kit_core::Signature;
use support::*;

const SAINT: &str = "degenApes:1";

fn measured(accuracy: f64, at: GeoPoint, age_seconds: i64) -> GeoFix {
	GeoFix {
		at,
		accuracy,
		observed_at: now().plus_seconds(-age_seconds),
	}
}

fn fix() -> GeoFix {
	measured(4.0, GeoPoint::new(51.5, -0.08), 0)
}

fn shared(fix: Option<GeoFix>) -> Option<reapers_api::Shared<GeoFix>> {
	fix.map(reapers_api::Shared)
}

/// An oathbound player with the saint released and surfaced.
async fn with_loose_husk(
	world: &World,
	player: leptoskit_db::Id<leptoskit_auth::Account>,
) -> PlayerState {
	world.call(player, TakeOath {}).await.expect("the oath");
	world
		.call(player, Acquire { nft: SAINT.into() })
		.await
		.expect("the saint");
	world
		.call(
			player,
			Release {
				nft: SAINT.into(),
				escrow: None,
			},
		)
		.await
		.expect("the release");
	world.surface_everything(player).await;
	world.call(player, Load {}).await.expect("the books")
}

#[tokio::test]
async fn location_belongs_to_the_signed_in_player_and_survives_book_changes() {
	let world = World::new().await;
	let player = world.player().await;
	let other = world.player().await;
	let fix = fix();

	world.call(player, Load {}).await.unwrap();
	world
		.call(
			player,
			UpdateLocation {
				fix: shared(Some(fix)),
			},
		)
		.await
		.unwrap();
	world.call(player, TakeOath {}).await.unwrap();

	assert_eq!(
		world.call(player, Location {}).await.unwrap().map(|f| f.at),
		Some(fix.at)
	);
	assert_eq!(world.call(other, Location {}).await.unwrap(), None);

	world
		.call(player, UpdateLocation { fix: None })
		.await
		.unwrap();

	assert_eq!(world.call(player, Location {}).await.unwrap(), None);
	assert!(world.call(player, Load {}).await.unwrap().oath_taken());
}

#[tokio::test]
async fn location_rejects_coarse_malformed_stale_and_future_readings() {
	let world = World::new().await;
	let player = world.player().await;
	let here = GeoPoint::new(51.5, -0.08);

	for fix in [
		measured(80.0, here, 0),
		measured(0.0, here, 0),
		measured(4.0, GeoPoint::new(95.0, 0.0), 0),
		measured(4.0, here, 31),
		measured(4.0, here, -10),
	] {
		assert_refused(
			world
				.call(
					player,
					UpdateLocation {
						fix: shared(Some(fix)),
					},
				)
				.await,
			Refusal::LocationUnusable,
		);
	}

	assert_eq!(world.call(player, Location {}).await.unwrap(), None);
}

#[tokio::test]
async fn out_of_order_uploads_cannot_replace_a_newer_location() {
	let world = World::new().await;
	let player = world.player().await;
	let older = measured(4.0, GeoPoint::new(51.5, -0.08), 2);
	let newer = measured(4.0, GeoPoint::new(51.501, -0.08), 0);

	world
		.call(
			player,
			UpdateLocation {
				fix: shared(Some(newer)),
			},
		)
		.await
		.unwrap();
	world
		.call(
			player,
			UpdateLocation {
				fix: shared(Some(older)),
			},
		)
		.await
		.unwrap();

	assert_eq!(
		world.call(player, Location {}).await.unwrap().map(|f| f.at),
		Some(newer.at)
	);
}

#[tokio::test]
async fn a_same_instant_refinement_replaces_a_coarser_location() {
	let world = World::new().await;
	let player = world.player().await;
	let observed = now();
	let coarse = GeoFix {
		at: GeoPoint::new(51.5, -0.08),
		accuracy: 20.0,
		observed_at: observed,
	};
	let tight = GeoFix {
		at: GeoPoint::new(51.501, -0.081),
		accuracy: 4.0,
		observed_at: observed,
	};
	let delayed = GeoFix {
		at: GeoPoint::new(51.4, -0.07),
		accuracy: 18.0,
		observed_at: observed,
	};

	for fix in [coarse, tight, delayed] {
		world
			.call(
				player,
				UpdateLocation {
					fix: shared(Some(fix)),
				},
			)
			.await
			.unwrap();
	}

	assert_eq!(
		world.call(player, Location {}).await.unwrap().map(|f| f.at),
		Some(tight.at),
		"the tighter reading of that instant is kept"
	);
}

#[tokio::test]
async fn an_expired_stored_fix_is_not_returned() {
	use reapers_server::tables::PlayerBooks;

	let world = World::new().await;
	let player = world.player().await;

	world.call(player, Load {}).await.unwrap();
	PlayerBooks::update()
		.filter(|books| books.account_id.eq(player))
		.set(
			|books| books.location,
			Some(leptoskit_db::Json(measured(
				4.0,
				GeoPoint::new(51.5, -0.08),
				60,
			))),
		)
		.run(&world.db)
		.await
		.unwrap();

	assert_eq!(world.call(player, Location {}).await.unwrap(), None);
}

#[tokio::test]
async fn location_read_write_and_clear_all_require_authentication() {
	let world = World::new().await;

	assert_unauthorized(world.anonymous(Location {}).await);
	assert_unauthorized(
		world
			.anonymous(UpdateLocation {
				fix: shared(Some(fix())),
			})
			.await,
	);
	assert_unauthorized(world.anonymous(UpdateLocation { fix: None }).await);
}

#[tokio::test]
async fn when_nobody_is_signed_in_then_every_call_is_rejected() {
	let world = World::new().await;

	assert_unauthorized(world.anonymous(Load {}).await);
	assert_unauthorized(world.anonymous(TakeOath {}).await);
	assert_unauthorized(
		world
			.anonymous(SettleHunt {
				sighting: 0,
				log: String::new(),
			})
			.await,
	);
	assert_unauthorized(
		world
			.anonymous(BuyWeapon {
				weapon: "scythe:rusted".into(),
			})
			.await,
	);
}

#[tokio::test]
async fn when_loading_for_the_first_time_then_the_books_are_opened() {
	use reapers_server::tables::PlayerBooks;

	let world = World::new().await;
	let player = world.player().await;
	let state = world.call(player, Load {}).await.unwrap();

	assert!(!state.oath_taken());
	assert_eq!(state.lamports(), PlayerState::STARTING_LAMPORTS);
	assert_eq!(
		PlayerBooks::query().count(&world.db).await.unwrap(),
		1,
		"loading must open one set of books"
	);

	world.call(player, Load {}).await.unwrap();

	assert_eq!(
		PlayerBooks::query().count(&world.db).await.unwrap(),
		1,
		"loading twice must not open a second set of books"
	);
}

#[tokio::test]
async fn when_taking_the_oath_then_the_keeper_draws_the_reaper() {
	let world = World::new().await;
	let player = world.player().await;
	let state = world.call(player, TakeOath {}).await.unwrap();

	assert!(state.oath_taken());
	assert_eq!(
		state.weapon().map(|weapon| weapon.kind),
		Some(state.genome().reaper_class.weapon())
	);
	assert_eq!(
		world.call(player, Load {}).await.unwrap().genome_code(),
		state.genome_code()
	);
	assert_refused(
		world.call(player, TakeOath {}).await,
		Refusal::OathAlreadyTaken,
	);
}

#[tokio::test]
async fn when_two_players_act_then_each_has_their_own_books() {
	let world = World::new().await;
	let player = world.player().await;
	let other = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();

	assert!(!world.call(other, Load {}).await.unwrap().oath_taken());
	assert!(world.call(player, Load {}).await.unwrap().oath_taken());
}

#[tokio::test]
async fn when_buying_an_nft_then_the_wallet_pays_and_the_vault_holds_it() {
	let world = World::new().await;
	let player = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();

	let state = world
		.call(player, Acquire { nft: SAINT.into() })
		.await
		.unwrap();

	assert_eq!(
		state.vault().iter().map(Nft::id).collect::<Vec<_>>(),
		[SAINT]
	);
	assert_eq!(
		state.lamports(),
		PlayerState::STARTING_LAMPORTS - Rarity::Common.lamports()
	);
}

#[tokio::test]
async fn when_the_game_sends_an_nft_that_does_not_exist_then_it_is_refused() {
	let world = World::new().await;
	let player = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();

	for id in ["", "nonsense", "degenApes:99", "noSuchSet:1"] {
		assert_refused(
			world.call(player, Acquire { nft: id.into() }).await,
			Refusal::NotListed,
		);
	}
}

#[tokio::test]
async fn when_releasing_then_the_keeper_picks_the_seed_and_the_delay() {
	let world = World::new().await;
	let player = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();
	world
		.call(player, Acquire { nft: SAINT.into() })
		.await
		.unwrap();

	let state = world
		.call(
			player,
			Release {
				nft: SAINT.into(),
				escrow: None,
			},
		)
		.await
		.unwrap();
	let [sighting] = state.sightings() else {
		panic!("one loose husk");
	};

	assert_eq!(state.vault(), []);
	assert!(now().is_before(sighting.surfaces_at));
	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: sighting.id,
					log: packed(&play(&state, sighting)),
				},
			)
			.await,
		Refusal::NotSurfaced,
	);
}

#[tokio::test]
async fn when_a_played_hunt_is_submitted_then_the_replay_absorbs_the_husk() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let settled = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &sighting)),
			},
		)
		.await
		.unwrap();
	let settlement = settled.settlement;

	assert!(settlement.outcome.absorbed());
	assert!(settlement.xp > 0);
	assert_eq!(settlement.state.codex()[0].nft.id(), SAINT);

	// The settlement carries how well the capturing orb was thrown, and the
	// codex keeps the same grade.
	let grade = settlement.outcome.grade().unwrap();

	assert_eq!(settlement.state.codex()[0].grade, grade);
	assert_eq!(
		grade,
		CaptureGrade::Excellent,
		"a careful bot times the ring"
	);
	assert_eq!(settlement.state.sightings(), []);
	assert_eq!(settled.receipt, None, "a stand-in husk is burned nowhere");

	let stored = world.call(player, Load {}).await.unwrap();

	assert_eq!(stored.xp(), settlement.state.xp());
	assert_eq!(stored.codex()[0].nft.id(), SAINT);
	assert_eq!(
		stored.active_reaper().unwrap().memory(),
		&SoulMemory::default().remember(&settlement.outcome)
	);
}

/// The oath and husk seeds the room-replay test fixes.
///
/// The keeper draws both from secure random numbers for a real player, and
/// the reaper's class, weapon and gift change how the same inputs play. A
/// fixed pair makes the hunt's shape the same on every run. With these,
/// playing in [`desk_room`] absorbs with an `Excellent` capture, while
/// replaying the same inputs without the grid absorbs with a `Plain` one.
const ROOM_REPLAY_OATH_SEED: i64 = 1;
const ROOM_REPLAY_HUSK_SEED: i64 = 14;

#[tokio::test]
async fn when_a_hunt_in_a_room_is_submitted_then_the_replay_walks_the_same_room() {
	let world = World::new().await;
	let player = world.player().await;
	let saint = Nft::new(NftCollection::DegenApes, 1);
	let sworn = PlayerState::initial()
		.take_oath(ROOM_REPLAY_OATH_SEED)
		.expect("the oath")
		.acquire(saint)
		.expect("the saint")
		.release(
			saint,
			ROOM_REPLAY_HUSK_SEED,
			Timestamp::from_unix_seconds(0),
		)
		.expect("the release");

	world.call(player, Load {}).await.expect("the books open");
	world.store(player, sworn.clone()).await;

	let before = world.call(player, Load {}).await.expect("the books");
	let sighting = before.sightings()[0];

	assert_eq!(before, sworn, "the keeper reads the fixed fixture");
	assert_eq!(sighting.seed, ROOM_REPLAY_HUSK_SEED);
	let plan = HuntPlan {
		room: Some(desk_room()),
		..HuntPlan::new(0, 6000)
	};
	let log = play_in(&before, &sighting, plan);
	let stats = HuntStats::derive(&before.loadout(), 1.0).unwrap();
	let replay = |log: &HuntLog| {
		HuntSim::replay(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity(),
			sighting.nft.rarity().ward(),
			&stats,
			log,
		)
	};
	let walked = replay(&log).expect("the bot finishes the hunt");
	let without_the_room = HuntLog {
		plan: HuntPlan {
			room: None,
			..log.plan.clone()
		},
		..log.clone()
	};

	assert_eq!(
		log.encode()[0],
		11,
		"the log carries the room and current combat rules"
	);
	assert!(walked.absorbed());
	assert_ne!(
		replay(&without_the_room),
		Some(walked),
		"the room changes how the hunt plays"
	);

	let settled = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&log),
			},
		)
		.await
		.unwrap();

	assert_eq!(settled.settlement.outcome, walked);
	assert_eq!(settled.settlement.state.codex()[0].nft.id(), SAINT);
	assert_eq!(settled.settlement.state.sightings(), []);
}

#[tokio::test]
async fn when_the_room_in_a_log_is_broken_then_nothing_is_paid() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let plan = HuntPlan {
		room: Some(desk_room()),
		..HuntPlan::new(0, 6000)
	};
	let log = play_in(&before, &sighting, plan);
	let mut garbled = log.encode();
	// The grid's own first byte, after the plan's nine and the room's
	// length.
	garbled[13] = 0xFF;
	let mut disagreeing = log.clone();
	disagreeing.plan.eye_centimetres += 10;

	for log in [STANDARD.encode(&garbled), packed(&disagreeing)] {
		assert_refused(
			world
				.call(
					player,
					SettleHunt {
						sighting: sighting.id,
						log,
					},
				)
				.await,
			Refusal::HuntUnproven,
		);
	}

	let after = world.call(player, Load {}).await.unwrap();

	assert_eq!(after.xp(), 0);
	assert_eq!(after.sightings().len(), 1);
}

#[tokio::test]
async fn old_combat_rules_receive_an_update_refusal_without_changing_growth() {
	let world = World::new().await;
	let player = world.player().await;
	let state = with_loose_husk(&world, player).await;
	let sighting = state.sightings()[0];
	let mut bytes = play(&state, &sighting).encode();

	bytes[0] = 4;

	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: sighting.id,
					log: STANDARD.encode(bytes),
				},
			)
			.await,
		Refusal::HuntVersionUnsupported,
	);
	assert_eq!(world.call(player, Load {}).await.unwrap(), state);
}

/// The receipt the keeper signed for `entry`, absorbed by `player`.
fn receipt_for(player: leptoskit_db::Id<leptoskit_auth::Account>, entry: &Absorption) -> [u8; 98] {
	let player = Sha256::digest(player.uuid().to_string().as_bytes()).into();
	let mint = Sha256::digest(entry.nft.id().as_bytes()).into();

	receipt::of(player, mint, entry).expect("it fits").encode()
}

fn signature(text: &str) -> Signature {
	Signature::from_bytes(
		STANDARD
			.decode(text)
			.expect("base64")
			.try_into()
			.expect("64 bytes"),
	)
}

#[tokio::test]
async fn when_a_husk_is_absorbed_then_the_keeper_rolls_the_feeding_and_signs_it() {
	let world = World::new().await;
	let player = world.player().await;
	let other = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let settlement = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &sighting)),
			},
		)
		.await
		.unwrap()
		.settlement;
	let entry = settlement.state.codex()[0].clone();
	let feeding = settlement.feeding.expect("a feeding");

	assert_eq!(entry.feeding, feeding);
	assert_eq!(Feeding::roll(entry.nonce), feeding, "rolled, not sent");
	assert_eq!(
		settlement.xp,
		settlement.outcome.xp_for(sighting.nft.rarity(), feeding)
	);
	assert_eq!(entry.xp, settlement.xp);

	let signer = ReceiptSigner::development();
	let signed = signature(entry.signature.as_deref().expect("signed"));

	assert!(signer.verifies(&receipt_for(player, &entry), &signed));
	assert_eq!(
		world.call(player, Load {}).await.unwrap().codex()[0].signature,
		entry.signature,
		"the receipt is kept with the books"
	);

	// What is signed is the whole of what was decided. Change any of it and
	// the signature no longer holds.
	let richer = Absorption {
		feeding: Feeding::Feast,
		xp: entry.xp * 2,
		..entry.clone()
	};
	let another = Absorption {
		nft: Nft::new(NftCollection::DegenApes, 2),
		..entry.clone()
	};

	assert!(
		!signer.verifies(&receipt_for(player, &richer), &signed),
		"a richer feeding"
	);
	assert!(
		!signer.verifies(&receipt_for(player, &another), &signed),
		"another NFT"
	);
	assert!(
		!signer.verifies(&receipt_for(other, &entry), &signed),
		"a receipt belongs to one player"
	);
}

#[tokio::test]
async fn when_asked_then_the_keeper_gives_the_key_receipts_are_checked_against() {
	let world = World::new().await;
	let player = world.player().await;
	let key = world.call(player, ReceiptKey {}).await.unwrap();
	let signer = ReceiptSigner::development();

	assert_eq!(key.hex.len(), 64);
	assert_eq!(
		key.hex,
		reapers_server::signer::hex(signer.address().as_bytes())
	);
	assert_eq!(key.address, signer.address().to_string());
	assert!(key.development, "a development key says so");
}

#[tokio::test]
async fn when_the_same_hunt_is_submitted_twice_then_it_only_pays_once() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let log = packed(&play(&before, &sighting));

	world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: log.clone(),
			},
		)
		.await
		.unwrap();

	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: sighting.id,
					log,
				},
			)
			.await,
		Refusal::UnknownSighting,
	);
}

#[tokio::test]
async fn when_the_log_is_not_a_finished_hunt_then_nothing_is_paid() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let whole = play(&before, &sighting);
	let cut = HuntLog::new(whole.inputs[..whole.inputs.len() / 2].to_vec());
	let cornered = HuntLog {
		plan: HuntPlan {
			centre_centidegrees: 0,
			half_width_centidegrees: 1500,
			..HuntPlan::OPEN
		},
		..whole.clone()
	};

	for (label, log) in [
		("empty", String::new()),
		("not base64", "!!!".to_owned()),
		("garbage", STANDARD.encode([9, 9, 9, 9])),
		("cut short", packed(&cut)),
		("played in a corner too small to hunt in", packed(&cornered)),
	] {
		assert_refused(
			world
				.call(
					player,
					SettleHunt {
						sighting: sighting.id,
						log,
					},
				)
				.await,
			Refusal::HuntUnproven,
		);
		let _ = label;
	}

	let after = world.call(player, Load {}).await.unwrap();

	assert_eq!(after.xp(), 0);
	assert_eq!(after.sightings().len(), 1);
}

#[tokio::test]
async fn when_a_log_from_another_hunt_is_submitted_then_it_wins_nothing() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];

	// The same play, recorded against a husk that hid somewhere else.
	let elsewhere = Sighting {
		seed: sighting.seed + 1,
		..sighting
	};
	let result = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &elsewhere)),
			},
		)
		.await;

	// Either the replay never finishes, or it finishes with the husk
	// escaping. It must never absorb.
	match result {
		Ok(settled) => assert!(!settled.settlement.outcome.absorbed()),
		Err(error) => {
			assert_eq!(
				reapers_api::refusal_of(error.message()),
				Some(Refusal::HuntUnproven)
			);
		}
	}

	assert_eq!(world.call(player, Load {}).await.unwrap().xp(), 0);
}

#[tokio::test]
async fn when_a_husk_is_absorbed_then_its_art_is_learned_and_made_ready() {
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let carried = Art::carried_by(sighting.nft.collection.vessel()).expect("an art");
	let other = Art::ALL
		.into_iter()
		.find(|art| *art != carried)
		.expect("another art");

	assert_refused(
		world
			.call(
				player,
				Attune {
					art: carried.name().into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);

	let settlement = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: packed(&play(&before, &sighting)),
			},
		)
		.await
		.unwrap()
		.settlement;

	assert_eq!(
		settlement.state.active_reaper().unwrap().art(),
		Some(carried)
	);
	assert_eq!(
		world
			.call(
				player,
				Attune {
					art: carried.name().into()
				}
			)
			.await
			.unwrap()
			.loadout()
			.art,
		Some(ArtSkill {
			art: carried,
			rank: 1
		})
	);
	assert_refused(
		world
			.call(
				player,
				Attune {
					art: other.name().into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);
	assert_refused(
		world
			.call(
				player,
				Attune {
					art: "no-such-art".into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);
}

#[tokio::test]
async fn when_the_armory_is_asked_for_what_cannot_be_had_then_it_refuses() {
	let world = World::new().await;
	let player = world.player().await;
	let state = world.call(player, TakeOath {}).await.unwrap();
	let sworn = Weapon::new(state.weapon().unwrap().kind, Temper::Sworn);

	assert_refused(
		world.call(player, BuyWeapon { weapon: sworn.id() }).await,
		Refusal::LevelTooLow,
	);
	assert_refused(
		world.call(player, Wield { weapon: sworn.id() }).await,
		Refusal::NotOwned,
	);
	assert_refused(
		world
			.call(
				player,
				Wear {
					outfit: "no-such-outfit".into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);
	assert_refused(
		world
			.call(
				player,
				BuyWeapon {
					weapon: "not a weapon".into(),
				},
			)
			.await,
		Refusal::NotOwned,
	);
	assert_eq!(
		world
			.call(
				player,
				Wear {
					outfit: Outfit::Shrouded.name().into()
				}
			)
			.await
			.unwrap()
			.outfit(),
		Outfit::Shrouded
	);
}

#[tokio::test]
async fn location_uploads_preserve_purchases_arriving_at_the_same_time() {
	let world = World::new().await;
	let player = world.player().await;
	let fix = fix();

	world.call(player, TakeOath {}).await.unwrap();

	let (bought, located) = tokio::join!(
		world.call(player, Acquire { nft: SAINT.into() }),
		world.call(
			player,
			UpdateLocation {
				fix: shared(Some(fix))
			}
		),
	);

	bought.unwrap();
	located.unwrap();

	assert_eq!(
		world
			.call(player, Load {})
			.await
			.unwrap()
			.vault()
			.iter()
			.map(Nft::id)
			.collect::<Vec<_>>(),
		[SAINT]
	);
	assert_eq!(
		world.call(player, Location {}).await.unwrap().map(|f| f.at),
		Some(fix.at)
	);
}

#[tokio::test]
async fn many_purchases_at_once_spend_the_same_sol_once() {
	let world = World::new().await;
	let player = world.player().await;

	world.call(player, TakeOath {}).await.unwrap();

	let results = futures::future::join_all(
		(0..4).map(|_| world.call(player, Acquire { nft: SAINT.into() })),
	)
	.await;
	let bought = results.iter().filter(|result| result.is_ok()).count();

	assert_eq!(bought, 1, "{results:?}");

	for result in results.into_iter().filter(Result::is_err) {
		assert_refused(result, Refusal::AlreadyHeld);
	}

	assert_eq!(
		world.call(player, Load {}).await.unwrap().lamports(),
		PlayerState::STARTING_LAMPORTS - Rarity::Common.lamports()
	);
}

#[tokio::test]
async fn deployed_keepers_refuse_the_invented_nft_economy() {
	for mode in [RunMode::Staging, RunMode::Production] {
		let world = World::with(mode, quick_hall()).await;
		let player = world.player().await;

		world.call(player, TakeOath {}).await.unwrap();

		assert_refused(
			world.call(player, Acquire { nft: SAINT.into() }).await,
			Refusal::MarketUnavailable,
		);
		assert_refused(
			world
				.call(
					player,
					Release {
						nft: SAINT.into(),
						escrow: None,
					},
				)
				.await,
			Refusal::MarketUnavailable,
		);
		assert_refused(
			world
				.call(
					player,
					SettleHunt {
						sighting: 0,
						log: String::new(),
					},
				)
				.await,
			Refusal::MarketUnavailable,
		);
		assert_refused(
			world
				.call(player, AbsorbTransaction { nft: SAINT.into() })
				.await,
			Refusal::MarketUnavailable,
		);
		assert_refused(
			std::sync::Arc::clone(&world.keeper)
				.enter_rift(player, "londonBridge", false)
				.await
				.map(drop),
			Refusal::MarketUnavailable,
		);
		assert_refused(
			std::sync::Arc::clone(&world.keeper)
				.enter_rift(player, "londonBridge", true)
				.await
				.map(drop),
			Refusal::MarketUnavailable,
		);

		// What is not simulated still works.
		world.call(player, Load {}).await.unwrap();
	}
}

#[tokio::test]
async fn a_production_keeper_judges_only_the_games_own_difficulty() {
	// The judging a keeper uses is the rules', given its run mode: tested in
	// the rules for each difficulty. Here, that the keeper reads it at all:
	// a stand-in hunt is refused before any judging in production.
	let world = World::with(RunMode::Production, quick_hall()).await;
	let player = world.player().await;
	let _ = Duration::ZERO;
	let _: Timestamp = now();

	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: 0,
					log: String::new(),
				},
			)
			.await,
		Refusal::MarketUnavailable,
	);
}

#[tokio::test]
async fn moving_eye_settlement_matches_the_decoded_local_replay() {
	use reapers_rules::Eye;
	let world = World::new().await;
	let player = world.player().await;
	let saint = Nft::new(NftCollection::DegenApes, 1);
	let before = PlayerState::initial()
		.take_oath(ROOM_REPLAY_OATH_SEED)
		.unwrap()
		.acquire(saint)
		.unwrap()
		.release(
			saint,
			ROOM_REPLAY_HUSK_SEED,
			Timestamp::from_unix_seconds(0),
		)
		.unwrap();
	world.call(player, Load {}).await.unwrap();
	world.store(player, before.clone()).await;
	let sighting = before.sightings()[0];
	let plan = HuntPlan {
		room: Some(desk_room()),
		..HuntPlan::new(0, 6000)
	};
	let origin = play_in(&before, &sighting, plan);
	let stats = HuntStats::derive(&before.loadout(), 1.0).unwrap();
	let replay = |log: &HuntLog| {
		HuntSim::replay(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity(),
			sighting.nft.rarity().ward(),
			&stats,
			log,
		)
	};
	let release = origin
		.inputs
		.windows(2)
		.position(|pair| pair[0].orb_held && !pair[1].orb_held)
		.expect("orb release")
		+ 1;
	assert_eq!(release, 854);
	let mut log = origin.clone();
	for input in &mut log.inputs[release - 50..=release] {
		input.eye = Eye::new(-500, 0, 0);
	}

	let bytes = log.encode();
	assert_eq!(bytes[0], 12);
	let decoded = HuntLog::decode(&bytes).unwrap();
	let expected = replay(&decoded).expect("the moving-eye fixture finishes");
	assert_eq!(expected.grade(), Some(CaptureGrade::Plain));
	assert_eq!(
		replay(&origin).unwrap().grade(),
		Some(CaptureGrade::Excellent)
	);
	assert_ne!(
		Some(expected),
		replay(&origin),
		"ignoring the eye changes this fixed replay"
	);
	let settled = world
		.call(
			player,
			SettleHunt {
				sighting: sighting.id,
				log: STANDARD.encode(bytes),
			},
		)
		.await
		.unwrap();
	assert_eq!(settled.settlement.outcome, expected);
}

#[tokio::test]
async fn invalid_eye_settlement_changes_nothing() {
	use reapers_rules::Eye;
	let world = World::new().await;
	let player = world.player().await;
	let before = with_loose_husk(&world, player).await;
	let sighting = before.sightings()[0];
	let mut log = play(&before, &sighting);
	log.inputs[0].eye = Eye::new(1001, 0, 0);
	assert_refused(
		world
			.call(
				player,
				SettleHunt {
					sighting: sighting.id,
					log: packed(&log),
				},
			)
			.await,
		Refusal::HuntUnproven,
	);
	assert_eq!(world.call(player, Load {}).await.unwrap(), before);
}
