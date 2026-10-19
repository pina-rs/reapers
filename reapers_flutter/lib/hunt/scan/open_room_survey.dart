import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../dev/drawn_room.dart';
import '../../dev/drawn_room_survey.dart';
import '../hunt_arena.dart';
import 'depth/depth_model.dart';
import 'depth/depth_survey.dart';
import 'room_survey.dart';

/// Gets ready a reading of the room for a hunt in [arena] for [vessel], or
/// answers null on a phone that cannot read one.
///
/// A phone that feels its own motion, with a camera and something to run
/// the depth model on, sees the room. Anything else, such as a simulator
/// or a phone that refused the camera, knows nothing of the room, and its
/// husk keeps to the arc of its sigil with nothing in its way. Unless it is
/// a build for working on the game asked to play in a room drawn by hand
/// (`--dart-define=REAPERS_DRAWN_ROOM=true`).
Future<RoomSurvey?> openRoomSurvey({
  required HuntArena arena,
  required Vessel vessel,
}) async {
  await arena.feed.settled;

  final canSee =
      arena.feed.controller.value != null && arena.poseSource.followsDevice;

  if (!canSee) {
    if (DrawnRoomSurvey.enabled && !kReleaseMode) {
      arena.drawnRoom = drawnKitchen;

      return DrawnRoomSurvey(vessel: vessel, room: drawnKitchen);
    }

    return null;
  }

  final model = await DepthModel.load();

  if (model == null) {
    return null;
  }

  return DepthSurvey(
    camera: arena.feed,
    model: model,
    pose: () => arena.pose,
    vessel: vessel,
  );
}
