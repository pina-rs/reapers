import 'dart:io';

import 'package:solana_kit/solana_kit.dart';

/// Public deployment settings. No signing key or wallet secret lives here.
final class SolanaConfig {
  SolanaConfig({
    required this.chain,
    required this.rpcUrl,
    required this.identityUri,
    this.collection,
    this.marketplaceUri,
  }) {
    if (!genesisHashes.containsKey(chain) ||
        Uri.parse(rpcUrl).scheme != 'https' ||
        identityUri.scheme != 'https' ||
        identityUri.host.isEmpty ||
        identityUri.userInfo.isNotEmpty ||
        identityUri.hasQuery ||
        identityUri.hasFragment ||
        !identityUri.path.endsWith('/') ||
        (collection != null && !isAddress(collection!)) ||
        (marketplaceUri != null && marketplaceUri!.scheme != 'https')) {
      throw const FormatException('Invalid Solana deployment settings');
    }
  }

  factory SolanaConfig.environment() {
    final env = Platform.environment;
    final chain = env['REAPERS_SOLANA_CHAIN'] ?? 'solana:devnet';
    if (env['REAPERS_REAPER_COLLECTION'] != null &&
        env['REAPERS_WALLET_IDENTITY_URI'] == null) {
      throw const FormatException(
        'A reaper collection requires its wallet identity origin',
      );
    }
    return SolanaConfig(
      chain: chain,
      rpcUrl:
          env['REAPERS_SOLANA_RPC_URL'] ??
          (chain == 'solana:mainnet'
              ? 'https://api.mainnet-beta.solana.com'
              : 'https://api.devnet.solana.com'),
      identityUri: Uri.parse(
        env['REAPERS_WALLET_IDENTITY_URI'] ?? 'https://reapers.invalid/',
      ),
      collection: env['REAPERS_REAPER_COLLECTION'],
      marketplaceUri: switch (env['REAPERS_REAPER_MARKET_URL']) {
        final String value => Uri.parse(value),
        _ => null,
      },
    );
  }

  static const genesisHashes = {
    'solana:devnet': 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG',
    'solana:mainnet': '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d',
  };

  final String chain;
  final String rpcUrl;
  final Uri identityUri;
  final String? collection;
  final Uri? marketplaceUri;
}
