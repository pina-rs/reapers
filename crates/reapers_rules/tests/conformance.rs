//! The Rust rules against the Dart rules.
//!
//! `fixtures/generate.dart` asked the Dart rules a few thousand questions
//! with definite answers and wrote the answers down. Every test here asks
//! the Rust rules the same questions and expects the same answers, to the
//! last bit of every double: a hunt the phone played under one and the
//! server replays under the other must end the same way.
//!
//! One kind of answer is held a little looser by default. Dart took its
//! sines and arc tangents from the platform; the Rust rules take theirs from
//! the `libm` crate, so that the server and the Wasm game agree with each
//! other to the bit. The two libraries part in the last bit now and then,
//! so a husk's position, or the bearing between two points a few metres
//! apart, may differ from Dart's by that much ([`DRIFT`]). Everything that
//! decides a hunt (every input the bot makes, every event, every strike's
//! damage, every outcome) must still match exactly, and does. Built with
//! `--features system-math`, which takes those functions from the platform
//! as Dart did, every answer here must match to the bit.

#![allow(clippy::float_cmp, reason = "the answers must be identical, not close")]
#![allow(
	clippy::duplicate_mod,
	reason = "the pacing examples bring their own copy of the support"
)]

mod support;

#[path = "../examples/pacing.rs"]
mod pacing;
#[path = "../examples/rift_pacing.rs"]
mod rift_pacing;
#[path = "../examples/soul_pacing.rs"]
mod soul_pacing;

use std::fmt::Write;

use reapers_rules::*;
use serde_json::Value;
use support::*;

fn fixture(name: &str) -> Value {
	let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
	let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"));

	serde_json::from_str(&text).unwrap()
}

/// The 64 bits of `value`, in hex, as the fixtures write a double.
fn bits(value: f64) -> String {
	format!("{:016x}", value.to_bits())
}

/// How far a double that went through a sine or an arc tangent may stray
/// from Dart's, relative to its size, while the rules take those functions
/// from `libm` rather than from the platform as Dart did.
const DRIFT: f64 = 1e-9;

/// Whether `actual` is `expected`, with every double written as its bits
/// allowed to stray by [`DRIFT`] unless the platform's own functions are in
/// use.
fn same(actual: &Value, expected: &Value) -> bool {
	if cfg!(feature = "system-math") {
		return actual == expected;
	}

	match (actual, expected) {
		(Value::String(actual), Value::String(expected))
			if actual.len() == 16 && expected.len() == 16 =>
		{
			match (
				u64::from_str_radix(actual, 16),
				u64::from_str_radix(expected, 16),
			) {
				(Ok(actual), Ok(expected)) => {
					let (actual, expected) = (f64::from_bits(actual), f64::from_bits(expected));

					actual == expected
						|| (actual - expected).abs() <= DRIFT * expected.abs().max(1.0)
				}
				_ => actual == expected,
			}
		}
		(Value::Array(actual), Value::Array(expected)) => {
			actual.len() == expected.len()
				&& actual
					.iter()
					.zip(expected)
					.all(|(actual, expected)| same(actual, expected))
		}
		(Value::Object(actual), Value::Object(expected)) => {
			actual.len() == expected.len()
				&& actual.iter().all(|(key, actual)| {
					expected
						.get(key)
						.is_some_and(|expected| same(actual, expected))
				})
		}
		_ => actual == expected,
	}
}

fn assert_same(actual: &Value, expected: &Value, context: &str) {
	assert!(
		same(actual, expected),
		"{context}\n  left: {actual}\n right: {expected}"
	);
}

/// FNV-1a over `bytes`, 64 bits, in hex, as the fixtures write a digest.
fn digest(bytes: &[u8]) -> String {
	let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

	for &byte in bytes {
		hash ^= u64::from(byte);
		hash = hash.wrapping_mul(0x0100_0000_01b3);
	}

	format!("{hash:016x}")
}

fn int(value: &Value) -> i64 {
	value
		.as_i64()
		.unwrap_or_else(|| panic!("not an int: {value}"))
}

fn float(value: &Value) -> f64 {
	value
		.as_f64()
		.unwrap_or_else(|| panic!("not a number: {value}"))
}

fn text(value: &Value) -> &str {
	value
		.as_str()
		.unwrap_or_else(|| panic!("not text: {value}"))
}

fn list(value: &Value) -> &Vec<Value> {
	value
		.as_array()
		.unwrap_or_else(|| panic!("not a list: {value}"))
}

fn class(code: &Value) -> ReaperClass {
	ReaperClass::from_code(text(code)).unwrap()
}

