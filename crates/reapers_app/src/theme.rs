//! The look of everything that is not 3D: a funeral ledger, set in bone on
//! tinted black, ruled with hairlines.
//!
//! There is one colour, and it is not chosen here. Each reaper burns with a
//! single light, and the interface borrows it: a harvester's screens are lit
//! bone-white, a ferryman's amber, a chooser's red. [`reaper_light`] says
//! which.
//!
//! Colours are written as Tailwind arbitrary values (`text-[#e9e4d4]`), which
//! leptoskit compiles to native styles and to CSS alike. The two typefaces,
//! Fraunces for display and DM Sans for everything else, are the web shell's
//! (`assets/app.css`): leptoskit has no font-family utility, so headings
//! ([`leptoskit::prelude::Heading`]) are set in Fraunces by that stylesheet,
//! and native platforms use the system face.

use reapers_rules::LAMPORTS_PER_SOL;
use reapers_rules::Omen;
use reapers_rules::ReaperClass;
use reapers_rules::ReaperGenome;

/// An sRGB colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
	pub const fn from_hex(hex: u32) -> Self {
		Self((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
	}

	/// `#rrggbb`.
	pub fn hex(self) -> String {
		format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
	}

	/// `#rrggbbaa`, at `alpha` from 0 to 1.
	pub fn hex_alpha(self, alpha: f64) -> String {
		format!(
			"{}{:02x}",
			self.hex(),
			(alpha.clamp(0.0, 1.0) * 255.0).round() as u8
		)
	}

	/// The colour `t` of the way from this one to `other`, as Flutter's
	/// `Color.lerp` mixes them.
	#[must_use]
	pub fn lerp(self, other: Self, t: f64) -> Self {
		let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8;
		Self(
			mix(self.0, other.0),
			mix(self.1, other.1),
			mix(self.2, other.2),
		)
	}
}

/// The page: a blue-black, never a neutral one.
pub const INK: Rgb = Rgb::from_hex(0x07_08_0B);
/// A surface raised off the page.
pub const INK_RAISED: Rgb = Rgb::from_hex(0x0F_11_16);
/// Text and rules.
pub const BONE: Rgb = Rgb::from_hex(0xE9_E4_D4);
/// Text that gives way to other text.
pub const ASH: Rgb = Rgb::from_hex(0x8E_89_7D);
/// A loss, a refusal, a flinch.
pub const WOUND: Rgb = Rgb::from_hex(0xD4_55_3F);
/// The light of a reaper nobody has bound yet.
pub const UNBOUND: Rgb = Rgb::from_hex(0xBF_C8_CF);

/// Tailwind classes for the fixed colours, so a screen writes `text-bone`
/// the way the Flutter game wrote `Veil.bone`.
pub mod class {
	pub const INK_BG: &str = "bg-[#07080b]";
	pub const RAISED_BG: &str = "bg-[#0f1116]";
	pub const BONE: &str = "text-[#e9e4d4]";
	pub const ASH: &str = "text-[#8e897d]";
	pub const WOUND: &str = "text-[#d4553f]";
	pub const INK: &str = "text-[#07080b]";
	/// A rule or border that is barely there: bone at 18%.
	pub const HAIRLINE_BORDER: &str = "border-[#e9e4d42e]";
	pub const HAIRLINE_BG: &str = "bg-[#e9e4d42e]";
}

/// The widest a column of content grows, so screens hold their shape on a
/// tablet.
pub const CONTENT_WIDTH: &str = "max-w-[520px]";

/// The four accents of each calling's colourways, as the Flutter game's
/// `classColourways` lists them.
const fn accents(reaper_class: ReaperClass) -> [u32; 3] {
	match reaper_class {
		ReaperClass::Harvester => [0xF3_EB_D8, 0xFF_E9_C9, 0xE6_F0_D8],
		ReaperClass::Ferryman => [0xFF_B3_47, 0xFF_C5_6B, 0xFF_9A_3D],
		ReaperClass::Weigher => [0xE8_F4_FF, 0xF3_EC_FF, 0xDF_FB_F4],
		ReaperClass::Chooser => [0xFF_3B_2F, 0xFF_4D_6A, 0xFF_6A_2A],
		ReaperClass::Hollow => [0xFF_7A_1A, 0xFF_9B_2E, 0xFF_5A_1F],
		ReaperClass::Mourner => [0xDD_EB_FF, 0xED_E6_FF, 0xF2_F4_E2],
	}
}

/// The colour `genome`'s one light burns: its colourway's accent, as its
/// omen leaves it. The interface is lit with the same colour.
pub fn reaper_light(genome: &ReaperGenome) -> Rgb {
	let palette = accents(genome.reaper_class);
	let accent = Rgb::from_hex(palette[genome.palette as usize % palette.len()]);
	match genome.omen {
		Omen::Ashen => accent.lerp(Rgb::from_hex(0xF2_EE_E4), 0.7),
		Omen::Gilded => Rgb::from_hex(0xFF_C6_5A),
		Omen::None | Omen::Eclipsed => accent,
	}
}

/// The numerals a ledger counts in.
pub fn roman_numeral(value: i64) -> String {
	const SYMBOLS: [(i64, &str); 13] = [
		(1000, "M"),
		(900, "CM"),
		(500, "D"),
		(400, "CD"),
		(100, "C"),
		(90, "XC"),
		(50, "L"),
		(40, "XL"),
		(10, "X"),
		(9, "IX"),
		(5, "V"),
		(4, "IV"),
		(1, "I"),
	];
	if value <= 0 {
		return "N".into();
	}
	let mut remaining = value;
	let mut numeral = String::new();
	for (amount, symbol) in SYMBOLS {
		while remaining >= amount {
			numeral.push_str(symbol);
			remaining -= amount;
		}
	}
	numeral
}

/// A balance in lamports as SOL, to two or three places.
pub fn sol_amount(lamports: i64) -> String {
	let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;
	if (sol * 100.0 - (sol * 100.0).round()).abs() < f64::EPSILON {
		format!("{sol:.2}")
	} else {
		format!("{sol:.3}")
	}
}

/// An address or signature cut to its ends, as explorers show them.
pub fn short_address(address: &str) -> String {
	let characters: Vec<char> = address.chars().collect();
	if characters.len() <= 12 {
		return address.to_owned();
	}
	let head: String = characters[..4].iter().collect();
	let tail: String = characters[characters.len() - 4..].iter().collect();
	format!("{head}…{tail}")
}

/// A stable seed for `text`, so two boxes with different labels are not
/// drawn with the same wobble: the Flutter game's FNV-1a.
pub fn seed_for(text: &str) -> u32 {
	let mut hash: u32 = 0x811c_9dc5;
	for unit in text.encode_utf16() {
		hash = (hash ^ u32::from(unit)).wrapping_mul(0x0100_0193) & 0x7fff_ffff;
	}
	hash
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn numerals_count_as_a_ledger_does() {
		assert_eq!(roman_numeral(0), "N");
		assert_eq!(roman_numeral(4), "IV");
		assert_eq!(roman_numeral(14), "XIV");
		assert_eq!(roman_numeral(30), "XXX");
		assert_eq!(roman_numeral(1999), "MCMXCIX");
	}

	#[test]
	fn sol_shows_two_places_unless_it_needs_three() {
		assert_eq!(sol_amount(3_000_000_000), "3.00");
		assert_eq!(sol_amount(2_750_000_000), "2.75");
		assert_eq!(sol_amount(2_755_000_000), "2.755");
	}

	#[test]
	fn an_address_is_cut_to_its_ends() {
		assert_eq!(short_address("short"), "short");
		assert_eq!(short_address("ABCDEFGHIJKLMNOP"), "ABCD…MNOP");
	}

	#[test]
	fn a_gilded_omen_burns_gold_whatever_the_calling() {
		let mut genome = ReaperGenome::shuffle(1);
		genome.omen = Omen::Gilded;
		assert_eq!(reaper_light(&genome), Rgb::from_hex(0xFF_C6_5A));
		genome.omen = Omen::None;
		assert_eq!(
			reaper_light(&genome),
			Rgb::from_hex(accents(genome.reaper_class)[genome.palette as usize % 3])
		);
	}

	#[test]
	fn colours_write_as_hex() {
		assert_eq!(BONE.hex(), "#e9e4d4");
		assert_eq!(BONE.hex_alpha(0.18), "#e9e4d42e");
	}

	#[test]
	fn seeds_follow_the_text() {
		assert_eq!(seed_for("Hunt"), seed_for("Hunt"));
		assert_ne!(seed_for("Hunt"), seed_for("Vault"));
	}
}
