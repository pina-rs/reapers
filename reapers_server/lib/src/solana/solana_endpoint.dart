import 'dart:async';
import 'dart:io';

import 'package:http/http.dart' as http;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart';
import 'package:serverpod_auth_idp_server/core.dart' as auth;
import 'package:solana_kit/solana_kit.dart' as kit;

import '../generated/protocol.dart';
import '../player/player_books.dart';
import 'solana_config.dart';
import 'solana_gateway.dart';
import 'wallet_proof.dart';

/// Wallet proof, balance and verified collection imports. Payment and NFT
/// exchange happen in the configured marketplace, signed by the user's wallet.
class SolanaEndpoint extends Endpoint {
  static SolanaGateway gateway = SolanaGateway(
    SolanaConfig.environment(),
    kit.createSolanaRpc(url: SolanaConfig.environment().rpcUrl),
  );

  @override
  bool get requireLogin => true;

  Future<ChainStatus> status(Session session) async {
    final identity = await WalletIdentity.db.findFirstRow(
      session,
      where: (table) =>
          table.authUserId.equals(session.authenticated!.authUserId),
    );
    final config = gateway.config;
    return ChainStatus(
      chain: config.chain,
      identityUri: config.identityUri.toString(),
      collection: config.collection,
      marketplaceUri: config.marketplaceUri?.toString(),
      wallet: identity?.address,
      lamports: identity == null
          ? null
          : await chainRequest(() => gateway.balance(identity.address)),
    );
  }

  Future<WalletChallenge> challenge(Session session, String address) async {
    if (gateway.config.identityUri.host.endsWith('.invalid')) {
      throw Refused(refusal: Refusal.chainUnconfigured.name);
    }
    if (!kit.isAddress(address)) {
      throw Refused(refusal: Refusal.walletUnproven.name);
    }
    await PlayerBooks.change(session, (state) => state);

    return session.db.transaction((transaction) async {
      final player = await PlayerBooks.lockedRecord(session, transaction);
      final previous = await WalletChallenge.db.findFirstRow(
        session,
        where: (table) => table.authUserId.equals(player.authUserId),
        transaction: transaction,
      );
      final now = DateTime.now().toUtc();
      final expires = now.add(const Duration(minutes: 2));
      final id = UuidValue.fromString(const Uuid().v4());
      final config = gateway.config;
      final proof = WalletChallenge(
        id: previous?.id,
        challengeId: id,
        authUserId: player.authUserId,
        address: address,
        expiresAt: expires,
        message:
            '${config.identityUri.host} wants you to sign in with your Solana account:\n'
            '$address\n\nSign in to Reapers. This does not authorize a payment.\n\n'
            'URI: ${config.identityUri}\nVersion: 1\nChain ID: ${config.chain}\n'
            'Nonce: $id\nIssued At: ${now.toIso8601String()}\nExpiration Time: ${expires.toIso8601String()}',
      );
      return previous == null
          ? WalletChallenge.db.insertRow(
              session,
              proof,
              transaction: transaction,
            )
          : WalletChallenge.db.updateRow(
              session,
              proof,
              transaction: transaction,
            );
    });
  }

  /// First proof links this guest; later proofs restore that wallet's account.
  /// Accounts are switched rather than silently merging two players' saves.
  Future<auth.AuthSuccess> login(
    Session session,
    UuidValue challengeId,
    String signature,
  ) => session.db.transaction((transaction) async {
    await PlayerBooks.lockedRecord(session, transaction);
    final challenge = await WalletChallenge.db.findFirstRow(
      session,
      where: (table) =>
          table.challengeId.equals(challengeId) &
          table.authUserId.equals(session.authenticated!.authUserId),
      lockMode: LockMode.forUpdate,
      transaction: transaction,
    );
    if (challenge == null ||
        challenge.consumed ||
        !challenge.expiresAt.isAfter(DateTime.now().toUtc()) ||
        !verifyWalletProof(challenge.address, challenge.message, signature)) {
      throw Refused(refusal: Refusal.walletUnproven.name);
    }
    final linked = await WalletIdentity.db.findFirstRow(
      session,
      where: (table) => table.address.equals(challenge.address),
      transaction: transaction,
    );
    if (linked == null) {
      final own = await WalletIdentity.db.findFirstRow(
        session,
        where: (table) =>
            table.authUserId.equals(session.authenticated!.authUserId),
        transaction: transaction,
      );
      if (own != null) throw Refused(refusal: Refusal.alreadyOwned.name);
      await WalletIdentity.db.insertRow(
        session,
        WalletIdentity(
          address: challenge.address,
          authUserId: session.authenticated!.authUserId,
        ),
        transaction: transaction,
      );
    }
    await WalletChallenge.db.updateRow(
      session,
      challenge.copyWith(consumed: true),
      transaction: transaction,
    );
    return auth.AuthServices.instance.tokenManager.createToken(
      session,
      authUserId: linked?.authUserId ?? session.authenticated!.authUserId,
      method: 'solana',
      transaction: transaction,
    );
  });

  Future<PlayerState> importReaper(Session session, String asset) =>
      PlayerBooks.change(session, (state) => state, importAsset: asset);
}

/// External chain failures stay failures: never turn a timeout into an empty
/// roster or let an unverified asset earn growth. No RPC URL is exposed.
Future<T> chainRequest<T>(Future<T> Function() request) async {
  try {
    return await request();
  } on RuleBreach catch (error) {
    throw Refused(refusal: error.refusal.name);
  } on TimeoutException {
    throw Refused(refusal: Refusal.chainUnavailable.name);
  } on SocketException {
    throw Refused(refusal: Refusal.chainUnavailable.name);
  } on http.ClientException {
    throw Refused(refusal: Refusal.chainUnavailable.name);
  } on kit.SolanaError {
    throw Refused(refusal: Refusal.chainUnavailable.name);
  } on FormatException {
    throw Refused(refusal: Refusal.chainUnavailable.name);
  }
}
