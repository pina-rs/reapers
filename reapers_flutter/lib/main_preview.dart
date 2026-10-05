import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';

import 'l10n/l10n.dart';
import 'studio/decision_ledger.dart';
import 'studio/reaper_studio.dart';
import 'theme/veil.dart';

/// Standalone preview: no preferences, player, server or wallet are opened.
void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const ReaperPreviewApp());
}

class ReaperPreviewApp extends HookWidget {
  const ReaperPreviewApp({super.key});
  @override
  Widget build(BuildContext context) {
    useEffect(() {
      final semantics = kIsWeb
          ? WidgetsBinding.instance.ensureSemantics()
          : null;
      return semantics?.dispose;
    }, const []);

    return MaterialApp(
      onGenerateTitle: (context) => context.l10n.studioAppTitle,
      debugShowCheckedModeBanner: false,
      theme: Veil.theme(Veil.unbound),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: const ReaperStudio(),
      routes: {
        DecisionLedger.route: (context) => const DecisionLedger(),
      },
    );
  }
}
