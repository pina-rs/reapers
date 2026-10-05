import 'dart:ui' show SemanticsAction, Tristate;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/widgets/ledger.dart';

void main() {
  testWidgets('ledger actions support keyboard and semantic activation', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    try {
      var activations = 0;
      Widget button({required bool enabled}) => MaterialApp(
        home: Scaffold(
          body: LedgerButton(
            label: 'Action',
            onPressed: enabled ? () => activations++ : null,
          ),
        ),
      );
      await tester.pumpWidget(button(enabled: true));
      expect(
        tester.getSemantics(find.byType(LedgerButton)),
        matchesSemantics(
          label: 'Action',
          isButton: true,
          isFocusable: true,
          hasEnabledState: true,
          isEnabled: true,
          hasTapAction: true,
        ),
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      expect(
        tester
            .getSemantics(find.byType(LedgerButton))
            .flagsCollection
            .isFocused,
        Tristate.isTrue,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(activations, 1);
      await tester.sendKeyEvent(LogicalKeyboardKey.space);
      await tester.pump();
      expect(activations, 2);
      await tester.pumpWidget(button(enabled: false));
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(activations, 2);
      expect(
        tester
            .getSemantics(find.byType(LedgerButton))
            .getSemanticsData()
            .hasAction(SemanticsAction.tap),
        isFalse,
      );
    } finally {
      semantics.dispose();
    }
  });
}
