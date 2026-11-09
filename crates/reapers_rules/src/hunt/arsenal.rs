//! What a weapon slot holds, and so how its FIRE press is played.

/// What a weapon slot holds, and so how its FIRE press is played.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Archetype {
	/// A gun: one shot per press, from a magazine that has to be reloaded.
	Gun,

	/// A blade: a press dives in and each press after stabs.
	Blade,

	/// The reaper's class weapon: a press dives in and swings once.
	Heavy,

	/// The reaper's art: a press throws it at the crosshair.
	Art,
}

/// The four slots a reaper carries, in the order the weapon rail shows them.
///
/// Every slot is fired with the one FIRE button. Which slot is selected is
/// part of each tick's input, and a newly selected weapon takes
/// [`SWITCH_TICKS`](Arsenal::SWITCH_TICKS) to draw before it can be fired.
pub struct Arsenal;

impl Arsenal {
	pub const ART_SLOT: usize = 3;
	pub const BLADE_SLOT: usize = 1;
	/// A husk this close needs no dive to be stabbed or swung at: it is in
	/// arm's reach, as it is mid-charge or stunned after a parry. In metres.
	pub const CLOSE_METRES: f64 = 1.8;
	pub const GUN_SLOT: usize = 0;
	pub const HEAVY_SLOT: usize = 2;
	/// How many slots a reaper carries.
	pub const SLOTS: usize = 4;
	/// Ticks to draw a newly selected weapon.
	pub const SWITCH_TICKS: i64 = 21;

	/// The archetype in `slot`, or `None` for one no reaper carries.
	pub const fn archetype_of(slot: usize) -> Option<Archetype> {
		match slot {
			Self::GUN_SLOT => Some(Archetype::Gun),
			Self::BLADE_SLOT => Some(Archetype::Blade),
			Self::HEAVY_SLOT => Some(Archetype::Heavy),
			Self::ART_SLOT => Some(Archetype::Art),
			_ => None,
		}
	}
}

/// The Grave pistol, the reaper's slot-one gun.
pub struct GunRules;

impl GunRules {
	/// FIRE pressed between these ticks of a reload, counted from its start,
	/// finishes it at once and makes the next magazine hit harder. Both ends
	/// count.
	pub const ACTIVE_FROM_TICK: i64 = 48;
	pub const ACTIVE_UNTIL_TICK: i64 = 60;
	/// A shot within this many degrees of the husk's body bends onto it, for
	/// a reaper of middling reach. See [`HuntStats::assist_radians`].
	pub const ASSIST_DEGREES: f64 = 3.5;
	/// How many degrees wider or narrower the bend is for each point of the
	/// class's reach above or below the middle.
	pub const ASSIST_DEGREES_PER_REACH: f64 = 0.25;
	/// The class reach the bend is [`ASSIST_DEGREES`](Self::ASSIST_DEGREES)
	/// at.
	pub const ASSIST_MIDDLE_REACH: i64 = 3;
	/// Every shot from a magazine loaded by an active reload hits this much
	/// harder.
	pub const BOOSTED_STRIKE: f64 = 1.25;
	/// The fewest ticks between one shot and the next.
	pub const FIRE_INTERVAL_TICKS: i64 = 18;
	/// FIRE pressed earlier than that jams the reload for this much longer.
	pub const JAM_TICKS: i64 = 30;
	/// How long a reload takes.
	pub const RELOAD_TICKS: i64 = 84;
	/// Rounds in a full magazine.
	pub const ROUNDS: i64 = 6;
	/// A shot does this share of a hit.
	pub const SHOT_STRIKE: f64 = 0.55;
}

/// The Bone knife, the reaper's slot-two blade.
pub struct BladeRules;

impl BladeRules {
	pub const CHARGE_TICKS: i64 = 180;
	/// Dives the knife holds, and how long one takes to come back.
	pub const DIVE_CHARGES: i64 = 3;
	/// The farthest a husk may be, in metres, for a press to dive at it.
	pub const DIVE_METRES: f64 = 4.5;
	/// A dive's three parts: going in, the stab window, coming back out.
	pub const IN_TICKS: i64 = 13;
	pub const OUT_TICKS: i64 = 18;
	/// How far off the husk's body, in degrees, the crosshair may be for the
	/// knife to reach it.
	pub const REACH_DEGREES: f64 = 2.0;
	/// The most stabs one dive lands, and the fewest ticks between two.
	pub const STABS_PER_DIVE: i64 = 3;
	pub const STAB_INTERVAL_TICKS: i64 = 10;
	/// A stab does this share of a hit.
	pub const STAB_STRIKE: f64 = 0.8;
	pub const WINDOW_TICKS: i64 = 30;
}

/// The reaper's class weapon, swung as the slot-three heavy.
pub struct HeavyRules;

impl HeavyRules {
	/// Added to the weapon's own reach, in degrees, for how far off the
	/// husk's body the crosshair may be: a heavy swing is wide.
	pub const CONE_DEGREES: f64 = 3.0;
	/// The farthest a husk may be, in metres, for a press to dive at it.
	pub const DIVE_METRES: f64 = 3.5;
	/// A heavy dive's parts: in, the swing (as long as the weapon's ticks
	/// per swing), then out.
	pub const IN_TICKS: i64 = 13;
	/// Every heavy swing that lands stops the husk in its tracks for at
	/// least this long. The greatsword stops it for longer.
	pub const KNOCK_TICKS: i64 = 12;
	pub const OUT_TICKS: i64 = 18;
}
