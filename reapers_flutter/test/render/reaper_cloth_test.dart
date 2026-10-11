import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_appearance.dart';
import 'package:reapers_flutter/render/reaper_cloth.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  test(
    'switching form and clothing replaces the fit without double shortening',
    () {
      final genome = ReaperGenome.shuffle(11);
      for (final form in ReaperForm.values) {
        for (final clothing in [ReaperClothing.robe, ReaperClothing.tunic]) {
          final weights = reaperClothWeights(
            genome,
            ReaperAppearance(form: form, clothing: clothing),
            ReaperMotion.walk,
            .3,
          );
          final active = [
            for (final name in [
              'short_robe',
              'humanoid_robe',
              'humanoid_tunic',
            ])
              if (weights[name] == 1) name,
          ];
          expect(active, switch ((form, clothing)) {
            (ReaperForm.spirit, ReaperClothing.robe) => <String>[],
            (ReaperForm.spirit, _) => ['short_robe'],
            (ReaperForm.humanoid, ReaperClothing.robe) => ['humanoid_robe'],
            _ => ['humanoid_tunic'],
          });
          expect(weights['gaunt'], ReaperGenome.fraction(genome.gauntness));
          expect(weights.values.every((weight) => weight.isFinite), isTrue);
        }
      }
    },
  );
}
