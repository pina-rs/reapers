import 'dart:convert';
import 'dart:typed_data';

import 'package:solana_kit/solana_kit.dart';

/// Exact Ed25519 message proof. No transaction signing or key custody occurs.
bool verifyWalletProof(String wallet, String message, String signature) {
  if (!isAddress(wallet) || signature.length > 88) return false;

  try {
    final bytes = base64Decode(signature);
    if (bytes.length != 64) return false;
    return verifySignature(
      getAddressEncoder().encode(Address(wallet)),
      SignatureBytes(bytes),
      Uint8List.fromList(utf8.encode(message)),
    );
  } on FormatException {
    return false;
  }
}
