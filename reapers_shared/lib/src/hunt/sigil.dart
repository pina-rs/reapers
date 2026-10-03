import 'dart:math' as math;

import '../seeded_random.dart';
import '../vec3.dart';
import '../vessel.dart';
import 'difficulty.dart';
import 'hunt_stats.dart';

/// How hard a husk's sigil is to trace.
///
/// Every rarity has one. The rarer the husk, the more lights its sigil has,
/// the wider they are spread, the smaller each one is, the longer each has
/// to be held, and the more often the trace doubles back across itself.
/// The rarest let a light go out again if the next is too long coming.
final class SigilShape {
  const SigilShape({
    required this.lights,
    required this.spanDegrees,
    required this.radiusDegrees,
    required this.holdSeconds,
    required this.crossings,
    this.fraySeconds,
  }) : assert(lights >= 2, 'A sigil needs a light to go from and one to go to');

  /// How many lights the sigil has.
  final int lights;

  /// How far apart round the reaper its two outermost lights hang.
  final double spanDegrees;

  /// How near the middle of the view has to come to a light to rest on it.
  final double radiusDegrees;

  /// How long the view has to rest on a light, steadily, for it to catch.
  final double holdSeconds;

  /// How many times the trace from light to light crosses itself.
  final int crossings;

  /// How long a caught light waits for the next to catch before it goes out
  /// again, or null when a caught light stays caught.
  final double? fraySeconds;
}

/// One light of a sigil.
final class SigilLight {
  const SigilLight({required this.yaw, required this.pitch});

  /// How far to the right of the sigil's middle it hangs, in radians.
  final double yaw;

  /// How far above the horizon it hangs, in radians. Below is negative.
  final double pitch;

  /// Which way it is from the reaper, for a sigil whose middle is straight
  /// ahead.
  Vec3 get direction => Vec3.fromAngles(yaw, pitch);

  @override
  String toString() =>
      'SigilLight(${degrees(yaw).round()}°, ${degrees(pitch).round()}°)';
}

/// The pattern a reaper traces with the phone to draw a husk out of hiding.
///
/// A sigil is a handful of lights hung in the air around the reaper. They
/// catch one at a time, in order, each once the middle of the view has
/// rested on it steadily for a moment. When the last one catches, the husk
/// shows itself.
///
/// Tracing it is also how the room is read. The lights hang where the phone
/// has to look for this husk: over the floor for one that walks, at about
/// head height for one that flies, and across the whole arc it will keep
/// to. Resting on a light holds the phone still long enough for the camera
/// to take a picture there.
///
/// A sigil is drawn from the hunt's seed, so the same husk always has the
/// same one.
final class Sigil {
  const Sigil({
    required this.lights,
    required this.radius,
    required this.holdSeconds,
    this.fraySeconds,
  });

  /// The sigil of a husk with [shape] that gets about by [gait], for the
  /// hunt with [seed], as forgiving as [difficulty] makes it.
  ///
  /// The lights stand in a row of evenly spaced columns, one to a column,
  /// each at one of three heights and never at the height of the one beside
  /// it. The order they catch in is what makes a sigil hard: in a simple one
  /// it runs straight along the row, and in a hard one it doubles back and
  /// crosses itself. The trace never passes over a light it does not stop
  /// at, so the order can always be read from the shape.
  factory Sigil.draw({
    required int seed,
    required SigilShape shape,
    required Gait gait,
    Difficulty difficulty = Difficulty.normal,
  }) {
    final random = SeededRandom(mixSeed([seed, _salt]));
    final band = switch (gait) {
      Gait.ground => HuntRules.sigilGroundBand,
      Gait.air || Gait.still => HuntRules.sigilAirBand,
    };
    final clearance = radians(shape.radiusDegrees) * _clearanceShare;
    List<SigilLight>? nearest;
    var nearestMiss = shape.lights * shape.lights;

    // Heights and order are drawn together until they make a sigil that
    // crosses itself as often as its shape asks. A straight row is taken
    // at once.
    for (var attempt = 0; attempt < _attempts && nearestMiss > 0; attempt++) {
      final lights = _attempt(shape, band, random);

      if (_passesOver(lights, clearance)) {
        continue;
      }

      final miss = (_crossings(lights) - shape.crossings).abs();

      if (miss < nearestMiss) {
        nearest = lights;
        nearestMiss = miss;
      }
    }

    final fray = shape.fraySeconds;

    return Sigil(
      lights: nearest ?? _row(shape, band, random),
      radius: radians(shape.radiusDegrees) * difficulty.reach,
      holdSeconds: shape.holdSeconds,
      fraySeconds: fray == null ? null : fray * difficulty.patience,
    );
  }

