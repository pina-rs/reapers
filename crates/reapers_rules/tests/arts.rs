//! The arts: Cinder, Rime and Gale, learned and thrown.

mod support;

use reapers_rules::*;
use support::*;

fn stats_with(art: Option<Art>, rank: i64, weapon: Option<WeaponKind>) -> HuntStats {
	HuntStats::derive(
		&HuntLoadout {
			art: art.map(|art| ArtSkill::new(art, rank).unwrap()),
			..HuntLoadout::new(
				ReaperClass::Harvester,
				6,
				Weapon::new(weapon.unwrap_or(WeaponKind::Scythe), Temper::Shrouded),
			)
		},
		1.0,
	)
	.unwrap()
}

fn stats(art: Option<Art>) -> HuntStats {
	stats_with(art, 1, None)
}

/// What a hunt in these tests is played with, for [`Opts::sim`].
struct Opts {
	art: Option<Art>,
	rank: i64,
	weapon: Option<WeaponKind>,
	plan: HuntPlan,
	vessel: Vessel,
	seed: i64,
}

impl Default for Opts {
	fn default() -> Self {
		Self {
			art: None,
			rank: 1,
			weapon: None,
			plan: HuntPlan::OPEN,
			vessel: Vessel::Ape,
			seed: 3,
		}
	}
}

impl Opts {
	fn sim(self) -> HuntSim {
		HuntSim::new(
			self.seed,
			self.vessel,
			Rarity::Epic,
			Rarity::Epic.ward(),
			&stats_with(self.art, self.rank, self.weapon),
			self.plan,
			Difficulty::Normal,
		)
		.unwrap()
	}
}

fn sim_with(art: Art) -> HuntSim {
	Opts {
		art: Some(art),
		..Opts::default()
	}
	.sim()
}

const MAX_TICKS: i64 = HuntLog::MAX_TICKS as i64;

/// Steps `sim` with `input` until `until` holds, and returns what happened.
fn run(
	sim: &mut HuntSim,
	mut input: impl FnMut(&HuntSim) -> HuntInput,
	until: impl Fn(&HuntSim) -> bool,
	ticks: i64,
) -> Vec<HuntEvent> {
	let mut events = Vec::new();
	let end = sim.tick() + ticks;

	while !until(sim) && !sim.is_over() && sim.tick() < end {
		let next = input(sim);

		events.extend(sim.step(&next));
	}

	events
}

fn never(_: &HuntSim) -> bool {
	false
}

/// Looks at the husk and holds still, with the art in hand when the reaper
/// knows one, or `slot`.
fn steady_with_slot(sim: &HuntSim, slot: Option<usize>) -> HuntInput {
	HuntInput::measured(
		sim.pose().direction(),
		0.0,
		false,
		slot.unwrap_or_else(|| {
			if sim.stats().art.is_none() {
				Arsenal::GUN_SLOT
			} else {
				Arsenal::ART_SLOT
			}
		}),
		false,
	)
}

fn steady(sim: &HuntSim) -> HuntInput {
	steady_with_slot(sim, None)
}

/// Presses FIRE along `aim`, or at the husk, with the art in hand.
fn fire(sim: &HuntSim, aim: Option<Vec3>) -> HuntInput {
	HuntInput::measured(
		aim.unwrap_or_else(|| sim.pose().direction()),
		0.0,
		true,
		Arsenal::ART_SLOT,
		false,
	)
}

/// Springs an ambush, so the fight starts with the husk standing still. The
/// art is drawn on the way.
fn ambush(sim: &mut HuntSim) {
	run(
		sim,
		steady,
		|sim| sim.phase() != HuntPhase::Stalk,
		MAX_TICKS,
	);
	assert!(sim.ambushed());
}

/// Waits for the art to be in hand and ready, then throws it at the husk,
/// or along `aim`. Returns what the throw did.
fn throw_at(sim: &mut HuntSim, aim: Option<Vec3>) -> Vec<HuntEvent> {
	run(
		sim,
		steady,
		|sim| sim.selected_slot() == Arsenal::ART_SLOT && sim.cooldown_ticks() == 0,
		Arsenal::SWITCH_TICKS + 1,
	);

	let input = fire(sim, aim);

	sim.step(&input)
}

