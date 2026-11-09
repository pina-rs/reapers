//! The six callings a reaper can be born to, each with the weapon it
//! carries, drawn in the light that calling burns with.

use std::time::Duration;

use leptoskit::native::Role;
use leptoskit::prelude::*;

use super::art::colour;
use super::ui::ASH;
use super::ui::BONE;
use super::ui::COLUMN;
use super::ui::HAIRLINE;
use super::ui::label;
use super::ui::paragraph;
use super::ui::prose;
use super::ui::section;
use super::ui::subtitle;
use super::ui::title;
use crate::l10n::L10n;
use crate::l10n::use_l10n;

/// A reaper's calling, as the rules name them (`reapers_rules::ReaperClass`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calling {
	Harvester,
	Ferryman,
	Weigher,
	Chooser,
	Hollow,
	Mourner,
}

impl Calling {
	/// Every calling, in the rules' order.
	pub const ALL: [Self; 6] = [
		Self::Harvester,
		Self::Ferryman,
		Self::Weigher,
		Self::Chooser,
		Self::Hollow,
		Self::Mourner,
	];

	/// The light the calling burns with: the first of its colourways, as the
	/// game's `reaper_light` reads it.
	pub const fn light(self) -> &'static str {
		match self {
			Self::Harvester => "#f3ebd8",
			Self::Ferryman => "#ffb347",
			Self::Weigher => "#e8f4ff",
			Self::Chooser => "#ff3b2f",
			Self::Hollow => "#ff7a1a",
			Self::Mourner => "#ddebff",
		}
	}

	/// Its name, its weapon, and what its gift does.
	fn words(self, words: &L10n) -> (&'static str, &'static str, &'static str) {
		match self {
			Self::Harvester => {
				(
					words.harvester_name(),
					words.harvester_weapon(),
					words.harvester_gift(),
				)
			}
			Self::Ferryman => {
				(
					words.ferryman_name(),
					words.ferryman_weapon(),
					words.ferryman_gift(),
				)
			}
			Self::Weigher => {
				(
					words.weigher_name(),
					words.weigher_weapon(),
					words.weigher_gift(),
				)
			}
			Self::Chooser => {
				(
					words.chooser_name(),
					words.chooser_weapon(),
					words.chooser_gift(),
				)
			}
			Self::Hollow => {
				(
					words.hollow_name(),
					words.hollow_weapon(),
					words.hollow_gift(),
				)
			}
			Self::Mourner => {
				(
					words.mourner_name(),
					words.mourner_weapon(),
					words.mourner_gift(),
				)
			}
		}
	}

	/// The weapon, drawn upright in a 64 by 128 box: its outline, and what
	/// is filled.
	const fn weapon(self) -> (&'static str, &'static str) {
		match self {
			// A scythe: the snath leaning, the blade sweeping back from it.
			Self::Harvester => {
				(
					"M36 122 L28 14 M30 40 L40 38 M28 14 C18 6 4 10 2 26 C12 18 22 18 29 24",
					"M28 14 C18 6 4 10 2 26 C12 18 22 18 29 24 Z",
				)
			}
			// A lantern on a crooked pole.
			Self::Ferryman => {
				(
					"M24 124 L24 14 C24 6 40 6 40 14 L40 24 M33 28 L47 28 M34 28 L34 46 L46 46 L46 28 \
				 M32 46 L48 46",
					"M40 33 C37 37 38 41 40 42 C42 41 43 37 40 33 Z",
				)
			}
			// A balance on a staff, its two pans hanging.
			Self::Weigher => {
				(
					"M32 124 L32 44 M8 52 L56 52 M32 44 L32 52 M10 52 L4 78 M10 52 L16 78 M54 52 L48 78 \
				 M54 52 L60 78",
					"M2 78 C4 86 16 86 18 78 Z M46 78 C48 86 60 86 62 78 Z M32 38 A4 4 0 1 0 32.01 38 Z",
				)
			}
			// A spear with a leaf blade.
			Self::Chooser => {
				(
					"M32 124 L32 34 M28 38 L36 38 M28 43 L36 43",
					"M32 4 C39 14 39 24 32 34 C25 24 25 14 32 4 Z",
				)
			}
			// A chain whip, its links snaking up from the grip.
			Self::Hollow => {
				(
					"M32 124 L32 98 M27 98 L37 98 M32 92 A3 3 0 1 0 32.01 92 M29 84 A3 3 0 1 0 29.01 84 \
				 M26 76 A3 3 0 1 0 26.01 76 M26 68 A3 3 0 1 0 26.01 68 M30 60 A3 3 0 1 0 30.01 60 \
				 M36 53 A3 3 0 1 0 36.01 53 M40 45 A3 3 0 1 0 40.01 45 M39 37 A3 3 0 1 0 39.01 37 \
				 M35 30 A3 3 0 1 0 35.01 30",
					"M33 26 L28 12 L38 22 Z",
				)
			}
			// A bell hanging from a crook.
			Self::Mourner => {
				(
					"M22 124 L22 30 C22 18 46 18 46 30 L46 36 M46 40 C40 40 38 48 38 56 L35 64 L57 64 \
				 L54 56 C54 48 52 40 46 40",
					"M46 67 A3 3 0 1 0 46.01 67 Z",
				)
			}
		}
	}

	/// The weapon, in the calling's light.
	fn drawing(self) -> Element {
		let (outline, filled) = self.weapon();
		let light = self.light();
		let stroke = Stroke::new(colour(light, 0xff), 1.6)
			.expect("a positive width")
			.round();
		Vector(ViewBox::new(0.0, 0.0, 64.0, 128.0).expect("a positive size"))
			.aria_hidden(true)
			.path(
				VectorPath::new(PathData::parse(filled).expect("the weapon's fill is valid"))
					.fill(colour(light, 0x55))
					.stroke(stroke.clone()),
			)
			.path(
				VectorPath::new(PathData::parse(outline).expect("the weapon's outline is valid"))
					.stroke(stroke),
			)
	}
}

