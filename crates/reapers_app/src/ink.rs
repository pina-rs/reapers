//! Second Ink: the paper and the inks every printed mark is made of, as the
//! art bible's palette (§3) names them, and the arithmetic that prints with
//! them.
//!
//! A mark is key on paper, and the second ink, the spot, is the reaper's own
//! light: one per class, whatever its colourway ([`light`]). Wound is a
//! third ink kept for loss and error alone.
//!
//! Every other colour is an overprint ([`overprint`]): key at one coverage
//! and the spot at another, on paper. A surface [`Family`] prints three
//! bands that way, lit, shade and deep: on a reaper in its light
//! ([`ReaperInk::print`]), on a husk in key alone ([`husk`]), and a husk's
//! own colours as a ghost of themselves ([`ghost`]). The numbers are
//! cel.toml's and the arithmetic is cel.py's, so the Blender review sheets
//! and the game print the same colours: `tests/fixtures/second_ink.txt` holds
//! the sheets' values, and the tests here hold the game to them.

use reapers_rules::Omen;
use reapers_rules::Outfit;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;

use crate::theme::Rgb;

/// The stock: slips, cards, the knockout, specular highlights.
pub const PAPER: Rgb = Rgb::from_hex(0x00EC_E3CF);
/// Every line, and the text on paper and on spot.
pub const KEY: Rgb = Rgb::from_hex(0x0016_121A);
/// The third ink: loss and error only.
pub const WOUND: Rgb = Rgb::from_hex(0x00B9_2A20);

/// The harvester's light.
pub const CHAFF: Rgb = Rgb::from_hex(0x00F4_CD50);
/// The ferryman's light.
pub const LANTERN: Rgb = Rgb::from_hex(0x00F0_7428);
/// The weigher's light.
pub const VERDIGRIS: Rgb = Rgb::from_hex(0x002F_BD9F);
/// The chooser's light.
pub const COBALT: Rgb = Rgb::from_hex(0x0047_7FE4);
/// The hollow's light.
pub const WISP: Rgb = Rgb::from_hex(0x00A8_F2D8);
/// The mourner's light.
pub const MAUVE: Rgb = Rgb::from_hex(0x00CD_B2F6);

/// The spot: the one light `reaper_class` prints its marks in.
pub const fn light(reaper_class: ReaperClass) -> Rgb {
	match reaper_class {
		ReaperClass::Harvester => CHAFF,
		ReaperClass::Ferryman => LANTERN,
		ReaperClass::Weigher => VERDIGRIS,
		ReaperClass::Chooser => COBALT,
		ReaperClass::Hollow => WISP,
		ReaperClass::Mourner => MAUVE,
	}
}

/// `colour` in linear light, as the renderer takes colours.
pub fn linear(colour: Rgb) -> [f32; 3] {
	tone(colour).map(|channel| to_linear(channel) as f32)
}

// Overprinting ---------------------------------------------------------------

/// How much of each ink is printed over the paper, from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coverage {
	pub key: f64,
	pub spot: f64,
}

/// Key at `key` and the spot at `spot`.
pub const fn cover(key: f64, spot: f64) -> Coverage {
	Coverage { key, spot }
}

/// sRGB channels from 0 to 1, where ink films multiply.
type Tone = [f64; 3];

fn tone(colour: Rgb) -> Tone {
	[colour.0, colour.1, colour.2].map(|channel| f64::from(channel) / 255.0)
}

fn rgb(tone: Tone) -> Rgb {
	let [red, green, blue] = tone.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8);
	Rgb(red, green, blue)
}

fn printed(base: Tone, coverage: Coverage, spot: Tone) -> Tone {
	let (paper, key) = (tone(PAPER), tone(KEY));
	std::array::from_fn(|channel| {
		let keyed = 1.0 - coverage.key + coverage.key * key[channel] / paper[channel];
		let spotted = 1.0 - coverage.spot + coverage.spot * spot[channel] / paper[channel];
		(base[channel] * keyed * spotted).clamp(0.0, 1.0)
	})
}

