// Prints how long a human-paced bot takes to break each rarity with each
// class, for tuning. Run with `dart tool/pacing.dart`.
// ignore_for_file: avoid_print
import 'package:reapers_shared/reapers_shared.dart';

import '../test/support/hunt_bot.dart';

void main() {
  for (final (level, temper) in [
    (1, Temper.shrouded),
    (5, Temper.sworn),
    (10, Temper.scorched),
    (16, Temper.hollowed),
  ]) {
    print('--- level $level, ${temper.name} ---');

    for (final reaperClass in ReaperClass.values) {
      final cells = <String>[];

      for (final rarity in Rarity.values) {
        var won = 0;
        var seconds = 0;
        const seeds = 8;

        for (var seed = 1; seed <= seeds; seed++) {
          final bot = HuntBot.human(
            simFor(
              seed: seed,
              rarity: rarity,
              reaperClass: reaperClass,
              level: level,
              temper: temper,
            ),
          )..play();

          if (bot.sim.outcome!.absorbed) {
            won++;
            seconds += bot.sim.outcome!.ticks ~/ HuntRules.tickRate;
          }
        }

        final average = won == 0 ? '--' : '${seconds ~/ won}s';

        cells.add('${rarity.name}: $won/$seeds $average'.padRight(22));
      }

      print('${reaperClass.name.padRight(10)} ${cells.join()}');
    }
  }
}
