import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:reapers_flutter/main_preview.dart' as preview;

void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('read evolution, exercise chance and distinguish the NFT plan', (
    tester,
  ) async {
    preview.main();
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Read the design decisions'));
    await tester.pumpAndSettle();
    expect(find.text('The Book of Becoming'), findsOneWidget);

    if (defaultTargetPlatform == TargetPlatform.android) {
      await binding.convertFlutterSurfaceToImage();
      await tester.pump();
    }
    await binding.takeScreenshot('decisions-${defaultTargetPlatform.name}');
    await tester.ensureVisible(find.text('Ferryman').first);
    await tester.tap(find.text('Ferryman').first);
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.byKey(const Key('ledger-gift-anchor')));
    await tester.tap(find.byKey(const Key('ledger-gift-anchor')));
    await tester.pumpAndSettle();
    final level = find.byKey(const Key('ledger-level'));
    await tester.ensureVisible(level);
    await tester.tapAt(tester.getTopRight(level) + const Offset(-10, 24));
    await tester.pumpAndSettle();
    expect(find.text('Awakening 0 / 3'), findsOneWidget);
    await tester.ensureVisible(find.byKey(const Key('ledger-marks-3')));
    await tester.tap(find.byKey(const Key('ledger-marks-3')));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Awakening 1 / 3'));
    expect(find.text('Awakening 1 / 3'), findsOneWidget);
    await binding.takeScreenshot('evolution-${defaultTargetPlatform.name}');

    await tester.ensureVisible(find.byKey(const Key('ledger-chapter-1')));
    await tester.tap(find.byKey(const Key('ledger-chapter-1')));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.byKey(const Key('ledger-draw-birth')));
    await tester.tap(find.byKey(const Key('ledger-draw-birth')));
    await tester.pumpAndSettle();
    expect(
      find.textContaining(
        RegExp(' · (Harvester|Ferryman|Weigher|Chooser|Hollow|Mourner)\\n'),
      ),
      findsOneWidget,
    );
    await tester.ensureVisible(find.text('Rare'));
    await tester.tap(find.text('Rare'));
    await tester.pumpAndSettle();
    expect(find.text('25% · ×0.85 XP · 128 XP'), findsOneWidget);
    expect(find.text('5% · ×2.0 XP · 300 XP'), findsOneWidget);
    await tester.ensureVisible(find.text('FEEDING ODDS'));
    await binding.takeScreenshot('chance-${defaultTargetPlatform.name}');
    await tester.ensureVisible(find.byKey(const Key('ledger-roll-feeding')));
    await tester.tap(find.byKey(const Key('ledger-roll-feeding')));
    await tester.pumpAndSettle();
    expect(find.textContaining(' XP · 60 anima'), findsOneWidget);

    await tester.ensureVisible(find.byKey(const Key('ledger-chapter-2')));
    await tester.tap(find.byKey(const Key('ledger-chapter-2')));
    await tester.pumpAndSettle();
    expect(find.text('Next: bring an owned husk'), findsOneWidget);
    expect(
      find.textContaining(
        'Real husk purchases, escrow and burning are not implemented.',
      ),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
    await binding.takeScreenshot('nft-plan-${defaultTargetPlatform.name}');
  });
}
