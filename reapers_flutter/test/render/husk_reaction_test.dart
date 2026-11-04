import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/husk_reaction.dart';

/// A plight with only what a test changes.
HuskPlight plight({
  double ward = 1,
  bool fighting = true,
  double fury = 0,
  bool staggered = false,
  bool broken = false,
  double reap = 0,
  bool dying = false,
  bool frozen = false,
}) => (
  ward: ward,
  fighting: fighting,
  fury: fury,
  staggered: staggered,
  broken: broken,
  reap: reap,
  dying: dying,
  frozen: frozen,
);

/// Steps [reaction] for [seconds] at sixty frames a second.
void play(HuskReaction reaction, HuskPlight plight, double seconds) {
  for (var t = 0.0; t < seconds; t += 1 / 60) {
    reaction.step(1 / 60, plight);
  }
}

void main() {
  test('an unaware husk is calm: no fear, no fury, its face at rest', () {
    final reaction = HuskReaction();

    play(reaction, calmPlight, 2);

    expect(reaction.fear, 0);
    expect(reaction.rage, 0);
    expect(reaction.eyeWidth, closeTo(1, 1e-9));
    expect(reaction.browTilt, closeTo(0, 1e-9));
  });

  test('fear grows as the ward fails and fills it once broken', () {
    double fearAt(HuskPlight plight) {
      final reaction = HuskReaction();

      play(reaction, plight, 3);

      return reaction.fear;
    }

    final fresh = fearAt(plight());
    final worn = fearAt(plight(ward: 0.4));
    final broken = fearAt(plight(ward: 0, broken: true));

    expect(fresh, lessThan(worn));
    expect(worn, lessThan(broken));
    expect(broken, greaterThan(0.95));
  });

  test(
    'a terrified husk cowers, its eyes go wide and the inner ends of '
    'its brows lift; a furious one narrows its eyes and drives them down',
    () {
      final terrified = HuskReaction();
      final furious = HuskReaction();

      play(terrified, plight(ward: 0, broken: true), 3);
      play(furious, plight(fury: 1), 3);

      expect(terrified.cower, greaterThan(0.9));
      expect(terrified.squash, lessThan(0), reason: 'it shrinks into itself');
      expect(terrified.eyeHeight, greaterThan(1.5));
      expect(terrified.browTilt, greaterThan(0.4));

      expect(furious.fear, lessThan(0.05), reason: 'fury pushes fear aside');
      expect(furious.rage, greaterThan(0.95));
      expect(furious.eyeHeight, lessThan(0.6));
      expect(furious.browTilt, lessThan(-0.5));
    },
  );

  test('a blow drives the body the way it was struck, overshoots and '
      'settles back', () {
    final reaction = HuskReaction();
    const away = math.pi / 2; // straight back, along +z

    reaction.hit(away: away, side: 0, strength: 1);
    play(reaction, plight(), 0.06);

    expect(reaction.leanZ, greaterThan(0.05), reason: 'driven back');
    expect(
      reaction.leanZ,
      lessThan(0.16),
      reason: 'not so far that its seals leave where they are struck',
    );
    expect(reaction.leanX.abs(), lessThan(0.02));

    var leanedForward = false;

    for (var t = 0.0; t < 0.4; t += 1 / 60) {
      reaction.step(1 / 60, plight());
      leanedForward = leanedForward || reaction.leanZ < 0;
    }

    expect(leanedForward, isTrue, reason: 'it rocks forward again');

    play(reaction, plight(), 2);

    expect(reaction.leanZ.abs(), lessThan(0.01), reason: 'and comes to rest');
  });

  test('a blow on one side twists the body away from it', () {
    final left = HuskReaction()..hit(away: 0, side: 1, strength: 1);
    final right = HuskReaction()..hit(away: 0, side: -1, strength: 1);

    play(left, plight(), 0.05);
    play(right, plight(), 0.05);

    expect(left.twist, lessThan(0));
    expect(right.twist, greaterThan(0));
  });

  test('a blow squeezes the eyes shut for a moment', () {
    final reaction = HuskReaction();

    play(reaction, plight(), 1);

    final open = reaction.eyeHeight;

    reaction.hit(away: 0, side: 0, strength: 1);
    reaction.step(1 / 60, plight());

    expect(reaction.eyeHeight, lessThan(open * 0.3));

    play(reaction, plight(), 1.5);

    expect(reaction.eyeHeight, greaterThan(open * 0.9));
  });

  test('a heavy blow stops time for an instant, and only an instant', () {
    final reaction = HuskReaction()
      ..hit(away: 0, side: 0, strength: 1, heavy: true);

    expect(reaction.timeScale, lessThan(0.2));

    play(reaction, plight(), HuskReaction.hitStopSeconds + 0.02);

    expect(reaction.timeScale, 1);
  });

  test('a light blow does not stop time', () {
    final reaction = HuskReaction()..hit(away: 0, side: 0, strength: 0.3);

    expect(reaction.timeScale, 1);
  });

  test('ice holds a husk exactly as it was: blows do not move it', () {
    final reaction = HuskReaction();

    play(reaction, plight(ward: 0.3), 2);

    final fear = reaction.fear;

    reaction.hit(away: 0, side: 1, strength: 1, heavy: true);
    play(reaction, plight(ward: 0.3, frozen: true), 1);

    expect(reaction.leanX, closeTo(0, 0.02));
    expect(reaction.fear, fear);
    expect(reaction.timeScale, 1);
  });

  test('the reap drags it up towards the blade, screaming and flailing', () {
    final reaction = HuskReaction();
    var flailed = 0.0;

    for (var t = 0.0; t < 1.5; t += 1 / 60) {
      reaction.step(1 / 60, plight(ward: 0, broken: true, reap: 0.7));
      flailed = math.max(flailed, reaction.flail.abs());
    }

    expect(reaction.rise, greaterThan(0.2));
    expect(reaction.drag, greaterThan(0.25));
    expect(reaction.gape, greaterThan(0.7));
    expect(flailed, greaterThan(0.5));
  });

  test('as it dies its eyes flare and then go out, its jaw wide', () {
    final reaction = HuskReaction();

    play(reaction, plight(ward: 0, broken: true), 1);

    final living = reaction.eyeGlow;

    play(reaction, plight(ward: 0, dying: true), 0.15);

    expect(reaction.eyeGlow, greaterThan(living));
    expect(reaction.gape, greaterThan(0.85));
    expect(reaction.death, greaterThan(0.85));

    play(reaction, plight(ward: 0, dying: true), 1.5);

    expect(reaction.eyeGlow, lessThan(0.5));
  });
}
