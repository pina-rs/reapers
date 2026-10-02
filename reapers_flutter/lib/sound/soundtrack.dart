import 'dart:async';

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/state/player.dart';
import 'score.dart';
import 'score_engine.dart';

part 'soundtrack.g.dart';

/// What makes the sound. The app hands in a real engine at startup; without
/// one the game is silent, which is what every test wants.
@Riverpod(keepAlive: true)
ScoreEngine scoreEngine(Ref ref) => const SilentEngine();

/// Whether the player wants music at all.
@Riverpod(keepAlive: true)
class MusicEnabled extends _$MusicEnabled {
  static const _key = 'reapers.music';

  @override
  bool build() => ref.watch(sharedPreferencesProvider).getBool(_key) ?? true;

  Future<void> toggle() async {
    state = !state;
    await ref.read(sharedPreferencesProvider).setBool(_key, state);
  }
}

/// The game's music: which piece is playing, and how it is heard.
@Riverpod(keepAlive: true)
Soundtrack soundtrack(Ref ref) {
  final soundtrack = Soundtrack(
    ref.watch(scoreEngineProvider),
    enabled: ref.read(musicEnabledProvider),
  );

  ref
    ..listen(
      musicEnabledProvider,
      (_, enabled) => soundtrack.setEnabled(enabled: enabled),
    )
    ..onDispose(soundtrack.dispose);

  return soundtrack;
}

/// Decides what the engine is told. Screens say which piece belongs to them
/// and how it should be heard; this remembers it, so that music turned off
/// and on again comes back as it should be, and says nothing to the engine
/// that would not be heard.
final class Soundtrack {
  Soundtrack(this._engine, {required this._enabled});

  /// How long one piece takes to give way to another.
  static const crossfade = Duration(milliseconds: 1600);

  final ScoreEngine _engine;
  bool _enabled;
  Score? _piece;
  Mix _mix = Mix.open;

  /// The piece that is playing, or would be if music were on.
  Score? get piece => _piece;

  /// How the piece is being heard.
  Mix get mix => _mix;

  /// Makes [piece] the music, heard as [mix]. Asking for the piece already
  /// playing only changes how it is heard.
  void play(Score piece, {Mix mix = Mix.open}) {
    if (piece == _piece) {
      shape(mix);

      return;
    }

    _piece = piece;
    _mix = mix;

    if (_enabled) {
      unawaited(_engine.play(piece, mix, fade: crossfade));
    }
  }

  /// Changes how the piece now playing is heard.
  void shape(Mix mix) {
    if (!mix.differsFrom(_mix)) {
      return;
    }

    _mix = mix;

    if (_enabled && _piece != null) {
      _engine.shape(mix);
    }
  }

  void setEnabled({required bool enabled}) {
    if (enabled == _enabled) {
      return;
    }

    _enabled = enabled;

    final piece = _piece;

    if (!enabled) {
      unawaited(_engine.silence(fade: crossfade));
    } else if (piece != null) {
      unawaited(_engine.play(piece, _mix, fade: crossfade));
    }
  }

  /// Holds the music while the game is off the screen.
  void hold({required bool held}) => _engine.hold(held: held);

  void dispose() => _engine.dispose();
}
