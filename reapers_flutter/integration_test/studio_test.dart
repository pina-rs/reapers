import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_scene/scene.dart';
import 'package:integration_test/integration_test.dart';
import 'package:reapers_flutter/main_preview.dart' as preview;

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'a specimen keeps its birth gift while earned memory awakens it',
    (
      tester,
    ) async {
      preview.main();
      await tester.pump();
      for (
        var frame = 0;
        frame < 40 && find.byType(SceneView).evaluate().isEmpty;
        frame++
      ) {
        await tester.pump(const Duration(milliseconds: 200));
      }
      expect(find.byType(SceneView), findsOneWidget);
      if (defaultTargetPlatform == TargetPlatform.android) {
        await binding.convertFlutterSurfaceToImage();
        await tester.pump();
      }
      await binding.takeScreenshot('studio-${defaultTargetPlatform.name}');
      await tester.ensureVisible(find.text('Chooser'));
      await tester.tap(find.text('Chooser'));
      await tester.pump();
      final soulTab = find.descendant(
        of: find.byType(SegmentedButton<int>),
        matching: find.text('Soul'),
      );
      await tester.ensureVisible(soulTab);
      await tester.tap(soulTab);
      await tester.pump();
      await tester.ensureVisible(find.byKey(const Key('studio-gift-reprisal')));
      await tester.tap(find.byKey(const Key('studio-gift-reprisal')));
      await tester.pump();
      final level = find.byKey(const Key('studio-soul-level'));
      await tester.ensureVisible(level);
      await tester.tapAt(tester.getTopRight(level) + const Offset(-10, 24));
      await tester.pump();
      expect(find.text('Awakening 0 / 3'), findsOneWidget);

      final remember = find.byKey(const Key('studio-memory-stillness'));
      for (var hunt = 0; hunt < 3; hunt++) {
        await tester.ensureVisible(remember);
        await tester.tap(remember);
        await tester.pump();
      }
      await tester.ensureVisible(find.text('Awakening 1 / 3'));
      expect(find.text('Awakening 1 / 3'), findsOneWidget);
      expect(find.text('Reprisal'), findsNWidgets(2));
      expect(tester.takeException(), isNull);
      // Give the harness's pointer markers time to disappear before evidence.
      for (var frame = 0; frame < 10; frame++) {
        await tester.pump(const Duration(milliseconds: 100));
      }
      await binding.takeScreenshot('soul-${defaultTargetPlatform.name}');
      await tester.ensureVisible(find.text('Reprisal').last);
      for (var frame = 0; frame < 10; frame++) {
        await tester.pump(const Duration(milliseconds: 100));
      }
      await binding.takeScreenshot('gift-${defaultTargetPlatform.name}');
    },
  );
}
