import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// What the server signs when a reaper absorbs an NFT.
///
/// How well a reaper feeds is rolled here, off-chain, because randomness on
/// a chain is slow and costly. The roll still has to be something a chain
/// can trust, so the server signs what it decided. Anyone holding the
/// server's public key can check the signature, and a Solana program can
/// check it with the Ed25519 program before it burns the NFT and records the
/// result.
///
/// The message is 98 bytes, laid out so that a program can read it without
/// parsing anything:
///
/// | Bytes | Field |
/// | --- | --- |
/// | 0..8 | the ASCII tag `RPRSABS1`: what this is, and its version |
/// | 8..40 | the player |
/// | 40..72 | the NFT's mint |
/// | 72 | the reap grade: 0 ragged, 1 clean, 2 perfect |
/// | 73 | the feeding: 0 meagre, 1 fair, 2 rich, 3 feast |
/// | 74..78 | experience paid, unsigned, little-endian |
/// | 78..82 | anima paid, unsigned, little-endian |
/// | 82..90 | when, in seconds since 1970, little-endian |
/// | 90..98 | the nonce the feeding was rolled from, little-endian |
///
/// The player and the mint are 32 bytes each because that is the size of a
/// Solana address. Until players have wallets and NFTs have real mints, each
/// holds the SHA-256 of the stand-in identifier: the player's account id,
/// and the NFT's mock mint string. When the real addresses exist they go in
/// as they are and the tag becomes `RPRSABS2`.
abstract final class AbsorptionReceipt {
  /// The bytes at the start of every message.
  static const tag = 'RPRSABS1';

  /// The length of a message in bytes.
  static const length = 98;

  /// The message that vouches for [absorption] by the player [playerId].
  static Future<Uint8List> message({
    required String playerId,
    required Absorption absorption,
  }) async {
    final message = Uint8List(length);
    final data = ByteData.sublistView(message);

    message
      ..setAll(0, ascii.encode(tag))
      ..setAll(8, await _standIn(playerId))
      ..setAll(40, await _standIn(absorption.nft.mockMint));
    data
      ..setUint8(72, absorption.grade.index)
      ..setUint8(73, absorption.feeding.index)
      ..setUint32(74, absorption.xp, Endian.little)
      ..setUint32(78, absorption.anima, Endian.little)
      ..setInt64(
        82,
        absorption.at.millisecondsSinceEpoch ~/ 1000,
        Endian.little,
      )
      ..setUint64(90, absorption.nonce, Endian.little);

    return message;
  }

  /// The 32 bytes that stand for [identifier] until it is a real address.
  static Future<List<int>> _standIn(String identifier) async =>
      (await Sha256().hash(utf8.encode(identifier))).bytes;
}

/// The server's signing key.
///
/// The key is derived from one secret, `receiptSigningKey`, so that a
/// deployment only has to keep a string. Whoever holds that secret can mint
/// receipts for anything, so it is one of the secrets a deployed server
/// refuses to start without (see `lib/server.dart`).
final class ReceiptSigner {
  ReceiptSigner._(this._keyPair, this.publicKey);

  /// The name of the secret the key is derived from.
  static const secretName = 'receiptSigningKey';

  /// What the key is derived from in development and in tests, where no
  /// secret is configured. It is public, so receipts signed with it prove
  /// nothing: that is the point of refusing to deploy without a real one.
  static const developmentSecret = 'reapers-development-receipt-key';

  static final _signers = <String, Future<ReceiptSigner>>{};

  /// The signer whose key is derived from [secret]. Deriving a key takes a
  /// moment, so each secret's signer is made once and kept.
  static Future<ReceiptSigner> fromSecret(String secret) =>
      _signers[secret] ??= _derive(secret);

  static Future<ReceiptSigner> _derive(String secret) async {
    final seed = await Sha256().hash(utf8.encode(secret));
    final keyPair = await Ed25519().newKeyPairFromSeed(seed.bytes);
    final publicKey = await keyPair.extractPublicKey();

    return ReceiptSigner._(keyPair, Uint8List.fromList(publicKey.bytes));
  }

  final SimpleKeyPair _keyPair;

  /// The 32-byte Ed25519 public key receipts are checked against.
  final Uint8List publicKey;

  /// Signs [message] and returns the 64-byte signature in base64.
  Future<String> sign(Uint8List message) async =>
      base64Encode((await Ed25519().sign(message, keyPair: _keyPair)).bytes);

  /// Whether [signature], in base64, is this signer's over [message].
  Future<bool> verifies(Uint8List message, String signature) =>
      Ed25519().verify(
        message,
        signature: Signature(
          base64Decode(signature),
          publicKey: SimplePublicKey(publicKey, type: KeyPairType.ed25519),
        ),
      );
}