fn loadout(json: &Value) -> HuntLoadout {
	let soul = json["gift"].as_str().map(|gift| {
		ReaperSoul {
			gift: ReaperGift::from_name(gift).unwrap(),
			temperament: Temperament::from_name(text(&json["temperament"])).unwrap(),
		}
	});
	let art = json["art"]
		.as_str()
		.map(|art| ArtSkill::new(Art::from_name(art).unwrap(), int(&json["artRank"])).unwrap());

	HuntLoadout {
		completed_sets: int(&json["completedSets"]),
		soul,
		soul_rank: int(&json["soulRank"]),
		art,
		..HuntLoadout::new(
			class(&json["class"]),
			int(&json["level"]),
			Weapon::parse(text(&json["weapon"])).unwrap(),
		)
	}
}

fn ints(json: &Value) -> Vec<i32> {
	list(json).iter().map(|value| int(value) as i32).collect()
}

fn plan(json: &Value) -> HuntPlan {
	HuntPlan {
		centre_centidegrees: int(&json["centre"]) as i32,
		half_width_centidegrees: int(&json["halfWidth"]) as i32,
		eye_centimetres: int(&json["eye"]) as i32,
		reach_decimetres: ints(&json["reach"]),
		cover_centimetres: ints(&json["cover"]),
		hide_decimetres: ints(&json["hide"]),
		ledges: list(&json["ledges"])
			.iter()
			.map(|value| value.as_bool().unwrap())
			.collect(),
	}
}

fn plan_json(plan: &HuntPlan) -> Value {
	serde_json::json!({
		"centre": plan.centre_centidegrees,
		"halfWidth": plan.half_width_centidegrees,
		"eye": plan.eye_centimetres,
		"reach": plan.reach_decimetres,
		"cover": plan.cover_centimetres,
		"hide": plan.hide_decimetres,
		"ledges": plan.ledges,
	})
}

fn difficulty(json: &Value) -> Difficulty {
	Difficulty::from_name(text(json)).unwrap()
}

/// An event as one line of text, with every double as its bits, as the
/// fixtures write it.
fn event(event: &HuntEvent) -> String {
	match event {
		HuntEvent::GiftStirred(gift) => format!("GiftStirred:{gift}"),
		HuntEvent::StalkEnded { ambush } => format!("StalkEnded:{ambush}"),
		HuntEvent::StrikeLanded {
			seal,
			damage,
			cadence,
			empowered,
			cleaved,
		} => {
			format!(
				"StrikeLanded:{seal}:{}:{cadence}:{empowered}:{}",
				bits(*damage),
				cleaved
					.iter()
					.map(ToString::to_string)
					.collect::<Vec<_>>()
					.join(","),
			)
		}
		HuntEvent::StrikeGlanced { damage } => format!("StrikeGlanced:{}", bits(*damage)),
		HuntEvent::SealOpened(seal) => format!("SealOpened:{seal}"),
		HuntEvent::Flinched {
			resolve_left,
			forgiven,
		} => format!("Flinched:{resolve_left}:{forgiven}"),
		HuntEvent::Struck { resolve_left } => format!("Struck:{resolve_left}"),
		HuntEvent::Leapt(toward) => format!("Leapt:{toward}"),
		HuntEvent::ArtThrown(art) => format!("ArtThrown:{art}"),
		HuntEvent::ArtLanded {
			art,
			at,
			hit,
			damage,
		} => {
			format!(
				"ArtLanded:{art}:{}:{}:{}:{hit}:{}",
				bits(at.x),
				bits(at.y),
				bits(at.z),
				bits(*damage),
			)
		}
		HuntEvent::ReapRecovered { attempts_left } => format!("ReapRecovered:{attempts_left}"),
		HuntEvent::HuskAbsorbed(grade) => format!("HuskAbsorbed:{grade}"),
		HuntEvent::HuskEscaped(cause) => format!("HuskEscaped:{cause}"),
		other => other.kind().to_owned(),
	}
}

// Seeded random ---------------------------------------------------------

#[test]
fn seeded_random_streams_match_bit_for_bit() {
	let fixture = fixture("seeded_random.json");

	for stream in list(&fixture["streams"]) {
		let seed = int(&stream["seed"]);
		let mut uints = SeededRandom::new(seed);
		let mut doubles = SeededRandom::new(seed);
		let mut ints = SeededRandom::new(seed);
		let mut ranges = SeededRandom::new(seed);
		let mut weighted = SeededRandom::new(seed);

		for expected in list(&stream["uint32"]) {
			assert_eq!(i64::from(uints.next_u32()), int(expected), "seed {seed}");
		}

		for expected in list(&stream["double"]) {
			assert_eq!(doubles.next_double(), float(expected), "seed {seed}");
		}

		for (max, expected) in [1, 2, 3, 7, 10, 100, 1000, 1 << 20, 1 << 31]
			.into_iter()
			.zip(list(&stream["int"]))
		{
			assert_eq!(ints.next_int(max), int(expected), "seed {seed}, max {max}");
		}

		for expected in list(&stream["range"]) {
			assert_eq!(
				bits(ranges.next_range(-2.5, 7.25)),
				text(expected),
				"seed {seed}"
			);
		}

		for expected in list(&stream["weighted"]) {
			let pick = weighted.pick_weighted(&[25.0, 50.0, 20.0, 5.0, 0.0, 0.5]);

			assert_eq!(pick as i64, int(expected), "seed {seed}");
		}
	}

	for case in list(&fixture["mixSeed"]) {
		let values: Vec<i64> = list(&case["values"]).iter().map(int).collect();

		assert_eq!(mix_seed(&values), int(&case["seed"]), "{values:?}");
	}

	for case in list(&fixture["hashText"]) {
		assert_eq!(
			hash_text(text(&case["text"])),
			int(&case["hash"]),
			"{}",
			case["text"]
		);
	}
}

