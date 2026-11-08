//! Sigils: how one is laid out, and how a trace follows it.

#![allow(clippy::float_cmp, reason = "these are compared exactly")]

mod support;

use std::collections::HashMap;
use std::collections::HashSet;

use reapers_rules::*;
use support::*;

/// What a sigil is drawn for, for [`sigil`].
#[derive(Clone, Copy)]
struct Draw {
	seed: i64,
	rarity: Rarity,
	gait: Gait,
	difficulty: Difficulty,
}

impl Default for Draw {
	fn default() -> Self {
		Self {
			seed: 7,
			rarity: Rarity::Uncommon,
			gait: Gait::Ground,
			difficulty: Difficulty::Normal,
		}
	}
}

fn sigil(draw: Draw) -> Sigil {
	Sigil::draw(draw.seed, &draw.rarity.sigil(), draw.gait, draw.difficulty)
}

fn seeded(seed: i64) -> Sigil {
	sigil(Draw {
		seed,
		..Draw::default()
	})
}

fn of(rarity: Rarity) -> Sigil {
	sigil(Draw {
		rarity,
		..Draw::default()
	})
}

/// A view of `trace`'s next light, dead on.
fn on_next(trace: &SigilTrace) -> Vec3 {
	trace.direction_of(trace.caught())
}

/// A view well away from every light: straight up.
const AWAY: Vec3 = Vec3::UP;

const FRAME: f64 = 1.0 / 60.0;

fn close_to(actual: f64, expected: f64, delta: f64) -> bool {
	(actual - expected).abs() <= delta
}

/// Each easier rarity, with the next one up.
const HARDER: [(Rarity, Rarity); 3] = [
	(Rarity::Common, Rarity::Uncommon),
	(Rarity::Uncommon, Rarity::Rare),
	(Rarity::Rare, Rarity::Epic),
];

mod drawing_a_sigil {
	use super::*;

	#[test]
	fn the_same_hunt_always_has_the_same_sigil() {
		let one = seeded(41);
		let other = seeded(41);
		let angles = |sigil: &Sigil| {
			sigil
				.lights
				.iter()
				.map(|light| (light.yaw, light.pitch))
				.collect::<Vec<_>>()
		};

		assert_eq!(angles(&one), angles(&other));
	}

	#[test]
	fn different_hunts_have_different_sigils() {
		let shapes: HashSet<String> = (0..40)
			.map(|seed| {
				seeded(seed)
					.lights
					.iter()
					.map(|light| {
						format!(
							"{}/{}",
							degrees(light.yaw).round(),
							degrees(light.pitch).round()
						)
					})
					.collect::<Vec<_>>()
					.join(" ")
			})
			.collect();

		assert!(shapes.len() > 15, "{}", shapes.len());
	}

	/// A sigil of `rarity` for a husk of `gait` is laid out as its shape
	/// says, in every hunt.
	fn laid_out_as_its_shape_says(rarity: Rarity, gait: Gait) {
		let shape = rarity.sigil();
		let (low, high) = if gait == Gait::Ground {
			HuntRules::SIGIL_GROUND_BAND
		} else {
			HuntRules::SIGIL_AIR_BAND
		};
		let half_span = radians(shape.span_degrees) / 2.0;
		let mut crossing_as_asked = 0;

		for seed in 0..300 {
			let sigil = sigil(Draw {
				seed,
				rarity,
				gait,
				..Draw::default()
			});
			let mut yaws: Vec<f64> = sigil.lights.iter().map(|light| light.yaw).collect();

			yaws.sort_by(f64::total_cmp);

			assert_eq!(sigil.lights.len(), shape.lights, "seed {seed}");
			assert!(close_to(yaws[0], -half_span, 1e-9), "seed {seed}");
			assert!(
				close_to(yaws[yaws.len() - 1], half_span, 1e-9),
				"seed {seed}"
			);

			for light in &sigil.lights {
				assert!(
					(low - 1e-9..=high + 1e-9).contains(&light.pitch),
					"seed {seed}: {light:?}"
				);
			}

			if sigil.crossings() == shape.crossings {
				crossing_as_asked += 1;
			}
		}

		assert_eq!(
			crossing_as_asked, 300,
			"every sigil crosses itself as often as its shape asks"
		);
	}

	macro_rules! laid_out {
		($($name:ident: $rarity:ident $gait:ident;)+) => {
			$(
				#[test]
				fn $name() {
					laid_out_as_its_shape_says(Rarity::$rarity, Gait::$gait);
				}
			)+
		};
	}

	laid_out! {
		a_common_ground_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Common Ground;
		a_common_air_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Common Air;
		a_uncommon_ground_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Uncommon Ground;
		a_uncommon_air_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Uncommon Air;
		a_rare_ground_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Rare Ground;
		a_rare_air_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Rare Air;
		a_epic_ground_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Epic Ground;
		a_epic_air_sigil_is_laid_out_as_its_shape_says_in_every_hunt: Epic Air;
	}

