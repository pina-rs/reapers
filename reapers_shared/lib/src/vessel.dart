import 'vec3.dart';

/// A place on a husk that a strike can land on.
///
/// The position is in the husk's own frame, in metres: `x` to the right as
/// the player sees it, `y` up, and `z` toward the player.
final class SealSpec {
  const SealSpec(this.offset, {this.orbitRadius = 0, this.orbitPhase = 0});

  /// Where the seal sits when the husk is at rest.
  final Vec3 offset;

  /// When above zero the seal circles the husk's centre at this radius, in
  /// the plane facing the player, instead of staying at [offset].
  final double orbitRadius;

  /// Where on its circle an orbiting seal starts, in radians.
  final double orbitPhase;
}

/// The 3D body a dead NFT wears in the spirit realm.
///
/// A vessel is an original shape that carries any square image, so a husk
/// looks like the NFT it came from without copying a collection's art. The
/// numbers here are the single source for the rules and for the models: the
/// art build reads them when it shapes each body.
enum Vessel {
  /// A cracked, floating picture frame trailing torn cloth. The image is the
  /// canvas and the seals are the four corner nails.
  frameWraith(
    halfWidth: 0.46,
    halfHeight: 0.62,
    distance: 3.0,
    driftDegrees: 11,
    seals: [
      SealSpec(Vec3(-0.40, 0.52, 0.07)),
      SealSpec(Vec3(0.40, 0.52, 0.07)),
      SealSpec(Vec3(0.40, -0.28, 0.07)),
      SealSpec(Vec3(-0.40, -0.28, 0.07)),
    ],
  ),

  /// A paper lantern turning around a flame. The image wraps the paper and
  /// the seals are tears that only show while they face the player.
  lanternShade(
    halfWidth: 0.36,
    halfHeight: 0.55,
    distance: 2.8,
    driftDegrees: 9,
    spinDegreesPerSecond: 42,
    seals: [
      SealSpec(Vec3(0, 0.12, 0.34)),
      SealSpec(Vec3(0.294, -0.10, -0.17)),
      SealSpec(Vec3(-0.294, 0.02, -0.17)),
    ],
  ),

  /// Mirror shards circling a dark core. The image is split across the
  /// shards, and the seals move with them.
  shardSwarm(
    halfWidth: 0.62,
    halfHeight: 0.62,
    distance: 3.2,
    driftDegrees: 14,
    seals: [
      SealSpec(Vec3(0, 0, 0.1)),
      SealSpec(Vec3.zero, orbitRadius: 0.48),
      SealSpec(Vec3.zero, orbitRadius: 0.48, orbitPhase: 2.0944),
      SealSpec(Vec3.zero, orbitRadius: 0.48, orbitPhase: 4.1888),
    ],
    orbitDegreesPerSecond: 55,
  ),

  /// A long printed ribbon that swims in a slow circle around the player,
  /// so it has to be followed with a steady turn.
  scrollEel(
    halfWidth: 0.85,
    halfHeight: 0.30,
    distance: 3.0,
    driftDegrees: 5,
    circleDegreesPerSecond: 16,
    seals: [
      SealSpec(Vec3(-0.62, 0.04, 0.05)),
      SealSpec(Vec3(-0.12, -0.03, 0.05)),
      SealSpec(Vec3(0.40, 0.05, 0.05)),
    ],
  ),

  /// A tower of image cubes held together by light. Too much for one reaper:
  /// it only appears in a rift, and its seals are the seams between cubes.
  reliquaryStack(
    halfWidth: 1.15,
    halfHeight: 5.0,
    distance: 17,
    driftDegrees: 0,
    seals: [
      SealSpec(Vec3(0, -3.0, 1.05)),
      SealSpec(Vec3(0, -1.0, 1.05)),
      SealSpec(Vec3(0, 1.0, 1.05)),
      SealSpec(Vec3(0, 3.0, 1.05)),
    ],
  );

  const Vessel({
    required this.halfWidth,
    required this.halfHeight,
    required this.distance,
    required this.driftDegrees,
    required this.seals,
    this.spinDegreesPerSecond = 0,
    this.orbitDegreesPerSecond = 0,
    this.circleDegreesPerSecond = 0,
  });

  /// Half the body's width, in metres.
  final double halfWidth;

  /// Half the body's height, in metres.
  final double halfHeight;

  /// How far from the player the husk hovers, in metres.
  final double distance;

  /// How far the husk wanders either side of where it settled.
  final double driftDegrees;

  final List<SealSpec> seals;

  /// How fast the whole body turns about its upright axis.
  final double spinDegreesPerSecond;

  /// How fast orbiting seals circle the centre.
  final double orbitDegreesPerSecond;

  /// How fast the husk circles the player. Zero means it holds its ground.
  final double circleDegreesPerSecond;

  /// Whether a single reaper may hunt this vessel.
  bool get isSolo => this != reliquaryStack;
}