// Genomes, feeding and progression -------------------------------------

#[test]
fn shuffled_genomes_match() {
	let fixture = fixture("genomes.json");

	for case in list(&fixture["shuffles"]) {
		let genome = ReaperGenome::shuffle(int(&case["seed"]));
		let soul = ReaperSoul::from_genome(&genome);

		assert_eq!(genome.code(), text(&case["code"]), "seed {}", case["seed"]);
		assert_eq!(genome.name(), text(&case["name"]));
		assert_eq!(soul.gift.name(), text(&case["gift"]));
		assert_eq!(soul.temperament.name(), text(&case["temperament"]));
	}

	for case in list(&fixture["parses"]) {
		let parsed = ReaperGenome::parse(text(&case["code"]))
			.ok()
			.map(|genome| genome.code());

		assert_eq!(
			parsed.as_deref(),
			case["parsed"].as_str(),
			"{}",
			case["code"]
		);
	}
}

#[test]
fn feeding_rolls_match() {
	let fixture = fixture("feeding.json");

	for (seed, expected) in list(&fixture["rolls"]).iter().enumerate() {
		assert_eq!(
			Feeding::roll(seed as i64).name(),
			text(expected),
			"seed {seed}"
		);
	}

	for case in list(&fixture["bigSeeds"]) {
		assert_eq!(
			Feeding::roll(int(&case["seed"])).name(),
			text(&case["feeding"])
		);
	}

	assert_eq!(Feeding::expected_share(), float(&fixture["expectedShare"]));
}

#[test]
fn progression_matches() {
	let fixture = fixture("progression.json");

	for case in list(&fixture["levels"]) {
		let level = int(&case["level"]);

		assert_eq!(Progression::xp_to_next(level), int(&case["xpToNext"]));
		assert_eq!(Progression::xp_for_level(level), int(&case["xpForLevel"]));
	}

	for case in list(&fixture["xp"]) {
		let xp = int(&case["xp"]);

		assert_eq!(
			Progression::level_for_xp(xp),
			int(&case["level"]),
			"xp {xp}"
		);
		assert_eq!(
			Progression::level_progress(xp),
			float(&case["progress"]),
			"xp {xp}"
		);
	}
}

// The catalogue --------------------------------------------------------

#[test]
fn the_catalogue_matches() {
	let fixture = fixture("catalogue.json");
	let listings = market_listings();

	assert_eq!(listings.len(), list(&fixture["nfts"]).len());

	for (nft, case) in listings.iter().zip(list(&fixture["nfts"])) {
		assert_eq!(nft.id(), text(&case["id"]));
		assert_eq!(nft.rarity().name(), text(&case["rarity"]));
		assert_eq!(nft.title(), text(&case["title"]));
		assert_eq!(nft.image_key(), text(&case["imageKey"]));
		assert_eq!(nft.mock_mint(), text(&case["mockMint"]));
		assert_eq!(
			serde_json::to_value(nft.look()).unwrap(),
			case["look"],
			"{}",
			nft.id()
		);
	}

	for case in list(&fixture["nftParses"]) {
		let parsed = Nft::parse(text(&case["id"])).ok().map(|nft| nft.id());

		assert_eq!(parsed.as_deref(), case["parsed"].as_str(), "{}", case["id"]);
	}

	let weapons = armory_weapons();

	assert_eq!(weapons.len(), list(&fixture["weapons"]).len());

	for (weapon, case) in weapons.iter().zip(list(&fixture["weapons"])) {
		assert_eq!(weapon.id(), text(&case["id"]));
		assert_eq!(weapon.level_required(), int(&case["levelRequired"]));

		for reaper_class in ReaperClass::ALL {
			assert_eq!(
				weapon.price_for(reaper_class),
				int(&case["prices"][reaper_class.code()])
			);
		}
	}

	for case in list(&fixture["weaponParses"]) {
		let parsed = Weapon::parse(text(&case["id"]))
			.ok()
			.map(|weapon| weapon.id());

		assert_eq!(parsed.as_deref(), case["parsed"].as_str(), "{}", case["id"]);
	}

	for case in list(&fixture["times"]) {
		let at = Timestamp::parse(text(&case["text"])).unwrap();

		assert_eq!(at.micros(), int(&case["micros"]), "{}", case["text"]);
		assert_eq!(at.to_iso8601(), text(&case["iso"]));
	}
}

