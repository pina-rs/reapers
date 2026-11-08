//! Prints how long a human-paced bot takes to break each rarity with each
//! class, for tuning. Run with `cargo run --release --example pacing`.

#[path = "../tests/support/mod.rs"]
mod support;

use std::fmt::Write;

use reapers_rules::HuntBot;
use reapers_rules::HuntRules;
use reapers_rules::Rarity;
use reapers_rules::ReaperClass;
use reapers_rules::Temper;
use support::SimFor;

/// The table, as the Dart tool printed it.
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

#[allow(dead_code, reason = "the tests include this file for its report")]
fn main() {
	print!("{}", report());
}
