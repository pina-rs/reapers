import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'player.dart';

part 'testing_difficulty.g.dart';

/// How forgiving hunts are while the game is being tested.
///
/// A build that is not a release starts on easy, so that whoever is trying
/// the game can bring a husk down and see a whole hunt, and can be switched
/// from the hunt's own threshold. A release build is always the game as it
/// is meant to be played, and has no switch.
@Riverpod(keepAlive: true)
class TestingDifficulty extends _$TestingDifficulty {
  /// Whether this build lets the difficulty be changed at all.
  static const available = !kReleaseMode;

  static const _key = 'reapers.testing.difficulty';

  @override
  Difficulty build() {
    if (!available) {
      return Difficulty.normal;
    }

    final stored = ref.watch(sharedPreferencesProvider).getString(_key);

    return Difficulty.values.asNameMap()[stored] ?? Difficulty.easy;
  }

  /// Moves on to the next difficulty, and from the hardest back to the
  /// easiest.
  Future<void> cycle() async {
    if (!available) {
      return;
    }

    final next =
        Difficulty.values[(state.index + 1) % Difficulty.values.length];

    state = next;
    await ref.read(sharedPreferencesProvider).setString(_key, next.name);
  }
}
