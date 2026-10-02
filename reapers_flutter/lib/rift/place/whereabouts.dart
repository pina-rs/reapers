import 'dart:async';

import 'package:flutter/services.dart';
import 'package:geolocator/geolocator.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'whereabouts.g.dart';

/// Where the phone is, and how far out that may be, in metres.
typedef GeoFix = ({GeoPoint at, double accuracy});

/// Whatever can say where the phone is.
abstract interface class Whereabouts {
  /// Where the phone is, each time it has a better or newer idea, for as
  /// long as anyone listens.
  ///
  /// A phone that cannot say, or whose owner would rather it did not, says
  /// nothing: the stream simply stays empty. A rift is played either way.
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
      if (!await _allowed()) {
        return;
      }

      yield* Geolocator.getPositionStream(
        locationSettings: const LocationSettings(
          accuracy: LocationAccuracy.best,
        ),
      ).map(
        (position) => (
          at: GeoPoint(position.latitude, position.longitude),
          accuracy: position.accuracy,
        ),
      );
    } on MissingPluginException {
      // A platform with no way of telling where it is.
      return;
    } on LocationServiceDisabledException {
      // Location was switched off part-way through. Nothing more is known.
      return;
    }
  }

  static Future<bool> _allowed() async {
    if (!await Geolocator.isLocationServiceEnabled()) {
      return false;
    }

    var permission = await Geolocator.checkPermission();

    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
    }

    return permission == LocationPermission.always ||
        permission == LocationPermission.whileInUse;
  }
}
