//! Authored clip IDs and timings shared by the art builder and both
//! renderers.

use crate::reaper_class::WeaponKind;

/// Authored clip IDs and timings shared by the art builder and both
/// renderers. These are presentation timings; they never award damage or
/// settle a hunt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ReaperMotion {
	Idle,
	Creep,
	Walk,
	Run,
	Sprint,
	Jump,
	Land,
	DodgeLeft,
	DodgeRight,
	Dash,
	DashLeft,
	DashRight,
	Laugh,
	Cry,
	Rage,
	Fear,
	Bow,
	Victory,
	Gigachad,
	Sigma,
	Shrug,
	AbsoluteCinema,
	TPose,
	AnimeCharge,
	Slash,
	Thrust,
	Parry,
	FinisherReap,
	FinisherImpale,
	FinisherSoulCrush,
	FinisherPalm,
	FinisherUppercut,
}

/// Studio filters; these do not add player progression or unlock rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum MotionGroup {
	Movement,
	Reaction,
	Meme,
	Combat,
	Finisher,
}

/// One clip's timing: length, group, and how it plays.
struct Clip {
	seconds: f64,
	group: MotionGroup,
	loops: bool,
	unarmed: bool,
	impact: Option<f64>,
}

const fn clip(seconds: f64, group: MotionGroup) -> Clip {
	Clip {
		seconds,
		group,
		loops: false,
		unarmed: false,
		impact: None,
	}
}

const fn looping(seconds: f64, group: MotionGroup) -> Clip {
	Clip {
		seconds,
		group,
		loops: true,
		unarmed: false,
		impact: None,
	}
}

const fn unarmed(seconds: f64, group: MotionGroup) -> Clip {
	Clip {
		seconds,
		group,
		loops: false,
		unarmed: true,
		impact: None,
	}
}

impl ReaperMotion {
	pub const ALL: [Self; 32] = [
		Self::Idle,
		Self::Creep,
		Self::Walk,
		Self::Run,
		Self::Sprint,
		Self::Jump,
		Self::Land,
		Self::DodgeLeft,
		Self::DodgeRight,
		Self::Dash,
		Self::DashLeft,
		Self::DashRight,
		Self::Laugh,
		Self::Cry,
		Self::Rage,
		Self::Fear,
		Self::Bow,
		Self::Victory,
		Self::Gigachad,
		Self::Sigma,
		Self::Shrug,
		Self::AbsoluteCinema,
		Self::TPose,
		Self::AnimeCharge,
		Self::Slash,
		Self::Thrust,
		Self::Parry,
		Self::FinisherReap,
		Self::FinisherImpale,
		Self::FinisherSoulCrush,
		Self::FinisherPalm,
		Self::FinisherUppercut,
	];

	#[allow(clippy::match_same_arms, reason = "one arm for each authored clip")]
	const fn clip(self) -> Clip {
		use MotionGroup::Combat;
		use MotionGroup::Finisher;
		use MotionGroup::Meme;
		use MotionGroup::Movement;
		use MotionGroup::Reaction;

		match self {
			Self::Idle => looping(4.0, Movement),
			Self::Creep => looping(2.4, Movement),
			Self::Walk => looping(1.2, Movement),
			Self::Run => looping(0.8, Movement),
			Self::Sprint => looping(0.6, Movement),
			Self::Jump => clip(1.4, Movement),
			Self::Land => clip(0.7, Movement),
			Self::DodgeLeft | Self::DodgeRight | Self::Dash | Self::DashLeft | Self::DashRight => {
				clip(0.8, Movement)
			}
			Self::Laugh => unarmed(2.4, Reaction),
			Self::Cry => unarmed(3.2, Reaction),
			Self::Rage => unarmed(2.0, Reaction),
			Self::Fear => unarmed(1.8, Reaction),
			Self::Bow => clip(2.4, Reaction),
			Self::Victory => clip(2.8, Reaction),
			Self::Gigachad => unarmed(3.0, Meme),
			Self::Sigma => unarmed(2.4, Meme),
			Self::Shrug => unarmed(2.2, Meme),
			Self::AbsoluteCinema => unarmed(3.0, Meme),
			Self::TPose => unarmed(2.4, Meme),
			Self::AnimeCharge => unarmed(2.4, Combat),
			Self::Slash => {
				Clip {
					impact: Some(0.45),
					..clip(1.2, Combat)
				}
			}
			Self::Thrust => {
				Clip {
					impact: Some(0.45),
					..clip(1.0, Combat)
				}
			}
			Self::Parry => clip(1.2, Combat),
			Self::FinisherReap => {
				Clip {
					impact: Some(0.52),
					..clip(2.8, Finisher)
				}
			}
			Self::FinisherImpale => {
				Clip {
					impact: Some(0.46),
					..clip(2.6, Finisher)
				}
			}
			Self::FinisherSoulCrush => {
				Clip {
					impact: Some(0.6),
					..unarmed(3.0, Finisher)
				}
			}
			Self::FinisherPalm => {
				Clip {
					impact: Some(0.5),
					..unarmed(2.2, Finisher)
				}
			}
			Self::FinisherUppercut => {
				Clip {
					impact: Some(0.48),
					..unarmed(2.4, Finisher)
				}
			}
		}
	}

	/// The clip's ID, as the art build names it.
	pub const fn name(self) -> &'static str {
		match self {
			Self::Idle => "idle",
			Self::Creep => "creep",
			Self::Walk => "walk",
			Self::Run => "run",
			Self::Sprint => "sprint",
			Self::Jump => "jump",
			Self::Land => "land",
			Self::DodgeLeft => "dodgeLeft",
			Self::DodgeRight => "dodgeRight",
			Self::Dash => "dash",
			Self::DashLeft => "dashLeft",
			Self::DashRight => "dashRight",
			Self::Laugh => "laugh",
			Self::Cry => "cry",
			Self::Rage => "rage",
			Self::Fear => "fear",
			Self::Bow => "bow",
			Self::Victory => "victory",
			Self::Gigachad => "gigachad",
			Self::Sigma => "sigma",
			Self::Shrug => "shrug",
			Self::AbsoluteCinema => "absoluteCinema",
			Self::TPose => "tPose",
			Self::AnimeCharge => "animeCharge",
			Self::Slash => "slash",
			Self::Thrust => "thrust",
			Self::Parry => "parry",
			Self::FinisherReap => "finisherReap",
			Self::FinisherImpale => "finisherImpale",
			Self::FinisherSoulCrush => "finisherSoulCrush",
			Self::FinisherPalm => "finisherPalm",
			Self::FinisherUppercut => "finisherUppercut",
		}
	}

	pub const fn seconds(self) -> f64 {
		self.clip().seconds
	}

	pub const fn group(self) -> MotionGroup {
		self.clip().group
	}

	pub const fn loops(self) -> bool {
		self.clip().loops
	}

	pub const fn unarmed(self) -> bool {
		self.clip().unarmed
	}

	/// Normalized contact point for a future synchronized enemy reaction.
	pub const fn impact(self) -> Option<f64> {
		self.clip().impact
	}

	/// Whether this clip can be shown with `weapon` in hand. `None` means
	/// empty hands, so weapon attacks cannot be previewed.
	pub fn supports(self, weapon: Option<WeaponKind>) -> bool {
		match self {
			Self::FinisherImpale => weapon == Some(WeaponKind::Spear),
			Self::FinisherReap => {
				weapon == Some(WeaponKind::Scythe) || weapon == Some(WeaponKind::ChainWhip)
			}
			Self::Slash | Self::Thrust | Self::Parry => weapon.is_some(),
			_ => true,
		}
	}
}
