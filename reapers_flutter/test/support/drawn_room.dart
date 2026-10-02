import 'dart:math' as math;
import 'dart:typed_data';

import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/scan/depth/room_map.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// Something standing on the floor of a [DrawnRoom]: a block [height] metres
/// tall, between [left] and [right] metres to the player's right and between
/// [near] and [far] metres ahead of where they first faced.
final class Block {
  const Block({
    required this.left,
    required this.right,
    required this.near,
    required this.far,
    required this.height,
  });

  final double left;
  final double right;
  final double near;
  final double far;
  final double height;
}

/// A room drawn by hand, for testing what the map makes of pictures of it.
///
/// The player stands in the middle of a square room with their eye
/// [eyeHeight] metres above the floor. The walls are [wall] metres away on
/// every side and the ceiling is a metre above the eye.
final class DrawnRoom {
  const DrawnRoom({
    this.eyeHeight = 1.45,
    this.wall = 4,
    this.blocks = const [],
  });

  final double eyeHeight;
  final double wall;
  final List<Block> blocks;

  /// The Seeker's camera, held upright: how much a picture takes in.
  static const tanHalfWidth = 0.4122;
  static const tanHalfHeight = 0.7328;

  /// The picture of the room a perfect depth model would hand back with
  /// the phone turned [yaw] degrees to the right and tipped [pitch] degrees
  /// up.
  ///
  /// [stretch] makes every distance that much too long, as a model that
  /// misjudges scale does. [noise] is how far, as a share of the distance,
  /// each pixel may be out on its own.
  DepthFrame picture({
    required double yaw,
    required double pitch,
    double stretch = 1,
    double noise = 0,
    int seed = 1,
    int width = 252,
    int height = 448,
  }) {
    final pose = ArPose.looking(
      Vec3.fromAngles(radians(yaw), radians(pitch)),
    );
    final depths = Float32List(width * height);
    final random = math.Random(seed);

    for (var row = 0; row < height; row++) {
      final up = (1 - (row + 0.5) / height * 2) * tanHalfHeight;

      for (var column = 0; column < width; column++) {
        final across = ((column + 0.5) / width * 2 - 1) * tanHalfWidth;
        final ray = pose.forward + pose.right * across + pose.up * up;
        final wobble = 1 + noise * (random.nextDouble() * 2 - 1);

        depths[row * width + column] = _depthAlong(ray) * stretch * wobble;
      }
    }

    return DepthFrame(
      width: width,
      height: height,
      depths: depths,
      pose: pose,
      tanHalfWidth: tanHalfWidth,
      tanHalfHeight: tanHalfHeight,
    );
  }

  /// How far along [ray], in multiples of its length, the first surface is.
  double _depthAlong(Vec3 ray) {
    var nearest = double.infinity;

    void consider(double distance) {
      if (distance > 1e-6 && distance < nearest) {
        nearest = distance;
      }
    }

    if (ray.y != 0) {
      consider(-eyeHeight / ray.y);
      consider(1 / ray.y);
    }

    if (ray.x != 0) {
      consider(wall / ray.x);
      consider(-wall / ray.x);
    }

    if (ray.z != 0) {
      consider(wall / ray.z);
      consider(-wall / ray.z);
    }

    for (final block in blocks) {
      final distance = _blockAlong(ray, block);

      if (distance != null) {
        consider(distance);
      }
    }

    return nearest;
  }

  /// Where [ray] enters [block], or null when it misses.
  double? _blockAlong(Vec3 ray, Block block) {
    var enter = 0.0;
    var leave = double.infinity;

    for (final (direction, low, high) in [
      (ray.x, block.left, block.right),
      (ray.y, -eyeHeight, -eyeHeight + block.height),
      (ray.z, block.near, block.far),
    ]) {
      if (direction == 0) {
        if (low > 0 || high < 0) {
          return null;
        }

        continue;
      }

      final one = low / direction;
      final other = high / direction;

      enter = math.max(enter, math.min(one, other));
      leave = math.min(leave, math.max(one, other));
    }

    return enter < leave ? enter : null;
  }
}

/// Takes pictures of [room] into [map] as a player would: sweeping from
/// [from] to [to] degrees with the phone tipped [pitch] degrees up, a
/// picture every [every] degrees.
void sweep(
  RoomMap map,
  DrawnRoom room, {
  required double from,
  required double to,
  required double pitch,
  double every = 20,
  double stretch = 1,
  double noise = 0,
}) {
  var picture = 0;

  for (var yaw = from; yaw <= to + 1e-9; yaw += every) {
    map.add(
      room.picture(
        yaw: yaw,
        pitch: pitch,
        stretch: stretch,
        noise: noise,
        seed: ++picture,
      ),
    );
  }
}