// Stats ----------------------------------------------------------------

#[test]
fn derived_stats_match() {
	let fixture = fixture("stats.json");

	for case in list(&fixture["cases"]) {
		let stats = HuntStats::derive(&loadout(&case["loadout"]), float(&case["leniency"]))
			.unwrap()
			.at(difficulty(&case["difficulty"]));
		let expected = &case["stats"];
		let label = &case["loadout"];

		assert_eq!(
			bits(stats.strike_damage),
			text(&expected["strikeDamage"]),
			"{label}"
		);
		assert_eq!(
			bits(stats.reach_radians),
			text(&expected["reachRadians"]),
			"{label}"
		);
		assert_eq!(stats.swing_ticks, int(&expected["swingTicks"]), "{label}");
		assert_eq!(stats.resolve, int(&expected["resolve"]), "{label}");
		assert_eq!(
			stats.capture_ticks,
			int(&expected["captureTicks"]),
			"{label}"
		);
		assert_eq!(
			bits(stats.reap_tolerance_radians),
			text(&expected["reapToleranceRadians"]),
			"{label}"
		);
		assert_eq!(
			bits(stats.flinch_speed),
			text(&expected["flinchSpeed"]),
			"{label}"
		);
		assert_eq!(
			bits(stats.parry_speed),
			text(&expected["parrySpeed"]),
			"{label}"
		);
		assert_eq!(stats.perk.name(), text(&expected["perk"]));
		assert_eq!(stats.gift.map(ReaperGift::name), expected["gift"].as_str());
		assert_eq!(stats.soul_rank, int(&expected["soulRank"]));
		assert_eq!(stats.knock_ticks, int(&expected["knockTicks"]));
		assert_eq!(bits(stats.glance_share), text(&expected["glanceShare"]));
	}

	for case in list(&fixture["riftWards"]) {
		let circle: Vec<HuntStats> = list(&case["levels"])
			.iter()
			.enumerate()
			.map(|(index, level)| {
				let reaper_class = ReaperClass::ALL[index % ReaperClass::ALL.len()];
				let level = int(level);

				HuntStats::derive(
					&HuntLoadout::new(
						reaper_class,
						level,
						Weapon::new(reaper_class.weapon(), Temper::best_for(level)),
					),
					1.0,
				)
				.unwrap()
			})
			.collect();

		assert_eq!(
			bits(RiftRules::ward_for(&circle)),
			text(&case["ward"]),
			"{}",
			case["levels"]
		);
	}
}

// Hunts ----------------------------------------------------------------

/// Plays one fixture hunt in Rust, checking it against what Dart wrote as
/// it goes, so that the first tick the two part ways is the one reported.
#[allow(clippy::too_many_lines, reason = "one hunt, checked as it is played")]
fn play_hunt(case: &Value) {
	let name = text(&case["name"]);
	let vessel = Vessel::from_name(text(&case["vessel"])).unwrap();
	let rarity = Rarity::from_name(text(&case["rarity"])).unwrap();
	let stats = HuntStats::derive(&loadout(&case["loadout"]), float(&case["leniency"])).unwrap();
	let plan = plan(&case["plan"]);
	let difficulty = difficulty(&case["difficulty"]);
	let seed = int(&case["seed"]);
	let mut sim = HuntSim::new(
		seed,
		vessel,
		rarity.ward(),
		&stats,
		plan.clone(),
		difficulty,
	)
	.unwrap();
	let style = (!case["bot"].is_null()).then(|| {
		let bot = &case["bot"];

		HuntBotStyle {
			turn_speed: float(&bot["turnSpeed"]),
			reap_speed: float(&bot["reapSpeed"]),
			reaction_ticks: int(&bot["reactionTicks"]),
			miss_every: int(&bot["missEvery"]),
			charge_nerve: float(&bot["chargeNerve"]),
		}
	});
	let mut bot = HuntBot::new(&sim, style.unwrap_or_default());
	let mut inputs = Vec::new();
	let mut lines = Vec::new();
	let mut kinds = serde_json::Map::new();
	let mut checkpoints = list(&case["checkpoints"]).iter();
	let expected_events = case["events"].as_array();

	while !sim.is_over() && sim.tick() < HuntLog::MAX_TICKS as i64 {
		let tick = sim.tick();
		let input = if style.is_some() {
			bot.decide(&sim)
		} else {
			HuntInput::new(Aim::AHEAD, 0)
		};
		let events = sim.step(&input);

		inputs.push(input);

		for happened in &events {
			let line = event(happened);
			let kind = line.split(':').next().unwrap().to_owned();
			let count = kinds.get(&kind).and_then(Value::as_i64).unwrap_or(0);

			kinds.insert(kind, Value::from(count + 1));
			lines.push(format!("{tick}|{line}"));

			if let Some(expected) = expected_events {
				assert_eq!(
					Some(lines.last().unwrap().as_str()),
					expected.get(lines.len() - 1).and_then(Value::as_str),
					"{name}: event {}",
					lines.len() - 1,
				);
			}
		}

		if sim.tick() % 120 == 0 || sim.is_over() {
			let pose = sim.pose();
			let expected = checkpoints
				.next()
				.unwrap_or_else(|| panic!("{name}: runs past Dart"));
			let actual = serde_json::json!({
				"tick": sim.tick(),
				"phase": sim.phase().name(),
				"ward": bits(sim.ward()),
				"resolve": sim.resolve(),
				"cadence": sim.cadence(),
				"x": bits(pose.position.x),
				"y": bits(pose.position.y),
				"z": bits(pose.position.z),
				"heading": bits(pose.heading),
				"footing": pose.footing.name(),
				"leaping": pose.leaping,
				"ground": pose.ground.map(bits),
				"speed": bits(sim.speed()),
				"artCharges": sim.art_charges(),
				"awareness": bits(sim.awareness()),
				"reapProgress": bits(sim.reap_progress()),
			});

			assert_same(&actual, expected, &format!("{name}: tick {}", sim.tick()));
		}
	}

	assert!(checkpoints.next().is_none(), "{name}: ends before Dart");

	let outcome = sim.outcome();
	let log = HuntLog {
		inputs,
		plan,
		difficulty,
	};

	assert_eq!(
		serde_json::to_value(outcome).unwrap(),
		case["outcome"],
		"{name}"
	);
	assert_eq!(sim.tick(), int(&case["ticks"]), "{name}");
	assert_eq!(
		digest(&log.encode()),
		text(&case["logDigest"]),
		"{name}: the bot's inputs"
	);
	assert_eq!(
		digest(lines.join("\n").as_bytes()),
		text(&case["eventDigest"]),
		"{name}"
	);
	assert_eq!(Value::Object(kinds), case["eventKinds"], "{name}");
	assert_eq!(
		HuntSim::replay(
			seed,
			vessel,
			rarity.ward(),
			&stats,
			&HuntLog::decode(&log.encode()).unwrap()
		),
		outcome,
		"{name}: the replay",
	);
}

