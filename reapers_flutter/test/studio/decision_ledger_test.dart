import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/studio/decision_ledger.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  for (final (width, scale) in [(1180.0, 1.0), (320.0, 2.0)]) {
    testWidgets(
      'the decision ledger explains earned growth and odds at $width/$scale',
      (
        tester,
      ) async {
        final semantics = tester.ensureSemantics();
        try {
          tester.view.physicalSize = Size(width, 844);
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
                    .copyWith(textScaler: TextScaler.linear(scale)),
                child: child!,
              ),
              home: DecisionLedger(random: Random(11)),
            ),
          );
          await tester.pumpAndSettle();
          await tester.ensureVisible(find.text('Ferryman').first);
          await tester.tap(find.text('Ferryman').first);
          await tester.pumpAndSettle();
          await tester.ensureVisible(
            find.byKey(const Key('ledger-gift-anchor')),
          );
          await tester.tap(find.byKey(const Key('ledger-gift-anchor')));
          await tester.pumpAndSettle();
          final level = find.byKey(const Key('ledger-level'));
          await tester.ensureVisible(level);
          await tester.tapAt(tester.getTopRight(level) + const Offset(-10, 24));
          await tester.pumpAndSettle();
          expect(find.text('Awakening 0 / 3'), findsOneWidget);
          await tester.ensureVisible(find.byKey(const Key('ledger-marks-30')));
          await tester.tap(find.byKey(const Key('ledger-marks-30')));
          await tester.pumpAndSettle();
          expect(find.text('Awakening 3 / 3'), findsOneWidget);
          await tester.ensureVisible(find.byKey(const Key('ledger-chapter-1')));
          await tester.tap(find.byKey(const Key('ledger-chapter-1')));
          await tester.pumpAndSettle();
          expect(find.text('25% · ×0.85 XP · 43 XP'), findsOneWidget);
          expect(
            find.text('5% · ×${Feeding.feast.share} XP · 100 XP'),
            findsOneWidget,
          );
          final l10n = AppLocalizations.of(
            tester.element(find.byType(DecisionLedger)),
          );
          await tester.ensureVisible(find.text(l10n.ledgerLuckBody));
          expect(
            find.bySemanticsLabel(l10n.ledgerLuckBody),
            findsOneWidget,
          );
          await tester.ensureVisible(
            find.byKey(const Key('ledger-draw-birth')),
          );
          await tester.tap(find.byKey(const Key('ledger-draw-birth')));
          await tester.pumpAndSettle();
          expect(
            find.textContaining(
              RegExp(
                ' · (Harvester|Ferryman|Weigher|Chooser|Hollow|Mourner)\\n',
              ),
            ),
            findsOneWidget,
          );
          await tester.ensureVisible(
            find.byKey(const Key('ledger-roll-feeding')),
          );
          await tester.tap(find.byKey(const Key('ledger-roll-feeding')));
          await tester.pumpAndSettle();
          expect(find.textContaining(' XP · 20 anima'), findsOneWidget);
          await tester.ensureVisible(find.text('Rare'));
          await tester.tap(find.text('Rare'));
          await tester.pumpAndSettle();
          expect(find.textContaining(' XP · 20 anima'), findsNothing);
          expect(find.text('25% · ×0.85 XP · 128 XP'), findsOneWidget);
          expect(
            find.text(
              'Expected XP for this scenario: 159.60. Anima: 60. '
              'Expectation uses each rounded reward and its weight.',
            ),
            findsOneWidget,
          );
          expect(tester.takeException(), isNull);
        } finally {
          semantics.dispose();
        }
      },
    );
  }
}
