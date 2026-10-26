import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/hunt_screen.dart';
import 'package:reapers_flutter/hunt/hunt_status.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../support/ledger_app.dart';

const _brief = HuntBrief(
  nft: Nft(collection: NftCollection.peskyPenguins, serial: 1),
  seed: 7,
  loadout: HuntLoadout.borrowed,
);

/// The hunt's real ledger at a short phone size with enlarged type.
Future<void> _show(WidgetTester tester, Widget child) async {
  tester.view
    ..physicalSize = const Size(360, 640)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);

  await tester.pumpWidget(
    MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: Veil.theme(Veil.unbound),
      home: MediaQuery(
        data: const MediaQueryData(
          size: Size(360, 640),
          textScaler: TextScaler.linear(1.3),
          disableAnimations: true,
        ),
        child: Scaffold(body: child),
      ),
    ),
  );
  await tester.pump();
}

void main() {
  testWidgets('a hunter sees recovery, hit feedback and motion advice', (
    tester,
  ) async {
    final l10n = await englishStrings();
    final sim = _brief.newSim(HuntPlan.open, Difficulty.normal);

    Widget status({double speed = 0, HuntBanner? banner}) => HuntStatus(
      title: _brief.nft.title,
      sim: sim,
      light: Veil.unbound,
      followsDevice: true,
      banner: banner,
      turnSpeed: speed,
    );

    await _show(tester, status(speed: HuntRules.startleSpeed + 0.1));
    expect(find.text(l10n.huntPromptTooFast), findsOneWidget);

    sim.step(
      HuntInput.measured(
        aim: sim.pose.direction,
        turnRadiansPerSecond: 0,
        taps: [sim.pose.direction],
      ),
    );
    await _show(tester, status());
    expect(find.text(l10n.huntBladeRecovering.toUpperCase()), findsOneWidget);

    for (var tick = 0; tick < sim.stats.swingTicks; tick++) {
      sim.step(const HuntInput(aim: Aim.ahead, turnSpeed: 0));
    }
    final missed = huntBannerFor(l10n, const StrikeWhiffed());
    await _show(
      tester,
      status(banner: missed),
    );
    expect(find.text(l10n.huntBladeReady.toUpperCase()), findsOneWidget);
    expect(find.text(l10n.huntStrikeMissed), findsOneWidget);

    sim.phase = HuntPhase.shriek;
    await _show(
      tester,
      status(speed: sim.stats.flinchSpeed + 0.1, banner: missed),
    );
    expect(find.text(l10n.huntMotionUnsteady.toUpperCase()), findsOneWidget);
    expect(find.text(l10n.huntPromptHold), findsOneWidget);
    expect(find.text(l10n.huntBladeReady.toUpperCase()), findsNothing);
    expect(find.text(l10n.huntStrikeMissed), findsNothing);

    sim.phase = HuntPhase.broken;
    await _show(tester, status(speed: HuntRules.reapSpeedLimit + 0.1));
    expect(find.text(l10n.huntPromptReapTooFast), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('briefing can scroll to begin with enlarged type', (
    tester,
  ) async {
    final l10n = await englishStrings();
    await _show(
      tester,
      HuntBriefing(
        brief: _brief,
        ready: true,
        difficulty: Difficulty.normal,
        onDifficulty: () {},
        onBegin: () => Navigator.of(
          tester.element(find.byType(HuntBriefing)),
        ).push(MaterialPageRoute<void>(builder: (_) => const Placeholder())),
      ),
    );

    await tester.scrollUntilVisible(find.byKey(const Key('hunt-begin')), 200);
    await tester.pump();
    expect(find.text(l10n.huntBegin.toUpperCase()), findsOneWidget);
    expect(find.byKey(const Key('hunt-begin')).hitTestable(), findsOneWidget);
    await tester.tap(find.byKey(const Key('hunt-begin')));
    await tester.pumpAndSettle();

    expect(find.byType(Placeholder), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('practice results offer another hunt and remain scrollable', (
    tester,
  ) async {
    final l10n = await englishStrings();
    await _show(
      tester,
      HuntReckoning(
        brief: _brief,
        reward: const HuntReward.practice(
          HuntOutcome.escaped(
            escape: EscapeCause.resolve,
            ambushed: false,
            ticks: 600,
          ),
        ),
        onPracticeAgain: () => Navigator.of(
          tester.element(find.byType(HuntReckoning)),
        ).push(MaterialPageRoute<void>(builder: (_) => const Placeholder())),
      ),
    );

    await tester.scrollUntilVisible(
      find.byKey(const Key('hunt-practice-again')),
      200,
    );
    await tester.pump();
    expect(find.text(l10n.huntPracticeAgain.toUpperCase()), findsOneWidget);
    expect(find.text(l10n.huntPracticeCadence.toUpperCase()), findsOneWidget);
    await tester.tap(find.byKey(const Key('hunt-practice-again')));
    await tester.pumpAndSettle();

    expect(find.byType(Placeholder), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('a full absorption reward can scroll to the exit', (
    tester,
  ) async {
    await _show(
      tester,
      const HuntReckoning(
        brief: HuntBrief(
          nft: Nft(collection: NftCollection.peskyPenguins, serial: 1),
          seed: 7,
          loadout: HuntLoadout.borrowed,
          sightingId: 1,
        ),
        reward: HuntReward(
          outcome: HuntOutcome.absorbed(
            grade: ReapGrade.perfect,
            ambushed: true,
            ticks: 1200,
            deeds: HuntDeeds(holds: 2, parries: 1, maxCadence: 4),
          ),
          xp: 100,
          anima: 40,
          feeding: Feeding.feast,
          signed: false,
          levelBefore: 1,
          levelAfter: 2,
          art: Art.rime,
          artRankBefore: 0,
          artRankAfter: 1,
        ),
      ),
    );

    await tester.scrollUntilVisible(find.byKey(const Key('hunt-leave')), 200);
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('hunt-leave')).hitTestable(), findsOneWidget);
    expect(find.byKey(const Key('hunt-practice-again')), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
