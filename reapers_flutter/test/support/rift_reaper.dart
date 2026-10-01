import 'package:reapers_flutter/src/backend/rift_session.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// One reaper in a rift, played by the shared bot: it stands ready, keeps
/// its own fight in step with the frames it is sent, and reports how that
/// fight stands.
final class RiftReaper {
  RiftReaper(this.session, this.loadout, {this.cuts = true}) {
    session.frames.listen(_onFrame, onDone: () => done = true);
  }

  final RiftSession session;
  final HuntLoadout loadout;

  /// Whether this reaper carries the cut through once the ward breaks.
  final bool cuts;

  final frames = <RiftFrame>[];

  /// Whether the session has closed its stream.
  bool done = false;

  RiftSeat? _seat;
  HuntBot? _bot;
  bool _ready = false;

  RiftFrame get last => frames.last;

  void _onFrame(RiftFrame frame) {
    frames.add(frame);

    if (!_ready) {
      _ready = true;
      session.report(const RiftReport(ready: true));
    }

    if (frame.phase == RiftPhase.gathering || frame.phase.isOver) {
      return;
    }

    final seat = _seat ??= RiftSeat(
      seed: frame.seed,
      site: frame.site,
      wardMax: frame.wardMax,
      loadout: loadout,
    );
    final bot = _bot ??= HuntBot(
      seat.sim,
      reactionTicks: 16,
      missEvery: 7,
      reapSpeed: cuts ? 0.9 : 0,
    );

    while (seat.sim.tick < frame.tick && !seat.sim.isOver) {
      seat
        ..sync(ward: frame.ward, counted: frame.you.dealt)
        ..step(bot.decide());
    }

    session.report(seat.report);
  }
}
