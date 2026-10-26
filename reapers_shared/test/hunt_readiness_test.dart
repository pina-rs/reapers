import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

void main() {
  test('blade recovery and cadence reflect the actual strike windows', () {
    final sim = simFor();
    final steady = HuntInput.measured(
      aim: sim.pose.direction,
      turnRadiansPerSecond: 0,
    );

    expect(sim.strikeTicksLeft, 0);
    expect(sim.cadenceTicksLeft, 0);

    for (var tick = 0; tick < HuntRules.ambushTicks; tick++) {
      sim.step(steady);
    }

    final seal = [
      for (var index = 0; index < sim.vessel.seals.length; index++)
        if (sim.sealOpen(index)) index,
    ].first;
    final hit = sim.step(
      HuntInput.measured(
        aim: sim.pose.direction,
        turnRadiansPerSecond: 0,
        taps: [sim.pose.sealDirection(seal)],
      ),
    );

    expect(hit.whereType<StrikeLanded>(), hasLength(1));
    expect(sim.strikeTicksLeft, sim.stats.swingTicks - 1);
    expect(sim.cadenceTicksLeft, HuntRules.cadenceWindowTicks - 1);

    for (var tick = 0; tick < sim.stats.swingTicks; tick++) {
      sim.step(steady);
    }

    expect(sim.strikeTicksLeft, 0);
    expect(sim.cadenceTicksLeft, greaterThan(0));

    for (var tick = 0; tick <= HuntRules.cadenceWindowTicks; tick++) {
      sim.step(steady);
    }

    expect(sim.strikeTicksLeft, 0);
    expect(sim.cadenceTicksLeft, 0);
    expect(sim.cadence, 0);

    final miss = sim.step(
      HuntInput.measured(
        aim: Vec3.forward,
        turnRadiansPerSecond: 0,
        taps: [Vec3(0, 1, 0)],
      ),
    );

    expect(miss.whereType<StrikeWhiffed>(), hasLength(1));
    expect(sim.strikeTicksLeft, sim.stats.swingTicks - 1);
    expect(sim.cadenceTicksLeft, 0);
  });
}
