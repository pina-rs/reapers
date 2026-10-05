import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

HuntSim _hunt(ReaperGift gift, {int rank = 0, int seed = 19}) => HuntSim(
  seed: seed,
  vessel: Vessel.penguin,
  ward: 1000,
  stats: HuntStats.derive(
    HuntLoadout(
      reaperClass: gift.reaperClass,
      level: 1,
      weapon: Weapon(gift.reaperClass.weapon, Temper.shrouded),
      soul: ReaperSoul(gift: gift, temperament: Temperament.keen),
      soulRank: rank,
    ),
  ),
);

HuntInput _still(HuntSim sim) =>
    HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 0);

List<HuntEvent> _until(HuntSim sim, bool Function() reached) {
  final events = <HuntEvent>[];
  while (!reached() && !sim.isOver && sim.tick < HuntLog.maxTicks) {
    events.addAll(sim.step(_still(sim)));
  }
  expect(reached(), isTrue, reason: 'encounter ended before the action');
  return events;
}

void main() {
  group('birth gifts change the fight', () {
    for (final gift in ReaperGift.values) {
      test('${gift.name} can be activated through player actions', () {
        final sim = _hunt(gift);
        final bot = HuntBot(sim, missEvery: 7);
        final events = <HuntEvent>[];
        var shrieks = 0;
        while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
          final decision = bot.decide();
          // Anchor needs lost resolve; Return Light needs a mistake after
          // its cadence promise. Other gifts face careful play and misses.
          final failCharge =
              gift == ReaperGift.anchor && sim.phase == HuntPhase.charge;
          final failShriek =
              gift == ReaperGift.returnLight &&
              sim.phase == HuntPhase.shriek &&
              shrieks > 0;
          final input = failCharge
              ? _still(sim)
              : failShriek
              ? HuntInput(aim: decision.aim, turnSpeed: 3000)
              : decision;
          final next = sim.step(input);
          if (next.any((event) => event is ShriekBegan)) shrieks++;
          events.addAll(next);
        }
        expect(
          events.whereType<GiftStirred>().where((event) => event.gift == gift),
          isNotEmpty,
          reason: 'a documented birth power must be reachable in combat',
        );
      });
    }

    test('Hunger starts cadence but never fabricates five landed hits', () {
      final sim = _hunt(ReaperGift.hunger, rank: 3);
      _until(sim, () => sim.phase == HuntPhase.fight);
      expect(sim.cadence, 4);
      expect(sim.deeds.maxCadence, 0);
      sim.step(_still(sim));
      expect(sim.cadence, 4);
    });

    test('Passage protects once, then an unmet charge costs resolve', () {
      final sim = _hunt(ReaperGift.passage);
      _until(sim, () => sim.phase == HuntPhase.charge);
      final resolve = sim.resolve;
      final protected = _until(sim, () => sim.phase != HuntPhase.charge);
      expect(protected.whereType<Parried>(), hasLength(1));
      expect(sim.resolve, resolve);
      expect(sim.deeds.parries, 0);
      _until(sim, () => sim.phase == HuntPhase.charge);
      final struck = _until(sim, () => sim.phase != HuntPhase.charge);
      expect(struck.whereType<Struck>(), hasLength(1));
      expect(sim.resolve, resolve - 1);
    });

    test('awakened Passage lengthens protection without adding uses', () {
      int stagger(int rank) {
        final sim = _hunt(ReaperGift.passage, rank: rank);
        _until(sim, () => sim.phase == HuntPhase.charge);
        _until(sim, () => sim.phase != HuntPhase.charge);
        final start = sim.tick;
        _until(sim, () => !sim.staggered);
        return sim.tick - start;
      }

      expect(stagger(3), greaterThan(stagger(0)));
    });

    test('Hush delays the first warning further as it awakens', () {
      int firstWarning(int rank) {
        final sim = _hunt(ReaperGift.hush, rank: rank);
        _until(sim, () => sim.phase == HuntPhase.windup);
        return sim.tick;
      }

      expect(firstWarning(3), greaterThan(firstWarning(0)));
    });

    test('Refrain adds more reap time while keeping its six-toll cap', () {
      int window(int rank) {
        final sim = _hunt(ReaperGift.refrain, rank: rank);
        final bot = HuntBot(sim);
        bot.play();
        final holds = bot.events.whereType<HeldFirm>().length;
        final tolls = bot.events
            .whereType<GiftStirred>()
            .where((event) => event.gift == ReaperGift.refrain)
            .length;
        expect(tolls, inInclusiveRange(1, SoulRules.refrainMaxTolls));
        return sim.captureTicks - holds * HuntRules.tollTicks;
      }

      expect(window(3), greaterThan(window(0)));
    });
  });

  group('untrusted combat logs', () {
    test('impossible aim and tap directions are refused', () {
      for (final input in [
        const HuntInput(aim: Aim(18001, 0), turnSpeed: 0),
        const HuntInput(aim: Aim(0, -9001), turnSpeed: 0),
        const HuntInput(aim: Aim.ahead, turnSpeed: 0, taps: [Aim(0, 9001)]),
      ]) {
        expect(
          () => HuntLog.decode(HuntLog([input]).encode()),
          throwsFormatException,
        );
      }
    });

    test('a bounded log is accepted; its first excess tick is refused', () {
      const idle = HuntInput(aim: Aim.ahead, turnSpeed: 0);
      final limit = HuntLog(List.filled(HuntLog.maxTicks, idle)).encode();
      expect(HuntLog.decode(limit).inputs, hasLength(HuntLog.maxTicks));
      final excess = Uint8List.fromList([...limit, ...List.filled(7, 0)]);
      expect(() => HuntLog.decode(excess), throwsFormatException);
    });
  });
}
