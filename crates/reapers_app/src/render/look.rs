//! How a reaper is coloured: its colourway, as its omen and outfit leave it,
//! turned into material overrides on the model's named surfaces.
//!
//! The Flutter game drew every reaper surface with its own shader
//! (`reaper.fmat`), whose vertex masks charred and hollowed the cloth.
//! `leptoskit_3d` draws the model's own materials, so a look here is what an
//! override can say: a tint, a light of its own, and a rim. The numbers are
//! the Flutter `ReaperLook`'s, so the colours are the same; the burn front and
//! the sketched outline are what is lost.

use leptoskit::scene3d::MaterialOverride;
use leptoskit::scene3d::Rim;
use reapers_rules::Omen;
use reapers_rules::Outfit;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;

/// A colour as the art build writes it: `0xRRGGBB` in sRGB.
pub type Srgb = u32;

/// One colourway of a calling: what the cloth is dyed and what colour the
/// reaper's single light burns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colourway {
	/// The dominant cloth, a tinted black. Never a neutral one.
	pub cloth: Srgb,
	/// The second cloth: a mantle, a hat, a wrap.
	pub trim: Srgb,
	/// The only saturated colour on the figure.
	pub accent: Srgb,
	/// The light that edges the figure from behind, so it separates from
	/// whatever the camera sees.
	pub rim: Srgb,
}

const fn way(cloth: Srgb, trim: Srgb, accent: Srgb, rim: Srgb) -> Colourway {
	Colourway {
		cloth,
		trim,
		accent,
		rim,
	}
}

/// The three colourways of each calling; see
/// docs/research/dark-games-and-ar-hunting.md for where they come from.
pub const fn colourways(calling: ReaperClass) -> [Colourway; 3] {
	match calling {
		// Straw ochre and rust on brown-black; a bone-white light.
		ReaperClass::Harvester => {
			[
				way(0x002B_2119, 0x007A_5A2E, 0x00F3_EBD8, 0x00B9_A98A),
				way(0x002C_1913, 0x006E_3520, 0x00FF_E9C9, 0x00C7_9A7A),
				way(0x001E_211B, 0x005E_5A34, 0x00E6_F0D8, 0x009F_AE8E),
			]
		}
		// Verdigris and wet slate on green-black; an amber lantern.
		ReaperClass::Ferryman => {
			[
				way(0x0016_221F, 0x002F_5A50, 0x00FF_B347, 0x006F_A596),
				way(0x001A_1F26, 0x003A_4A5C, 0x00FF_C56B, 0x007D_93AD),
				way(0x001F_2118, 0x004B_5230, 0x00FF_9A3D, 0x008F_9A6A),
			]
		}
		// Tarnished gold on blue-black; one cold white light in the mask.
		ReaperClass::Weigher => {
			[
				way(0x0014_1926, 0x006B_5524, 0x00E8_F4FF, 0x007F_92C2),
				way(0x001B_1626, 0x006B_4A24, 0x00F3_ECFF, 0x009A_86C7),
				way(0x0010_201F, 0x005E_5A2A, 0x00DF_FBF4, 0x006F_B0A6),
			]
		}
		// Raven blue-black with red thread; a red spear tip.
		ReaperClass::Chooser => {
			[
				way(0x0012_141C, 0x004A_1414, 0x00FF_3B2F, 0x006C_7CA8),
				way(0x0016_121A, 0x0055_182A, 0x00FF_4D6A, 0x008A_72A6),
				way(0x0011_1A1C, 0x004F_2212, 0x00FF_6A2A, 0x006C_9CA6),
			]
		}
		// Gunmetal on red-black; ember orange inside the collar.
		ReaperClass::Hollow => {
			[
				way(0x0021_1416, 0x003B_3E46, 0x00FF_7A1A, 0x00A0_7A7A),
				way(0x001C_1418, 0x0045_404A, 0x00FF_9B2E, 0x009A_7A92),
				way(0x0023_1A12, 0x003F_4244, 0x00FF_5A1F, 0x00A8_8E72),
			]
		}
		// Washed grey-green on grey-black; pale silver in the bell.
		ReaperClass::Mourner => {
			[
				way(0x001C_211F, 0x0055_645C, 0x00DD_EBFF, 0x00A5_B8B0),
				way(0x001E_1E24, 0x005C_5E70, 0x00ED_E6FF, 0x00AA_A8C4),
				way(0x0021_1F1B, 0x0066_604E, 0x00F2_F4E2, 0x00B8_B29A),
			]
		}
	}
}