#[test]
fn every_hunt_plays_out_as_it_did_in_dart() {
	let fixture = fixture("hunts.json");

	assert!(list(&fixture).len() >= 60);

	for case in list(&fixture) {
		play_hunt(case);
	}
}

fn base64(text: &str) -> Vec<u8> {
	const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
	let mut bytes = Vec::new();
	let mut buffer = 0_u32;
	let mut held = 0;

	for byte in text.bytes().filter(|&byte| byte != b'=') {
		let value = ALPHABET.iter().position(|&letter| letter == byte).unwrap() as u32;

		buffer = buffer << 6 | value;
		held += 6;

		if held >= 8 {
			held -= 8;
			bytes.push((buffer >> held) as u8);
		}
	}

	bytes
}

#[test]
fn hunt_logs_read_and_write_as_dart_wrote_them() {
	let fixture = fixture("hunt_logs.json");

	for case in list(&fixture["logs"]) {
		let bytes = base64(text(&case["bytes"]));
		let log = HuntLog::decode(&bytes).unwrap();

		assert_eq!(log.inputs.len() as i64, int(&case["ticks"]));
		assert_eq!(plan_json(&log.plan), case["plan"]);
		assert_eq!(log.difficulty, difficulty(&case["difficulty"]));
		assert_eq!(log.encode(), bytes, "{}", case["name"]);

		for (input, expected) in log.inputs.iter().zip(list(&case["firstInputs"])) {
			let actual = serde_json::json!({
				"aim": [input.aim.yaw_centidegrees, input.aim.pitch_centidegrees],
				"turnSpeed": input.turn_speed,
				"taps": input.taps.iter().map(|tap| [tap.yaw_centidegrees, tap.pitch_centidegrees]).collect::<Vec<_>>(),
				"focus": input.focus,
			});

			assert_eq!(&actual, expected);
		}
	}

	let replay = &fixture["replay"];
	let stats = HuntStats::derive(&loadout(&replay["loadout"]), 1.0).unwrap();
	let log = HuntLog::decode(&base64(text(&replay["bytes"]))).unwrap();
	let ward = Rarity::from_name(text(&replay["rarity"])).unwrap().ward();
	let vessel = Vessel::from_name(text(&replay["vessel"])).unwrap();
	let seed = int(&replay["seed"]);

	assert_eq!(
		serde_json::to_value(HuntSim::replay(seed, vessel, ward, &stats, &log)).unwrap(),
		replay["outcome"],
	);

	let truncated = HuntLog {
		inputs: log.inputs[..400].to_vec(),
		..log.clone()
	};

	assert_eq!(
		serde_json::to_value(HuntSim::replay(seed, vessel, ward, &stats, &truncated)).unwrap(),
		replay["truncatedOutcome"],
	);

	for case in list(&fixture["malformed"]) {
		let error = HuntLog::decode(&base64(text(&case["bytes"])))
			.err()
			.map(|error| error.to_string());

		assert_eq!(error.as_deref(), case["error"].as_str(), "{}", case["name"]);
	}
}

