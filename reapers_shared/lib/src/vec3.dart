import 'dart:math' as math;

/// An immutable 3D vector in the hunt's space: `x` to the player's right,
/// `y` up and `z` straight ahead of where the player first faced.
///
/// The rules use their own small vector so that they depend on nothing but
/// `dart:math` and stay in double precision on every platform.
final class Vec3 {
  const Vec3(this.x, this.y, this.z);

  /// The unit vector that points [yaw] radians to the right of straight
  /// ahead and [pitch] radians above the horizon.
  factory Vec3.fromAngles(double yaw, double pitch) {
    final horizontal = math.cos(pitch);

    return Vec3(
      math.sin(yaw) * horizontal,
      math.sin(pitch),
      math.cos(yaw) * horizontal,
    );
  }

  static const zero = Vec3(0, 0, 0);
  static const up = Vec3(0, 1, 0);
  static const forward = Vec3(0, 0, 1);

  final double x;
  final double y;
  final double z;

  double get length => math.sqrt(x * x + y * y + z * z);

  /// This vector scaled to length 1, or [forward] when it has no length.
  Vec3 get normalized {
    final size = length;

    return size == 0 ? forward : this / size;
  }

  /// How far to the right of straight ahead this vector points, in radians.
  double get yaw => math.atan2(x, z);

  /// How far above the horizon this vector points, in radians.
  double get pitch => math.atan2(y, math.sqrt(x * x + z * z));

  Vec3 operator +(Vec3 other) => Vec3(x + other.x, y + other.y, z + other.z);

  Vec3 operator -(Vec3 other) => Vec3(x - other.x, y - other.y, z - other.z);

  Vec3 operator *(double scale) => Vec3(x * scale, y * scale, z * scale);

  Vec3 operator /(double scale) => Vec3(x / scale, y / scale, z / scale);

  Vec3 operator -() => Vec3(-x, -y, -z);

  double dot(Vec3 other) => x * other.x + y * other.y + z * other.z;

  /// The vector at right angles to both this and [other].
  Vec3 cross(Vec3 other) => Vec3(
    y * other.z - z * other.y,
    z * other.x - x * other.z,
    x * other.y - y * other.x,
  );

  /// The angle between this vector and [other], in radians.
  double angleTo(Vec3 other) {
    final cosine = normalized.dot(other.normalized);

    return math.acos(cosine.clamp(-1.0, 1.0));
  }

  @override
  bool operator ==(Object other) =>
      other is Vec3 && other.x == x && other.y == y && other.z == z;

  @override
  int get hashCode => Object.hash(x, y, z);

  @override
  String toString() => 'Vec3($x, $y, $z)';
}

/// Wraps [radians] into `(-pi, pi]`.
double wrapAngle(double radians) {
  var wrapped = radians % (2 * math.pi);

  if (wrapped > math.pi) {
    wrapped -= 2 * math.pi;
  }

  return wrapped;
}

/// Degrees to radians.
double radians(double degrees) => degrees * math.pi / 180;

/// Radians to degrees.
double degrees(double radians) => radians * 180 / math.pi;

/// Hermite easing from 0 to 1 as [t] goes from 0 to 1.
double smoothstep(double t) {
  final clamped = t.clamp(0.0, 1.0);

  return clamped * clamped * (3 - 2 * clamped);
}
