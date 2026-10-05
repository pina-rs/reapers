import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:solana_kit/solana_kit.dart' as kit;
import 'package:solana_kit_mobile_wallet_adapter/solana_kit_mobile_wallet_adapter.dart'
    as mwa;
import 'package:solana_kit_mobile_wallet_adapter_protocol/solana_kit_mobile_wallet_adapter_protocol.dart'
    as protocol;
import 'package:reapers_client/reapers_client.dart';

/// Requires the exact requested message before extracting the MWA signature.
String walletMessageSignature(String message, String payload) {
  final bytes = base64Decode(payload);
  final expected = utf8.encode(message);
  if (bytes.length != expected.length + 64 ||
      !listEquals(bytes.sublist(0, expected.length), expected)) {
    throw const FormatException('Wallet returned a different message');
  }
  return base64Encode(bytes.sublist(expected.length));
}

/// The wallet keeps every key and asks the player to approve a sign-in.
/// Android MWA is the supported transport; other platforms expose the preview.
Future<({UuidValue challengeId, String signature})> proveReaperWallet({
  required ChainStatus status,
  required Future<WalletChallenge> Function(String) challenge,
}) => mwa.transact((wallet) async {
  final authorization = await wallet.authorize(
    identity: protocol.AppIdentity(
      name: 'Reapers',
      uri: Uri.parse(status.identityUri),
    ),
    chain: status.chain,
  );
  if (authorization.accounts.isEmpty) {
    throw const FormatException('No authorized wallet account');
  }
  final account = authorization.accounts.first;
  final bytes = base64Decode(account.address);
  if (bytes.length != 32) throw const FormatException('Invalid wallet address');
  final address = kit.getAddressDecoder().decode(bytes).value;
  final proof = await challenge(address);
  final signed = await wallet.signMessages(
    addresses: [account.address],
    payloads: [base64Encode(utf8.encode(proof.message))],
  );
  if (signed.length != 1) {
    throw const FormatException('Expected one signed message');
  }
  return (
    challengeId: proof.challengeId,
    signature: walletMessageSignature(proof.message, signed.single),
  );
});