// Sigils ---------------------------------------------------------------

#[test]
fn sigils_are_drawn_and_traced_as_in_dart() {
	let fixture = fixture("sigils.json");

	for case in list(&fixture["draws"]) {
		let rarity = Rarity::from_name(text(&case["rarity"])).unwrap();
		let gait = Gait::from_name(text(&case["gait"])).unwrap();
		let sigil = Sigil::draw(
			int(&case["seed"]),
			&rarity.sigil(),
			gait,
			difficulty(&case["difficulty"]),
		);
		let lights: Vec<Value> = sigil
			.lights
			.iter()
			.map(|light| serde_json::json!([bits(light.yaw), bits(light.pitch)]))
			.collect();

		assert_eq!(Value::from(lights), case["lights"], "{case}");
		assert_eq!(bits(sigil.radius), text(&case["radius"]));
		assert_eq!(bits(sigil.hold_seconds), text(&case["holdSeconds"]));
		assert_eq!(
			sigil.fray_seconds.map(bits).as_deref(),
			case["fraySeconds"].as_str()
		);
		assert_eq!(sigil.crossings() as i64, int(&case["crossings"]));
		assert_eq!(bits(sigil.half_arc()), text(&case["halfArc"]));
	}

	for case in list(&fixture["traces"]) {
		let rarity = Rarity::from_name(text(&case["rarity"])).unwrap();
		let gait = Gait::from_name(text(&case["gait"])).unwrap();
		let seed = int(&case["seed"]);
		let mut trace = SigilTrace::new(
			Sigil::draw(seed, &rarity.sigil(), gait, Difficulty::Normal),
			radians(10.0 * seed as f64),
		);
		let result = trace_sigil(
			&mut trace,
			Tracer {
				turn_speed: float(&case["turnSpeed"]),
				wobble: float(&case["wobble"]),
				..Tracer::default()
			},
		);

		assert_eq!(result.closed, case["closed"].as_bool().unwrap(), "{case}");
		assert_eq!(bits(result.seconds), text(&case["seconds"]), "{case}");
		assert_eq!(result.lost as i64, int(&case["lost"]));
		assert_eq!(trace.caught() as i64, int(&case["caught"]));
	}
}

// Geo ------------------------------------------------------------------

fn point(json: &Value) -> GeoPoint {
	GeoPoint::new(float(&json[0]), float(&json[1]))
}

#[test]
fn geography_matches() {
	let fixture = fixture("geo.json");

	for case in list(&fixture["pairs"]) {
		let from = point(&case["from"]);
		let to = point(&case["to"]);
		let moved = from.moved(float(&case["move"][0]), float(&case["move"][1]));
		let sighting = RiftSighting::of(&to, &from, float(&case["north"]));

		let context = case.to_string();

		assert_same(
			&bits(from.distance_to(&to)).into(),
			&case["distance"],
			&context,
		);
		assert_same(
			&bits(from.bearing_to(&to)).into(),
			&case["bearing"],
			&context,
		);
		assert_same(
			&serde_json::json!([bits(moved.latitude), bits(moved.longitude)]),
			&case["moved"],
			&context,
		);
		assert_same(
			&serde_json::to_value(
				sighting.map(|sighting| [bits(sighting.bearing), bits(sighting.metres)]),
			)
			.unwrap(),
			&case["sighting"],
			&context,
		);
	}

	for case in list(&fixture["anchors"]) {
		let site = RiftSite::from_name(text(&case["site"])).unwrap();
		let standing = GeoStanding {
			at: point(&case["at"]),
			facing: float(&case["facing"]),
			accuracy: 5.0,
			observed_at: None,
		};
		let anchor = site.anchor_for(&standing);

		assert_same(
			&serde_json::json!([bits(anchor.latitude), bits(anchor.longitude)]),
			&case["anchor"],
			&case.to_string(),
		);
	}
}

// Rifts ----------------------------------------------------------------