  /// The lights, in the order they catch.
  final List<SigilLight> lights;

  /// How near the middle of the view has to come to a light to rest on it,
  /// in radians.
  final double radius;

  /// How long the view has to rest on a light for it to catch.
  final double holdSeconds;

  /// How long a caught light waits for the next before going out, or null.
  final double? fraySeconds;

  /// Half the width of the arc tracing this sigil reads, in radians: out to
  /// its farthest light and [HuntRules.sigilMargin] beyond, which the camera
  /// takes in to either side of where it points.
  double get halfArc =>
      lights.map((light) => light.yaw.abs()).reduce(math.max) +
      HuntRules.sigilMargin;

  /// How many times the trace from light to light crosses itself.
  int get crossings => _crossings(lights);

  /// Mixed into the hunt's seed, so that the sigil does not follow the same
  /// rolls as everything else the seed decides.
  static const _salt = 0x5161;

  /// Tries this many sigils for one that crosses itself as its shape asks
  /// before settling for the nearest to it.
  static const _attempts = 2000;

  /// One try at a sigil with [shape] whose lights hang within [band]: the
  /// heights drawn afresh, and the order too unless the sigil is a straight
  /// row.
  static List<SigilLight> _attempt(
    SigilShape shape,
    (double, double) band,
    SeededRandom random,
  ) {
    if (shape.crossings == 0) {
      return _row(shape, band, random);
    }

    final places = _places(shape, band, random);

    for (var index = places.length - 1; index > 0; index--) {
      final other = random.nextInt(index + 1);
      final kept = places[index];

      places[index] = places[other];
      places[other] = kept;
    }

    return places;
  }

  /// A sigil that runs straight along its row, from one end or the other.
  static List<SigilLight> _row(
    SigilShape shape,
    (double, double) band,
    SeededRandom random,
  ) {
    final places = _places(shape, band, random);

    return random.nextChance(0.5) ? places : places.reversed.toList();
  }

  /// The lights of a sigil with [shape], from left to right: evenly spaced
  /// across its span, each at one of three heights within [band] and never
  /// at the height of the one beside it.
  static List<SigilLight> _places(
    SigilShape shape,
    (double, double) band,
    SeededRandom random,
  ) {
    final (low, high) = band;
    final span = radians(shape.spanDegrees);
    final count = shape.lights;
    final heights = <int>[random.nextInt(3)];

    while (heights.length < count) {
      heights.add((heights.last + 1 + random.nextInt(2)) % 3);
    }

    return [
      for (var column = 0; column < count; column++)
        SigilLight(
          yaw: -span / 2 + span * column / (count - 1),
          pitch: low + (high - low) * heights[column] / 2,
        ),
    ];
  }

  /// How close, as a share of a light's radius, the trace may pass to a
  /// light it does not stop at.
  static const _clearanceShare = 1.5;

  /// Whether any stretch of the trace through [lights] passes within
  /// [clearance] of a light that is not one of its ends.
  static bool _passesOver(List<SigilLight> lights, double clearance) {
    for (var step = 0; step + 1 < lights.length; step++) {
      final from = lights[step];
      final to = lights[step + 1];

      for (final light in lights) {
        if (identical(light, from) || identical(light, to)) {
          continue;
        }

        if (_distanceToStretch(light, from, to) < clearance) {
          return true;
        }
      }
    }

    return false;
  }

  /// How far [light] is, in radians, from the stretch of the trace between
  /// [from] and [to]: the shortest way round from one to the other, which is
  /// the straight line between them that the view draws.
  static double _distanceToStretch(
    SigilLight light,
    SigilLight from,
    SigilLight to,
  ) {
    final (point, start, end) = (light.direction, from.direction, to.direction);
    // The pole of the great circle through both ends.
    final pole = start.cross(end).normalized;
    final off = point.dot(pole);
    final foot = (point - pole * off).normalized;
    final within =
        start.angleTo(foot) + foot.angleTo(end) <= start.angleTo(end) + 1e-9;

    if (within) {
      return math.asin(off.abs().clamp(0.0, 1.0));
    }

    return math.min(point.angleTo(start), point.angleTo(end));
  }

