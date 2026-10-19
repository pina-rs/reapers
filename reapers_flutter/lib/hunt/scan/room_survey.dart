import 'package:reapers_shared/reapers_shared.dart';

import '../pose/ar_pose.dart';
import 'depth/occlusion_map.dart';

/// A reading of the room, made while the player traces the husk's sigil,
/// that ends in a plan of where the husk may go.
///
/// The circle around the player is divided into the slices of a
/// [HuntPlan], and a survey fills them in as the camera looks into them.
/// The sigil decides when the reading is over and which arc the husk keeps
/// to. The survey says what it learnt of that arc.
abstract interface class RoomSurvey {
  /// How well slice [index] has been read, from 0 to 1.
  double read(int index);

  /// How far the way is clear along slice [index], in metres, or null until
  /// it has been read.
  double? reach(int index);

  /// Whether slice [index] has been read and found tight: so little clear
  /// floor that the husk can only stand there as close as it ever comes.
  bool tight(int index);

  /// The plan for a hunt kept to [halfWidth] radians either side of the
  /// bearing [centre], with whatever has been learnt of the room so far.
  HuntPlan planWithin(double centre, double halfWidth);

  /// The room as it looks from where the player stands, for hiding the
  /// husk behind what stands in it, or null for a survey that cannot tell.
  OcclusionMap? get occlusion;

  /// What went wrong, if the survey has broken down and will read no more.
  Object? get failure;

  void dispose();
}

/// The plan for a hunt inside the arc tracing [trace] read, with what
/// [survey] learnt of the room, or nothing of it when there is no survey.
HuntPlan sigilPlan(SigilTrace trace, RoomSurvey? survey) =>
    survey?.planWithin(trace.centre, trace.sigil.halfArc) ??
    HuntPlan.between(
      trace.centre - trace.sigil.halfArc,
      trace.centre + trace.sigil.halfArc,
    );

/// Whether the camera, pointing as [pose] does, can read the room for a
/// husk with [gait]: whether the phone is turning slower than
/// [steadyTurnSpeed] radians a second and pointing at the height that
/// needs reading.
///
/// The floor a walker will stand on lies two to five metres out, which is
/// well below the horizon from standing height. A flier keeps to about head
/// height. A sigil's lights hang inside these bands.
bool canRead(ArPose pose, Gait gait, {required double steadyTurnSpeed}) {
  final (low, high) = switch (gait) {
    Gait.ground => (radians(-52), radians(-7)),
    Gait.air || Gait.still => (radians(-18), radians(38)),
  };
  final pitch = pose.forward.pitch;

  return pose.turnSpeed <= steadyTurnSpeed && pitch >= low && pitch <= high;
}
