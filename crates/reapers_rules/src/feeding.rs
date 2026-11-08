//! How well a reaper fed on a husk it absorbed.

use crate::named::named_enum;
use crate::seeded_random::SeededRandom;
use crate::seeded_random::mix_seed;

/// How well a reaper fed on a husk it absorbed.
///
/// Two reapers who cut the same husk equally well do not get exactly the
/// same out of it. The keeper of the books rolls this once, when the husk is
/// absorbed, from a seed the player never sees or chooses: the server's own
/// for an online player. It scales the experience the husk gives, and
/// nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feeding {
	/// There was less in it than it looked.
	Meagre,

	/// What the husk was worth, no more and no less.
	Fair,

	/// More than expected.
	Rich,

	/// Once in a while a husk gives twice what it should.
	Feast,
}

named_enum!(Feeding {
	Meagre => "meagre",
	Fair => "fair",
	Rich => "rich",
	Feast => "feast",
});

impl Feeding {
	/// Draws a feeding from `seed`.
	pub fn roll(seed: i64) -> Self {
		let weights = Self::ALL.map(|feeding| f64::from(feeding.weight()));

		Self::ALL[SeededRandom::new(mix_seed(&[seed, 0xFEED])).pick_weighted(&weights)]
	}

	/// Multiplier on the experience the husk gives.
	pub const fn share(self) -> f64 {
		match self {
			Self::Meagre => 0.85,
			Self::Fair => 1.0,
			Self::Rich => 1.25,
			Self::Feast => 2.0,
		}
	}

	/// How often this is drawn, against the other weights.
	pub const fn weight(self) -> u32 {
		match self {
			Self::Meagre => 25,
			Self::Fair => 50,
			Self::Rich => 20,
			Self::Feast => 5,
		}
	}

	/// The experience an average feeding gives, as a share of a fair one.
	/// A little above one: the roll is a small gift on average, never a tax.
	pub fn expected_share() -> f64 {
		let mut total = 0.0;
		let mut weights = 0;

		for feeding in Self::ALL {
			total += feeding.share() * f64::from(feeding.weight());
			weights += feeding.weight();
		}

		total / f64::from(weights)
	}
}
