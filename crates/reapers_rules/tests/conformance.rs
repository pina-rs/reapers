//! The rules against the record of what they did before they were Rust.
//!
//! `fixtures/generate.dart` asked the first implementation of the rules a
//! few thousand questions with definite answers and wrote the answers down
//! (`fixtures/README.md` says how it ran, and why it no longer can). Every
//! test here asks the rules the same questions and expects the same
//! answers, to the last bit of every double: a save, a hunt log or a receipt
//! written before the port must read and replay the same way.
//!
//! One kind of answer is held a little looser by default. The first rules
//! took their sines and arc tangents from the platform; these take theirs
//! from the `libm` crate, so that the server and the Wasm game agree with
//! each other to the bit. The two libraries part in the last bit now and
//! then, so a husk's position, or the bearing between two points a few
//! metres apart, may differ from the record by that much ([`DRIFT`]).
//! Everything that decides a hunt (every input the bot makes, every event,
//! every strike's damage, every outcome) must still match exactly, and
//! does. Built with `--features system-math`, which takes those functions
//! from the platform as the first rules did, every answer here must match
//! to the bit.

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

/// `value` with every saved `grade` written as the capture grade it now
/// names. The record kept how cleanly a husk was reaped; each old grade
/// became the capture grade that pays the same.
fn regraded(value: &Value) -> Value {
	match value {
		Value::Object(fields) => {
			Value::Object(
				fields
					.iter()
					.map(|(name, value)| {
						let value = if name == "grade" {
							value
								.as_str()
								.and_then(CaptureGrade::from_saved)
								.map_or_else(|| value.clone(), |grade| Value::from(grade.name()))
						} else {
							regraded(value)
						};

						(name.clone(), value)
					})
					.collect(),
			)
		}
		Value::Array(items) => Value::Array(items.iter().map(regraded).collect()),
		other => other.clone(),
	}
}

/// How far a double that went through a sine or an arc tangent may stray
/// from the record, relative to its size, while the rules take those
/// functions from `libm` rather than from the platform.
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

