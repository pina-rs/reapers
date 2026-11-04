import 'dart:convert';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/lookups.dart';
import 'package:reapers_flutter/screens/codex_screen.dart';
import 'package:reapers_flutter/screens/rifts_screen.dart';
import 'package:reapers_flutter/screens/vault_screen.dart';
import 'package:reapers_flutter/src/backend/local_backend.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/ledger_app.dart';

const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

void main() {
  late SharedPreferences preferences;
  late LocalBackend backend;
  late DateTime now;

  /// Releases [_saint], lets it surface, and absorbs it with the shared bot.
  Future<void> absorbSaint() async {
    await backend.acquire(_saint);

    final state = await backend.release(_saint);
    final sighting = state.sightings.single;
    final bot = HuntBot(
      HuntSim(
        seed: sighting.seed,
        vessel: sighting.nft.collection.vessel,
        ward: sighting.nft.rarity.ward,
        stats: HuntStats.derive(state.loadout),
      ),
    )..play();

    now = now.add(const Duration(minutes: 1));
    await backend.settleHunt(sighting.id, HuntLog(bot.recorded));
  }

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    now = DateTime.utc(2026, 10, 1, 12);
    preferences = await SharedPreferences.getInstance();
    backend = LocalBackend(
      preferences,
      clock: () => now,
      random: Random(5),
    );
    await backend.takeOath();
  });

  group('vault', () {
    testWidgets('buying moves an NFT into the vault and the wallet pays', (
      tester,
    ) async {
      final l10n = await englishStrings();

      await pumpScreen(tester, const VaultScreen(), backend: backend);

      expect(find.text(l10n.vaultStandIn), findsOneWidget);
      expect(
        tester.widget<Text>(find.byKey(const Key('vault-wallet'))).data,
        l10n.commonSol('3.00'),
      );

      await tester.tap(find.byKey(Key('vault-nft-${_saint.id}')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('vault-buy')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('vault-release')), findsOneWidget);
      expect(
        tester.widget<Text>(find.byKey(const Key('vault-wallet'))).data,
        l10n.commonSol('2.98'),
      );
      expect((await backend.load()).vault, [_saint]);
    });

    testWidgets('releasing sets the husk loose and returns to the start', (
      tester,
    ) async {
      await backend.acquire(_saint);
      await pumpScreen(tester, const VaultScreen(), backend: backend);

      await tester.tap(find.byKey(Key('vault-nft-${_saint.id}')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('vault-release')));
      await tester.pumpAndSettle();

      final state = await backend.load();

      expect(state.vault, isEmpty);
      expect(state.sightings.single.nft, _saint);
      expect(find.byKey(const Key('vault-release')), findsNothing);
    });

    testWidgets('a purchase the wallet cannot cover is refused and explained', (
      tester,
    ) async {
      final l10n = await englishStrings();

      // A player whose wallet has been emptied.
      await preferences.setString(
        LocalBackend.defaultStorageKey,
        jsonEncode({...(await backend.load()).toJson(), 'lamports': 1000}),
      );

      await pumpScreen(tester, const VaultScreen(), backend: backend);
      await tester.tap(find.byKey(Key('vault-nft-${_saint.id}')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('vault-buy')));
      await tester.pumpAndSettle();

      expect(
        find.text(refusalMessage(l10n, Refusal.insufficientFunds)),
        findsOneWidget,
      );

      await tester.tap(find.byKey(const Key('refusal-dismiss')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('vault-buy')), findsOneWidget);
    });
  });

  group('codex', () {
    testWidgets('an empty codex says so', (tester) async {
      final l10n = await englishStrings();

      await pumpScreen(tester, const CodexScreen(), backend: backend);

      expect(find.text(l10n.codexEmpty), findsOneWidget);
    });

    testWidgets('an absorbed husk is listed under its collection', (
      tester,
    ) async {
      final l10n = await englishStrings();

      await absorbSaint();
      await pumpScreen(tester, const CodexScreen(), backend: backend);

      expect(find.text(l10n.codexEmpty), findsNothing);
      expect(find.text(_saint.title), findsOneWidget);
      expect(
        find.text(l10n.codexSetProgress(NftCollection.setSize - 1)),
        findsOneWidget,
      );
    });
  });

  group('rifts', () {
    testWidgets('every rift is shut to a reaper who has absorbed nothing', (
      tester,
    ) async {
      final l10n = await englishStrings();

      await pumpScreen(tester, const RiftsScreen(), backend: backend);

      expect(find.text(l10n.riftsStandIn), findsOneWidget);

      for (final site in RiftSite.values) {
        await tester.scrollUntilVisible(
          find.byKey(Key('rift-shut-${site.name}')),
          200,
        );

        expect(find.text(site.title), findsOneWidget);
        expect(find.byKey(Key('rift-enter-${site.name}')), findsNothing);
      }
    });

    testWidgets('absorbing one of a collection opens its rift, and only its', (
      tester,
    ) async {
      await absorbSaint();
      await pumpScreen(tester, const RiftsScreen(), backend: backend);

      expect(find.byKey(const Key('rift-enter-londonBridge')), findsOneWidget);
      expect(find.byKey(const Key('rift-shut-londonBridge')), findsNothing);
      expect(find.byKey(const Key('rift-enter-shibuyaCrossing')), findsNothing);
    });

    testWidgets('a demo build says so and opens every rift unattuned', (
      tester,
    ) async {
      final l10n = await englishStrings();

      await pumpScreen(
        tester,
        const RiftsScreen(),
        backend: backend,
        demo: true,
      );

      expect(find.byKey(const Key('rifts-demo-note')), findsOneWidget);

      await tester.scrollUntilVisible(
        find.byKey(Key('rift-enter-${RiftSite.values.last.name}')),
        200,
      );

      expect(
        find.text(l10n.riftsDemoOpen.toUpperCase()),
        findsWidgets,
        reason: 'the gate is waived, so the rifts offer to open',
      );
    });
  });

  test('every refusal has something to say for itself', () async {
    final l10n = await englishStrings();
    final messages = {
      for (final refusal in Refusal.values) refusalMessage(l10n, refusal),
    };

    expect(messages, hasLength(Refusal.values.length));
    expect(messages, everyElement(isNotEmpty));
  });
}
