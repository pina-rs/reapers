import 'dart:math' as math;

import '../vec3.dart';
import '../vessel.dart';

/// Where a husk is and how it is turned at one instant.
///
/// The rules own this, and the view draws exactly what it says: a model is
/// placed at [centre], turned to face the player, then turned again by
/// [spin]. Nothing about a hit is decided by what was drawn.
final class HuskPose {
  const HuskPose({
    required this.vessel,
    required this.yaw,
    required this.pitch,
    required this.spin,
    required this.orbit,
  });

  final Vessel vessel;

  /// The husk's bearing around the player, in radians.
  final double yaw;

  /// The husk's elevation above the horizon, in radians.
  final double pitch;

  /// How far the body has turned about its upright axis, in radians.
  final double spin;

  /// How far orbiting seals have travelled around the centre, in radians.
  final double orbit;

  /// The direction from the player to the husk's centre.
  Vec3 get direction => Vec3.fromAngles(yaw, pitch);

  /// The husk's centre, in metres from the player.
  Vec3 get centre => direction * vessel.distance;

  /// Whether [ray], a direction out from the player, passes through the
  /// husk's body, taken as an ellipse and widened by [pad] radians.
  ///
  /// A tap that covers the body but misses every seal is a glance.
  bool covers(Vec3 ray, {double pad = 0}) {
    final across =
        wrapAngle(ray.yaw - yaw) *
        math.cos(pitch) /
        (math.atan2(vessel.halfWidth, vessel.distance) + pad);
    final along =
        (ray.pitch - pitch) /
        (math.atan2(vessel.halfHeight, vessel.distance) + pad);

    return across * across + along * along <= 1;
  }

  /// Where seal [index] is, in metres from the player.
  Vec3 sealPosition(int index) {
    final local = _sealLocal(index);
    final right = Vec3(math.cos(yaw), 0, -math.sin(yaw));
    final towardPlayer = Vec3(-math.sin(yaw), 0, -math.cos(yaw));

    return centre +
        right * local.x +
        Vec3.up * local.y +
        towardPlayer * local.z;
  }

  /// The direction from the player to seal [index].
  Vec3 sealDirection(int index) => sealPosition(index).normalized;

  /// Whether seal [index] is turned toward the player.
  ///
  /// Only a turning body hides its seals: one that has come round to the
  /// far side, or sits edge-on, cannot be struck.
  bool sealFacesPlayer(int index) {
    if (vessel.spinDegreesPerSecond == 0) {
      return true;
    }

    final local = _sealLocal(index);
    final radius = math.sqrt(local.x * local.x + local.z * local.z);

    return local.z >= _facingShare * radius;
  }

  /// A seal counts as facing the player while it sits within about 70
  /// degrees of the nearest point of the body.
  static const _facingShare = 0.34;

  /// The seal's place in the husk's own frame after spin and orbit.
  Vec3 _sealLocal(int index) {
    final seal = vessel.seals[index];
    var offset = seal.offset;

    if (seal.orbitRadius > 0) {
      final angle = orbit + seal.orbitPhase;

      offset = Vec3(
        math.cos(angle) * seal.orbitRadius,
        math.sin(angle) * seal.orbitRadius,
        0.06,
      );
    }

    if (spin == 0) {
      return offset;
    }

    final cosine = math.cos(spin);
    final sine = math.sin(spin);

    return Vec3(
      offset.x * cosine + offset.z * sine,
      offset.y,
      offset.z * cosine - offset.x * sine,
    );
  }
}
