import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:integration_test/integration_test.dart';
import 'package:reapers_flutter/app.dart';
import 'package:reapers_flutter/screens/oath_screen.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a fresh arrival reaches the branded threshold and the oath', (
    tester,
  ) async {
    // A volatile save gives this native journey its own player without
    // touching the install's existing books. Composition stays production.
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          sharedPreferencesProvider.overrideWithValue(preferences),
        ],
        child: const ReapersApp(),
      ),
    );

    final oath = find.byKey(const Key('threshold-oath')).hitTestable();
    for (var frame = 0; frame < 24 && oath.evaluate().isEmpty; frame++) {
      await tester.pump(const Duration(milliseconds: 250));
    }
    expect(find.byKey(const Key('threshold-title')), findsOneWidget);
    expect(oath, findsOneWidget);
    // The threshold has its own finite entrance after the splash. Its
    // background drifts forever, so pumpAndSettle cannot settle this page.
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 1800));
    await binding.takeScreenshot('phone-ios');
    await tester.tap(oath);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byType(OathScreen), findsOneWidget);
    expect(find.byKey(const Key('oath-speak')).hitTestable(), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
