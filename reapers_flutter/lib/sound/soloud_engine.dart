import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_soloud/flutter_soloud.dart';

import 'score.dart';
import 'score_engine.dart';

/// Plays the music through SoLoud.
///
/// One piece plays at a time, on a loop, read from disk as it goes so that a
/// ten-minute recording never has to sit in memory whole. Everything passes
/// through one low-pass filter, which is what the veil is.
final class SoLoudEngine implements ScoreEngine {
  final _sources = <Score, AudioSource>{};
  Future<void>? _starting;
  SoundHandle? _playing;
  int _asked = 0;
  bool _disposed = false;

  /// How quickly a change of [Mix] is eased in, so that nothing clicks.
  static const _ease = Duration(milliseconds: 180);

  SoLoud get _soloud => SoLoud.instance;

  @override
  Future<void> play(Score piece, Mix mix, {required Duration fade}) async {
    final asked = ++_asked;

    await (_starting ??= _start());

    final source = _sources[piece] ??= await _soloud.loadAsset(
      piece.asset,
      mode: LoadMode.disk,
    );

    // Loading takes a moment. If another piece was asked for meanwhile,
    // this one is no longer wanted.
    if (_disposed || asked != _asked) {
      return;
    }

    _fadeOut(fade);
    _setVeil(mix);

    final handle = _soloud.play(source, volume: 0, looping: true);

    _soloud.fadeVolume(handle, mix.volume, fade);
    _playing = handle;

    // A line for whoever is wondering why there is no sound. Never in a
    // release build.
    if (!kReleaseMode) {
      debugPrint('Music: ${piece.name} is playing');
    }
  }

  @override
  void shape(Mix mix) {
    final handle = _playing;

    if (handle == null || _disposed) {
      return;
    }

    _soloud.fadeVolume(handle, mix.volume, _ease);
    _setVeil(mix);
  }

  @override
  Future<void> silence({required Duration fade}) async {
    _asked++;

    if (!_disposed) {
      _fadeOut(fade);
    }
  }

  @override
  void hold({required bool held}) {
    final handle = _playing;

    if (handle != null && !_disposed) {
      _soloud.setPause(handle, held);
    }
  }

  @override
  void dispose() {
    _disposed = true;
    _playing = null;

    if (_starting != null) {
      _soloud.deinit();
    }
  }

  Future<void> _start() async {
    await _soloud.init();

    final veil = _soloud.filters.biquadResonantFilter..activate();

    // A plain low-pass with no ring to it.
    veil.type.value = 0;
    veil.resonance.value = 0.7;
    veil.wet.value = 1;
  }

  void _setVeil(Mix mix) =>
      _soloud.filters.biquadResonantFilter.frequency.value = mix.cutoff;

  void _fadeOut(Duration fade) {
    final handle = _playing;

    if (handle == null) {
      return;
    }

    _playing = null;
    _soloud
      ..fadeVolume(handle, 0, fade)
      ..scheduleStop(handle, fade);
  }
}
