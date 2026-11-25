//! Second Ink: the paper and the inks every printed mark is made of, as the
//! art bible's palette (§3) names them.
//!
//! A mark is key on paper, and the second ink, the spot, is the reaper's own
//! light: one per class, whatever its colourway ([`light`]). Wound is a
//! third ink kept for loss and error alone.
//!
//! Only the hunt's printed marks take these so far: the rest of the
//! interface keeps [`crate::theme`] until it is reprinted.

use reapers_rules::ReaperClass;

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

#[cfg(test)]
mod tests {
	use super::*;

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
}
