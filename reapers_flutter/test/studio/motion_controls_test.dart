import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/render/reaper_playback.dart';
import 'package:reapers_flutter/studio/motion_controls.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';

void main() {
  for (final reduced in [false, true]) {
    testWidgets('select, scrub and replay with reduced motion $reduced', (
      tester,
    ) async {
      final clock = ReaperPlayback();
      addTearDown(clock.dispose);
      tester.view.physicalSize = const Size(390, 1200);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        MaterialApp(
          theme: Veil.theme(Veil.unbound),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: MediaQuery(
            data: MediaQueryData(disableAnimations: reduced),
            child: Scaffold(
              body: SingleChildScrollView(
                child: MotionControls(
                  playback: clock,
                  weapon: WeaponKind.spear,
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Animations · 32 states'));
      await tester.pumpAndSettle();
      for (final (label, motion) in [
        ('Dash left', ReaperMotion.dashLeft),
        ('Dash right', ReaperMotion.dashRight),
      ]) {
        await tester.tap(find.byType(DropdownButtonFormField<ReaperMotion>));
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.text(label).last);
        await tester.tap(find.text(label).last);
        await tester.pumpAndSettle();
        expect(clock.motion, motion);
        expect(clock.playing, !reduced);
      }
      await tester.tap(find.text('Finishers'));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButtonFormField<ReaperMotion>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Finisher · Impale').last);
      await tester.pumpAndSettle();
      expect(clock.motion, ReaperMotion.finisherImpale);
      expect(clock.playing, !reduced);
      await tester.tap(find.text('Inspect contact pose'));
      await tester.pumpAndSettle();
      expect(
        clock.progress,
        closeTo(ReaperMotion.finisherImpale.impact!, 1e-9),
      );
      expect(clock.playing, isFalse);
      await tester.tap(find.byTooltip('Replay'));
      await tester.pumpAndSettle();
      expect(clock.seconds, 0);
      expect(clock.playing, !reduced);
      expect(tester.takeException(), isNull);
    });
  }
}
