//! Who is hunting, and the numbers one hunt is played with.

use thiserror::Error;

use crate::armory::Temper;
use crate::armory::Weapon;
use crate::arts::ArtSkill;
use crate::hunt::difficulty::Difficulty;
use crate::math;
use crate::progression::Progression;
use crate::reaper_class::ClassPerk;
use crate::reaper_class::ReaperClass;
use crate::reaper_class::WeaponKind;
use crate::reaper_soul::ReaperGift;
use crate::reaper_soul::ReaperSoul;
use crate::reaper_soul::SoulRules;
use crate::reaper_soul::Temperament;
use crate::vec3::radians;

/// Who is hunting: everything about a reaper that changes how a hunt plays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntLoadout {
	pub reaper_class: ReaperClass,

	pub level: i64,

	pub weapon: Weapon,

	/// How many collection sets the reaper has absorbed in full.
	pub completed_sets: i64,
	pub soul: Option<ReaperSoul>,
	pub soul_rank: i64,

	/// The art the reaper throws, or `None` for one that has learned none.
	pub art: Option<ArtSkill>,
}

impl HuntLoadout {
	/// The fixed-strength reaper lent for a practice hunt.
	pub const BORROWED: Self = Self::new(
		ReaperClass::Harvester,
		4,
		Weapon::new(WeaponKind::Scythe, Temper::Shrouded),
	);

	/// A reaper of `reaper_class` at `level` with `weapon`, and nothing else
	/// to its name: no sets, no soul, no art.
	pub const fn new(reaper_class: ReaperClass, level: i64, weapon: Weapon) -> Self {
		Self {
			reaper_class,
			level,
			weapon,
			completed_sets: 0,
			soul: None,
			soul_rank: 0,
			art: None,
		}
	}
}

/// A loadout no reaper could have: a level out of range, a soul rank out of
/// range, or a gift that belongs to another class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("Invalid reaper loadout")]
pub struct InvalidLoadout;

/// The numbers one hunt is played with, derived from a [`HuntLoadout`].
///
/// Every tunable of the encounter lives here or in [`HuntRules`], so the
/// game and the server read the same values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntStats {
	/// Ward removed by one strike on an open seal, before cadence and perks.
	pub strike_damage: f64,

	/// How far from a seal a tap may land and still strike it.
	pub reach_radians: f64,

	/// Ticks between one strike and the next.
	pub swing_ticks: i64,

	/// Flinches the reaper can take before the husk slips away.
	pub resolve: i64,

	/// How long a broken husk stays open to the reap.
	pub capture_ticks: i64,

	/// How far the phone may stray from the reap arc and still cut clean.
	pub reap_tolerance_radians: f64,

	/// The turning speed, in radians a second, above which a reaper is
	/// flinching during a shriek.
	pub flinch_speed: f64,

	/// The turning speed, in radians a second, at or above which a swing of
	/// the phone turns a charging husk aside.
	pub parry_speed: f64,

	pub perk: ClassPerk,
	pub gift: Option<ReaperGift>,
	pub soul_rank: i64,

	/// How long a struck seal stops the husk in its tracks: the weight of a
	/// heavy blade.
	pub knock_ticks: i64,

	/// The share of a strike a blow on the body away from a seal does.
	pub glance_share: f64,

	/// The art the reaper throws, or `None`.
	pub art: Option<ArtSkill>,
}

