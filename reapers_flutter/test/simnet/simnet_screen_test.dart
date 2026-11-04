import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/simnet/simnet_backend.dart';
import 'package:reapers_flutter/simnet/simnet_chain.dart';
import 'package:reapers_flutter/simnet/simnet_screen.dart';
import 'package:reapers_flutter/simnet/simnet_state.dart';
import 'package:reapers_flutter/simnet/simnet_toasts.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/ledger_app.dart';

const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

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

  Future<void> pump(WidgetTester tester) async {
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
}
