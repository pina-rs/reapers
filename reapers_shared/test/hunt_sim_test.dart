import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

/// Steps [sim] with [input] until [phase] is reached or [limit] ticks pass.
List<HuntEvent> runUntil(
  HuntSim sim,
  HuntPhase phase, {
  HuntInput Function(HuntSim sim)? input,
  int limit = 4000,
}) {
  final events = <HuntEvent>[];

  for (var ticks = 0; ticks < limit && sim.phase != phase; ticks++) {
    events.addAll(sim.step(input?.call(sim) ?? idle));
  }

  expect(sim.phase, phase, reason: 'never reached $phase');

  return events;
}

/// Looks straight at the husk without moving.
HuntInput steadyOnHusk(HuntSim sim) =>
    HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 0);

/// Looks at the husk and taps the first open seal.
HuntInput tapOpenSeal(HuntSim sim) {
  for (var index = 0; index < sim.vessel.seals.length; index++) {
    if (sim.sealOpen(index)) {
      return HuntInput.measured(
        aim: sim.pose.direction,
        turnRadiansPerSecond: 0,
        taps: [sim.pose.sealDirection(index)],
      );
    }
  }

  return steadyOnHusk(sim);
}

void main() {
  group('stalk', () {
    test('holding a steady aim on the husk springs an ambush', () {
      final sim = simFor();
      final events = runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      expect(events.whereType<StalkEnded>().single.ambush, isTrue);
      expect(sim.ambushed, isTrue);
      expect(sim.tick, HuntRules.ambushTicks);
      expect(
        [for (var seal = 0; seal < 4; seal++) sim.sealOpen(seal)],
        everyElement(isTrue),
        reason: 'an ambush exposes every seal',
      );
    });

    test('turning fast alarms the husk and loses the ambush', () {
      final sim = simFor();
      final events = runUntil(
        sim,
        HuntPhase.fight,
        input: (_) => const HuntInput(aim: Aim.ahead, turnSpeed: 6000),
      );

      expect(events.whereType<StalkEnded>().single.ambush, isFalse);
      expect(sim.awareness, 1);
    });

    test('turning slowly never alarms the husk', () {
      final sim = simFor();

      for (var tick = 0; tick < 600; tick++) {
        sim.step(const HuntInput(aim: Aim.ahead, turnSpeed: 1200));
      }

      expect(sim.phase, HuntPhase.stalk);
      expect(sim.awareness, 0);
    });

    test('a husk that is never found notices on its own', () {
      final sim = simFor();

      runUntil(sim, HuntPhase.fight);

      expect(sim.tick, HuntRules.stalkLimitTicks + 1);
      expect(sim.ambushed, isFalse);
    });

    test('striking the husk first starts the fight without an ambush', () {
      final sim = simFor();
      final events = sim.step(
        HuntInput.measured(
          aim: Vec3.forward,
          turnRadiansPerSecond: 0,
          taps: [sim.pose.direction],
        ),
      );

      expect(events.whereType<StalkEnded>().single.ambush, isFalse);
      expect(events.whereType<StrikeGlanced>(), hasLength(1));
    });
  });

  group('strike', () {
    late HuntSim sim;

    setUp(() {
      // A ferryman, so no strike in this group cleaves.
      sim = simFor(reaperClass: ReaperClass.ferryman);
      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);
    });

    test('a tap on an open seal removes ward and shuts the seal', () {
      final events = sim.step(tapOpenSeal(sim));
      final strike = events.whereType<StrikeLanded>().single;

      expect(sim.ward, closeTo(Rarity.common.ward - strike.damage, 1e-9));
      expect(sim.sealOpen(strike.seal), isFalse);
      expect(strike.empowered, isTrue, reason: 'the ambush strike');
    });

    test('a tap away from the husk whiffs and breaks the cadence', () {
      sim.step(tapOpenSeal(sim));

      expect(sim.cadence, 1);

      for (var tick = 0; tick < sim.stats.swingTicks; tick++) {
        sim.step(steadyOnHusk(sim));
      }

      final events = sim.step(
        HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: 0,
          taps: [-sim.pose.direction],
        ),
      );

      expect(events.whereType<StrikeWhiffed>(), hasLength(1));
      expect(sim.cadence, 0);
    });

    test('a tap during a swing waits for it instead of being lost', () {
      sim.step(tapOpenSeal(sim));

      // Tap again a few ticks before the weapon is ready.
      for (var tick = 1; tick < sim.stats.swingTicks - 5; tick++) {
        sim.step(steadyOnHusk(sim));
      }

      final early = sim.step(tapOpenSeal(sim));

      expect(early.whereType<StrikeLanded>(), isEmpty);

      final later = <HuntEvent>[];

      for (var tick = 0; tick < 6; tick++) {
        later.addAll(sim.step(steadyOnHusk(sim)));
      }

      expect(later.whereType<StrikeLanded>(), hasLength(1));
    });

    test('a tap made too early in a swing is dropped', () {
      sim.step(tapOpenSeal(sim));
      sim.step(tapOpenSeal(sim));

      final later = <HuntEvent>[];

      for (var tick = 0; tick < sim.stats.swingTicks + 5; tick++) {
        later.addAll(sim.step(steadyOnHusk(sim)));
      }

      expect(later.whereType<StrikeLanded>(), isEmpty);
    });

    test('hits in cadence grow, up to the cap', () {
      final damages = <double>[];

      for (var tick = 0; tick < 2000 && damages.length < 8; tick++) {
        final events = sim.step(
          tick % sim.stats.swingTicks == 0
              ? tapOpenSeal(sim)
              : steadyOnHusk(sim),
        );

        if (sim.phase != HuntPhase.fight) {
          break;
        }

        damages.addAll([
          for (final strike in events.whereType<StrikeLanded>())
            if (!strike.empowered && strike.cleaved.isEmpty) strike.damage,
        ]);
      }

      expect(damages.length, greaterThan(3));

      final cap =
          sim.stats.strikeDamage *
          (1 + HuntRules.cadenceCap * HuntRules.cadenceStep);

      expect(damages, everyElement(lessThanOrEqualTo(cap + 1e-9)));
      expect(damages.last, greaterThan(damages.first));
    });
  });

  group('shriek', () {
    HuntSim atShriek({ReaperClass reaperClass = ReaperClass.harvester}) {
      final sim = simFor(reaperClass: reaperClass);

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);
      runUntil(sim, HuntPhase.shriek, input: steadyOnHusk);

      return sim;
    }

    test('the warning comes first and lasts its full length', () {
      final sim = simFor();

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      final start = sim.tick;
      final events = runUntil(sim, HuntPhase.windup, input: steadyOnHusk);

      expect(events.whereType<ShriekWindup>(), hasLength(1));
      expect(
        sim.tick - start,
        greaterThanOrEqualTo(HuntRules.shriekGapMinTicks),
      );

      final warned = sim.tick;

      runUntil(sim, HuntPhase.shriek, input: steadyOnHusk);

      expect(sim.tick - warned, HuntRules.windupTicks);
    });

    test('holding still through it exposes every seal', () {
      final sim = atShriek();
      final events = runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      expect(events.whereType<HeldFirm>(), hasLength(1));
      expect(events.whereType<Flinched>(), isEmpty);
      expect(sim.resolve, sim.stats.resolve);
      expect([
        for (var seal = 0; seal < 4; seal++) sim.sealOpen(seal),
      ], everyElement(isTrue));
    });

    test('moving during it costs one resolve, once', () {
      final sim = atShriek();
      final events = runUntil(
        sim,
        HuntPhase.fight,
        input: (sim) => HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: 3,
        ),
      );

      expect(events.whereType<Flinched>(), hasLength(1));
      expect(events.whereType<HeldFirm>(), isEmpty);
      expect(sim.resolve, sim.stats.resolve - 1);
    });

    test('tapping during it is a flinch too', () {
      final sim = atShriek();

      for (var tick = 0; tick < HuntRules.shriekGraceTicks; tick++) {
        sim.step(steadyOnHusk(sim));
      }

      final events = sim.step(
        HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: 0,
          taps: [sim.pose.direction],
        ),
      );

      expect(events.whereType<Flinched>(), hasLength(1));
      expect(events.whereType<StrikeLanded>(), isEmpty);
    });

    test('movement in the first moments is forgiven', () {
      final sim = atShriek();
      final events = <HuntEvent>[];

      for (var tick = 0; tick < HuntRules.shriekGraceTicks - 1; tick++) {
        events.addAll(
          sim.step(
            HuntInput.measured(
              aim: sim.pose.direction,
              turnRadiansPerSecond: 3,
            ),
          ),
        );
      }

      events.addAll(runUntil(sim, HuntPhase.fight, input: steadyOnHusk));

      expect(events.whereType<Flinched>(), isEmpty);
      expect(events.whereType<HeldFirm>(), hasLength(1));
    });

    test('running out of resolve lets the husk escape', () {
      final sim = simFor();

      runUntil(
        sim,
        HuntPhase.escaped,
        input: (sim) => HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: sim.phase == HuntPhase.shriek ? 3 : 0,
        ),
        limit: HuntLog.maxTicks,
      );

      expect(sim.outcome!.escape, EscapeCause.resolve);
      expect(sim.resolve, 0);
    });

    test('a ferryman is forgiven the first flinch of a hunt', () {
      final sim = atShriek(reaperClass: ReaperClass.ferryman);
      HuntInput moving(HuntSim sim) =>
          HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 3);

      final first = runUntil(sim, HuntPhase.fight, input: moving);

      expect(first.whereType<Flinched>().single.forgiven, isTrue);
      expect(sim.resolve, sim.stats.resolve);

      runUntil(sim, HuntPhase.shriek, input: steadyOnHusk);

      final second = runUntil(sim, HuntPhase.fight, input: moving);

      expect(second.whereType<Flinched>().single.forgiven, isFalse);
      expect(sim.resolve, sim.stats.resolve - 1);
    });
  });

  group('break and reap', () {
    test('a careful player absorbs a common husk inside the time limit', () {
      final bot = HuntBot(simFor());
      final outcome = bot.play();

      expect(outcome, isNotNull);
      expect(outcome!.absorbed, isTrue);
      expect(outcome.grade, isNot(ReapGrade.ragged));
      expect(bot.events.whereType<Broken>(), hasLength(1));
    });

    test('a steady sweep grades better than a hurried one', () {
      final steady = HuntBot(simFor(), reapSpeed: 0.8)..play();
      final hurried = HuntBot(simFor(), reapSpeed: 6)..play();

      expect(steady.sim.outcome!.grade, ReapGrade.perfect);
      expect(hurried.sim.outcome!.grade, ReapGrade.ragged);
    });

    test('a window left unfinished lets the husk recover, then escape', () {
      final sim = simFor();
      final bot = HuntBot(sim);

      while (sim.phase != HuntPhase.broken) {
        bot.stepOnce();
      }

      final recovered = runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      expect(recovered.whereType<ReapRecovered>().single.attemptsLeft, 1);
      expect(sim.ward, Rarity.common.ward * HuntRules.recoverShare);

      while (sim.phase != HuntPhase.broken) {
        bot.stepOnce();
      }

      runUntil(sim, HuntPhase.escaped, input: steadyOnHusk);

      expect(sim.outcome!.escape, EscapeCause.reap);
    });

    test('the cut only moves forward', () {
      final sim = simFor();
      final bot = HuntBot(sim);

      while (!sim.reapBegun || sim.reapProgress < 0.4) {
        bot.stepOnce();
      }

      final reached = sim.reapProgress;

      sim.step(
        HuntInput.measured(
          aim: sim.reapArcPoint(0.1),
          turnRadiansPerSecond: 0.5,
        ),
      );

      expect(sim.reapProgress, reached);
    });

    test('a fight that runs too long ends with the husk escaping', () {
      final sim = simFor(rarity: Rarity.epic);

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);
      runUntil(
        sim,
        HuntPhase.escaped,
        input: steadyOnHusk,
        limit: HuntLog.maxTicks,
      );

      expect(sim.outcome!.escape, EscapeCause.time);
    });
  });

  group('class perks', () {
    test('a harvester cleaves through neighbouring seals', () {
      final sim = simFor(vessel: Vessel.lanternShade);

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      final cleaves = <StrikeLanded>[];
      final bot = HuntBot(sim);

      while (!sim.isOver && sim.phase != HuntPhase.broken) {
        cleaves.addAll(bot.stepOnce().whereType<StrikeLanded>());
      }

      expect(cleaves.where((strike) => strike.cleaved.isNotEmpty), isNotEmpty);
    });

    test('a weigher striking in order hits twice as hard', () {
      final sim = simFor(reaperClass: ReaperClass.weigher);

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      // The ambush has exposed all four seals; the one that opened first
      // ranks 0.
      final oldest = [
        for (var seal = 0; seal < 4; seal++)
          if (sim.sealRank(seal) == 0) seal,
      ].single;
      final newest = [
        for (var seal = 0; seal < 4; seal++)
          if (sim.sealRank(seal) == 3) seal,
      ].single;

      HuntInput tap(int seal) => HuntInput.measured(
        aim: sim.pose.direction,
        turnRadiansPerSecond: 0,
        taps: [sim.pose.sealDirection(seal)],
      );

      final outOfOrder = sim.step(tap(newest)).whereType<StrikeLanded>().single;

      for (var tick = 0; tick < sim.stats.swingTicks; tick++) {
        sim.step(steadyOnHusk(sim));
      }

      final inOrder = sim.step(tap(oldest)).whereType<StrikeLanded>().single;

      // Take the ambush bonus off the first and the cadence off the second.
      final plain = outOfOrder.damage / HuntRules.ambushStrike;
      final measured = inOrder.damage / (1 + HuntRules.cadenceStep);

      expect(measured, closeTo(plain * HuntRules.measureStrike, 1e-9));
    });

    test('a mourner gains reap time from each unflinching hold', () {
      final sim = simFor(reaperClass: ReaperClass.mourner);
      final before = sim.captureTicks;

      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);
      runUntil(sim, HuntPhase.shriek, input: steadyOnHusk);
      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      expect(sim.captureTicks, before + HuntRules.tollTicks);
    });

    test('a chooser strikes three times as hard after holding firm', () {
      final sim = simFor(reaperClass: ReaperClass.chooser);

      runUntil(sim, HuntPhase.fight);
      runUntil(sim, HuntPhase.shriek, input: steadyOnHusk);
      runUntil(sim, HuntPhase.fight, input: steadyOnHusk);

      final strike = sim
          .step(tapOpenSeal(sim))
          .whereType<StrikeLanded>()
          .single;

      expect(
        strike.damage,
        closeTo(sim.stats.strikeDamage * HuntRules.chosenBlowStrike, 1e-9),
      );
    });

    test('a hollow builds cadence past the usual cap', () {
      final bot = HuntBot(simFor(reaperClass: ReaperClass.hollow))..play();
      final best = bot.events
          .whereType<StrikeLanded>()
          .map((strike) => strike.cadence)
          .reduce((a, b) => a > b ? a : b);

      expect(best, greaterThan(HuntRules.cadenceCap));
    });
  });

  group('every vessel', () {
    for (final vessel in Vessel.values.where((vessel) => vessel.isSolo)) {
      test('${vessel.name} can be absorbed by a careful player', () {
        final outcomes = [
          for (var seed = 1; seed <= 12; seed++)
            (HuntBot(simFor(seed: seed, vessel: vessel))..play()).sim.outcome,
        ];

        expect(
          outcomes.where((outcome) => outcome?.absorbed ?? false).length,
          greaterThanOrEqualTo(10),
          reason: 'a level-one reaper should usually take a common husk',
        );
      });
    }

    test('seals sit inside the body they belong to', () {
      for (final vessel in Vessel.values) {
        for (final seal in vessel.seals) {
          expect(
            seal.offset.x.abs(),
            lessThanOrEqualTo(vessel.halfWidth * 1.1),
          );
          expect(
            seal.offset.y.abs(),
            lessThanOrEqualTo(vessel.halfHeight * 1.1),
          );
          expect(seal.orbitRadius, lessThanOrEqualTo(vessel.halfWidth));
        }
      }
    });
  });

  group('pacing', () {
    /// Seconds of fighting, from the end of the stalk to the reap.
    int fightSeconds(HuntBot bot) {
      final stalk = bot.events.indexWhere((event) => event is StalkEnded);

      expect(stalk, isNot(-1));

      return (bot.sim.outcome!.ticks - HuntRules.ambushTicks) ~/
          HuntRules.tickRate;
    }

    List<HuntOutcome> humanHunts({
      required Rarity rarity,
      int level = 1,
      Temper temper = Temper.shrouded,
    }) => [
      for (final reaperClass in ReaperClass.values)
        for (var seed = 1; seed <= 4; seed++)
          (HuntBot.human(
            simFor(
              seed: seed,
              rarity: rarity,
              reaperClass: reaperClass,
              level: level,
              temper: temper,
            ),
          )..play()).sim.outcome!,
    ];

    int absorbed(List<HuntOutcome> outcomes) =>
        outcomes.where((outcome) => outcome.absorbed).length;

    test('a common husk takes a level-one reaper about half a minute', () {
      for (final reaperClass in ReaperClass.values) {
        final bot = HuntBot.human(simFor(reaperClass: reaperClass))..play();

        expect(bot.sim.outcome!.absorbed, isTrue, reason: reaperClass.name);
        expect(
          fightSeconds(bot),
          inInclusiveRange(15, 60),
          reason: reaperClass.name,
        );
      }
    });

    test('every class can take an uncommon husk at level one', () {
      final outcomes = humanHunts(rarity: Rarity.uncommon);

      expect(
        absorbed(outcomes),
        greaterThanOrEqualTo(outcomes.length * 3 ~/ 4),
      );
    });

    test('an epic husk is too much for any level-one reaper', () {
      expect(absorbed(humanHunts(rarity: Rarity.epic)), 0);
    });

    test('the same epic husk falls to a levelled, better-armed reaper', () {
      final outcomes = humanHunts(
        rarity: Rarity.epic,
        level: 10,
        temper: Temper.scorched,
      );

      expect(absorbed(outcomes), outcomes.length);
    });

    test('no class is far stronger than another', () {
      final seconds = {
        for (final reaperClass in ReaperClass.values)
          reaperClass: [
            for (var seed = 1; seed <= 6; seed++)
              fightSeconds(
                HuntBot.human(
                  simFor(seed: seed, reaperClass: reaperClass, level: 3),
                )..play(),
              ),
          ].reduce((a, b) => a + b),
      };
      final fastest = seconds.values.reduce((a, b) => a < b ? a : b);
      final slowest = seconds.values.reduce((a, b) => a > b ? a : b);

      expect(slowest / fastest, lessThan(2.2), reason: '$seconds');
    });
  });

  group('determinism and replay', () {
    test('the same seed and inputs give the same events', () {
      final first = HuntBot(simFor(seed: 31))..play();
      final second = HuntSim(
        seed: 31,
        vessel: first.sim.vessel,
        ward: Rarity.common.ward,
        stats: first.sim.stats,
      );
      final replayed = <HuntEvent>[];

      for (final input in first.recorded) {
        replayed.addAll(second.step(input));
      }

      expect(
        [for (final event in replayed) event.runtimeType],
        [for (final event in first.events) event.runtimeType],
      );
      expect(second.outcome!.grade, first.sim.outcome!.grade);
      expect(second.outcome!.ticks, first.sim.outcome!.ticks);
    });

    test('a log survives being packed into bytes', () {
      final bot = HuntBot(simFor(seed: 12))..play();
      final decoded = HuntLog.decode(HuntLog(bot.recorded).encode());

      expect(decoded.inputs.length, bot.recorded.length);

      for (var tick = 0; tick < decoded.inputs.length; tick++) {
        expect(decoded.inputs[tick].aim, bot.recorded[tick].aim);
        expect(decoded.inputs[tick].turnSpeed, bot.recorded[tick].turnSpeed);
        expect(decoded.inputs[tick].taps, bot.recorded[tick].taps);
      }
    });

    test('a packed log replays to the outcome the phone saw', () {
      for (var seed = 1; seed <= 20; seed++) {
        final bot = HuntBot(simFor(seed: seed))..play();
        final replayed = HuntSim.replay(
          seed: seed,
          vessel: Vessel.frameWraith,
          ward: Rarity.common.ward,
          stats: simFor().stats,
          log: HuntLog.decode(HuntLog(bot.recorded).encode()),
        );

        expect(replayed!.absorbed, bot.sim.outcome!.absorbed, reason: '$seed');
        expect(replayed.grade, bot.sim.outcome!.grade, reason: '$seed');
        expect(replayed.ticks, bot.sim.outcome!.ticks, reason: '$seed');
      }
    });

    test('a log that stops early proves nothing', () {
      final bot = HuntBot(simFor())..play();
      final cut = HuntLog(bot.recorded.sublist(0, bot.recorded.length ~/ 2));

      expect(
        HuntSim.replay(
          seed: 7,
          vessel: Vessel.frameWraith,
          ward: Rarity.common.ward,
          stats: simFor().stats,
          log: cut,
        ),
        isNull,
      );
    });

    test('a malformed log is rejected', () {
      final bytes = HuntLog(HuntBot(simFor()..step(idle)).recorded).encode();

      expect(() => HuntLog.decode(bytes.sublist(0, 0)), throwsFormatException);
      expect(
        () => HuntLog.decode(HuntLog([idle]).encode().sublist(0, 5)),
        throwsFormatException,
      );
    });
  });
}
