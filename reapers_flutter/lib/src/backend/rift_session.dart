import 'dart:async';

import 'package:reapers_shared/reapers_shared.dart';

/// One reaper's place in a rift, as the game talks to it.
///
/// Whoever hosts the rift, the phone alone or the server for many, the game
/// sees the same thing: a stream of [RiftFrame]s coming one way and this
/// reaper's [RiftReport]s going the other.
abstract interface class RiftSession {
  /// The rift as this reaper sees it, several times a second. The last frame
  /// says how the rift ended, and then the stream closes.
  Stream<RiftFrame> get frames;

  /// Tells the rift how this reaper's own fight stands.
  void report(RiftReport report);

  /// Walks away. A rift left before it ends pays this reaper nothing.
  void leave();
}

/// A rift hosted on the phone itself: one person, and shades in every other
/// seat.
///
/// The room and its rules are the shared ones, so this plays exactly as a
/// rift on the server does with nobody else in it.
final class LocalRiftSession implements RiftSession {
  LocalRiftSession({
    required RiftRoom room,
    required int seat,
    required this._onSealed,
    required this._microseconds,
  }) : _room = room,
       _seat = seat {
    _timer = Timer.periodic(_beat, (_) => unawaited(_advance()));
    _frames.add(room.frameFor(seat));
  }

  /// How often the room is moved on and a frame is sent.
  static const _beat = Duration(milliseconds: 50);

  /// The room catches up at most this many ticks at once, so a phone that
  /// was put to sleep does not wake to a rift that ran without it.
  static const _maxTicksPerBeat = 30;

  final RiftRoom _room;
  final int _seat;

  /// Settles the books once the rift is sealed.
  final Future<void> Function(RiftReward reward) _onSealed;

  /// Microseconds since the session opened, by the keeper's clock.
  final int Function() _microseconds;
  final _frames = StreamController<RiftFrame>();

  late final Timer _timer;
  int _ticksPlayed = 0;
  bool _closing = false;

  @override
  Stream<RiftFrame> get frames => _frames.stream;

  @override
  void report(RiftReport report) => _room.report(_seat, report);

  @override
  void leave() {
    if (_closing) {
      return;
    }

    _closing = true;
    _timer.cancel();
    _room.leave(_seat);
    // Closing only finishes once a listener has been told, and there may
    // never have been one, so nothing waits on it.
    unawaited(_frames.close());
  }

  Future<void> _advance() async {
    if (_closing) {
      return;
    }

    final due =
        _microseconds() * HuntRules.tickRate ~/ Duration.microsecondsPerSecond;
    final ticks = due - _ticksPlayed;

    _ticksPlayed = due;
    _room.advance(ticks.clamp(0, _maxTicksPerBeat));

    if (!_room.phase.isOver) {
      _frames.add(_room.frameFor(_seat));
      return;
    }

    _closing = true;
    _timer.cancel();

    try {
      // The books are settled before the last frame is shown, so whoever
      // reads the player after it sees what the rift paid.
      if (_room.phase == RiftPhase.sealed) {
        await _onSealed(_room.rewardFor(_seat));
      }

      _frames.add(_room.frameFor(_seat));
    } catch (error, stack) {
      _frames.addError(error, stack);
    } finally {
      unawaited(_frames.close());
    }
  }
}
