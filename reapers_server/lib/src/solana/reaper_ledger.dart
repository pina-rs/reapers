import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:solana_kit/solana_kit.dart' show isAddress;

import '../generated/protocol.dart';
import 'solana_gateway.dart';

/// Refreshes ownership and locks canonical growth before any player mutation.
/// Offline saves never enter this ledger and cannot be imported as NFT growth.
abstract final class ReaperLedger {
  static Future<PlayerState> refresh(
    Session session,
    Transaction transaction,
    PlayerRecord player,
    SolanaGateway gateway, {
    String? importAsset,
  }) async {
    final assets = [
      for (final reaper in player.state.reapers)
        if (reaper.isNft) reaper.id,
    ];
    if (importAsset != null) {
      if (!isAddress(importAsset)) throw const RuleBreach(Refusal.notOwned);
      if (assets.contains(importAsset)) {
        throw const RuleBreach(Refusal.alreadyOwned);
      }
      if (assets.length >= PlayerState.maxNftReapers) {
        throw const RuleBreach(Refusal.rosterFull);
      }
      assets.add(importAsset);
    }
    if (assets.isEmpty) return player.state;

    final identity = await WalletIdentity.db.findFirstRow(
      session,
      where: (table) => table.authUserId.equals(player.authUserId),
      transaction: transaction,
    );
    if (identity == null) throw const RuleBreach(Refusal.walletUnproven);
    final owned = await gateway.owned(identity.address, assets);
    if (importAsset != null && !owned.containsKey(importAsset)) {
      throw const RuleBreach(Refusal.notOwned);
    }

    // A stable order prevents two owners refreshing overlapping rosters from
    // acquiring asset locks in opposite orders during a transfer.
    assets.sort();
    final roster = <ReaperState>[];
    for (final address in assets) {
      final genome = owned[address];
      if (genome == null) continue;
      final key = '${gateway.config.chain}:$address';
      final record = await ReaperRecord.db.findFirstRow(
        session,
        where: (table) => table.assetKey.equals(key),
        lockMode: LockMode.forUpdate,
        transaction: transaction,
      );
      if (record == null) {
        // Only a verified, first import creates growth. Never trust the
        // player's roster, a pasted genome or an offline experience count.
        final reaper = ReaperState.bound(id: address, genome: genome);
        await ReaperRecord.db.insertRow(
          session,
          ReaperRecord(
            assetKey: key,
            collection: gateway.config.collection!,
            state: reaper,
          ),
          transaction: transaction,
        );
        roster.add(reaper);
        continue;
      }
      if (record.collection != gateway.config.collection ||
          record.state.genome != genome) {
        throw const RuleBreach(Refusal.notOwned);
      }
      roster.add(record.state);
    }
    return player.state.reconcileReapers(roster);
  }

  static Future<void> save(
    Session session,
    Transaction transaction,
    PlayerState state,
    SolanaGateway gateway,
  ) async {
    for (final reaper in state.reapers) {
      if (!reaper.isNft) continue;
      final record = await ReaperRecord.db.findFirstRow(
        session,
        where: (table) =>
            table.assetKey.equals('${gateway.config.chain}:${reaper.id}'),
        transaction: transaction,
      );
      if (record == null) throw StateError('Missing verified asset ledger');
      await ReaperRecord.db.updateRow(
        session,
        record.copyWith(state: reaper),
        transaction: transaction,
      );
    }
  }
}
