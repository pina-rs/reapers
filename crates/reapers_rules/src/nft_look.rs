//! How one NFT dresses its collection's creature.

use serde::Deserialize;
use serde::Serialize;

use crate::nft_catalogue::Nft;
use crate::nft_catalogue::NftCollection;
use crate::nft_catalogue::Rarity;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::hash_text;

/// How one NFT dresses its collection's creature.
///
/// Every NFT of a collection wears the same body. What tells one from
/// another is this: its colours and the one thing it wears. The colours are
/// `0xRRGGBB`.
///
/// These looks are ours. They follow the spirit of a collection's traits (a
/// cap here, a gold coat there) without reproducing any NFT's artwork.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NftLook {
	/// The main colour of the creature: fur, feathers, skin.
	pub body: u32,

	/// Its second colour: a belly, a beak, a face.
	pub trim: u32,

	/// Small hard things: claws, teeth, and whatever it wears.
	pub detail: u32,

	/// The light in its eyes, and of its seals.
	pub accent: u32,

	/// The flat colour behind it in its portrait.
	pub backdrop: u32,

	/// Which thing it wears, from 1, or 0 for nothing.
	#[serde(rename = "trait")]
	pub trait_index: i64,
}

impl NftLook {
	const GILT: u32 = 0x00C9_A227;
	/// How many different things a creature can wear. A model carries one
	/// node for each, named `trait_1` up to `trait_<TRAIT_COUNT>`.
	pub const TRAIT_COUNT: i64 = 2;

	/// The look of `nft`, which never changes: it is drawn from the NFT's
	/// own identity.
	pub fn of(nft: &Nft) -> Self {
		let palette = palette(nft.collection);
		let mut random = SeededRandom::new(hash_text(&format!("look:{}", nft.id())));
		let gilded = nft.rarity() == Rarity::Epic;

		// The one epic of each collection is always the gilded one.
		let body = if gilded {
			Self::GILT
		} else {
			random.pick(palette.bodies)
		};
		let trim = if gilded {
			0x00F4_E3B0
		} else {
			random.pick(palette.trims)
		};
		let detail = random.pick(palette.details);
		let accent = random.pick(palette.accents);
		let backdrop = random.pick(palette.backdrops);
		// Commons mostly go bare; the rarer an NFT, the surer it wears
		// something.
		let trait_index = if random.next_chance(0.3 + 0.2 * nft.rarity().index() as f64) {
			1 + random.next_int(Self::TRAIT_COUNT)
		} else {
			0
		};

		Self {
			body,
			trim,
			detail,
			accent,
			backdrop,
			trait_index,
		}
	}
}

/// The colours one collection's NFTs are drawn from.
struct Palette {
	bodies: &'static [u32],
	trims: &'static [u32],
	details: &'static [u32],
	accents: &'static [u32],
	backdrops: &'static [u32],
}

#[allow(clippy::unreadable_literal, reason = "colours read as #RRGGBB")]
const fn palette(collection: NftCollection) -> Palette {
	match collection {
		// Fur in earth colours and a few unnatural ones, on flat pastel
		// grounds.
		NftCollection::DegenApes => {
			Palette {
				bodies: &[0x5A4032, 0x3B3F46, 0x7A5B3E, 0x4F6B4A, 0x2E5E6E, 0x8A3B3B],
				trims: &[0xC9A07A, 0xB7A99A, 0xD9B38C, 0x9DB59A],
				details: &[0xE8E1D0, 0x1C1C1F, 0xD9B441, 0xB23A3A],
				accents: &[0xFF4B3A, 0x57E0FF, 0xFFD23F, 0xB8FF5A],
				backdrops: &[0x8FB8C9, 0xD9B38C, 0xA7C4A0, 0xC9A0C0, 0xE0C56B],
			}
		}
		// Street birds in any colour there is, on flat pastel grounds.
		NftCollection::Thugbirdz => {
			Palette {
				bodies: &[0x2E6FD9, 0xD9452E, 0x2EAF6E, 0x8A4FD9, 0xE0A02E, 0x3A3A44],
				trims: &[0xF2C14E, 0xF28C28, 0xE8E1D0, 0xF25C7A],
				details: &[0x1C1C1F, 0xD9B441, 0xE8E1D0, 0xB23A3A],
				accents: &[0xFFE14B, 0xFF5A8A, 0x6BFFD9, 0xFF7A2E],
				backdrops: &[0xF2A0B5, 0xA0D8F2, 0xF2E08A, 0xB5F2A0, 0xC8A0F2],
			}
		}
		// Penguins in the usual black, and in colours no penguin comes in.
		NftCollection::PeskyPenguins => {
			Palette {
				bodies: &[0x23262E, 0x2D4A7A, 0x2F6B62, 0xC9722E, 0x5B3F7A, 0x6B6F78],
				trims: &[0xF2EFE6, 0xE6EEF2, 0xF2E6D8, 0xDDE8D8],
				details: &[0xF2A03B, 0xE0563B, 0xD9B441, 0xE8E1D0],
				accents: &[0x7AE0FF, 0xFFE07A, 0xFF8A7A, 0xB0FF8A],
				backdrops: &[0x6FA8D9, 0x5F8F92, 0x8C6FD9, 0x9A7FBF, 0x7FB0A0],
			}
		}
		// Woolly coats in wool colours, under a bright blanket.
		NftCollection::Sollamas => {
			Palette {
				bodies: &[0xE6D8BF, 0xB98A5A, 0x6E5444, 0xF2EEE6, 0x8A8F96, 0x7DA36E],
				trims: &[0xD9452E, 0x2E8FD9, 0xE0A02E, 0x8A4FD9, 0x2EAF6E],
				details: &[0x3A2E28, 0xD9B441, 0xE8E1D0, 0x1C1C1F],
				accents: &[0xFFB03B, 0x7AE0FF, 0xFF6A8A, 0xC8FF6A],
				backdrops: &[0xE0B87A, 0xBFD8E6, 0x9CC48A, 0x5A6A9A, 0xD9C7E0],
			}
		}
		// Plate metal in workshop colours, with one bright part.
		NftCollection::Solarians => {
			Palette {
				bodies: &[0x8A9099, 0x5F6B78, 0xA67C52, 0x4E6E73, 0x7A5F8A, 0x9A4A3A],
				trims: &[0x33383F, 0x4A4038, 0x2A3A44, 0x443A4A],
				details: &[0xD9B441, 0xC0C6D0, 0xE0563B, 0x3BC0E0],
				accents: &[0x3BFFE0, 0xFF5A3B, 0xFFE83B, 0x9A6BFF],
				backdrops: &[0x2A2F38, 0x3A2F2A, 0x24343A, 0x33293A, 0x2F3A2A],
			}
		}
	}
}