/// `base` printed over with key and `spot` at `coverage` (§3.1): each ink
/// filters the light by its share of what it does to paper, channel by
/// channel in sRGB. On paper, full key is [`KEY`] and a full spot is the
/// spot itself.
pub fn overprint(base: Rgb, coverage: Coverage, spot: Rgb) -> Rgb {
	rgb(printed(tone(base), coverage, tone(spot)))
}

// CIELAB, under D65 ----------------------------------------------------------

const WHITE: [f64; 3] = [0.950_47, 1.0, 1.088_83];
const EPSILON: f64 = 216.0 / 24389.0;
const KAPPA: f64 = 24389.0 / 27.0;

fn to_linear(channel: f64) -> f64 {
	if channel <= 0.04045 {
		channel / 12.92
	} else {
		((channel + 0.055) / 1.055).powf(2.4)
	}
}

fn to_srgb(channel: f64) -> f64 {
	let channel = channel.clamp(0.0, 1.0);
	if channel <= 0.003_130_8 {
		12.92 * channel
	} else {
		1.055 * channel.powf(1.0 / 2.4) - 0.055
	}
}

/// `tone` as CIELAB: lightness L*, then a* and b*.
fn lab(tone: Tone) -> [f64; 3] {
	let [red, green, blue] = tone.map(to_linear);
	let x = (0.4124 * red + 0.3576 * green + 0.1805 * blue) / WHITE[0];
	let y = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
	let z = (0.0193 * red + 0.1192 * green + 0.9505 * blue) / WHITE[2];
	let compand = |value: f64| {
		if value > EPSILON {
			value.cbrt()
		} else {
			(KAPPA * value + 16.0) / 116.0
		}
	};
	let (fx, fy, fz) = (compand(x), compand(y), compand(z));
	[116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// The sRGB tone of a CIELAB colour, clipped to what sRGB can show.
fn from_lab([lightness, a_star, b_star]: [f64; 3]) -> Tone {
	let fy = (lightness + 16.0) / 116.0;
	let (fx, fz) = (fy + a_star / 500.0, fy - b_star / 200.0);
	let inverse = |value: f64| {
		let cubed = value * value * value;
		if cubed > EPSILON {
			cubed
		} else {
			(116.0 * value - 16.0) / KAPPA
		}
	};
	let (x, y, z) = (inverse(fx) * WHITE[0], inverse(fy), inverse(fz) * WHITE[2]);
	[
		3.2406 * x - 1.5372 * y - 0.4986 * z,
		-0.9689 * x + 1.8758 * y + 0.0415 * z,
		0.0557 * x - 0.2040 * y + 1.0570 * z,
	]
	.map(to_srgb)
}

/// A colour's CIELAB hue, in degrees from 0 to 360.
pub fn hue(colour: Rgb) -> f64 {
	let [_, a, b] = lab(tone(colour));
	b.atan2(a).to_degrees().rem_euclid(360.0)
}

/// A colour's CIELAB lightness, L*, from 0 to 100.
pub fn lightness(colour: Rgb) -> f64 {
	lab(tone(colour))[0]
}

// Families -------------------------------------------------------------------

/// What a surface is made of, as a material is named in the models (§3.3):
/// each prints its own bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
	/// The robe.
	Cloth,
	/// The mantle and the trim; the robe in the Ember colourway.
	ClothAlt,
	Feather,
	Bone,
	Metal,
	/// Spot-printed parts: cages, pans and the band where the hand meets
	/// the weapon.
	Gold,
	Wood,
	Rope,
	/// Tags, seals and scrolls.
	Paper,
	/// What shows of the reaper through its robe.
	Spirit,
	/// Emptiness: flat key.
	Void,
	Dark,
	/// The figure's one light.
	Accent,
	Glow,
}

impl Family {
	pub const ALL: [Self; 14] = [
		Self::Cloth,
		Self::ClothAlt,
		Self::Feather,
		Self::Bone,
		Self::Metal,
		Self::Gold,
		Self::Wood,
		Self::Rope,
		Self::Paper,
		Self::Spirit,
		Self::Void,
		Self::Dark,
		Self::Accent,
		Self::Glow,
	];

	/// The family's name, as a material that prints in it is named.
	pub const fn name(self) -> &'static str {
		match self {
			Self::Cloth => "cloth",
			Self::ClothAlt => "cloth_alt",
			Self::Feather => "feather",
			Self::Bone => "bone",
			Self::Metal => "metal",
			Self::Gold => "gold",
			Self::Wood => "wood",
			Self::Rope => "rope",
			Self::Paper => "paper",
			Self::Spirit => "spirit",
			Self::Void => "void",
			Self::Dark => "dark",
			Self::Accent => "accent",
			Self::Glow => "glow",
		}
	}

	/// The family called exactly `name`.
	pub fn named(name: &str) -> Option<Self> {
		Self::ALL.into_iter().find(|family| family.name() == name)
	}

	/// cel.toml's `[families]`.
	const fn recipe(self) -> Recipe {
		match self {
			Self::Cloth => {
				banded([cover(0.46, 0.84), cover(0.80, 0.50), cover(0.93, 0.16)]).hatched()
			}
			Self::ClothAlt => {
				banded([cover(0.32, 0.92), cover(0.70, 0.60), cover(0.90, 0.20)]).hatched()
			}
			Self::Feather => {
				banded([cover(0.56, 0.74), cover(0.84, 0.36), cover(0.95, 0.10)]).hatched()
			}
			Self::Bone => {
				banded([cover(0.0, 0.0), cover(0.24, 0.10), cover(0.55, 0.08)]).specular(0.95)
			}
			Self::Metal => {
				banded([cover(0.34, 0.0), cover(0.64, 0.06), cover(0.86, 0.05)])
					.hatched()
					.specular(0.90)
			}
			Self::Gold => {
				Recipe {
					line: 0.75,
					..banded([cover(0.0, 1.0), cover(0.34, 1.0), cover(0.66, 0.90)]).specular(0.92)
				}
			}
			Self::Wood => {
				banded([cover(0.46, 0.55), cover(0.72, 0.40), cover(0.90, 0.18)]).hatched()
			}
			Self::Rope => banded([cover(0.08, 0.42), cover(0.38, 0.46), cover(0.74, 0.26)]),
			Self::Paper => banded([cover(0.04, 0.08), cover(0.24, 0.12), cover(0.50, 0.10)]),
			Self::Spirit => flat(cover(0.94, 0.10), 1.0),
			Self::Void | Self::Dark => flat(cover(1.0, 0.0), 0.0),
			Self::Accent | Self::Glow => {
				Recipe {
					light: true,
					..flat(cover(0.0, 1.0), 0.0)
				}
			}
		}
	}
}

