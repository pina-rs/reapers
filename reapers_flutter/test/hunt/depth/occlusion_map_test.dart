import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/dev/drawn_room_survey.dart';
import 'package:reapers_flutter/hunt/scan/depth/occlusion_map.dart';
import 'package:reapers_flutter/hunt/scan/depth/room_map.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../support/drawn_room.dart';

/// Whether the room hides [point], in metres from the eye.
bool _hides(OcclusionMap map, Vec3 point) =>
    map.hides(point.yaw, point.pitch, point.length);

/// Reads [room] the way a hunt does, at a model that reads everything
/// [stretch] times too long.
OcclusionMap _read(DrawnRoom room, {double stretch = 1}) {
  final map = RoomMap(vessel: Vessel.ape);
  final occlusion = OcclusionMap();

  for (var yaw = -60.0; yaw <= 60; yaw += 10) {
    final frame = room.picture(yaw: yaw, pitch: -30, stretch: stretch);

    if (map.add(frame)) {
      occlusion.add(
        frame,
        floor: map.lastPictureFloor!,
        scale: map.floorDepth! / map.lastPictureFloor!,
      );
    }
  }

  return occlusion;
}

void main() {
  const eye = 1.45;

  group('what the room hides', () {
    test('a husk behind the counter is hidden below its top', () {
      final room = _read(drawnKitchen);

      expect(_hides(room, const Vec3(0, -eye + 0.3, 3.4)), isTrue);
      expect(_hides(room, const Vec3(0.6, -eye + 0.6, 3.4)), isTrue);
    });

    test('what rises above the counter is seen over it', () {
      final room = _read(drawnKitchen);

      expect(_hides(room, const Vec3(0, -eye + 1.4, 3.4)), isFalse);
    });

    test('in front of the counter, nothing is hidden', () {
      final room = _read(drawnKitchen);

      expect(_hides(room, const Vec3(0, -eye + 0.05, 1.8)), isFalse);
      expect(_hides(room, const Vec3(-0.5, -eye + 0.5, 1.6)), isFalse);
    });

    test('the floor never hides a husk standing on it', () {
      final room = _read(const DrawnRoom(wall: 4.5));

      expect(_hides(room, const Vec3(0, -eye + 0.02, 3.5)), isFalse);
      expect(_hides(room, const Vec3(1, -eye + 0.02, 2.5)), isFalse);
    });

    test('a model that reads long still puts the counter in front', () {
      final room = _read(drawnKitchen, stretch: 1.25);

      expect(_hides(room, const Vec3(0, -eye + 0.3, 3.4)), isTrue);
      expect(_hides(room, const Vec3(0, -eye + 0.05, 1.8)), isFalse);
    });

    test('where nothing was seen, nothing is hidden', () {
      final room = _read(drawnKitchen);

      expect(room.distanceAt(math.pi, 0), isNull);
      expect(_hides(room, const Vec3(0, -eye + 0.3, -3.4)), isFalse);
    });
  });

  test('the shader reads the same distances the map holds', () {
    final room = _read(drawnKitchen);
    final pixels = room.toPixels();
    const towards = Vec3(0, -eye + 0.4, 2.5);
    final column = ((degrees(towards.yaw) + 180) / OcclusionMap.cellDegrees)
        .floor();
    final row =
        ((OcclusionMap.highest - degrees(towards.pitch)) /
                OcclusionMap.cellDegrees)
            .floor();
    final at = (row * OcclusionMap.columns + column) * 4;
    final centimetres = pixels[at] << 8 | pixels[at + 1];

    expect(pixels[at + 2], 255, reason: 'something stands there');
    expect(
      centimetres / 100,
      closeTo(room.distanceAt(towards.yaw, towards.pitch)!, 0.01),
    );
  });

  test('a room drawn by hand is read like a real one', () {
    final survey = DrawnRoomSurvey(vessel: Vessel.ape, room: drawnKitchen);
    final plan = survey.planWithin(0, radians(80));

    expect(plan.knowsCover, isTrue);
    expect(plan.isWellFormed, isTrue);
    expect(survey.occlusion.pictures, greaterThan(10));
    expect(_hides(survey.occlusion, const Vec3(0, -eye + 0.3, 3.4)), isTrue);
  });
}
