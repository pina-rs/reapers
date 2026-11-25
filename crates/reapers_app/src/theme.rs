//! Second Ink: every colour, size, measure and timing the interface is
//! printed with, as the art bible names them. This module is the one place
//! they are written; every screen, the kit in [`crate::ui`] and the 3D views
//! read them from here.
//!
//! The interface is a two-plate relief print on a dark page. Key is every
//! line and the words on paper; paper is the stock slips and sheets are cut
//! from; the second ink, the spot, is the reaper's own light, one per class
//! ([`light`]). Wound is a third ink, kept for loss and error, and never
//! carries words on a dark ground.
//!
//! - **Inks and grounds** (§3.1): [`PAPER`], [`ASH`], [`RAISED`],
//!   [`VIGIL`], [`SCRIM`], [`KEY`], [`WOUND`].
//! - **The six lights** (§3.2): [`CHAFF`], [`LANTERN`], [`VERDIGRIS`],
//!   [`COBALT`], [`WISP`], [`MAUVE`], and [`light`] to pick one.
//! - **Type** (§7): three [`Face`]s and the seven [`Step`]s of the scale.
//! - **Measure**: the spacing scale in [`space`], stroke widths in
//!   [`stroke`], and springs and durations in [`motion`].
//! - **Contrast** (§15.1): [`Rgb::contrast`], and a test that holds every
//!   allowed pair to its threshold.
//!
//! Colours are written as Tailwind arbitrary values (`text-[#ece3cf]`),
//! which leptoskit compiles to native styles and to CSS alike; [`class`]
//! holds the ones screens use most, each checked against its token.

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

	/// The colour as a vector paint, opaque.
	pub const fn color(self) -> leptoskit::prelude::Color {
		leptoskit::prelude::Color::rgba(self.0, self.1, self.2, 255)
	}

	/// The colour as a vector paint at `alpha`, from 0 to 1.
	pub fn color_alpha(self, alpha: f64) -> leptoskit::prelude::Color {
		leptoskit::prelude::Color::rgba(
			self.0,
			self.1,
			self.2,
			(alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
		)
	}

	/// WCAG relative luminance, from 0 (black) to 1 (white).
	pub fn luminance(self) -> f64 {
		let linear = |channel: u8| {
			let value = f64::from(channel) / 255.0;
			if value <= 0.040_45 {
				value / 12.92
			} else {
				((value + 0.055) / 1.055).powf(2.4)
			}
		};
		0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
	}

	/// The WCAG contrast ratio between this colour and `other`, from 1 to 21,
	/// the same whichever is the ground.
	pub fn contrast(self, other: Self) -> f64 {
		let (a, b) = (self.luminance() + 0.05, other.luminance() + 0.05);
		a.max(b) / a.min(b)
	}
}

// Inks and grounds (§3.1). Every neutral is key printed over paper at a
// coverage; the comments give it.

/// The stock: slips, sheets, cards on the page, and words on every dark
/// ground. Coverage 0.
pub const PAPER: Rgb = Rgb::from_hex(0x00EC_E3CF);
/// Quiet words on vigil and scrim, inactive icons, disabled labels. Never on
/// paper, and never on raised under [`Step::S5`]. Coverage about .34.
pub const ASH: Rgb = Rgb::from_hex(0x00A3_9C93);
/// Cards and wells on the page, and rules. Coverage .78.
pub const RAISED: Rgb = Rgb::from_hex(0x0045_4042);
/// The page: every screen's ground, always. Coverage .86.
pub const VIGIL: Rgb = Rgb::from_hex(0x0034_2F33);
/// Wells, a disabled dial's fill. Never under small words. Coverage .90.
pub const SCRIM: Rgb = Rgb::from_hex(0x002B_272C);
/// Every line, and the words on paper and on spot. Coverage 1.
pub const KEY: Rgb = Rgb::from_hex(0x0016_121A);
/// The third ink: loss and error only, always a mark with paper words
/// beside it on a dark ground. Words in wound sit on paper alone.
pub const WOUND: Rgb = Rgb::from_hex(0x00B9_2A20);

