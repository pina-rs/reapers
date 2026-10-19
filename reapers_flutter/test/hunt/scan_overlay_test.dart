import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/scan/depth/occlusion_map.dart';
import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/scan/room_survey.dart';
import 'package:reapers_flutter/hunt/scan/scan_overlay.dart';
import 'package:reapers_flutter/hunt/scan/sigil_painter.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// A survey that sees: it says whatever it is told to.
final class _SeeingSurvey implements RoomSurvey {
  ({double centre, double halfWidth})? askedFor;

  @override
  Object? failure;

  @override
  OcclusionMap? get occlusion => null;

  @override
  HuntPlan planWithin(double centre, double halfWidth) {
    askedFor = (centre: centre, halfWidth: halfWidth);

    return HuntPlan.between(
      centre - halfWidth,
      centre + halfWidth,
      eyeCentimetres: 120,
    );
  }

  @override
  double read(int index) => 1;

  @override
  double? reach(int index) => 3;

  @override
  bool tight(int index) => index.isEven;

  @override
  void dispose() {}
}

SigilTrace _trace({Rarity rarity = Rarity.uncommon, double centre = 0.3}) =>
    SigilTrace(
      Sigil.draw(seed: 11, shape: rarity.sigil, gait: Gait.ground),
      centre: centre,
    );

/// Holds the view on [trace]'s next light until it catches.
void _catchNext(SigilTrace trace) {
  final caught = trace.caught;

  while (trace.caught == caught) {
    trace.advance(
      trace.directionOf(caught),
      turnSpeed: 0,
      deltaSeconds: 1 / 60,
    );
  }
}