  /// How many pairs of stretches of the trace through [lights] cross.
  /// Stretches that meet end to end do not count.
  static int _crossings(List<SigilLight> lights) {
    var count = 0;

    for (var first = 0; first + 1 < lights.length; first++) {
      for (var second = first + 2; second + 1 < lights.length; second++) {
        if (_cross(
          lights[first],
          lights[first + 1],
          lights[second],
          lights[second + 1],
        )) {
          count++;
        }
      }
    }

    return count;
  }

  static bool _cross(SigilLight a, SigilLight b, SigilLight c, SigilLight d) {
    double turn(SigilLight from, SigilLight to, SigilLight point) =>
        (to.yaw - from.yaw) * (point.pitch - from.pitch) -
        (to.pitch - from.pitch) * (point.yaw - from.yaw);

    return turn(a, b, c) * turn(a, b, d) < 0 &&
        turn(c, d, a) * turn(c, d, b) < 0;
  }
}

/// Something that happened while a sigil was traced.
sealed class SigilEvent {
  const SigilEvent();
}

/// The light at [index] caught.
final class LightCaught extends SigilEvent {
  const LightCaught(this.index);

  final int index;
}

/// The light at [index] went out: the next was too long coming.
final class LightLost extends SigilEvent {
  const LightLost(this.index);

  final int index;
}

/// The last light caught. The husk shows itself.
final class SigilClosed extends SigilEvent {
  const SigilClosed();
}

/// A sigil as it is traced: which of its lights have caught, and how near
/// the next one is to catching.
///
/// It reads no clock. Each frame it is told where the phone points, how
/// fast it is turning and how long the frame was.
final class SigilTrace {
  SigilTrace(this.sigil, {required this.centre});

  final Sigil sigil;

  /// The bearing the sigil hangs around, in radians: where the reaper faced
  /// when it appeared.
  final double centre;

  int _caught = 0;
  double _fill = 0;
  double _slack = 0;
  double _seconds = 0;

  /// How long the sigil has been traced, in seconds.
  double get seconds => _seconds;

  /// How many lights have caught.
  int get caught => _caught;

  /// How near the next light is to catching, from 0 to 1.
  double get fill => _fill;

  /// Whether every light has caught.
  bool get isClosed => _caught == sigil.lights.length;

  /// How near the last light caught is to going out, from 0 to 1. Always 0
  /// for a sigil whose lights stay caught.
  double get fraying => switch (sigil.fraySeconds) {
    final seconds? when _caught > 0 && !isClosed => _slack / seconds,
    _ => 0,
  };

  /// Which way the light at [index] is from the reaper.
  Vec3 directionOf(int index) {
    final light = sigil.lights[index];

    return Vec3.fromAngles(centre + light.yaw, light.pitch);
  }

  /// Whether a view along [aim] rests on the next light.
  bool restsOn(Vec3 aim) =>
      !isClosed && aim.angleTo(directionOf(_caught)) <= sigil.radius;

  /// Takes in where the phone pointed, [aim], and how fast it was turning,
  /// for the last [deltaSeconds], and answers what came of it.
  ///
  /// The next light fills while the view rests on it and turns slower than
  /// [HuntRules.sigilSteadySpeed]. Resting on it while turning faster holds
  /// it where it is, and looking away drains it.
  List<SigilEvent> advance(
    Vec3 aim, {
    required double turnSpeed,
    required double deltaSeconds,
  }) {
    if (isClosed) {
      return const [];
    }

    _seconds += deltaSeconds;

    final resting = restsOn(aim);
    final step = deltaSeconds / sigil.holdSeconds;

    if (resting && turnSpeed <= HuntRules.sigilSteadySpeed) {
      _fill += step;
    } else if (!resting) {
      _fill = math.max(0, _fill - step * HuntRules.sigilDrain);
    }

    if (_fill >= 1) {
      final index = _caught;

      _caught++;
      _fill = 0;
      _slack = 0;

      return [LightCaught(index), if (isClosed) const SigilClosed()];
    }

    final fray = sigil.fraySeconds;

    // A light waits while the next is being held, and frays only while the
    // reaper is looking for it.
    if (fray == null || _caught == 0 || resting) {
      return const [];
    }

    _slack += deltaSeconds;

    if (_slack < fray) {
      return const [];
    }

    _caught--;
    _slack = 0;
    _fill = 0;

    return [LightLost(_caught)];
  }
}
