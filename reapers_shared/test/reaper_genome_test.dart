import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

void main() {
  group('ReaperStats', () {
    test('every class totals the same, so no shuffle is simply better', () {
      for (final reaperClass in ReaperClass.values) {
        expect(
          reaperClass.stats.total,
          ReaperStats.classTotal,
          reason: reaperClass.name,
        );
      }
    });

    test('every class has epithets and a weapon profile', () {
      for (final reaperClass in ReaperClass.values) {
        expect(reaperEpithets[reaperClass], isNotEmpty);
        expect(weaponProfiles[reaperClass.weapon], isNotNull);
      }
    });
  });

  group('ReaperGenome.shuffle', () {
    test('the same seed always draws the same reaper', () {
      expect(ReaperGenome.shuffle(42).code, ReaperGenome.shuffle(42).code);
    });

    test('different seeds draw different reapers', () {
      final codes = {
        for (var seed = 0; seed < 500; seed++) ReaperGenome.shuffle(seed).code,
      };

      expect(codes.length, greaterThan(495));
    });

    test('every class and every omen turns up', () {
      final genomes = [
        for (var seed = 0; seed < 4000; seed++) ReaperGenome.shuffle(seed),
      ];

      expect({
        for (final genome in genomes) genome.reaperClass,
      }, ReaperClass.values.toSet());
      expect({for (final genome in genomes) genome.omen}, Omen.values.toSet());
    });

    test('an omen is rare', () {
      final marked = [
        for (var seed = 0; seed < 4000; seed++)
          if (ReaperGenome.shuffle(seed).omen != Omen.none) seed,
      ];

      expect(marked.length / 4000, closeTo(0.2, 0.04));
    });
  });

  group('ReaperGenome.code', () {
    test('round-trips exactly', () {
      for (var seed = 0; seed < 300; seed++) {
        final genome = ReaperGenome.shuffle(seed);
        final parsed = ReaperGenome.parse(genome.code);

        expect(parsed, genome);
        expect(parsed.name, genome.name);
        expect(parsed.stature, genome.stature);
      }
    });

    test('the borrowed reaper has a valid code', () {
      expect(
        ReaperGenome.parse(ReaperGenome.borrowed.code),
        ReaperGenome.borrowed,
      );
    });

    test('rejects a malformed code', () {
      final valid = ReaperGenome.shuffle(1).code;

      for (final broken in [
        '',
        'nonsense',
        valid.replaceFirst('R1', 'R9'),
        valid.replaceFirst(RegExp('-h[0-9]+'), '-h99'),
        valid.replaceFirst(RegExp('-s[0-9]+'), '-s5000'),
        valid.replaceFirst(RegExp('-(har|fer|wei|cho|hol|mou)-'), '-xyz-'),
        '$valid-extra',
      ]) {
        expect(
          () => ReaperGenome.parse(broken),
          throwsFormatException,
          reason: broken,
        );
      }
    });
  });

  group('SeededRandom', () {
    test('has a fixed sequence that must never change', () {
      final random = SeededRandom(1);

      // These values are what minted reapers and recorded hunts were drawn
      // from. Changing the generator changes every one of them.
      expect(
        [for (var draw = 0; draw < 4; draw++) random.nextUint32()],
        [2693262067, 11749833, 2265367787, 4213581821],
      );
    });

    test('stays inside its bounds', () {
      final random = SeededRandom(99);

      for (var draw = 0; draw < 2000; draw++) {
        expect(random.nextDouble(), inInclusiveRange(0, 0.9999999999));
        expect(random.nextInt(7), inInclusiveRange(0, 6));
      }
    });

    test('mixSeed separates nearby inputs', () {
      expect(mixSeed([1, 2]), isNot(mixSeed([2, 1])));
      expect(hashText('degenApes:1'), isNot(hashText('degenApes:2')));
    });
  });

  group('Progression', () {
    test('levels need more experience each time, up to the cap', () {
      for (var level = 1; level < Progression.levelCap - 1; level++) {
        expect(
          Progression.xpToNext(level + 1),
          greaterThan(Progression.xpToNext(level)),
        );
      }

      expect(Progression.levelForXp(0), 1);
      expect(Progression.levelForXp(Progression.xpToNext(1) - 1), 1);
      expect(Progression.levelForXp(Progression.xpToNext(1)), 2);
      expect(Progression.levelForXp(1 << 40), Progression.levelCap);
      expect(Progression.levelProgress(1 << 40), 1);
    });

    test('xpForLevel is the inverse of levelForXp', () {
      for (var level = 1; level <= Progression.levelCap; level++) {
        expect(Progression.levelForXp(Progression.xpForLevel(level)), level);
      }
    });
  });
}
