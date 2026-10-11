import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/l10n/soul_lookups.dart';
import 'package:reapers_flutter/render/reaper_appearance.dart';
import 'package:reapers_flutter/solana/reaper_wallet.dart';
import 'package:reapers_flutter/studio/reaper_studio.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  test(
    'wallet payload must contain the exact challenge before its signature',
    () {
      const message = 'Sign in to Reapers. No payment.';
      final signature = Uint8List.fromList(List.generate(64, (index) => index));
      expect(
        walletMessageSignature(
          message,
          base64Encode([...utf8.encode(message), ...signature]),
        ),
        base64Encode(signature),
      );
      expect(
        () => walletMessageSignature(
          message,
          base64Encode([...utf8.encode('Different message'), ...signature]),
        ),
        throwsFormatException,
      );
      expect(
        () =>
            walletMessageSignature(message, base64Encode(utf8.encode(message))),
        throwsFormatException,
      );
    },
  );

  for (final size in [
    const Size(390, 844),
    const Size(1180, 820),
    const Size(320, 568),
  ]) {
    testWidgets(
      'customization and progression work at $size without changing a save',
      (tester) async {
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final saved = PlayerState.initial().takeOath(seed: 11);
        final before = jsonEncode(saved.toJson());
        ReaperGenome? specimen;
        Weapon? equipment;
        ReaperAppearance? presentation;
        await tester.pumpWidget(
          MaterialApp(
            theme: Veil.theme(Veil.unbound),
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: ReaperStudio(
              initialGenome: saved.genome,
              stageBuilder: (genome, weapon, outfit, appearance) {
                specimen = genome;
                equipment = weapon;
                presentation = appearance;
                return const SizedBox.expand();
              },
            ),
          ),
        );
        await tester.ensureVisible(find.text('Chooser'));
        await tester.tap(find.text('Chooser'));
        await tester.pumpAndSettle();
        expect(specimen!.reaperClass, ReaperClass.chooser);
        await tester.ensureVisible(find.text('Humanoid'));
        await tester.tap(find.text('Humanoid'));
        await tester.pumpAndSettle();
        expect(presentation!.form, ReaperForm.humanoid);
        expect(presentation!.clothing, ReaperClothing.tunic);
        await tester.ensureVisible(find.text('No robe'));
        await tester.tap(find.text('No robe'));
        await tester.pumpAndSettle();
        expect(presentation!.clothing, ReaperClothing.unrobed);
        final held = find.byType(DropdownButtonFormField<WeaponKind>);
        await tester.ensureVisible(held);
        await tester.tap(held);
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.text('Scythe').last);
        await tester.tap(find.text('Scythe').last);
        await tester.pumpAndSettle();
        expect(equipment!.kind, WeaponKind.scythe);
        await tester.ensureVisible(find.text('Hold an object'));
        await tester.tap(find.text('Hold an object'));
        await tester.pumpAndSettle();
        expect(presentation!.holding, isFalse);
        await tester.ensureVisible(
          find.bySemanticsLabel('Head / face covering 2'),
        );
        await tester.tap(find.bySemanticsLabel('Head / face covering 2'));
        await tester.pumpAndSettle();
        expect(specimen!.head, 1);
        await tester.ensureVisible(find.text('Growth'));
        await tester.tap(find.text('Growth'));
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.byKey(const Key('studio-absorb')));
        await tester.tap(find.byKey(const Key('studio-absorb')));
        await tester.pumpAndSettle();
        expect(find.textContaining('XP in this preview'), findsOneWidget);
        expect(find.text('1 absorbed'), findsOneWidget);
        await tester.ensureVisible(find.text('SEE FULL POTENTIAL'));
        await tester.tap(find.text('SEE FULL POTENTIAL'));
        await tester.pumpAndSettle();
        expect(equipment!.temper, Temper.hollowed);
        expect(find.text('30 / 30'), findsOneWidget);
        await tester.ensureVisible(find.text('Soul'));
        await tester.tap(find.text('Soul'));
        await tester.pumpAndSettle();
        await tester.ensureVisible(
          find.byKey(const Key('studio-gift-reprisal')),
        );
        await tester.tap(find.byKey(const Key('studio-gift-reprisal')));
        await tester.pumpAndSettle();
        expect(ReaperSoul.fromGenome(specimen!).gift, ReaperGift.reprisal);
        expect(find.text('Awakening 3 / 3'), findsOneWidget);
        await tester.ensureVisible(find.text('Clear specimen memories'));
        await tester.tap(find.text('Clear specimen memories'));
        await tester.pumpAndSettle();
        expect(find.text('Awakening 0 / 3'), findsOneWidget);
        final l10n = await AppLocalizations.delegate.load(const Locale('en'));
        expect(
          find.text(giftEffect(l10n, ReaperGift.reprisal, 1)),
          findsOneWidget,
        );
        await tester.ensureVisible(
          find.byKey(const Key('studio-memory-stillness')),
        );
        await tester.tap(find.byKey(const Key('studio-memory-stillness')));
        await tester.pumpAndSettle();
        expect(find.text('Awakening 0 / 3'), findsOneWidget);
        expect(jsonEncode(saved.toJson()), before);
        expect(tester.takeException(), isNull);
      },
    );
  }

  testWidgets('the soul remains readable on a narrow phone with large text', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(320, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    await tester.pumpWidget(
      MaterialApp(
        theme: Veil.theme(Veil.unbound),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        builder: (context, child) => MediaQuery(
          data: MediaQuery.of(context)
              .copyWith(textScaler: const TextScaler.linear(2)),
          child: child!,
        ),
        home: ReaperStudio(
          stageBuilder: (_, _, _, _) => const SizedBox.expand(),
        ),
      ),
    );
    final soulTab = find.descendant(
      of: find.byType(SegmentedButton<int>),
      matching: find.text('Soul'),
    );
    await tester.ensureVisible(soulTab);
    await tester.tap(soulTab);
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Clear specimen memories'));
    expect(tester.takeException(), isNull);
  });

  testWidgets('inspect expression pauses a reaction and frames the face', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: Veil.theme(Veil.unbound),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: ReaperStudio(
          stageBuilder: (_, _, _, _) => const SizedBox.expand(),
        ),
      ),
    );
    await tester.tap(find.text('Animations · 32 states'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Reactions'));
    await tester.tap(find.text('Reactions'));
    await tester.pumpAndSettle();
    final selector = find.byType(DropdownButtonFormField<ReaperMotion>);
    await tester.ensureVisible(selector);
    await tester.tap(selector);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Laugh').last);
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Inspect expression'));
    await tester.tap(find.text('Inspect expression'));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Full figure'), findsOneWidget);
    expect(find.byTooltip('Play animation'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
