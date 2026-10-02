import 'dart:async';

import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../l10n/l10n.dart';
import '../sound/soundtrack.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/ledger_page.dart';

/// Who made what the game is made of, and the one switch that goes with it:
/// whether the music plays.
///
/// Two of the three recordings are free to use on the condition that their
/// composer is named, so this page is part of what lets the game use them.
class ColophonScreen extends HookConsumerWidget {
  const ColophonScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final music = ref.watch(musicEnabledProvider);

    return LedgerPage(
      overline: l10n.colophonOverline,
      title: l10n.colophonTitle,
      children: [
        const SizedBox(height: 14),
        LedgerButton(
          key: const Key('colophon-music'),
          label: l10n.colophonMusic,
          detail: music ? l10n.colophonMusicOn : l10n.colophonMusicOff,
          onPressed: () =>
              unawaited(ref.read(musicEnabledProvider.notifier).toggle()),
        ),
        LedgerHeading(l10n.colophonMusicHeading),
        _Credit(
          where: l10n.colophonSanctum,
          what: l10n.colophonBachPiece,
          who: l10n.colophonBachCredit,
        ),
        _Credit(
          where: l10n.colophonHunt,
          what: l10n.colophonAnabasisPiece,
          who: l10n.colophonAnabasisCredit,
        ),
        _Credit(
          where: l10n.colophonRift,
          what: l10n.colophonKatabasisPiece,
          who: l10n.colophonKatabasisCredit,
        ),
        LedgerHeading(l10n.colophonSightHeading),
        _Credit(
          where: l10n.colophonSight,
          what: l10n.colophonDepthPiece,
          who: l10n.colophonDepthCredit,
        ),
        LedgerHeading(l10n.colophonTypeHeading),
        _Credit(
          where: l10n.colophonType,
          what: l10n.colophonTypePiece,
          who: l10n.colophonTypeCredit,
        ),
      ],
    );
  }
}

/// One thing somebody else made: where it is used, what it is, and whose it
/// is.
class _Credit extends HookConsumerWidget {
  const _Credit({required this.where, required this.what, required this.who});

  final String where;
  final String what;
  final String who;

  @override
  Widget build(BuildContext context, WidgetRef ref) => Padding(
    padding: const EdgeInsets.only(bottom: 16),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(where.toUpperCase(), style: Veil.label(size: 10)),
        const SizedBox(height: 4),
        Text(what, style: Veil.body(size: 15)),
        const SizedBox(height: 2),
        Text(who, style: Veil.body(size: 13, color: Veil.ash)),
      ],
    ),
  );
}
