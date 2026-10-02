import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

final _stats = HuntStats.derive(
  const HuntLoadout(
    reaperClass: ReaperClass.harvester,
    level: 1,
    weapon: Weapon(WeaponKind.scythe, Temper.shrouded),
  ),
);

HuntSim _hunt(
  Difficulty difficulty, {
  int seed = 7,
  Vessel vessel = Vessel.ape,
}) => HuntSim(
  seed: seed,
  vessel: vessel,
  ward: Rarity.uncommon.ward,
  stats: _stats,
  difficulty: difficulty,
);

/// Someone who has never played: slow to see a seal open, and wide of the
/// mark with every other tap.
HuntBot _beginner(HuntSim sim) =>
    HuntBot(sim, turnSpeed: 0.9, reactionTicks: 40, missEvery: 2);

void main() {
  group('what a difficulty changes', () {
    test('normal is the game as written', () {
      final sim = _hunt(Difficulty.normal);

      expect(sim.wardMax, Rarity.uncommon.ward);
      expect(sim.stats.resolve, _stats.resolve);
      expect(sim.stats.reachRadians, _stats.reachRadians);
      expect(sim.stats.flinchSpeed, _stats.flinchSpeed);
      expect(sim.stats.captureTicks, _stats.captureTicks);
      expect(sim.reapAttemptsLeft, HuntRules.reapAttempts);
    });

    test('easy thins the ward and widens every window', () {
      final sim = _hunt(Difficulty.easy);

      expect(sim.wardMax, closeTo(Rarity.uncommon.ward * 0.35, 1e-9));
      expect(sim.stats.resolve, _stats.resolve + 3);
      expect(sim.stats.reachRadians, greaterThan(_stats.reachRadians * 1.5));
      expect(sim.stats.flinchSpeed, greaterThan(_stats.flinchSpeed * 2));
      expect(sim.stats.captureTicks, greaterThan(_stats.captureTicks * 2));
      expect(
        sim.stats.reapToleranceRadians,
        greaterThan(_stats.reapToleranceRadians * 1.5),
      );
      expect(sim.reapAttemptsLeft, HuntRules.reapAttempts + 2);
      // What a strike does is the reaper's own, whatever the difficulty.
      expect(sim.stats.strikeDamage, _stats.strikeDamage);
    });

    test('hard stiffens the ward and narrows the windows', () {
      final sim = _hunt(Difficulty.hard);

      expect(sim.wardMax, greaterThan(Rarity.uncommon.ward));
      expect(sim.stats.reachRadians, lessThan(_stats.reachRadians));
      expect(sim.stats.flinchSpeed, lessThan(_stats.flinchSpeed));
      expect(sim.stats.resolve, _stats.resolve);
    });

    test('a husk gets about slower on easy and faster on hard', () {
      double topSpeed(Difficulty difficulty) {
        final sim = _hunt(difficulty);
        final bot = HuntBot.human(sim);
        var top = 0.0;

        while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
          bot.stepOnce();
          top = sim.speed > top ? sim.speed : top;
        }

        return top;
      }

      expect(topSpeed(Difficulty.easy), lessThan(topSpeed(Difficulty.normal)));
      expect(
        topSpeed(Difficulty.hard),
        greaterThan(topSpeed(Difficulty.normal)),
      );
    });
  });

  group('who can bring a husk down', () {
    test('a beginner takes it on easy, on every seed tried', () {
      for (var seed = 1; seed <= 12; seed++) {
        for (final vessel in Vessel.values.where((vessel) => vessel.isSolo)) {
          final sim = _hunt(Difficulty.easy, seed: seed, vessel: vessel);

          expect(
            _beginner(sim).play()?.absorbed,
            isTrue,
            reason: '${vessel.name}, seed $seed',
          );
        }
      }
    });

    test('the same beginner mostly loses it on hard', () {
      var absorbed = 0;
      var hunts = 0;

      for (var seed = 1; seed <= 12; seed++) {
        for (final vessel in Vessel.values.where((vessel) => vessel.isSolo)) {
          hunts++;

          if (_beginner(_hunt(Difficulty.hard, seed: seed, vessel: vessel))
                  .play()
                  ?.absorbed ??
              false) {
            absorbed++;
          }
        }
      }

      expect(absorbed, lessThan(hunts ~/ 2));
    });

    test('a hunt runs longer on easy before the husk gives up waiting', () {
      // Nobody strikes, so the fight runs to its limit.
      int ticksToEscape(Difficulty difficulty) {
        final sim = _hunt(difficulty);

        while (!sim.isOver) {
          sim.step(waryOnHusk(sim));
        }

        expect(sim.outcome!.escape, EscapeCause.time);

        return sim.tick;
      }

      expect(
        ticksToEscape(Difficulty.easy),
        greaterThan(ticksToEscape(Difficulty.normal) + 60 * HuntRules.tickRate),
      );
    });
  });

  group('the record of a hunt', () {
    test('carries the difficulty it was played at', () {
      for (final difficulty in Difficulty.values) {
        final log = HuntLog.decode(
          HuntLog(const [idle], difficulty: difficulty).encode(),
        );

        expect(log.difficulty, difficulty);
        expect(log.inputs, hasLength(1));
      }
    });

    test('with a difficulty nobody has heard of is refused', () {
      final bytes = HuntLog(const [idle]).encode();

      bytes[7] = Difficulty.values.length;

      expect(() => HuntLog.decode(bytes), throwsFormatException);
    });

    test('replays to the same end at the difficulty it was played at', () {
      final sim = _hunt(Difficulty.easy, seed: 3);
      final bot = _beginner(sim)..play();
      final log = HuntLog(bot.recorded, difficulty: Difficulty.easy);

      HuntOutcome? replayed(HuntLog log) => HuntSim.replay(
        seed: 3,
        vessel: Vessel.ape,
        ward: Rarity.uncommon.ward,
        stats: _stats,
        log: log,
      );

      expect(sim.outcome!.absorbed, isTrue);
      expect(replayed(log)?.absorbed, isTrue);
      expect(replayed(log)?.ticks, sim.outcome!.ticks);
      // The same play against the husk at its full strength does not hold.
      expect(replayed(HuntLog(bot.recorded))?.absorbed ?? false, isFalse);
    });
  });

  group('a keeper that pays for real', () {
    final now = DateTime.utc(2026, 10, 2, 12);
    final later = now.add(const Duration(minutes: 1));
    const husk = Nft(collection: NftCollection.degenApes, serial: 1);

    PlayerState released() => PlayerState.initial()
        .takeOath(seed: 11)
        .acquire(husk)
        .release(husk, seed: 5, now: now);

    HuntLog played(PlayerState state, Difficulty difficulty) {
      final sighting = state.sightings.single;
      final sim = HuntSim(
        seed: sighting.seed,
        vessel: husk.collection.vessel,
        ward: husk.rarity.ward,
        stats: HuntStats.derive(state.loadout),
        difficulty: difficulty,
      );

      return HuntLog((HuntBot(sim)..play()).recorded, difficulty: difficulty);
    }

    test('takes a hunt played on easy while it is only being tested', () {
      final state = released();
      final settlement = state.judgeHunt(
        1,
        played(state, Difficulty.easy),
        now: later,
        feedingSeed: 1,
      );

      expect(settlement.outcome.absorbed, isTrue);
    });

    test('refuses one once only the game itself is allowed', () {
      final state = released();

      expect(
        () => state.judgeHunt(
          1,
          played(state, Difficulty.easy),
          now: later,
          feedingSeed: 1,
          difficulties: const {Difficulty.normal},
        ),
        throwsA(
          isA<RuleBreach>().having(
            (breach) => breach.refusal,
            'refusal',
            Refusal.huntUnproven,
          ),
        ),
      );
      expect(
        state
            .judgeHunt(
              1,
              played(state, Difficulty.normal),
              now: later,
              feedingSeed: 1,
              difficulties: const {Difficulty.normal},
            )
            .outcome
            .absorbed,
        isTrue,
      );
    });
  });
}
