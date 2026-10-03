import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/sigil_tracer.dart';

Sigil _sigil({
  int seed = 7,
  Rarity rarity = Rarity.uncommon,
  Gait gait = Gait.ground,
  Difficulty difficulty = Difficulty.normal,
}) => Sigil.draw(
  seed: seed,
  shape: rarity.sigil,
  gait: gait,
  difficulty: difficulty,
);

/// A view of [trace]'s next light, dead on.
Vec3 _onNext(SigilTrace trace) => trace.directionOf(trace.caught);

/// A view well away from every light: straight up.
const _away = Vec3.up;

void main() {
  group('drawing a sigil', () {
    test('the same hunt always has the same sigil', () {
      final one = _sigil(seed: 41);
      final other = _sigil(seed: 41);

      expect(
        [for (final light in one.lights) (light.yaw, light.pitch)],
        [for (final light in other.lights) (light.yaw, light.pitch)],
      );
    });

    test('different hunts have different sigils', () {
      final shapes = {
        for (var seed = 0; seed < 40; seed++)
          [
            for (final light in _sigil(seed: seed).lights)
              '${degrees(light.yaw).round()}/${degrees(light.pitch).round()}',
          ].join(' '),
      };

      expect(shapes.length, greaterThan(15));
    });

    for (final rarity in Rarity.values) {
      for (final gait in [Gait.ground, Gait.air]) {
        test('a ${rarity.name} ${gait.name} sigil is laid out as its shape '
            'says, in every hunt', () {
          final shape = rarity.sigil;
          final (low, high) = gait == Gait.ground
              ? HuntRules.sigilGroundBand
              : HuntRules.sigilAirBand;
          var crossingAsAsked = 0;

          for (var seed = 0; seed < 300; seed++) {
            final sigil = _sigil(seed: seed, rarity: rarity, gait: gait);
            final yaws = [for (final light in sigil.lights) light.yaw]..sort();

            expect(sigil.lights, hasLength(shape.lights));
            expect(yaws.first, closeTo(-radians(shape.spanDegrees) / 2, 1e-9));
            expect(yaws.last, closeTo(radians(shape.spanDegrees) / 2, 1e-9));

            for (final light in sigil.lights) {
              expect(light.pitch, inInclusiveRange(low - 1e-9, high + 1e-9));
            }

            if (sigil.crossings == shape.crossings) {
              crossingAsAsked++;
            }
          }

          expect(
            crossingAsAsked,
            300,
            reason:
                'every sigil crosses itself as '
                'often as its shape asks',
          );
        });
      }
    }

    test('the trace never passes over a light it does not stop at', () {
      for (final rarity in Rarity.values) {
        for (var seed = 0; seed < 300; seed++) {
          final sigil = _sigil(seed: seed, rarity: rarity);
          final lights = sigil.lights;

          for (var step = 0; step + 1 < lights.length; step++) {
            final from = lights[step];
            final to = lights[step + 1];

            for (final light in lights) {
              if (identical(light, from) || identical(light, to)) {
                continue;
              }

              // Sampled along the shortest way from one light to the next,
              // which is the straight line the view draws between them.
              for (var share = 0.0; share <= 1; share += 0.02) {
                final along =
                    (from.direction * (1 - share) + to.direction * share)
                        .normalized;
                final apart = along.angleTo(light.direction);

                expect(
                  apart,
                  greaterThan(radians(rarity.sigil.radiusDegrees)),
                  reason:
                      '${rarity.name} sigil $seed: $from to $to passes '
                      'over $light',
                );
              }
            }
          }
        }
      }
    });

    test('a rarer husk has a harder sigil', () {
      for (final (easier, harder) in [
        (Rarity.common, Rarity.uncommon),
        (Rarity.uncommon, Rarity.rare),
        (Rarity.rare, Rarity.epic),
      ]) {
        final (one, other) = (easier.sigil, harder.sigil);

        expect(other.lights, greaterThan(one.lights));
        expect(other.spanDegrees, greaterThan(one.spanDegrees));
        expect(other.radiusDegrees, lessThan(one.radiusDegrees));
        expect(other.holdSeconds, greaterThan(one.holdSeconds));
        expect(other.crossings, greaterThan(one.crossings));
        expect(
          other.fraySeconds ?? double.infinity,
          lessThanOrEqualTo(one.fraySeconds ?? double.infinity),
        );
      }
    });

    test('every sigil reads an arc wide enough to hunt in', () {
      for (final rarity in Rarity.values) {
        final halfArc = _sigil(rarity: rarity).halfArc;

        expect(
          degrees(halfArc) * 100,
          greaterThanOrEqualTo(HuntPlan.minHalfWidthCentidegrees),
        );
      }
    });

    test('the testing difficulty changes how forgiving a sigil is, never its '
        'shape', () {
      final normal = _sigil(rarity: Rarity.rare);
      final easy = _sigil(rarity: Rarity.rare, difficulty: Difficulty.easy);
      final hard = _sigil(rarity: Rarity.rare, difficulty: Difficulty.hard);

      expect(easy.radius, greaterThan(normal.radius));
      expect(hard.radius, lessThan(normal.radius));
      expect(easy.fraySeconds, greaterThan(normal.fraySeconds!));

      for (final other in [easy, hard]) {
        expect(
          [for (final light in other.lights) light.yaw],
          [for (final light in normal.lights) light.yaw],
        );
      }
    });
  });

  group('tracing a sigil', () {
    test('a light catches after it is held still for as long as it asks', () {
      final trace = SigilTrace(_sigil(), centre: 0.4);
      final frames = (trace.sigil.holdSeconds * 60).ceil();
      final events = <SigilEvent>[];

      for (var frame = 0; frame < frames - 1; frame++) {
        events.addAll(
          trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60),
        );
      }

      expect(events, isEmpty);
      expect(trace.fill, greaterThan(0.9));

      events.addAll(
        trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60),
      );

      expect(events.single, isA<LightCaught>());
      expect((events.single as LightCaught).index, 0);
      expect(trace.caught, 1);
      expect(trace.fill, 0);
    });

    test('the lights catch in order, and only in order', () {
      final trace = SigilTrace(_sigil(), centre: 0);
      final later = trace.directionOf(2);

      for (var frame = 0; frame < 120; frame++) {
        trace.advance(later, turnSpeed: 0, deltaSeconds: 1 / 60);
      }

      expect(trace.caught, 0);
      expect(trace.fill, 0);
    });

    test('turning too fast holds a light where it is', () {
      final trace = SigilTrace(_sigil(), centre: 0);

      for (var frame = 0; frame < 15; frame++) {
        trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60);
      }

      final held = trace.fill;

      for (var frame = 0; frame < 120; frame++) {
        trace.advance(
          _onNext(trace),
          turnSpeed: HuntRules.sigilSteadySpeed + 0.1,
          deltaSeconds: 1 / 60,
        );
      }

      expect(trace.fill, held);
      expect(trace.caught, 0);
    });

    test('looking away drains a light twice as fast as it filled', () {
      final trace = SigilTrace(_sigil(), centre: 0);

      for (var frame = 0; frame < 20; frame++) {
        trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60);
      }

      final held = trace.fill;

      for (var frame = 0; frame < 5; frame++) {
        trace.advance(_away, turnSpeed: 0, deltaSeconds: 1 / 60);
      }

      expect(trace.fill, closeTo(held * (1 - 10 / 20), 1e-9));
    });

    test('the view rests on a light anywhere inside its radius', () {
      final trace = SigilTrace(_sigil(), centre: 0);
      final light = trace.sigil.lights.first;
      final radius = trace.sigil.radius;

      expect(
        trace.restsOn(Vec3.fromAngles(light.yaw, light.pitch + radius * 0.95)),
        isTrue,
      );
      expect(
        trace.restsOn(Vec3.fromAngles(light.yaw, light.pitch + radius * 1.05)),
        isFalse,
      );
    });

    test('the last light closes the sigil, once', () {
      final trace = SigilTrace(_sigil(rarity: Rarity.common), centre: 0);
      final events = <SigilEvent>[];

      while (!trace.isClosed) {
        events.addAll(
          trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60),
        );
      }

      expect(events.whereType<LightCaught>().map((event) => event.index), [
        0,
        1,
        2,
      ]);
      expect(events.last, isA<SigilClosed>());
      expect(trace.advance(_away, turnSpeed: 0, deltaSeconds: 1 / 60), isEmpty);
    });

    group('fraying', () {
      SigilTrace caughtOne() {
        final trace = SigilTrace(_sigil(rarity: Rarity.rare), centre: 0);

        while (trace.caught == 0) {
          trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60);
        }

        return trace;
      }

      test('a caught light goes out when the next is too long coming', () {
        final trace = caughtOne();
        final fray = trace.sigil.fraySeconds!;
        final events = <SigilEvent>[];
        var seconds = 0.0;

        while (events.isEmpty) {
          events.addAll(trace.advance(_away, turnSpeed: 0, deltaSeconds: 0.1));
          seconds += 0.1;
        }

        expect(seconds, closeTo(fray, 0.11));
        expect((events.single as LightLost).index, 0);
        expect(trace.caught, 0);
        expect(trace.fraying, 0);
      });

      test('it does not fray while the next is being held', () {
        final trace = caughtOne();

        for (var frame = 0; frame < 600; frame++) {
          // Resting on the next light, but turning too fast for it to fill.
          trace.advance(
            _onNext(trace),
            turnSpeed: HuntRules.sigilSteadySpeed + 0.1,
            deltaSeconds: 1 / 60,
          );
        }

        expect(trace.caught, 1);
        expect(trace.fraying, 0);
      });

      test('a common sigil never frays', () {
        final trace = SigilTrace(_sigil(rarity: Rarity.common), centre: 0);

        while (trace.caught == 0) {
          trace.advance(_onNext(trace), turnSpeed: 0, deltaSeconds: 1 / 60);
        }

        for (var frame = 0; frame < 6000; frame++) {
          trace.advance(_away, turnSpeed: 0, deltaSeconds: 1 / 60);
        }

        expect(trace.caught, 1);
        expect(trace.fraying, 0);
      });
    });

    test('someone turning at an easy pace traces every sigil, and a rarer '
        'one takes longer', () {
      final medians = <Rarity, double>{};

      for (final rarity in Rarity.values) {
        final times = <double>[];

        for (var seed = 0; seed < 100; seed++) {
          final result = traceSigil(
            SigilTrace(_sigil(seed: seed, rarity: rarity), centre: 0),
          );

          expect(result.closed, isTrue, reason: '${rarity.name} sigil $seed');
          expect(result.lost, 0, reason: '${rarity.name} sigil $seed');
          times.add(result.seconds);
        }

        times.sort();
        medians[rarity] = times[times.length ~/ 2];
      }

      expect(medians[Rarity.common], lessThan(5));
      expect(medians[Rarity.epic], lessThan(16));

      for (final (easier, harder) in [
        (Rarity.common, Rarity.uncommon),
        (Rarity.uncommon, Rarity.rare),
        (Rarity.rare, Rarity.epic),
      ]) {
        expect(medians[harder], greaterThan(medians[easier]!));
      }
    });

    test('a slow hand loses lights on an epic sigil at normal, and '
        'keeps them at easy', () {
      int lostAt(Difficulty difficulty) {
        var lost = 0;

        for (var seed = 0; seed < 60; seed++) {
          lost += traceSigil(
            SigilTrace(
              _sigil(seed: seed, rarity: Rarity.epic, difficulty: difficulty),
              centre: 0,
            ),
            turnSpeed: 0.3,
          ).lost;
        }

        return lost;
      }

      expect(lostAt(Difficulty.normal), greaterThan(0));
      expect(lostAt(Difficulty.easy), 0);
    });
  });
}