// The six lights (§3.2): one spot per class, whatever its colourway.

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

// The ledger's names. Screens that are not yet rebuilt on the kit still
// say them; each now prints in the Second Ink it stands for, and goes when
// the last screen saying it does.

/// Words and rules on a dark ground: [`PAPER`].
pub const BONE: Rgb = PAPER;
/// The darkest ground, under the hunt's words and the screens not yet on
/// the vigil page: [`KEY`].
pub const INK: Rgb = KEY;
/// The interface light before any reaper is bound: [`PAPER`], since there
/// is no spot until the oath prints one (§9, Oath).
pub const UNBOUND: Rgb = PAPER;

/// Tailwind classes for the inks screens use most, each the token's own hex
/// (a test holds them to it).
pub mod class {
	/// The page.
	pub const VIGIL_BG: &str = "bg-[#342f33]";
	/// A card or well on the page.
	pub const RAISED_BG: &str = "bg-[#454042]";
	/// A slip, a sheet.
	pub const PAPER_BG: &str = "bg-[#ece3cf]";
	/// The darkest ground: key.
	pub const KEY_BG: &str = "bg-[#16121a]";
	/// A well darker than the page.
	pub const SCRIM_BG: &str = "bg-[#2b272c]";
	/// Words on a dark ground.
	pub const PAPER: &str = "text-[#ece3cf]";
	/// Quiet words on vigil or scrim.
	pub const ASH: &str = "text-[#a39c93]";
	/// Words on paper or on spot.
	pub const KEY: &str = "text-[#16121a]";
	/// Words in wound: on paper only.
	pub const WOUND: &str = "text-[#b92a20]";
	/// The ledger's name for the key ground.
	pub const INK_BG: &str = KEY_BG;
	/// The ledger's name for paper words.
	pub const BONE: &str = PAPER;
	/// The ledger's name for key words.
	pub const INK: &str = KEY;
	/// A rule or border that is barely there: paper at 18%.
	pub const HAIRLINE_BORDER: &str = "border-[#ece3cf2e]";
	/// A hairline drawn as a filled box: paper at 18%.
	pub const HAIRLINE_BG: &str = "bg-[#ece3cf2e]";
}

/// The widest a column of content grows, so screens hold their shape on a
/// tablet (§9).
pub const CONTENT_WIDTH: &str = "max-w-[520px]";

/// The three Skribble faces (§7.1), each named on the web by a CSS custom
/// property the shell defines (`assets/app.css`), so a class reads
/// `font-[var(--display)]` on every platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
	/// Skribble Black: names, stamps and chops, the one primary action per
	/// screen, the dial words and stamped figures. Nothing else.
	Display,
	/// Skribble Noto Sketch: sentences, secondary buttons, notices, errors.
	Text,
	/// Skribble Mono Gentle Bold: every number, and every uppercase label.
	Figures,
}

impl Face {
	/// The class that sets this face.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Display => "font-[var(--display)]",
			Self::Text => "font-[var(--text)]",
			Self::Figures => "font-[var(--figures)]",
		}
	}
}

/// A step of the type scale (§7.2): seven sizes at a ratio of about 1.25,
/// each with its own line height.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
	/// 11/14, Figures labels: the smallest anywhere, never in the hunt.
	S1,
	/// 12/16, Figures: the hunt's smallest, captions, field labels.
	S2,
	/// 14/19, Text and Figures: sentences, the SOL balance, notices.
	S3,
	/// 17/22, Text and Figures: secondary buttons, lead sentences.
	S4,
	/// 22/24, Display: dial words, sheet titles. The smallest the spot may
	/// carry.
	S5,
	/// 30/30, Display: the primary action.
	S6,
	/// 40/38, Display: names, result tallies.
	S7,
}

impl Step {
	/// Every step, smallest first.
	pub const ALL: [Self; 7] = [
		Self::S1,
		Self::S2,
		Self::S3,
		Self::S4,
		Self::S5,
		Self::S6,
		Self::S7,
	];
	/// The smallest step words in a light may be set at, on a dark ground.
	pub const SPOT_MINIMUM: Self = Self::S5;

