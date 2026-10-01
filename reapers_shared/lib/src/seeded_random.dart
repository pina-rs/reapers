/// A small deterministic generator (mulberry32).
///
/// Everything a reaper's shuffle, a husk's path or a rift's schedule derives
/// from a seed goes through this class, so the game and the server always
/// agree. `dart:math`'s `Random` is not used because its sequence is not
/// guaranteed to stay the same between SDK releases.
final class SeededRandom {
  SeededRandom(int seed) : _state = seed & _mask;

  static const _mask = 0xFFFFFFFF;

  int _state;

  /// The next value in `[0, 2^32)`.
  int nextUint32() {
    _state = (_state + 0x6D2B79F5) & _mask;

    var t = _state;
    t = _multiply(t ^ (t >> 15), t | 1);
    t ^= (t + _multiply(t ^ (t >> 7), t | 61)) & _mask;

    return (t ^ (t >> 14)) & _mask;
  }

  /// The next value in `[0, 1)`.
  double nextDouble() => nextUint32() / 4294967296.0;

  /// The next integer in `[0, max)`.
  int nextInt(int max) {
    if (max <= 0) {
      throw ArgumentError.value(max, 'max', 'must be positive');
    }

    return (nextDouble() * max).floor();
  }

  /// The next value in `[min, max)`.
  double nextRange(double min, double max) => min + nextDouble() * (max - min);

  /// True with probability [chance].
  bool nextChance(double chance) => nextDouble() < chance;

  /// Picks one of [options] uniformly.
  T pick<T>(List<T> options) => options[nextInt(options.length)];

  /// Picks an index from [weights], each chosen in proportion to its weight.
  int pickWeighted(List<num> weights) {
    final total = weights.fold<double>(0, (sum, weight) => sum + weight);
    var roll = nextDouble() * total;

    for (var index = 0; index < weights.length; index++) {
      roll -= weights[index];

      if (roll < 0) {
        return index;
      }
    }

    return weights.length - 1;
  }

  /// 32-bit wrapping multiplication. The operands are below 2^32, so the low
  /// 32 bits of the 64-bit product are exact even when the product wraps.
  static int _multiply(int a, int b) => (a * b) & _mask;
}

/// Mixes [values] into one 32-bit seed (FNV-1a over each value's bytes).
///
/// Used wherever a seed is derived from identifiers, such as the seed of the
/// hunt for one husk, so the derivation is stable across platforms.
int mixSeed(Iterable<int> values) {
  var hash = 0x811C9DC5;

  for (final value in values) {
    for (var shift = 0; shift < 32; shift += 8) {
      hash ^= (value >> shift) & 0xFF;
      hash = (hash * 0x01000193) & 0xFFFFFFFF;
    }
  }

  return hash;
}

/// A stable 32-bit hash of [text], for seeding from string identifiers.
int hashText(String text) => mixSeed(text.codeUnits);