fn throw(sim: &mut HuntSim) -> Vec<HuntEvent> {
	throw_at(sim, None)
}

/// Lets whatever is in the air come down, holding still.
fn land(sim: &mut HuntSim) -> Vec<HuntEvent> {
	run(sim, steady, |sim| sim.flights().is_empty(), 120)
}

/// A hunt in the kitchen whose husk has gone behind the counter before it
/// has noticed anyone, and stays there a while.
fn hidden_in_kitchen(art: Art) -> HuntSim {
	let settled =
		|sim: &HuntSim| sim.footing() == Footing::Hidden && !sim.leaping() && sim.speed() == 0.0;

	for seed in 1..200 {
		let mut sim = Opts {
			art: Some(art),
			plan: kitchen(),
			seed,
			..Opts::default()
		}
		.sim();

		// Looking away, so as not to spring an ambush.
		run(
			&mut sim,
			|_| HuntInput::new(Aim::new(18000, 0), 0),
			settled,
			20 * HuntRules::TICK_RATE,
		);

		if settled(&sim) && sim.phase() == HuntPhase::Stalk {
			return sim;
		}
	}

	panic!("no husk hid in the kitchen");
}

fn count(events: &[HuntEvent], kind: &str) -> usize {
	events.iter().filter(|event| event.kind() == kind).count()
}

/// The one art that landed among `events`: whether it hit, and its damage.
fn single_landed(events: &[HuntEvent]) -> (bool, f64) {
	let landed: Vec<_> = events
		.iter()
		.filter_map(|event| {
			match event {
				HuntEvent::ArtLanded { hit, damage, .. } => Some((*hit, *damage)),
				_ => None,
			}
		})
		.collect();

	assert_eq!(landed.len(), 1, "exactly one art lands");

	landed[0]
}

fn single_thrown(events: &[HuntEvent]) -> Art {
	let thrown: Vec<_> = events
		.iter()
		.filter_map(|event| {
			match event {
				HuntEvent::ArtThrown(art) => Some(*art),
				_ => None,
			}
		})
		.collect();

	assert_eq!(thrown.len(), 1, "exactly one art is thrown");

	thrown[0]
}

mod learning_an_art {
	use super::*;

	#[test]
	fn each_creature_carries_an_art_and_the_rifts_carries_none() {
		for vessel in Vessel::ALL {
			assert_eq!(Art::carried_by(vessel).is_some(), vessel.is_solo());
		}

		assert_eq!(Art::carried_by(Vessel::Penguin), Some(Art::Rime));
		assert_eq!(Art::carried_by(Vessel::Ape), Some(Art::Cinder));
		assert_eq!(Art::carried_by(Vessel::Bird), Some(Art::Gale));
	}

	#[test]
	fn it_deepens_with_every_husk_absorbed_that_carries_it() {
		assert_eq!(
			(0..=11).map(Art::rank_for).collect::<Vec<_>>(),
			[0, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3],
		);
	}

	#[test]
	fn a_deeper_art_holds_more_throws_and_readies_them_sooner() {
		let first = ArtSkill::new(Art::Rime, 1).unwrap();
		let third = ArtSkill::new(Art::Rime, 3).unwrap();

		assert!(third.max_charges() > first.max_charges());
		assert!(third.recharge_ticks() < first.recharge_ticks());
		assert!(third.freeze_ticks() > first.freeze_ticks());
		assert!(third.shatter() > first.shatter());
	}
}

mod throwing_an_art {
	use super::*;