	/// Size in points.
	pub const fn size(self) -> u32 {
		match self {
			Self::S1 => 11,
			Self::S2 => 12,
			Self::S3 => 14,
			Self::S4 => 17,
			Self::S5 => 22,
			Self::S6 => 30,
			Self::S7 => 40,
		}
	}

	/// Line height in points.
	pub const fn leading(self) -> u32 {
		match self {
			Self::S1 => 14,
			Self::S2 => 16,
			Self::S3 => 19,
			Self::S4 => 22,
			Self::S5 => 24,
			Self::S6 => 30,
			Self::S7 => 38,
		}
	}
}

/// The advance of each printable ASCII character in the Display face
/// (Skribble Black), in thousandths of an em, from U+0020 to U+007E: read
/// from the font's `hmtx` table, so a name can be fitted to its box before
/// it is laid out.
const DISPLAY_ADVANCES: [u16; 95] = [
	300, 350, 600, 600, 600, 600, 600, 350, 400, 400, 600, 600, 350, 450, 350, 600, 600, 600, 600,
	600, 600, 600, 600, 600, 600, 600, 350, 350, 600, 600, 600, 600, 1050, 650, 600, 600, 650, 550,
	550, 650, 650, 500, 500, 600, 550, 800, 650, 650, 600, 650, 600, 600, 600, 650, 650, 950, 600,
	650, 600, 450, 600, 450, 600, 600, 350, 600, 600, 500, 600, 600, 400, 600, 600, 350, 350, 600,
	350, 850, 600, 600, 600, 600, 450, 550, 500, 600, 600, 800, 600, 600, 550, 550, 300, 550, 600,
];

/// How wide `text` sets in the Display face at one em, in ems. Characters
/// outside ASCII take the face's widest common advance, so the estimate
/// errs toward shrinking.
pub fn display_width(text: &str) -> f32 {
	text.chars()
		.map(|character| {
			let index = (character as u32).wrapping_sub(0x20) as usize;
			f32::from(DISPLAY_ADVANCES.get(index).copied().unwrap_or(650)) / 1000.0
		})
		.sum()
}

/// The size a Display word in a box `width` points wide sets at (§7.2,
/// fitted Display): its `step`, shrunk 1 pt at a time until it breaks
/// into no more than two lines, and never below [`Step::S5`].
pub fn fit_display(text: &str, width: f32, step: Step) -> u32 {
	let words: Vec<f32> = text.split_whitespace().map(display_width).collect();
	let space = display_width(" ");
	let lines_at = |size: u32| {
		let size = size as f32;
		let mut lines = 1;
		let mut line = 0.0;
		for &word in &words {
			let word = word * size;
			if line > 0.0 && line + space * size + word > width {
				lines += 1;
				line = word;
			} else if line > 0.0 {
				line += space * size + word;
			} else {
				line = word;
			}
			if word > width {
				// A word wider than the box would break mid-word: count it
				// as a line too many.
				lines += 2;
			}
		}
		lines
	};
	let floor = Step::S5.size();
	(floor..=step.size().max(floor))
		.rev()
		.find(|&size| lines_at(size) <= 2)
		.unwrap_or(floor)
}

/// The spacing scale. The bible gives its measures component by component;
/// they all fall on a 4 pt grid, and these are the steps of it the kit and
/// the screens use, with the named measures (§8, §9, §15.3).
pub mod space {
	/// 4 pt: a mark and its word.
	pub const S1: u32 = 4;
	/// 8 pt: a label and what it labels.
	pub const S2: u32 = 8;
	/// 12 pt: lines of one group.
	pub const S3: u32 = 12;
	/// 16 pt: one group and the next.
	pub const S4: u32 = 16;
	/// 20 pt: the page's side margins.
	pub const S5: u32 = 20;
	/// 24 pt: one section and the next.
	pub const S6: u32 = 24;
	/// 32 pt: around the primary action.
	pub const S7: u32 = 32;
	/// 40 pt: a screen's head and its body.
	pub const S8: u32 = 40;