impl HuntStats {
	/// Derives the hunt numbers for `loadout`.
	///
	/// `leniency` widens every window in the player's favour. The game plays
	/// at 1.0; the server replays a little above it, so a hunt that was won
	/// on the phone by a hair is never judged lost over a rounding
	/// difference.
	pub fn derive(loadout: &HuntLoadout, leniency: f64) -> Result<Self, InvalidLoadout> {
		let stats = loadout.reaper_class.stats();
		let profile = loadout.weapon.profile();
		let level_scale = 1.0 + Progression::STRIKE_PER_LEVEL * (loadout.level - 1) as f64;
		let set_scale = 1.0 + Progression::STRIKE_PER_SET * loadout.completed_sets as f64;
		let temperament = loadout.soul.map(|soul| soul.temperament);
		let patient = temperament == Some(Temperament::Patient);
		let keen = temperament == Some(Temperament::Keen);
		let stalwart = temperament == Some(Temperament::Stalwart);
		let hungry = temperament == Some(Temperament::Hungry);

		if loadout.level < 1
			|| loadout.level > Progression::LEVEL_CAP
			|| loadout.soul_rank < 0
			|| loadout.soul_rank > SoulRules::MAX_RANK
			|| loadout
				.soul
				.is_some_and(|soul| soul.gift.reaper_class() != loadout.reaper_class)
		{
			return Err(InvalidLoadout);
		}

		Ok(Self {
			strike_damage: HuntRules::BASE_STRIKE
				* (0.85 + 0.1 * stats.strike as f64)
				* profile.strike
				* loadout.weapon.temper.strike()
				* level_scale
				* set_scale
				* if stalwart {
					SoulRules::STALWART_STRIKE
				} else {
					1.0
				}
				* if hungry {
					SoulRules::HUNGRY_STRIKE
				} else {
					1.0
				},
			reach_radians: radians(
				3.2 + 0.55 * stats.reach as f64
					+ profile.reach_degrees
					+ if keen {
						SoulRules::KEEN_REACH_DEGREES
					} else {
						0.0
					},
			) * leniency,
			swing_ticks: profile.swing_ticks
				+ if patient {
					SoulRules::PATIENT_SWING_TICKS
				} else {
					0
				},
			resolve: 2 + (stats.resilience + 1) / 2 + i64::from(stalwart),
			capture_ticks: math::round(
				((3.5 + 0.5 * stats.capture as f64) * HuntRules::TICK_RATE as f64
					+ if patient {
						SoulRules::PATIENT_CAPTURE_TICKS
					} else {
						0
					} as f64
					- if keen {
						SoulRules::KEEN_CAPTURE_TICKS
					} else {
						0
					} as f64)
					* leniency,
			),
			reap_tolerance_radians: radians(5.0 + 0.8 * stats.capture as f64) * leniency,
			flinch_speed: (0.42 + 0.035 * stats.resilience as f64)
				* leniency
				* if hungry {
					SoulRules::HUNGRY_FLINCH
				} else {
					1.0
				},
			parry_speed: HuntRules::PARRY_SPEED / leniency,
			perk: loadout.reaper_class.perk(),
			gift: loadout.soul.map(|soul| soul.gift),
			soul_rank: loadout.soul_rank,
			knock_ticks: profile.knock_ticks,
			glance_share: profile.glance_share,
			art: loadout.art,
		})
	}

	/// These numbers as they are at `difficulty`: wider or narrower windows,
	/// and more room to flinch.
	#[must_use]
	pub fn at(&self, difficulty: Difficulty) -> Self {
		Self {
			reach_radians: self.reach_radians * difficulty.reach(),
			resolve: self.resolve + difficulty.resolve(),
			capture_ticks: math::round(self.capture_ticks as f64 * difficulty.window()),
			reap_tolerance_radians: self.reap_tolerance_radians * difficulty.tolerance(),
			flinch_speed: self.flinch_speed * difficulty.flinch(),
			parry_speed: self.parry_speed * difficulty.cut(),
			..*self
		}
	}

	/// Every gift works at birth, then deepens through three awakenings.
	pub const fn gift_strength(&self) -> i64 {
		self.soul_rank + 1
	}
}

/// The fixed numbers of an encounter.
///
/// These are playtest starting points, written down in docs/gameplay.md.
pub struct HuntRules;

impl HuntRules {
	// Cover -------------------------------------------------------------

	/// The chance that a husk ambling about the room makes for cover or a
	/// ledge nearby instead, when there is one.
	pub const AMBLE_COVER_CHANCE: f64 = 0.35;
	/// The farthest round the player a husk ambles in one go, in degrees.
	pub const AMBLE_DEGREES: f64 = 32.0;
	/// The aim must rest within this angle of the husk to steady an ambush.
	pub const AMBUSH_AIM: f64 = radians(9.0);
	/// An ambushed husk is worth this much more.
	pub const AMBUSH_REWARD: f64 = 1.1;
	/// An ambush multiplies the first strike by this.
	pub const AMBUSH_STRIKE: f64 = 2.0;
	/// Steady ticks needed to spring an ambush.
	pub const AMBUSH_TICKS: i64 = 66;
	/// A level-one strike with a plain weapon, before the class's lean.
	pub const BASE_STRIKE: f64 = 4.0;
	/// The chance that a bolting husk makes for cover or a ledge.
	pub const BOLT_COVER_CHANCE: f64 = 0.45;
	pub const BOLT_DELAY_MAX_TICKS: i64 = 200;
	/// How long a husk waits after a shriek before it bolts.
	pub const BOLT_DELAY_MIN_TICKS: i64 = 110;
	pub const BOLT_MAX_DEGREES: f64 = 105.0;
	/// The nearest and farthest a husk bolts, in degrees around the player.
	pub const BOLT_MIN_DEGREES: f64 = 45.0;
	// Strike ------------------------------------------------------------

