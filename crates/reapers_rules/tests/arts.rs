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

fn steady_with(sim: &HuntSim, focus: bool) -> HuntInput {
	HuntInput::measured(sim.pose().direction(), 0.0, &[], focus)
}

fn steady(sim: &HuntSim) -> HuntInput {
	steady_with(sim, false)
}

fn focused(sim: &HuntSim) -> HuntInput {
	steady_with(sim, true)
}

fn flick(sim: &HuntSim) -> HuntInput {
	HuntInput::measured(
		sim.pose().direction(),
		ArtRules::THROW_SPEED + 0.1,
		&[],
		true,
	)
}

/// Springs an ambush, so the fight starts with the husk standing still.
fn ambush(sim: &mut HuntSim) {
	run(
		sim,
		steady,
		|sim| sim.phase() != HuntPhase::Stalk,
		MAX_TICKS,
	);
	assert!(sim.ambushed());
}

/// Gathers the art steadily on the husk, then flicks it. Returns what the
/// flick did.
fn throw(sim: &mut HuntSim) -> Vec<HuntEvent> {
	run(
		sim,
		focused,
		|sim| sim.art_gathered() >= 1.0,
		ArtRules::GATHER_TICKS + 1,
	);

	let input = flick(sim);

	sim.step(&input)
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

/// A tap on seal `seal`, looking at the husk.
fn tap_seal(sim: &HuntSim, seal: usize) -> HuntInput {
	HuntInput {
		aim: Aim::from_direction(sim.pose().direction()),
		turn_speed: 0,
		taps: vec![Aim::from_direction(sim.pose().seal_direction(seal))],
		focus: false,
	}
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
	fn it_is_gathered_while_the_phone_is_steady_and_a_flick_throws_it() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);

		// A flick before it is gathered throws nothing.
		let input = focused(&sim);
		sim.step(&input);
		let input = flick(&sim);
		assert_eq!(count(&sim.step(&input), "ArtThrown"), 0);

		let events = throw(&mut sim);

		assert_eq!(single_thrown(&events), Art::Cinder);
		assert_eq!(sim.art_charges(), 0);
		assert_eq!(sim.flights().len(), 1);
	}

	#[test]
	fn turning_while_holding_it_does_not_gather_it() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);
		run(
			&mut sim,
			|sim| {
				HuntInput::measured(
					sim.pose().direction(),
					HuntRules::STEADY_SPEED + 0.2,
					&[],
					true,
				)
			},
			never,
			60,
		);

		assert_eq!(sim.art_gathered(), 0.0);
	}

	#[test]
	fn letting_go_before_the_flick_loses_nothing() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);
		run(&mut sim, focused, |sim| sim.art_gathered() >= 1.0, 60);
		let input = steady(&sim);
		sim.step(&input);

		assert_eq!(sim.art_gathered(), 0.0);
		assert_eq!(sim.art_charges(), 1);
	}

	#[test]
	fn nothing_is_thrown_into_a_shriek_and_the_flick_is_a_flinch() {
		let mut sim = sim_with(Art::Cinder);

		ambush(&mut sim);
		run(
			&mut sim,
			focused,
			|sim| sim.phase() == HuntPhase::Shriek,
			MAX_TICKS,
		);
		run(&mut sim, focused, never, HuntRules::SHRIEK_GRACE_TICKS);

		let input = flick(&sim);
		let events = sim.step(&input);

		assert_eq!(count(&events, "ArtThrown"), 0);
		assert_eq!(count(&events, "Flinched"), 1);
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
		run(&mut sim, focused, never, 40);

		let input = flick(&sim);
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
		let strike = sim.stats().strike_damage;

		assert!(hit);
		assert!((damage - strike * 2.0).abs() <= 1e-9);
		assert!(sim.burning());

		let burning = sim.ward();
		let events = run(&mut sim, steady, |sim| !sim.burning(), 400);

		assert!((burning - sim.ward() - strike * 1.5).abs() <= 0.5);
		assert!(before - sim.ward() > strike * 3.4);
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
		let swing = sim.stats().swing_ticks;

		ambush(&mut sim);
		// Spend the ambush's doubled first strike first.
		run(&mut sim, steady, never, swing);
		throw(&mut sim);
		land(&mut sim);
		run(&mut sim, steady, never, swing);

		let seal = (0..sim.vessel().seals().len())
			.find(|&index| sim.seal_open(index))
			.unwrap();
		let input = tap_seal(&sim, seal);
		let events = sim.step(&input);
		let landed: Vec<_> = events
			.iter()
			.filter_map(|event| {
				match event {
					HuntEvent::StrikeLanded { empowered, .. } => Some(*empowered),
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
		run(
			&mut sim,
			|sim| {
				let pose = sim.pose();

				HuntInput::measured(
					Vec3::from_angles(pose.yaw() + radians(15.0), pose.pitch()),
					0.0,
					&[],
					true,
				)
			},
			|sim| sim.art_gathered() >= 1.0,
			40,
		);
		let input = flick(&sim);
		sim.step(&input);

		assert!(single_landed(&land(&mut sim)).0);
	}
}

mod the_heavy_weapons {
	use super::*;

	fn strike_open_seal(sim: &HuntSim) -> HuntInput {
		(0..sim.vessel().seals().len())
			.find(|&seal| sim.seal_open(seal))
			.map_or_else(|| steady(sim), |seal| tap_seal(sim, seal))
	}

	#[test]
	fn a_greatsword_stops_a_husk_in_its_tracks() {
		let mut sim = Opts {
			weapon: Some(WeaponKind::Greatsword),
			..Opts::default()
		}
		.sim();

		ambush(&mut sim);
		run(&mut sim, steady, |sim| sim.speed() > 0.0, 900);
		assert!(sim.speed() > 0.0, "it should move off");

		let input = strike_open_seal(&sim);
		let events = sim.step(&input);

		assert_eq!(count(&events, "StrikeLanded"), 1);
		assert!(sim.staggered());
		assert_eq!(sim.speed(), 0.0);
	}

	#[test]
	fn an_axe_bites_the_body_wherever_it_lands() {
		let weapon = |kind| {
			Opts {
				weapon: Some(kind),
				..Opts::default()
			}
			.sim()
		};
		let mut axe = weapon(WeaponKind::Axe);
		// A spear reaches no further than an axe, so the same blow misses
		// every seal with both.
		let mut spear = weapon(WeaponKind::Spear);

		for sim in [&mut axe, &mut spear] {
			ambush(sim);
		}

		let glance = |sim: &mut HuntSim| {
			let pose = sim.pose();
			// Low on the body, well away from any seal.
			let body = (pose.position - Vec3::UP * 0.5).normalized();
			let events = sim.step(&HuntInput {
				aim: Aim::from_direction(body),
				turn_speed: 0,
				taps: vec![Aim::from_direction(body)],
				focus: false,
			});
			let glanced: Vec<_> = events
				.iter()
				.filter_map(|event| {
					match event {
						HuntEvent::StrikeGlanced { damage } => Some(*damage),
						_ => None,
					}
				})
				.collect();

			assert_eq!(glanced.len(), 1);

			glanced[0] / sim.stats().strike_damage
		};

		assert!((glance(&mut axe) - 0.7).abs() <= 1e-9);
		assert!((glance(&mut spear) - HuntRules::GLANCE_SHARE).abs() <= 1e-9);
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
				&HuntOutcome::absorbed_with(ReapGrade::Clean, false, 600, HuntDeeds::default()),
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
