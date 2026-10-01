import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';

import '../generated/protocol.dart';
import '../player/player_books.dart';
import 'rift_hall.dart';

/// Rifts: husks too strong for one reaper, fought by a circle of them.
class RiftEndpoint extends Endpoint {
  /// The rifts this server is hosting. Replaced in tests with a hall that
  /// keeps faster time.
  static RiftHall hall = RiftHall();

  @override
  bool get requireLogin => true;

  /// Joins the rift gathering at [site] and stays in it until it ends.
  ///
  /// The returned stream is the rift as this reaper sees it, several times a
  /// second; [reports] is how their own fight stands, sent as it changes.
  /// The last frame says how the rift ended. If it was sealed, the reaper
  /// has been paid by the time that frame arrives.
  ///
  /// Who the reaper is and what they fight with is read from their stored
  /// books, never from the game. What they report of their own fight is, in
  /// this version, taken at their word: a lone hunt is replayed before it
  /// pays, a rift is not yet (see docs/architecture.md).
  Stream<RiftFrame> enter(
    Session session,
    String site,
    Stream<RiftReport> reports,
  ) async* {
    final riftSite = RiftSite.values.asNameMap()[site];

    if (riftSite == null) {
      throw Refused(refusal: Refusal.notAttuned.name);
    }

    late final RiftEntrant entrant;

    await PlayerBooks.change(session, (state) {
      entrant = state.entrantFor(riftSite);

      return state;
    });

    final chair = hall.join(riftSite, entrant);
    final listening = reports.listen(chair.report);

    try {
      yield chair.frame;

      await for (final _ in chair.beats) {
        final frame = chair.frame;

        if (!frame.phase.isOver) {
          yield frame;
          continue;
        }

        if (frame.phase == RiftPhase.sealed) {
          final reward = chair.reward;

          await PlayerBooks.change(
            session,
            (state) => state.settleRift(xp: reward.xp, anima: reward.anima),
          );
        }

        yield frame;
        break;
      }
    } finally {
      await listening.cancel();
      chair.leave();
    }
  }
}
