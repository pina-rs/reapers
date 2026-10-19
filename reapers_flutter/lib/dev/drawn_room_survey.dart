import 'package:reapers_shared/reapers_shared.dart';

import '../hunt/scan/depth/occlusion_map.dart';
import '../hunt/scan/depth/room_map.dart';
import '../hunt/scan/room_survey.dart';
import 'drawn_room.dart';

/// A reading of a room drawn by hand, for a phone or a simulator that
/// cannot see a real one.
///
/// It is read the way a real room is, through [RoomMap] and [OcclusionMap],
/// from the pictures a perfect depth model would take of it, so a husk in it
/// takes cover, leaps and hides exactly as it would in a real one. Only for
/// building and testing the game: `--dart-define=REAPERS_DRAWN_ROOM=true`.
final class DrawnRoomSurvey implements RoomSurvey {
  DrawnRoomSurvey({required Vessel vessel, required this.room})
    : _map = RoomMap(vessel: vessel) {
    // Looked at all the way round, low and level, as a careful player
    // tracing a long sigil would.
    for (final pitch in [-30.0, 5.0]) {
      for (var yaw = -180.0; yaw < 180; yaw += 15) {
        final frame = room.picture(yaw: yaw, pitch: pitch);

        if (!_map.add(frame)) {
          continue;
        }

        final pictureFloor = _map.lastPictureFloor!;

        occlusion.add(
          frame,
          floor: pictureFloor,
          scale: _map.floorDepth! / pictureFloor,
        );
      }
    }
  }

  /// Set with `--dart-define=REAPERS_DRAWN_ROOM=true`.
  static const enabled = bool.fromEnvironment('REAPERS_DRAWN_ROOM');

  final DrawnRoom room;

  final RoomMap _map;

  @override
  final OcclusionMap occlusion = OcclusionMap();

  @override
  Object? get failure => null;

  @override
  double read(int index) => _map.read(index);

  @override
  double? reach(int index) => _map.reach(index);

  @override
  bool tight(int index) => _map.tight(index);

  @override
  HuntPlan planWithin(double centre, double halfWidth) =>
      _map.planWithin(centre, halfWidth);

  @override
  void dispose() {}
}