	#[test]
	fn fire_with_the_art_in_hand_throws_it_at_the_crosshair_at_once() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);

		let events = throw(&mut sim);

		assert_eq!(single_thrown(&events), Art::Cinder);
		assert_eq!(sim.art_charges(), 0);
		assert_eq!(sim.flights().len(), 1);
	}

	#[test]
	fn nothing_is_thrown_until_the_art_is_drawn() {
		let mut sim = sim_with(Art::Cinder);

		run(
			&mut sim,
			|sim| steady_with_slot(sim, Some(Arsenal::GUN_SLOT)),
			|sim| sim.phase() != HuntPhase::Stalk,
			MAX_TICKS,
		);

		let early = sim.step(&fire(&sim, None));

		assert_eq!(count(&early, "WeaponDrawn"), 1);
		assert_eq!(count(&early, "ArtThrown"), 0);
		assert_eq!(count(&throw(&mut sim), "ArtThrown"), 1);
	}

	#[test]
	fn nothing_is_thrown_into_a_shriek_and_fire_is_a_flinch() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);
		run(
			&mut sim,
			steady,
			|sim| sim.phase() == HuntPhase::Shriek,
			MAX_TICKS,
		);
		run(&mut sim, steady, never, HuntRules::SHRIEK_GRACE_TICKS);

		let charges = sim.art_charges();
		let events = sim.step(&fire(&sim, None));

		assert_eq!(count(&events, "ArtThrown"), 0);
		assert_eq!(count(&events, "Flinched"), 1);
		assert_eq!(sim.art_charges(), charges);
	}

	#[test]
	fn a_throw_is_readied_again_in_time_and_by_holding_firm() {
		let mut sim = sim_with(Art::Rime);

		ambush(&mut sim);
		throw(&mut sim);
		assert_eq!(sim.art_charges(), 0);

		run(&mut sim, steady, |sim| sim.art_charges() > 0, 2000);
		assert_eq!(sim.art_charges(), 1);

		let mut held = sim_with(Art::Rime);

		ambush(&mut held);
		throw(&mut held);
		run(
			&mut held,
			steady,
			|sim| sim.phase() == HuntPhase::Shriek,
			MAX_TICKS,
		);
		run(&mut held, steady, never, HuntRules::SHRIEK_TICKS - 1);

		let before = held.art_charges();
		let events = run(
			&mut held,
			steady,
			|sim| sim.phase() != HuntPhase::Shriek,
			MAX_TICKS,
		);

		assert_eq!(count(&events, "HeldFirm"), 1);
		assert_eq!(held.art_charges(), before + 1);
	}

	#[test]
	fn a_reaper_with_no_art_has_nothing_to_throw() {
		let mut sim = Opts::default().sim();

		ambush(&mut sim);
		run(
			&mut sim,
			|sim| steady_with_slot(sim, Some(Arsenal::ART_SLOT)),
			never,
			Arsenal::SWITCH_TICKS + 1,
		);

		let input = fire(&sim, None);

		assert_eq!(count(&sim.step(&input), "ArtThrown"), 0);
		assert_eq!(sim.art_charges(), 0);
	}
}

mod fire {
	use super::*;

	#[test]
	fn lands_on_a_husk_at_rest_burns_it_and_sends_it_running() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);

		let before = sim.ward();

		throw(&mut sim);

		let (hit, damage) = single_landed(&land(&mut sim));
		let hit_damage = sim.stats().hit;

		assert!(hit);
		assert!((damage - hit_damage * 2.0).abs() <= 1e-9);
		assert!(sim.burning());

		let burning = sim.ward();
		let events = run(&mut sim, steady, |sim| !sim.burning(), 400);

		assert!((burning - sim.ward() - hit_damage * 1.5).abs() <= 0.5);
		assert!(before - sim.ward() > hit_damage * 3.4);
		assert!(count(&events, "Relocating") > 0);
	}

	#[test]
	fn is_lobbed_over_the_counter_onto_a_husk_hiding_behind_it() {
		let mut sim = hidden_in_kitchen(Art::Cinder);

		// The top of its head shows over the counter; aim there.
		assert!(sim.plan().hides(sim.pose().position));

		let events = throw(&mut sim);

		assert_eq!(count(&events, "ArtThrown"), 1);
		assert!(single_landed(&land(&mut sim)).0);
	}

	#[test]
	fn a_burning_husk_does_not_run_for_cover() {
		let mut sim = Opts {
			art: Some(Art::Cinder),
			plan: kitchen(),
			seed: 8,
			..Opts::default()
		}
		.sim();

		ambush(&mut sim);
		throw(&mut sim);
		land(&mut sim);

		let mut footings = Vec::new();

		run(&mut sim, steady, |sim| !sim.burning(), 400);
		run(
			&mut sim,
			|sim| {
				if sim.burning() {
					footings.push(sim.footing());
				}
				steady(sim)
			},
			|sim| !sim.burning(),
			MAX_TICKS,
		);

		assert!(!footings.contains(&Footing::Hidden));
	}
}

