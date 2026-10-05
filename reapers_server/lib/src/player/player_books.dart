import 'dart:async';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:serverpod_auth_idp_server/core.dart';

import '../generated/protocol.dart';

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
    FutureOr<PlayerState> Function(PlayerState state) rule,
  ) => session.db.transaction((transaction) async {
    final record = await _record(session, transaction);
    final PlayerState next;

    try {
      next = await rule(record.state);
    } on RuleBreach catch (breach) {
      throw Refused(refusal: breach.refusal.name);
    }

    if (identical(next, record.state)) {
      return next;
    }

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
      final record = await _record(session, transaction);
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

  static Future<PlayerRecord> _record(
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
