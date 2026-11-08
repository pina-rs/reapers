//! Everything that makes one reaper look unlike another.

use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::FormatError;
use crate::named::named_enum;
use crate::reaper_class::ReaperClass;
use crate::reaper_names::REAPER_GIVEN_NAMES;
use crate::reaper_names::reaper_epithets;
use crate::seeded_random::SeededRandom;

/// A rare mark the shuffle sometimes leaves on a reaper.
///
/// An omen changes how a reaper is coloured and nothing about how it hunts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Omen {
	None,

	/// Drained to bone and ash.
	Ashen,

	/// Tarnished gold worked into every edge.
	Gilded,

	/// Near-black, with the accent burning at its brightest.
	Eclipsed,
}

named_enum!(Omen {
	None => "none",
	Ashen => "ashen",
	Gilded => "gilded",
	Eclipsed => "eclipsed",
});

impl Omen {
	/// How often the shuffle lands on this omen, out of 100.
	pub const fn weight(self) -> u32 {
		match self {
			Self::None => 80,
			Self::Ashen => 12,
			Self::Gilded => 6,
			Self::Eclipsed => 2,
		}
	}
}

/// What drifts around a reaper while it waits in the sanctum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Aura {
	Motes,
	Embers,
	Mist,
	Wisps,
}

named_enum!(Aura {
	Motes => "motes",
	Embers => "embers",
	Mist => "mist",
	Wisps => "wisps",
});

/// Everything that makes one reaper look unlike another.
///
/// A genome is drawn once, by [`ReaperGenome::shuffle`], and then stored as
/// its [`code`](Self::code). The code is the permanent record: it does not
/// depend on the seed or on the shuffle's tables staying the same, so a
/// reaper minted today looks the same after the tables grow. Two genomes are
/// equal when their codes are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReaperGenome {
	pub reaper_class: ReaperClass,

	/// Which head of the class's head family.
	pub head: u32,

	/// Which shoulders of the class's shoulder family.
	pub mantle: u32,

	/// Which hem.
	pub robe: u32,

	/// Which of the class's colourways.
	pub palette: u32,

	pub omen: Omen,

	pub aura: Aura,

	/// How tall the reaper stands, from 0 (shortest) to
	/// [`TRAIT_SCALE`](Self::TRAIT_SCALE).
	pub stature: u32,

	/// How drawn and narrow the reaper's body is.
	pub gauntness: u32,

	/// How large the reaper's one light burns.
	pub gaze: u32,

	/// How ragged the hem is.
	pub tatter: u32,

	/// Index into [`REAPER_GIVEN_NAMES`].
	pub given_name: u32,

	/// Index into the class's list in [`reaper_epithets`].
	pub epithet: u32,
}

/// Birth traits a studio may try on a genome. Unset fields keep the
/// genome's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct GenomeChanges {
	pub reaper_class: Option<ReaperClass>,
	pub head: Option<u32>,
	pub mantle: Option<u32>,
	pub robe: Option<u32>,
	pub palette: Option<u32>,
	pub omen: Option<Omen>,
	pub aura: Option<Aura>,
	pub stature: Option<u32>,
	pub gauntness: Option<u32>,
	pub gaze: Option<u32>,
	pub tatter: Option<u32>,
}

impl ReaperGenome {
	/// The reaper lent to someone who plays without taking the oath.
	pub const BORROWED: Self = Self {
		reaper_class: ReaperClass::Harvester,
		head: 0,
		mantle: 0,
		robe: 0,
		palette: 0,
		omen: Omen::None,
		aura: Aura::Mist,
		stature: 500,
		gauntness: 500,
		gaze: 400,
		tatter: 500,
		given_name: 0,
		epithet: 0,
	};
	const FIELD_COUNT: usize = 14;
	/// How many head shapes each class's head family holds.
	pub const HEAD_VARIANTS: u32 = 3;
	/// How many shoulder shapes each class's shoulder family holds.
	pub const MANTLE_VARIANTS: u32 = 3;
	/// How many colourways each class has.
	pub const PALETTE_VARIANTS: u32 = 3;
	/// How many robe hems every class shares.
	pub const ROBE_VARIANTS: u32 = 4;
	/// Continuous traits are stored as whole numbers from 0 to this value,
	/// so a code round-trips exactly.
	pub const TRAIT_SCALE: u32 = 1000;
	const VERSION: &'static str = "R1";

	/// Draws a reaper from `seed`. The same seed always draws the same
	/// reaper.
	pub fn shuffle(seed: i64) -> Self {
		let mut random = SeededRandom::new(seed);
		let reaper_class = random.pick(&ReaperClass::ALL);
		let mut next = |limit: u32| random.next_int(i64::from(limit)) as u32;
		let head = next(Self::HEAD_VARIANTS);
		let mantle = next(Self::MANTLE_VARIANTS);
		let robe = next(Self::ROBE_VARIANTS);
		let palette = next(Self::PALETTE_VARIANTS);
		let omen = Omen::ALL[random.pick_weighted(&Omen::ALL.map(|omen| f64::from(omen.weight())))];
		let aura = random.pick(&Aura::ALL);
		let mut next = |limit: usize| random.next_index(limit) as u32;
		let scale = Self::TRAIT_SCALE as usize + 1;

		Self {
			reaper_class,
			head,
			mantle,
			robe,
			palette,
			omen,
			aura,
			stature: next(scale),
			gauntness: next(scale),
			gaze: next(scale),
			tatter: next(scale),
			given_name: next(REAPER_GIVEN_NAMES.len()),
			epithet: next(reaper_epithets(reaper_class).len()),
		}
	}

