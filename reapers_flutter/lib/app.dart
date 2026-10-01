import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'dev/reaper_gallery.dart';
import 'l10n/l10n.dart';
import 'render/reaper_look.dart';
import 'screens/sanctum_screen.dart';
import 'screens/threshold_screen.dart';
import 'src/state/player.dart';
import 'theme/veil.dart';
import 'widgets/ledger.dart';

/// The game. Which screen is home depends on one thing: whether the player
/// has been bound to a reaper yet.
class ReapersApp extends HookConsumerWidget {
  const ReapersApp({super.key});

  /// Whether this build opens at the developer's reaper bench. Only a debug
  /// build honours it.
  static const _gallery = kDebugMode && bool.fromEnvironment('REAPERS_GALLERY');

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final player = ref.watch(playerProvider);
    final bound = player.value?.oathTaken ?? false;

    return MaterialApp(
      onGenerateTitle: (context) => context.l10n.appTitle,
      debugShowCheckedModeBanner: false,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      // The whole interface is lit with the reaper's own light.
      theme: Veil.theme(
        bound ? reaperLight(player.requireValue.genome) : Veil.unbound,
      ),
      home: switch (player) {
        _ when _gallery => const ReaperGallery(),
        AsyncData(:final value) when value.oathTaken => SanctumScreen(
          player: value,
        ),
        AsyncData() => const ThresholdScreen(),
        AsyncError(:final error) => _Unreadable(
          reason: '$error',
          onRetry: () => ref.invalidate(playerProvider),
        ),
        AsyncLoading() => const Scaffold(body: SizedBox.expand()),
      },
    );
  }
}

/// Shown when the player could not be read at all: a damaged save, or a
/// keeper that cannot be reached.
class _Unreadable extends HookConsumerWidget {
  const _Unreadable({required this.reason, required this.onRetry});

  final String reason;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context, WidgetRef ref) => Scaffold(
    body: LedgerNotice(
      title: context.l10n.commonFailedTitle,
      body: context.l10n.commonFailedBody(reason),
      action: context.l10n.commonRetry,
      onAction: onRetry,
    ),
  );
}
