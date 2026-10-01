import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/hunt_controller.dart';
import 'package:reapers_flutter/hunt/pose/pose_sources.dart';
import 'package:reapers_shared/reapers_shared.dart';

HuntSim _sim() => HuntSim(
  seed: 7,
  vessel: Vessel.frameWraith,
  ward: Rarity.common.ward,
  stats: HuntStats.derive(HuntLoadout.borrowed),
);

void main() {
  test('the rules advance at their own rate, whatever the display does', () {
    for (final refresh in [30, 60, 120]) {
      final controller = HuntController(
        sim: _sim(),
        poseSource: DragPoseSource(),
      );

      for (var frame = 0; frame < refresh * 2; frame++) {
        controller.frame(1 / refresh);
      }

      expect(
        controller.sim.tick,
        closeTo(2 * HuntRules.tickRate, 1),
        reason: '$refresh Hz',
      );
      expect(controller.recorded.length, controller.sim.tick);
    }
  });

  test('a tap is played on the next tick and recorded with it', () {
    final controller = HuntController(
      sim: _sim(),
      poseSource: DragPoseSource(),
    );
    final target = controller.sim.pose.direction;

    controller
      ..tap(target)
      ..frame(1 / 60);

    expect(controller.recorded.single.taps, [Aim.fromDirection(target)]);
    expect(controller.sim.phase, HuntPhase.fight, reason: 'struck first');

    controller.frame(1 / 60);

    expect(controller.recorded.last.taps, isEmpty);
  });

  test('a long stall is caught up a little, then dropped', () {
    final controller = HuntController(sim: _sim(), poseSource: DragPoseSource())
      ..frame(3);

    expect(controller.sim.tick, lessThanOrEqualTo(6));

    controller.frame(1 / 60);

    expect(controller.sim.tick, lessThanOrEqualTo(7));
  });

  test('the recorded log replays to the same hunt', () {
    final controller = HuntController(
      sim: _sim(),
      poseSource: DragPoseSource(),
    );
    final source = controller.poseSource;

    for (var frame = 0; frame < 600 && !controller.sim.isOver; frame++) {
      source.drag(0.01, 0);

      if (frame % 20 == 0) {
        controller.tap(controller.sim.pose.direction);
      }

      controller.frame(1 / 60);
    }

    final replayed = _sim();

    for (final input in HuntLog.decode(controller.log.encode()).inputs) {
      replayed.step(input);
    }

    expect(replayed.tick, controller.sim.tick);
    expect(replayed.phase, controller.sim.phase);
    expect(replayed.ward, controller.sim.ward);
  });
}
