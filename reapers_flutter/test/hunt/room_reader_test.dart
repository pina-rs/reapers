import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/scan/room_reader.dart';
import 'package:reapers_shared/reapers_shared.dart';

const _frame = 1 / 60;

/// The phone pointing [yawDegrees] round and [pitchDegrees] up.
ArPose _looking(
  double yawDegrees,
  double pitchDegrees, {
  double turnSpeed = 0.3,
}) => ArPose.looking(
  Vec3.fromAngles(radians(yawDegrees), radians(pitchDegrees)),
  turnSpeed: turnSpeed,
);

/// Sweeps [reader] from [from] to [to] degrees at [pitch], at a steady
/// [degreesPerSecond].
void _sweep(
  RoomReader reader, {
  required double from,
  required double to,
  required double pitch,
  double degreesPerSecond = 30,
}) {
  final frames = ((to - from).abs() / degreesPerSecond / _frame).ceil();

  for (var frame = 0; frame <= frames; frame++) {
    reader.observe(
      _looking(
        from + (to - from) * frame / frames,
        pitch,
        turnSpeed: radians(degreesPerSecond),
      ),
      _frame,
    );
  }
}

void main() {
  group('reading the floor for a husk that walks', () {
    test('nothing is known before the player looks', () {
      final reader = RoomReader(gait: Gait.ground);

      expect(reader.progress, 0);
      expect(reader.plan, isNull);
    });

    test('looking straight ahead reads nothing and says to look lower', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -90, to: 90, pitch: 0);

      expect(reader.progress, 0);
      expect(reader.advice, ReadingAdvice.lookLower);
    });

    test('looking at your own feet reads nothing and says to look up', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -90, to: 90, pitch: -75);

      expect(reader.progress, 0);
      expect(reader.advice, ReadingAdvice.lookHigher);
    });

    test('whipping the phone round reads nothing and says to slow down', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -90, to: 90, pitch: -25, degreesPerSecond: 200);

      expect(reader.progress, 0);
      expect(reader.advice, ReadingAdvice.slowDown);
    });

    test('a slow sweep across the floor is read as it goes', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -30, to: 30, pitch: -25);

      expect(reader.advice, ReadingAdvice.keepGoing);
      expect(reader.progress, inExclusiveRange(0.3, 1));
      expect(reader.plan, isNull, reason: 'not enough of the room yet');
    });

    test('once enough is read, the plan is the part that was swept', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -70, to: 70, pitch: -25);

      final plan = reader.plan!;

      expect(reader.progress, 1);
      expect(plan.isPlayable, isTrue);
      expect(plan.centreCentidegrees.abs(), lessThan(800));
      expect(plan.halfWidthCentidegrees, inInclusiveRange(6000, 8500));
      expect(plan.contains(radians(60)), isTrue);
      expect(plan.contains(radians(-60)), isTrue);
      expect(plan.contains(radians(120)), isFalse, reason: 'never looked');
      expect(plan.contains(radians(180)), isFalse);
    });

    test('two patches that do not join are not one plan', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -100, to: -40, pitch: -25);
      _sweep(reader, from: 40, to: 100, pitch: -25);

      expect(reader.plan, isNull);
      expect(reader.progress, lessThan(1));
    });

    test('a sweep can run across straight behind', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: 110, to: 250, pitch: -25);

      final plan = reader.plan!;

      expect(plan.contains(radians(180)), isTrue);
      expect(plan.contains(radians(-150)), isTrue);
      expect(plan.contains(radians(0)), isFalse);
    });

    test('turning all the way round opens the whole circle', () {
      final reader = RoomReader(gait: Gait.ground);

      _sweep(reader, from: -200, to: 200, pitch: -25);

      expect(reader.plan, HuntPlan.open);
    });
  });

  group('reading the air for a husk that flies', () {
    test('looking at the floor reads nothing and says to look up', () {
      final reader = RoomReader(gait: Gait.air);

      _sweep(reader, from: -70, to: 70, pitch: -40);

      expect(reader.progress, 0);
      expect(reader.advice, ReadingAdvice.lookHigher);
    });

    test('looking around at head height is what it wants', () {
      final reader = RoomReader(gait: Gait.air);

      _sweep(reader, from: -70, to: 70, pitch: 5);

      expect(reader.plan, isNotNull);
    });
  });

  test('a plan read from a room is one the rules will play in', () {
    final reader = RoomReader(gait: Gait.ground);

    _sweep(reader, from: -70, to: 70, pitch: -25);

    final sim = HuntSim(
      seed: 3,
      vessel: Vessel.penguin,
      ward: Rarity.common.ward,
      stats: HuntStats.derive(HuntLoadout.borrowed),
      plan: reader.plan!,
    );

    expect(reader.plan!.contains(sim.pose.yaw), isTrue);
  });
}
