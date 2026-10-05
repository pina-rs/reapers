import 'dart:convert';
import 'dart:typed_data';

import 'package:reapers_server/src/generated/protocol.dart';
import 'package:reapers_server/src/player/player_books.dart';
import 'package:reapers_server/src/solana/solana_config.dart';
import 'package:reapers_server/src/solana/solana_endpoint.dart';
import 'package:reapers_server/src/solana/solana_gateway.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart' show UuidValue;
import 'package:serverpod_auth_idp_server/core.dart' as auth;
import 'package:serverpod_auth_idp_server/providers/anonymous.dart';
import 'package:solana_kit/solana_kit.dart' as kit;
import 'package:solana_kit_mpl_core/solana_kit_mpl_core.dart' as core;
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

Matcher refused(Refusal value) => throwsA(
  isA<Refused>().having((error) => error.refusal, 'refusal', value.name),
);

void main() {
  const firstId = '550e8400-e29b-41d4-a716-446655440000';
  const secondId = '6ba7b810-9dad-11d1-80b4-00c04fd430c8';
  const collection = '11111111111111111111111111111111';
  final firstWallet = kit.generateKeyPairSigner();
  final secondWallet = kit.generateKeyPairSigner();
  final asset = kit
      .getAddressDecoder()
      .decode(Uint8List.fromList(List.filled(32, 9)))
      .value;
  final genome = ReaperGenome.shuffle(11);
  final config = SolanaConfig(
    chain: 'solana:devnet',
    rpcUrl: 'https://api.devnet.solana.com',
    identityUri: Uri.parse('https://reapers.example/'),
    collection: collection,
  );
  var owner = firstWallet.address;
  var failChain = false;

  String sign(WalletChallenge proof, kit.KeyPairSigner wallet) => base64Encode(
    kit
        .signBytes(
          wallet.keyPair.privateKey,
          Uint8List.fromList(utf8.encode(proof.message)),
        )
        .value,
  );

  withServerpod('Solana ownership and growth', (sessionBuilder, endpoints) {
    final first = sessionBuilder.copyWith(
      authentication: AuthenticationOverride.authenticationInfo(
        firstId,
        const {},
      ),
    );
    final second = sessionBuilder.copyWith(
      authentication: AuthenticationOverride.authenticationInfo(
        secondId,
        const {},
      ),
    );
    var initialized = false;
    late SolanaGateway original;

    setUp(() async {
      final session = sessionBuilder.build();
      if (!initialized) {
        session.serverpod.initializeAuthServices(
          tokenManagerBuilders: [auth.JwtConfigFromPasswords()],
          identityProviderBuilders: [const AnonymousIdpConfig()],
        );
        initialized = true;
      }
      for (final id in [firstId, secondId]) {
        await auth.AuthUser.db.insertRow(
          session,
          auth.AuthUser(id: UuidValue.fromString(id), scopeNames: {}),
        );
      }
      original = SolanaEndpoint.gateway;
      owner = firstWallet.address;
      failChain = false;
      SolanaEndpoint.gateway = SolanaGateway(
        config,
        kit.createSolanaRpcFromTransport((request) async {
          if (failChain) throw const FormatException('Unavailable fixture');
          final payload = request.payload! as Map<String, Object?>;
          final result = switch (payload['method']) {
            'getGenesisHash' => SolanaConfig.genesisHashes[config.chain],
            'getBalance' => {
              'context': {'slot': 10},
              'value': 2000000000,
            },
            'getMultipleAccounts' => {
              'context': {'slot': 10},
              'value': [
                {
                  'owner': core.mplCoreProgramAddress,
                  'executable': false,
                  'data': [
                    base64Encode(
                      core.getAssetV1Encoder().encode(
                        core.AssetV1(
                          key: core.Key.assetV1,
                          owner: owner,
                          updateAuthority: const core.UpdateAuthorityCollection(
                            kit.Address(collection),
                          ),
                          name: genome.name,
                          uri:
                              'https://reapers.example/reapers/${genome.code}.json',
                          seq: null,
                        ),
                      ),
                    ),
                    'base64',
                  ],
                },
              ],
            },
            _ => throw StateError(
              'Unexpected chain method ${payload['method']}',
            ),
          };
          return {'jsonrpc': '2.0', 'id': '1', 'result': result};
        }),
      );
    });
    tearDown(() => SolanaEndpoint.gateway = original);

    test('wallet challenge is signed, expires and cannot be replayed or used by another guest', () async {
      final proof = await endpoints.solana.challenge(
        first,
        firstWallet.address.value,
      );
      expect(proof.message, contains('Chain ID: solana:devnet'));
      expect(proof.message, contains('does not authorize a payment'));
      await expectLater(
        endpoints.solana.login(
          second,
          proof.challengeId,
          sign(proof, firstWallet),
        ),
        refused(Refusal.walletUnproven),
      );
      await expectLater(
        endpoints.solana.login(
          first,
          proof.challengeId,
          sign(proof, secondWallet),
        ),
        refused(Refusal.walletUnproven),
      );
      final login = await endpoints.solana.login(
        first,
        proof.challengeId,
        sign(proof, firstWallet),
      );
      expect(login.authUserId, UuidValue.fromString(firstId));
      await expectLater(
        endpoints.solana.login(
          first,
          proof.challengeId,
          sign(proof, firstWallet),
        ),
        refused(Refusal.walletUnproven),
      );
      final next = await endpoints.solana.challenge(
        first,
        firstWallet.address.value,
      );
      await WalletChallenge.db.updateRow(
        sessionBuilder.build(),
        next.copyWith(expiresAt: DateTime.utc(2020)),
      );
      await expectLater(
        endpoints.solana.login(
          first,
          next.challengeId,
          sign(next, firstWallet),
        ),
        refused(Refusal.walletUnproven),
      );
    });

    test(
      'a wallet restores its original player instead of merging guest progress',
      () async {
        final proof = await endpoints.solana.challenge(
          first,
          firstWallet.address.value,
        );
        await endpoints.solana.login(
          first,
          proof.challengeId,
          sign(proof, firstWallet),
        );
        await endpoints.player.takeOath(first);
        final restore = await endpoints.solana.challenge(
          second,
          firstWallet.address.value,
        );
        final login = await endpoints.solana.login(
          second,
          restore.challengeId,
          sign(restore, firstWallet),
        );
        expect(login.authUserId, UuidValue.fromString(firstId));
        expect((await endpoints.player.load(second)).oathTaken, isFalse);
        final status = await endpoints.solana.status(first);
        expect(status.lamports, 2000000000);
        expect(status.wallet, firstWallet.address.value);
      },
    );

    test('verified NFT growth follows a transfer and the old owner loses use of it', () async {
      for (final pair in [(first, firstWallet), (second, secondWallet)]) {
        final proof = await endpoints.solana.challenge(
          pair.$1,
          pair.$2.address.value,
        );
        await endpoints.solana.login(
          pair.$1,
          proof.challengeId,
          sign(proof, pair.$2),
        );
        await endpoints.player.takeOath(pair.$1);
      }
      await endpoints.solana.importReaper(first, asset);
      await endpoints.player.selectReaper(first, asset);
      await PlayerBooks.change(
        first.build(),
        (state) => state.settleRift(xp: 700, anima: 70),
      );
      final before = await endpoints.player.load(first);
      expect(before.xp, 700);
      owner = secondWallet.address;
      final imported = await endpoints.solana.importReaper(second, asset);
      expect(imported.reapers.last.xp, 700);
      final selected = await endpoints.player.selectReaper(second, asset);
      expect(selected.genome, genome);
      expect(selected.xp, 700);
      expect(selected.anima, 0);
      await expectLater(
        endpoints.player.selectReaper(first, asset),
        refused(Refusal.notOwned),
      );
      expect((await endpoints.player.load(first)).reapers.length, 1);
      expect(
        (await ReaperRecord.db.find(sessionBuilder.build())).single.state.xp,
        700,
      );
    });

    test('unlinked wallets, wrong owners and chain failures cannot import or erase growth', () async {
      await endpoints.player.takeOath(first);
      await expectLater(
        endpoints.solana.importReaper(first, asset),
        refused(Refusal.walletUnproven),
      );
      final proof = await endpoints.solana.challenge(
        first,
        firstWallet.address.value,
      );
      await endpoints.solana.login(
        first,
        proof.challengeId,
        sign(proof, firstWallet),
      );
      owner = secondWallet.address;
      await expectLater(
        endpoints.solana.importReaper(first, asset),
        refused(Refusal.notOwned),
      );
      owner = firstWallet.address;
      await endpoints.solana.importReaper(first, asset);
      failChain = true;
      await expectLater(
        endpoints.player.load(first),
        refused(Refusal.chainUnavailable),
      );
      expect(
        (await PlayerRecord.db.find(sessionBuilder.build()))
            .single
            .state
            .reapers
            .length,
        2,
      );
      expect((await ReaperRecord.db.find(sessionBuilder.build())).length, 1);
    });
  });
  tearDownAll(() {
    firstWallet.keyPair.dispose();
    secondWallet.keyPair.dispose();
  });
}
