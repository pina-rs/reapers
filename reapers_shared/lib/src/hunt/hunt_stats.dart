import '../armory.dart';
import '../progression.dart';
import '../reaper_class.dart';
import '../vec3.dart';
import 'difficulty.dart';

/// Who is hunting: everything about a reaper that changes how a hunt plays.
final class HuntLoadout {
  const HuntLoadout({
    required this.reaperClass,
    required this.level,
    required this.weapon,
    this.completedSets = 0,
  });

  /// The fixed-strength reaper lent for a practice hunt.
  static const borrowed = HuntLoadout(
    reaperClass: ReaperClass.harvester,
    level: 4,
    weapon: Weapon(WeaponKind.scythe, Temper.shrouded),
  );

  final ReaperClass reaperClass;

  final int level;

  final Weapon weapon;

  /// How many collection sets the reaper has absorbed in full.
  final int completedSets;
}

/// The numbers one hunt is played with, derived from a [HuntLoadout].
///
/// Every tunable of the encounter lives here or in [HuntRules], so the game
/// and the server read the same values.
final class HuntStats {
  const HuntStats({
    required this.strikeDamage,
    required this.reachRadians,
    required this.swingTicks,
    required this.resolve,
    required this.captureTicks,
    required this.reapToleranceRadians,
    required this.flinchSpeed,
    required this.parrySpeed,
    required this.perk,
  });

  /// Derives the hunt numbers for [loadout].
  ///
  /// [leniency] widens every window in the player's favour. The game plays at
  /// 1.0; the server replays a little above it, so a hunt that was won on the
  /// phone by a hair is never judged lost over a rounding difference.
  factory HuntStats.derive(HuntLoadout loadout, {double leniency = 1}) {
    final stats = loadout.reaperClass.stats;
    final profile = loadout.weapon.profile;
    final levelScale = 1 + Progression.strikePerLevel * (loadout.level - 1);
    final setScale = 1 + Progression.strikePerSet * loadout.completedSets;

    return HuntStats(
      strikeDamage:
          HuntRules.baseStrike *
          (0.85 + 0.1 * stats.strike) *
          profile.strike *
          loadout.weapon.temper.strike *
          levelScale *
          setScale,
      reachRadians:
          radians(3.2 + 0.55 * stats.reach + profile.reachDegrees) * leniency,
      swingTicks: profile.swingTicks,
      resolve: 2 + (stats.resilience + 1) ~/ 2,
      captureTicks:
          ((3.5 + 0.5 * stats.capture) * HuntRules.tickRate * leniency).round(),
      reapToleranceRadians: radians(5 + 0.8 * stats.capture) * leniency,
      flinchSpeed: (0.42 + 0.035 * stats.resilience) * leniency,
      parrySpeed: HuntRules.parrySpeed / leniency,
      perk: loadout.reaperClass.perk,
    );
  }

  /// These numbers as they are at [difficulty]: wider or narrower windows,
  /// and more room to flinch.
  HuntStats at(Difficulty difficulty) => HuntStats(
    strikeDamage: strikeDamage,
    reachRadians: reachRadians * difficulty.reach,
    swingTicks: swingTicks,
    resolve: resolve + difficulty.resolve,
    captureTicks: (captureTicks * difficulty.window).round(),
    reapToleranceRadians: reapToleranceRadians * difficulty.tolerance,
    flinchSpeed: flinchSpeed * difficulty.flinch,
    parrySpeed: parrySpeed * difficulty.cut,
    perk: perk,
  );

  /// Ward removed by one strike on an open seal, before cadence and perks.
  final double strikeDamage;

  /// How far from a seal a tap may land and still strike it.
  final double reachRadians;

  /// Ticks between one strike and the next.
  final int swingTicks;

  /// Flinches the reaper can take before the husk slips away.
  final int resolve;

  /// How long a broken husk stays open to the reap.
  final int captureTicks;

