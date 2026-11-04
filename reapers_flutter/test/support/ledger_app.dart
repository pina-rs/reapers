import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_flutter/l10n/l10n.dart';
import 'package:reapers_flutter/src/backend/game_backend.dart';
import 'package:reapers_flutter/src/state/player.dart';
import 'package:reapers_flutter/theme/veil.dart';

/// The strings a test compares against: the same ones the screens show.
Future<AppLocalizations> englishStrings() =>
    AppLocalizations.delegate.load(const Locale('en'));

/// Shows [screen] as the game would, with [backend] keeping the books.
///
/// The screen is shown at a phone's size, and only once the player has been
/// read, since every screen behind the threshold takes the player as given.
/// With [demo], the build acts as `--dart-define=REAPERS_DEMO_RIFT=true`.
Future<void> pumpScreen(
  WidgetTester tester,
  Widget screen, {
  required GameBackend backend,
  bool demo = false,
}) async {
  tester.view
    ..physicalSize = const Size(402, 874)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);

  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        gameBackendProvider.overrideWithValue(backend),
        if (demo) demoRiftProvider.overrideWithValue(true),
      ],
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        theme: Veil.theme(Veil.unbound),
        home: Consumer(
          builder: (context, ref, _) => ref.watch(playerProvider).hasValue
              ? screen
              : const SizedBox.shrink(),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}
