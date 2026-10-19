import 'dart:math' as math;

import '../vec3.dart';
import 'hunt_stats.dart';

/// Where a husk may go: what the phone made of the room before the hunt.
///
/// Before a hunt the phone reads the surroundings while the player looks
/// around. What it has read becomes the plan, and the husk appears inside it
/// and never leaves it, so it is never somewhere the phone knows nothing
/// about. A plan says three things:
///
/// - an arc around the player that the husk keeps to;
/// - how far below the player's eye the floor is;
/// - how far the way is clear in each direction, when the phone could tell;
/// - what stands at the end of that way: how tall it is, whether there is
///   room to hide behind it, and whether its top is level enough to stand
///   on.
///
/// A plan never says a hunt cannot be played. Where the room is tight the
/// husk stands closer; it does not stay away.
///
/// Everything is stored in whole numbers so that a hunt replays exactly.
final class HuntPlan {
  const HuntPlan({
    required this.centreCentidegrees,
    required this.halfWidthCentidegrees,
    this.eyeCentimetres = HuntRules.eyeCentimetres,
    this.reachDecimetres = const [],
    this.coverCentimetres = const [],
    this.hideDecimetres = const [],
    this.ledges = const [],
  });

  /// The plan covering the arc from [fromRadians] round to the right as far
  /// as [toRadians].
  factory HuntPlan.between(
    double fromRadians,
    double toRadians, {
    int eyeCentimetres = HuntRules.eyeCentimetres,
    List<int> reachDecimetres = const [],
    List<int> coverCentimetres = const [],
    List<int> hideDecimetres = const [],
    List<bool> ledges = const [],
  }) {
    var width = (toRadians - fromRadians) % (2 * math.pi);

    if (width == 0 && toRadians != fromRadians) {
      width = 2 * math.pi;
    }

    return HuntPlan(
      centreCentidegrees: (degrees(wrapAngle(fromRadians + width / 2)) * 100)
          .round(),
      halfWidthCentidegrees: (degrees(width / 2) * 100).round().clamp(0, 18000),
      eyeCentimetres: eyeCentimetres,
      reachDecimetres: reachDecimetres,
      coverCentimetres: coverCentimetres,
      hideDecimetres: hideDecimetres,
      ledges: ledges,
    );
  }

  /// The whole circle, with nothing known about the room: for a husk that
  /// may go anywhere, as in practice against the rules alone.
  static const open = HuntPlan(
    centreCentidegrees: 0,
    halfWidthCentidegrees: 18000,
  );

  /// The narrowest plan a hunt may be played in: a hundred degrees across.
  /// A narrower one would pen the husk in where it is too easy to follow.
  static const minHalfWidthCentidegrees = 5000;

  /// The directions [reachDecimetres] is measured in: one every five
  /// degrees, all the way round.
  static const slices = 72;

  /// The width of one slice, in radians.
  static const sliceWidth = 2 * math.pi / slices;

  /// The lowest and highest a phone can plausibly be held above a floor.
  static const minEyeCentimetres = 50;
  static const maxEyeCentimetres = 220;

  /// The farthest a reach is recorded: what fits in a byte.
  static const maxReachDecimetres = 255;

  /// The tallest cover is recorded as: anything taller is as good as a wall.
  static const maxCoverCentimetres = 255;

  /// The middle of the arc, to the right of where the player first faced.
  final int centreCentidegrees;

  /// Half the arc's width. 18000 is the whole circle.
  final int halfWidthCentidegrees;

  /// How far the floor is below the player's eye.
  final int eyeCentimetres;

  /// How far the way is clear along each slice, from the player to the
  /// first thing standing in it, or to the end of what the phone saw.
  ///
  /// Empty when the phone could not tell, and the husk is then free to keep
  /// whatever distance it likes. Otherwise there is one entry for each of
  /// the [slices], starting directly behind the player and going round to
  /// the right.
  final List<int> reachDecimetres;

  /// How tall the thing standing at the end of each slice's reach is, in
  /// centimetres above the floor: a counter, a sofa, a table. Zero where the
  /// slice ends because the floor that was seen ends, not at anything.
  ///
  /// Empty when the phone could not tell. Otherwise there is one entry for
  /// each slice, in the order of [reachDecimetres].
  final List<int> coverCentimetres;

  /// Where a husk can hide behind the cover along each slice, in decimetres
  /// from the player: in the gap between its far side and whatever stands
  /// behind it. Zero where there is no room behind it.
  final List<int> hideDecimetres;

  /// Whether the top of the cover along each slice is level enough to stand
  /// on: a table or a counter, not the back of a chair.
  final List<bool> ledges;

  double get centre => radians(centreCentidegrees / 100);

  double get halfWidth => radians(halfWidthCentidegrees / 100);

  /// How far the floor is below the player's eye, in metres.
  double get eyeHeight => eyeCentimetres / 100;

  bool get isWholeCircle => halfWidthCentidegrees >= 18000;

  /// Whether the phone measured how far the way is clear.
  bool get isSurveyed => reachDecimetres.isNotEmpty;

  /// Whether the phone also measured what stands in the way.
  bool get knowsCover => coverCentimetres.isNotEmpty;

  /// The slice the bearing [yaw] falls in.
  static int sliceOf(double yaw) =>
      ((wrapAngle(yaw) + math.pi) / sliceWidth).floor().clamp(0, slices - 1);

  /// The bearing of the middle of slice [index].
  static double bearingOf(int index) =>
      wrapAngle(index * sliceWidth - math.pi + sliceWidth / 2);

