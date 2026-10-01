import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

/// Everything a hunt for [vessel] did, tick by tick, played by the shared
/// bot inside [plan].
final class _Hunt {
  _Hunt(Vessel vessel, {int seed = 3, HuntPlan plan = HuntPlan.open})
    : sim = HuntSim(
        seed: seed,
        vessel: vessel,
        ward: Rarity.uncommon.ward,
        stats: HuntStats.derive(HuntLoadout.borrowed),
        plan: plan,
      ) {
    final bot = HuntBot.human(sim);

    while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
      final before = sim.pose;
      final phase = sim.phase;

      events.add(bot.stepOnce());
      steps.add((before: before, after: sim.pose, phase: phase));
    }
  }

  final HuntSim sim;
  final steps = <({HuskPose before, HuskPose after, HuntPhase phase})>[];
  final events = <List<HuntEvent>>[];
}

double _range(HuskPose pose) => math.sqrt(
  pose.position.x * pose.position.x + pose.position.z * pose.position.z,
);

final _creatures = Vessel.values.where((vessel) => vessel.isSolo).toList();

void main() {
  group('where a husk can be', () {
    for (final vessel in _creatures.where((v) => v.gait == Gait.ground)) {
      test('${vessel.name} keeps its feet on the floor', () {
        final hunt = _Hunt(vessel);
        final floor = -HuntRules.eyeHeight;

        for (final step in hunt.steps) {
          expect(
            step.after.position.y - vessel.halfHeight,
            closeTo(floor, 1e-9),
          );
        }

        expect(hunt.steps.first.after.pitch, lessThan(0), reason: 'look down');
      });
    }

    for (final vessel in _creatures.where((v) => v.gait == Gait.air)) {
      test('${vessel.name} stays in its band of air', () {
        final hunt = _Hunt(vessel);

        for (final step in hunt.steps) {
          expect(
            step.after.position.y,
            inInclusiveRange(vessel.lowMetres - 0.06, vessel.highMetres + 0.06),
          );
        }
      });
    }

    for (final vessel in _creatures) {
      test('${vessel.name} keeps its distance', () {
        for (var seed = 1; seed <= 6; seed++) {
          for (final step in _Hunt(vessel, seed: seed).steps) {
            expect(
              _range(step.after),
              inInclusiveRange(
                vessel.nearMetres - 1e-6,
                vessel.farMetres + 1e-6,
              ),
            );
          }
        }
      });

      test('${vessel.name} never leaves the part of the room it was given', () {
        const plan = HuntPlan(
          centreCentidegrees: 4000,
          halfWidthCentidegrees: 6000,
        );

        for (var seed = 1; seed <= 6; seed++) {
          for (final step in _Hunt(vessel, seed: seed, plan: plan).steps) {
            expect(
              plan.contains(step.after.yaw),
              isTrue,
              reason: 'seed $seed at ${degrees(step.after.yaw)}°',
            );
          }
        }
      });
    }

    test('a plan too narrow to hunt in is refused', () {
      const narrow = HuntPlan(
        centreCentidegrees: 0,
        halfWidthCentidegrees: HuntPlan.minHalfWidthCentidegrees - 1,
      );

      expect(narrow.isPlayable, isFalse);
      expect(
        () => HuntSim(
          seed: 1,
          vessel: Vessel.penguin,
          ward: 280,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          plan: narrow,
        ),
        throwsArgumentError,
      );
      expect(
        HuntSim.replay(
          seed: 1,
          vessel: Vessel.penguin,
          ward: 280,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          log: HuntLog(const [idle], plan: narrow),
        ),
        isNull,
        reason: 'a record played in such a plan proves nothing',
      );
    });

    test('a rift\'s husk stands where it is and faces the circle', () {
      final sim = HuntSim.rift(
        seed: 9,
        vessel: Vessel.reliquaryStack,
        ward: 1e9,
        stats: HuntStats.derive(HuntLoadout.borrowed),
      );
      final start = sim.pose.position;

      for (var tick = 0; tick < 20 * HuntRules.tickRate; tick++) {
        sim.step(idle);

        expect(sim.pose.position, start);
        expect(sim.pose.facesPlayer, isTrue);
      }

      expect(sim.pose.yaw.abs(), lessThan(1e-9));
    });
  });

  group('how a husk moves', () {
    test('it moves smoothly: never more than a step in one tick', () {
      for (final vessel in _creatures) {
        final limit = vessel.boltSpeed / HuntRules.tickRate * 1.6;

        for (final step in _Hunt(vessel).steps) {
          expect(
            (step.after.position - step.before.position).length,
            lessThanOrEqualTo(limit),
            reason: vessel.name,
          );
        }
      }
    });

    test('it turns its body a little at a time', () {
      final limit = radians(HuntRules.turnDegreesPerTick) + 1e-9;

      for (final vessel in _creatures) {
        for (final step in _Hunt(vessel).steps) {
          expect(
            wrapAngle(step.after.heading - step.before.heading).abs(),
            lessThanOrEqualTo(limit),
          );
        }
      }
    });

    test('once it is under way it faces the way it is going', () {
      for (final vessel in _creatures) {
        final hunt = _Hunt(vessel);
        var moving = 0;
        var facing = 0;

        for (final step in hunt.steps) {
          final travel = step.after.position - step.before.position;
          final along = Vec3(travel.x, 0, travel.z);

          if (along.length < 0.4 * vessel.walkSpeed / HuntRules.tickRate) {
            continue;
          }

          moving++;

          if (along.normalized.dot(step.after.forward) > 0.5) {
            facing++;
          }
        }

        expect(moving, greaterThan(60), reason: '${vessel.name} moves');
        expect(
          facing / moving,
          greaterThan(0.7),
          reason: '${vessel.name} mostly faces where it goes',
        );
      }
    });

    test('a husk that notices the reaper bolts', () {
      final sim = simFor(vessel: Vessel.ape);
      final events = <HuntEvent>[];

      // Spinning on the spot is what alarms it.
      while (sim.phase == HuntPhase.stalk) {
        events.addAll(
          sim.step(
            HuntInput.measured(aim: Vec3.forward, turnRadiansPerSecond: 4),
          ),
        );
      }

      expect(events.whereType<StalkEnded>().single.ambush, isFalse);
      expect(events.whereType<Relocating>(), hasLength(1));

      sim.step(idle);

      expect(sim.speed, closeTo(Vessel.ape.boltSpeed, 0.4));
    });

    test('a husk that is ambushed stays where it is', () {
      final sim = simFor(vessel: Vessel.ape);
      final events = <HuntEvent>[];

      while (sim.phase == HuntPhase.stalk) {
        events.addAll(
          sim.step(
            HuntInput.measured(
              aim: sim.pose.direction,
              turnRadiansPerSecond: 0,
            ),
          ),
        );
      }

      expect(events.whereType<StalkEnded>().single.ambush, isTrue);
      expect(events.whereType<Relocating>(), isEmpty);
    });

    test('drawing breath, it stops and turns on the reaper', () {
      for (final vessel in _creatures) {
        final hunt = _Hunt(vessel);
        var shrieks = 0;

        for (var index = 0; index < hunt.steps.length; index++) {
          final step = hunt.steps[index];

          if (step.phase == HuntPhase.shriek) {
            expect(
              (step.after.position - step.before.position).length,
              lessThan(0.002),
              reason: '${vessel.name} holds still to shriek',
            );
          }

          if (hunt.events[index].any((event) => event is ShriekBegan)) {
            shrieks++;

            expect(step.after.facesPlayer, isTrue, reason: vessel.name);
          }
        }

        expect(shrieks, greaterThan(0), reason: vessel.name);
      }
    });

    test('it runs off again some time after each shriek', () {
      final hunt = _Hunt(Vessel.llama, seed: 5);
      final all = hunt.events.expand((events) => events).toList();

      expect(all.whereType<ShriekBegan>(), isNotEmpty);
      expect(
        all.whereType<Relocating>().length,
        greaterThanOrEqualTo(all.whereType<HeldFirm>().length),
      );
    });

    test('a broken husk does not move', () {
      for (final vessel in _creatures) {
        for (final step in _Hunt(vessel).steps) {
          if (step.phase == HuntPhase.broken) {
            // A husk in the air still bobs where it hangs.
            expect(
              (step.after.position - step.before.position).length,
              lessThan(0.002),
            );
          }
        }
      }
    });
  });

  group('seals and the side they are on', () {
    test('a seal on the far side cannot be seen', () {
      // A penguin three metres ahead on the floor, first facing the reaper
      // and then with its back turned.
      const position = Vec3(0, 0.45 - HuntRules.eyeHeight, 3);
      const facing = HuskPose(
        vessel: Vessel.penguin,
        position: position,
        heading: math.pi,
      );
      const turned = HuskPose(
        vessel: Vessel.penguin,
        position: position,
        heading: 0,
      );
      const belly = 1;
      const back = 4;

      expect(facing.facesPlayer, isTrue);
      expect(facing.sealFacesPlayer(belly), isTrue);
      expect(facing.sealFacesPlayer(back), isFalse);
      expect(turned.facesPlayer, isFalse);
      expect(turned.sealFacesPlayer(belly), isFalse);
      expect(turned.sealFacesPlayer(back), isTrue);
    });

    test('a seal on its left is on the reaper\'s right when it faces them', () {
      const pose = HuskPose(
        vessel: Vessel.penguin,
        position: Vec3(0, 0, 3),
        heading: math.pi,
      );
      const leftFlipper = 2;

      expect(Vessel.penguin.seals[leftFlipper].offset.x, greaterThan(0));
      expect(pose.sealPosition(leftFlipper).x, greaterThan(0));
    });

    test('whichever way a husk turns, at least two seals are in view', () {
      for (final vessel in _creatures) {
        for (var turn = 0; turn < 36; turn++) {
          for (final range in [vessel.nearMetres, vessel.farMetres]) {
            final height = vessel.gait == Gait.ground
                ? vessel.halfHeight - HuntRules.eyeHeight
                : (vessel.lowMetres + vessel.highMetres) / 2;
            final pose = HuskPose(
              vessel: vessel,
              position: Vec3(0, height, range),
              heading: radians(turn * 10),
            );
            final seen = [
              for (var seal = 0; seal < vessel.seals.length; seal++)
                if (pose.sealFacesPlayer(seal)) seal,
            ];

            expect(
              seen.length,
              greaterThanOrEqualTo(2),
              reason: '${vessel.name} turned ${turn * 10}° at $range m',
            );
          }
        }
      }
    });
  });

  group('plans', () {
    test('an arc is described by where it starts and ends', () {
      final plan = HuntPlan.between(radians(-30), radians(90));

      expect(plan.centreCentidegrees, 3000);
      expect(plan.halfWidthCentidegrees, 6000);
      expect(plan.contains(radians(0)), isTrue);
      expect(plan.contains(radians(89)), isTrue);
      expect(plan.contains(radians(-31)), isFalse);
      expect(plan.contains(radians(180)), isFalse);
    });

    test('an arc can run across straight behind', () {
      final plan = HuntPlan.between(radians(120), radians(-120));

      expect(plan.halfWidthCentidegrees, 6000);
      expect(plan.contains(radians(180)), isTrue);
      expect(plan.contains(radians(-150)), isTrue);
      expect(plan.contains(radians(0)), isFalse);
    });

    test('a bearing outside is brought to the nearest edge', () {
      final plan = HuntPlan.between(radians(-30), radians(90));

      expect(degrees(plan.clamp(radians(100))), closeTo(90, 1e-6));
      expect(degrees(plan.clamp(radians(-100))), closeTo(-30, 1e-6));
      // Straight behind is nearer the far edge, the long way round.
      expect(degrees(plan.clamp(radians(-170))), closeTo(90, 1e-6));
      expect(degrees(plan.clamp(radians(10))), closeTo(10, 1e-6));
    });

    test('the whole circle holds everything', () {
      expect(HuntPlan.open.isPlayable, isTrue);

      for (var turn = -180; turn <= 180; turn += 15) {
        expect(HuntPlan.open.contains(radians(turn.toDouble())), isTrue);
      }
    });

    test('a plan travels with the record of its hunt', () {
      const plan = HuntPlan(
        centreCentidegrees: -7250,
        halfWidthCentidegrees: 9100,
      );
      final log = HuntLog.decode(
        HuntLog(const [idle, idle], plan: plan).encode(),
      );

      expect(log.plan, plan);
      expect(log.inputs, hasLength(2));
    });

    test('the same hunt in a different plan is a different hunt', () {
      HuntOutcome? replayed(HuntPlan plan) {
        final sim = HuntSim(
          seed: 4,
          vessel: Vessel.llama,
          ward: Rarity.common.ward,
          stats: HuntStats.derive(HuntLoadout.borrowed),
        );
        final record = (HuntBot(sim)..play()).recorded;

        return HuntSim.replay(
          seed: 4,
          vessel: Vessel.llama,
          ward: Rarity.common.ward,
          stats: HuntStats.derive(HuntLoadout.borrowed),
          log: HuntLog(record, plan: plan),
        );
      }

      const elsewhere = HuntPlan(
        centreCentidegrees: 9000,
        halfWidthCentidegrees: 5000,
      );

      expect(replayed(HuntPlan.open)?.absorbed, isTrue);
      expect(
        replayed(elsewhere)?.absorbed ?? false,
        isFalse,
        reason: 'taps aimed at where the husk was in one room miss in another',
      );
    });
  });
}
