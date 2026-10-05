import 'dart:convert';
import 'dart:typed_data';

import 'package:reapers_server/src/solana/solana_config.dart';
import 'package:reapers_server/src/solana/solana_gateway.dart';
import 'package:reapers_server/src/solana/wallet_proof.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:solana_kit/solana_kit.dart' as kit;
import 'package:solana_kit_mpl_core/solana_kit_mpl_core.dart' as core;
import 'package:test/test.dart';

void main() {
  const address = '11111111111111111111111111111111';
  final signer = kit.generateKeyPairSigner();
  final other = kit.generateKeyPairSigner();
  tearDownAll(() {
    signer.keyPair.dispose();
    other.keyPair.dispose();
  });
  final genome = ReaperGenome.shuffle(11);
  final config = SolanaConfig(
    chain: 'solana:devnet',
    rpcUrl: 'https://api.devnet.solana.com',
    identityUri: Uri.parse('https://reapers.example/'),
    collection: address,
  );

  test('Ed25519 proof binds the wallet and exact message, including weak-key rejection', () {
    final message = Uint8List.fromList(utf8.encode('Reapers nonce 123'));
    final signature = base64Encode(
      kit.signBytes(signer.keyPair.privateKey, message).value,
    );
    expect(
      verifyWalletProof(signer.address.value, 'Reapers nonce 123', signature),
      isTrue,
    );
    expect(
      verifyWalletProof(other.address.value, 'Reapers nonce 123', signature),
      isFalse,
    );
    expect(
      verifyWalletProof(signer.address.value, 'Reapers nonce 124', signature),
      isFalse,
    );
    expect(
      verifyWalletProof(signer.address.value, 'Reapers nonce 123', 'bad!'),
      isFalse,
    );
    expect(
      verifyWalletProof(address, 'anything', base64Encode(Uint8List(64))),
      isFalse,
    );
  });

  test('Kit decodes only owned Core assets in the verified collection at a finalized slot', () async {
    final values = [
      core.AssetV1(
        key: core.Key.assetV1,
        owner: signer.address,
        updateAuthority: const core.UpdateAuthorityCollection(
          kit.Address(address),
        ),
        name: genome.name,
        uri: 'https://reapers.example/reapers/${genome.code}.json',
        seq: null,
      ),
      core.AssetV1(
        key: core.Key.assetV1,
        owner: other.address,
        updateAuthority: const core.UpdateAuthorityCollection(
          kit.Address(address),
        ),
        name: genome.name,
        uri: 'https://reapers.example/reapers/${genome.code}.json',
        seq: null,
      ),
      core.AssetV1(
        key: core.Key.assetV1,
        owner: signer.address,
        updateAuthority: core.UpdateAuthorityAddress(signer.address),
        name: genome.name,
        uri: 'https://reapers.example/reapers/${genome.code}.json',
        seq: null,
      ),
    ];
    final addresses = [
      for (var index = 0; index < values.length + 1; index++)
        kit
            .getAddressDecoder()
            .decode(Uint8List.fromList(List.filled(32, index + 8)))
            .value,
    ];
    final rpc = kit.createSolanaRpcFromTransport((request) async {
      final payload = request.payload! as Map<String, Object?>;
      final Object? result;
      if (payload['method'] == 'getGenesisHash') {
        result = SolanaConfig.genesisHashes[config.chain];
      } else {
        expect(payload['method'], 'getMultipleAccounts');
        final params = payload['params'] as List;
        // Kit removes finalized, the RPC node default. Confirmed must not
        // replace this ownership read, even though it is Kit's usual default.
        expect(params.last, {'encoding': 'base64'});
        expect(params.first, addresses);
        result = {
          'context': {'slot': 9},
          'value': [
            for (final asset in values)
              {
                'owner': core.mplCoreProgramAddress,
                'executable': false,
                'data': [
                  base64Encode(core.getAssetV1Encoder().encode(asset)),
                  'base64',
                ],
              },
            null,
          ],
        };
      }
      return {'jsonrpc': '2.0', 'id': '1', 'result': result};
    });
    expect(
      await SolanaGateway(config, rpc).owned(signer.address.value, addresses),
      {addresses.first: genome},
    );
  });

  test('wrong cluster refuses the operation before reading assets', () async {
    final rpc = kit.createSolanaRpcFromTransport(
      (request) async => {
        'jsonrpc': '2.0',
        'id': '1',
        'result': SolanaConfig.genesisHashes['solana:mainnet'],
      },
    );
    expect(
      () => SolanaGateway(config, rpc).owned(signer.address.value, [address]),
      throwsA(
        isA<RuleBreach>().having(
          (error) => error.refusal,
          'refusal',
          Refusal.chainUnconfigured,
        ),
      ),
    );
  });

  test(
    'genome URIs must use the configured origin and a valid permanent code',
    () {
      final gateway = SolanaGateway(
        config,
        kit.createSolanaRpcFromTransport((_) async => null),
      );
      expect(
        gateway.genomeFromUri(
          'https://reapers.example/reapers/${genome.code}.json',
        ),
        genome,
      );
      for (final uri in [
        'https://evil.example/reapers/${genome.code}.json',
        'https://reapers.example/reapers/../../${genome.code}.json',
        'https://reapers.example/reapers/R1-invalid.json',
      ]) {
        expect(gateway.genomeFromUri(uri), isNull);
      }
    },
  );
}