	/// Reads a genome back from its `code`.
	///
	/// Fails when the code is malformed or names a part that does not exist,
	/// so a corrupted record is never drawn as a reaper.
	pub fn parse(code: &str) -> Result<Self, FormatError> {
		let fields: Vec<&str> = code.split('-').collect();

		if fields.len() != Self::FIELD_COUNT || fields[0] != Self::VERSION {
			return Err(FormatError::with_input("Not a reaper genome", code));
		}

		let reaper_class = ReaperClass::from_code(fields[1])?;
		let scale = Self::TRAIT_SCALE + 1;

		Ok(Self {
			reaper_class,
			head: field(fields[2], "h", Self::HEAD_VARIANTS)?,
			mantle: field(fields[3], "m", Self::MANTLE_VARIANTS)?,
			robe: field(fields[4], "r", Self::ROBE_VARIANTS)?,
			palette: field(fields[5], "p", Self::PALETTE_VARIANTS)?,
			omen: Omen::ALL[field(fields[6], "o", Omen::ALL.len() as u32)? as usize],
			aura: Aura::ALL[field(fields[7], "a", Aura::ALL.len() as u32)? as usize],
			stature: field(fields[8], "s", scale)?,
			gauntness: field(fields[9], "g", scale)?,
			gaze: field(fields[10], "z", scale)?,
			tatter: field(fields[11], "t", scale)?,
			given_name: field(fields[12], "n", REAPER_GIVEN_NAMES.len() as u32)?,
			epithet: field(fields[13], "e", reaper_epithets(reaper_class).len() as u32)?,
		})
	}

	/// The reaper's full name, such as "Vesper of the Last Furrow".
	pub fn name(&self) -> String {
		let given = REAPER_GIVEN_NAMES
			.get(self.given_name as usize)
			.copied()
			.unwrap_or_default();
		let epithet = reaper_epithets(self.reaper_class)
			.get(self.epithet as usize)
			.copied()
			.unwrap_or_default();

		format!("{given} {epithet}")
	}

	/// The permanent, compact record of this genome.
	pub fn code(&self) -> String {
		format!(
			"{}-{}-h{}-m{}-r{}-p{}-o{}-a{}-s{}-g{}-z{}-t{}-n{}-e{}",
			Self::VERSION,
			self.reaper_class.code(),
			self.head,
			self.mantle,
			self.robe,
			self.palette,
			self.omen.index(),
			self.aura.index(),
			self.stature,
			self.gauntness,
			self.gaze,
			self.tatter,
			self.given_name,
			self.epithet,
		)
	}

	/// Tries birth traits in the studio. A bound reaper's genome stays
	/// fixed. Parsing the result enforces the same bounds as a minted
	/// genome. A change of class starts the epithet over, since each class
	/// has its own.
	pub fn with(&self, changes: GenomeChanges) -> Result<Self, FormatError> {
		let epithet = match changes.reaper_class {
			Some(class) if class != self.reaper_class => 0,
			_ => self.epithet,
		};

		Self::parse(
			&Self {
				reaper_class: changes.reaper_class.unwrap_or(self.reaper_class),
				head: changes.head.unwrap_or(self.head),
				mantle: changes.mantle.unwrap_or(self.mantle),
				robe: changes.robe.unwrap_or(self.robe),
				palette: changes.palette.unwrap_or(self.palette),
				omen: changes.omen.unwrap_or(self.omen),
				aura: changes.aura.unwrap_or(self.aura),
				stature: changes.stature.unwrap_or(self.stature),
				gauntness: changes.gauntness.unwrap_or(self.gauntness),
				gaze: changes.gaze.unwrap_or(self.gaze),
				tatter: changes.tatter.unwrap_or(self.tatter),
				given_name: self.given_name,
				epithet,
			}
			.code(),
		)
	}

	/// A continuous trait as a fraction from 0 to 1.
	pub fn fraction(trait_value: u32) -> f64 {
		f64::from(trait_value) / f64::from(Self::TRAIT_SCALE)
	}
}

impl fmt::Display for ReaperGenome {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "ReaperGenome({})", self.code())
	}
}

impl Serialize for ReaperGenome {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.code())
	}
}

impl<'de> Deserialize<'de> for ReaperGenome {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let code = String::deserialize(deserializer)?;

		Self::parse(&code).map_err(serde::de::Error::custom)
	}
}

/// One `prefix`ed field of a genome code, below `limit`.
fn field(field: &str, prefix: &str, limit: u32) -> Result<u32, FormatError> {
	let Some(digits) = field.strip_prefix(prefix) else {
		return Err(FormatError::with_input(
			format!("Expected a \"{prefix}\" field"),
			field,
		));
	};

	match crate::dart::parse_int(digits) {
		Some(value) if value >= 0 && value < i64::from(limit) => Ok(value as u32),
		_ => Err(FormatError::with_input("Out of range", field)),
	}
}
