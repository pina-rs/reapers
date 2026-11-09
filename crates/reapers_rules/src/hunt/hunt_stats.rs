//! Who is hunting, and the numbers one hunt is played with.

use thiserror::Error;

use crate::armory::Temper;
use crate::armory::Weapon;
use crate::arts::ArtSkill;
use crate::hunt::arsenal::BladeRules;
use crate::hunt::arsenal::GunRules;
use crate::hunt::arsenal::HeavyRules;
use crate::hunt::capture::CaptureRules;
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
/// Every tunable of the encounter lives here, in [`HuntRules`] or in the
/// rules of the arsenal and the orb, so the game and the server read the
/// same values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HuntStats {
	/// The base hit, `H`: what one hit on the body does before the weapon's
	/// share of it, cadence and perks. A hit on an open seal does twice
	/// this.
	pub hit: f64,

	/// The class weapon's own strike: a heavy swing does this many hits.
	pub heavy_strike: f64,

	/// How far from an open seal the crosshair may be and still hit it: the
	/// seal cone.
	pub reach_radians: f64,

	/// How far off the husk's body a gunshot may be and still bend onto it.
	pub assist_radians: f64,

	/// How far off the husk's body the crosshair may be for the blade to
	/// reach it.
	pub blade_reach_radians: f64,

	/// The same for the class weapon's wide swing.
	pub heavy_reach_radians: f64,

	/// Ticks the class weapon takes to swing.
	pub swing_ticks: i64,

	/// Flinches the reaper can take before the husk slips away.
	pub resolve: i64,

	/// How long a broken husk stays down, waiting for an orb.
	pub broken_ticks: i64,

	/// How far off the husk's body an orb may be thrown and still strike
	/// it.
	pub orb_aim_radians: f64,

	/// The class's capture rating, from 1 to 5, which sets the odds of an
	/// orb.
	pub capture: i64,

	/// The turning speed, in radians a second, above which a reaper is
	/// flinching during a shriek.
	pub flinch_speed: f64,

	/// The turning speed, in radians a second, at or above which a swing of
	/// the phone turns a charging husk aside.
	pub parry_speed: f64,

	pub perk: ClassPerk,
	pub gift: Option<ReaperGift>,
	pub soul_rank: i64,

	/// How long a heavy swing that lands stops the husk in its tracks.
	pub knock_ticks: i64,

	/// The share of a hit on an open seal that a heavy swing on the body
	/// does. An axe bites wherever it lands.
	pub body_share: f64,

	/// The art the reaper throws, or `None`.
	pub art: Option<ArtSkill>,
}