fn difficulty(json: &Value) -> Difficulty {
	Difficulty::from_name(text(json)).unwrap()
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

/// The stats the combat-rules change (the crosshair, the arsenal and the
/// orb) derives for every case the record holds, pinned to the answers the
/// Dart rules gave after that change. The record's own fields were named
/// for the fight before it, so its old numbers no longer describe these.
#[test]
fn derived_stats_match_the_combat_rules_as_dart_wrote_them() {
	let fixture = fixture("stats.json");
	let mut lines = Vec::new();

	for case in list(&fixture["cases"]) {
		let stats = HuntStats::derive(&loadout(&case["loadout"]), float(&case["leniency"]))
			.unwrap()
			.at(difficulty(&case["difficulty"]));

		lines.push(
			[
				bits(stats.hit),
				bits(stats.heavy_strike),
				bits(stats.reach_radians),
				bits(stats.assist_radians),
				bits(stats.blade_reach_radians),
				bits(stats.heavy_reach_radians),
				stats.swing_ticks.to_string(),
				stats.resolve.to_string(),
				stats.broken_ticks.to_string(),
				bits(stats.orb_aim_radians),
				stats.capture.to_string(),
				bits(stats.flinch_speed),
				bits(stats.parry_speed),
				stats.perk.name().to_owned(),
				stats
					.gift
					.map_or_else(|| "null".to_owned(), |gift| gift.name().to_owned()),
				stats.soul_rank.to_string(),
				stats.knock_ticks.to_string(),
				bits(stats.body_share),
			]
			.join(" "),
		);
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

		lines.push(format!("riftWard {}", bits(RiftRules::ward_for(&circle))));
	}

	let stats = lines.join("\n");

	assert_eq!(digest(stats.as_bytes()), "c7f20440927cefbc", "{stats}");
}

// Hunts ----------------------------------------------------------------

// `hunts.json` holds 69 hunts played by the fight before the crosshair, the
// arsenal and the orb. Their bots tapped at seals, reaped with a sweep and
// carried a `reapSpeed` the hunt bot no longer has, so their inputs cannot
// be played again and their outcomes are not the ones these rules reach.
// The record stays untouched; `tests/hunt_sim.rs` and `tests/orb.rs` pin
// the new fight, its log and its replay instead.

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

/// Every whole log the record holds was written by the fight before the
/// crosshair, at version 6. The crosshair's log is version 7, with FIRE, the
/// selected slot and ORB in place of the taps, so the rules refuse the old
/// log rather than read a tap as a FIRE press. A log too short to carry a
/// version still fails as it always did.
#[test]
fn hunt_logs_from_before_the_crosshair_are_refused() {
	let fixture = fixture("hunt_logs.json");

	for case in list(&fixture["logs"]) {
		let bytes = base64(text(&case["bytes"]));

		assert_eq!(
			HuntLog::decode(&bytes).err(),
			Some(HuntLogError::UnsupportedVersion),
			"{}",
			case["name"]
		);
	}

	for case in list(&fixture["malformed"]) {
		let bytes = base64(text(&case["bytes"]));
		let expected = match case["error"].as_str() {
			// A whole version-six log is refused at its version byte, before
			// the rest of it is read.
			None
			| Some(
				"Unsupported combat rules"
				| "Hunt log has no such difficulty"
				| "Hunt log has a broken plan"
				| "Hunt log has broken cover"
				| "Hunt log ends mid-plan"
				| "Hunt log ends mid-tap"
				| "Hunt log has unknown input",
			) => HuntLogError::UnsupportedVersion,
			Some("Not a hunt log") => HuntLogError::Empty,
			Some("Hunt log has no plan") => HuntLogError::NoPlan,
			Some(other) => panic!("the record expected something else: {other}"),
		};

		assert_eq!(
			HuntLog::decode(&bytes).err(),
			Some(expected),
			"{}",
			case["name"]
		);
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

// `rifts.json`'s played rifts were fought by the bot of the fight before the
// crosshair, against a ward sized by strikes at seals. Both changed on
// purpose, so their frames and pay no longer describe these rules. What is
// left of the record is the circle itself, which the arena still fills the
// same way.
#[test]
fn the_shades_fill_a_rift_as_they_did_in_dart() {
	let fixture = fixture("rifts.json");
	let mut room = RiftRoom::new(RiftSite::ShibuyaCrossing, 0x51DE);
	let seat = room
		.join(entrant("Vesper", ReaperClass::Harvester, 13))
		.unwrap();

	room.report(seat, &RiftReport::readiness(true));
	room.advance(RiftRules::COUNTDOWN_TICKS + 1);

	let mut expected = fixture["shades"].clone();
	let mut actual = serde_json::to_value(room.frame_for(seat).unwrap()).unwrap();

	// The ward is now sized to a careful gun's pace rather than to strikes
	// at seals, so its two fields are the change and are checked by the
	// rift pacing table instead.
	for frame in [&mut expected, &mut actual] {
		let fields = frame.as_object_mut().unwrap();

		fields.remove("ward");
		fields.remove("wardMax");
	}

	assert_eq!(actual, expected);
}

// The player -----------------------------------------------------------

/// Plays the fixture's scripted session, step by step, and checks each
/// state the rules return against the one the record holds.
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
						regraded(&expected["state"]),
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
		let recorded = settlements.next().unwrap();
		let outcome: HuntOutcome = serde_json::from_value(regraded(&recorded["outcome"])).unwrap();
		let next = state
			.settle_hunt(
				sighting.id,
				&outcome,
				now.plus_seconds(60),
				sighting.seed * 3 + 1,
			)
			.unwrap();
		let expected = steps.next().unwrap();

		assert_eq!(
			text(&expected["step"]),
			format!("judge {}", sighting.nft.id())
		);
		assert_eq!(
			serde_json::to_value(&next).unwrap(),
			regraded(&expected["state"])
		);
		state = next;
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
				CaptureGrade::Excellent,
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
		grade: CaptureGrade::Excellent,
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

// The fixtures hold the tables the fight before the crosshair printed. Those
// numbers were tuned around taps and the reap sweep, so they say nothing
// about the fight these rules play. Each report is pinned to what the Dart
// rules printed after the change instead; the table itself is in
// docs/gameplay.md.

#[test]
fn the_pacing_tool_prints_the_crosshair_fight() {
	let report = pacing::report();

	assert_eq!(digest(report.as_bytes()), "cee7bac19ef7790f", "{report}");
}

#[test]
fn the_rift_pacing_tool_prints_the_crosshair_fight() {
	let report = rift_pacing::report();

	assert_eq!(digest(report.as_bytes()), "2f6c9991863ca3bd", "{report}");
}

#[test]
fn the_soul_pacing_tool_prints_the_crosshair_fight() {
	let report = soul_pacing::report();

	assert_eq!(digest(report.as_bytes()), "9206be3939838659", "{report}");
}
