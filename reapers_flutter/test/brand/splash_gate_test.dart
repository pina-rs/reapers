import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/brand/splash_gate.dart';
import 'package:reapers_flutter/brand/reapers_mark.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/theme/veil.dart';

void main() {
  Future<void> show(
    WidgetTester tester, {
    bool reduced = false,
    bool ready = true,
  }) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: Veil.theme(Veil.unbound),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: MediaQuery(
          data: MediaQueryData(disableAnimations: reduced),
          child: SplashGate(
            ready: ready,
            child: const Scaffold(body: Text('Ready', key: Key('ready'))),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.runAsync(
      () => precacheImage(
        ReapersMark.splashImage,
        tester.element(find.byType(SplashGate)),
      ),
    );
    await tester.pump();
  }

  testWidgets(
    'the animated arrival hides game semantics and releases the game',
    (tester) async {
      await show(tester);
      expect(find.byType(IgnorePointer), findsWidgets);
      expect(
        tester
            .widget<ExcludeSemantics>(find.byType(ExcludeSemantics).first)
            .excluding,
        isTrue,
      );
      await tester.pump(const Duration(milliseconds: 1900));
      await tester.pump(const Duration(milliseconds: 250));
      await tester.pump();
      expect(find.text('Reapers'), findsNothing);
      expect(find.byKey(const Key('ready')), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('reduced motion hands off after a short still', (tester) async {
    await show(tester, reduced: true);
    await tester.pump(const Duration(milliseconds: 300));
    await tester.pump();
    expect(find.text('Reapers'), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('slow startup exposes loading after four seconds', (
    tester,
  ) async {
    await show(tester, ready: false);
    await tester.pump(const Duration(seconds: 4));
    await tester.pump(const Duration(milliseconds: 250));
    await tester.pump();
    expect(find.text('Reapers'), findsNothing);
    expect(find.byKey(const Key('ready')), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('removing splash early leaves no timers or animation callbacks', (
    tester,
  ) async {
    await show(tester);
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(seconds: 5));
    expect(tester.takeException(), isNull);
  });
}
