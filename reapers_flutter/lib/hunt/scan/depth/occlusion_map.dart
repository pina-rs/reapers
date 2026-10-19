import 'dart:math' as math;
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';

import 'room_map.dart';

/// The room as it looks from where the player stands: how far away the
/// nearest thing standing in it is, in every direction the camera saw.
///
/// It is what lets a husk go behind the counter and be hidden by it. The
/// phone only ever turns where the player stands, so a picture taken while
/// the sigil was traced still lines up with the view from the same spot
/// during the fight, whichever way the phone points then: nothing has to be
/// read again. The floor is left out. A husk stands on the floor, and is
/// never hidden by it.
///
/// Like [RoomMap] this is arithmetic and nothing else, so that it can be
/// tested against rooms drawn by hand.
final class OcclusionMap {
  OcclusionMap() : _metres = Float32List(columns * rows);

  /// How wide one cell of the map is, in degrees, each way.
  static const cellDegrees = 0.25;

  /// The map runs all the way round, and from [lowest] to [highest] degrees
  /// above the horizon: from the floor under a sitting player's feet to the
  /// top of a flier's air.
  static const lowest = -72.0;
  static const highest = 48.0;

  static const columns = 360 ~/ cellDegrees;
  static const rows = (highest - lowest) ~/ cellDegrees;

  /// The farthest a cell records, in metres.
  static const farthest = 40.0;

  /// How far off the floor a point may be and still be floor, as in
  /// [RoomMap].
  static const _floorSlack = 0.07;
  static const _floorSlackPerMetre = 0.045;

  /// Nearer than this, in metres, is the player's own feet and hands.
  static const _selfMetres = 0.6;

  /// For each cell: how far away the nearest thing standing on the floor is,
  /// in metres; [double.infinity] where only floor was seen; zero where
  /// nothing was seen.
  final Float32List _metres;

  /// Pictures taken in so far.
  int pictures = 0;

  /// Takes in one picture whose floor is [floor] metres below the eye in
  /// its own metres, scaling every length by [scale] to agree with the
  /// floor the hunt stands on.
  void add(DepthFrame frame, {required double floor, required double scale}) {
    final pose = frame.pose;

    for (var row = 0; row < frame.height; row++) {
      final up = (1 - (row + 0.5) / frame.height * 2) * frame.tanHalfHeight;

      for (var column = 0; column < frame.width; column++) {
        final across =
            ((column + 0.5) / frame.width * 2 - 1) * frame.tanHalfWidth;
        final depth = frame.depths[row * frame.width + column];

        if (!(depth > 0) || !depth.isFinite) {
          continue;
        }

        final x =
            depth * (pose.forward.x + pose.right.x * across + pose.up.x * up);
        final y =
            depth * (pose.forward.y + pose.right.y * across + pose.up.y * up);
        final z =
            depth * (pose.forward.z + pose.right.z * across + pose.up.z * up);
        final away = math.sqrt(x * x + z * z);
        final distance = math.sqrt(away * away + y * y);

        if (distance < _selfMetres) {
          continue;
        }

        final cell = _cellOf(math.atan2(x, z), math.atan2(y, away));

        if (cell == null) {
          continue;
        }

        final onFloor =
            (floor + y).abs() <= _floorSlack + _floorSlackPerMetre * away;

        if (onFloor) {
          if (_metres[cell] == 0) {
            _metres[cell] = double.infinity;
          }

          continue;
        }

        final metres = math.min(farthest, distance * scale);
        final known = _metres[cell];

        if (known == 0 || metres < known) {
          _metres[cell] = metres;
        }
      }
    }

    pictures++;
  }

  /// How far away the nearest thing standing in the room is along [yaw]
  /// and [pitch], in metres: infinite where only floor was seen there, and
  /// null where nothing was.
  double? distanceAt(double yaw, double pitch) {
    final cell = _cellOf(yaw, pitch);
    final metres = cell == null ? 0.0 : _metres[cell];

    return metres == 0 ? null : metres;
  }

  /// Whether a point [metres] away along [yaw] and [pitch] is behind
  /// something the camera saw.
  bool hides(double yaw, double pitch, double metres) {
    final nearest = distanceAt(yaw, pitch);

    return nearest != null && nearest < metres;
  }

  /// The map as the pixels of an image [columns] wide and [rows] tall, for
  /// a shader to read: the distance in centimetres in red (high byte) and
  /// green (low byte), and in blue whether anything standing was seen
  /// there (255), only floor (128), or nothing (0).
  Uint8List toPixels() {
    final pixels = Uint8List(columns * rows * 4);

    for (var cell = 0; cell < _metres.length; cell++) {
      final metres = _metres[cell];
      final at = cell * 4;

      if (metres == 0) {
        pixels[at + 3] = 255;
        continue;
      }

      final centimetres = metres.isFinite
          ? (metres * 100).round().clamp(0, 0xFFFF)
          : 0xFFFF;

      pixels
        ..[at] = centimetres >> 8
        ..[at + 1] = centimetres & 0xFF
        ..[at + 2] = metres.isFinite ? 255 : 128
        ..[at + 3] = 255;
    }

    return pixels;
  }

  /// The cell that holds the direction [yaw], [pitch], or null when it is
  /// above or below what the map covers. Row 0 is the highest.
  static int? _cellOf(double yaw, double pitch) {
    final up = degrees(pitch);

    if (up < lowest || up >= highest) {
      return null;
    }

    final column = ((degrees(wrapAngle(yaw)) + 180) / cellDegrees)
        .floor()
        .clamp(0, columns - 1);
    final row = ((highest - up) / cellDegrees).floor().clamp(0, rows - 1);

    return row * columns + column;
  }
}