#[test]
fn rifts_play_out_as_they_did_in_dart() {
	let fixture = fixture("rifts.json");

	for case in list(&fixture["rifts"]) {
		let expected = &case["play"];
		let name = text(&expected["name"]);
		let reapers = list(&case["reapers"])
			.iter()
			.map(|reaper| {
				entrant(
					text(&reaper["name"]),
					class(&reaper["class"]),
					int(&reaper["level"]),
				)
			})
			.collect();
		let site = RiftSite::from_name(text(&case["site"])).unwrap();
		let mut table = RiftTable::with(reapers, site, int(&case["seed"]), careful);
		let walk_away_at = case["walkAwayAt"].as_i64();
		let mut frames = list(&expected["frames"]).iter();

		table.begin();

		while !table.room.phase().is_over() {
			if Some(table.room.tick()) == walk_away_at {
				table.walk_away(table.seats.len() - 1);
			}

			table.step();

			if table.room.tick() % 600 == 0 {
				let frame =
					serde_json::to_value(table.room.frame_for(table.seats[0]).unwrap()).unwrap();

				assert_eq!(
					Some(&frame),
					frames.next(),
					"{name}: tick {}",
					table.room.tick()
				);
			}
		}

		assert!(frames.next().is_none(), "{name}: ends before Dart");
		assert_eq!(
			table.room.phase().name(),
			text(&expected["phase"]),
			"{name}"
		);
		assert_eq!(table.room.tick(), int(&expected["tick"]), "{name}");
		assert_eq!(
			bits(table.room.ward_max()),
			text(&expected["wardMax"]),
			"{name}"
		);
		assert_eq!(bits(table.room.ward()), text(&expected["ward"]), "{name}");

		for (&seat, reward) in table.seats.iter().zip(list(&expected["rewards"])) {
			let paid = table.room.reward_for(seat);

			assert_eq!(paid.xp, int(&reward["xp"]), "{name}");
			assert_eq!(paid.anima, int(&reward["anima"]), "{name}");
		}

		assert_eq!(
			serde_json::to_value(table.room.frame_for(table.seats[0]).unwrap()).unwrap(),
			expected["final"],
			"{name}"
		);
	}

	let mut room = RiftRoom::new(RiftSite::ShibuyaCrossing, 0x51DE);
	let seat = room
		.join(entrant("Vesper", ReaperClass::Harvester, 13))
		.unwrap();

	room.report(seat, &RiftReport::readiness(true));
	room.advance(RiftRules::COUNTDOWN_TICKS + 1);

	assert_eq!(
		serde_json::to_value(room.frame_for(seat).unwrap()).unwrap(),
		fixture["shades"]
	);
}

// The player -----------------------------------------------------------

/// Plays the fixture's scripted session, step by step, and checks each
/// state the rules return against the one Dart returned.
#[test]
#[allow(clippy::too_many_lines, reason = "one script, in order")]
fn a_player_session_matches_step_for_step() {
	let fixture = fixture("player.json");
	let mut steps = list(&fixture["steps"]).iter();
	let mut settlements = list(&fixture["settlements"]).iter();
	let now = Timestamp::utc(2026, 10, 1, 12, 0, 0);
	let mut state = PlayerState::initial();
	let ape = Nft::new(NftCollection::DegenApes, 2);
	let bird = Nft::new(NftCollection::Thugbirdz, 8);
	let penguin = Nft::new(NftCollection::PeskyPenguins, 7);

	macro_rules! record {
		($name:expr, $request:expr $(,)?) => {{
			let result = $request;
			let expected = steps.next().unwrap();

			assert_eq!(text(&expected["step"]), $name);

			match result {
				Ok(next) => {
					assert_eq!(
						serde_json::to_value(&next).unwrap(),
						expected["state"],
						"{}",
						$name
					);
					state = next;
				}
				Err(breach) => {
					assert_eq!(
						breach.refusal.name(),
						text(&expected["refusal"]),
						"{}",
						$name
					)
				}
			}
		}};
	}

	record!("release before the oath", state.release(ape, 1, now));
	record!("take the oath", state.take_oath(0xBEEF));
	record!("take it again", state.take_oath(1));
	record!("acquire an ape", state.acquire(ape));
	record!("acquire it again", state.acquire(ape));
	record!("acquire a bird", state.acquire(bird));
	record!("acquire a penguin", state.acquire(penguin));
	record!("release the ape", state.release(ape, 41, now));
	record!("release the bird", state.release(bird, 42, now));

	for sighting in state.sightings().to_vec() {
		let stats = HuntStats::derive(&state.loadout(), 1.0).unwrap();
		let mut sim = HuntSim::open(
			sighting.seed,
			sighting.nft.collection.vessel(),
			sighting.nft.rarity().ward(),
			&stats,
		);
		let mut bot = HuntBot::human(&sim);

		bot.play(&mut sim);

		let settlement = state
			.judge_hunt(
				sighting.id,
				&HuntLog::new(bot.recorded),
				now.plus_seconds(60),
				sighting.seed * 3 + 1,
				&Judging {
					leniency: 1.03,
					..Judging::default()
				},
			)
			.unwrap();

		assert_eq!(
			serde_json::to_value(&settlement).unwrap(),
			*settlements.next().unwrap()
		);

		let expected = steps.next().unwrap();

		assert_eq!(
			text(&expected["step"]),
			format!("judge {}", sighting.nft.id())
		);
		assert_eq!(
			serde_json::to_value(&settlement.state).unwrap(),
			expected["state"]
		);
		state = settlement.state;
	}

	record!("release the penguin", state.release(penguin, 43, now));

	let last = state.sightings().last().unwrap().id;

	record!(
		"hunt too early",
		state.huntable(last, now).map(|_| state.clone())
	);
	record!(
		"settle an escape",
		state.settle_hunt(
			last,
			&HuntOutcome::escaped_by(EscapeCause::Time, false, 4500, HuntDeeds::default()),
			now.plus_seconds(120),
			5,
		),
	);
	record!(
		"settle an absorption",
		state.settle_hunt(
			last,
			&HuntOutcome::absorbed_with(
				ReapGrade::Perfect,
				true,
				1800,
				HuntDeeds {
					holds: 2,
					parries: 1,
					max_cadence: 6,
				},
			),
			now.plus_seconds(180),
			0xFEED_FACE,
		),
	);
	record!(
		"sign the penguin",
		RuleResult::Ok(state.sign_absorption(penguin, "c2lnbmVk"))
	);

	let axe = Weapon::new(WeaponKind::Axe, Temper::Shrouded);

	record!("buy a heavy weapon too soon", state.buy_weapon(axe));
	record!("settle a rift", state.settle_rift(400, 900, None));
	record!("buy an axe", state.buy_weapon(axe));
	record!("wield it", state.wield(axe));
	record!("buy an outfit", state.buy_outfit(Outfit::Sworn));
	record!("wear it", state.wear(Outfit::Sworn));
	record!("attune rime", state.attune(Art::Rime));
	record!("attune gale", state.attune(Art::Gale));
	record!(
		"enter a rift",
		state
			.entrant_for(RiftSite::LondonBridge)
			.map(|_| state.clone()),
	);

	assert!(steps.next().is_none());

	let loadout = state.loadout();
	let expected = &fixture["loadout"];

	assert_eq!(loadout.reaper_class.code(), text(&expected["class"]));
	assert_eq!(loadout.level, int(&expected["level"]));
	assert_eq!(loadout.weapon.id(), text(&expected["weapon"]));
	assert_eq!(loadout.completed_sets, int(&expected["completedSets"]));
	assert_eq!(
		loadout.soul.map(|soul| soul.gift.name()),
		expected["gift"].as_str()
	);
	assert_eq!(loadout.soul_rank, int(&expected["soulRank"]));
	assert_eq!(
		loadout.art.map(|art| art.art.name()),
		expected["art"].as_str()
	);
	assert_eq!(
		loadout.art.map(|art| art.rank),
		expected["artRank"].as_i64()
	);
	assert_eq!(
		state.entrant_for(RiftSite::LondonBridge).unwrap().name,
		text(&fixture["entrant"])
	);

	let migrated: PlayerState = serde_json::from_value(fixture["schemaTwo"].clone()).unwrap();

	assert_eq!(
		serde_json::to_value(&migrated).unwrap(),
		fixture["schemaTwoMigrated"]
	);

	let reread: PlayerState = serde_json::from_value(fixture["schemaTwoMigrated"].clone()).unwrap();

	assert_eq!(reread, migrated);
}

