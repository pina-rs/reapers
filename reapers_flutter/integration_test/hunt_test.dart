import 'package:camera/camera.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:integration_test/integration_test.dart';
import 'package:reapers_flutter/app.dart';
import 'package:reapers_flutter/hunt/hunt_arena.dart';
import 'package:reapers_flutter/hunt/hunt_screen.dart';
import 'package:reapers_flutter/hunt/hunt_status.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Waits for a visible state while the real camera, GPU and simulation run.
Future<void> _waitFor(WidgetTester tester, Finder finder) async {
  await tester.pump();
  final deadline = DateTime.now().add(const Duration(seconds: 100));

  while (finder.evaluate().length != 1 && DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 100));
  }

  expect(finder, findsOneWidget);
  expect(tester.takeException(), isNull);
}

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('two practice hunts, a rematch and camera reopening work', (
    tester,
  ) async {
    expect(
      HuntArena.autoplay,
      isTrue,
      reason: 'Run this journey with --dart-define=REAPERS_AUTOPLAY=true.',
    );
    await SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);
    // Keep the device's existing player untouched.
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [sharedPreferencesProvider.overrideWithValue(preferences)],
        child: const ReapersApp(),
      ),
    );

    final practice = find.byKey(const Key('threshold-shade')).hitTestable();
    final result = find.byKey(const Key('hunt-result'));

    // Reopen the camera once, and repeat once without reopening it.
    for (var visit = 0; visit < 2; visit++) {
      await _waitFor(tester, practice);
      final l10n = AppLocalizations.of(tester.element(practice));
      final begin = find.text(l10n.huntBegin.toUpperCase()).hitTestable();
      await tester.tap(practice);
      await _waitFor(tester, begin);
      await _waitFor(tester, find.byType(CameraPreview));
      if (visit == 0) {
        await binding.convertFlutterSurfaceToImage();
        await tester.pump();
        await binding.takeScreenshot('seeker-briefing');
      }
      await tester.tap(begin);
      await _waitFor(tester, find.byType(HuntStatus));
      if (visit == 0) {
        await _waitFor(
          tester,
          find.text(l10n.huntBladeRecovering.toUpperCase()),
        );
        await binding.takeScreenshot('seeker-hunt');
      }
      await _waitFor(tester, result);
      expect(find.text(l10n.huntEscapedTitle), findsNothing);

      if (visit == 0) {
        await binding.takeScreenshot('seeker-reaped');
        await tester.tap(find.byKey(const Key('hunt-practice-again')));
        await _waitFor(tester, find.byType(HuntStatus));
        expect(find.byType(HuntReckoning), findsNothing);
        await _waitFor(tester, result);
        expect(find.text(l10n.huntEscapedTitle), findsNothing);
      }

      await tester.tap(find.byKey(const Key('hunt-leave')));
      await _waitFor(tester, practice);
      expect(tester.takeException(), isNull);
    }
  });
}