impl HuntStats {
	/// Derives the hunt numbers for `loadout`.
	///
	/// `leniency` widens every window in the player's favour. The game plays
	/// at 1.0; the server replays a little above it, so a hunt that was won
	/// on the phone by a hair is never judged lost over a rounding
	/// difference. It leaves the orb alone: an orb that struck on the phone
	/// and missed on the server, or the other way round, would roll a
	/// different capture.
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
		let keen_degrees = if keen {
			SoulRules::KEEN_REACH_DEGREES
		} else {
			0.0
		};

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
			hit: HuntRules::BASE_STRIKE
				* (0.85 + 0.1 * stats.strike as f64)
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
			heavy_strike: profile.strike,
			reach_radians: radians(
				3.2 + 0.55 * stats.reach as f64 + profile.reach_degrees + keen_degrees,
			) * leniency,
			assist_radians: radians(
				GunRules::ASSIST_DEGREES
					+ GunRules::ASSIST_DEGREES_PER_REACH
						* (stats.reach - GunRules::ASSIST_MIDDLE_REACH) as f64
					+ keen_degrees,
			) * leniency,
			blade_reach_radians: radians(BladeRules::REACH_DEGREES) * leniency,
			heavy_reach_radians: radians(profile.reach_degrees + HeavyRules::CONE_DEGREES)
				* leniency,
			swing_ticks: profile.swing_ticks
				+ if patient {
					SoulRules::PATIENT_SWING_TICKS
				} else {
					0
				},
			resolve: 2 + (stats.resilience + 1) / 2 + i64::from(stalwart),
			broken_ticks: math::round(
				((3.5 + 0.5 * stats.capture as f64) * HuntRules::TICK_RATE as f64
					+ if patient {
						f64::from(SoulRules::PATIENT_BROKEN_TICKS as i32)
					} else {
						0.0
					}
					- if keen {
						f64::from(SoulRules::KEEN_BROKEN_TICKS as i32)
					} else {
						0.0
					})
					* leniency,
			),
			orb_aim_radians: radians(CaptureRules::AIM_DEGREES),
			capture: stats.capture,
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
			knock_ticks: profile.knock_ticks.max(HeavyRules::KNOCK_TICKS),
			body_share: profile.body_share,
			art: loadout.art,
		})
	}

	/// These numbers as they are at `difficulty`: wider or narrower windows,
	/// and more room to flinch.
	#[must_use]
	pub fn at(&self, difficulty: Difficulty) -> Self {
		Self {
			reach_radians: self.reach_radians * difficulty.reach(),
			assist_radians: self.assist_radians * difficulty.reach(),
			resolve: self.resolve + difficulty.resolve(),
			broken_ticks: math::round(self.broken_ticks as f64 * difficulty.window()),
			orb_aim_radians: self.orb_aim_radians * difficulty.orb_aim(),
			flinch_speed: self.flinch_speed * difficulty.flinch(),
			parry_speed: self.parry_speed * difficulty.cut(),
			..*self
		}
	}

	/// Every gift works at birth, then deepens through three awakenings.
	pub const fn gift_strength(&self) -> i64 {
		self.soul_rank + 1
	}

	/// Ward a careful reaper takes off each second with the gun alone, every
	/// shot on the body and every reload a plain one. The yardstick a
	/// rift's ward is measured against.
	pub fn gun_per_second(&self) -> f64 {
		let cycle =
			(GunRules::ROUNDS * GunRules::FIRE_INTERVAL_TICKS + GunRules::RELOAD_TICKS) as f64;

		self.hit * GunRules::SHOT_STRIKE * GunRules::ROUNDS as f64 * HuntRules::TICK_RATE as f64
			/ cycle
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
	/// An ambush multiplies the first hit by this.
	pub const AMBUSH_STRIKE: f64 = 2.0;
	/// Steady ticks needed to spring an ambush.
	pub const AMBUSH_TICKS: i64 = 66;
	/// A level-one hit with a plain weapon, before the class's lean: the
	/// `H` every weapon's damage is a share of.
	pub const BASE_STRIKE: f64 = 4.0;
	/// The chance that a bolting husk makes for cover or a ledge.
	pub const BOLT_COVER_CHANCE: f64 = 0.45;
	pub const BOLT_DELAY_MAX_TICKS: i64 = 200;
	/// How long a husk waits after a shriek before it bolts.
	pub const BOLT_DELAY_MIN_TICKS: i64 = 110;
	pub const BOLT_MAX_DEGREES: f64 = 105.0;
	/// The nearest and farthest a husk bolts, in degrees around the player.
	pub const BOLT_MIN_DEGREES: f64 = 45.0;
	// Hits --------------------------------------------------------------

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
	/// A cleaving hit does this share to every other open seal.
	pub const CLEAVE_SHARE: f64 = 0.4;
	/// The nearest a husk stands to the player when the room leaves it no
	/// more than that, in metres. In a tight corner it comes this close
	/// rather than not appear at all; nearer and it would fill the view.
	pub const CLOSEST_METRES: f64 = 0.8;
	/// How far round the player, in degrees, an ambling husk looks for
	/// cover.
	pub const COVER_SEARCH_DEGREES: f64 = 50.0;
	/// A hit on an open seal, or a stab at a stunned husk, lands this many
	/// times as hard.
	pub const CRITICAL_STRIKE: f64 = 2.0;
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
	// Break -------------------------------------------------------------

	/// The fight ends, with the husk escaping, after this long. The clock
	/// stops while the husk lies broken.
	pub const FIGHT_LIMIT_TICKS: i64 = 150 * Self::TICK_RATE;
	/// The farthest behind its usual range a husk hides, in metres.
	pub const HIDE_BEYOND: f64 = 1.5;
	pub const HOP_METRES: f64 = 0.22;
	/// Broken windows a husk lets pass without an orb before it slips away
	/// for good. The last one to pass is its escape.
	pub const LAPSES: i64 = 2;
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
	/// Hitting seals in the order they opened multiplies the hit by this.
	pub const MEASURE_STRIKE: f64 = 1.2;
	/// The nearest a husk walking a room's grid comes to the player, in
	/// metres, except while it charges or stands stunned after one. No
	/// place it stands, perches or hides is nearer, and its routes go round.
	pub const NEAREST_METRES: f64 = 1.5;
	/// Seals open at once during an ordinary stretch of the fight.
	pub const OPEN_SEALS: usize = 2;
	/// Swinging the phone at least this fast, in radians a second, turns a
	/// charge aside. About a third of a turn a second: a flick of the wrist,
	/// well clear of the speed anyone holds a phone still at.
	pub const PARRY_SPEED: f64 = 2.0;
	/// How quickly the room a husk has opens out to the side of something
	/// in its way, in metres for every radian round the player. It is what
	/// turns the edge of a sofa into a curve a husk can run along.
	pub const REACH_EASE: f64 = 2.0;
	/// Ward a husk regains, as a share of its full ward, when a broken
	/// window passes without an orb.
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
	pub const STAGING_FAR_METRES: f64 = 3.5;
	/// The nearest and farthest from the player a husk walking a room's
	/// grid takes up open floor, in metres. A move picks a distance in this
	/// band and the open floor nearest it.
	pub const STAGING_NEAR_METRES: f64 = 2.5;
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
	/// Each unflinching hold adds this to the broken window, with the toll
	/// perk.
	pub const TOLL_TICKS: i64 = 80;
	/// How fast a husk turns its body, in degrees a tick.
	pub const TURN_DEGREES_PER_TICK: f64 = 5.0;
	// Shriek ------------------------------------------------------------

	/// The warning before a shriek.
	pub const WINDUP_TICKS: i64 = 54;
}
