//! Every listed creature has telegraphed attacks that can hurt the reaper.

use reapers_rules::HuntEvent;
use reapers_rules::HuntInput;
use reapers_rules::HuntLoadout;
use reapers_rules::HuntLog;
use reapers_rules::HuntSim;
use reapers_rules::HuntStats;
use reapers_rules::market_listings;

#[test]
fn every_solo_nft_can_charge_shriek_and_remove_resolve() {
	let stats = HuntStats::derive(&HuntLoadout::BORROWED, 1.0).unwrap();
	let mut schedules = std::collections::BTreeSet::new();
	for (seed, nft) in market_listings().into_iter().enumerate() {
		let mut sim = HuntSim::open(
			seed as i64 + 100,
			nft.collection.vessel(),
			nft.rarity(),
			nft.rarity().ward(),
			&stats,
		);
		let resolve = sim.resolve();
		let mut charged = false;
		let mut shrieked = false;
		let mut struck = false;
		let mut schedule = Vec::new();
		while !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
			let input = HuntInput::still(sim.pose().direction(), 0.0);
			for event in sim.step(&input) {
				match event {
					HuntEvent::ChargeBegan => {
						charged = true;
						schedule.push((sim.tick(), 0));
					}
					HuntEvent::ShriekWindup => {
						shrieked = true;
						schedule.push((sim.tick(), 1));
					}
					HuntEvent::Struck { resolve_left, .. } => {
						struck = true;
						assert!(resolve_left < resolve);
					}
					_ => {}
				}
			}
		}
		assert!(
			charged && shrieked && struck,
			"{} never used its damaging pattern",
			nft.id()
		);
		assert!(sim.resolve() < resolve, "{} dealt no damage", nft.id());
		schedules.insert(schedule);
	}
	assert!(
		schedules.len() > 1,
		"different encounters must not all share one timetable"
	);
}
