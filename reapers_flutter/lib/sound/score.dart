import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

/// A piece of the game's music. Each is one recording, played on a loop.
enum Score {
  /// The reaper alone in the dark: a Bach prelude, on a piano.
  sanctum('assets/music/sanctum.ogg'),

  /// A hunt: slow, unresolved strings.
  hunt('assets/music/hunt.ogg'),

  /// A rift: the same strings, lower and graver.
  rift('assets/music/rift.ogg');

  const Score(this.asset);

  final String asset;
}

/// How a piece is heard at one moment.
///
/// The music is never switched for another tune as a hunt goes on. The same
/// recording is brought nearer or pushed away: louder or quieter, and clear
/// or muffled, as if heard through the veil the hunt takes place behind.
final class Mix {
  const Mix({required this.volume, required this.veil});

  /// The piece as recorded, at a level that sits under everything else.
  static const open = Mix(volume: 0.55, veil: 0);

  /// Something is nearby and has not shown itself: the music is a long way
  /// off.
  static const waiting = Mix(volume: 0.22, veil: 0.9);

  /// The husk has shown itself and not yet seen the reaper: nearer, still
  /// muffled.
  static const unseen = Mix(volume: 0.3, veil: 0.8);

  /// From 0 (silent) to 1 (as loud as the game gets).
  final double volume;

  /// From 0 (clear) to 1 (muffled until only the low strings come through).
  final double veil;

  /// The frequency, in hertz, above which a veil of this weight lets
  /// nothing through. Each step of the veil takes off the same share of what
  /// is left, which is how the ear measures it.
  double get cutoff => _clear * math.pow(_muffled / _clear, veil.clamp(0, 1));

  static const _clear = 16000.0;
  static const _muffled = 320.0;

  /// Whether going from [other] to this would be heard: a change too small
  /// to notice is not worth telling the engine about.
  bool differsFrom(Mix other) =>
      (volume - other.volume).abs() > 0.01 || (veil - other.veil).abs() > 0.01;

  @override
  String toString() => 'Mix(volume $volume, veil $veil)';
}

/// How the hunt's music is heard while its sigil is traced: far off at
/// first, and nearer with every light that catches, until it is where the
/// stalk takes it up.
Mix sigilMix(SigilTrace trace) {
  final drawn =
      (trace.caught + trace.fill) / trace.sigil.lights.length.toDouble();

  return Mix(
    volume:
        Mix.waiting.volume + (Mix.unseen.volume - Mix.waiting.volume) * drawn,
    veil: Mix.waiting.veil + (Mix.unseen.veil - Mix.waiting.veil) * drawn,
  );
}

/// How the hunt's music is heard at this moment of [sim].
///
/// The music follows what the hunt asks of the player. It comes clear when
/// the fight is on, drains away as the husk draws breath so that the shriek
/// is met in near silence, swells as the husk charges, and is gone by the
/// time the husk has come apart: the quiet after the reap is part of the
/// hunt.
Mix huntMix(HuntSim sim) => switch (sim.phase) {
  // Unseen, the music is far off. Holding steady on the husk draws it in.
  HuntPhase.stalk => Mix(
    volume: Mix.unseen.volume + 0.15 * sim.steadiness,
    veil: Mix.unseen.veil - 0.3 * sim.steadiness,
  ),
  HuntPhase.fight => Mix(volume: sim.staggered ? 1 : 0.8, veil: 0),
  HuntPhase.windup => Mix(
    volume: 0.8 - 0.68 * sim.shriekProgress,
    veil: 0.9 * sim.shriekProgress,
  ),
  HuntPhase.shriek => const Mix(volume: 0.06, veil: 1),
  HuntPhase.charge => const Mix(volume: 1, veil: 0),
  HuntPhase.broken => const Mix(volume: 0.45, veil: 0.45),
  HuntPhase.absorbed => const Mix(volume: 0, veil: 0.6),
  HuntPhase.escaped => const Mix(volume: 0.25, veil: 0.85),
};
