/// Levels and what they are worth.
abstract final class Progression {
  /// The highest level a reaper reaches.
  static const levelCap = 30;

  /// Each level adds this fraction of a level-one strike.
  static const strikePerLevel = 0.07;

  /// Each completed collection set adds this fraction to every strike.
  static const strikePerSet = 0.03;

  /// Experience needed to go from [level] to the next one.
  static int xpToNext(int level) {
    final step = level - 1;

    return 90 + 55 * step + 6 * step * step;
  }

  /// Total experience a reaper holds on reaching [level].
  static int xpForLevel(int level) {
    var total = 0;

    for (var current = 1; current < level; current++) {
      total += xpToNext(current);
    }

    return total;
  }

  /// The level reached with [xp] total experience.
  static int levelForXp(int xp) {
    var level = 1;
    var remaining = xp;

    while (level < levelCap && remaining >= xpToNext(level)) {
      remaining -= xpToNext(level);
      level++;
    }

    return level;
  }

  /// How far [xp] has carried a reaper through its current level, from 0 to
  /// 1. A reaper at the cap is always at 1.
  static double levelProgress(int xp) {
    final level = levelForXp(xp);

    if (level >= levelCap) {
      return 1;
    }

    return (xp - xpForLevel(level)) / xpToNext(level);
  }
}
