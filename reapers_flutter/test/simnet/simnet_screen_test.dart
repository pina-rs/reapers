import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:http/http.dart' as http;
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/simnet/simnet_backend.dart';
import 'package:reapers_flutter/simnet/simnet_chain.dart';
import 'package:reapers_flutter/simnet/simnet_screen.dart';
import 'package:reapers_flutter/simnet/simnet_state.dart';
import 'package:reapers_flutter/simnet/simnet_toasts.dart';
import 'package:reapers_flutter/simnet/surfpool_rpc.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/ledger_app.dart';

const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

/// Answers the panel's JSON-RPC without a network, counting the calls a
/// test wants to have happened.
final class _StubRpc implements http.Client {
  int balances = 0;
  int airdrops = 0;

  @override
  Future<http.Response> post(
    Uri url, {
    Map<String, String>? headers,
    Object? body,
    Encoding? encoding,
  }) async {
    final request = (jsonDecode(body! as String) as Map)
        .cast<String, dynamic>();
    final method = request['method'] as String;
    Object? result;

    switch (method) {
      case 'getHealth':
        result = 'ok';
      case 'getGenesisHash':
        result = 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG';
      case 'getBalance':
        balances++;
        result = {
          'context': {'slot': 7},
          'value': 2_500_000_000,
        };
      case 'requestAirdrop':
        airdrops++;
        result = 'airdrop-signature-1';
      case 'getSignatureStatuses':
        result = {
          'context': {'slot': 9},
          'value': [
            {'slot': 9, 'err': null, 'confirmationStatus': 'finalized'},
          ],
        };
      default:
        throw UnsupportedError('Unexpected RPC method: $method');
    }

    return http.Response(
      jsonEncode({'jsonrpc': '2.0', 'id': request['id'], 'result': result}),
      HttpStatus.ok,
    );
  }

  @override
  void close() {}

  @override
  Object noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'Unexpected HTTP call: ${invocation.memberName}',
  );
}