/// The colourway `genome` was born to, as its omen leaves it.
pub fn colourway(genome: &ReaperGenome) -> Colourway {
	let ways = colourways(genome.reaper_class);
	omened(ways[genome.palette as usize % ways.len()], genome.omen)
}

fn omened(colourway: Colourway, omen: Omen) -> Colourway {
	match omen {
		Omen::None => colourway,
		Omen::Ashen => {
			Colourway {
				cloth: lerp(grey(colourway.cloth), 0x008A_857C, 0.45),
				trim: lerp(grey(colourway.trim), 0x00B5_AFA2, 0.5),
				accent: lerp(colourway.accent, 0x00F2_EEE4, 0.7),
				rim: 0x00BF_C6CC,
			}
		}
		Omen::Gilded => {
			Colourway {
				trim: lerp(colourway.trim, 0x008A_6A2A, 0.6),
				accent: 0x00FF_C65A,
				rim: 0x00E0_B870,
				..colourway
			}
		}
		Omen::Eclipsed => {
			Colourway {
				cloth: scale(colourway.cloth, 0.3),
				trim: scale(colourway.trim, 0.3),
				rim: colourway.accent,
				..colourway
			}
		}
	}
}

/// What an outfit does to every surface it reaches.
#[derive(Clone, Copy, Debug)]
struct OutfitState {
	/// How far the cloth is pulled back toward raw, undyed cloth.
	undyed: f64,
	/// Multiplier on the cloth's brightness.
	shade: f64,
	char: f64,
	hollow: f64,
	ember: f64,
	glow: f64,
	rim: f64,
}

const fn outfit_state(outfit: Outfit) -> OutfitState {
	match outfit {
		Outfit::Shrouded => {
			OutfitState {
				undyed: 0.55,
				shade: 1.0,
				char: 0.0,
				hollow: 0.0,
				ember: 0.0,
				glow: 2.4,
				rim: 1.1,
			}
		}
		Outfit::Sworn => {
			OutfitState {
				undyed: 0.0,
				shade: 1.0,
				char: 0.0,
				hollow: 0.0,
				ember: 0.0,
				glow: 3.6,
				rim: 1.3,
			}
		}
		Outfit::Scorched => {
			OutfitState {
				undyed: 0.0,
				shade: 0.8,
				char: 0.5,
				hollow: 0.0,
				ember: 3.2,
				glow: 5.5,
				rim: 1.6,
			}
		}
		Outfit::Hollowed => {
			OutfitState {
				undyed: 0.0,
				shade: 0.6,
				char: 0.8,
				hollow: 0.42,
				ember: 2.2,
				glow: 9.0,
				rim: 1.9,
			}
		}
	}
}

/// Raw cloth: what a shrouded outfit looks like before any dye.
const RAW_CLOTH: Srgb = 0x005E_5245;

/// The burning edge between cloth and char.
const EMBER: Srgb = 0x00FF_6A1F;

/// How one named surface of a reaper is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceLook {
	pub tint: Srgb,
	/// Whether the outfit's char and hollowing reach this surface. Bone and
	/// the reaper's light do not burn.
	pub burns: bool,
	/// How strongly the surface lights itself with the accent colour.
	pub glow_strength: f64,
	/// Whether the rim reaches it: the Flutter shader left it off surfaces
	/// with no grime (the light, the spirit, the void).
	pub rimmed: bool,
}

/// Everything needed to draw one reaper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReaperLook {
	pub colourway: Colourway,
	cloth: Srgb,
	trim: Srgb,
	gilded: bool,
	eclipsed: bool,
	glow: f64,
	pub rim_strength: f64,
	pub ember_strength: f64,
	/// How far char has climbed, from 0 (none) to 1.
	pub char_level: f64,
	/// How much of the figure has been eaten away, from 0 (none) to 1.
	pub hollow_level: f64,
	/// Whether the head has gone, leaving only what the reaper wears on it.
	pub headless: bool,
}

