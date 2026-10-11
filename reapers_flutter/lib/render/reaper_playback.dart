import 'package:reapers_shared/reapers_shared.dart';
import 'package:flutter/foundation.dart';

/// One clock for native and browser clips, including paused timeline sampling.
final class ReaperPlayback extends ChangeNotifier {
  ReaperMotion get motion => _motion;
  ReaperMotion _motion = ReaperMotion.idle;
  double get seconds => _seconds;
  double _seconds = 0;
  bool get playing => _playing;
  bool _playing = true;
  double get speed => _speed;
  double _speed = 1;
  double get progress => _seconds / _motion.seconds;

  void select(ReaperMotion motion, {bool play = true}) {
    _motion = motion;
    _seconds = 0;
    _playing = play;
    notifyListeners();
  }

  void toggle() {
    if (!_playing && _seconds >= _motion.seconds) _seconds = 0;
    _playing = !_playing;
    notifyListeners();
  }

  void replay() => select(_motion);

  void seek(double progress) {
    if (!progress.isFinite) throw ArgumentError.value(progress, 'progress');
    _seconds = progress.clamp(0, 1) * _motion.seconds;
    _playing = false;
    notifyListeners();
  }

  void setSpeed(double speed) {
    if (!speed.isFinite || speed <= 0) {
      throw ArgumentError.value(speed, 'speed');
    }
    _speed = speed;
    notifyListeners();
  }

  /// Renderers call this only when motion is permitted by accessibility settings.
  void advance(double delta) {
    if (!_playing || !delta.isFinite || delta <= 0) return;
    _seconds += delta * _speed;
    if (_motion.loops) {
      _seconds %= _motion.seconds;
    } else if (_seconds >= _motion.seconds) {
      _seconds = _motion.seconds;
      _playing = false;
    }
    notifyListeners();
  }
}
