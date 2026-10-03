import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/scan/depth/room_map.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../support/drawn_room.dart';

/// The slice that holds the bearing [degreesRight].
int _sliceAt(double degreesRight) =>
    ((radians(degreesRight) + radians(180)) / HuntPlan.sliceWidth).floor();

/// The plan for a hunt kept to [degreesEitherSide] of straight ahead.
HuntPlan _ahead(RoomMap map, double degreesEitherSide) =>
    map.planWithin(0, radians(degreesEitherSide));

/// A sofa: 80 cm tall, a metre and a half ahead and a little to the right.
const _sofa = Block(left: 0.4, right: 2, near: 1.5, far: 2.3, height: 0.8);

void main() {
  group('finding the floor', () {
    test('standing, the floor is where it really is', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, const DrawnRoom(), from: -60, to: 60, pitch: -30);

      expect(map.floorDepth, closeTo(1.45, 0.03));
    });

    test('sitting down, the floor comes up to meet the phone', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(
        map,
        const DrawnRoom(eyeHeight: 0.95),
        from: -60,
        to: 60,
        pitch: -30,
      );

      expect(map.floorDepth, closeTo(0.95, 0.03));
      expect(_ahead(map, 60).eyeCentimetres, inInclusiveRange(92, 98));
    });

    test('a table top is not taken for the floor', () {
      // A big table across the far side of the view, with floor in front.
      const room = DrawnRoom(
        blocks: [
          Block(left: -2, right: 2, near: 1.8, far: 3.4, height: 0.75),
        ],
      );
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, room, from: -40, to: 40, pitch: -32);

      expect(map.floorDepth, closeTo(1.45, 0.03));
    });

    test('depth that wobbles from pixel to pixel still finds it', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, const DrawnRoom(), from: -60, to: 60, pitch: -30, noise: 0.03);

      expect(map.floorDepth, closeTo(1.45, 0.06));
      expect(map.reach(_sliceAt(0)), isNotNull);
    });

    test('a picture that takes a table for the floor is set aside', () {
      // A table right across the view, so near that no floor shows at all.
      const table = DrawnRoom(
        blocks: [Block(left: -3, right: 3, near: 0.4, far: 4, height: 0.75)],
      );
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, const DrawnRoom(), from: -40, to: 20, pitch: -30);

      final before = map.frames;

      expect(map.add(table.picture(yaw: 40, pitch: -30)), isFalse);
      expect(map.frames, before);
      expect(map.floorDepth, closeTo(1.45, 0.03));
    });

    test('a picture with no floor in it teaches nothing', () {
      // Level with a wall a metre away, there is no floor in view at all.
      const room = DrawnRoom(wall: 1);
      final map = RoomMap(vessel: Vessel.penguin);

      expect(map.add(room.picture(yaw: 0, pitch: 25)), isFalse);
      expect(map.frames, 0);
      expect(map.floorDepth, isNull);
      expect(_ahead(map, 60).isSurveyed, isFalse);
      expect(_ahead(map, 60).eyeCentimetres, HuntRules.eyeCentimetres);
    });
  });

  group('how far the way is clear', () {
    test('an open floor is clear as far as the wall', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, const DrawnRoom(), from: -60, to: 60, pitch: -30);

      expect(map.reach(_sliceAt(0)), closeTo(4, 0.25));
      expect(map.reach(_sliceAt(-120)), isNull, reason: 'never looked there');
      expect(map.read(_sliceAt(-120)), 0);
      expect(map.tight(_sliceAt(0)), isFalse);
    });

    test('a sofa is as far as the way is clear in its direction', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(
        map,
        const DrawnRoom(blocks: [_sofa]),
        from: -150,
        to: 60,
        pitch: -30,
      );

      // Straight at the sofa, the way is clear up to its front.
      expect(map.reach(_sliceAt(30)), inInclusiveRange(1.5, 1.9));
      // To the left of it, all the way to the wall.
      expect(map.reach(_sliceAt(-40)), greaterThan(3.5));
    });

    test('where there is no room the husk may still go, and stands close', () {
      // The sofa is too near for a penguin to keep its distance from.
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(
        map,
        const DrawnRoom(
          blocks: [
            Block(left: 0.3, right: 2, near: 0.9, far: 2, height: 0.8),
          ],
        ),
        from: -70,
        to: 70,
        pitch: -30,
      );

      final plan = _ahead(map, 70);

      expect(map.tight(_sliceAt(35)), isTrue);
      expect(map.tight(_sliceAt(-40)), isFalse);
      expect(plan.contains(radians(35)), isTrue);
      expect(plan.reach(radians(35)), lessThan(1.2));
      expect(plan.isWellFormed, isTrue);
    });

    test('with room in front of it, the husk may go there and no farther', () {
      const room = DrawnRoom(
        blocks: [
          Block(left: 0.4, right: 2.4, near: 2.6, far: 3.4, height: 0.8),
        ],
      );
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, room, from: -70, to: 70, pitch: -30);

      final plan = _ahead(map, 70);

      expect(plan.contains(radians(25)), isTrue);
      expect(plan.reach(radians(25)), inInclusiveRange(2.5, 3.0));
      // Well to the side of it there is more room, though the edge of what
      // was swept, where only the nearest floor was seen, holds it in a
      // little.
      expect(plan.reach(radians(-40)), greaterThan(3.2));
    });

    test('a model that reads everything long still lands on the floor', () {
      // Every distance 20% too long. The floor and the wall are both out by
      // the same share, so the wall is still where the picture shows it:
      // the same number of eye-heights away.
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(
        map,
        const DrawnRoom(),
        from: -60,
        to: 60,
        pitch: -30,
        stretch: 1.2,
      );

      expect(map.floorDepth, closeTo(1.45 * 1.2, 0.04));
      expect(map.reach(_sliceAt(0))! / map.floorDepth!, closeTo(4 / 1.45, 0.2));
    });

    test('pictures that disagree about scale are brought into step', () {
      // One picture reads long and the next reads short. Measured against
      // its own floor, each puts the wall the same number of eye-heights
      // away.
      final map = RoomMap(vessel: Vessel.penguin);
      const room = DrawnRoom();

      for (final (yaw, stretch) in [
        (-40.0, 1.15),
        (-20.0, 0.9),
        (0.0, 1.1),
        (20.0, 0.92),
        (40.0, 1.0),
      ]) {
        map.add(room.picture(yaw: yaw, pitch: -30, stretch: stretch));
      }

      for (final bearing in [-30.0, 0.0, 30.0]) {
        // The wall is 4 m ahead, so farther off to either side.
        final wall = 4 / math.cos(radians(bearing));

        expect(
          map.reach(_sliceAt(bearing))! / map.floorDepth!,
          closeTo(wall / 1.45, 0.3),
          reason: 'at $bearing°',
        );
      }
    });
  });

  group('the plan', () {
    test('it keeps to the arc it is asked for', () {
      final map = RoomMap(vessel: Vessel.penguin);

      sweep(map, const DrawnRoom(), from: -60, to: 60, pitch: -30);

      final plan = map.planWithin(radians(90), radians(65));

      expect(plan.centre, closeTo(radians(90), 1e-3));
      expect(plan.halfWidth, closeTo(radians(65), 1e-3));
    });

    test('a direction never looked in is left open', () {
      final map = RoomMap(vessel: Vessel.penguin);

      map.add(const DrawnRoom().picture(yaw: 0, pitch: -30));

      final plan = _ahead(map, 90);

      expect(plan.isSurveyed, isTrue);
      expect(
        plan.reachDecimetres[_sliceAt(-80)],
        HuntPlan.maxReachDecimetres,
      );
      expect(plan.reachDecimetres[_sliceAt(0)], lessThan(50));
    });

    test('a swept room gives a plan a hunt can be played in', () {
      final map = RoomMap(vessel: Vessel.ape);

      sweep(map, const DrawnRoom(), from: -70, to: 70, pitch: -30);

      final plan = _ahead(map, 70);

      expect(plan.isSurveyed, isTrue);
      expect(plan.isWellFormed, isTrue);
      expect(plan.eyeCentimetres, inInclusiveRange(142, 148));
      expect(plan.contains(radians(50)), isTrue);
      expect(plan.contains(radians(-50)), isTrue);
      expect(plan.contains(radians(150)), isFalse, reason: 'not asked for');

      // The rules take it, and it survives being written down and read back.
      final log = HuntLog.decode(HuntLog(const [], plan: plan).encode());

      expect(log.plan, plan);
      expect(
        () => HuntSim(
          seed: 5,
          vessel: Vessel.ape,
          ward: Rarity.common.ward,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          plan: log.plan,
        ),
        returnsNormally,
      );
    });

    test('after a single picture, a hunt still gets the floor', () {
      final map = RoomMap(vessel: Vessel.penguin);

      expect(_ahead(map, 60).eyeCentimetres, HuntRules.eyeCentimetres);

      map.add(const DrawnRoom(eyeHeight: 1.1).picture(yaw: 0, pitch: -30));

      expect(_ahead(map, 60).eyeCentimetres, inInclusiveRange(107, 113));
    });

    test('a room too small all round still gives a plan, all of it tight', () {
      final map = RoomMap(vessel: Vessel.ape);

      sweep(map, const DrawnRoom(wall: 1.1), from: -70, to: 70, pitch: -35);

      final plan = _ahead(map, 70);

      expect(map.tight(_sliceAt(0)), isTrue);
      expect(plan.reach(0), lessThan(1.3));

      // The rules take it, and the husk stands as close as it ever does.
      final sim = HuntSim(
        seed: 5,
        vessel: Vessel.ape,
        ward: Rarity.common.ward,
        stats: HuntStats.derive(HuntLoadout.borrowed),
        plan: plan,
      );
      final flat = math.sqrt(
        math.pow(sim.pose.position.x, 2) + math.pow(sim.pose.position.z, 2),
      );

      expect(flat, closeTo(HuntRules.closestMetres, 1e-9));
    });
  });

  group('a husk that flies', () {
    test('is given the air as far as the walls', () {
      final map = RoomMap(vessel: Vessel.bird);

      sweep(map, const DrawnRoom(wall: 3), from: -70, to: 70, pitch: 5);

      final plan = _ahead(map, 70);

      expect(map.reach(_sliceAt(0)), closeTo(3, 0.2));
      expect(plan.isWellFormed, isTrue);
      expect(plan.reach(0), closeTo(3, 0.2));
    });

    test('is kept from a cupboard that reaches up into its air', () {
      const room = DrawnRoom(
        blocks: [
          Block(left: -0.6, right: 0.6, near: 1.3, far: 2, height: 2.2),
        ],
      );
      final map = RoomMap(vessel: Vessel.bird);

      sweep(map, room, from: -130, to: 70, pitch: 5);

      expect(map.reach(_sliceAt(0)), closeTo(1.3, 0.15));
      expect(map.tight(_sliceAt(0)), isFalse);
      expect(_ahead(map, 70).reach(0), closeTo(1.3, 0.2));
    });
  });
}