mod ice {
	use super::*;

	#[test]
	fn freezes_the_husk_where_it_stands_every_seal_in_view_open() {
		let mut sim = sim_with(Art::Rime);

		ambush(&mut sim);
		run(&mut sim, steady, |sim| sim.speed() > 0.0, 600);
		throw(&mut sim);

		let (hit, _) = single_landed(&land(&mut sim));

		assert!(hit);
		assert!(sim.frozen());
		assert_eq!(sim.speed(), 0.0);

		let at = sim.pose().position;

		run(&mut sim, steady, never, 60);
		assert_eq!(sim.pose().position, at);

		let pose = sim.pose();

		for seal in 0..sim.vessel().seals().len() {
			if pose.seal_faces_player(seal) {
				assert!(sim.seal_open(seal), "seal {seal} is open");
			}
		}
	}

	#[test]
	fn the_next_blow_shatters_it_and_lands_harder() {
		let mut sim = sim_with(Art::Rime);

		ambush(&mut sim);
		throw(&mut sim);
		land(&mut sim);
		// The gun, drawn while the husk stands frozen.
		run(
			&mut sim,
			|sim| steady_with_slot(sim, Some(Arsenal::GUN_SLOT)),
			never,
			Arsenal::SWITCH_TICKS + 1,
		);

		assert!(sim.frozen());

		let seal = (0..sim.vessel().seals().len())
			.find(|&index| sim.seal_open(index))
			.unwrap();
		let input = HuntInput::measured(
			sim.pose().seal_direction(seal),
			0.0,
			true,
			Arsenal::GUN_SLOT,
			false,
		);
		let events = sim.step(&input);
		let landed: Vec<_> = events
			.iter()
			.filter_map(|event| {
				match event {
					HuntEvent::HitLanded { empowered, .. } => Some(*empowered),
					_ => None,
				}
			})
			.collect();

		assert_eq!(landed.len(), 1);
		assert_eq!(count(&events, "IceShattered"), 1);
		assert!(landed[0]);
		assert!(!sim.frozen());
	}

	#[test]
	fn stops_a_charge_dead() {
		let mut sim = Opts {
			art: Some(Art::Rime),
			rank: 2,
			..Opts::default()
		}
		.sim();

		ambush(&mut sim);
		run(
			&mut sim,
			steady,
			|sim| sim.phase() == HuntPhase::Charge,
			MAX_TICKS,
		);
		throw(&mut sim);

		let events = land(&mut sim);

		assert!(single_landed(&events).0);
		assert_eq!(count(&events, "Struck"), 0);
		assert_ne!(sim.phase(), HuntPhase::Charge);
		assert!(sim.frozen());
	}

	#[test]
	fn breaks_on_the_counter_in_front_of_a_husk_hiding_behind_it() {
		let mut sim = hidden_in_kitchen(Art::Rime);

		throw(&mut sim);

		assert!(!single_landed(&land(&mut sim)).0);
		assert!(!sim.frozen());
	}
}

mod wind {
	use super::*;

	#[test]
	fn blows_a_husk_out_from_behind_the_counter_and_stuns_it() {
		let mut sim = hidden_in_kitchen(Art::Gale);

		throw(&mut sim);

		let events = land(&mut sim);
		let leapt: Vec<_> = events
			.iter()
			.filter_map(|event| {
				match event {
					HuntEvent::Leapt(toward) => Some(*toward),
					_ => None,
				}
			})
			.collect();

		assert!(single_landed(&events).0);
		assert_eq!(leapt, [Footing::Open]);

		run(&mut sim, steady, |sim| !sim.leaping(), 200);

		assert_eq!(sim.footing(), Footing::Open);
		assert!(sim.staggered());
	}

