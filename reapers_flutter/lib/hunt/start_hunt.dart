import 'dart:math';

import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../src/state/player.dart';
import 'hunt_screen.dart';

/// Opens a practice hunt: a stray husk, the reaper's own strength, and
/// nothing gained or lost.
Future<void> startPracticeHunt(BuildContext context, WidgetRef ref) {
  final random = Random();
  final strays = marketListings
      .where((nft) => nft.rarity.index <= Rarity.uncommon.index)
      .toList();
  final brief = HuntBrief(
    nft: strays[random.nextInt(strays.length)],
    seed: random.nextInt(1 << 32),
    loadout: ref.read(playerProvider).value?.loadout ?? HuntLoadout.borrowed,
  );

  return Navigator.of(context).push(
    MaterialPageRoute<void>(
      builder: (_) => HuntScreen(
        brief: brief,
        settle: (_, outcome) async => HuntReward.practice(outcome),
      ),
    ),
  );
}

/// Opens the hunt for a loose husk and has the keeper judge how it ends.
Future<void> startHunt(BuildContext context, WidgetRef ref, Sighting sighting) {
  final player = ref.read(playerProvider).requireValue;
  final art = Art.carriedBy(sighting.nft.collection.vessel);
  final brief = HuntBrief(
    nft: sighting.nft,
    seed: sighting.seed,
    loadout: player.loadout,
    sightingId: sighting.id,
  );

  return Navigator.of(context).push(
    MaterialPageRoute<void>(
      builder: (_) => HuntScreen(
        brief: brief,
        settle: (log, _) async {
          final settlement = await ref
              .read(playerProvider.notifier)
              .settleHunt(sighting.id, log);

          return HuntReward(
            outcome: settlement.outcome,
            xp: settlement.xp,
            anima: settlement.anima,
            feeding: settlement.feeding,
            signed:
                settlement.outcome.absorbed &&
                settlement.state.codex.last.signature != null,
            levelBefore: player.level,
            levelAfter: settlement.state.level,
            gift: player.activeReaper?.soul.gift,
            soulRankBefore: player.activeReaper?.soulRank ?? 0,
            soulRankAfter: settlement.state.activeReaper?.soulRank ?? 0,
            art: art,
            artRankBefore: art == null
                ? 0
                : player.activeReaper?.artRank(art) ?? 0,
            artRankAfter: art == null
                ? 0
                : settlement.state.activeReaper?.artRank(art) ?? 0,
          );
        },
      ),
    ),
  );
}
