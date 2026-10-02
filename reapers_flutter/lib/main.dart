import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'app.dart';
import 'sound/soloud_engine.dart';
import 'sound/soundtrack.dart';
import 'src/state/player.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // The hunt reads the phone's motion in portrait, so the game never turns.
  await SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);

  final preferences = await SharedPreferences.getInstance();

  runApp(
    ProviderScope(
      overrides: [
        sharedPreferencesProvider.overrideWithValue(preferences),
        scoreEngineProvider.overrideWithValue(SoLoudEngine()),
      ],
      child: const ReapersApp(),
    ),
  );
}
