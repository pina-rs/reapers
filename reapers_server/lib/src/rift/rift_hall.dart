import 'dart:async';
import 'dart:math';

import 'package:reapers_shared/reapers_shared.dart';

/// The rifts open on this server right now.
///
/// A rift lives in the memory of the server process that hosts it, so every
/// reaper in one circle has to be connected to the same process. That is
/// true of a single server and is the first thing to change when there are
/// several: see docs/architecture.md.
final class RiftHall {
  RiftHall({
    this.beat = const Duration(milliseconds: 100),
    this.ticksPerBeat,
    Random? random,
  }) : _random = random ?? Random.secure();

  /// How often each rift is moved on and its reapers are sent a frame.
  final Duration beat;

  /// Ticks played each beat. Null plays as many as real time has passed,
  /// which is what a real rift wants; a test passes a number to play a rift
  /// through faster than it could be fought.
  final int? ticksPerBeat;

  final Random _random;
  final _gathering = <({RiftSite site, bool demo}), HostedRift>{};

  /// Seats [entrant] in the rift gathering at [site], opening a new one when
  /// none is gathering or the one that is has filled.
  ///
  /// A [demo] circle is kept apart from the real one at the same site, so a
  /// development demo can never be mixed into a circle of ordinary reapers.
  RiftChair join(RiftSite site, RiftEntrant entrant, {bool demo = false}) {
    final key = (site: site, demo: demo);
    var rift = _gathering[key];

    if (rift == null || !rift.room.isOpen) {
      rift = HostedRift(
        RiftRoom(site: site, seed: _random.nextInt(1 << 32)),
        beat: beat,
        ticksPerBeat: ticksPerBeat,
      );
      _gathering[key] = rift;
    }

    return rift.seat(entrant);
  }
}

/// One rift being kept in time by this server.
final class HostedRift {
  HostedRift(this.room, {required this.beat, required this.ticksPerBeat});

  final RiftRoom room;
  final Duration beat;
  final int? ticksPerBeat;

  final _beats = StreamController<void>.broadcast();
  final _watch = Stopwatch();
  Timer? _timer;
  int _ticksPlayed = 0;

  /// A rift that fell this far behind catches up no further in one beat.
  static const _maxTicksPerBeat = 60;

  /// Fires each time the room has been moved on.
  Stream<void> get beats => _beats.stream;

  /// Seats [entrant] and starts the rift's clock if this is its first reaper.
  RiftChair seat(RiftEntrant entrant) {
    final chair = RiftChair._(this, room.join(entrant));

    if (_timer == null) {
      _watch.start();
      _timer = Timer.periodic(beat, (_) => _play());
    }

    return chair;
  }

  void _play() {
    final due =
        _watch.elapsedMicroseconds *
        HuntRules.tickRate ~/
        Duration.microsecondsPerSecond;
    final ticks =
        ticksPerBeat ?? (due - _ticksPlayed).clamp(0, _maxTicksPerBeat);

    _ticksPlayed = due;
    room.advance(ticks);
    _beats.add(null);

    if (room.phase.isOver) {
      _stop();
    }
  }

  void _leave(int seat) {
    room.leave(seat);

    // Nobody is left to fight or to be told how it ended.
    if (room.isAbandoned) {
      _stop();
    }
  }

  void _stop() {
    _timer?.cancel();
    _watch.stop();
    unawaited(_beats.close());
  }
}

/// One reaper's seat in a hosted rift.
final class RiftChair {
  RiftChair._(this._rift, this._seat);

  final HostedRift _rift;
  final int _seat;

  /// Fires each time the rift has moved on, and closes when it stops.
  Stream<void> get beats => _rift.beats;

  /// The rift as this reaper sees it now.
  RiftFrame get frame => _rift.room.frameFor(_seat);

  /// What the rift paid this reaper.
  RiftReward get reward => _rift.room.rewardFor(_seat);

  void report(RiftReport report) => _rift.room.report(_seat, report);

  void leave() => _rift._leave(_seat);
}