	#[test]
	fn turns_a_charge_aside() {
		let mut sim = sim_with(Art::Gale);

		ambush(&mut sim);
		run(
			&mut sim,
			steady,
			|sim| sim.phase() == HuntPhase::Charge,
			MAX_TICKS,
		);
		throw(&mut sim);

		let events = land(&mut sim);

		assert_eq!(count(&events, "Struck"), 0);
		assert!(sim.staggered());
	}

	#[test]
	fn catches_a_husk_it_was_only_thrown_near() {
		let mut sim = sim_with(Art::Gale);

		ambush(&mut sim);

		let pose = sim.pose();
		let events = throw_at(
			&mut sim,
			Some(Vec3::from_angles(pose.yaw() + radians(15.0), pose.pitch())),
		);

		assert_eq!(count(&events, "ArtThrown"), 1);
		assert!(single_landed(&land(&mut sim)).0);
	}
}

mod the_heavy_weapons {
	use super::*;

	#[test]
	fn a_greatsword_stops_a_husk_in_its_tracks() {
		fn heavy(sim: &HuntSim) -> HuntInput {
			steady_with_slot(sim, Some(Arsenal::HEAVY_SLOT))
		}

		// A husk on the move, within a dive, with no shriek about to start:
		// a husk drawing breath is going nowhere anyway.
		for seed in 1..=20 {
			let mut sim = Opts {
				weapon: Some(WeaponKind::Greatsword),
				seed,
				..Opts::default()
			}
			.sim();

			ambush(&mut sim);
			run(
				&mut sim,
				heavy,
				|sim| {
					sim.phase() == HuntPhase::Fight
						&& sim.speed() > 0.0
						&& !sim.leaping()
						&& sim.pose().distance() <= HeavyRules::DIVE_METRES
				},
				3000,
			);

			if sim.speed() == 0.0 || sim.phase() != HuntPhase::Fight {
				continue;
			}

			let dive = sim.step(&HuntInput::measured(
				sim.pose().direction(),
				0.0,
				true,
				Arsenal::HEAVY_SLOT,
				false,
			));

			assert_eq!(count(&dive, "DiveStarted"), 1);

			let swing_ticks = sim.stats().swing_ticks;
			let events = run(&mut sim, heavy, never, HeavyRules::IN_TICKS + swing_ticks);

			if sim.phase() != HuntPhase::Fight {
				continue;
			}

			assert_eq!(count(&events, "HitLanded"), 1);
			assert!(sim.staggered());
			assert_eq!(sim.speed(), 0.0);

			return;
		}

		panic!("no husk was caught on the move");
	}

	#[test]
	fn an_axe_bites_the_body_harder_than_any_other_weapon() {
		assert_eq!(stats_with(None, 1, Some(WeaponKind::Axe)).body_share, 0.7);

		for kind in WeaponKind::ALL {
			if kind == WeaponKind::Axe {
				continue;
			}

			assert_eq!(
				stats_with(None, 1, Some(kind)).body_share,
				WeaponProfile::PLAIN_BODY_SHARE,
				"{kind:?}",
			);
		}
	}

	#[test]
	fn they_cost_more_than_a_class_weapon_and_need_a_few_levels() {
		let greatsword = Weapon::new(WeaponKind::Greatsword, Temper::Shrouded);

		assert_eq!(
			greatsword.price_for(ReaperClass::Harvester),
			Weapon::HEAVY_PRICE
		);
		assert_eq!(greatsword.level_required(), Weapon::HEAVY_LEVEL);
		assert_eq!(
			Weapon::new(WeaponKind::Axe, Temper::Scorched).level_required(),
			Temper::Scorched.level_required(),
		);
	}
}

mod a_hunt_with_an_art {
	use super::*;

	#[test]
	fn a_careful_player_with_an_art_brings_husks_down_sooner() {
		for art in Art::ALL {
			let mut with_art = 0;
			let mut without = 0;

			for seed in 1..=10 {
				for known in [Some(art), None] {
					let mut sim = HuntSim::new(
						seed,
						Vessel::Ape,
						Rarity::Rare,
						Rarity::Rare.ward(),
						&stats(known),
						kitchen(),
						Difficulty::Normal,
					)
					.unwrap();

					HuntBot::careful(&sim).play(&mut sim);

					if known.is_some() {
						with_art += sim.tick();
					} else {
						without += sim.tick();
					}
				}
			}

			assert!(with_art < without, "{art:?}");
		}
	}

