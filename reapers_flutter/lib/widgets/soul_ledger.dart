import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/soul_lookups.dart';
import '../theme/veil.dart';
import 'ledger.dart';

/// The same birth contract and earned growth, in the codex and the studio.
class SoulLedger extends HookWidget {
  const SoulLedger({
    super.key,
    required this.genome,
    required this.level,
    required this.memory,
  });

  final ReaperGenome genome;
  final int level;
  final SoulMemory memory;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final soul = ReaperSoul.fromGenome(genome);
    final gift = soul.gift;
    final rank = SoulRules.rank(level: level, memory: memory, gift: gift);
    final light = Theme.of(context).colorScheme.primary;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 22),
        Text(giftName(l10n, gift), style: Veil.display(28, color: light)),
        const SizedBox(height: 10),
        Text(giftEffect(l10n, gift, rank + 1), style: Veil.body()),
        const SizedBox(height: 8),
        Text(
          l10n.soulRank(rank, SoulRules.maxRank),
          style: Veil.body(color: light, size: 13),
        ),
        const SizedBox(height: 12),
        Text(
          rank == SoulRules.maxRank
              ? l10n.soulComplete
              : l10n.soulNext(
                  SoulRules.rankLevels[rank + 1],
                  SoulRules.rankMarks[rank + 1],
                  soulPathName(l10n, gift.path),
                  memory.marks(gift.path),
                ),
          style: Veil.body(color: Veil.ash, size: 13),
        ),
        const SizedBox(height: 22),
        Text(
          l10n.soulTemperament(temperamentName(l10n, soul.temperament)),
          style: Veil.display(22),
        ),
        const SizedBox(height: 8),
        Text(
          temperamentHelp(l10n, soul.temperament),
          style: Veil.body(color: Veil.ash),
        ),
        LedgerHeading(l10n.soulMarks),
        for (final path in SoulPath.values) ...[
          LedgerRow(
            label: soulPathName(l10n, path),
            value: '${memory.marks(path)}',
            emphasised: path == gift.path,
          ),
          Text(
            soulPathHelp(l10n, path),
            style: Veil.body(color: Veil.ash, size: 13),
          ),
          const SizedBox(height: 10),
        ],
        Text(l10n.soulMemoryHelp, style: Veil.body(color: Veil.ash, size: 13)),
        const SizedBox(height: 18),
        ExpansionTile(
          tilePadding: EdgeInsets.zero,
          childrenPadding: const EdgeInsets.only(bottom: 16),
          title: Text(l10n.soulLore, style: Veil.body()),
          children: [
            Text(giftLore(l10n, gift), style: Veil.body(color: Veil.ash)),
          ],
        ),
        for (var next = 1; next <= SoulRules.maxRank; next++)
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: Icon(
              next <= rank ? Icons.check_circle_outline : Icons.lock_outline,
            ),
            title: Text(
              l10n.soulMilestone(
                SoulRules.rankLevels[next],
                SoulRules.rankMarks[next],
                soulPathName(l10n, gift.path),
              ),
              style: Veil.body(),
            ),
            subtitle: Text(
              giftEffect(l10n, gift, next + 1),
              style: Veil.body(color: Veil.ash, size: 13),
            ),
          ),
      ],
    );
  }
}