	/// The side margin every screen keeps.
	pub const MARGIN: u32 = S5;
	/// The widest a column grows.
	pub const COLUMN: u32 = 520;
	/// The smallest touch target anywhere.
	pub const TARGET: u32 = 44;
	/// The primary button's height.
	pub const PRIMARY: u32 = 54;
	/// A secondary button's height.
	pub const SECONDARY: u32 = 46;
	/// A tab's touch target.
	pub const TAB_WIDTH: u32 = 78;
	pub const TAB_HEIGHT: u32 = 56;
	/// How far the spot plate sits out of register at rest, down and right.
	pub const REGISTER: f32 = 3.0;
	/// The lamp disc behind the sanctum's reaper: its radius.
	pub const LAMP_RADIUS: f32 = 132.0;
}

/// Stroke widths, in points (§8). Two passes for outlines that carry
/// state, one for rules and ticks, never three.
pub mod stroke {
	/// A slip's outline, a secondary button's, an input's.
	pub const SLIP: f32 = 1.8;
	/// A card's outline.
	pub const CARD: f32 = 1.6;
	/// The primary button's outline.
	pub const PRIMARY: f32 = 2.4;
	/// Switches, checkboxes, radios.
	pub const TOGGLE: f32 = 1.7;
	/// Icons.
	pub const ICON: f32 = 1.7;
	/// A meter's outline.
	pub const METER: f32 = 1.5;
	/// A divider.
	pub const DIVIDER: f32 = 1.4;
	/// The rule over the tab bar.
	pub const TAB_RULE: f32 = 1.6;
	/// The underline under the active tab, a focused input's outline, a
	/// pressed link's underline, a wound strike.
	pub const EMPHASIS: f32 = 2.2;
	/// A text link's underline at rest.
	pub const LINK: f32 = 1.2;
	/// A stroke of the tally.
	pub const TALLY: f32 = 1.6;
	/// A stamp's ring.
	pub const STAMP: f32 = 2.2;
	/// The class mark.
	pub const MARK: f32 = 1.6;
}

/// Springs and durations (§13). Printed decoration moves on twos; anything
/// that follows a finger or the world runs on springs.
pub mod motion {
	/// A spring: stiffness and damping for a mass of 1.
	#[derive(Clone, Copy, Debug, PartialEq)]
	pub struct Spring {
		pub stiffness: f64,
		pub damping: f64,
		/// About how long it takes to settle, in milliseconds.
		pub settles_ms: u32,
	}

	/// Registration returns, a button press.
	pub const SNAP: Spring = Spring {
		stiffness: 1537.0,
		damping: 78.4,
		settles_ms: 120,
	};
	/// Recoil and sway.
	pub const ARM: Spring = Spring {
		stiffness: 683.0,
		damping: 44.4,
		settles_ms: 180,
	};
	/// A sheet's release, a tag's rise.
	pub const SETTLE: Spring = Spring {
		stiffness: 587.0,
		damping: 36.4,
		settles_ms: 220,
	};
	/// Screen transitions.
	pub const CARRY: Spring = Spring {
		stiffness: 216.0,
		damping: 29.4,
		settles_ms: 320,
	};

	/// One held frame on twos: 12 prints a second.
	pub const ON_TWOS_MS: u32 = 83;
	/// A slip or sheet rising: four held steps.
	pub const RISE_STEPS: u32 = 4;
	pub const RISE_MS: u32 = 333;
	/// A stepped exit: three held steps.
	pub const EXIT_STEPS: u32 = 3;
	pub const EXIT_MS: u32 = 250;
	/// How far a slip rises as it enters, in points.
	pub const RISE_PT: u32 = 8;
	/// A screen transition.
	pub const TRANSITION_MS: u32 = 280;
	/// The longest a count-up runs.
	pub const COUNT_UP_MS: u32 = 800;
}

/// The four accents of each calling's colourways, as the Flutter game's
/// `classColourways` lists them. The 3D reapers still burn these; moving
/// them onto [`light`] is the look's own change (the bible's appendix A,
/// item 6).
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