	#[test]
	fn the_trace_never_passes_over_a_light_it_does_not_stop_at() {
		for rarity in Rarity::ALL {
			let radius = radians(rarity.sigil().radius_degrees);

			for seed in 0..300 {
				let sigil = sigil(Draw {
					seed,
					rarity,
					..Draw::default()
				});
				let lights = &sigil.lights;

				for step in 0..lights.len() - 1 {
					let from = lights[step];
					let to = lights[step + 1];

					for (index, light) in lights.iter().enumerate() {
						if index == step || index == step + 1 {
							continue;
						}

						// Sampled along the shortest way from one light to the
						// next, which is the straight line the view draws
						// between them.
						let mut share = 0.0;

						while share <= 1.0 {
							let along = (from.direction() * (1.0 - share) + to.direction() * share)
								.normalized();
							let apart = along.angle_to(light.direction());

							assert!(
								apart > radius,
								"{rarity} sigil {seed}: {from:?} to {to:?} passes over {light:?}"
							);

							share += 0.02;
						}
					}
				}
			}
		}
	}

	#[test]
	fn a_rarer_husk_has_a_harder_sigil() {
		for (easier, harder) in HARDER {
			let (one, other) = (easier.sigil(), harder.sigil());

			assert!(other.lights > one.lights);
			assert!(other.span_degrees > one.span_degrees);
			assert!(other.radius_degrees < one.radius_degrees);
			assert!(other.hold_seconds > one.hold_seconds);
			assert!(other.crossings > one.crossings);
			assert!(
				other.fray_seconds.unwrap_or(f64::INFINITY)
					<= one.fray_seconds.unwrap_or(f64::INFINITY)
			);
		}
	}

	#[test]
	fn every_sigil_reads_an_arc_wide_enough_to_hunt_in() {
		for rarity in Rarity::ALL {
			let half_arc = of(rarity).half_arc();

			assert!(
				degrees(half_arc) * 100.0 >= f64::from(HuntPlan::MIN_HALF_WIDTH_CENTIDEGREES),
				"{rarity}"
			);
		}
	}

	#[test]
	fn the_testing_difficulty_changes_how_forgiving_a_sigil_is_never_its_shape() {
		let at = |difficulty| {
			sigil(Draw {
				rarity: Rarity::Rare,
				difficulty,
				..Draw::default()
			})
		};
		let normal = at(Difficulty::Normal);
		let easy = at(Difficulty::Easy);
		let hard = at(Difficulty::Hard);
		let yaws = |sigil: &Sigil| {
			sigil
				.lights
				.iter()
				.map(|light| light.yaw)
				.collect::<Vec<_>>()
		};

		assert!(easy.radius > normal.radius);
		assert!(hard.radius < normal.radius);
		assert!(easy.fray_seconds.unwrap() > normal.fray_seconds.unwrap());

		for other in [&easy, &hard] {
			assert_eq!(yaws(other), yaws(&normal));
		}
	}
}

mod tracing_a_sigil {
	use super::*;

	#[test]
	fn a_light_catches_after_it_is_held_still_for_as_long_as_it_asks() {
		let mut trace = SigilTrace::new(seeded(7), 0.4);
		let frames = (trace.sigil.hold_seconds * 60.0).ceil() as usize;
		let mut events = Vec::new();

		for _ in 0..frames - 1 {
			let aim = on_next(&trace);

			events.extend(trace.advance(aim, 0.0, FRAME));
		}

		assert!(events.is_empty(), "{events:?}");
		assert!(trace.fill() > 0.9, "{}", trace.fill());

		let aim = on_next(&trace);

		events.extend(trace.advance(aim, 0.0, FRAME));

		assert_eq!(events, [SigilEvent::LightCaught(0)]);
		assert_eq!(trace.caught(), 1);
		assert_eq!(trace.fill(), 0.0);
	}

	#[test]
	fn the_lights_catch_in_order_and_only_in_order() {
		let mut trace = SigilTrace::new(seeded(7), 0.0);
		let later = trace.direction_of(2);

		for _ in 0..120 {
			trace.advance(later, 0.0, FRAME);
		}

		assert_eq!(trace.caught(), 0);
		assert_eq!(trace.fill(), 0.0);
	}

	#[test]
	fn turning_too_fast_holds_a_light_where_it_is() {
		let mut trace = SigilTrace::new(seeded(7), 0.0);

		for _ in 0..15 {
			let aim = on_next(&trace);

			trace.advance(aim, 0.0, FRAME);
		}

		let held = trace.fill();

		for _ in 0..120 {
			let aim = on_next(&trace);

			trace.advance(aim, HuntRules::SIGIL_STEADY_SPEED + 0.1, FRAME);
		}

		assert_eq!(trace.fill(), held);
		assert_eq!(trace.caught(), 0);
	}

	#[test]
	fn looking_away_drains_a_light_twice_as_fast_as_it_filled() {
		let mut trace = SigilTrace::new(seeded(7), 0.0);

		for _ in 0..20 {
			let aim = on_next(&trace);

			trace.advance(aim, 0.0, FRAME);
		}

		let held = trace.fill();

		for _ in 0..5 {
			trace.advance(AWAY, 0.0, FRAME);
		}

		assert!(close_to(trace.fill(), held * (1.0 - 10.0 / 20.0), 1e-9));
	}

