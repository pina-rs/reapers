import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

/// Where the phone is pointing and how fast it is turning.
///
/// The three directions are in the hunt's space (x right, y up, z ahead of
/// where the player first faced) and always at right angles to each other.
final class ArPose {
  const ArPose({
    required this.forward,
    required this.up,
    required this.right,
    required this.turnSpeed,
  });

  /// A level view along [forward]: the phone held upright, not rolled.
  factory ArPose.looking(Vec3 forward, {double turnSpeed = 0}) {
    final yaw = forward.yaw;
    final pitch = forward.pitch;

    return ArPose(
      forward: Vec3.fromAngles(yaw, pitch),
      up: Vec3.fromAngles(yaw, pitch + math.pi / 2),
      right: Vec3(math.cos(yaw), 0, -math.sin(yaw)),
      turnSpeed: turnSpeed,
    );
  }

  /// Looking straight ahead and level, perfectly still.
  static const ahead = ArPose(
    forward: Vec3.forward,
    up: Vec3.up,
    right: Vec3(1, 0, 0),
    turnSpeed: 0,
  );

  /// Out through the back of the phone: the middle of the view.
  final Vec3 forward;

  /// Toward the top edge of the phone.
  final Vec3 up;

  /// Toward the right edge of the phone.
  final Vec3 right;

  /// How fast the phone is turning, in radians a second.
  final double turnSpeed;

  /// The direction, out from the player, of a touch on the screen.
  ///
  /// [x] and [y] run from -1 to 1 across the view, with y up. [verticalFov]
  /// is the full vertical field of view in radians and [aspect] is the
  /// view's width over its height. This is the inverse of the projection
  /// the 3D view draws with, so a tap lands exactly on what was drawn there.
  Vec3 rayThrough({
    required double x,
    required double y,
    required double verticalFov,
    required double aspect,
  }) {
    final halfHeight = math.tan(verticalFov / 2);

    return (forward + right * (x * halfHeight * aspect) + up * (y * halfHeight))
        .normalized;
  }

  /// Where [direction] appears on the screen, as x and y from -1 to 1 with y
  /// up, or null when it is behind the player.
  ({double x, double y})? project(
    Vec3 direction, {
    required double verticalFov,
    required double aspect,
  }) {
    final depth = direction.dot(forward);

    if (depth <= 1e-6) {
      return null;
    }

    final halfHeight = math.tan(verticalFov / 2);

    return (
      x: direction.dot(right) / depth / (halfHeight * aspect),
      y: direction.dot(up) / depth / halfHeight,
    );
  }
}

/// Something that knows where the phone is pointing.
///
/// A phone with motion sensors answers from them. Anything else, such as a
/// simulator or a desk, answers from a finger dragged across the screen, so
/// the hunt can be played everywhere it can be drawn.
abstract interface class PoseSource {
  /// The pose right now.
  ArPose get pose;

  /// Whether the pose follows the phone itself rather than a drag.
  bool get followsDevice;

  /// Advances the source by [deltaSeconds]. Called once a frame.
  void advance(double deltaSeconds);

  /// Turns the view by a drag of ([dx], [dy]) radians. Only a source that
  /// does not follow the device responds.
  void drag(double dx, double dy);

  void dispose();
}
