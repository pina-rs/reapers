import 'seeded_random.dart';

/// How well a reaper fed on a husk it absorbed.
///
/// Two reapers who cut the same husk equally well do not get exactly the
/// same out of it. The keeper of the books rolls this once, when the husk is
/// absorbed, from a seed the player never sees or chooses: the server's own
/// for an online player. It scales the experience the husk gives, and
/// nothing else.
enum Feeding {
  /// There was less in it than it looked.
  meagre(share: 0.85, weight: 25),

  /// What the husk was worth, no more and no less.
  fair(share: 1.0, weight: 50),

  /// More than expected.
  rich(share: 1.25, weight: 20),

  /// Once in a while a husk gives twice what it should.
  feast(share: 2.0, weight: 5);

  const Feeding({required this.share, required this.weight});

  /// Draws a feeding from [seed].
  factory Feeding.roll(int seed) =>
      values[SeededRandom(mixSeed([seed, 0xFEED]))
          .pickWeighted([for (final feeding in values) feeding.weight])];

  /// Multiplier on the experience the husk gives.
  final double share;

  /// How often this is drawn, against the other weights.
  final int weight;

  /// The experience an average feeding gives, as a share of a fair one.
  /// A little above one: the roll is a small gift on average, never a tax.
  static double get expectedShare {
    var total = 0.0;
    var weights = 0;

    for (final feeding in values) {
      total += feeding.share * feeding.weight;
      weights += feeding.weight;
    }

    return total / weights;
  }
}
