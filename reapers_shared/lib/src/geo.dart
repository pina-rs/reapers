import 'dart:math' as math;

import 'vec3.dart';
import 'rift/rift_rules.dart';

/// A place on the earth, in degrees.
final class GeoPoint {
  const GeoPoint(this.latitude, this.longitude);

  /// Reads a point written by [toJson].
  factory GeoPoint.fromJson(Map<String, dynamic> json) => GeoPoint(
    (json['latitude'] as num).toDouble(),
    (json['longitude'] as num).toDouble(),
  );

  /// The earth's mean radius, in metres. The distances the game deals in
  /// are tens of metres, over which the earth is as good as a sphere.
  static const earthRadius = 6371008.8;

  /// Degrees north of the equator.
  final double latitude;

  /// Degrees east of Greenwich.
  final double longitude;

  /// Whether these are finite coordinates on the earth.
  bool get isValid =>
      latitude.isFinite &&
      longitude.isFinite &&
      latitude.abs() <= 90 &&
      longitude.abs() <= 180;

  /// How far it is to [other] over the ground, in metres.
  double distanceTo(GeoPoint other) {
    final fromLatitude = radians(latitude);
    final toLatitude = radians(other.latitude);
    final acrossLatitude = toLatitude - fromLatitude;
    final acrossLongitude = radians(other.longitude - longitude);
    final chord =
        math.pow(math.sin(acrossLatitude / 2), 2) +
        math.cos(fromLatitude) *
            math.cos(toLatitude) *
            math.pow(math.sin(acrossLongitude / 2), 2);

    return 2 * earthRadius * math.asin(math.min(1, math.sqrt(chord)));
  }

  /// Which way [other] lies from here: radians clockwise from north.
  double bearingTo(GeoPoint other) {
    final fromLatitude = radians(latitude);
    final toLatitude = radians(other.latitude);
    final acrossLongitude = radians(other.longitude - longitude);

    return wrapAngle(
      math.atan2(
        math.sin(acrossLongitude) * math.cos(toLatitude),
        math.cos(fromLatitude) * math.sin(toLatitude) -
            math.sin(fromLatitude) *
                math.cos(toLatitude) *
                math.cos(acrossLongitude),
      ),
    );
  }

  /// The place [metres] from here along [bearing], in radians clockwise
  /// from north.
  GeoPoint moved({required double bearing, required double metres}) {
    final arc = metres / earthRadius;
    final fromLatitude = radians(latitude);
    final toLatitude = math.asin(
      math.sin(fromLatitude) * math.cos(arc) +
          math.cos(fromLatitude) * math.sin(arc) * math.cos(bearing),
    );
    final acrossLongitude = math.atan2(
      math.sin(bearing) * math.sin(arc) * math.cos(fromLatitude),
      math.cos(arc) - math.sin(fromLatitude) * math.sin(toLatitude),
    );

    return GeoPoint(
      degrees(toLatitude),
      degrees(wrapAngle(radians(longitude) + acrossLongitude)),
    );
  }

  Map<String, dynamic> toJson() => {
    'latitude': latitude,
    'longitude': longitude,
  };

  @override
  bool operator ==(Object other) =>
      other is GeoPoint &&
      other.latitude == latitude &&
      other.longitude == longitude;

  @override
  int get hashCode => Object.hash(latitude, longitude);

  @override
  String toString() => 'GeoPoint($latitude, $longitude)';
}

/// One measured location. Times are supplied by the device; trust and
/// freshness are checked again by the server, with its own supplied time.
final class GeoFix {
  const GeoFix({
    required this.at,
    required this.accuracy,
    required this.observedAt,
  });

  factory GeoFix.fromJson(Map<String, dynamic> json) => GeoFix(
    at: GeoPoint.fromJson((json['at'] as Map).cast<String, dynamic>()),
    accuracy: (json['accuracy'] as num).toDouble(),
    observedAt: DateTime.parse(json['observedAt'] as String).toUtc(),
  );

  final GeoPoint at;
  final double accuracy;
  final DateTime observedAt;

  bool get isPrecise =>
      at.isValid &&
      accuracy.isFinite &&
      accuracy > 0 &&
      accuracy <= RiftRules.locationAccuracyMetres;

  bool isFreshAt(DateTime now) {
    final age = now.difference(observedAt);
    return age <= RiftRules.locationMaxAge &&
        age >= -RiftRules.locationClockSkew;
  }

  bool isUsableAt(DateTime now) => isPrecise && isFreshAt(now);

  GeoFix copyWith({GeoPoint? at, double? accuracy, DateTime? observedAt}) =>
      GeoFix(
        at: at ?? this.at,
        accuracy: accuracy ?? this.accuracy,
        observedAt: observedAt ?? this.observedAt,
      );

  Map<String, dynamic> toJson() => {
    'at': at.toJson(),
    'accuracy': accuracy,
    'observedAt': observedAt.toUtc().toIso8601String(),
  };
}

/// Where a reaper is standing and which way their phone points, as the phone
/// itself measured it.
final class GeoStanding {
  const GeoStanding({
    required this.at,
    required this.facing,
    required this.accuracy,
    this.observedAt,
  });

  /// Reads a standing written by [toJson].
  factory GeoStanding.fromJson(Map<String, dynamic> json) => GeoStanding(
    at: GeoPoint.fromJson((json['at'] as Map).cast<String, dynamic>()),
    facing: (json['facing'] as num).toDouble(),
    accuracy: (json['accuracy'] as num).toDouble(),
    observedAt: json['observedAt'] == null
        ? null
        : DateTime.parse(json['observedAt'] as String).toUtc(),
  );

  final GeoPoint at;

  /// The way the phone points, in radians clockwise from north by its
  /// compass.
  final double facing;

  /// How far out [at] may be, in metres, by the phone's own reckoning.
  final double accuracy;

  /// Older recordings may omit this. Live servers require a timestamp.
  final DateTime? observedAt;

  bool isUsableAt(DateTime now) =>
      facing.isFinite &&
      observedAt != null &&
      GeoFix(
        at: at,
        accuracy: accuracy,
        observedAt: observedAt!,
      ).isUsableAt(now);

  Map<String, dynamic> toJson() => {
    'at': at.toJson(),
    'facing': facing,
    'accuracy': accuracy,
    if (observedAt != null) 'observedAt': observedAt!.toUtc().toIso8601String(),
  };
}