void main() {
  late SharedPreferences preferences;
  late SimnetChain chain;
  late SimnetBackend backend;

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    preferences = await SharedPreferences.getInstance();
    chain = SimnetChain.open(
      preferences,
      random: Random(3),
      latency: Duration.zero,
    );
    backend = SimnetBackend(preferences, chain, random: Random(5));
    await backend.takeOath();
  });

  Future<void> pump(WidgetTester tester, {SurfpoolRpc? local}) async {
    tester.view
      ..physicalSize = const Size(402, 874)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          sharedPreferencesProvider.overrideWithValue(preferences),
          simnetChainProvider.overrideWithValue(chain),
          gameBackendProvider.overrideWithValue(backend),
          if (local != null)
            surfpoolSetupProvider.overrideWithValue(
              (rpc: local, problem: null),
            ),
        ],
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          theme: Veil.theme(Veil.unbound),
          builder: (context, child) => SimnetToasts(child: child!),
          home: Consumer(
            builder: (context, ref, _) => ref.watch(playerProvider).hasValue
                ? const SimnetScreen()
                : const SizedBox.shrink(),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('the ledger shows the wallet and each transaction, and a slip '
      'pins each new one as it is confirmed', (tester) async {
    final l10n = await englishStrings();

    await pump(tester);

    expect(find.text(chain.wallet), findsOneWidget);
    expect(find.text(l10n.simnetInstructionAirdrop.toUpperCase()), findsOne);

    await tester.runAsync(() => backend.acquire(_saint));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.byKey(const Key('simnet-slip')), findsOneWidget);
    expect(
      find.ancestor(
        of: find.byKey(const Key('simnet-slip')),
        matching: find.byType(Material),
      ),
      findsOneWidget,
      reason: 'above the navigator, the slip brings its own Material',
    );
    expect(find.textContaining(l10n.simnetConfirmed), findsOneWidget);

    await tester.pump(SimnetToasts.finality);

    expect(find.textContaining(l10n.simnetFinalized), findsOneWidget);

    await tester.pump(SimnetToasts.shown);
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('simnet-slip')), findsNothing);
    expect(find.text(_saint.title), findsOneWidget, reason: 'in the ledger');

    await tester.tap(
      find.byKey(Key('simnet-tx-${chain.transactions.last.signature}')),
    );
    await tester.pumpAndSettle();

    expect(find.text(chain.transactions.last.signature), findsOneWidget);
    expect(find.text(_saint.mockMint), findsOneWidget);
  });

  testWidgets('the surfpool panel checks the validator and airdrops once', (
    tester,
  ) async {
    final l10n = await englishStrings();
    final stub = _StubRpc();
    final rpc = SurfpoolRpc(
      endpoint: Uri.parse('http://localhost:8899'),
      client: stub,
    );

    await pump(tester, local: rpc);

    expect(find.byKey(const Key('simnet-local-rpc')), findsOneWidget);
    expect(
      find.text('EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG'),
      findsOneWidget,
      reason: 'the genesis hash is on the record',
    );
    expect(stub.balances, 1);

    await tester.ensureVisible(
      find.byKey(const Key('simnet-local-rpc-airdrop')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('simnet-local-rpc-airdrop')));
    await tester.pumpAndSettle();

    expect(stub.airdrops, 1);
    expect(find.text('airdrop-signature-1'), findsOneWidget);
    expect(
      find.text(l10n.simnetLocalRpcAirdropped.toUpperCase()),
      findsOneWidget,
      reason: 'the action is spent',
    );
    expect(stub.balances, 2, reason: 'the balance is read again after paying');
  });

  testWidgets('a pending airdrop can resume from the panel', (tester) async {
    final stub = _StubRpc();
    await preferences.setString(
      SurfpoolFaucet.storageKey,
      jsonEncode({
        'wallets': {
          chain.wallet: {
            'genesis': 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG',
            'signature': 'pending-signature',
            'slot': null,
            'at': '2026-11-04T00:00:00Z',
            'confirmed': false,
          },
        },
      }),
    );
    await pump(
      tester,
      local: SurfpoolRpc(
        endpoint: Uri.parse('http://localhost:8899'),
        client: stub,
      ),
    );
    final action = find.byKey(const Key('simnet-local-rpc-airdrop'));
    await tester.ensureVisible(action);
    await tester.pumpAndSettle();
    await tester.tap(action);
    await tester.pumpAndSettle();

    final l10n = await englishStrings();
    expect(
      find.text(l10n.simnetLocalRpcAirdropped.toUpperCase()),
      findsOneWidget,
    );
    expect(stub.airdrops, 0);
    expect(find.text('pending-signature'), findsOneWidget);
  });

  testWidgets('a reset wallet starts the panel over', (tester) async {
    final stub = _StubRpc();
    await pump(
      tester,
      local: SurfpoolRpc(
        endpoint: Uri.parse('http://localhost:8899'),
        client: stub,
      ),
    );

    await tester.ensureVisible(
      find.byKey(const Key('simnet-local-rpc-airdrop')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('simnet-local-rpc-airdrop')));
    await tester.pumpAndSettle();
    expect(find.text('airdrop-signature-1'), findsOneWidget);

    // Start over, as the ledger's reset does: the same chain object, a new
    // burner wallet. A posted transaction then rebuilds the explorer in
    // place, which is where a stale panel would show the old wallet's paid
    // receipt to a wallet that was never paid.
    await tester.runAsync(() async {
      await chain.reset();
      await chain.submit(
        SimnetInstruction.airdrop,
        lamports: 1,
        accounts: [
          (role: SimnetRole.faucet, address: SimnetChain.faucet),
          (role: SimnetRole.wallet, address: chain.wallet),
        ],
      );
    });
    await tester.pumpAndSettle();

    expect(find.text(chain.wallet), findsOneWidget);
    expect(
      find.text('airdrop-signature-1'),
      findsNothing,
      reason: 'the old receipt does not belong to the new wallet',
    );
    expect(stub.balances, 3, reason: 'the panel checks the new wallet too');
  });
}