	/// A tap made while the weapon is still swinging waits this long for the
	/// swing to finish before it is dropped.
	pub const BUFFER_TICKS: i64 = 8;
	/// Hits in cadence that count toward the bonus.
	pub const CADENCE_CAP: i64 = 5;
	/// Each hit in cadence adds this much to the next.
	pub const CADENCE_STEP: f64 = 0.1;
	/// Hits closer together than this keep the cadence going.
	pub const CADENCE_WINDOW_TICKS: i64 = 78;
	/// The same cap for a reaper with the chain perk.
	pub const CHAIN_CADENCE_CAP: i64 = 10;
	/// A charge never starts this close ahead of a shriek, and a shriek
	/// never follows this soon after one: the two ask opposite things.
	pub const CHARGE_CLEAR_TICKS: i64 = 90;
	pub const CHARGE_GAP_MAX_TICKS: i64 = 14 * Self::TICK_RATE;
	/// The shortest and longest gaps between one charge and the next.
	pub const CHARGE_GAP_MIN_TICKS: i64 = 8 * Self::TICK_RATE;
	/// A swing in the first moments of a charge does not count: the reaper
	/// was still turning to follow the husk, not cutting at it.
	pub const CHARGE_GRACE_TICKS: i64 = 10;
	/// How near a charging husk comes, in metres.
	pub const CHARGE_METRES: f64 = 0.9;
	// Charge ------------------------------------------------------------

	/// How long a charging husk takes to reach the reaper.
	pub const CHARGE_TICKS: i64 = 66;
	/// The strike after an unflinching hold, for a reaper with that perk.
	pub const CHOSEN_BLOW_STRIKE: f64 = 3.0;
	/// A cleaving strike catches seals within this many reaches of the tap.
	pub const CLEAVE_REACH: f64 = 2.4;
	/// A cleaving strike does this share to each further seal it catches.
	pub const CLEAVE_SHARE: f64 = 0.4;
	/// The nearest a husk stands to the player when the room leaves it no
	/// more than that, in metres. In a tight corner it comes this close
	/// rather than not appear at all; nearer and it would fill the view.
	pub const CLOSEST_METRES: f64 = 0.8;
	/// How far round the player, in degrees, an ambling husk looks for
	/// cover.
	pub const COVER_SEARCH_DEGREES: f64 = 50.0;
	/// How long every seal stays open after an ambush or an unflinching
	/// hold.
	pub const EXPOSED_TICKS: i64 = 170;
	// Movement ----------------------------------------------------------

	/// How high the phone is taken to be above the floor, in centimetres,
	/// when it has not measured it: a standing adult holding a phone at
	/// chest height. A phone that can read the room measures this instead,
	/// and the hunt's plan carries what it found. See docs/gameplay.md.
	pub const EYE_CENTIMETRES: i32 = 145;
	/// [`EYE_CENTIMETRES`](Self::EYE_CENTIMETRES) in metres.
	pub const EYE_HEIGHT: f64 = Self::EYE_CENTIMETRES as f64 / 100.0;
	// Break and reap ----------------------------------------------------

