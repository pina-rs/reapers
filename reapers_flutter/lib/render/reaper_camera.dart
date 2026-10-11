import 'dart:math' as math;

import 'package:vector_math/vector_math.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// The exported face points along glTF +Z with Y up. Native compilation
/// reflects Z. A lower, closer view clears the Harvester's drooping hat brim.
({Vector3 offset, Vector3 up}) reaperFaceOrbit(
  double turn, {
  required ReaperClass calling,
  required bool native,
}) {
  final front = native ? -1.0 : 1.0;
  final underHat = calling == ReaperClass.harvester;
  final distance = underHat ? 1.15 : 1.25;
  return (
    offset: Vector3(
      math.sin(turn) * distance,
      underHat ? -.6 : -.04,
      math.cos(turn) * distance * front,
    ),
    up: Vector3(0, 1, 0),
  );
}