  /// How far the phone may stray from the reap arc and still cut clean.
  final double reapToleranceRadians;

  /// The turning speed, in radians a second, above which a reaper is
  /// flinching during a shriek.
  final double flinchSpeed;

  /// The turning speed, in radians a second, at or above which a swing of
  /// the phone turns a charging husk aside.
  final double parrySpeed;

  final ClassPerk perk;
}

/// The fixed numbers of an encounter.
///
/// These are playtest starting points, written down in docs/gameplay.md.
abstract final class HuntRules {
  /// The simulation advances this many times a second, whatever the display
  /// refresh rate is.
  static const tickRate = 60;

  /// A level-one strike with a plain weapon, before the class's lean.
  static const baseStrike = 4.0;

  // Sigil -------------------------------------------------------------

  /// The lowest and highest a sigil's lights hang for a husk that walks, in
  /// radians above the horizon. Resting on any of them, the camera sees the
  /// floor from under the reaper's feet to several metres out.
  static final sigilGroundBand = (radians(-42), radians(-14));

  /// The same for a husk that flies: about head height, where it will be.
  static final sigilAirBand = (radians(-6), radians(26));

  /// Turning faster than this, in radians a second, a light the view rests
  /// on does not fill. It is also as fast as the camera can turn and still
  /// take a picture the room can be read from.
  static const sigilSteadySpeed = 0.7;

  /// A light the view has left empties this many times as fast as it
  /// filled.
  static const sigilDrain = 2.0;

  /// How far past a sigil's outermost lights the room is read, either side:
  /// half of what the camera takes in across.
  static final sigilMargin = radians(15);

  // Stalk -------------------------------------------------------------

  /// Turning faster than this, in radians a second, alarms an unaware husk.
  static const startleSpeed = 1.7;

  /// Alarm added each second for every radian a second above
  /// [startleSpeed]. The husk notices at 1.
  static const startleGain = 0.55;

  /// Alarm lost each second while the reaper moves slowly.
  static const startleDecay = 0.2;

  /// The aim must rest within this angle of the husk to steady an ambush.
  static final ambushAim = radians(9);

  /// Turning slower than this counts as steady, in radians a second.
  static const steadySpeed = 0.6;

  /// Steady ticks needed to spring an ambush.
  static const ambushTicks = 66;

  /// A husk that is never found notices the reaper after this long.
  static const stalkLimitTicks = 25 * tickRate;

  /// How long every seal stays open after an ambush or an unflinching hold.
  static const exposedTicks = 170;

  /// An ambush multiplies the first strike by this.
  static const ambushStrike = 2.0;

  /// An ambushed husk is worth this much more.
  static const ambushReward = 1.1;

  // Strike ------------------------------------------------------------

  /// A tap made while the weapon is still swinging waits this long for the
  /// swing to finish before it is dropped.
  static const bufferTicks = 8;

  /// A strike on the body, away from any seal, does this share of a strike.
  static const glanceShare = 0.25;

  /// Hits closer together than this keep the cadence going.
  static const cadenceWindowTicks = 78;

  /// Each hit in cadence adds this much to the next.
  static const cadenceStep = 0.1;

  /// Hits in cadence that count toward the bonus.
  static const cadenceCap = 5;

  /// The same cap for a reaper with the chain perk.
  static const chainCadenceCap = 10;

  /// A cleaving strike does this share to each further seal it catches.
  static const cleaveShare = 0.4;

  /// A cleaving strike catches seals within this many reaches of the tap.
  static const cleaveReach = 2.4;

  /// Striking seals in the order they opened multiplies the strike by this.
  static const measureStrike = 1.2;

  /// The strike after an unflinching hold, for a reaper with that perk.
  static const chosenBlowStrike = 3.0;

  /// How long a struck seal stays shut.
  static const sealCooldownTicks = 44;

  /// Seals open at once during an ordinary stretch of the fight.
  static const openSeals = 2;

  // Shriek ------------------------------------------------------------

