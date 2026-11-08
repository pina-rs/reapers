//! Reproducible birth-build trials; keep stdout with the research record.
//! Run with `cargo run --release --example soul_pacing`.

use std::fmt::Write;

use reapers_rules::HuntBot;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntRules;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::Rarity;
use reapers_rules::ReaperGift;
use reapers_rules::ReaperSoul;
use reapers_rules::Temper;
use reapers_rules::Temperament;
use reapers_rules::Vessel;
use reapers_rules::Weapon;

/// `value` with `digits` decimals, rounded half away from zero on its exact
/// value, as Dart's `toStringAsFixed` writes it.
fn to_fixed(value: f64, digits: usize) -> String {
	let exact = format!("{value:.1100}");
	let (whole, fraction) = exact.split_once('.').unwrap();
	let kept = format!("{whole}{}", &fraction[..digits]);
	let up = fraction.as_bytes()[digits] >= b'5';
	let mut number: u128 = kept.trim_start_matches('-').parse().unwrap();

	if up {
		number += 1;
	}

	let text = format!("{number:0width$}", width = digits + 1);
	let (head, tail) = text.split_at(text.len() - digits);
	let sign = if value < 0.0 { "-" } else { "" };

	format!("{sign}{head}.{tail}")
}

/// The table, as the Dart tool printed it.
pub fn report() -> String {
	let mut out = String::new();

	writeln!(out, "# Soul pacing trials").unwrap();
	writeln!(out).unwrap();
	writeln!(
		out,
		"Seeds 1–8, rare penguin, shrouded weapon, normal difficulty. Human bot: 16-tick reaction and \
		 every seventh tap wide. Each row spans all four temperaments. Level 20 assumes 30 earned marks \
		 in the gift’s path; this is potential, not free progress."
	)
	.unwrap();
	writeln!(out).unwrap();
	writeln!(
		out,
		"Times include stalk and reap, averaged only over absorptions. Careful play understates \
		 defensive powers. This does not measure physical comfort, cooperative balance or player \
		 enjoyment."
	)
	.unwrap();
	writeln!(out).unwrap();
	writeln!(
		out,
		"| Gift | Level / rank | Wins / trials | Mean seconds |"
	)
	.unwrap();
	writeln!(out, "| --- | --- | --- | --- |").unwrap();

	for gift in ReaperGift::ALL {
		for (level, rank) in [(1, 0), (20, 3)] {
			let mut wins = 0;
			let mut ticks = 0;

			for temperament in Temperament::ALL {
				for seed in 1..=8 {
					let loadout = HuntLoadout {
						soul: Some(ReaperSoul { gift, temperament }),
						soul_rank: rank,
						..HuntLoadout::new(
							gift.reaper_class(),
							level,
							Weapon::new(gift.reaper_class().weapon(), Temper::Shrouded),
						)
					};
					let stats = HuntStats::derive(&loadout, 1.0).unwrap();
					let mut sim = HuntSim::open(seed, Vessel::Penguin, Rarity::Rare.ward(), &stats);
					let outcome = HuntBot::human(&sim)
						.play(&mut sim)
						.unwrap_or_else(|| panic!("Unfinished {} trial", gift.name()));

					if outcome.absorbed() {
						wins += 1;
						ticks += outcome.ticks;
					}
				}
			}

			let mean = if wins == 0 {
				"—".to_owned()
			} else {
				to_fixed(
					ticks as f64 / f64::from(wins) / HuntRules::TICK_RATE as f64,
					1,
				)
			};

			writeln!(
				out,
				"| {} | {level} / {rank} | {wins} / 32 | {mean} |",
				gift.name()
			)
			.unwrap();
		}
	}

	out
}

#[allow(dead_code, reason = "the tests include this file for its report")]
fn main() {
	print!("{}", report());
}
