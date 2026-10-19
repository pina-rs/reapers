import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/rooms.dart';

HuntSim _sim({
  required int seed,
  Vessel vessel = Vessel.ape,
  HuntPlan? plan,
  Rarity rarity = Rarity.epic,
}) => HuntSim(
  seed: seed,
  vessel: vessel,
  ward: rarity.ward,
  stats: HuntStats.derive(HuntLoadout.borrowed),
  plan: plan ?? kitchen(),
);

/// Plays [sim] with a careful bot, calling [look] after every tick.
void _play(HuntSim sim, void Function(List<HuntEvent> events) look) {
  final bot = HuntBot.human(sim);

  while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
    look(bot.stepOnce());
  }
}

double _range(Vec3 at) => math.sqrt(at.x * at.x + at.z * at.z);

void main() {
  group('what the plan knows of cover', () {
    test('something is hidden only when the counter is in front of it', () {
      final plan = kitchen();
      const eye = 1.45;

      // Behind the counter: low down hidden, high up seen over the top.
      expect(plan.hides(const Vec3(0, -eye + 0.4, 3.2)), isTrue);
      expect(plan.hides(const Vec3(0, -eye + 1.3, 3.2)), isFalse);
      // In front of it, nothing is hidden however low.
      expect(plan.hides(const Vec3(0, -eye + 0.05, 2.0)), isFalse);
      // Where nothing stands, nothing is hidden.
      expect(plan.hides(const Vec3(3.2, -eye + 0.05, 0)), isFalse);
      // A plan that knows no cover hides nothing.
      expect(HuntPlan.open.hides(const Vec3(0, -eye, 3.2)), isFalse);
    });

    test('a plan with cover travels whole with the record of its hunt', () {
      final plan = kitchen();
      final log = HuntLog([
        const HuntInput(aim: Aim(120, -400), turnSpeed: 30, focus: true),
        const HuntInput(aim: Aim(-50, 10), turnSpeed: 2000),
      ], plan: plan);
      final decoded = HuntLog.decode(log.encode());

      expect(decoded.plan, plan);
      expect(decoded.plan.ledgeAt(36), closeTo(0.92, 1e-9));
      expect(decoded.inputs.first.focus, isTrue);
      expect(decoded.inputs.last.focus, isFalse);
      expect(decoded.inputs.last.turnSpeed, 2000);
    });

    test('cover no phone could have measured is refused', () {
      final good = kitchen();

      for (final (label, plan) in [
        (
          'cover without a reach to stand at',
          HuntPlan(
            centreCentidegrees: 0,
            halfWidthCentidegrees: 9000,
            coverCentimetres: good.coverCentimetres,
            hideDecimetres: good.hideDecimetres,
            ledges: good.ledges,
          ),
        ),
        (
          'a hiding place in front of its cover',
          HuntPlan(
            centreCentidegrees: 0,
            halfWidthCentidegrees: 9000,
            reachDecimetres: good.reachDecimetres,
            coverCentimetres: good.coverCentimetres,
            hideDecimetres: [
              for (final hide in good.hideDecimetres) hide == 0 ? 0 : 10,
            ],
            ledges: good.ledges,
          ),
        ),
        (
          'a hiding place behind nothing',
          HuntPlan(
            centreCentidegrees: 0,
            halfWidthCentidegrees: 9000,
            reachDecimetres: good.reachDecimetres,
            coverCentimetres: List.filled(HuntPlan.slices, 0),
            hideDecimetres: good.hideDecimetres,
            ledges: good.ledges,
          ),
        ),
      ]) {
        expect(plan.isWellFormed, isFalse, reason: label);
        expect(
          HuntSim.replay(
            seed: 1,
            vessel: Vessel.ape,
            ward: 280,
            stats: HuntStats.derive(HuntLoadout.borrowed),
            log: HuntLog(const [], plan: plan),
          ),
          isNull,
          reason: label,
        );
      }
    });
  });

  group('a husk in a furnished room', () {
    test('leaps up onto the counter and down behind it, like a cat', () {
      final footings = <Footing>{};
      var leaps = 0;

      for (var seed = 1; seed <= 12; seed++) {
        _play(_sim(seed: seed), (events) {
          leaps += events.whereType<Leapt>().length;
        });

        final sim = _sim(seed: seed);

        _play(sim, (_) {
          if (!sim.leaping) {
            footings.add(sim.footing);
          }
        });
      }

      expect(leaps, greaterThan(10));
      expect(footings, containsAll(Footing.values));
    });

    test('stands on the counter top, and behind it on the floor', () {
      const floor = -HuntRules.eyeHeight;

      for (var seed = 1; seed <= 8; seed++) {
        final sim = _sim(seed: seed);

        _play(sim, (_) {
          if (sim.leaping || sim.phase == HuntPhase.charge) {
            return;
          }

          final pose = sim.pose;
          final feet = pose.position.y - Vessel.ape.halfHeight;

          switch (sim.footing) {
            case Footing.ledge:
              expect(feet, closeTo(floor + 0.92, 1e-9));
              expect(pose.ground, closeTo(floor + 0.92, 1e-9));
              expect(_range(pose.position), greaterThan(2.2));
            case Footing.hidden:
              expect(feet, closeTo(floor, 1e-9));
              expect(_range(pose.position), closeTo(3.2, 1e-9));
            case Footing.open:
              expect(feet, closeTo(floor, 1e-9));
              expect(sim.plan.hides(pose.position), isFalse);
          }
        });
      }
    });

    test('moves smoothly through every leap: no step longer than a stride', () {
      for (var seed = 1; seed <= 8; seed++) {
        final sim = _sim(seed: seed);
        var before = sim.pose.position;

        _play(sim, (_) {
          final after = sim.pose.position;

          expect((after - before).length, lessThan(0.2), reason: 'seed $seed');
          before = after;
        });
      }
    });

    test('clears the counter when it leaps over it', () {
      final plan = kitchen();

      for (var seed = 1; seed <= 8; seed++) {
        final sim = _sim(seed: seed);

        _play(sim, (_) {
          final at = sim.pose.position;
          final away = _range(at);
          final passing =
              sim.leaping &&
              (away - 2.2).abs() < 0.05 &&
              plan.coverAt(at.yaw) > 0;

          if (passing) {
            final feet = at.y - Vessel.ape.halfHeight;

            expect(feet, greaterThan(-HuntRules.eyeHeight + 0.92));
          }
        });
      }
    });

    test('its seals behind the counter are out of sight and out of reach', () {
      var hiddenTicks = 0;

      for (var seed = 1; seed <= 12; seed++) {
        final sim = _sim(seed: seed);

        _play(sim, (_) {
          if (sim.footing != Footing.hidden || sim.leaping) {
            return;
          }

          hiddenTicks++;

          for (var seal = 0; seal < sim.vessel.seals.length; seal++) {
            if (sim.plan.hides(sim.pose.sealPosition(seal))) {
              expect(sim.sealOpen(seal), isFalse);
            }
          }
        });
      }

      expect(hiddenTicks, greaterThan(0));
    });

    test('a blow at the part behind the counter hits the counter', () {
      final sim = _sim(seed: 5);

      while (!(sim.footing == Footing.hidden && !sim.leaping) &&
          sim.tick < HuntLog.maxTicks &&
          !sim.isOver) {
        sim.step(
          HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 0),
        );
      }

      expect(sim.footing, Footing.hidden, reason: 'it should hide in time');

      // Low on the body, below the top of the counter.
      final pose = sim.pose;
      final low = Vec3(
        pose.position.x,
        pose.position.y - Vessel.ape.halfHeight * 0.8,
        pose.position.z,
      ).normalized;

      expect(sim.plan.hides(low * pose.distance), isTrue);

      final events = sim.step(
        HuntInput(
          aim: Aim.fromDirection(pose.direction),
          turnSpeed: 0,
          taps: [Aim.fromDirection(low)],
        ),
      );

      expect(events.whereType<StrikeGlanced>(), isEmpty);
      expect(events.whereType<StrikeLanded>(), isEmpty);
    });

    test('it comes out from behind the counter to shriek', () {
      for (var seed = 1; seed <= 12; seed++) {
        final sim = _sim(seed: seed);

        _play(sim, (_) {
          if (sim.phase == HuntPhase.shriek) {
            expect(sim.footing, isNot(Footing.hidden), reason: 'seed $seed');
          }
        });
      }
    });

    test('it charges only from open floor', () {
      for (var seed = 1; seed <= 12; seed++) {
        final sim = _sim(seed: seed);

        _play(sim, (events) {
          if (events.any((event) => event is ChargeBegan)) {
            expect(sim.footing, Footing.open);
            expect(sim.leaping, isFalse);
          }
        });
      }
    });

    test('a penguin cannot jump onto a counter, but hides behind it', () {
      final footings = <Footing>{};

      for (var seed = 1; seed <= 12; seed++) {
        final sim = _sim(seed: seed, vessel: Vessel.penguin);

        _play(sim, (_) => footings.add(sim.footing));
      }

      expect(footings, isNot(contains(Footing.ledge)));
      expect(footings, contains(Footing.hidden));
    });

    test('a bird comes down to perch on the counter, and never hides', () {
      final footings = <Footing>{};

      for (var seed = 1; seed <= 12; seed++) {
        final sim = _sim(seed: seed, vessel: Vessel.bird);

        _play(sim, (_) {
          footings.add(sim.footing);

          if (sim.footing == Footing.ledge && !sim.leaping) {
            expect(
              sim.pose.position.y - Vessel.bird.halfHeight,
              closeTo(-HuntRules.eyeHeight + 0.92, 1e-9),
            );
          }
        });
      }

      expect(footings, contains(Footing.ledge));
      expect(footings, isNot(contains(Footing.hidden)));
    });

    test('cover too low to hide half a husk is not hidden behind', () {
      for (var seed = 1; seed <= 8; seed++) {
        final sim = _sim(
          seed: seed,
          plan: kitchen(counterCentimetres: 40, ledge: false),
        );

        _play(sim, (_) => expect(sim.footing, Footing.open));
      }
    });

    test('a hunt in a furnished room replays to the same end', () {
      final sim = _sim(seed: 9, rarity: Rarity.common);
      final bot = HuntBot.human(sim)..play();
      final log = HuntLog.decode(
        HuntLog(bot.recorded, plan: sim.plan).encode(),
      );
      final replayed = HuntSim.replay(
        seed: 9,
        vessel: Vessel.ape,
        ward: Rarity.common.ward,
        stats: HuntStats.derive(HuntLoadout.borrowed),
        log: log,
      );

      expect(replayed?.toJson(), sim.outcome?.toJson());
    });

    test('a room with nothing in it plays as it did before cover', () {
      final bare = kitchen();
      final plain = HuntPlan(
        centreCentidegrees: bare.centreCentidegrees,
        halfWidthCentidegrees: bare.halfWidthCentidegrees,
        reachDecimetres: bare.reachDecimetres,
      );
      final empty = HuntPlan(
        centreCentidegrees: bare.centreCentidegrees,
        halfWidthCentidegrees: bare.halfWidthCentidegrees,
        reachDecimetres: bare.reachDecimetres,
        coverCentimetres: List.filled(HuntPlan.slices, 0),
        hideDecimetres: List.filled(HuntPlan.slices, 0),
        ledges: List.filled(HuntPlan.slices, false),
      );
      final one = _sim(seed: 4, plan: plain);
      final other = _sim(seed: 4, plan: empty);

      HuntBot.human(one).play();
      HuntBot.human(other).play();

      expect(other.outcome?.toJson(), one.outcome?.toJson());
      expect(other.tick, one.tick);
    });
  });
}
