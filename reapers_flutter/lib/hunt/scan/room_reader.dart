import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import '../pose/ar_pose.dart';

/// What the player should do next to finish reading the room.
enum ReadingAdvice {
  /// Carry on: this part of the room is being read.
  keepGoing,

  /// The phone is turning too fast for anything to be made out.
  slowDown,

  /// The phone is pointing too high to see what needs reading.
  lookLower,

  /// The phone is pointing too low to see what needs reading.
  lookHigher,
}

/// Reads the room as the player looks around it, and says where a husk may
/// go once enough has been read.
///
/// A husk only ever appears, and only ever moves, where the player has
/// already looked: over the floor for one that walks, through the air at
/// about head height for one that flies. This is what decides that. It
/// divides the circle around the player into narrow slices and counts a
/// slice as read once the phone has looked into it steadily for a moment at
/// the right height.
///
/// What it knows is where the player has looked and where down is. It does
/// not yet know what is standing in the way there: see docs/gameplay.md.
final class RoomReader {
  RoomReader({required this.gait});

  /// How the husk that will appear gets about, which decides where the
  /// player has to look.
  final Gait gait;

  /// Slices in the full circle: five degrees each.
  static const slices = 72;

  /// How long a slice has to be looked into before it counts as read.
  static const secondsToRead = 0.3;

  /// Turning faster than this, in radians a second, reads nothing: the
  /// picture is a blur.
  static const steadyTurnSpeed = 1.5;

  /// How far either side of where the phone points is read at once. The
  /// middle of the picture, where the lens is sharpest.
  static final _reach = radians(13);

  /// The arc that has to be read before a husk appears.
  static final neededArc = radians(115);

  final _seconds = List<double>.filled(slices, 0);

  /// What the player should do next. Updated by [observe].
  ReadingAdvice advice = ReadingAdvice.keepGoing;

  /// How far below or above the horizon the phone has to point.
  ///
  /// The floor a walker will stand on lies two to five metres out, which is
  /// well below the horizon from standing height. A flier keeps to about
  /// head height.
  (double, double) get _band => switch (gait) {
    Gait.ground => (radians(-52), radians(-7)),
    Gait.air || Gait.still => (radians(-18), radians(38)),
  };

  /// Takes in where the phone pointed for the last [deltaSeconds].
  void observe(ArPose pose, double deltaSeconds) {
    final (low, high) = _band;
    final pitch = pose.forward.pitch;

    advice = switch (pitch) {
      _ when pose.turnSpeed > steadyTurnSpeed => ReadingAdvice.slowDown,
      _ when pitch < low => ReadingAdvice.lookHigher,
      _ when pitch > high => ReadingAdvice.lookLower,
      _ => ReadingAdvice.keepGoing,
    };

    if (advice != ReadingAdvice.keepGoing) {
      return;
    }

    final yaw = pose.forward.yaw;

    for (var slice = 0; slice < slices; slice++) {
      if (wrapAngle(_bearingOf(slice) - yaw).abs() <= _reach) {
        _seconds[slice] += deltaSeconds;
      }
    }
  }

  /// How well slice [index] has been read, from 0 to 1.
  double read(int index) => (_seconds[index] / secondsToRead).clamp(0.0, 1.0);

  /// How close the room is to being read, from 0 to 1.
  double get progress {
    final (_, length) = _widestRun();

    return (length * _sliceWidth / neededArc).clamp(0.0, 1.0);
  }

  /// Where a husk may go, or null until enough of the room has been read.
  HuntPlan? get plan {
    final (start, length) = _widestRun();

    if (length * _sliceWidth < neededArc) {
      return null;
    }

    if (length >= slices) {
      return HuntPlan.open;
    }

    final from = _bearingOf(start) - _sliceWidth / 2;

    return HuntPlan.between(from, from + length * _sliceWidth);
  }

  static const _sliceWidth = 2 * math.pi / slices;

  /// The bearing of the middle of slice [index].
  static double _bearingOf(int index) =>
      wrapAngle(index * _sliceWidth - math.pi + _sliceWidth / 2);

  /// The longest unbroken run of read slices, going round the circle: where
  /// it starts and how many slices it holds.
  (int, int) _widestRun() {
    var best = (0, 0);

    if (Iterable<int>.generate(slices).every((slice) => read(slice) >= 1)) {
      return (0, slices);
    }

    for (var start = 0; start < slices; start++) {
      // A run starts just after an unread slice.
      if (read(start) < 1 || read((start - 1) % slices) >= 1) {
        continue;
      }

      var length = 0;

      while (length < slices && read((start + length) % slices) >= 1) {
        length++;
      }

      if (length > best.$2) {
        best = (start, length);
      }
    }

    return best;
  }
}