impl ReaperLook {
	/// The look of `genome` wearing `outfit`.
	pub fn of(genome: &ReaperGenome, outfit: Outfit) -> Self {
		let colourway = colourway(genome);
		let state = outfit_state(outfit);
		let gaze = 0.7 + 0.6 * ReaperGenome::fraction(genome.gaze);
		let eclipse = if genome.omen == Omen::Eclipsed {
			1.6
		} else {
			1.0
		};
		Self {
			colourway,
			cloth: dye(colourway.cloth, state),
			trim: dye(colourway.trim, state),
			gilded: genome.omen == Omen::Gilded,
			eclipsed: genome.omen == Omen::Eclipsed,
			glow: state.glow * gaze * eclipse,
			rim_strength: state.rim,
			ember_strength: state.ember,
			char_level: state.char,
			hollow_level: state.hollow,
			headless: outfit == Outfit::Hollowed,
		}
	}

	/// The look for a material the art build named `material`, which may
	/// carry Blender's `.001` suffix. Unknown names are drawn as cloth.
	pub fn surface(&self, material: &str) -> SurfaceLook {
		let plain = |tint| {
			SurfaceLook {
				tint,
				burns: true,
				glow_strength: 0.0,
				rimmed: true,
			}
		};
		match material.split('.').next().unwrap_or(material) {
			"cloth_alt" => plain(self.trim),
			"feather" => plain(scale(self.cloth, 0.7)),
			"paper" => plain(lerp(self.trim, 0x00B8_A888, 0.5)),
			"rope" => plain(lerp(self.trim, 0x006E_5B3A, 0.6)),
			"wood" => plain(0x002A_1D14),
			"bone" => {
				SurfaceLook {
					burns: false,
					..plain(if self.eclipsed {
						0x004A_4640
					} else {
						0x00B9_AE95
					})
				}
			}
			"metal" => {
				plain(if self.gilded {
					0x00B0_8A3C
				} else {
					0x004A_4C52
				})
			}
			"gold" => plain(0x00B0_8A3C),
			"accent" => {
				SurfaceLook {
					tint: 0x0000_0000,
					burns: false,
					glow_strength: self.glow,
					rimmed: false,
				}
			}
			"spirit" => {
				SurfaceLook {
					tint: lerp(0x0011_1B24, self.colourway.accent, 0.15),
					burns: false,
					// The Flutter shader's 0.08 was tonemapped with its bloom;
					// drawn here without it, the spirit's limbs, which show
					// through the robe's gaps, glowed as solid colour.
					glow_strength: self.glow * 0.025,
					rimmed: false,
				}
			}
			"void" => {
				SurfaceLook {
					tint: 0x0001_0102,
					burns: false,
					glow_strength: 0.0,
					rimmed: false,
				}
			}
			_ => plain(self.cloth),
		}
	}

	/// The override that draws `material` in this look.
	///
	/// Char darkens a surface it reaches, and its embers light it faintly
	/// from within; the accent and the spirit light themselves in the
	/// reaper's light; everything with grime takes the rim.
	pub fn material(&self, material: &str) -> MaterialOverride {
		let surface = self.surface(material);
		let charred = if surface.burns {
			1.0 - 0.55 * self.char_level
		} else {
			1.0
		};
		let [red, green, blue] = linear(surface.tint);
		let emissive = if surface.glow_strength > 0.0 {
			Some((linear(self.colourway.accent), surface.glow_strength))
		} else if surface.burns && self.ember_strength > 0.0 {
			Some((linear(EMBER), self.ember_strength * self.char_level * 0.03))
		} else {
			None
		};
		MaterialOverride {
			color: Some([
				red * charred as f32,
				green * charred as f32,
				blue * charred as f32,
				1.0,
			]),
			emissive: Some(emissive.map_or([0.0; 3], |(colour, _)| colour)),
			emissive_strength: Some(emissive.map_or(0.0, |(_, strength)| strength as f32)),
			opacity: None,
			rim: surface.rimmed.then(|| {
				Rim {
					color: linear(self.colourway.rim),
					strength: (0.35 * self.rim_strength) as f32,
					power: 3.0,
				}
			}),
		}
	}
}

/// Every material name the art build gives a reaper or its weapons.
pub const MATERIALS: [&str; 12] = [
	"cloth",
	"cloth_alt",
	"feather",
	"paper",
	"rope",
	"wood",
	"bone",
	"metal",
	"gold",
	"accent",
	"spirit",
	"void",
];

fn dye(dyed: Srgb, state: OutfitState) -> Srgb {
	scale(lerp(dyed, RAW_CLOTH, state.undyed), state.shade)
}