	#[test]
	fn replays_to_the_same_end_throws_and_all() {
		for art in Art::ALL {
			let mut sim = Opts {
				art: Some(art),
				plan: kitchen(),
				seed: 11,
				..Opts::default()
			}
			.sim();
			let mut bot = HuntBot::careful(&sim);

			bot.play(&mut sim);

			assert!(count(&bot.events, "ArtThrown") > 0, "{art:?}");

			let log = HuntLog::decode(
				&HuntLog {
					plan: sim.plan().clone(),
					..HuntLog::new(bot.recorded.clone())
				}
				.encode(),
			)
			.unwrap();
			let replayed = HuntSim::replay(
				11,
				Vessel::Ape,
				Rarity::Epic,
				Rarity::Epic.ward(),
				&stats(Some(art)),
				&log,
			);

			assert_eq!(
				serde_json::to_value(replayed).unwrap(),
				serde_json::to_value(sim.outcome()).unwrap(),
				"{art:?}",
			);
		}
	}
}

mod a_reapers_arts {
	use super::*;

	fn now() -> Timestamp {
		Timestamp::utc(2026, 10, 19, 0, 0, 0)
	}

	fn absorbing(state: &PlayerState, nft: Nft, seed: i64) -> PlayerState {
		let released = state
			.acquire(nft)
			.unwrap()
			.release(nft, seed, now())
			.unwrap();
		let sighting = *released.sightings().last().unwrap();

		released
			.settle_hunt(
				sighting.id,
				&HuntOutcome::absorbed_with(CaptureGrade::Great, false, 600, HuntDeeds::default()),
				sighting.surfaces_at,
				seed,
			)
			.unwrap()
	}

	fn of(collection: NftCollection) -> Nft {
		market_listings()
			.into_iter()
			.find(|nft| nft.collection == collection)
			.unwrap()
	}

	#[test]
	fn the_first_husk_carrying_an_art_teaches_it_and_makes_it_ready() {
		let mut state = PlayerState::initial().take_oath(5).unwrap();

		assert_eq!(state.loadout().art, None);

		state = absorbing(&state, of(NftCollection::PeskyPenguins), 1);

		assert_eq!(state.active_reaper().unwrap().art(), Some(Art::Rime));
		assert_eq!(state.loadout().art, ArtSkill::new(Art::Rime, 1));

		// A second art learned leaves the first one ready.
		state = absorbing(&state, of(NftCollection::Thugbirdz), 2);

		assert_eq!(state.active_reaper().unwrap().art(), Some(Art::Rime));
		assert_eq!(state.active_reaper().unwrap().art_rank(Art::Gale), 1);
		assert_eq!(
			state.attune(Art::Gale).unwrap().loadout().art,
			ArtSkill::new(Art::Gale, 1),
		);
	}

	#[test]
	fn an_art_that_has_not_been_learned_cannot_be_made_ready() {
		let state = PlayerState::initial().take_oath(5).unwrap();

		assert_eq!(
			state.attune(Art::Cinder).unwrap_err().refusal,
			Refusal::NotOwned,
		);
	}

	#[test]
	fn the_art_a_reaper_has_ready_is_kept_with_it() {
		let state = absorbing(
			&PlayerState::initial().take_oath(5).unwrap(),
			of(NftCollection::DegenApes),
			1,
		);
		let restored: PlayerState =
			serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();

		assert_eq!(restored.active_reaper().unwrap().art(), Some(Art::Cinder));
		assert_eq!(restored.loadout().art, ArtSkill::new(Art::Cinder, 1));
	}

	#[test]
	fn a_saved_reaper_from_before_arts_loads_with_none_ready() {
		let state = PlayerState::initial().take_oath(5).unwrap();
		let mut json = serde_json::to_value(&state).unwrap();

		for reaper in json["reapers"].as_array_mut().unwrap() {
			reaper.as_object_mut().unwrap().remove("art");
		}

		let loaded: PlayerState = serde_json::from_value(json).unwrap();

		assert_eq!(loaded.active_reaper().unwrap().art(), None);
	}
}