/// The colour `genome`'s 3D light burns: its colourway's accent, as its
/// omen leaves it. The interface prints in [`light`] instead.
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
		assert_eq!(PAPER.hex(), "#ece3cf");
		assert_eq!(PAPER.hex_alpha(0.18), "#ece3cf2e");
	}

	#[test]
	fn seeds_follow_the_text() {
		assert_eq!(seed_for("Hunt"), seed_for("Hunt"));
		assert_ne!(seed_for("Hunt"), seed_for("Vault"));
	}

	#[test]
	fn the_inks_are_the_bibles() {
		let inks = [
			(PAPER, "#ece3cf"),
			(ASH, "#a39c93"),
			(RAISED, "#454042"),
			(VIGIL, "#342f33"),
			(SCRIM, "#2b272c"),
			(KEY, "#16121a"),
			(WOUND, "#b92a20"),
		];
		for (ink, hex) in inks {
			assert_eq!(ink.hex(), hex);
		}
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
	fn the_classes_are_their_tokens() {
		let pairs = [
			(class::VIGIL_BG, format!("bg-[{}]", VIGIL.hex())),
			(class::RAISED_BG, format!("bg-[{}]", RAISED.hex())),
			(class::PAPER_BG, format!("bg-[{}]", PAPER.hex())),
			(class::KEY_BG, format!("bg-[{}]", KEY.hex())),
			(class::SCRIM_BG, format!("bg-[{}]", SCRIM.hex())),
			(class::PAPER, format!("text-[{}]", PAPER.hex())),
			(class::ASH, format!("text-[{}]", ASH.hex())),
			(class::KEY, format!("text-[{}]", KEY.hex())),
			(class::WOUND, format!("text-[{}]", WOUND.hex())),
			(
				class::HAIRLINE_BORDER,
				format!("border-[{}]", PAPER.hex_alpha(0.18)),
			),
			(
				class::HAIRLINE_BG,
				format!("bg-[{}]", PAPER.hex_alpha(0.18)),
			),
		];
		for (class, expected) in pairs {
			assert_eq!(class, expected);
		}
	}

	/// Test C (§15.4): every pair the bible allows holds its threshold, 4.5
	/// for words and 3.0 for Display at s5 and up and for marks. The ratios
	/// are the bible's own table (§15.1), so a token that moves without the
	/// table fails here.
	#[test]
	fn every_allowed_pair_holds_its_contrast() {
		const WORDS: f64 = 4.5;
		const LARGE: f64 = 3.0;
		let allowed: [(&str, Rgb, Rgb, f64, f64); 25] = [
			("paper on vigil", PAPER, VIGIL, 10.28, WORDS),
			("paper on raised", PAPER, RAISED, 7.97, WORDS),
			("paper on scrim", PAPER, SCRIM, 11.51, WORDS),
			("paper on key", PAPER, KEY, 14.49, WORDS),
			("key on paper", KEY, PAPER, 14.49, WORDS),
			("ash on vigil", ASH, VIGIL, 4.83, WORDS),
			("ash on scrim", ASH, SCRIM, 5.41, WORDS),
			("ash on key", ASH, KEY, 6.81, WORDS),
			("ash on raised, s5 and up", ASH, RAISED, 3.75, LARGE),
			("wound on paper", WOUND, PAPER, 4.82, WORDS),
			("key on chaff", KEY, CHAFF, 12.06, WORDS),
			("key on lantern", KEY, LANTERN, 6.37, WORDS),
			("key on verdigris", KEY, VERDIGRIS, 7.84, WORDS),
			("key on cobalt", KEY, COBALT, 4.78, WORDS),
			("key on wisp", KEY, WISP, 14.40, WORDS),
			("key on mauve", KEY, MAUVE, 9.95, WORDS),
			("chaff on vigil, s5 and up", CHAFF, VIGIL, 8.56, LARGE),
			("lantern on vigil, s5 and up", LANTERN, VIGIL, 4.52, LARGE),
			(
				"verdigris on vigil, s5 and up",
				VERDIGRIS,
				VIGIL,
				5.56,
				LARGE,
			),
			("cobalt on vigil, s5 and up", COBALT, VIGIL, 3.39, LARGE),
			("wisp on vigil, s5 and up", WISP, VIGIL, 10.22, LARGE),
			("mauve on vigil, s5 and up", MAUVE, VIGIL, 7.06, LARGE),
			("cobalt on scrim, s5 and up", COBALT, SCRIM, 3.79, LARGE),
			("raised rule on paper, a mark", RAISED, PAPER, 7.97, LARGE),
			("ash mark on vigil", ASH, VIGIL, 4.83, LARGE),
		];
		for (pair, foreground, ground, table, threshold) in allowed {
			let ratio = foreground.contrast(ground);
			assert!(
				(ratio - table).abs() < 0.01,
				"{pair}: {ratio:.2}, the table says {table}"
			);
			assert!(ratio >= threshold, "{pair}: {ratio:.2} under {threshold}");
		}
	}

	/// The pairs the bible forbids really do fail, so the rules that keep
	/// them apart (no wound words on dark, no ash on paper, no light on paper
	/// without key) are needed.
	#[test]
	fn the_forbidden_pairs_fail() {
		for ground in [VIGIL, RAISED, SCRIM, KEY] {
			assert!(WOUND.contrast(ground) < 4.5, "wound on {ground:?}");
		}
		assert!(ASH.contrast(PAPER) < 3.0);
		for reaper_class in ReaperClass::ALL {
			assert!(
				light(reaper_class).contrast(PAPER) < 4.5,
				"{reaper_class:?}"
			);
		}
	}

	/// Every light holds words on the key ground the screens not yet on the
	/// vigil page still use, so the interface light can be any class's.
	#[test]
	fn every_light_reads_on_key() {
		for reaper_class in ReaperClass::ALL {
			let ratio = light(reaper_class).contrast(KEY);
			assert!(ratio >= 4.5, "{reaper_class:?}: {ratio:.2}");
		}
	}

	/// Part of test L (§15.4): the lights keep at least 6.5 L* apart, so no
	/// two depend on hue alone.
	#[test]
	fn the_lights_keep_apart_in_lightness() {
		fn lightness(colour: Rgb) -> f64 {
			let y = colour.luminance();
			let f = if y > 216.0 / 24389.0 {
				y.cbrt()
			} else {
				(24389.0 / 27.0 * y + 16.0) / 116.0
			};
			116.0 * f - 16.0
		}
		let mut values: Vec<f64> = ReaperClass::ALL
			.into_iter()
			.map(|class| lightness(light(class)))
			.collect();
		values.sort_by(f64::total_cmp);
		for pair in values.windows(2) {
			assert!(pair[1] - pair[0] >= 6.45, "{pair:?}");
		}
	}

	#[test]
	fn the_scale_climbs_by_about_a_quarter() {
		for pair in Step::ALL.windows(2) {
			let ratio = f64::from(pair[1].size()) / f64::from(pair[0].size());
			assert!((1.05..=1.40).contains(&ratio), "{pair:?}: {ratio}");
		}
		assert_eq!(Step::S1.size(), 11, "nothing on the page is under 11 pt");
		assert_eq!(Step::SPOT_MINIMUM.size(), 22);
	}

	#[test]
	fn a_long_name_shrinks_to_two_lines_and_no_further() {
		// The proof's name fits its 286 pt box in two lines at its step.
		assert_eq!(fit_display("Umbra of the Slow Water", 286.0, Step::S7), 40);
		// A narrower box shrinks it a point at a time.
		let narrow = fit_display("Umbra of the Slow Water", 220.0, Step::S7);
		assert!((22..40).contains(&narrow), "{narrow}");
		assert!(fit_display("Umbra of the Slow Water", 220.0, Step::S7) >= Step::S5.size());
		// Nothing is set under s5, however little room there is.
		assert_eq!(fit_display("Umbra of the Slow Water", 40.0, Step::S7), 22);
		assert_eq!(fit_display("IV", 70.0, Step::S7), 40);
	}

	#[test]
	fn every_target_is_at_least_forty_four() {
		for target in [
			space::TARGET,
			space::PRIMARY,
			space::SECONDARY,
			space::TAB_HEIGHT,
		] {
			assert!(target >= 44);
		}
	}
}
