import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/scan/room_reader.dart';
import 'package:reapers_flutter/hunt/scan/room_survey.dart';
import 'package:reapers_flutter/hunt/scan/scan_overlay.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_flutter/widgets/ledger.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// A survey that sees: it says whatever it is told to.
final class _SeeingSurvey implements RoomSurvey {
  @override
  ReadingAdvice advice = ReadingAdvice.keepGoing;

  @override
  double progress = 0;

  @override
  Object? failure;

  @override
  HuntPlan? plan;

  @override
  HuntPlan get planUnread => HuntPlan.open;

  @override
  void observe(ArPose pose, double deltaSeconds) {}

  @override
  double read(int index) => 1;

  @override
  double? reach(int index) => 3;

  @override
  bool blocked(int index) => index.isEven;

  @override
  void dispose() {}
}

void main() {
  /// Shows the overlay for [reader] with the phone pointing at [pose].
  Future<AppLocalizations> pump(
    WidgetTester tester,
    RoomSurvey reader,
    ValueNotifier<int> frame, {
    Gait gait = Gait.ground,
    ArPose pose = ArPose.ahead,
    bool followsDevice = true,
    double? Function()? floorDepth,
    VoidCallback? onSkip,
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
            survey: reader,
            gait: gait,
            pose: () => pose,
            followsDevice: followsDevice,
            repaint: frame,
            floorDepth: floorDepth,
            onSkip: onSkip,
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
      gait: Gait.air,
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

  testWidgets('a survey that sees says how far down it found the floor', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    double? floor;
    final l10n = await pump(
      tester,
      _SeeingSurvey(),
      frame,
      floorDepth: () => floor,
    );

    expect(find.byKey(const Key('scan-floor')), findsNothing);

    floor = 1.318;
    frame.value++;
    await tester.pump();

    expect(find.text(l10n.scanFloorFound('1.32')), findsOneWidget);
  });

  testWidgets('with no room for the husk it says to turn somewhere else', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    final survey = _SeeingSurvey()..advice = ReadingAdvice.findRoom;
    final l10n = await pump(tester, survey, frame);

    expect(advice(tester), l10n.scanFindRoom);
  });

  testWidgets('the way on without the room is offered only when given', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    var skipped = 0;
    final l10n = await pump(tester, _SeeingSurvey(), frame);

    expect(find.byKey(const Key('scan-skip')), findsNothing);

    await pump(tester, _SeeingSurvey(), frame, onSkip: () => skipped++);
    await tester.tap(find.byKey(const Key('scan-skip')));

    expect(find.bySemanticsLabel(l10n.scanSkip), findsOneWidget);
    expect(skipped, 1);
  });

  testWidgets('everything but the way on lets a touch through', (tester) async {
    final frame = ValueNotifier(0);
    var touched = 0;

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
          body: Stack(
            fit: StackFit.expand,
            children: [
              GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => touched++,
              ),
              ScanOverlay(
                survey: _SeeingSurvey(),
                gait: Gait.ground,
                pose: () => ArPose.ahead,
                followsDevice: false,
                repaint: frame,
                onSkip: () {},
              ),
            ],
          ),
        ),
      ),
    );

    await tester.tapAt(const Offset(200, 300));

    expect(touched, 1);
  });
}