/// How a family prints before any light is chosen.
#[derive(Clone, Copy, Debug)]
struct Recipe {
	/// Lit, shade and deep; the same three times for a flat family.
	bands: [Coverage; 3],
	flat: bool,
	/// The figure's light: the spot on a reaper, and on a husk, which has
	/// none, paper or key.
	light: bool,
	hatch: bool,
	specular: f64,
	line: f64,
}

const fn banded(bands: [Coverage; 3]) -> Recipe {
	Recipe {
		bands,
		flat: false,
		light: false,
		hatch: false,
		specular: 0.0,
		line: 1.0,
	}
}

const fn flat(coverage: Coverage, line: f64) -> Recipe {
	Recipe {
		bands: [coverage; 3],
		flat: true,
		light: false,
		hatch: false,
		specular: 0.0,
		line,
	}
}

impl Recipe {
	const fn hatched(self) -> Self {
		Self {
			hatch: true,
			..self
		}
	}

	const fn specular(self, specular: f64) -> Self {
		Self { specular, ..self }
	}
}

/// How a surface prints: its three bands and what else marks it (§5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Printed {
	pub lit: Rgb,
	pub shade: Rgb,
	pub deep: Rgb,
	/// One colour, with no bands: lit, shade and deep are the same.
	pub flat: bool,
	/// The least duty of the key hatch in the deep band; 0 draws none.
	pub hatch: f64,
	/// A second hatch below the scene's crosshatch threshold.
	pub cross: bool,
	/// The N·H a paper highlight needs, on turning edges; 0 for none.
	pub specular: f64,
	/// A rim of the spot on the side away from the light.
	pub rim: bool,
	/// The key line's weight on the surface; 0 draws none.
	pub line: f64,
}

