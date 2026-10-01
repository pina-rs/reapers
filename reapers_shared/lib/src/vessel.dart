import 'vec3.dart';

/// A place on a husk that a strike can land on.
///
/// The position is in the husk's own frame, in metres from the middle of its
/// body: `x` toward its left side (the right, as someone it is facing sees
/// it), `y` up, and `z` forward, out of its chest.
final class SealSpec {
  const SealSpec(this.offset);

  final Vec3 offset;
}

/// How a husk gets about.
enum Gait {
  /// It walks on the floor, and runs across it when it bolts.
  ground,

  /// It holds the air: it drifts, and darts when it bolts.
  air,

  /// It stays where it is.
  still,
}

/// The body a dead NFT wears in the spirit realm.
///
/// Every supported collection has a creature of its own: an ape for a
/// collection of apes, a whale for a collection of whales. The creature is an
/// original model; an NFT's own colours and traits dress it. The numbers
/// here are the single source for the rules and for the models: the art
/// build reads them when it shapes each body.
enum Vessel {
  /// A heavy ape that walks on its knuckles and charges off when startled.
  ape(
    gait: Gait.ground,
    halfWidth: 0.55,
    halfHeight: 0.55,
    nearMetres: 2.6,
    farMetres: 4.6,
    walkSpeed: 0.7,
    boltSpeed: 2.6,
    seals: [
      SealSpec(Vec3(0, 0.38, 0.30)),
      SealSpec(Vec3(0, 0.02, 0.36)),
      SealSpec(Vec3(0.36, 0.30, 0.05)),
      SealSpec(Vec3(-0.36, 0.30, 0.05)),
      SealSpec(Vec3(0, 0.36, -0.28)),
    ],
  ),

  /// A hunched bird that hangs in the air at about head height.
  bird(
    gait: Gait.air,
    halfWidth: 0.45,
    halfHeight: 0.28,
    nearMetres: 2.2,
    farMetres: 3.6,
    lowMetres: -0.15,
    highMetres: 0.85,
    walkSpeed: 0.5,
    boltSpeed: 3.2,
    seals: [
      SealSpec(Vec3(0, 0.22, 0.08)),
      SealSpec(Vec3(0, -0.04, 0.2)),
      SealSpec(Vec3(0.2, 0.06, 0)),
      SealSpec(Vec3(-0.2, 0.06, 0)),
      SealSpec(Vec3(0, 0.1, -0.2)),
    ],
  ),

  /// An upright penguin that waddles, and is easy to keep up with.
  penguin(
    gait: Gait.ground,
    halfWidth: 0.32,
    halfHeight: 0.45,
    nearMetres: 2.2,
    farMetres: 4.2,
    walkSpeed: 0.45,
    boltSpeed: 1.8,
    seals: [
      SealSpec(Vec3(0, 0.3, 0.2)),
      SealSpec(Vec3(0, -0.04, 0.27)),
      SealSpec(Vec3(0.28, 0.08, 0.02)),
      SealSpec(Vec3(-0.28, 0.08, 0.02)),
      SealSpec(Vec3(0, 0.1, -0.25)),
    ],
  ),

  /// A long-necked llama that trots, and gallops when it bolts.
  llama(
    gait: Gait.ground,
    halfWidth: 0.5,
    halfHeight: 0.72,
    nearMetres: 2.8,
    farMetres: 4.8,
    walkSpeed: 0.6,
    boltSpeed: 2.8,
    seals: [
      SealSpec(Vec3(0, 0.6, 0.3)),
      SealSpec(Vec3(0, 0.22, 0.32)),
      SealSpec(Vec3(0.22, -0.06, -0.08)),
      SealSpec(Vec3(-0.22, -0.06, -0.08)),
      SealSpec(Vec3(0, 0.1, -0.34)),
    ],
  ),

  /// A robot put together from parts that do not match, which walks on two
  /// legs and is in no hurry.
  robot(
    gait: Gait.ground,
    halfWidth: 0.36,
    halfHeight: 0.56,
    nearMetres: 2.4,
    farMetres: 4.4,
    walkSpeed: 0.5,
    boltSpeed: 2.0,
    seals: [
      SealSpec(Vec3(0, 0.42, 0.17)),
      SealSpec(Vec3(0, 0.08, 0.2)),
      SealSpec(Vec3(0.3, 0.2, 0)),
      SealSpec(Vec3(-0.3, 0.2, 0)),
      SealSpec(Vec3(0, 0.1, -0.2)),
    ],
  ),

  /// A tower of image cubes held together by light. Too much for one reaper:
  /// it only appears in a rift, and its seals are the seams between cubes.
  reliquaryStack(
    gait: Gait.still,
    halfWidth: 1.15,
    halfHeight: 5.0,
    nearMetres: 17,
    farMetres: 17,
    lowMetres: 1.9,
    highMetres: 1.9,
    walkSpeed: 0,
    boltSpeed: 0,
    seals: [
      SealSpec(Vec3(0, -3.0, 1.05)),
      SealSpec(Vec3(0, -1.0, 1.05)),
      SealSpec(Vec3(0, 1.0, 1.05)),
      SealSpec(Vec3(0, 3.0, 1.05)),
    ],
  );

  const Vessel({
    required this.gait,
    required this.halfWidth,
    required this.halfHeight,
    required this.nearMetres,
    required this.farMetres,
    required this.walkSpeed,
    required this.boltSpeed,
    required this.seals,
    this.lowMetres = 0,
    this.highMetres = 0,
  });

  final Gait gait;

  /// Half the body's width, in metres, taken as the ellipse a strike can
  /// land in.
  final double halfWidth;

  /// Half the body's height, in metres.
  final double halfHeight;

  /// The nearest the husk comes to the player, measured along the floor.
  final double nearMetres;

  /// The farthest it goes.
  final double farMetres;

  /// The lowest the middle of an airborne body flies, in metres above the
  /// player's eye. A body on the ground ignores this: it stands on the floor.
  final double lowMetres;

  /// The highest it flies.
  final double highMetres;

  /// How fast it ambles when nothing has alarmed it, in metres a second.
  final double walkSpeed;

  /// How fast it flees.
  final double boltSpeed;

  final List<SealSpec> seals;

  /// Whether a single reaper may hunt this vessel.
  bool get isSolo => this != reliquaryStack;
}