#[test]
fn an_absorption_carries_every_field_its_receipt_signs() {
	let absorption = Absorption {
		nft: Nft::new(NftCollection::Thugbirdz, 3),
		grade: ReapGrade::Perfect,
		feeding: Feeding::Rich,
		xp: 0x0102_0304,
		anima: 77,
		nonce: 0xCAFE_F00D,
		at: Timestamp::utc(2026, 10, 1, 12, 0, 0),
		signature: None,
	};
	let mut summary = String::new();

	write!(
		summary,
		"{} {} {} {} {} {}",
		absorption.grade.index(),
		absorption.feeding.index(),
		absorption.xp,
		absorption.anima,
		absorption.at.unix_seconds(),
		absorption.nonce,
	)
	.unwrap();

	// The receipt's bytes 72..98, as reapers_receipt lays them out: the
	// grade, the feeding, the experience, the anima, the time and the
	// nonce.
	assert_eq!(summary, "2 2 16909060 77 1790856000 3405705229");
}

// The pacing tools -----------------------------------------------------

#[test]
fn the_pacing_tool_prints_what_dart_printed() {
	let expected = std::fs::read_to_string(format!(
		"{}/tests/fixtures/pacing.txt",
		env!("CARGO_MANIFEST_DIR")
	))
	.unwrap();

	assert_eq!(pacing::report(), expected);
}

#[test]
fn the_rift_pacing_tool_prints_what_dart_printed() {
	let expected = std::fs::read_to_string(format!(
		"{}/tests/fixtures/rift_pacing.txt",
		env!("CARGO_MANIFEST_DIR")
	))
	.unwrap();

	assert_eq!(rift_pacing::report(), expected);
}

#[test]
fn the_soul_pacing_tool_prints_what_dart_printed() {
	let expected = std::fs::read_to_string(format!(
		"{}/tests/fixtures/soul_pacing.txt",
		env!("CARGO_MANIFEST_DIR")
	))
	.unwrap();

	assert_eq!(soul_pacing::report(), expected);
}