  /// The warning before a shriek.
  static const windupTicks = 54;

  /// How long the reaper has to hold still.
  static const shriekTicks = 66;

  /// Movement in the first moments of a shriek is forgiven.
  static const shriekGraceTicks = 7;

  /// The shortest and longest gaps between shrieks.
  static const shriekGapMinTicks = 6 * tickRate + 30;
  static const shriekGapMaxTicks = 10 * tickRate;

  /// Each unflinching hold adds this to the reap window, with the toll perk.
  static const tollTicks = 80;

  // Charge ------------------------------------------------------------

  /// How long a charging husk takes to reach the reaper.
  static const chargeTicks = 66;

  /// A swing in the first moments of a charge does not count: the reaper
  /// was still turning to follow the husk, not cutting at it.
  static const chargeGraceTicks = 10;

  /// Swinging the phone at least this fast, in radians a second, turns a
  /// charge aside. About a third of a turn a second: a flick of the wrist,
  /// well clear of the speed anyone holds a phone still at.
  static const parrySpeed = 2.0;

  /// How near a charging husk comes, in metres.
  static const chargeMetres = 0.9;

  /// The shortest and longest gaps between one charge and the next.
  static const chargeGapMinTicks = 8 * tickRate;
  static const chargeGapMaxTicks = 14 * tickRate;

  /// How long a husk that was turned aside stands stunned, every seal in
  /// view open.
  static const staggerTicks = 100;

  /// A charge never starts this close ahead of a shriek, and a shriek never
  /// follows this soon after one: the two ask opposite things.
  static const chargeClearTicks = 90;

  // Movement ----------------------------------------------------------

  /// How high the phone is taken to be above the floor, in centimetres, when
  /// it has not measured it: a standing adult holding a phone at chest
  /// height. A phone that can read the room measures this instead, and the
  /// hunt's plan carries what it found. See docs/gameplay.md.
  static const eyeCentimetres = 145;

  /// [eyeCentimetres] in metres.
  static const eyeHeight = eyeCentimetres / 100;

  /// The nearest a husk stands to the player when the room leaves it no
  /// more than that, in metres. In a tight corner it comes this close
  /// rather than not appear at all; nearer and it would fill the view.
  static const closestMetres = 0.8;

  /// How quickly the room a husk has opens out to the side of something in
  /// its way, in metres for every radian round the player. It is what turns
  /// the edge of a sofa into a curve a husk can run along.
  static const reachEase = 2.0;

  /// The shortest and longest a husk rests between one amble and the next.
  static const restMinTicks = 50;
  static const restMaxTicks = 170;

  /// The farthest round the player a husk ambles in one go, in degrees.
  static const ambleDegrees = 32.0;

  /// The nearest and farthest a husk bolts, in degrees around the player.
  static const boltMinDegrees = 45.0;
  static const boltMaxDegrees = 105.0;

  /// How long a husk waits after a shriek before it bolts.
  static const boltDelayMinTicks = 110;
  static const boltDelayMaxTicks = 200;

  /// How fast a husk turns its body, in degrees a tick.
  static const turnDegreesPerTick = 5.0;

  // Break and reap ----------------------------------------------------

  /// The fight ends, with the husk escaping, after this long.
  static const fightLimitTicks = 75 * tickRate;

  /// Half the width of the reap arc.
  static final reapHalfSpan = radians(26);

  /// The aim must come within this of the arc's start to begin the cut.
  static final reapStartAim = radians(7);

  /// Sweeping faster than this, in radians a second, tears the cut.
  static const reapSpeedLimit = 2.1;

  /// A cut this much steadier than the tolerance is perfect.
  static const perfectShare = 0.45;

  /// Ward a husk regains, as a share of its full ward, when a reap window
  /// closes unfinished.
  static const recoverShare = 0.3;

  /// Unfinished reaps a husk allows before it slips away.
  static const reapAttempts = 2;
}
