import 'dart:convert';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_mpl_core/solana_kit_mpl_core.dart' as core;
import 'package:solana_kit_rpc_api/solana_kit_rpc_api.dart';

import 'solana_config.dart';

/// Solana Kit is the sole boundary for address codecs, RPC and Core decoding.
final class SolanaGateway {
  SolanaGateway(this.config, this.rpc);
  final SolanaConfig config;
  final Rpc rpc;

  Future<void> checkCluster() async {
    // getGenesisHash has no typed convenience method in Kit 0.10. Check each
    // operation rather than trusting a URL or a permanently cached result.
    final genesis = await rpc
        .request<String>('getGenesisHash')
        .send()
        .timeout(const Duration(seconds: 8));
    if (genesis != SolanaConfig.genesisHashes[config.chain]) {
      throw const RuleBreach(Refusal.chainUnconfigured);
    }
  }

  Future<int> balance(String wallet) async {
    await checkCluster();
    final result = await rpc
        .getBalanceValue(
          Address(wallet),
          const GetBalanceConfig(commitment: Commitment.finalized),
        )
        .send()
        .timeout(const Duration(seconds: 8));
    return result.value.value.toInt();
  }

  /// Checks all roster assets at one finalized slot, including the program,
  /// collection, owner and immutable genome contract. No metadata URL is fetched.
  Future<Map<String, ReaperGenome>> owned(
    String wallet,
    List<String> assets,
  ) async {
    final collection = config.collection;
    if (collection == null) throw const RuleBreach(Refusal.chainUnconfigured);
    if (assets.isEmpty) return {};

    await checkCluster();
    final response = await rpc
        .getMultipleAccountsValue(
          [for (final asset in assets) Address(asset)],
          const GetMultipleAccountsConfig(
            encoding: AccountEncoding.base64,
            commitment: Commitment.finalized,
          ),
        )
        .send()
        .timeout(const Duration(seconds: 8));
    if (response.value.length != assets.length) {
      throw const FormatException('Invalid account response');
    }

    final owned = <String, ReaperGenome>{};
    for (var index = 0; index < assets.length; index++) {
      final account = response.value[index];
      if (account == null ||
          account['owner'] != core.mplCoreProgramAddress ||
          account['executable'] != false) {
        continue;
      }
      final data = account['data'];
      if (data is! List ||
          data.length != 2 ||
          data[1] != 'base64' ||
          data[0] is! String) {
        throw const FormatException('Invalid Core account data');
      }
      final bytes = base64Decode(data[0] as String);
      // Bound untrusted decoding work before the generated decoder reads strings.
      if (bytes.length > 32768 ||
          bytes.isEmpty ||
          bytes.first != core.Key.assetV1.index) {
        continue;
      }
      final asset = core.getAssetV1Decoder().read(bytes, 0).$1;
      if (asset.owner.value != wallet ||
          asset.updateAuthority !=
              core.UpdateAuthorityCollection(Address(collection))) {
        continue;
      }
      final genome = genomeFromUri(asset.uri);
      if (genome != null) owned[assets[index]] = genome;
    }
    return owned;
  }

  /// Collection minting must use this URI convention. The trusted collection
  /// binds the URI; first import freezes that genome in the canonical ledger.
  ReaperGenome? genomeFromUri(String value) {
    final prefix = config.identityUri.resolve('reapers/').toString();
    if (!value.startsWith(prefix) || !value.endsWith('.json')) return null;
    try {
      return ReaperGenome.parse(
        value.substring(prefix.length, value.length - 5),
      );
    } on FormatException {
      return null;
    }
  }
}