  /// Whether the plan is one a phone could have made.
  bool get isWellFormed =>
      halfWidthCentidegrees >= minHalfWidthCentidegrees &&
      halfWidthCentidegrees <= 18000 &&
      centreCentidegrees >= -18000 &&
      centreCentidegrees <= 18000 &&
      eyeCentimetres >= minEyeCentimetres &&
      eyeCentimetres <= maxEyeCentimetres &&
      (reachDecimetres.isEmpty || reachDecimetres.length == slices) &&
      reachDecimetres.every(
        (reach) => reach >= 0 && reach <= maxReachDecimetres,
      ) &&
      _coverIsWellFormed;

  /// Cover is all there or not there at all, and only ever with a reach to
  /// stand at the end of. A husk hides behind cover, never in front of it.
  bool get _coverIsWellFormed {
    if (coverCentimetres.isEmpty) {
      return hideDecimetres.isEmpty && ledges.isEmpty;
    }

    if (!isSurveyed ||
        coverCentimetres.length != slices ||
        hideDecimetres.length != slices ||
        ledges.length != slices) {
      return false;
    }

    for (var slice = 0; slice < slices; slice++) {
      final cover = coverCentimetres[slice];
      final hide = hideDecimetres[slice];

      if (cover < 0 ||
          cover > maxCoverCentimetres ||
          hide < 0 ||
          hide > maxReachDecimetres ||
          (hide != 0 && (cover == 0 || hide <= reachDecimetres[slice]))) {
        return false;
      }
    }

    return true;
  }

  /// Whether the bearing [yaw] lies inside the plan.
  bool contains(double yaw) =>
      isWholeCircle || wrapAngle(yaw - centre).abs() <= halfWidth + _edge;

  /// A bearing brought to the edge by [clamp] lands on it to within the last
  /// bit of a double, and still counts as inside.
  static const _edge = 1e-9;

  /// [yaw], or the nearest bearing to it that lies inside the plan.
  double clamp(double yaw) {
    if (isWholeCircle) {
      return wrapAngle(yaw);
    }

    final offset = wrapAngle(yaw - centre).clamp(-halfWidth, halfWidth);

    return wrapAngle(centre + offset);
  }

  /// How far a husk may go along the bearing [yaw], in metres, or infinity
  /// when the phone could not tell.
  ///
  /// That is as far as the way is clear there, and no farther than it is
  /// clear nearby with [HuntRules.reachEase] added for every radian between:
  /// a husk coming up on a sofa starts to draw in before it reaches it, and
  /// is round the front of it by the time it gets there.
  double reach(double yaw) {
    var least = double.infinity;

    for (var slice = 0; slice < reachDecimetres.length; slice++) {
      final away = math.max(
        0,
        wrapAngle(yaw - bearingOf(slice)).abs() - sliceWidth / 2,
      );

      least = math.min(
        least,
        reachDecimetres[slice] / 10 + HuntRules.reachEase * away,
      );
    }

    return least;
  }

  /// How tall the cover along the bearing [yaw] is, in metres above the
  /// floor, or zero where nothing stands at the end of the way.
  double coverAt(double yaw) =>
      knowsCover ? coverCentimetres[sliceOf(yaw)] / 100 : 0;

  /// Whether the line from the player's eye to [point], in metres from the
  /// eye, passes through the cover standing along its bearing: whether a
  /// counter, a sofa or a table is in front of it.
  bool hides(Vec3 point) {
    if (!knowsCover) {
      return false;
    }

    final slice = sliceOf(point.yaw);
    final cover = coverCentimetres[slice] / 100;

    if (cover == 0) {
      return false;
    }

    final front = reachDecimetres[slice] / 10;
    final away = math.sqrt(point.x * point.x + point.z * point.z);

    if (away <= front) {
      return false;
    }

    // Where the line of sight crosses the front of the cover, and the top
    // of the cover, both measured from the eye.
    final crossing = point.y * front / away;

    return crossing < cover - eyeHeight;
  }

  /// How far out along slice [index] a husk can hide behind the cover
  /// there, in metres, or null when there is no room behind it.
  double? hideAt(int index) {
    if (!knowsCover || hideDecimetres[index] == 0) {
      return null;
    }

    return hideDecimetres[index] / 10;
  }

  /// How high above the floor the level top of the cover along slice
  /// [index] is, in metres, or null when there is nothing there to stand on.
  double? ledgeAt(int index) {
    if (!knowsCover || !ledges[index] || coverCentimetres[index] == 0) {
      return null;
    }

    return coverCentimetres[index] / 100;
  }

  @override
  bool operator ==(Object other) =>
      other is HuntPlan &&
      other.centreCentidegrees == centreCentidegrees &&
      other.halfWidthCentidegrees == halfWidthCentidegrees &&
      other.eyeCentimetres == eyeCentimetres &&
      _same(other.reachDecimetres, reachDecimetres) &&
      _same(other.coverCentimetres, coverCentimetres) &&
      _same(other.hideDecimetres, hideDecimetres) &&
      _same(other.ledges, ledges);

  @override
  int get hashCode => Object.hash(
    centreCentidegrees,
    halfWidthCentidegrees,
    eyeCentimetres,
    Object.hashAll(reachDecimetres),
    Object.hashAll(coverCentimetres),
    Object.hashAll(hideDecimetres),
    Object.hashAll(ledges),
  );

  @override
  String toString() =>
      'HuntPlan($centreCentidegrees ± $halfWidthCentidegrees, '
      'eye $eyeCentimetres cm, '
      '${isSurveyed ? 'surveyed' : 'not surveyed'}'
      '${knowsCover ? ', with cover' : ''})';

  static bool _same<T>(List<T> one, List<T> other) {
    if (one.length != other.length) {
      return false;
    }

    for (var index = 0; index < one.length; index++) {
      if (one[index] != other[index]) {
        return false;
      }
    }

    return true;
  }
}
