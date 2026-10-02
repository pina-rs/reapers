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

/// A reading of the room, made while the player looks around it, that ends
/// in a plan of where a husk may go.
///
/// A husk only ever appears, and only ever moves, where the room has been
/// read. The circle around the player is divided into the slices of a
/// [HuntPlan], and a survey fills them in one by one.
abstract interface class RoomSurvey {
  /// The arc that has to be read before a husk appears.
  static final neededArc = radians(115);

  /// What the player should do next.
  ReadingAdvice get advice;

  /// Takes in where the phone pointed for the last [deltaSeconds]. Called
  /// once a frame.
  void observe(ArPose pose, double deltaSeconds);

  /// How well slice [index] has been read, from 0 to 1.
  double read(int index);

  /// How far the way is clear along slice [index], in metres, or null where
  /// this survey has no way of telling.
  double? reach(int index);

  /// Whether slice [index] has been read and found tight: so little clear
  /// floor that the husk can only stand there as close as it ever comes.
  bool tight(int index);

  /// How close the room is to being read, from 0 to 1.
  double get progress;

  /// Where a husk may go, or null until enough of the room has been read.
  HuntPlan? get plan;

  /// The plan for a hunt begun without waiting for the room to be read: the
  /// whole circle, with whatever has been learnt of the floor so far and
  /// nothing about what stands on it.
  HuntPlan get planUnread;

  /// What went wrong, if the survey has broken down and will read no more.
  Object? get failure;

  void dispose();
}

/// What [pose] calls for when the room is being read for a husk with
/// [gait]: whether the phone is turning slowly enough, under
/// [steadyTurnSpeed] radians a second, and pointing at the height that
/// needs reading.
///
/// The floor a walker will stand on lies two to five metres out, which is
/// well below the horizon from standing height. A flier keeps to about head
/// height.
ReadingAdvice postureAdvice(
  ArPose pose,
  Gait gait, {
  required double steadyTurnSpeed,
}) {
  final (low, high) = switch (gait) {
    Gait.ground => (radians(-52), radians(-7)),
    Gait.air || Gait.still => (radians(-18), radians(38)),
  };
  final pitch = pose.forward.pitch;

  return switch (pitch) {
    _ when pose.turnSpeed > steadyTurnSpeed => ReadingAdvice.slowDown,
    _ when pitch < low => ReadingAdvice.lookHigher,
    _ when pitch > high => ReadingAdvice.lookLower,
    _ => ReadingAdvice.keepGoing,
  };
}

/// The longest unbroken run of true values in [marks], going round the
/// circle: where it starts and how many it holds.
(int, int) widestRun(List<bool> marks) {
  final count = marks.length;

  if (marks.every((mark) => mark)) {
    return (0, count);
  }

  var best = (0, 0);

  for (var start = 0; start < count; start++) {
    // A run starts just after a gap.
    if (!marks[start] || marks[(start - 1) % count]) {
      continue;
    }

    var length = 0;

    while (length < count && marks[(start + length) % count]) {
      length++;
    }

    if (length > best.$2) {
      best = (start, length);
    }
  }

  return best;
}
