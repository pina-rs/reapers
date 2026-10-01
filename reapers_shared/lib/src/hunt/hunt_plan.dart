import 'dart:math' as math;

import '../vec3.dart';

/// Where a husk may go: the part of the room the reaper has looked over.
///
/// Before a hunt the phone reads the surroundings while the player looks
/// around. What it has read becomes the plan, and the husk appears inside it
/// and never leaves it, so it is never somewhere the phone knows nothing
/// about. A plan is an arc around the player, stored in whole hundredths of
/// a degree so that it replays exactly.
final class HuntPlan {
  const HuntPlan({
    required this.centreCentidegrees,
    required this.halfWidthCentidegrees,
  });

  /// The plan covering the arc from [fromRadians] round to the right as far
  /// as [toRadians].
  factory HuntPlan.between(double fromRadians, double toRadians) {
    var width = (toRadians - fromRadians) % (2 * math.pi);

    if (width == 0 && toRadians != fromRadians) {
      width = 2 * math.pi;
    }

    return HuntPlan(
      centreCentidegrees: (degrees(wrapAngle(fromRadians + width / 2)) * 100)
          .round(),
      halfWidthCentidegrees: (degrees(width / 2) * 100).round().clamp(0, 18000),
    );
  }

  /// The whole circle: for a husk that may go anywhere, as in practice
  /// against the rules alone, with no room to read.
  static const open = HuntPlan(
    centreCentidegrees: 0,
    halfWidthCentidegrees: 18000,
  );

  /// The narrowest plan a hunt may be played in: a hundred degrees across.
  /// A narrower one would pen the husk in where it is too easy to follow.
  static const minHalfWidthCentidegrees = 5000;

  /// The middle of the arc, to the right of where the player first faced.
  final int centreCentidegrees;

  /// Half the arc's width. 18000 is the whole circle.
  final int halfWidthCentidegrees;

  double get centre => radians(centreCentidegrees / 100);

  double get halfWidth => radians(halfWidthCentidegrees / 100);

  bool get isWholeCircle => halfWidthCentidegrees >= 18000;

  /// Whether a hunt may be played in this plan.
  bool get isPlayable =>
      halfWidthCentidegrees >= minHalfWidthCentidegrees &&
      halfWidthCentidegrees <= 18000 &&
      centreCentidegrees >= -18000 &&
      centreCentidegrees <= 18000;

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

  @override
  bool operator ==(Object other) =>
      other is HuntPlan &&
      other.centreCentidegrees == centreCentidegrees &&
      other.halfWidthCentidegrees == halfWidthCentidegrees;

  @override
  int get hashCode => Object.hash(centreCentidegrees, halfWidthCentidegrees);

  @override
  String toString() => 'HuntPlan($centreCentidegrees ± $halfWidthCentidegrees)';
}