	/// The fight ends, with the husk escaping, after this long.
	pub const FIGHT_LIMIT_TICKS: i64 = 75 * Self::TICK_RATE;
	/// A strike on the body, away from any seal, does this share of a
	/// strike.
	pub const GLANCE_SHARE: f64 = 0.25;
	/// The farthest behind its usual range a husk hides, in metres.
	pub const HIDE_BEYOND: f64 = 1.5;
	pub const HOP_METRES: f64 = 0.22;
	/// How much higher than it has to a leaping husk's feet clear what it
	/// leaps over, in metres, and the hop of a leap with nothing to clear.
	pub const LEAP_CLEARANCE: f64 = 0.12;
	/// The shortest a leap takes.
	pub const LEAP_MIN_TICKS: i64 = 34;
	/// A leap goes at this share of the husk's bolting speed.
	pub const LEAP_PACE: f64 = 0.75;
	/// How far onto a ledge a husk lands, as a share of its own half width,
	/// past the ledge's front edge.
	pub const LEDGE_INSET: f64 = 0.6;
	/// The lowest ledge worth jumping onto, in metres: lower is a step.
	pub const LOWEST_LEDGE: f64 = 0.3;
	/// Striking seals in the order they opened multiplies the strike by
	/// this.
	pub const MEASURE_STRIKE: f64 = 1.2;
	/// Seals open at once during an ordinary stretch of the fight.
	pub const OPEN_SEALS: usize = 2;
	/// Swinging the phone at least this fast, in radians a second, turns a
	/// charge aside. About a third of a turn a second: a flick of the wrist,
	/// well clear of the speed anyone holds a phone still at.
	pub const PARRY_SPEED: f64 = 2.0;
	/// A cut this much steadier than the tolerance is perfect.
	pub const PERFECT_SHARE: f64 = 0.45;
	/// How quickly the room a husk has opens out to the side of something
	/// in its way, in metres for every radian round the player. It is what
	/// turns the edge of a sofa into a curve a husk can run along.
	pub const REACH_EASE: f64 = 2.0;
	/// Unfinished reaps a husk allows before it slips away.
	pub const REAP_ATTEMPTS: i64 = 2;
	/// Half the width of the reap arc.
	pub const REAP_HALF_SPAN: f64 = radians(26.0);
	/// Sweeping faster than this, in radians a second, tears the cut.
	pub const REAP_SPEED_LIMIT: f64 = 2.1;
	/// The aim must come within this of the arc's start to begin the cut.
	pub const REAP_START_AIM: f64 = radians(7.0);
	/// Ward a husk regains, as a share of its full ward, when a reap window
	/// closes unfinished.
	pub const RECOVER_SHARE: f64 = 0.3;
	pub const REST_MAX_TICKS: i64 = 170;
	/// The shortest and longest a husk rests between one amble and the
	/// next.
	pub const REST_MIN_TICKS: i64 = 50;
	/// How long a struck seal stays shut.
	pub const SEAL_COOLDOWN_TICKS: i64 = 44;
	pub const SHRIEK_GAP_MAX_TICKS: i64 = 10 * Self::TICK_RATE;
	/// The shortest and longest gaps between shrieks.
	pub const SHRIEK_GAP_MIN_TICKS: i64 = 6 * Self::TICK_RATE + 30;
	/// Movement in the first moments of a shriek is forgiven.
	pub const SHRIEK_GRACE_TICKS: i64 = 7;
	/// How long the reaper has to hold still.
	pub const SHRIEK_TICKS: i64 = 66;
	/// The same for a husk that flies: about head height, where it will be.
	pub const SIGIL_AIR_BAND: (f64, f64) = (radians(-6.0), radians(26.0));
	/// A light the view has left empties this many times as fast as it
	/// filled.
	pub const SIGIL_DRAIN: f64 = 2.0;
	// Sigil -------------------------------------------------------------

	/// The lowest and highest a sigil's lights hang for a husk that walks,
	/// in radians above the horizon. Resting on any of them, the camera sees
	/// the floor from under the reaper's feet to several metres out.
	pub const SIGIL_GROUND_BAND: (f64, f64) = (radians(-42.0), radians(-14.0));
	/// How far past a sigil's outermost lights the room is read, either
	/// side: half of what the camera takes in across.
	pub const SIGIL_MARGIN: f64 = radians(15.0);
	/// Turning faster than this, in radians a second, a light the view rests
	/// on does not fill. It is also as fast as the camera can turn and still
	/// take a picture the room can be read from.
	pub const SIGIL_STEADY_SPEED: f64 = 0.7;
	/// How long a husk that was turned aside stands stunned, every seal in
	/// view open.
	pub const STAGGER_TICKS: i64 = 100;
	/// A husk that is never found notices the reaper after this long.
	pub const STALK_LIMIT_TICKS: i64 = 25 * Self::TICK_RATE;
	/// Alarm lost each second while the reaper moves slowly.
	pub const STARTLE_DECAY: f64 = 0.2;
	/// Alarm added each second for every radian a second above
	/// [`STARTLE_SPEED`](Self::STARTLE_SPEED). The husk notices at 1.
	pub const STARTLE_GAIN: f64 = 0.55;
	// Stalk -------------------------------------------------------------

	/// Turning faster than this, in radians a second, alarms an unaware
	/// husk.
	pub const STARTLE_SPEED: f64 = 1.7;
	/// The chance that a husk in cover or up on a ledge stays put when it is
	/// next ready to move.
	pub const STAY_COVER_CHANCE: f64 = 0.4;
	/// Turning slower than this counts as steady, in radians a second.
	pub const STEADY_SPEED: f64 = 0.6;
	/// The simulation advances this many times a second, whatever the
	/// display refresh rate is.
	pub const TICK_RATE: i64 = 60;
	/// Each unflinching hold adds this to the reap window, with the toll
	/// perk.
	pub const TOLL_TICKS: i64 = 80;
	/// How fast a husk turns its body, in degrees a tick.
	pub const TURN_DEGREES_PER_TICK: f64 = 5.0;
	// Shriek ------------------------------------------------------------

	/// The warning before a shriek.
	pub const WINDUP_TICKS: i64 = 54;
}
