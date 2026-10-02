import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'ar_pose.dart';

/// Works out which way a phone is pointing from its gyroscope, corrected by
/// its accelerometer.
///
/// The gyroscope is fast and smooth but drifts; the accelerometer always
/// knows which way is down but is noisy. Each gyroscope sample turns the
/// estimate, and each steady accelerometer sample pulls its sense of "down"
/// a little back toward the truth. Heading is never corrected: it only has
/// to stay put for the length of one hunt, and a compass would bring more
/// noise than it removes.
///
/// A compass is still listened to, for one thing: where north lies in the
/// view. That is what lets something be drawn at a real place on the earth,
/// where everyone standing nearby sees it in the same direction. It is kept
/// beside the pose, never mixed into it.
///
/// Readings use the sensor convention shared by Android and `sensors_plus`:
/// x to the right of the screen, y toward its top, z out toward the player,
/// with a resting accelerometer reading pointing up.
final class AttitudeFilter {
  /// How strongly one accelerometer sample corrects the estimate.
  static const _gravityGain = 0.02;

  /// Accelerometer samples further than this from one g are the player
  /// moving the phone, not gravity, and are ignored.
  static const _gravityTolerance = 0.18;

  static const _standardGravity = 9.80665;

  /// How quickly the reported turn speed follows the gyroscope, as the time
  /// in seconds to cover most of a change.
  static const _speedSmoothing = 0.07;

  /// How strongly one compass sample moves the sense of where north is. A
  /// compass jumps about, and north does not move, so each sample counts
  /// for little.
  static const _northGain = 0.03;

  /// A field pointing this nearly straight up or down has no heading in it
  /// worth reading.
  static const _flatFieldShare = 0.2;

  static const _up = Vec3.up;

  /// Device to world. The world's y axis is up; its heading is wherever the
  /// phone pointed when the first accelerometer sample arrived.
  _Rotation _attitude = _Rotation.identity;
  bool _levelled = false;
  double _turnSpeed = 0;
  double? _north;

  /// Whether enough has been heard from the sensors to trust [pose].
  bool get isReady => _levelled;

  /// Feeds one gyroscope sample, in radians a second, taken [deltaSeconds]
  /// after the last.
  void addRotation(double x, double y, double z, double deltaSeconds) {
    final speed = math.sqrt(x * x + y * y + z * z);
    final blend = 1 - math.exp(-deltaSeconds / _speedSmoothing);

    _turnSpeed += (speed - _turnSpeed) * blend;

    if (speed < 1e-9) {
      return;
    }

    // The gyroscope measures in the phone's own frame, so its turn is
    // applied before the attitude so far.
    final turn = _Rotation.about(Vec3(x, y, z) / speed, speed * deltaSeconds);

    _attitude = (_attitude * turn).normalized;
  }

  /// Feeds one accelerometer sample, in metres a second squared.
  void addAcceleration(double x, double y, double z) {
    final size = math.sqrt(x * x + y * y + z * z);
    final steady = (size / _standardGravity - 1).abs() <= _gravityTolerance;

    if (!steady) {
      return;
    }

    final measuredUp = Vec3(x, y, z) / size;

    if (!_levelled) {
      // Start level with gravity and facing wherever the phone points.
      _attitude = _Rotation.between(measuredUp, _up);
      _levelled = true;
      return;
    }

    final believedUp = _attitude.rotate(measuredUp);
    final correction = _Rotation.between(believedUp, _up, share: _gravityGain);

    _attitude = (correction * _attitude).normalized;
  }

  /// Feeds one magnetometer sample. Only its direction is used, so the
  /// unit does not matter.
  void addMagneticField(double x, double y, double z) {
    final size = math.sqrt(x * x + y * y + z * z);

    if (!_levelled || size < 1e-9) {
      return;
    }

    final field = _toHunt(_attitude.rotate(Vec3(x, y, z) / size));
    final flat = math.sqrt(field.x * field.x + field.z * field.z);

    if (flat < _flatFieldShare) {
      return;
    }

    final measured = math.atan2(field.x, field.z);
    final north = _north;

    _north = north == null
        ? measured
        : wrapAngle(north + wrapAngle(measured - north) * _northGain);
  }

  /// Where magnetic north lies in the hunt's space, as a bearing in radians
  /// to the right of where the view first faced. Null until the compass has
  /// been heard from.
  double? get north => _north;

  /// The current pose, in the hunt's space.
  ArPose get pose => ArPose(
    forward: _toHunt(_attitude.rotate(const Vec3(0, 0, -1))),
    up: _toHunt(_attitude.rotate(_up)),
    right: _toHunt(_attitude.rotate(const Vec3(1, 0, 0))),
    turnSpeed: _turnSpeed,
  );

  /// The sensors' world has z toward the player; the hunt's has z ahead.
  static Vec3 _toHunt(Vec3 world) => Vec3(world.x, world.y, -world.z);
}

/// A rotation as a unit quaternion, with the textbook conventions: a
/// positive angle turns anticlockwise about its axis, and `a * b` applies
/// `b` first.
///
/// The filter keeps its own because the sign conventions of a rotation are
/// exactly what it must not get wrong, and they are easier to trust when
/// they are written down next to the code that depends on them.
final class _Rotation {
  const _Rotation(this.w, this.x, this.y, this.z);

  /// A turn of [angle] radians about [axis], which must have length 1.
  factory _Rotation.about(Vec3 axis, double angle) {
    final sine = math.sin(angle / 2);

    return _Rotation(
      math.cos(angle / 2),
      axis.x * sine,
      axis.y * sine,
      axis.z * sine,
    );
  }

  /// The shortest turn that carries [from] onto [to], or the first [share]
  /// of that turn. Both must have length 1.
  factory _Rotation.between(Vec3 from, Vec3 to, {double share = 1}) {
    final angle = from.angleTo(to);
    final axis = Vec3(
      from.y * to.z - from.z * to.y,
      from.z * to.x - from.x * to.z,
      from.x * to.y - from.y * to.x,
    );

    if (axis.length < 1e-9) {
      // Already aligned, or exactly opposed. Opposed has no single shortest
      // turn, so pick the one about the x axis.
      return angle < 1
          ? identity
          : _Rotation.about(const Vec3(1, 0, 0), math.pi * share);
    }

    return _Rotation.about(axis / axis.length, angle * share);
  }

  static const identity = _Rotation(1, 0, 0, 0);

  final double w;
  final double x;
  final double y;
  final double z;

  _Rotation get normalized {
    final size = math.sqrt(w * w + x * x + y * y + z * z);

    return _Rotation(w / size, x / size, y / size, z / size);
  }

  _Rotation operator *(_Rotation other) => _Rotation(
    w * other.w - x * other.x - y * other.y - z * other.z,
    w * other.x + x * other.w + y * other.z - z * other.y,
    w * other.y - x * other.z + y * other.w + z * other.x,
    w * other.z + x * other.y - y * other.x + z * other.w,
  );

  /// Turns [vector] by this rotation.
  Vec3 rotate(Vec3 vector) {
    // v + 2w(q x v) + 2 q x (q x v), with q the vector part.
    final cx = y * vector.z - z * vector.y;
    final cy = z * vector.x - x * vector.z;
    final cz = x * vector.y - y * vector.x;

    return Vec3(
      vector.x + 2 * (w * cx + y * cz - z * cy),
      vector.y + 2 * (w * cy + z * cx - x * cz),
      vector.z + 2 * (w * cz + x * cy - y * cx),
    );
  }
}
