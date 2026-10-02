import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/screens/colophon_screen.dart';
import 'package:reapers_flutter/sound/score.dart';
import 'package:reapers_flutter/sound/score_engine.dart';
import 'package:reapers_flutter/sound/soundtrack.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:reapers_flutter/theme/veil.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/ledger_app.dart';

/// An engine that writes down what it is told.
final class _Listening implements ScoreEngine {
  final told = <String>[];

  @override
  Future<void> play(Score piece, Mix mix, {required Duration fade}) async =>
      told.add('play ${piece.name}');

  @override
  void shape(Mix mix) {}

  @override
  Future<void> silence({required Duration fade}) async => told.add('silence');

  @override
  void hold({required bool held}) {}

  @override
  void dispose() {}
}

void main() {
  Future<(_Listening, ProviderContainer)> pump(
    WidgetTester tester, {
    Map<String, Object> stored = const {},
  }) async {
    SharedPreferences.setMockInitialValues(stored);

    final engine = _Listening();
    final container = ProviderContainer(
      overrides: [
        sharedPreferencesProvider.overrideWithValue(
          await SharedPreferences.getInstance(),
        ),
        scoreEngineProvider.overrideWithValue(engine),
      ],
    );
    addTearDown(container.dispose);

    tester.view
      ..physicalSize = const Size(402, 874)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          theme: Veil.theme(Veil.unbound),
          home: const ColophonScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    return (engine, container);
  }

  testWidgets('everyone whose work is in the game is named', (tester) async {
    final l10n = await englishStrings();

    await pump(tester);

    // The composer's own wording, which is the condition of using the music.
    expect(find.text(l10n.colophonAnabasisCredit), findsOneWidget);
    expect(find.text(l10n.colophonKatabasisCredit), findsOneWidget);
    expect(l10n.colophonAnabasisCredit, contains('Scott Buckley'));
    expect(l10n.colophonAnabasisCredit, contains('CC-BY 4.0'));
    expect(find.text(l10n.colophonBachCredit), findsOneWidget);

    await tester.scrollUntilVisible(find.text(l10n.colophonTypeCredit), 200);

    expect(find.text(l10n.colophonDepthCredit), findsOneWidget);
  });

  testWidgets('the music can be turned off and on, and stays that way', (
    tester,
  ) async {
    final l10n = await englishStrings();
    final (engine, container) = await pump(tester);

    container.read(soundtrackProvider).play(Score.sanctum);

    expect(find.text(l10n.colophonMusicOn), findsOneWidget);

    await tester.tap(find.byKey(const Key('colophon-music')));
    await tester.pumpAndSettle();

    expect(find.text(l10n.colophonMusicOff), findsOneWidget);
    expect(engine.told, ['play sanctum', 'silence']);
    expect(
      (await SharedPreferences.getInstance()).getBool('reapers.music'),
      isFalse,
    );

    await tester.tap(find.byKey(const Key('colophon-music')));
    await tester.pumpAndSettle();

    expect(engine.told, ['play sanctum', 'silence', 'play sanctum']);
  });

  testWidgets('a player who turned it off comes back to silence', (
    tester,
  ) async {
    final l10n = await englishStrings();
    final (engine, container) = await pump(
      tester,
      stored: {'reapers.music': false},
    );

    container.read(soundtrackProvider).play(Score.sanctum);

    expect(find.text(l10n.colophonMusicOff), findsOneWidget);
    expect(engine.told, isEmpty);
  });
}