/// One calling: its weapon standing in its light, and its entry.
fn entry(calling: Calling, words: &L10n, index: usize) -> Element {
	let (name, weapon, gift) = calling.words(words);
	let light = calling.light();
	let stand = View()
		.id(format!("calling-{}", index + 1))
		.class(format!(
			"relative w-[88px] h-[150px] shrink-0 items-center justify-center bg-[{light}0f]"
		))
		.child(calling.drawing().class("w-[56px] h-[112px]"));
	View()
		.role(Role::ListItem)
		.class(format!(
			"w-full flex-row gap-5 py-6 border-t border-[{HAIRLINE}]"
		))
		.child(stand)
		.child(
			View()
				.class("flex-1 gap-3 justify-center")
				.child(label(weapon, light))
				.child(subtitle(name))
				.child(paragraph(gift, 16, ASH)),
		)
}

/// The callings.
pub fn reapers() -> Element {
	let words = use_l10n();
	let mut callings = View()
		.role(Role::List)
		.class("w-full grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-10")
		.stagger(Duration::from_millis(90));
	for (index, calling) in Calling::ALL.into_iter().enumerate() {
		callings =
			callings.child(entry(calling, &words, index).motion(
				Motion::enter(Pose::new().y(24.0).opacity(0.0)).in_view(ViewRange::entry()),
			));
	}
	section("reapers", words.reapers_title()).child(
		View()
			.class(format!("{COLUMN} py-20 md:py-28 gap-12"))
			.child(title(words.reapers_title()).class("md:w-[70%]"))
			.child(
				View()
					.class("w-full gap-5 md:flex-row md:gap-10")
					.child(View().class("flex-1").child(prose(words.reapers_intro())))
					.child(View().class("flex-1").child(paragraph(
						words.reapers_growth(),
						17,
						ASH,
					))),
			)
			.child(callings)
			.child(View().class(format!("w-full h-[1px] bg-[{BONE}1a]"))),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_page_shows_every_calling_the_rules_have() {
		assert_eq!(Calling::ALL.len(), reapers_rules::ReaperClass::ALL.len());
		for (calling, class) in Calling::ALL.iter().zip(reapers_rules::ReaperClass::ALL) {
			assert_eq!(format!("{calling:?}"), format!("{class:?}"));
		}
	}

	#[test]
	fn every_weapon_draws() {
		for calling in Calling::ALL {
			let (outline, filled) = calling.weapon();
			assert!(PathData::parse(outline).is_ok(), "{calling:?}");
			assert!(PathData::parse(filled).is_ok(), "{calling:?}");
		}
	}
}
