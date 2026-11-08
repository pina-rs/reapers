//! The fixed numbers of a rift.

use core::time::Duration;

use crate::hunt::hunt_stats::HuntRules;
use crate::hunt::hunt_stats::HuntStats;

/// The fixed numbers of a rift: a husk too strong for one reaper, fought by
/// a circle of them at once.
///
/// These are playtest starting points, written down in docs/gameplay.md.
pub struct RiftRules;

impl RiftRules {
	/// Anima for the same.
	pub const ANIMA: i64 = 90;
	/// Absorbed NFTs from a rift's collection needed before it opens for a
	/// reaper.
	pub const ATTUNEMENT: usize = 1;
	/// The ward is sized so that a circle playing carefully breaks it in
	/// about this long, whatever its levels are.
	pub const BREAK_SECONDS: i64 = 70;
	/// What a reaper who was cast out before the end earns.
	pub const CAST_OUT_SHARE: f64 = 0.5;
	/// Reapers in a full circle. Seats nobody takes are filled by shades.
	pub const CIRCLE: usize = 4;
	/// The count before the fight, once every reaper present stands ready.
	pub const COUNTDOWN_TICKS: i64 = 3 * HuntRules::TICK_RATE;
	pub const FARTHEST_METRES: f64 = 40.0;
	/// The rift closes, unsealed, after this much fighting.
	pub const FIGHT_LIMIT_TICKS: i64 = 120 * HuntRules::TICK_RATE;
	/// How long a rift waits for reapers before it starts with whoever came.
	pub const GATHER_LIMIT_TICKS: i64 = 60 * HuntRules::TICK_RATE;
	/// A reaper who pulled their share of the ward earns the whole reward;
	/// one who did nothing still earns this share of it for standing in the
	/// circle.
	pub const IDLE_SHARE: f64 = 0.6;
	/// Coarse, old or future-dated fixes cannot place a shared rift.
	pub const LOCATION_ACCURACY_METRES: f64 = 25.0;
	pub const LOCATION_CLOCK_SKEW: Duration = Duration::from_secs(5);
	pub const LOCATION_MAX_AGE: Duration = Duration::from_secs(30);
	/// The latest foreground reading is shared at most this often.
	pub const LOCATION_REPORT_INTERVAL: Duration = Duration::from_secs(5);
	/// The nearest and farthest the husk is drawn from a reaper, in metres,
	/// however near or far they stand. Nearer and it would not fit on the
	/// screen; farther and its seals would be too small to strike.
	pub const NEAREST_METRES: f64 = 14.0;
	/// Anywhere else, the rift stands this far in front of the first reaper
	/// to say where they are. Everyone who comes after sees it in that same
	/// place.
	pub const PLANT_METRES: f64 = 25.0;
	/// How long the circle has to cut once the ward breaks.
	pub const REAP_WINDOW_TICKS: i64 = 8 * HuntRules::TICK_RATE;
	/// A shade fights this many levels below the circle's average.
	pub const SHADE_LEVEL_GAP: i64 = 1;
	/// A reaper farther than this from where the rift stands is not there in
	/// any useful sense, and sees it dead ahead, as through the veil.
	pub const SIGHT_METRES: f64 = 150.0;
	// Where a rift stands -----------------------------------------------

	/// A reaper standing this near a rift's site, in metres, finds the rift
	/// at the site itself.
	pub const SITE_METRES: f64 = 250.0;
	/// The share of its best possible damage a careful reaper really lands:
	/// seals are not always open, shrieks stop every blade, and taps go
	/// wide.
	pub const STEADY_SHARE: f64 = 0.84;
	/// What a reaper who was still standing but made no cut earns, as a
	/// share of a clean cut.
	pub const UNCUT_SHARE: f64 = 0.75;
	/// Experience for sealing a rift with a clean cut and a full share of
	/// the work: about what one epic husk is worth. A rift costs no NFT to
	/// enter, so it must not pay better than the hunts that do.
	pub const XP: i64 = 220;

	/// The ward a rift's husk raises against `circle`, one entry for every
	/// reaper and shade in it.
	///
	/// A rift is as hard for a circle of beginners as for a circle of
	/// veterans: what it asks for is company and care, not levels.
	pub fn ward_for<'a>(circle: impl IntoIterator<Item = &'a HuntStats>) -> f64 {
		let mut per_second = 0.0;

		for stats in circle {
			per_second +=
				stats.strike_damage * HuntRules::TICK_RATE as f64 / stats.swing_ticks as f64;
		}

		per_second * Self::STEADY_SHARE * Self::BREAK_SECONDS as f64
	}
}