fn channels(colour: Srgb) -> [f64; 3] {
	[
		f64::from((colour >> 16) & 0xFF) / 255.0,
		f64::from((colour >> 8) & 0xFF) / 255.0,
		f64::from(colour & 0xFF) / 255.0,
	]
}

fn pack([red, green, blue]: [f64; 3]) -> Srgb {
	let byte = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u32;
	(byte(red) << 16) | (byte(green) << 8) | byte(blue)
}

/// Mixes two colours as Flutter's `Color.lerp` does: channel by channel, in
/// sRGB.
pub fn lerp(from: Srgb, to: Srgb, t: f64) -> Srgb {
	let (from, to) = (channels(from), channels(to));
	pack([0, 1, 2].map(|channel| from[channel] + (to[channel] - from[channel]) * t))
}

fn scale(colour: Srgb, factor: f64) -> Srgb {
	pack(channels(colour).map(|channel| channel * factor))
}

fn grey(colour: Srgb) -> Srgb {
	let [red, green, blue] = channels(colour);
	let luma = 0.2126 * red + 0.7152 * green + 0.0722 * blue;
	pack([luma; 3])
}

/// An sRGB colour in linear light, as the renderer takes colours.
pub fn linear(colour: Srgb) -> [f32; 3] {
	channels(colour).map(|channel| {
		let value = if channel <= 0.04045 {
			channel / 12.92
		} else {
			((channel + 0.055) / 1.055).powf(2.4)
		};
		value as f32
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::theme::reaper_light;

	#[test]
	fn the_light_is_the_interface_light() {
		for seed in 0..60 {
			let genome = ReaperGenome::shuffle(seed);
			let light = reaper_light(&genome);
			let accent = colourway(&genome).accent;
			assert_eq!(
				accent,
				(u32::from(light.0) << 16) | (u32::from(light.1) << 8) | u32::from(light.2),
				"{genome:?}"
			);
		}
	}

	#[test]
	fn an_eclipse_darkens_the_cloth_and_rims_with_the_light() {
		let genome = ReaperGenome {
			omen: Omen::Eclipsed,
			..ReaperGenome::BORROWED
		};
		let plain = colourways(genome.reaper_class)[0];
		let eclipsed = colourway(&genome);
		assert_eq!(eclipsed.rim, plain.accent);
		assert!(channels(eclipsed.cloth)[0] < channels(plain.cloth)[0]);
	}

	#[test]
	fn outfits_burn_brighter_as_they_go() {
		let genome = ReaperGenome::BORROWED;
		let glow = |outfit| {
			ReaperLook::of(&genome, outfit)
				.material("accent")
				.emissive_strength
				.unwrap()
		};
		assert!(glow(Outfit::Shrouded) < glow(Outfit::Sworn));
		assert!(glow(Outfit::Sworn) < glow(Outfit::Scorched));
		assert!(glow(Outfit::Scorched) < glow(Outfit::Hollowed));
		let hollowed = ReaperLook::of(&genome, Outfit::Hollowed);
		assert!(hollowed.headless);
		// Char reaches cloth but never bone.
		let cloth = hollowed.material("cloth").color.unwrap()[0];
		let fresh = ReaperLook::of(&genome, Outfit::Sworn).material("cloth");
		assert!(cloth < fresh.color.unwrap()[0] || fresh.color.unwrap()[0] == 0.0);
		assert_eq!(
			hollowed.material("bone").color,
			ReaperLook::of(&genome, Outfit::Sworn)
				.material("bone")
				.color
		);
	}

	#[test]
	fn blender_suffixes_and_unknown_names_fall_back() {
		let look = ReaperLook::of(&ReaperGenome::BORROWED, Outfit::Sworn);
		assert_eq!(look.surface("metal.001"), look.surface("metal"));
		assert_eq!(look.surface("anything"), look.surface("cloth"));
	}

	#[test]
	fn colours_convert_to_linear_light() {
		assert_eq!(linear(0x00FF_FFFF), [1.0; 3]);
		assert_eq!(linear(0x0000_0000), [0.0; 3]);
		let half = linear(0x0080_8080)[0];
		assert!((half - 0.2158).abs() < 1e-3);
		assert_eq!(lerp(0x0000_0000, 0x00FF_FFFF, 0.5), 0x0080_8080);
	}
}
