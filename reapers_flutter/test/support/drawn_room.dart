import 'package:reapers_flutter/dev/drawn_room.dart';
import 'package:reapers_flutter/hunt/scan/depth/room_map.dart';

export 'package:reapers_flutter/dev/drawn_room.dart';

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
