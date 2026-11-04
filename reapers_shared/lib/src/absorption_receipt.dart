import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

import 'player_state.dart';

/// What a keeper signs when a reaper absorbs an NFT.
///
/// How well a reaper feeds is rolled off-chain, because randomness on a
/// chain is slow and costly. The roll still has to be something a chain can
/// trust, so the keeper signs what it decided. Anyone holding the keeper's
/// public key can check the signature, and a Solana program can check it
/// with the Ed25519 program before it burns the NFT and records the result.
///
/// The server signs these, and so does the simnet on the phone, so the
/// layout is written once, here.
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
/// holds the SHA-256 of the stand-in identifier: the player's id, and the
/// NFT's mock mint string. When the real addresses exist they go in as they
/// are and the tag becomes `RPRSABS2`.
abstract final class AbsorptionReceipt {
  /// The bytes at the start of every message.
  static const tag = 'RPRSABS1';

  /// The length of a message in bytes.
  static const length = 98;

  /// The message that vouches for [absorption] by the player [playerId].
  static Uint8List message({
    required String playerId,
    required Absorption absorption,
  }) {
    final message = Uint8List(length);
    final data = ByteData.sublistView(message);

    message
      ..setAll(0, ascii.encode(tag))
      ..setAll(8, _standIn(playerId))
      ..setAll(40, _standIn(absorption.nft.mockMint));
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
  static List<int> _standIn(String identifier) =>
      sha256.convert(utf8.encode(identifier)).bytes;
}
