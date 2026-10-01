import 'dart:math' as math;

import '../vec3.dart';
import '../vessel.dart';

/// Where a husk is and which way it is turned at one instant.
///
/// The rules own this, and the view draws exactly what it says: a model is
/// placed at [position] and turned to [heading]. Nothing about a hit is
/// decided by what was drawn.
final class HuskPose {
  const HuskPose({
    required this.vessel,
    required this.position,
    required this.heading,
  });

  final Vessel vessel;

  /// The middle of the husk's body, in metres from the player's eye.
  final Vec3 position;

  /// The way the husk's body faces, as a bearing in radians: 0 is the way
  /// the player first faced, and it grows to the right.
  final double heading;

  /// The husk's bearing around the player, in radians.
  double get yaw => position.yaw;

  /// The husk's elevation above the horizon, in radians.
  double get pitch => position.pitch;

  /// How far away the middle of the husk is, in metres.
  double get distance => position.length;

  /// The direction from the player to the husk's centre.
  Vec3 get direction => position.normalized;

  /// The way the husk's chest points.
  Vec3 get forward => Vec3(math.sin(heading), 0, math.cos(heading));

  /// The way the husk's left side points.
  Vec3 get left => Vec3(-math.cos(heading), 0, math.sin(heading));

  /// Whether the husk is turned toward the player, give or take a quarter
  /// turn.
  bool get facesPlayer => forward.dot(-direction) > 0;

  /// Whether [ray], a direction out from the player, passes through the
  /// husk's body, taken as an ellipse and widened by [pad] radians.
  ///
  /// A tap that covers the body but misses every seal is a glance.
  bool covers(Vec3 ray, {double pad = 0}) {
    final across =
        wrapAngle(ray.yaw - yaw) *
        math.cos(pitch) /
        (math.atan2(vessel.halfWidth, distance) + pad);
    final along =
        (ray.pitch - pitch) / (math.atan2(vessel.halfHeight, distance) + pad);

    return across * across + along * along <= 1;
  }

  /// Where seal [index] is, in metres from the player.
  Vec3 sealPosition(int index) => position + _sealOffset(index);

  /// The direction from the player to seal [index].
  Vec3 sealDirection(int index) => sealPosition(index).normalized;

  /// Whether seal [index] is on a side of the husk the player can see.
  ///
  /// A husk hides the seals on its far side, so one that turns its back
  /// shows different seals from one that faces the reaper.
  bool sealFacesPlayer(int index) {
    final outward = _sealOffset(index);

    return outward.dot(-direction) >= _facingShare * outward.length;
  }

  /// A seal is hidden only once it is well round the far side: more than
  /// about 105 degrees from the nearest point of the body. One on the very
  /// edge of the silhouette can still be seen, and struck.
  static const _facingShare = -0.25;

  /// Seal [index]'s place relative to the middle of the body, in the hunt's
  /// space.
  Vec3 _sealOffset(int index) {
    final offset = vessel.seals[index].offset;

    return left * offset.x + Vec3.up * offset.y + forward * offset.z;
  }
}