/// The least hatch duty on a reaper, and on a husk (§5.2).
pub const REAPER_HATCH: f64 = 0.24;
pub const HUSK_HATCH: f64 = 0.30;

// Reapers --------------------------------------------------------------------

/// The colourway: how much light is in the robe (§3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Robe {
	/// The robe prints as `cloth`.
	Night,
	/// The robe prints with `cloth_alt`'s coverages.
	Ember,
	/// The robe is key only; the light stays on the trim, rim and eyes.
	Ash,
}

impl Robe {
	/// The robe of a genome's `palette`, the shuffle's colourway.
	pub const fn of(palette: u32) -> Self {
		match palette % 3 {
			0 => Self::Night,
			1 => Self::Ember,
			_ => Self::Ash,
		}
	}
}

/// The Ash robe's coverages.
const ASH: [Coverage; 3] = [cover(0.52, 0.10), cover(0.82, 0.06), cover(0.94, 0.0)];

/// Over the camera nothing on a figure but its line prints darker than L*
/// 16 (§4.6): the arm's deep band takes key at no more than this.
const FIRST_PERSON_DEEP: f64 = 0.85;

/// Undyed: a shrouded reaper's robe carries this share of its spot.
const SHROUDED_SPOT: f64 = 0.55;

/// How one reaper prints: its light, and the axes of its look that change
/// the print (§3.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReaperInk {
	pub light: Rgb,
	pub robe: Robe,
	pub omen: Omen,
	pub outfit: Outfit,
	/// Seen from inside, over the camera: the first-person arm.
	pub first_person: bool,
}

impl ReaperInk {
	/// `genome` wearing `outfit`, seen from outside.
	pub const fn of(genome: &ReaperGenome, outfit: Outfit) -> Self {
		Self {
			light: light(genome.reaper_class),
			robe: Robe::of(genome.palette),
			omen: genome.omen,
			outfit,
			first_person: false,
		}
	}

	/// Ashen halves the spot on every family; the interface keeps it whole.
	fn spot_share(self) -> f64 {
		if matches!(self.omen, Omen::Ashen) {
			0.5
		} else {
			1.0
		}
	}

	/// The spot as this reaper's rim and lights print it.
	pub fn spot(self) -> Rgb {
		overprint(PAPER, cover(0.0, self.spot_share()), self.light)
	}

	/// What `family` prints as on this reaper.
	pub fn print(self, family: Family) -> Printed {
		// Gilded: every buckle and blade is gilt in the class light.
		let family = match (family, self.omen) {
			(Family::Metal, Omen::Gilded) => Family::Gold,
			_ => family,
		};
		let recipe = family.recipe();
		let mut bands = recipe.bands;
		if family == Family::Cloth {
			bands = match (self.robe, self.omen) {
				// Eclipsed: the spot leaves the body.
				(_, Omen::Eclipsed) | (Robe::Ash, _) => ASH,
				(Robe::Ember, _) => Family::ClothAlt.recipe().bands,
				(Robe::Night, _) => bands,
			};
			if self.outfit == Outfit::Shrouded {
				bands = bands.map(|band| cover(band.key, band.spot * SHROUDED_SPOT));
			}
		}
		if self.first_person && !recipe.flat {
			bands[2].key = bands[2].key.min(FIRST_PERSON_DEEP);
		}
		let share = self.spot_share();
		let [lit, shade, deep] =
			bands.map(|band| overprint(PAPER, cover(band.key, band.spot * share), self.light));
		Printed {
			lit,
			shade,
			deep,
			flat: recipe.flat,
			hatch: if recipe.hatch { REAPER_HATCH } else { 0.0 },
			cross: !recipe.flat,
			specular: recipe.specular,
			rim: !recipe.flat,
			line: recipe.line,
		}
	}
}

// Husks ----------------------------------------------------------------------

