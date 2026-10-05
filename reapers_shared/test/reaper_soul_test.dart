import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

void main() {
  group('birth identity', () {
    test('R1 identity round-trips; rare decoration never changes power', () {
      for (var seed = 0; seed < 200; seed++) {
        final genome = ReaperGenome.shuffle(seed);
        final soul = ReaperSoul.fromGenome(genome);
        final restored = ReaperSoul.fromGenome(ReaperGenome.parse(genome.code));

        expect(restored.gift, soul.gift);
        expect(restored.temperament, soul.temperament);
        expect(soul.gift.reaperClass, genome.reaperClass);

        for (final omen in Omen.values) {
          final decorated = ReaperSoul.fromGenome(
            genome.copyWith(omen: omen, palette: 2, aura: Aura.embers),
          );
          expect(decorated.gift, soul.gift);
          expect(decorated.temperament, soul.temperament);
        }
      }
    });

    test('all 72 birth builds occur; each class has one gift per path', () {
      final builds = <(ReaperGift, Temperament)>{};
      for (var seed = 0; seed < 4000; seed++) {
        final soul = ReaperSoul.fromGenome(ReaperGenome.shuffle(seed));
        builds.add((soul.gift, soul.temperament));
      }
      expect(builds.length, 72);

      for (final reaperClass in ReaperClass.values) {
        final gifts = ReaperGift.values.where(
          (gift) => gift.reaperClass == reaperClass,
        );
        expect(gifts.map((gift) => gift.path).toSet(), SoulPath.values.toSet());
      }
    });

    test('each disposition benefits one affordance and pays a real cost', () {
      final baseline = HuntStats.derive(HuntLoadout.borrowed);
      HuntStats withTemperament(Temperament temperament) => HuntStats.derive(
        HuntLoadout(
          reaperClass: HuntLoadout.borrowed.reaperClass,
          level: HuntLoadout.borrowed.level,
          weapon: HuntLoadout.borrowed.weapon,
          soul: ReaperSoul(gift: ReaperGift.furrow, temperament: temperament),
        ),
      );
      final patient = withTemperament(Temperament.patient);
      expect(patient.captureTicks, greaterThan(baseline.captureTicks));
      expect(patient.swingTicks, greaterThan(baseline.swingTicks));
      final keen = withTemperament(Temperament.keen);
      expect(keen.reachRadians, greaterThan(baseline.reachRadians));
      expect(keen.captureTicks, lessThan(baseline.captureTicks));
      final stalwart = withTemperament(Temperament.stalwart);
      expect(stalwart.resolve, greaterThan(baseline.resolve));
      expect(stalwart.strikeDamage, lessThan(baseline.strikeDamage));
      final hungry = withTemperament(Temperament.hungry);
      expect(hungry.strikeDamage, greaterThan(baseline.strikeDamage));
      expect(hungry.flinchSpeed, lessThan(baseline.flinchSpeed));
    });
  });

  group('earned memory', () {
    const rich = HuntOutcome.absorbed(
      grade: ReapGrade.perfect,
      ambushed: true,
      ticks: 2000,
      deeds: HuntDeeds(holds: 12, parries: 8, maxCadence: 40),
    );

    test('a successful hunt awards at most one mark per path', () {
      final memory = const SoulMemory().remember(rich);
      expect(memory.toJson(), {'unseen': 1, 'stillness': 1, 'rhythm': 1});
      expect(memory.remember(rich).rhythm, 2);
      final restored = HuntOutcome.fromJson(rich.toJson());
      expect(restored.deeds.toJson(), rich.deeds.toJson());
    });

    test('escapes and incomplete cadence cannot farm growth', () {
      const escaped = HuntOutcome.escaped(
        escape: EscapeCause.time,
        ambushed: true,
        ticks: 5000,
        deeds: HuntDeeds(holds: 20, maxCadence: 50),
      );
      const short = HuntOutcome.absorbed(
        grade: ReapGrade.clean,
        ambushed: false,
        ticks: 800,
        deeds: HuntDeeds(maxCadence: 4),
      );
      expect(
        const SoulMemory().remember(escaped).toJson(),
        const SoulMemory().toJson(),
      );
      expect(const SoulMemory().remember(short).rhythm, 0);
    });

    test('level alone cannot awaken a gift; other paths cannot substitute', () {
      int rank(int level, int marks) => SoulRules.rank(
        level: level,
        memory: SoulMemory(stillness: marks),
        gift: ReaperGift.anchor,
      );
      expect(rank(30, 0), 0);
      expect(rank(4, 30), 0);
      expect(rank(5, 2), 0);
      expect(rank(5, 3), 1);
      expect(rank(11, 30), 1);
      expect(rank(12, 12), 2);
      expect(rank(19, 30), 2);
      expect(rank(20, 30), 3);
      expect(
        SoulRules.rank(
          level: 30,
          memory: const SoulMemory(unseen: 100, rhythm: 100),
          gift: ReaperGift.anchor,
        ),
        0,
      );
    });

    test('legacy saves keep growth without inventing remembered deeds', () {
      final veteran = ReaperState.bound(
        id: ReaperState.oathId,
        genome: ReaperGenome.shuffle(13),
      ).copyWith(xp: Progression.xpForLevel(12), huntsLost: 3);
      final json = veteran.toJson()..remove('memory');
      final restored = ReaperState.fromJson(json);
      expect(restored.level, 12);
      expect(restored.genome, veteran.genome);
      expect(restored.huntsLost, 3);
      expect(restored.soulRank, 0);
    });

    test('invalid stored counters are refused rather than clamped', () {
      expect(
        () => SoulMemory.fromJson({'unseen': -1, 'stillness': 0, 'rhythm': 0}),
        throwsFormatException,
      );
      expect(
        () => HuntDeeds.fromJson({'holds': -1, 'parries': 0, 'maxCadence': 0}),
        throwsFormatException,
      );
    });
  });

  group('power replay', () {
    for (final gift in ReaperGift.values) {
      for (final rank in [0, SoulRules.maxRank]) {
        test(
          '${gift.name} at awakening $rank replays every result and deed',
          () {
            final stats = HuntStats.derive(
              HuntLoadout(
                reaperClass: gift.reaperClass,
                level: 4,
                weapon: Weapon(gift.reaperClass.weapon, Temper.shrouded),
                soul: ReaperSoul(gift: gift, temperament: Temperament.keen),
                soulRank: rank,
              ),
            );
            final sim = HuntSim(
              seed: 19,
              vessel: Vessel.penguin,
              ward: 400,
              stats: stats,
            );
            final bot = HuntBot(sim, missEvery: 7);
            final outcome = bot.play();
            final log = HuntLog.decode(HuntLog(bot.recorded).encode());
            final replay = HuntSim.replay(
              seed: 19,
              vessel: Vessel.penguin,
              ward: 400,
              stats: stats,
              log: log,
            );
            expect(outcome, isNotNull);
            expect(replay?.toJson(), outcome!.toJson());
            expect(
              outcome.deeds.holds,
              bot.events.whereType<HeldFirm>().length,
            );
            // Passage's automatic protection must not count as a player parry.
            final protections = bot.events
                .whereType<GiftStirred>()
                .where((event) => event.gift == ReaperGift.passage)
                .length;
            expect(
              outcome.deeds.parries,
              bot.events.whereType<Parried>().length -
                  (protections > 1 ? 1 : 0),
            );
          },
        );
      }
    }

    test('previous combat logs cannot be interpreted under new powers', () {
      final bytes = HuntLog(const []).encode()..[0] = 4;
      expect(() => HuntLog.decode(bytes), throwsFormatException);
    });
  });
}
