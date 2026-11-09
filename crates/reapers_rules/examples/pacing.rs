//! Prints how long a human-paced bot takes to break each rarity with each
//! class, for tuning. Run with `cargo run --release --example pacing`.

#[path = "../tests/support/mod.rs"]
mod support;

use std::fmt::Write;

use reapers_rules::HuntBot;
use reapers_rules::HuntPlan;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::Temper;
use support::SimFor;

/// The table, as `tests/fixtures` holds it.
pub fn report() -> String {
	let mut out = String::new();

	for (level, temper) in [
		(1, Temper::Shrouded),
		(5, Temper::Sworn),
		(10, Temper::Scorched),
		(16, Temper::Hollowed),
	] {
		writeln!(out, "--- level {level}, {} ---", temper.name()).unwrap();

		for reaper_class in ReaperClass::ALL {
			let mut cells = String::new();

			for rarity in Rarity::ALL {
				let mut won = 0;
				let mut seconds = 0;
				let seeds = 8;

				for seed in 1..=seeds {
					let mut sim = SimFor {
						seed,
						rarity,
						reaper_class,
						level,
						temper,
						..SimFor::default()
					}
					.sim();
					let outcome = HuntBot::human(&sim).play(&mut sim).unwrap();

					if outcome.absorbed() {
						won += 1;
						seconds += outcome.ticks / HuntRules::TICK_RATE;
					}
				}

				let average = if won == 0 {
					"--".to_owned()
				} else {
					format!("{}s", seconds / won)
				};

				write!(
					cells,
					"{:<22}",
					format!("{}: {won}/{seeds} {average}", rarity.name())
				)
				.unwrap();
			}

			writeln!(out, "{:<10} {cells}", reaper_class.name()).unwrap();
		}
	}

	out
}

/// The same bot against the same penguin in a furnished room the phone
/// read as a grid, looked at across the half of the room ahead: the
/// husk walks the floor, the chair, the desk and the sofa instead of the
/// open plan's slices.
#[allow(dead_code, reason = "the tests include this file for its report")]
pub fn room_report() -> String {
	let mut out = String::new();
	let plan = HuntPlan {
		room: Some(support::room::furnished_room()),
		..HuntPlan::new(0, 9000)
	};

	for (level, temper) in [(1, Temper::Shrouded), (10, Temper::Scorched)] {
		writeln!(out, "--- in a room, level {level}, {} ---", temper.name()).unwrap();

		for reaper_class in ReaperClass::ALL {
			let mut cells = String::new();

			for rarity in Rarity::ALL {
				let mut won = 0;
				let mut seconds = 0;
				let seeds = 8;

				for seed in 1..=seeds {
					let options = SimFor {
						seed,
						rarity,
						reaper_class,
						level,
						temper,
						..SimFor::default()
					};
					let mut sim = HuntSim::new(
						seed,
						options.vessel,
						rarity,
						rarity.ward(),
						&options.stats(),
						plan.clone(),
						options.difficulty,
					)
					.unwrap();
					let outcome = HuntBot::human(&sim).play(&mut sim).unwrap();

					if outcome.absorbed() {
						won += 1;
						seconds += outcome.ticks / HuntRules::TICK_RATE;
					}
				}

				let average = if won == 0 {
					"--".to_owned()
				} else {
					format!("{}s", seconds / won)
				};

				write!(
					cells,
					"{:<22}",
					format!("{}: {won}/{seeds} {average}", rarity.name())
				)
				.unwrap();
			}

			writeln!(out, "{:<10} {cells}", reaper_class.name()).unwrap();
		}
	}

	out
}

#[allow(dead_code, reason = "the tests include this file for its report")]
fn main() {
	print!("{}", report());
	print!("{}", room_report());
}