/// What `family` prints as on a husk, which has no light: key alone, its
/// eyes (`accent`) paper and its seals (`glow`) key (§3.6).
pub fn husk(family: Family) -> Printed {
	let recipe = family.recipe();
	let [lit, shade, deep] = if recipe.light {
		[if family == Family::Accent { PAPER } else { KEY }; 3]
	} else {
		recipe
			.bands
			.map(|band| overprint(PAPER, cover(band.key, 0.0), PAPER))
	};
	Printed {
		lit,
		shade,
		deep,
		flat: recipe.flat,
		hatch: if recipe.hatch { HUSK_HATCH } else { 0.0 },
		cross: false,
		specular: recipe.specular,
		rim: false,
		line: recipe.line,
	}
}

/// The ghost wash's numbers (§3.6).
const GHOST_CHROMA: f64 = 16.0;
const GHOST_LIGHTNESS: [f64; 2] = [34.0, 90.0];
const CLASH_DEGREES: f64 = 20.0;
const CLASH_CHROMA: f64 = 8.0;
const GHOST_SHADE: f64 = 0.36;
const GHOST_DEEP: f64 = 0.60;
const DEEP_FLOOR: f64 = 16.0;
const SPECULAR_LIGHTNESS: f64 = 70.0;
const GHOST_SPECULAR: f64 = 0.96;