	#[test]
	fn the_view_rests_on_a_light_anywhere_inside_its_radius() {
		let trace = SigilTrace::new(seeded(7), 0.0);
		let light = trace.sigil.lights[0];
		let radius = trace.sigil.radius;

		assert!(trace.rests_on(Vec3::from_angles(light.yaw, light.pitch + radius * 0.95)));
		assert!(!trace.rests_on(Vec3::from_angles(light.yaw, light.pitch + radius * 1.05)));
	}

	#[test]
	fn the_last_light_closes_the_sigil_once() {
		let mut trace = SigilTrace::new(of(Rarity::Common), 0.0);
		let mut events = Vec::new();

		while !trace.is_closed() {
			let aim = on_next(&trace);

			events.extend(trace.advance(aim, 0.0, FRAME));
		}

		let caught: Vec<usize> = events
			.iter()
			.filter_map(|event| {
				match event {
					SigilEvent::LightCaught(index) => Some(*index),
					_ => None,
				}
			})
			.collect();

		assert_eq!(caught, [0, 1, 2]);
		assert_eq!(events.last(), Some(&SigilEvent::SigilClosed));
		assert_eq!(trace.advance(AWAY, 0.0, FRAME), []);
	}

	mod fraying {
		use super::*;

		fn caught_one() -> SigilTrace {
			let mut trace = SigilTrace::new(of(Rarity::Rare), 0.0);

			while trace.caught() == 0 {
				let aim = on_next(&trace);

				trace.advance(aim, 0.0, FRAME);
			}

			trace
		}

		#[test]
		fn a_caught_light_goes_out_when_the_next_is_too_long_coming() {
			let mut trace = caught_one();
			let fray = trace.sigil.fray_seconds.unwrap();
			let mut events = Vec::new();
			let mut seconds = 0.0;

			while events.is_empty() {
				events.extend(trace.advance(AWAY, 0.0, 0.1));
				seconds += 0.1;
			}

			assert!(close_to(seconds, fray, 0.11), "{seconds}");
			assert_eq!(events, [SigilEvent::LightLost(0)]);
			assert_eq!(trace.caught(), 0);
			assert_eq!(trace.fraying(), 0.0);
		}

		#[test]
		fn it_does_not_fray_while_the_next_is_being_held() {
			let mut trace = caught_one();

			for _ in 0..600 {
				// Resting on the next light, but turning too fast for it to
				// fill.
				let aim = on_next(&trace);

				trace.advance(aim, HuntRules::SIGIL_STEADY_SPEED + 0.1, FRAME);
			}

			assert_eq!(trace.caught(), 1);
			assert_eq!(trace.fraying(), 0.0);
		}

		#[test]
		fn a_common_sigil_never_frays() {
			let mut trace = SigilTrace::new(of(Rarity::Common), 0.0);

			while trace.caught() == 0 {
				let aim = on_next(&trace);

				trace.advance(aim, 0.0, FRAME);
			}

			for _ in 0..6000 {
				trace.advance(AWAY, 0.0, FRAME);
			}

			assert_eq!(trace.caught(), 1);
			assert_eq!(trace.fraying(), 0.0);
		}
	}

	#[test]
	fn someone_turning_at_an_easy_pace_traces_every_sigil_and_a_rarer_one_takes_longer() {
		let mut medians = HashMap::new();

		for rarity in Rarity::ALL {
			let mut times = Vec::new();

			for seed in 0..100 {
				let mut trace = SigilTrace::new(
					sigil(Draw {
						seed,
						rarity,
						..Draw::default()
					}),
					0.0,
				);
				let result = trace_sigil(&mut trace, Tracer::default());

				assert!(result.closed, "{rarity} sigil {seed}");
				assert_eq!(result.lost, 0, "{rarity} sigil {seed}");
				times.push(result.seconds);
			}

			times.sort_by(f64::total_cmp);
			medians.insert(rarity, times[times.len() / 2]);
		}

		assert!(medians[&Rarity::Common] < 5.0, "{medians:?}");
		assert!(medians[&Rarity::Epic] < 16.0, "{medians:?}");

		for (easier, harder) in HARDER {
			assert!(medians[&harder] > medians[&easier], "{medians:?}");
		}
	}

	#[test]
	fn a_slow_hand_loses_lights_on_an_epic_sigil_at_normal_and_keeps_them_at_easy() {
		let lost_at = |difficulty| {
			let mut lost = 0;

			for seed in 0..60 {
				let mut trace = SigilTrace::new(
					sigil(Draw {
						seed,
						rarity: Rarity::Epic,
						difficulty,
						..Draw::default()
					}),
					0.0,
				);

				lost += trace_sigil(
					&mut trace,
					Tracer {
						turn_speed: 0.3,
						..Tracer::default()
					},
				)
				.lost;
			}

			lost
		};

		assert!(lost_at(Difficulty::Normal) > 0);
		assert_eq!(lost_at(Difficulty::Easy), 0);
	}
}
