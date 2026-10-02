import 'package:reapers_shared/reapers_shared.dart';

import '../pose/ar_pose.dart';
import 'room_survey.dart';

/// Reads the room from nothing but where the player has looked.
///
/// This is the survey for anywhere the room cannot be seen: a simulator, a
/// desktop, a phone with no camera to spare. It counts a slice as read once
/// the phone has looked into it steadily for a moment at the right height:
/// over the floor for a husk that walks, at about head height for one that
/// flies.
///
/// It knows where the player has looked and where down is. It does not know
/// how far away the floor is or what stands on it, so its plan leaves the
/// husk free to keep whatever distance it likes. `DepthSurvey` is the one
/// that sees.
final class RoomReader implements RoomSurvey {
  RoomReader({required this.gait});

  /// How the husk that will appear gets about, which decides where the
  /// player has to look.
  final Gait gait;

  /// How long a slice has to be looked into before it counts as read.
  static const secondsToRead = 0.3;

  /// Turning faster than this, in radians a second, reads nothing: the
  /// picture is a blur.
  static const steadyTurnSpeed = 1.5;

  /// How far either side of where the phone points is read at once. The
  /// middle of the picture, where the lens is sharpest.
  static final _span = radians(13);

  final _seconds = List<double>.filled(HuntPlan.slices, 0);

  @override
  ReadingAdvice advice = ReadingAdvice.keepGoing;

  @override
  void observe(ArPose pose, double deltaSeconds) {
    advice = postureAdvice(pose, gait, steadyTurnSpeed: steadyTurnSpeed);

    if (advice != ReadingAdvice.keepGoing) {
      return;
    }

    final yaw = pose.forward.yaw;

    for (var slice = 0; slice < HuntPlan.slices; slice++) {
      if (wrapAngle(HuntPlan.bearingOf(slice) - yaw).abs() <= _span) {
        _seconds[slice] += deltaSeconds;
      }
    }
  }

  @override
  double read(int index) => (_seconds[index] / secondsToRead).clamp(0.0, 1.0);

  @override
  double? reach(int index) => null;

  @override
  bool tight(int index) => false;

  @override
  double get progress {
    final (_, length) = _run();

    return (length * HuntPlan.sliceWidth / RoomSurvey.neededArc).clamp(
      0.0,
      1.0,
    );
  }

  @override
  HuntPlan? get plan {
    final (start, length) = _run();

    if (length * HuntPlan.sliceWidth < RoomSurvey.neededArc) {
      return null;
    }

    if (length >= HuntPlan.slices) {
      return HuntPlan.open;
    }

    final from = HuntPlan.bearingOf(start) - HuntPlan.sliceWidth / 2;

    return HuntPlan.between(from, from + length * HuntPlan.sliceWidth);
  }

  @override
  HuntPlan get planUnread => HuntPlan.open;

  @override
  Object? get failure => null;

  @override
  void dispose() {}

  (int, int) _run() => widestRun([
    for (var slice = 0; slice < HuntPlan.slices; slice++) read(slice) >= 1,
  ]);
}
