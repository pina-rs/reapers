import 'package:reapers_shared/reapers_shared.dart';

import '../hunt_arena.dart';
import 'depth/depth_model.dart';
import 'depth/depth_survey.dart';
import 'room_reader.dart';
import 'room_survey.dart';

/// Gets ready the best reading of the room this phone can make for a hunt
/// in [arena] for [vessel].
///
/// A phone that feels its own motion, with a camera and something to run
/// the depth model on, sees the room. Anything else, such as a simulator
/// or a phone that refused the camera, goes by where the player has
/// looked.
Future<RoomSurvey> openRoomSurvey({
  required HuntArena arena,
  required Vessel vessel,
}) async {
  await arena.feed.settled;

  final canSee =
      arena.feed.controller.value != null && arena.poseSource.followsDevice;

  if (canSee) {
    final model = await DepthModel.load();

    if (model != null) {
      return DepthSurvey(
        camera: arena.feed,
        model: model,
        pose: () => arena.pose,
        vessel: vessel,
      );
    }
  }

  return RoomReader(gait: vessel.gait);
}
