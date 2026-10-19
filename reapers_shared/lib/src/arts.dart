import 'hunt/hunt_stats.dart';
import 'vec3.dart';
import 'vessel.dart';

/// A reaper's second way to fight: something it throws.
///
/// A reaper learns an art by absorbing the creatures that carry it. The
/// first husk of one teaches it, and more of them deepen it. Nothing about
/// an art is bought: what a reaper can throw is a record of what it has
/// hunted.
///
/// An art is thrown, not tapped. The reaper holds a thumb on it to gather
/// it while keeping the phone steady on where it should go, then flicks the
/// phone to let it fly. Each one answers the room differently: fire is
/// lobbed over whatever a husk hides behind, ice flies straight and breaks
/// on it, and wind blows a husk out of cover.
enum Art {
  /// Fire. A lobbed ember that bursts where it lands and sets a husk
  /// burning. It arcs over cover, and a burning husk runs into the open.
  cinder(
    vessels: {Vessel.ape, Vessel.robot},
    speed: 6,
    homing: false,
    strike: [2.0, 2.5, 3.0],
  ),

  /// Ice. A shard that flies straight at the husk and freezes it where it
  /// stands, every seal in view open. The next blow shatters the ice.
  rime(
    vessels: {Vessel.penguin, Vessel.llama},
    speed: 14,
    homing: true,
    strike: [1.5, 1.5, 1.5],
  ),

  /// Wind. A wide gust that knocks a husk off whatever it stands on or out
  /// from behind whatever it hides behind, and stuns it there.
  gale(
    vessels: {Vessel.bird},
    speed: 12,
    homing: true,
    strike: [0.8, 1.0, 1.2],
  );

  const Art({
    required this.vessels,
    required this.speed,
    required this.homing,
    required this.strike,
  });

  /// The creatures that carry this art. Absorbing one teaches it.
  final Set<Vessel> vessels;

  /// How fast it flies, in metres a second.
  final double speed;

  /// Whether, thrown at a husk, it follows the husk in flight. Fire does
  /// not: it is lobbed at a place, and bursts there.
  final bool homing;

  /// Ward removed when it lands on the husk, in strikes of the reaper's own
  /// weapon, at each rank.
  final List<double> strike;

  /// The art a husk on [vessel] carries, or null for one that carries none.
  static Art? carriedBy(Vessel vessel) {
    for (final art in values) {
      if (art.vessels.contains(vessel)) {
        return art;
      }
    }

    return null;
  }

  /// How deep this art runs in a reaper that has absorbed [absorbed] husks
  /// carrying it: 0 when it has absorbed none, up to [ArtRules.maxRank].
  static int rankFor(int absorbed) {
    var rank = 0;

    for (final needed in ArtRules.absorbedForRank) {
      if (absorbed >= needed) {
        rank++;
      }
    }

    return rank;
  }
}

/// One art as a reaper knows it: which, and how deep.
final class ArtSkill {
  const ArtSkill(this.art, this.rank)
    : assert(rank >= 1 && rank <= ArtRules.maxRank);

  final Art art;

  /// From 1, when one husk carrying it has been absorbed, to
  /// [ArtRules.maxRank].
  final int rank;

  /// Ward removed when it lands, in strikes.
  double get strike => art.strike[rank - 1];

  /// The most throws a reaper can have ready at once.
  int get maxCharges => rank >= ArtRules.maxRank ? 3 : 2;

  /// Ticks until another throw is ready.
  int get rechargeTicks =>
      ArtRules.rechargeTicks - ArtRules.rechargeTicksPerRank * (rank - 1);

  /// How long a frozen husk stays frozen.
  int get freezeTicks =>
      ArtRules.freezeTicks + ArtRules.freezeTicksPerRank * (rank - 1);

  /// How much harder the blow that shatters a frozen husk lands.
  double get shatter => ArtRules.shatter + ArtRules.shatterPerRank * (rank - 1);

  /// How long a burning husk burns, and the ward the burning takes in all,
  /// in strikes.
  int get burnTicks => ArtRules.burnTicks;
  double get burnStrike =>
      ArtRules.burnStrike + ArtRules.burnStrikePerRank * (rank - 1);

  /// How long a husk the wind caught stands stunned.
  int get gustStunTicks =>
      ArtRules.gustStunTicks + ArtRules.gustStunTicksPerRank * (rank - 1);

  @override
  bool operator ==(Object other) =>
      other is ArtSkill && other.art == art && other.rank == rank;

  @override
  int get hashCode => Object.hash(art, rank);

  @override
  String toString() => 'ArtSkill(${art.name}, $rank)';
}

/// The fixed numbers of the arts. Playtest starting points, written down in
/// docs/gameplay.md.
abstract final class ArtRules {
  /// Husks carrying an art a reaper must have absorbed for each rank.
  static const absorbedForRank = [1, 4, 10];

  static const maxRank = 3;

  /// How long a thumb must rest on the art before a flick throws it.
  static const gatherTicks = 24;

  /// Turning faster than this, in radians a second, with the art gathered,
  /// throws it. A flick of the wrist, slower than the swing that turns a
  /// charge aside, so a hard flick does both.
  static const throwSpeed = 1.5;

  /// Throws ready when a fight begins.
  static const startingCharges = 1;

  /// Ticks for another throw to be ready, at the first rank, and how much
  /// sooner at each rank after.
  static const rechargeTicks = 9 * HuntRules.tickRate;
  static const rechargeTicksPerRank = HuntRules.tickRate;

  /// A thrown art lands at once on anything nearer than this many ticks.
  static const shortestFlightTicks = 8;

  /// The farthest an art thrown at nothing flies, in metres.
  static const farthestMetres = 8.0;

  /// Where an art leaves the reaper's hand, in metres from the eye: low and
  /// a little ahead.
  static const handDrop = 0.25;

  // Cinder ------------------------------------------------------------

  /// How far from where it bursts an ember catches a husk, in metres beyond
  /// the husk's own half width.
  static const burstMetres = 0.8;

  /// How high the ember's lob rises above the straight line, in metres,
  /// and how much more for every metre it is thrown.
  static const lobMetres = 0.5;
  static const lobPerMetre = 0.18;

  /// How long a husk burns.
  static const burnTicks = 150;

  /// The ward its burning takes in all, in strikes, at the first rank and
  /// for each rank after.
  static const burnStrike = 1.5;
  static const burnStrikePerRank = 0.5;

  /// A burning husk bolts this soon, and never into cover.
  static const burnBoltTicks = 20;

  // Rime --------------------------------------------------------------

  /// How long a frozen husk stays frozen, and how much longer each rank.
  static const freezeTicks = 150;
  static const freezeTicksPerRank = 40;

  /// The blow that shatters the ice lands this much harder, and more for
  /// each rank.
  static const shatter = 2.0;
  static const shatterPerRank = 0.25;

  // Gale --------------------------------------------------------------

  /// How far off the husk the gust may be aimed and still catch it.
  static final gustAngle = radians(22);

  /// How long a husk the gust caught stands stunned, and how much longer
  /// each rank.
  static const gustStunTicks = 90;
  static const gustStunTicksPerRank = 20;

  /// How long a husk the gust caught keeps away from cover after it comes
  /// round.
  static const shakenTicks = 4 * HuntRules.tickRate;
}
