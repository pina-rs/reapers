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
/// - how far the way is clear in each direction, when the phone could tell.
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
  });

  /// The plan covering the arc from [fromRadians] round to the right as far
  /// as [toRadians].
  factory HuntPlan.between(
    double fromRadians,
    double toRadians, {
    int eyeCentimetres = HuntRules.eyeCentimetres,
    List<int> reachDecimetres = const [],
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

  double get centre => radians(centreCentidegrees / 100);

  double get halfWidth => radians(halfWidthCentidegrees / 100);

  /// How far the floor is below the player's eye, in metres.
  double get eyeHeight => eyeCentimetres / 100;

  bool get isWholeCircle => halfWidthCentidegrees >= 18000;

  /// Whether the phone measured how far the way is clear.
  bool get isSurveyed => reachDecimetres.isNotEmpty;

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
      );

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

  @override
  bool operator ==(Object other) =>
      other is HuntPlan &&
      other.centreCentidegrees == centreCentidegrees &&
      other.halfWidthCentidegrees == halfWidthCentidegrees &&
      other.eyeCentimetres == eyeCentimetres &&
      _sameReach(other.reachDecimetres, reachDecimetres);

  @override
  int get hashCode => Object.hash(
    centreCentidegrees,
    halfWidthCentidegrees,
    eyeCentimetres,
    Object.hashAll(reachDecimetres),
  );

  @override
  String toString() =>
      'HuntPlan($centreCentidegrees ± $halfWidthCentidegrees, '
      'eye $eyeCentimetres cm, '
      '${isSurveyed ? 'surveyed' : 'not surveyed'})';

  static bool _sameReach(List<int> one, List<int> other) {
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
