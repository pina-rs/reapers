import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_appearance.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  final genome = ReaperGenome.shuffle(11).copyWith(head: 2, mantle: 1, robe: 3);

  test(
    'spirit and humanoid forms are exclusive; clothing never hides a hand',
    () {
      for (final form in ReaperForm.values) {
        for (final clothing in ReaperClothing.values) {
          final appearance = ReaperAppearance(form: form, clothing: clothing);
          bool? visible(String name) =>
              reaperPartVisible(name, genome, Outfit.shrouded, appearance);
          expect(visible('humanoid_form'), form == ReaperForm.humanoid);
          expect(visible('spirit_form'), form == ReaperForm.spirit);
          expect(visible('robe_3'), clothing != ReaperClothing.unrobed);
          expect(visible('robe_0'), isFalse);
          expect(
            visible('arm_l_lower_sleeve'),
            clothing != ReaperClothing.unrobed,
          );
          expect(visible('arm_l_hand'), isNull);
          expect(visible('head_2'), isTrue);
          expect(visible('head_0'), isFalse);
          expect(visible('face_2'), isNull);
        }
      }
    },
  );

  test('removing shoulder pieces does not remove heads or carried skulls', () {
    const appearance = ReaperAppearance(mantle: false, holding: false);
    expect(
      reaperPartVisible('mantle_1', genome, Outfit.shrouded, appearance),
      isFalse,
    );
    expect(
      reaperPartVisible('head_2', genome, Outfit.shrouded, appearance),
      isTrue,
    );
    expect(
      reaperPartVisible('lantern_head', genome, Outfit.shrouded, appearance),
      isNull,
    );
  });

  test('empty hands exclude weapon attacks but retain reactions and hand finishers', () {
    for (final motion in [
      ReaperMotion.slash,
      ReaperMotion.thrust,
      ReaperMotion.parry,
      ReaperMotion.finisherImpale,
      ReaperMotion.finisherReap,
    ]) {
      expect(motion.supports(null), isFalse);
    }
    for (final motion in [
      ReaperMotion.laugh,
      ReaperMotion.sigma,
      ReaperMotion.finisherPalm,
      ReaperMotion.finisherUppercut,
      ReaperMotion.finisherSoulCrush,
    ]) {
      expect(motion.supports(null), isTrue);
    }
  });
}
