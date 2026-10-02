import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

double _range(HuskPose pose) => math.sqrt(
  pose.position.x * pose.position.x + pose.position.z * pose.position.z,
);

/// Steps [sim] with [input] until [stop] holds, and returns what happened.
List<HuntEvent> _until(
  HuntSim sim,
  bool Function() stop, {
  HuntInput Function(HuntSim sim) input = waryOnHusk,
}) {
  final events = <HuntEvent>[];

  while (!stop() && !sim.isOver && sim.tick < HuntLog.maxTicks) {
    events.addAll(sim.step(input(sim)));
  }

  return events;
}

HuntInput _still(HuntSim sim) =>
    HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 0);

/// A hunt for an epic husk, which nothing here will break, brought to the
/// moment the husk starts its first charge.
HuntSim _charging({Vessel vessel = Vessel.ape, int seed = 4}) {
  final sim = simFor(seed: seed, vessel: vessel, rarity: Rarity.epic);

  _until(sim, () => sim.phase == HuntPhase.charge);

  expect(sim.phase, HuntPhase.charge, reason: 'it should charge in the end');

  return sim;
}

void main() {
  group('a charge', () {
    test('comes in the fight, never in the stalk', () {
      final sim = simFor(vessel: Vessel.ape, rarity: Rarity.epic);
      final stalk = _until(sim, () => sim.phase != HuntPhase.stalk);

      expect(stalk.whereType<ChargeBegan>(), isEmpty);

      final fight = _until(sim, () => sim.phase == HuntPhase.charge);

      expect(fight.whereType<ChargeBegan>(), hasLength(1));
    });

    for (final vessel in Vessel.values.where((vessel) => vessel.isSolo)) {
      test('brings the ${vessel.name} to within arm\'s reach, facing you', () {
        final sim = _charging(vessel: vessel);
        final before = _range(sim.pose);

        _until(sim, () => sim.chargeProgress > 0.97, input: _still);

        expect(before, greaterThan(HuntRules.chargeMetres));
        expect(_range(sim.pose), closeTo(HuntRules.chargeMetres, 0.15));
        expect(sim.pose.facesPlayer, isTrue);

        if (vessel.gait == Gait.air) {
          expect(
            sim.pose.position.y.abs(),
            lessThan(0.15),
            reason: 'eye level',
          );
        }
      });
    }

    test('that is not met costs resolve, and the husk runs off', () {
      final sim = _charging();
      final resolve = sim.resolve;
      final events = _until(
        sim,
        () => sim.phase != HuntPhase.charge,
        input: _still,
      );

      expect(events.whereType<Struck>().single.resolveLeft, resolve - 1);
      expect(sim.resolve, resolve - 1);
      expect(events.whereType<Relocating>(), hasLength(1));
      expect(sim.staggered, isFalse);
    });

    test('is turned aside by a swing of the phone', () {
      final sim = _charging();
      final resolve = sim.resolve;
      final events = _until(sim, () => sim.phase != HuntPhase.charge);

      expect(events.whereType<Parried>(), hasLength(1));
      expect(events.whereType<Struck>(), isEmpty);
      expect(sim.resolve, resolve);
    });

    test('turned aside leaves the husk stunned with its seals open', () {
      final sim = _charging();

      _until(sim, () => sim.phase != HuntPhase.charge);

      final where = sim.pose.position;
      final open = [
        for (var seal = 0; seal < sim.vessel.seals.length; seal++)
          if (sim.sealOpen(seal)) seal,
      ];

      expect(sim.staggered, isTrue);
      expect(open.length, greaterThanOrEqualTo(2));

      for (var tick = 0; tick < HuntRules.staggerTicks - 2; tick++) {
        sim.step(_still(sim));
      }

      expect((sim.pose.position - where).length, lessThan(1e-9));

      // Then it is off.
      final after = _until(sim, () => !sim.staggered && sim.speed > 0);

      expect(after.whereType<Relocating>(), hasLength(1));
    });

    test('a swing in its first moments does not count', () {
      final sim = _charging();

      for (var tick = 0; tick < HuntRules.chargeGraceTicks - 1; tick++) {
        final events = sim.step(
          HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 5),
        );

        expect(events.whereType<Parried>(), isEmpty);
      }

      expect(sim.phase, HuntPhase.charge);
    });

    test('a slow turn is not a swing', () {
      final sim = _charging();
      final events = _until(
        sim,
        () => sim.phase != HuntPhase.charge,
        input: (sim) => HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: sim.stats.parrySpeed - 0.1,
        ),
      );

      expect(events.whereType<Parried>(), isEmpty);
      expect(events.whereType<Struck>(), hasLength(1));
    });

    test('struck with no resolve left, the reaper loses the husk', () {
      final sim = simFor(vessel: Vessel.ape, rarity: Rarity.epic);

      _until(sim, () => sim.isOver, input: _still);

      // Standing still is right for a shriek and wrong for a charge, so
      // the charges are what wear this reaper down.
      expect(sim.outcome!.escape, EscapeCause.resolve);
      expect(sim.resolve, 0);
    });

    test('never comes with a shriek: they ask opposite things', () {
      for (var seed = 1; seed <= 8; seed++) {
        final sim = simFor(
          seed: seed,
          vessel: Vessel.llama,
          rarity: Rarity.epic,
        );
        var lastShriekEnd = -1000;
        var lastChargeEnd = -1000;
        var previous = sim.phase;

        while (!sim.isOver) {
          final events = sim.step(waryOnHusk(sim));

          if (events.any((event) => event is ChargeBegan)) {
            expect(
              sim.tick - lastShriekEnd,
              greaterThanOrEqualTo(HuntRules.chargeClearTicks),
              reason: 'seed $seed: a charge straight after a shriek',
            );
          }

          if (events.any((event) => event is ShriekWindup)) {
            expect(
              sim.tick - lastChargeEnd,
              greaterThanOrEqualTo(HuntRules.chargeClearTicks),
              reason: 'seed $seed: a shriek straight after a charge',
            );
          }

          if (previous == HuntPhase.shriek && sim.phase != HuntPhase.shriek) {
            lastShriekEnd = sim.tick;
          }

          if (previous == HuntPhase.charge && sim.phase != HuntPhase.charge) {
            lastChargeEnd = sim.tick;
          }

          previous = sim.phase;
        }
      }
    });

    test('a rift\'s husk does not charge', () {
      final sim = HuntSim.rift(
        seed: 3,
        vessel: Vessel.reliquaryStack,
        ward: 100000,
        stats: HuntStats.derive(HuntLoadout.borrowed),
      );
      final events = _until(sim, () => sim.isOver, input: _still);

      expect(events.whereType<ChargeBegan>(), isEmpty);
    });

    test('gives more warning on easy, and asks for less of a swing', () {
      int ticksOfWarning(Difficulty difficulty) {
        final sim = HuntSim(
          seed: 4,
          vessel: Vessel.ape,
          ward: Rarity.epic.ward,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          difficulty: difficulty,
        );

        _until(sim, () => sim.phase == HuntPhase.charge);

        final start = sim.tick;

        _until(sim, () => sim.phase != HuntPhase.charge, input: _still);

        return sim.tick - start;
      }

      expect(
        ticksOfWarning(Difficulty.easy),
        greaterThan(ticksOfWarning(Difficulty.normal) + 30),
      );

      final stats = HuntStats.derive(HuntLoadout.borrowed);

      expect(stats.at(Difficulty.easy).parrySpeed, lessThan(stats.parrySpeed));
      expect(
        stats.at(Difficulty.hard).parrySpeed,
        greaterThan(stats.parrySpeed),
      );
    });

    test('the bot meets a charge the way a careful person would', () {
      final sim = simFor(vessel: Vessel.ape, rarity: Rarity.rare, level: 6);
      final bot = HuntBot.human(sim)..play();

      expect(bot.events.whereType<ChargeBegan>(), isNotEmpty);
      expect(bot.events.whereType<Struck>(), isEmpty);
      expect(bot.events.whereType<Parried>(), isNotEmpty);
    });
  });
}
