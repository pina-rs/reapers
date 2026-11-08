//! Prints how rifts go for circles of different sizes and levels, for
//! tuning. Run with `cargo run --release --example rift_pacing`.

#[path = "../tests/support/mod.rs"]
mod support;

use std::fmt::Write;

use reapers_rules::HuntBot;
use reapers_rules::HuntBotStyle;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::ReaperClass;
use reapers_rules::RiftPhase;
use reapers_rules::RiftRules;
use reapers_rules::RiftSite;
use support::RiftTable;
use support::careful;
use support::entrant;

/// A reaper who stands in the circle and never strikes.
fn idle(sim: &HuntSim, _person: usize) -> HuntBot {
	HuntBot::new(
		sim,
		HuntBotStyle {
			reaction_ticks: 1 << 30,
			..HuntBotStyle::CAREFUL
		},
	)
}

/// The table, as the Dart tool printed it.
pub fn report() -> String {
	let mut out = String::new();

	for level in [1, 6, 12, 20] {
		writeln!(out, "--- level {level} ---").unwrap();

		for people in [1, 2, 4] {
			let mut sealed = 0;
			let mut seconds = 0;
			let mut xp = 0;
			let seeds = 8;

			for seed in 1..=seeds {
				let reapers = (0..people)
					.map(|person| {
						entrant(
							"Vesper",
							ReaperClass::ALL[(seed as usize + person) % ReaperClass::ALL.len()],
							level,
						)
					})
					.collect();
				let mut table = RiftTable::with(reapers, RiftSite::LondonBridge, seed, careful);

				table.play();

				if table.room.phase() == RiftPhase::Sealed {
					sealed += 1;
					seconds += table.room.tick() / HuntRules::TICK_RATE;
					xp += table.room.reward_for(table.seats[0]).xp;
				}
			}

			let average = if sealed == 0 {
				"--".to_owned()
			} else {
				format!("{}s, {} xp", seconds / sealed, xp / sealed)
			};

			writeln!(
				out,
				"{people} reaper{} + {} shades: sealed {sealed}/{seeds}  {average}",
				if people == 1 { " " } else { "s" },
				RiftRules::CIRCLE - people,
			)
			.unwrap();
		}
	}

	// What a reaper who only watches does to the circle.
	let mut carried = 0;

	for seed in 1..=8 {
		let mut table = RiftTable::with(
			vec![entrant("Vesper", ReaperClass::Harvester, 6)],
			RiftSite::LondonBridge,
			seed,
			idle,
		);

		table.play();

		if table.room.phase() == RiftPhase::Sealed {
			carried += 1;
		}
	}

	writeln!(out, "one idle reaper + 3 shades: sealed {carried}/8").unwrap();

	out
}

#[allow(dead_code, reason = "the tests include this file for its report")]
fn main() {
	print!("{}", report());
}
