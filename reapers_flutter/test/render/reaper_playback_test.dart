import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_playback.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  test('one-shots stop, scrub stays still, and replay starts at zero', () {
    final clock = ReaperPlayback();
    addTearDown(clock.dispose);
    clock.select(ReaperMotion.finisherPalm);
    clock.advance(10);
    expect(clock.progress, 1);
    expect(clock.playing, isFalse);
    clock.seek(.5);
    clock.advance(1);
    expect(clock.progress, .5);
    clock.replay();
    expect(clock.seconds, 0);
    expect(clock.playing, isTrue);
    clock.setSpeed(.5);
    clock.advance(1);
    expect(clock.seconds, .5);
    clock.select(ReaperMotion.jump, play: false);
    expect(clock.seconds, 0);
    expect(clock.playing, isFalse);
  });

  test('loops retain overflow, invalid time cannot corrupt the clock', () {
    final clock = ReaperPlayback();
    addTearDown(clock.dispose);
    clock.select(ReaperMotion.walk);
    clock.advance(2 * ReaperMotion.walk.seconds + .5);
    expect(clock.seconds, closeTo(.5, 1e-10));
    clock.advance(double.nan);
    clock.advance(-1);
    expect(clock.seconds, closeTo(.5, 1e-10));
    expect(() => clock.seek(double.nan), throwsArgumentError);
    expect(() => clock.setSpeed(0), throwsArgumentError);
    expect(() => clock.setSpeed(double.infinity), throwsArgumentError);
    clock.seek(2);
    expect(clock.progress, 1);
    clock.toggle();
    expect(clock.seconds, 0);
  });

  test(
    'weapon finishers have explicit capabilities, hand moves work for all',
    () {
      expect(ReaperMotion.values, hasLength(32));
      expect(ReaperMotion.finisherImpale.supports(WeaponKind.spear), isTrue);
      expect(ReaperMotion.finisherImpale.supports(WeaponKind.bell), isFalse);
      for (final weapon in WeaponKind.values) {
        for (final motion in ReaperMotion.values.where(
          (value) => value.unarmed,
        )) {
          expect(motion.supports(weapon), isTrue);
        }
      }
      for (final motion in ReaperMotion.values) {
        if (motion.impact case final impact?) {
          expect(impact, inExclusiveRange(0, 1));
        }
      }
    },
  );
}
