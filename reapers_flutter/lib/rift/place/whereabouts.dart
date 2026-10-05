import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:geolocator/geolocator.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'whereabouts.g.dart';

enum LocationIssue { denied, blocked, services, unsupported, unavailable }

/// A permission or device condition the player can recover from.
final class LocationFailure implements Exception {
  const LocationFailure(this.issue);
  final LocationIssue issue;
}

/// Whatever can say where the phone is.
abstract interface class Whereabouts {
  /// Where the phone is, each time it has a better or newer idea, for as
  /// long as anyone listens.
  ///
  /// Permission and service failures use [LocationFailure]. A rift remains
  /// playable without location, with its husk standing dead ahead.
  Stream<GeoFix> watch();
}

/// What says where the phone is. The app hands in the real thing at
/// startup; without it nothing is known, which is what every test wants.
@Riverpod(keepAlive: true)
Whereabouts whereabouts(Ref ref) => const Nowhere();

/// A phone that never knows where it is.
final class Nowhere implements Whereabouts {
  const Nowhere();

  @override
  Stream<GeoFix> watch() => const Stream.empty();
}

/// Asks the phone itself: satellites outdoors, and whatever else it has.
///
/// Listening asks the player for leave to use their location if they have
/// not been asked before. Refusing is an answer, not an error.
final class DeviceWhereabouts implements Whereabouts {
  const DeviceWhereabouts();

  @override
  Stream<GeoFix> watch() async* {
    try {
      await _allowed();

      yield* Geolocator.getPositionStream(
        locationSettings: switch (defaultTargetPlatform) {
          TargetPlatform.android => AndroidSettings(
            accuracy: LocationAccuracy.best,
            intervalDuration: RiftRules.locationReportInterval,
          ),
          TargetPlatform.iOS || TargetPlatform.macOS => AppleSettings(
            accuracy: LocationAccuracy.best,
            allowBackgroundLocationUpdates: false,
            pauseLocationUpdatesAutomatically: true,
          ),
          _ => const LocationSettings(accuracy: LocationAccuracy.best),
        },
      ).map(
        (position) => GeoFix(
          at: GeoPoint(position.latitude, position.longitude),
          accuracy: position.accuracy,
          observedAt: position.timestamp.toUtc(),
        ),
      );
    } on MissingPluginException {
      throw const LocationFailure(LocationIssue.unsupported);
    } on LocationServiceDisabledException {
      throw const LocationFailure(LocationIssue.services);
    } on PermissionDeniedException {
      throw const LocationFailure(LocationIssue.denied);
    }
  }

  static Future<void> _allowed() async {
    if (!await Geolocator.isLocationServiceEnabled()) {
      throw const LocationFailure(LocationIssue.services);
    }

    var permission = await Geolocator.checkPermission();

    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
    }

    if (permission == LocationPermission.deniedForever) {
      throw const LocationFailure(LocationIssue.blocked);
    }
    if (permission != LocationPermission.always &&
        permission != LocationPermission.whileInUse) {
      throw const LocationFailure(LocationIssue.denied);
    }
  }
}
