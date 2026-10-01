import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../theme/veil.dart';
import 'ledger.dart';

/// A class's four ratings, each as five pips.
class ReaperStatRows extends HookWidget {
  const ReaperStatRows({super.key, required this.stats});

  final ReaperStats stats;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;

    return Column(
      children: [
        for (final (label, rating) in [
          (l10n.sanctumStatStrike, stats.strike),
          (l10n.sanctumStatReach, stats.reach),
          (l10n.sanctumStatCapture, stats.capture),
          (l10n.sanctumStatResilience, stats.resilience),
        ])
          Semantics(
            label: label,
            value: '$rating',
            excludeSemantics: true,
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 7),
              child: Row(
                children: [
                  Expanded(
                    child: Text(label.toUpperCase(), style: Veil.label()),
                  ),
                  for (var pip = 1; pip <= 5; pip++)
                    Padding(
                      padding: const EdgeInsets.only(left: 9),
                      child: LedgerPip(lit: pip <= rating, colour: light),
                    ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
