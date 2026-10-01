// Prints how rifts go for circles of different sizes and levels, for tuning.
// Run with `dart tool/rift_pacing.dart`.
// ignore_for_file: avoid_print
import 'package:reapers_shared/reapers_shared.dart';

import '../test/support/rift_table.dart';

void main() {
  for (final level in [1, 6, 12, 20]) {
    print('--- level $level ---');

    for (final people in [1, 2, 4]) {
      var sealed = 0;
      var seconds = 0;
      var xp = 0;
      const seeds = 8;

      for (var seed = 1; seed <= seeds; seed++) {
        final table = RiftTable(
          seed: seed,
          reapers: [
            for (var person = 0; person < people; person++)
              entrant(
                reaperClass: ReaperClass
                    .values[(seed + person) % ReaperClass.values.length],
                level: level,
              ),
          ],
        )..play();

        if (table.room.phase == RiftPhase.sealed) {
          sealed++;
          seconds += table.room.tick ~/ HuntRules.tickRate;
          xp += table.room.rewardFor(table.seats.first).xp;
        }
      }

      final average = sealed == 0
          ? '--'
          : '${seconds ~/ sealed}s, ${xp ~/ sealed} xp';

      print(
        '$people reaper${people == 1 ? ' ' : 's'} + ${RiftRules.circle - people} '
        'shades: sealed $sealed/$seeds  $average',
      );
    }
  }

  // What a reaper who only watches does to the circle.
  var carried = 0;

  for (var seed = 1; seed <= 8; seed++) {
    final table = RiftTable(
      seed: seed,
      reapers: [entrant(level: 6)],
      play: (sim, _) => HuntBot(sim, reactionTicks: 1 << 30),
    )..play();

    if (table.room.phase == RiftPhase.sealed) {
      carried++;
    }
  }

  print('one idle reaper + 3 shades: sealed $carried/8');
}