/// One of a husk's own colours (its NFT's body, trim or detail) as the dead
/// print it, seen by a reaper whose light is `player` (§3.6).
///
/// The lit band keeps the colour's hue with its chroma capped at C* 16 and
/// its lightness clamped to L* 34–90; within 20° of the player's light the
/// cap is C* 8, for the dead never wear your light. Shade and deep are key
/// printed over the lit band, and deep never falls under L* 16.
pub fn ghost(colour: Rgb, player: Option<Rgb>) -> Printed {
	let [lightness, mut a, mut b] = lab(tone(colour));
	let lightness = lightness.clamp(GHOST_LIGHTNESS[0], GHOST_LIGHTNESS[1]);
	let chroma = a.hypot(b);
	let clashes = player.is_some_and(|player| {
		let distance = (b.atan2(a).to_degrees() - hue(player) + 180.0).rem_euclid(360.0) - 180.0;
		distance.abs() < CLASH_DEGREES
	});
	let cap = if clashes { CLASH_CHROMA } else { GHOST_CHROMA };
	if chroma > cap {
		a *= cap / chroma;
		b *= cap / chroma;
	}
	let lit = from_lab([lightness, a, b]);
	let paper = tone(PAPER);
	let shade = printed(lit, cover(GHOST_SHADE, 0.0), paper);
	let mut deep = printed(lit, cover(GHOST_DEEP, 0.0), paper);
	if lab(deep)[0] < DEEP_FLOOR {
		deep = from_lab([DEEP_FLOOR, 0.0, 0.0]);
	}
	Printed {
		lit: rgb(lit),
		shade: rgb(shade),
		deep: rgb(deep),
		flat: false,
		hatch: HUSK_HATCH,
		cross: false,
		specular: if lightness > SPECULAR_LIGHTNESS {
			GHOST_SPECULAR
		} else {
			0.0
		},
		rim: false,
		line: 1.0,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// What the Blender reference printed, cel.py through the review sheets.
	const REFERENCE: &str = include_str!("../tests/fixtures/second_ink.txt");

	/// The lines of the reference's `section`, split into words.
	fn section(name: &str) -> Vec<Vec<&'static str>> {
		let heading = format!("# {name}");
		let mut lines = REFERENCE.lines().skip_while(|line| *line != heading);
		assert!(lines.next().is_some(), "the reference has no {name}");
		let rows: Vec<Vec<&str>> = lines
			.take_while(|line| !line.starts_with('#'))
			.map(|line| line.split_whitespace().collect())
			.collect();
		assert!(!rows.is_empty(), "the reference's {name} is empty");
		rows
	}

	fn hex(text: &str) -> Rgb {
		let digits = text.strip_prefix('#').expect("a #rrggbb colour");
		Rgb::from_hex(u32::from_str_radix(digits, 16).expect("hex digits"))
	}

	fn number(text: &str) -> f64 {
		text.parse().expect("a number")
	}

	/// Within one 8-bit step on every channel.
	#[track_caller]
	fn assert_near(game: Rgb, reference: &str, what: &str) {
		let reference = hex(reference);
		let steps = [
			(game.0, reference.0),
			(game.1, reference.1),
			(game.2, reference.2),
		]
		.map(|(a, b)| a.abs_diff(b));
		assert!(
			steps.iter().all(|step| *step <= 1),
			"{what}: the game prints {}, the reference {}",
			game.hex(),
			reference.hex()
		);
	}

	fn class(name: &str) -> ReaperClass {
		*ReaperClass::ALL
			.iter()
			.find(|class| format!("{class:?}").eq_ignore_ascii_case(name))
			.unwrap_or_else(|| panic!("no class {name}"))
	}

	fn light_named(name: &str) -> Rgb {
		match name {
			"chaff" => CHAFF,
			"lantern" => LANTERN,
			"verdigris" => VERDIGRIS,
			"cobalt" => COBALT,
			"wisp" => WISP,
			"mauve" => MAUVE,
			_ => panic!("no light {name}"),
		}
	}

	fn reaper(reaper_class: ReaperClass) -> ReaperInk {
		ReaperInk {
			light: light(reaper_class),
			robe: Robe::Night,
			omen: Omen::None,
			outfit: Outfit::Sworn,
			first_person: false,
		}
	}

	#[test]
	fn the_inks_are_the_bibles() {
		assert_eq!(PAPER.hex(), "#ece3cf");
		assert_eq!(KEY.hex(), "#16121a");
		assert_eq!(WOUND.hex(), "#b92a20");
		let lights = [
			(ReaperClass::Harvester, "#f4cd50"),
			(ReaperClass::Ferryman, "#f07428"),
			(ReaperClass::Weigher, "#2fbd9f"),
			(ReaperClass::Chooser, "#477fe4"),
			(ReaperClass::Hollow, "#a8f2d8"),
			(ReaperClass::Mourner, "#cdb2f6"),
		];
		for (reaper_class, hex) in lights {
			assert_eq!(light(reaper_class).hex(), hex, "{reaper_class:?}");
		}
	}

	#[test]
	fn every_class_has_its_own_light_and_none_is_wound() {
		let lights: Vec<Rgb> = ReaperClass::ALL.into_iter().map(light).collect();
		for (index, one) in lights.iter().enumerate() {
			assert_ne!(*one, WOUND);
			assert!(!lights[index + 1..].contains(one), "{one:?} twice");
		}
	}

	#[test]
	fn overprinting_is_the_references() {
		for row in section("overprint") {
			let coverage = cover(number(row[0]), number(row[1]));
			let printed = overprint(PAPER, coverage, light_named(row[2]));
			assert_near(printed, row[3], &row[..3].join(" "));
		}
		assert_eq!(overprint(PAPER, cover(0.0, 0.0), LANTERN), PAPER);
		assert_eq!(overprint(PAPER, cover(1.0, 0.0), LANTERN), KEY);
		assert_eq!(overprint(PAPER, cover(0.0, 1.0), LANTERN), LANTERN);
	}

	#[test]
	fn the_lights_hues_are_the_references() {
		for row in section("light hues") {
			let hue = hue(light_named(row[0]));
			assert!((hue - number(row[1])).abs() < 0.01, "{}: {hue}", row[0]);
		}
	}

	#[test]
	fn every_family_prints_the_references_bands_in_every_light() {
		let rows = section("reaper families, roster scene, night robe");
		assert_eq!(rows.len(), ReaperClass::ALL.len() * Family::ALL.len());
		for row in rows {
			let family = Family::named(row[1]).expect("a family");
			let printed = reaper(class(row[0])).print(family);
			let what = format!("{} {}", row[0], row[1]);
			if row[2] == "flat" {
				assert!(printed.flat, "{what} is flat");
				assert_near(printed.lit, row[3], &what);
				assert_eq!((printed.lit, printed.lit), (printed.shade, printed.deep));
				continue;
			}
			assert!(!printed.flat, "{what} is banded");
			assert_near(printed.lit, row[2], &format!("{what} lit"));
			assert_near(printed.shade, row[3], &format!("{what} shade"));
			assert_near(printed.deep, row[4], &format!("{what} deep"));
			let hatch = if number(row[5]) > 0.0 {
				number(row[9])
			} else {
				0.0
			};
			assert_eq!(printed.hatch, hatch, "{what} hatch");
			assert_eq!(printed.specular, number(row[6]), "{what} specular");
			assert_eq!(printed.rim, number(row[7]) > 0.0, "{what} rim");
			assert_eq!(printed.cross, number(row[8]) > 0.0, "{what} cross");
		}
	}

	#[test]
	fn every_robe_prints_the_references_cloth() {
		for row in section("robe colourways") {
			let robe = match row[0] {
				"ember" => Robe::Ember,
				"ash" => Robe::Ash,
				other => panic!("no robe {other}"),
			};
			let ink = ReaperInk {
				robe,
				..reaper(class(row[1]))
			};
			let printed = ink.print(Family::Cloth);
			let what = format!("{} {}", row[0], row[1]);
			assert_near(printed.lit, row[2], &format!("{what} lit"));
			assert_near(printed.shade, row[3], &format!("{what} shade"));
			assert_near(printed.deep, row[4], &format!("{what} deep"));
		}
		assert_eq!(Robe::of(0), Robe::Night);
		assert_eq!(Robe::of(1), Robe::Ember);
		assert_eq!(Robe::of(2), Robe::Ash);
	}

	#[test]
	fn the_arm_prints_the_references_cloth_and_nothing_under_l_16() {
		for row in section("first person cloth") {
			let ink = ReaperInk {
				first_person: true,
				..reaper(class(row[0]))
			};
			let printed = ink.print(Family::Cloth);
			assert_near(printed.lit, row[1], &format!("{} lit", row[0]));
			assert_near(printed.shade, row[2], &format!("{} shade", row[0]));
			assert_near(printed.deep, row[3], &format!("{} deep", row[0]));
		}
		// §4.6: over the camera only the line is darker than L* 16, in every
		// robe and every light.
		for reaper_class in ReaperClass::ALL {
			for robe in [Robe::Night, Robe::Ember, Robe::Ash] {
				let ink = ReaperInk {
					robe,
					first_person: true,
					..reaper(reaper_class)
				};
				for family in Family::ALL {
					let printed = ink.print(family);
					if printed.flat {
						continue;
					}
					let deep = lightness(printed.deep);
					assert!(
						deep >= 16.0,
						"{reaper_class:?} {robe:?} {family:?}: L* {deep}"
					);
				}
			}
		}
	}

	#[test]
	fn omens_change_the_spot_and_nothing_else() {
		let plain = reaper(ReaperClass::Ferryman);
		// Ashen halves the spot everywhere: the bible's Ferryman lit robe.
		let ashen = ReaperInk {
			omen: Omen::Ashen,
			..plain
		};
		assert_near(ashen.print(Family::Cloth).lit, "#8B6852", "ashen robe");
		assert_near(
			ashen.print(Family::Accent).lit,
			&overprint(PAPER, cover(0.0, 0.5), LANTERN).hex(),
			"ashen light",
		);
		// What prints in key alone is untouched.
		assert_eq!(ashen.print(Family::Void), plain.print(Family::Void));
		assert_eq!(ashen.print(Family::Bone).lit, plain.print(Family::Bone).lit);
		assert_ne!(ashen.spot(), plain.spot());
		assert_eq!(plain.spot(), LANTERN);
		// Gilded: metal is gilt.
		let gilded = ReaperInk {
			omen: Omen::Gilded,
			..plain
		};
		assert_eq!(gilded.print(Family::Metal), plain.print(Family::Gold));
		// Eclipsed: the robe at Ash coverages, the light kept on the rest.
		let eclipsed = ReaperInk {
			omen: Omen::Eclipsed,
			..plain
		};
		let ash = ReaperInk {
			robe: Robe::Ash,
			..plain
		};
		assert_eq!(eclipsed.print(Family::Cloth), ash.print(Family::Cloth));
		assert_eq!(eclipsed.print(Family::Gold), plain.print(Family::Gold));
		assert_eq!(eclipsed.print(Family::Accent), plain.print(Family::Accent));
		assert_eq!(eclipsed.spot(), LANTERN);
	}

	#[test]
	fn a_shrouded_robe_is_undyed() {
		let sworn = reaper(ReaperClass::Harvester);
		let shrouded = ReaperInk {
			outfit: Outfit::Shrouded,
			..sworn
		};
		let undyed = shrouded.print(Family::Cloth);
		assert_eq!(
			undyed.lit,
			overprint(PAPER, cover(0.46, 0.84 * 0.55), CHAFF)
		);
		assert_ne!(undyed, sworn.print(Family::Cloth));
		assert_eq!(
			shrouded.print(Family::ClothAlt),
			sworn.print(Family::ClothAlt)
		);
	}

	#[test]
	fn a_husk_prints_without_light() {
		for row in section("husk families (hunt)") {
			let family = Family::named(row[1]).expect("a family");
			let printed = husk(family);
			let what = row[1];
			if row[2] == "flat" {
				assert!(printed.flat, "{what} is flat");
				assert_near(printed.lit, row[3], what);
				continue;
			}
			assert_near(printed.lit, row[2], &format!("{what} lit"));
			assert_near(printed.shade, row[3], &format!("{what} shade"));
			assert_near(printed.deep, row[4], &format!("{what} deep"));
			let hatch = if number(row[5]) > 0.0 {
				number(row[9])
			} else {
				0.0
			};
			assert_eq!(printed.hatch, hatch, "{what} hatch");
			assert_eq!(printed.specular, number(row[6]), "{what} specular");
			assert!(!printed.rim && !printed.cross, "{what} has no rim or cross");
		}
		assert_eq!(husk(Family::Accent).lit, PAPER);
		assert_eq!(husk(Family::Glow).lit, KEY);
		// The spot never prints on the dead: gilt is key over paper.
		assert_eq!(husk(Family::Gold).lit, PAPER);
	}

	#[test]
	fn the_ghost_wash_is_the_references() {
		let rows = section("ghost wash");
		assert!(rows.len() >= 60);
		for row in rows {
			let player = (row[0] != "-").then(|| light(class(row[0])));
			let printed = ghost(hex(row[1]), player);
			let what = format!("{} {}", row[0], row[1]);
			assert_near(printed.lit, row[2], &format!("{what} lit"));
			assert_near(printed.shade, row[3], &format!("{what} shade"));
			assert_near(printed.deep, row[4], &format!("{what} deep"));
			assert_eq!(printed.specular, number(row[5]), "{what} specular");
			assert_eq!(printed.hatch, HUSK_HATCH);
			assert!(!printed.rim && !printed.cross && !printed.flat);
		}
	}

	#[test]
	fn the_bibles_husks_wash_as_it_says() {
		// §3.6's table, and its notes.
		let table = [
			("#C9722E", "#9F8370", "#6B574D", "#483A35"),
			("#2E6FD9", "#6C718C", "#494C60", "#313343"),
			("#3B3F46", "#4C5058", "#33363C", "#282828"),
			("#7A5F8A", "#73647C", "#4E4355", "#352D3B"),
			("#F2EEE6", "#E6E2DA", "#9B9795", "#696568"),
		];
		for (body, lit, shade, deep) in table {
			let printed = ghost(hex(body), None);
			assert_near(printed.lit, lit, body);
			assert_near(printed.shade, shade, body);
			assert_near(printed.deep, deep, body);
		}
		let penguin = ghost(hex("#C9722E"), Some(LANTERN));
		assert_near(penguin.lit, "#94857C", "the penguin a ferryman sees");
		assert_eq!(ghost(hex("#3B3F46"), None).deep, Rgb(0x28, 0x28, 0x28));
		assert!(ghost(hex("#F2EEE6"), None).specular > 0.0);
		assert_eq!(ghost(hex("#C9722E"), None).specular, 0.0);
	}

	#[test]
	fn colours_convert_to_linear_light() {
		assert_eq!(linear(Rgb(255, 255, 255)), [1.0; 3]);
		assert_eq!(linear(Rgb(0, 0, 0)), [0.0; 3]);
		assert!((linear(Rgb(128, 128, 128))[0] - 0.2158).abs() < 1e-3);
	}
}
