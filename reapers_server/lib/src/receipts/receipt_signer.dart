import 'dart:convert';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

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
