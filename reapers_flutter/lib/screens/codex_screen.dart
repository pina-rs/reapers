import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/ledger_page.dart';
import '../widgets/nft_thumbnail.dart';

/// The record of everything this reaper has absorbed, collection by
/// collection, and how far each set is from complete.
class CodexScreen extends HookConsumerWidget {
  const CodexScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).requireValue;
    final light = Theme.of(context).colorScheme.primary;

    return LedgerPage(
      overline: l10n.codexOverline,
      title: l10n.codexTitle,
      children: [
        if (player.codex.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 14),
            child: Text(l10n.codexEmpty, style: Veil.body(color: Veil.ash)),
          ),
        for (final collection in NftCollection.values)
          if (player.absorbedFrom(collection) > 0) ...[
            LedgerHeading(
              collection.title,
              trailing: l10n.codexSet(
                player.absorbedFrom(collection),
                collection.rarities.length,
              ),
            ),
            Text(
              player.absorbedFrom(collection) >= NftCollection.setSize
                  ? l10n.codexSetComplete
                  : l10n.codexSetProgress(
                      NftCollection.setSize - player.absorbedFrom(collection),
                    ),
              style: Veil.body(
                size: 13,
                color: player.absorbedFrom(collection) >= NftCollection.setSize
                    ? light
                    : Veil.ash,
              ),
            ),
            const SizedBox(height: 8),
            for (final entry in player.codex.where(
              (entry) => entry.nft.collection == collection,
            ))
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 6),
                child: Row(
                  children: [
                    NftThumbnail(nft: entry.nft, size: 44),
                    const SizedBox(width: 14),
                    Expanded(
                      child: Text(entry.nft.title, style: Veil.body(size: 15)),
                    ),
                    Text(
                      switch (entry.grade) {
                        ReapGrade.perfect => l10n.huntGradePerfect,
                        ReapGrade.clean => l10n.huntGradeClean,
                        ReapGrade.ragged => l10n.huntGradeRagged,
                      }.toUpperCase(),
                      style: Veil.label(size: 10),
                    ),
                  ],
                ),
              ),
          ],
        if (player.huntsLost > 0) ...[
          const SizedBox(height: 18),
          LedgerRow(label: l10n.codexLost, value: '${player.huntsLost}'),
        ],
      ],
    );
  }
}
