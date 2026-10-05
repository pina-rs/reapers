// Reproducible birth-build trials; keep stdout with the research record.
// Run in reapers_shared: fvm dart tool/soul_pacing.dart
// ignore_for_file: avoid_print
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  print('# Soul pacing trials');
  print('');
  print(
    'Seeds 1–8, rare penguin, shrouded weapon, normal difficulty. '
    'Human bot: 16-tick reaction and every seventh tap wide. '
    'Each row spans all four temperaments. Level 20 assumes 30 earned '
    'marks in the gift’s path; this is potential, not free progress.',
  );
  print('');
  print(
    'Times include stalk and reap, averaged only over absorptions. '
    'Careful play understates defensive powers. This does not measure '
    'physical comfort, cooperative balance or player enjoyment.',
  );
  print('');
  print('| Gift | Level / rank | Wins / trials | Mean seconds |');
  print('| --- | --- | --- | --- |');
  for (final gift in ReaperGift.values) {
    for (final (level, rank) in [(1, 0), (20, 3)]) {
      var wins = 0;
      var ticks = 0;
      for (final temperament in Temperament.values) {
        for (var seed = 1; seed <= 8; seed++) {
          final sim = HuntSim(
            seed: seed,
            vessel: Vessel.penguin,
            ward: Rarity.rare.ward,
            stats: HuntStats.derive(
              HuntLoadout(
                reaperClass: gift.reaperClass,
                level: level,
                weapon: Weapon(gift.reaperClass.weapon, Temper.shrouded),
                soul: ReaperSoul(gift: gift, temperament: temperament),
                soulRank: rank,
              ),
            ),
          );
          final outcome = HuntBot.human(sim).play();
          if (outcome == null) {
            throw StateError('Unfinished ${gift.name} trial');
          }
          if (outcome.absorbed) {
            wins++;
            ticks += outcome.ticks;
          }
        }
      }
      final mean = wins == 0
          ? '—'
          : (ticks / wins / HuntRules.tickRate).toStringAsFixed(1);
      print('| ${gift.name} | $level / $rank | $wins / 32 | $mean |');
    }
  }
}