void main() {
  /// Shows the overlay for [trace] with the phone pointing at [pose].
  Future<AppLocalizations> pump(
    WidgetTester tester,
    SigilTrace trace,
    ValueNotifier<int> frame, {
    ArPose Function()? pose,
    bool followsDevice = true,
    RoomSurvey? survey,
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
            trace: trace,
            pose: pose ?? () => ArPose.ahead,
            followsDevice: followsDevice,
            light: Veil.unbound,
            repaint: frame,
            survey: survey,
            floorDepth: floorDepth,
            onSkip: onSkip,
          ),
        ),
      ),
    );

    return AppLocalizations.delegate.load(const Locale('en'));
  }

  String status(WidgetTester tester) =>
      tester.widget<Text>(find.byKey(const Key('sigil-status'))).data!;

  testWidgets('it says something is near and how to draw it out', (
    tester,
  ) async {
    final l10n = await pump(tester, _trace(), ValueNotifier(0));

    expect(find.text(l10n.scanTitle), findsOneWidget);
    expect(find.text(l10n.sigilHow), findsOneWidget);
    expect(find.byKey(const Key('sigil-glyph')), findsOneWidget);
  });

  testWidgets('with no motion sensors it explains dragging instead', (
    tester,
  ) async {
    final l10n = await pump(
      tester,
      _trace(),
      ValueNotifier(0),
      followsDevice: false,
    );

    expect(find.text(l10n.sigilHowDrag), findsOneWidget);
  });

  testWidgets('it counts the lights as they catch', (tester) async {
    final frame = ValueNotifier(0);
    final trace = _trace();
    final l10n = await pump(tester, trace, frame);

    expect(status(tester), l10n.sigilCaught(0, 4));

    _catchNext(trace);
    _catchNext(trace);
    frame.value++;
    await tester.pump();

    expect(status(tester), l10n.sigilCaught(2, 4));
  });

  testWidgets('it says to hold still when the ring is on a light but moving', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    final trace = _trace();
    var turnSpeed = 0.0;
    final l10n = await pump(
      tester,
      trace,
      frame,
      pose: () => ArPose.looking(trace.directionOf(0), turnSpeed: turnSpeed),
    );

    expect(status(tester), l10n.sigilCaught(0, 4));

    turnSpeed = HuntRules.sigilSteadySpeed + 0.2;
    frame.value++;
    await tester.pump();

    expect(status(tester), l10n.sigilSteady);
  });

  testWidgets('it warns when a light is about to go out', (tester) async {
    final frame = ValueNotifier(0);
    final trace = _trace(rarity: Rarity.rare);
    final l10n = await pump(tester, trace, frame);

    _catchNext(trace);
    // Looking straight up, at nothing, for most of the time it waits.
    trace.advance(
      Vec3.up,
      turnSpeed: 0,
      deltaSeconds: trace.sigil.fraySeconds! * 0.6,
    );
    frame.value++;
    await tester.pump();

    expect(status(tester), l10n.sigilSlipping);
  });

  testWidgets('a survey that sees says how far down it found the floor', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    double? floor;
    final l10n = await pump(
      tester,
      _trace(),
      frame,
      survey: _SeeingSurvey(),
      floorDepth: () => floor,
    );

    expect(find.byKey(const Key('scan-floor')), findsNothing);

    floor = 1.318;
    frame.value++;
    await tester.pump();

    expect(find.text(l10n.scanFloorFound('1.32')), findsOneWidget);
  });

  testWidgets('the way on without the sigil is offered only when given', (
    tester,
  ) async {
    final frame = ValueNotifier(0);
    var skipped = 0;
    final l10n = await pump(tester, _trace(), frame);

    expect(find.byKey(const Key('scan-skip')), findsNothing);

    await pump(tester, _trace(), frame, onSkip: () => skipped++);
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
                trace: _trace(),
                pose: () => ArPose.ahead,
                followsDevice: false,
                light: Veil.unbound,
                repaint: frame,
                survey: _SeeingSurvey(),
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

  group('the plan a sigil leaves', () {
    test('without a survey, it is the arc the sigil read and nothing more', () {
      final trace = _trace(centre: 0.5);
      final plan = sigilPlan(trace, null);

      expect(plan.isSurveyed, isFalse);
      expect(plan.eyeCentimetres, HuntRules.eyeCentimetres);
      expect(plan.centre, closeTo(0.5, 1e-3));
      expect(plan.halfWidth, closeTo(trace.sigil.halfArc, 1e-3));
      expect(plan.isWellFormed, isTrue);
    });

    test('with one, the survey is asked about that arc', () {
      final trace = _trace(centre: -1.2);
      final survey = _SeeingSurvey();
      final plan = sigilPlan(trace, survey);

      expect(survey.askedFor, (centre: -1.2, halfWidth: trace.sigil.halfArc));
      expect(plan.eyeCentimetres, 120);
    });
  });

  group('drawing the sigil', () {
    /// Paints [painter] on a phone-sized canvas, which is all a painter
    /// can be asked to do without a screen to look at.
    void paint(CustomPainter painter) {
      final recorder = ui.PictureRecorder();

      painter.paint(Canvas(recorder), const Size(402, 874));
      recorder.endRecording().dispose();
    }

    test('it can be drawn from anywhere, with any of it caught', () {
      final trace = _trace(rarity: Rarity.epic);
      final frame = ValueNotifier(0);
      final views = [
        ArPose.ahead,
        ArPose.looking(trace.directionOf(0)),
        // The sigil behind the player, and straight down at the floor.
        ArPose.looking(Vec3.fromAngles(trace.centre + 3.1, -0.4)),
        ArPose.looking(const Vec3(0, -1, 0.001)),
      ];

      for (var caught = 0; caught <= trace.sigil.lights.length; caught++) {
        for (final view in views) {
          paint(
            SigilPainter(
              trace: trace,
              pose: () => view,
              verticalFov: () => radians(72),
              light: Veil.unbound,
              repaint: frame,
            ),
          );
          paint(
            SigilGlyphPainter(
              trace: trace,
              pose: () => view,
              light: Veil.unbound,
              repaint: frame,
            ),
          );
        }

        if (!trace.isClosed) {
          _catchNext(trace);
        }
      }

      expect(trace.isClosed, isTrue);
    });
  });
}
