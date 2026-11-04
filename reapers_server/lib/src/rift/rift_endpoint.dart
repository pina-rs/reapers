import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';

import '../generated/protocol.dart';
import '../player/player_books.dart';
import '../player/player_endpoint.dart';
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
  ) => _enter(session, site, reports, demo: false);

  /// Joins the development demo circle at [site], where attunement is waived
  /// so several people can meet one enemy without first hunting a collection.
  ///
  /// Demo circles are kept apart from ordinary ones. The shared [RiftRoom]
  /// still owns the ward, the roster, the shrieks and each reaper's reward;
  /// only the entry requirement is waived. Development and test servers
  /// answer this; staging and production refuse it with the rest of the
  /// simulated economy.
  Stream<RiftFrame> enterDemo(
    Session session,
    String site,
    Stream<RiftReport> reports,
  ) => _enter(session, site, reports, demo: true);

  Stream<RiftFrame> _enter(
    Session session,
    String site,
    Stream<RiftReport> reports, {
    required bool demo,
  }) async* {
    // Reported results are not replayed yet; their rewards belong only in
    // the simulated development economy. This is also what keeps the demo
    // out of staging and production: there is deliberately no bypass flag.
    PlayerEndpoint.requireSimulation(session.serverpod.runMode);
    final riftSite = RiftSite.values.asNameMap()[site];

    if (riftSite == null) {
      throw Refused(refusal: Refusal.notAttuned.name);
    }

    late final RiftEntrant entrant;
    late final String reaperId;

    await PlayerBooks.change(session, (state) {
      entrant = demo ? _demoEntrant(state) : state.entrantFor(riftSite);
      reaperId = state.activeReaperId!;

      return state;
    });

    final chair = hall.join(riftSite, entrant, demo: demo);
    Object? reportFailure;
    final listening = reports.listen(
      (report) {
        final standing = report.standing;
        if (standing != null && !standing.isUsableAt(DateTime.now().toUtc())) {
          reportFailure = Refused(refusal: Refusal.locationUnusable.name);
          return;
        }
        chair.report(report);
      },
      onError: (Object error) => reportFailure = error,
      cancelOnError: true,
    );

    try {
      yield chair.frame;

      await for (final _ in chair.beats) {
        if (reportFailure case final failure?) {
          throw failure;
        }
        final frame = chair.frame;

        if (!frame.phase.isOver) {
          yield frame;
          continue;
        }

        if (frame.phase == RiftPhase.sealed) {
          final reward = chair.reward;

          // Each seat is settled in its own player row, from its own share
          // of the ward. The room reports a sealed rift once; the player
          // row's next load sees exactly one payment.
          await PlayerBooks.change(
            session,
            (state) => state.settleRift(
              xp: reward.xp,
              anima: reward.anima,
              reaperId: reaperId,
            ),
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

  /// What a demo reaper takes into the circle.
  ///
  /// The starter is the only reaper allowed into a rift until rift reports
  /// can be replayed, and the demo keeps that rule. It waives attunement
  /// alone, and only a development or test server reaches this method.
  static RiftEntrant _demoEntrant(PlayerState state) {
    if (!state.oathTaken) {
      throw const RuleBreach(Refusal.oathRequired);
    }

    if (state.activeReaper!.isNft) {
      throw const RuleBreach(Refusal.riftUnproven);
    }

    return RiftEntrant(name: state.genome.name, loadout: state.loadout);
  }
}
