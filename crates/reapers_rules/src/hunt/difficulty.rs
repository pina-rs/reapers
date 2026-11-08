//! How forgiving a hunt is.

use crate::named::named_enum;

/// How forgiving a hunt is.
///
/// [`Difficulty::Normal`] is the game. The others exist for testing:
/// [`Difficulty::Easy`] so that someone who has never hunted can bring a
/// husk down and see the whole of a hunt, and [`Difficulty::Hard`] to see
/// how it holds up when pushed. A hunt records the difficulty it was played
/// at, and the server takes anything but normal only outside production.
///
/// A difficulty changes how much a hunt asks of the player and nothing about
/// what it pays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Difficulty {
	Easy,
	#[default]
	Normal,
	Hard,
}

named_enum!(Difficulty {
	Easy => "easy",
	Normal => "normal",
	Hard => "hard",
});

/// What one difficulty changes.
struct Tuning {
	ward: f64,
	reach: f64,
	flinch: f64,
	window: f64,
	tolerance: f64,
	resolve: i64,
	reaps: i64,
	pace: f64,
	patience: f64,
	tell: f64,
	cut: f64,
}

impl Difficulty {
	const fn tuning(self) -> Tuning {
		match self {
			Self::Easy => {
				Tuning {
					ward: 0.35,
					reach: 1.75,
					flinch: 3.0,
					window: 2.5,
					tolerance: 2.0,
					resolve: 3,
					reaps: 2,
					pace: 0.6,
					patience: 2.0,
					tell: 1.6,
					cut: 0.6,
				}
			}
			Self::Normal => {
				Tuning {
					ward: 1.0,
					reach: 1.0,
					flinch: 1.0,
					window: 1.0,
					tolerance: 1.0,
					resolve: 0,
					reaps: 0,
					pace: 1.0,
					patience: 1.0,
					tell: 1.0,
					cut: 1.0,
				}
			}
			Self::Hard => {
				Tuning {
					ward: 1.3,
					reach: 0.85,
					flinch: 0.8,
					window: 0.8,
					tolerance: 0.8,
					resolve: 0,
					reaps: 0,
					pace: 1.15,
					patience: 1.0,
					tell: 0.85,
					cut: 1.15,
				}
			}
		}
	}

	/// The share of its ward a husk has.
	pub const fn ward(self) -> f64 {
		self.tuning().ward
	}

	/// How much wider or narrower the reach of a strike is: how far from a
	/// seal a tap may land and still strike it.
	pub const fn reach(self) -> f64 {
		self.tuning().reach
	}

	/// How much faster the phone may move during a shriek before it counts
	/// as a flinch.
	pub const fn flinch(self) -> f64 {
		self.tuning().flinch
	}

	/// How much longer or shorter the reap window is.
	pub const fn window(self) -> f64 {
		self.tuning().window
	}

	/// How much farther the cut may stray from the reap arc.
	pub const fn tolerance(self) -> f64 {
		self.tuning().tolerance
	}

	/// Flinches the reaper can take on top of their own.
	pub const fn resolve(self) -> i64 {
		self.tuning().resolve
	}

	/// Unfinished reaps the husk allows on top of the usual.
	pub const fn reaps(self) -> i64 {
		self.tuning().reaps
	}

	/// How fast the husk gets about, as a share of its usual speed.
	pub const fn pace(self) -> f64 {
		self.tuning().pace
	}

	/// How much longer the fight may run before the husk escapes.
	pub const fn patience(self) -> f64 {
		self.tuning().patience
	}

	/// How much longer or shorter a charging husk takes to reach the
	/// reaper: how long there is to see it coming.
	pub const fn tell(self) -> f64 {
		self.tuning().tell
	}

	/// How hard the phone has to be swung to turn a charge aside, as a share
	/// of the usual.
	pub const fn cut(self) -> f64 {
		self.tuning().cut
	}
}
