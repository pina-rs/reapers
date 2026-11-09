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
	orb_aim: f64,
	resolve: i64,
	lapses: i64,
	pace: f64,
	fight_length: f64,
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
					window: 2.0,
					orb_aim: 2.0,
					resolve: 3,
					lapses: 2,
					pace: 0.6,
					fight_length: 1.5,
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
					orb_aim: 1.0,
					resolve: 0,
					lapses: 0,
					pace: 1.0,
					fight_length: 1.0,
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
					orb_aim: 0.8,
					resolve: 0,
					lapses: 0,
					pace: 1.15,
					fight_length: 0.5,
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

	/// How much wider or narrower the seal cone and the gun's aim assist
	/// are: how far off a seal or the body a shot may be and still hit it.
	pub const fn reach(self) -> f64 {
		self.tuning().reach
	}

	/// How much faster the phone may move during a shriek before it counts
	/// as a flinch.
	pub const fn flinch(self) -> f64 {
		self.tuning().flinch
	}

	/// How much longer or shorter a broken husk stays down.
	pub const fn window(self) -> f64 {
		self.tuning().window
	}

	/// How much farther off the husk an orb may be thrown and still strike
	/// it.
	pub const fn orb_aim(self) -> f64 {
		self.tuning().orb_aim
	}

	/// Flinches the reaper can take on top of their own.
	pub const fn resolve(self) -> i64 {
		self.tuning().resolve
	}

	/// Broken windows the husk lets pass on top of the usual.
	pub const fn lapses(self) -> i64 {
		self.tuning().lapses
	}

	/// How fast the husk gets about, as a share of its usual speed.
	pub const fn pace(self) -> f64 {
		self.tuning().pace
	}

	/// How much longer or shorter the fight may run before the husk
	/// escapes.
	pub const fn fight_length(self) -> f64 {
		self.tuning().fight_length
	}

	/// How much longer a light of the sigil that has caught waits for the
	/// next.
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
