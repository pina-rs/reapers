import 'dart:math' as math;

import 'vec3.dart';

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

/// Where a reaper is standing and which way their phone points, as the phone
/// itself measured it.
final class GeoStanding {
  const GeoStanding({
    required this.at,
    required this.facing,
    required this.accuracy,
  });

  /// Reads a standing written by [toJson].
  factory GeoStanding.fromJson(Map<String, dynamic> json) => GeoStanding(
    at: GeoPoint.fromJson((json['at'] as Map).cast<String, dynamic>()),
    facing: (json['facing'] as num).toDouble(),
    accuracy: (json['accuracy'] as num).toDouble(),
  );

  final GeoPoint at;

  /// The way the phone points, in radians clockwise from north by its
  /// compass.
  final double facing;

  /// How far out [at] may be, in metres, by the phone's own reckoning.
  final double accuracy;

  Map<String, dynamic> toJson() => {
    'at': at.toJson(),
    'facing': facing,
    'accuracy': accuracy,
  };
}
