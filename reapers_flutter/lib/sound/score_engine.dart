import 'score.dart';

/// Whatever actually makes the sound.
///
/// The game decides which piece plays and how it is heard; an engine does as
/// it is told. Keeping the two apart is what lets the deciding be tested
/// with nothing audible at all.
abstract interface class ScoreEngine {
  /// Brings [piece] in over [fade], heard as [mix], and lets whatever was
  /// playing fall away over the same time.
  Future<void> play(Score piece, Mix mix, {required Duration fade});

  /// Changes how the piece now playing is heard.
  void shape(Mix mix);

  /// Lets whatever is playing fall away over [fade].
  Future<void> silence({required Duration fade});

  /// Holds the music where it is, or lets it carry on: for when the game
  /// leaves the screen and comes back.
  void hold({required bool held});

  void dispose();
}

/// An engine that makes no sound: what the game has until one is given, and
/// all a test needs.
final class SilentEngine implements ScoreEngine {
  const SilentEngine();

  @override
  Future<void> play(Score piece, Mix mix, {required Duration fade}) async {}

  @override
  void shape(Mix mix) {}

  @override
  Future<void> silence({required Duration fade}) async {}

  @override
  void hold({required bool held}) {}

  @override
  void dispose() {}
}
