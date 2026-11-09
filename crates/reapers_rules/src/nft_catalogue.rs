//! The collections whose dead can be hunted, and the NFTs in them.

use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::error::FormatError;
use crate::hunt::sigil::SigilShape;
use crate::named::named_enum;
use crate::nft_look::NftLook;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::hash_text;
use crate::vessel::Vessel;

/// How scarce an NFT is inside its collection, and so how hard its husk
/// fights, how hard it is to draw out, and how much it is worth to absorb.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Rarity {
	Common,
	Uncommon,
	Rare,
	Epic,
}

named_enum!(Rarity {
	Common => "common",
	Uncommon => "uncommon",
	Rare => "rare",
	Epic => "epic",
});

impl Rarity {
	/// The ward a husk of this rarity starts a hunt with.
	pub const fn ward(self) -> i64 {
		match self {
			Self::Common => 280,
			Self::Uncommon => 420,
			Self::Rare => 760,
			Self::Epic => 2800,
		}
	}

	/// Experience for absorbing it with a clean reap.
	pub const fn xp(self) -> i64 {
		match self {
			Self::Common => 40,
			Self::Uncommon => 70,
			Self::Rare => 120,
			Self::Epic => 210,
		}
	}

	/// Anima for absorbing it with a clean reap.
	pub const fn anima(self) -> i64 {
		match self {
			Self::Common => 16,
			Self::Uncommon => 28,
			Self::Rare => 48,
			Self::Epic => 85,
		}
	}

	/// The stand-in market price, in lamports (a billionth of one SOL).
	pub const fn lamports(self) -> i64 {
		match self {
			Self::Common => 20_000_000,
			Self::Uncommon => 45_000_000,
			Self::Rare => 110_000_000,
			Self::Epic => 260_000_000,
		}
	}

	/// How hard a husk of this rarity's sigil is to trace.
	pub const fn sigil(self) -> SigilShape {
		match self {
			Self::Common => SigilShape::new(3, 90.0, 8.0, 0.6, 0, None),
			Self::Uncommon => SigilShape::new(4, 110.0, 7.0, 0.7, 1, None),
			Self::Rare => SigilShape::new(5, 130.0, 6.0, 0.8, 2, Some(6.0)),
			Self::Epic => SigilShape::new(6, 160.0, 5.5, 0.9, 3, Some(5.0)),
		}
	}
}

/// A collection whose NFTs can be released into the veil and hunted.
///
/// These are real collections from Solana's first wave, chosen because each
/// has one clear creature and has long since gone quiet. They are
/// simulated: the game owns no NFT of theirs, copies none of their art and
/// has no agreement with their holders. Each is given a creature we
/// modelled ourselves. See docs/nfts.md and
/// docs/research/solana-collections.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum NftCollection {
	DegenApes,
	Thugbirdz,
	PeskyPenguins,
	Sollamas,
	Solarians,
}

named_enum!(NftCollection {
	DegenApes => "degenApes",
	Thugbirdz => "thugbirdz",
	PeskyPenguins => "peskyPenguins",
	Sollamas => "sollamas",
	Solarians => "solarians",
});

const STANDARD_RARITIES: [Rarity; 8] = [
	Rarity::Common,
	Rarity::Common,
	Rarity::Common,
	Rarity::Common,
	Rarity::Uncommon,
	Rarity::Uncommon,
	Rarity::Rare,
	Rarity::Epic,
];

impl NftCollection {
	/// How many absorbed items complete the collection's set.
	pub const SET_SIZE: usize = 3;

	/// The collection's name, as its creators would have published it.
	pub const fn title(self) -> &'static str {
		match self {
			Self::DegenApes => "Degenerate Ape Academy",
			Self::Thugbirdz => "Thugbirdz",
			Self::PeskyPenguins => "Pesky Penguins",
			Self::Sollamas => "Sollamas",
			Self::Solarians => "Solarians",
		}
	}

	/// The body every husk from this collection wears.
	pub const fn vessel(self) -> Vessel {
		match self {
			Self::DegenApes => Vessel::Ape,
			Self::Thugbirdz => Vessel::Bird,
			Self::PeskyPenguins => Vessel::Penguin,
			Self::Sollamas => Vessel::Llama,
			Self::Solarians => Vessel::Robot,
		}
	}

	/// The rarity of each numbered item, in serial order.
	pub const fn rarities(self) -> &'static [Rarity] {
		&STANDARD_RARITIES
	}
}

/// One NFT a player can own, release and hunt. On the wire it is its
/// [`id`](Self::id), such as `thugbirdz:3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Nft {
	pub collection: NftCollection,

	/// The item's number inside its collection, starting at 1.
	pub serial: usize,
}

impl Nft {
	pub const fn new(collection: NftCollection, serial: usize) -> Self {
		Self { collection, serial }
	}

	/// Reads an NFT back from its `id`, such as `thugbirdz:3`.
	pub fn parse(id: &str) -> Result<Self, FormatError> {
		let parts: Vec<&str> = id.split(':').collect();
		let serial = match parts[..] {
			[_, serial] => crate::parse::parse_int(serial),
			_ => None,
		};

		let Some(serial) = serial else {
			return Err(FormatError::with_input("Not an NFT id", id));
		};

		let collection = NftCollection::from_name(parts[0])
			.ok_or_else(|| FormatError::with_input("No such collection", id))?;

		if serial < 1 || serial > collection.rarities().len() as i64 {
			return Err(FormatError::with_input(
				format!("No such item in {}", collection.title()),
				id,
			));
		}

		Ok(Self::new(collection, serial as usize))
	}

	pub fn id(&self) -> String {
		format!("{}:{}", self.collection.name(), self.serial)
	}

	pub fn rarity(&self) -> Rarity {
		self.collection.rarities()[self.serial.saturating_sub(1)]
	}

	/// The name shown on a market listing, such as "Thugbirdz #3".
	pub fn title(&self) -> String {
		format!("{} #{}", self.collection.title(), self.serial)
	}

	/// The file stem of the item's portrait.
	pub fn image_key(&self) -> String {
		format!("{}_{}", self.collection.name(), self.serial)
	}

	/// How this NFT dresses its collection's creature.
	pub fn look(&self) -> NftLook {
		NftLook::of(self)
	}

	/// A stand-in for the item's on-chain mint address.
	///
	/// It has the shape of a Solana address so the interface can be laid out
	/// honestly, and it belongs to no real account.
	pub fn mock_mint(&self) -> String {
		const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
		let mut random = SeededRandom::new(hash_text(&format!("mint:{}", self.id())));

		(0..44)
			.map(|_| char::from(ALPHABET[random.next_index(ALPHABET.len())]))
			.collect()
	}
}

impl fmt::Display for Nft {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "Nft({})", self.id())
	}
}

impl Serialize for Nft {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.id())
	}
}

impl<'de> Deserialize<'de> for Nft {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let id = String::deserialize(deserializer)?;

		Self::parse(&id).map_err(serde::de::Error::custom)
	}
}

/// Every NFT the stand-in market lists.
pub fn market_listings() -> Vec<Nft> {
	NftCollection::ALL
		.into_iter()
		.flat_map(|collection| {
			(1..=collection.rarities().len()).map(move |serial| Nft::new(collection, serial))
		})
		.collect()
}

/// Lamports in one SOL.
pub const LAMPORTS_PER_SOL: i64 = 1_000_000_000;
