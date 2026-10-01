import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/scan/room_reader.dart';
import 'package:reapers_flutter/hunt/scan/scan_overlay.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_flutter/widgets/ledger.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  /// Shows the overlay for [reader] with the phone pointing at [pose].
  Future<AppLocalizations> pump(
    WidgetTester tester,
    RoomReader reader,
    ValueNotifier<int> frame, {
    ArPose pose = ArPose.ahead,
    bool followsDevice = true,
  }) async {
    tester.view
      ..physicalSize = const Size(402, 874)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        theme: Veil.theme(Veil.unbound),
        home: Scaffold(
          body: ScanOverlay(
            reader: reader,
            pose: () => pose,
            followsDevice: followsDevice,
            repaint: frame,
          ),
        ),
      ),
    );

    return AppLocalizations.delegate.load(const Locale('en'));
  }

  String advice(WidgetTester tester) =>
      tester.widget<Text>(find.byKey(const Key('scan-advice'))).data!;

  testWidgets('it says something is near and what to do about a walker', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    final l10n = await pump(tester, RoomReader(gait: Gait.ground), frame);

    expect(find.text(l10n.scanTitle), findsOneWidget);
    expect(find.text(l10n.scanFloor), findsOneWidget);
    expect(find.text(l10n.scanAir), findsNothing);
  });

  testWidgets('with no motion sensors it explains dragging instead', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    final l10n = await pump(
      tester,
      RoomReader(gait: Gait.air),
      frame,
      followsDevice: false,
    );

    expect(find.text(l10n.scanAirDrag), findsOneWidget);
  });

  testWidgets('its advice and its meter follow what the reader has seen', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    final reader = RoomReader(gait: Gait.ground);
    final l10n = await pump(tester, reader, frame);

    double read() =>
        tester.widget<LedgerMeter>(find.byType(LedgerMeter)).fraction;

    expect(read(), 0);

    // Looking level at a husk that walks.
    reader.observe(ArPose.looking(Vec3.forward), 1 / 60);
    frame.value++;
    await tester.pump();

    expect(advice(tester), l10n.scanLower);

    // Turning too fast.
    reader.observe(
      ArPose.looking(Vec3.fromAngles(0, radians(-25)), turnSpeed: 4),
      1 / 60,
    );
    frame.value++;
    await tester.pump();

    expect(advice(tester), l10n.scanSlower);

    // Looking steadily at the floor.
    for (var tick = 0; tick < 60; tick++) {
      reader.observe(
        ArPose.looking(Vec3.fromAngles(0, radians(-25)), turnSpeed: 0.1),
        1 / 60,
      );
    }

    frame.value++;
    await tester.pump();

    expect(advice(tester), l10n.scanReading);
    expect(read(), greaterThan(0));
    expect(read(), lessThan(1));
  });
}
