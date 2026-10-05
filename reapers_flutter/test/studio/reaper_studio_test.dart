import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
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
        await tester.pumpWidget(
          MaterialApp(
            theme: Veil.theme(Veil.unbound),
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: ReaperStudio(
              initialGenome: saved.genome,
              stageBuilder: (genome, weapon, outfit) {
                specimen = genome;
                equipment = weapon;
                return const SizedBox.expand();
              },
            ),
          ),
        );
        await tester.ensureVisible(find.text('Chooser'));
        await tester.tap(find.text('Chooser'));
        await tester.pumpAndSettle();
        expect(specimen!.reaperClass, ReaperClass.chooser);
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
        expect(jsonEncode(saved.toJson()), before);
        expect(tester.takeException(), isNull);
      },
    );
  }
}
