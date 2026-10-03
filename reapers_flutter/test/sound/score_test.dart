import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/sound/score.dart';
import 'package:reapers_shared/reapers_shared.dart';

HuntSim _hunt() => HuntSim(
  seed: 4,
  vessel: Vessel.ape,
  ward: Rarity.epic.ward,
  stats: HuntStats.derive(HuntLoadout.borrowed),
);

HuntInput _still(HuntSim sim) =>
    HuntInput.measured(aim: sim.pose.direction, turnRadiansPerSecond: 0);

/// Looks at the husk and swings at a charge, so the hunt runs on.
HuntInput _wary(HuntSim sim) => HuntInput.measured(
  aim: sim.pose.direction,
  turnRadiansPerSecond: sim.chargeProgress > 0.3
      ? sim.stats.parrySpeed + 0.5
      : 0,
);

void _until(HuntSim sim, bool Function() stop) {
  while (!stop() && !sim.isOver) {
    sim.step(_wary(sim));
  }
}

void main() {
  group('the veil', () {
    test('clear lets everything through and full lets only the lows', () {
      expect(const Mix(volume: 1, veil: 0).cutoff, closeTo(16000, 1));
      expect(const Mix(volume: 1, veil: 1).cutoff, closeTo(320, 1));
    });

    test('each step takes off the same share of what is left', () {
      const quarter = Mix(volume: 1, veil: 0.25);
      const half = Mix(volume: 1, veil: 0.5);
      const most = Mix(volume: 1, veil: 0.75);

      expect(
        half.cutoff / quarter.cutoff,
        closeTo(most.cutoff / half.cutoff, 1e-9),
      );
    });

    test('a change too small to hear is no change', () {
      const mix = Mix(volume: 0.5, veil: 0.5);

      expect(const Mix(volume: 0.505, veil: 0.5).differsFrom(mix), isFalse);
      expect(const Mix(volume: 0.55, veil: 0.5).differsFrom(mix), isTrue);
      expect(const Mix(volume: 0.5, veil: 0.6).differsFrom(mix), isTrue);
    });
  });

  group('the music of a sigil', () {
    test('draws nearer with every light, to where the stalk takes it up', () {
      final trace = SigilTrace(
        Sigil.draw(seed: 3, shape: Rarity.rare.sigil, gait: Gait.ground),
        centre: 0,
      );
      final heard = [sigilMix(trace)];

      while (!trace.isClosed) {
        trace.advance(
          trace.directionOf(trace.caught),
          turnSpeed: 0,
          deltaSeconds: 1 / 60,
        );

        if (trace.fill == 0) {
          heard.add(sigilMix(trace));
        }
      }

      expect(heard.first.volume, Mix.waiting.volume);
      expect(heard.first.veil, Mix.waiting.veil);

      for (var step = 1; step < heard.length; step++) {
        expect(heard[step].volume, greaterThan(heard[step - 1].volume));
        expect(heard[step].veil, lessThan(heard[step - 1].veil));
      }

      expect(heard.last.volume, closeTo(huntMix(_hunt()).volume, 1e-9));
      expect(heard.last.veil, closeTo(huntMix(_hunt()).veil, 1e-9));
    });
  });

  group('the music of a hunt', () {
    test('is far off while the husk has not been found', () {
      final sim = _hunt();
      final mix = huntMix(sim);

      expect(sim.phase, HuntPhase.stalk);
      expect(mix.volume, lessThan(0.4));
      expect(mix.veil, greaterThan(0.7));
    });

    test('draws nearer as the reaper steadies on the husk', () {
      final sim = _hunt();
      final unseen = huntMix(sim);

      while (sim.phase == HuntPhase.stalk && sim.steadiness < 0.8) {
        sim.step(_still(sim));
      }

      final steadied = huntMix(sim);

      expect(steadied.volume, greaterThan(unseen.volume));
      expect(steadied.veil, lessThan(unseen.veil));
    });

    test('comes clear when the fight is on', () {
      final sim = _hunt();

      _until(sim, () => sim.phase == HuntPhase.fight);

      expect(huntMix(sim).veil, 0);
      expect(huntMix(sim).volume, greaterThan(0.7));
    });

    test('drains away as the husk draws breath, to near silence', () {
      final sim = _hunt();

      _until(sim, () => sim.phase == HuntPhase.windup);

      final early = huntMix(sim);

      _until(sim, () => sim.shriekProgress > 0.8);

      final late = huntMix(sim);

      _until(sim, () => sim.phase == HuntPhase.shriek);

      final shriek = huntMix(sim);

      expect(late.volume, lessThan(early.volume));
      expect(late.veil, greaterThan(early.veil));
      expect(shriek.volume, lessThan(0.1));
      expect(shriek.veil, 1);
    });

    test('swells when the husk charges', () {
      final sim = _hunt();

      _until(sim, () => sim.phase == HuntPhase.charge);

      expect(huntMix(sim).volume, 1);
      expect(huntMix(sim).veil, 0);
    });

    test('is gone once the husk is reaped', () {
      final sim = HuntSim(
        seed: 4,
        vessel: Vessel.ape,
        ward: Rarity.common.ward,
        stats: HuntStats.derive(HuntLoadout.borrowed),
      );

      HuntBot(sim).play();

      expect(sim.phase, HuntPhase.absorbed);
      expect(huntMix(sim).volume, 0);
    });

    test('never asks for more than the engine can give', () {
      final sim = _hunt();
      final bot = HuntBot.human(sim);

      while (!sim.isOver) {
        bot.stepOnce();

        final mix = huntMix(sim);

        expect(mix.volume, inInclusiveRange(0, 1));
        expect(mix.veil, inInclusiveRange(0, 1));
      }
    });
  });
}
