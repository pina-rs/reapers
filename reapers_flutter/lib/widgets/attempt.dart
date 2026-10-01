import 'package:flutter/material.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../src/backend/game_backend.dart';
import '../theme/veil.dart';
import 'ledger.dart';

/// Runs [request] and, when the rules refuse it or the keeper cannot be
/// reached, tells the player why. Returns whether it went through.
///
/// Every button that changes the player goes through here, so a refusal is
/// always explained and never silently dropped.
Future<bool> attempt(
  BuildContext context,
  Future<void> Function() request,
) async {
  try {
    await request();

    return true;
  } on RuleBreach catch (breach) {
    if (context.mounted) {
      await _explain(
        context,
        title: context.l10n.commonRefusedTitle,
        body: refusalMessage(context.l10n, breach.refusal),
      );
    }

    return false;
  } on BackendUnavailable catch (failure) {
    debugPrint('Keeper unavailable: ${failure.reason}');

    if (context.mounted) {
      await _explain(
        context,
        title: context.l10n.commonFailedTitle,
        body: context.l10n.commonUnreachable,
      );
    }

    return false;
  }
}

Future<void> _explain(
  BuildContext context, {
  required String title,
  required String body,
}) => showDialog<void>(
  context: context,
  barrierColor: Veil.ink.withValues(alpha: 0.8),
  builder: (context) => Dialog(
    backgroundColor: Veil.inkRaised,
    shape: const RoundedRectangleBorder(
      side: BorderSide(color: Veil.hairline),
    ),
    child: Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title.toUpperCase(), style: Veil.label(color: Veil.wound)),
          const SizedBox(height: 12),
          Text(body, style: Veil.display(24)),
          const SizedBox(height: 22),
          LedgerButton(
            key: const Key('refusal-dismiss'),
            label: context.l10n.commonDismiss,
            onPressed: () => Navigator.of(context).pop(),
          ),
        ],
      ),
    ),
  ),
);
