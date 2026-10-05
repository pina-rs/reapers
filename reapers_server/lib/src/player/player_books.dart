import 'dart:async';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:serverpod_auth_idp_server/core.dart';

import '../generated/protocol.dart';
import '../solana/reaper_ledger.dart';
import '../solana/solana_endpoint.dart';

/// The one way a signed-in player's stored state is read or changed.
abstract final class PlayerBooks {
  /// Applies [rule] to the signed-in player's state and stores the result,
  /// opening their books first if this is the first they are heard of.
  ///
  /// The row is locked for the length of the change, so two requests from
  /// the same player cannot both spend the same anima. A rule the shared
  /// rules refuse reaches the game as a [Refused] naming the refusal.
  static Future<PlayerState> change(
    Session session,
    FutureOr<PlayerState> Function(PlayerState state) rule, {
    String? importAsset,
  }) => session.db.transaction((transaction) async {
    final record = await lockedRecord(session, transaction);
    final PlayerState next;

    try {
      final verified = await chainRequest(
        () => ReaperLedger.refresh(
          session,
          transaction,
          record,
          SolanaEndpoint.gateway,
          importAsset: importAsset,
        ),
      );
      next = await rule(verified);
    } on RuleBreach catch (breach) {
      throw Refused(refusal: breach.refusal.name);
    }

    if (identical(next, record.state)) {
      return next;
    }

    await ReaperLedger.save(session, transaction, next, SolanaEndpoint.gateway);

    await PlayerRecord.db.updateRow(
      session,
      record.copyWith(state: next, updatedAt: DateTime.now().toUtc()),
      transaction: transaction,
    );

    return next;
  });

  /// Updates the same locked row as game mutations, so a location update
  /// cannot overwrite a purchase or a hunt settlement.
  static Future<void> locate(Session session, GeoFix? fix) async {
    if (fix != null && !fix.isUsableAt(DateTime.now().toUtc())) {
      throw Refused(refusal: Refusal.locationUnusable.name);
    }

    await session.db.transaction((transaction) async {
      final record = await lockedRecord(session, transaction);
      final before = record.location;

      // Delayed requests must not move a player back to an older reading.
      if (fix != null &&
          before != null &&
          !fix.observedAt.isAfter(before.observedAt)) {
        return;
      }

      await PlayerRecord.db.updateRow(
        session,
        record.copyWith(location: fix),
        transaction: transaction,
      );
    });
  }

  static Future<GeoFix?> location(Session session) async {
    final record = await PlayerRecord.db.findFirstRow(
      session,
      where: (table) =>
          table.authUserId.equals(session.authenticated!.authUserId),
    );
    final fix = record?.location;
    return fix != null && fix.isUsableAt(DateTime.now().toUtc()) ? fix : null;
  }

  static Future<PlayerRecord> lockedRecord(
    Session session,
    Transaction transaction,
  ) async {
    final authUserId = session.authenticated!.authUserId;
    return await PlayerRecord.db.findFirstRow(
          session,
          where: (table) => table.authUserId.equals(authUserId),
          lockMode: LockMode.forUpdate,
          transaction: transaction,
        ) ??
        await PlayerRecord.db.insertRow(
          session,
          PlayerRecord(authUserId: authUserId, state: PlayerState.initial()),
          transaction: transaction,
        );
  }
}
