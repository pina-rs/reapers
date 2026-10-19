import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/scan/depth/room_map.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../support/drawn_room.dart';

/// The slice that holds the bearing [degreesRight].
int _sliceAt(double degreesRight) => HuntPlan.sliceOf(radians(degreesRight));

/// A kitchen counter 92 cm high across the room ahead, 2.2 m away and 60 cm
/// deep, with the far wall 4.5 m away.
const _kitchen = DrawnRoom(
  wall: 4.5,
  blocks: [Block(left: -1.6, right: 1.6, near: 2.2, far: 2.8, height: 0.92)],
);

RoomMap _swept(
  DrawnRoom room, {
  Vessel vessel = Vessel.ape,
  double pitch = -30,
}) {
  final map = RoomMap(vessel: vessel);

  sweep(map, room, from: -70, to: 70, pitch: pitch, every: 10);

  return map;
}

void main() {
  group('what stands in the room', () {
    test('a counter is as tall as it is, and its top is a ledge', () {
      final map = _swept(_kitchen);
      final counter = map.furniture(_sliceAt(0))!;

      expect(counter.front, closeTo(2.2, 0.2));
      expect(counter.cover, closeTo(0.92, 0.08));
      expect(counter.ledge, isTrue);
    });

    test('there is room to hide behind it, short of the wall', () {
      final map = _swept(_kitchen);
      final hide = map.furniture(_sliceAt(0))!.hide;

      expect(hide, isNotNull);
      expect(hide, greaterThan(2.8 + Vessel.ape.halfWidth - 0.15));
      expect(hide, lessThan(4.5 - Vessel.ape.halfWidth + 0.1));
    });

    test('there is no room behind a counter against the wall', () {
      const room = DrawnRoom(
        wall: 3,
        blocks: [
          Block(left: -1.6, right: 1.6, near: 2.2, far: 2.8, height: 0.92),
        ],
      );
      final map = _swept(room);

      expect(map.furniture(_sliceAt(0))!.hide, isNull);
    });

    test('nothing is known to be behind a cupboard taller than the eye', () {
      const room = DrawnRoom(
        wall: 4.5,
        blocks: [Block(left: -0.8, right: 0.8, near: 2, far: 2.6, height: 2.1)],
      );
      final map = _swept(room);
      final cupboard = map.furniture(_sliceAt(0))!;

      // Nothing above the eye is charted: it is as tall as a wall.
      expect(cupboard.cover, greaterThan(1.3));
      expect(cupboard.ledge, isFalse);
      expect(cupboard.hide, isNull);
    });

    test('where nothing stands, the way ends at a wall: no ledge, no room', () {
      final map = _swept(const DrawnRoom(wall: 4.5));
      final wall = map.furniture(_sliceAt(0))!;

      expect(wall.front, closeTo(4.5, 0.3));
      expect(wall.cover, greaterThan(1.3));
      expect(wall.ledge, isFalse);
      expect(wall.hide, isNull);
    });

    test('the plan carries all of it, and a hunt can be played in it', () {
      final map = _swept(_kitchen);
      final plan = map.planWithin(0, radians(70));

      expect(plan.isWellFormed, isTrue);
      expect(plan.knowsCover, isTrue);
      expect(plan.coverAt(0), closeTo(0.92, 0.08));
      expect(plan.ledgeAt(_sliceAt(0)), isNotNull);
      expect(plan.hideAt(_sliceAt(0)), greaterThan(plan.reach(0)));

      final decoded = HuntLog.decode(HuntLog(const [], plan: plan).encode());

      expect(decoded.plan, plan);
    });

    test('a husk in the kitchen leaps onto the counter and behind it', () {
      final plan = _swept(_kitchen).planWithin(0, radians(70));
      final footings = <Footing>{};

      for (var seed = 1; seed <= 10; seed++) {
        final sim = HuntSim(
          seed: seed,
          vessel: Vessel.ape,
          ward: Rarity.epic.ward,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          plan: plan,
        );
        final bot = HuntBot.human(sim);

        while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
          bot.stepOnce();
          footings.add(sim.footing);
        }
      }

      expect(footings, containsAll(Footing.values));
    });
  });

  group('for a husk that flies', () {
    test('a counter top is somewhere to perch, and it keeps in front', () {
      final map = _swept(_kitchen, vessel: Vessel.bird, pitch: -12);
      final counter = map.furniture(_sliceAt(0))!;

      expect(counter.ledge, isTrue);
      expect(counter.cover, closeTo(0.92, 0.08));
      expect(counter.hide, isNull);
      expect(map.reach(_sliceAt(0)), closeTo(counter.front, 1e-9));
      expect(map.planWithin(0, radians(70)).isWellFormed, isTrue);
    });
  });
}
