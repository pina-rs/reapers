import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/attempt.dart';
import '../widgets/ledger.dart';
import '../widgets/ledger_page.dart';
import '../widgets/nft_thumbnail.dart';

/// Where an NFT stands with this player.
enum _Holding { listed, held, loose, absorbed }

_Holding _holdingOf(PlayerState player, Nft nft) {
  if (player.vault.contains(nft)) {
    return _Holding.held;
  }

  if (player.sightings.any((sighting) => sighting.nft == nft)) {
    return _Holding.loose;
  }

  if (player.codex.any((entry) => entry.nft == nft)) {
    return _Holding.absorbed;
  }

  return _Holding.listed;
}

/// The vault and the market in one ledger: every NFT of every collection,
/// and where each one stands.
///
/// Buying here is a stand-in. See docs/nfts.md for what replaces it.
class VaultScreen extends HookConsumerWidget {
  const VaultScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).requireValue;

    return LedgerPage(
      overline: l10n.vaultOverline,
      title: l10n.vaultTitle,
      trailing: Text(
        l10n.commonSol(solAmount(player.lamports)),
        key: const Key('vault-wallet'),
        style: Veil.figure(size: 14),
      ),
      children: [
        Text(l10n.vaultStandIn, style: Veil.body(color: Veil.ash, size: 13)),
        for (final collection in NftCollection.values) ...[
          LedgerHeading(
            collection.title,
            trailing: l10n.codexSet(
              player.absorbedFrom(collection),
              collection.rarities.length,
            ),
          ),
          for (final nft in marketListings.where(
            (nft) => nft.collection == collection,
          ))
            _NftRow(
              nft: nft,
              holding: _holdingOf(player, nft),
              onOpen: () => showModalBottomSheet<void>(
                context: context,
                backgroundColor: Veil.inkRaised,
                isScrollControlled: true,
                shape: const RoundedRectangleBorder(),
                builder: (_) => _NftSheet(nft: nft),
              ),
            ),
        ],
      ],
    );
  }
}

class _NftRow extends HookWidget {
  const _NftRow({
    required this.nft,
    required this.holding,
    required this.onOpen,
  });

  final Nft nft;
  final _Holding holding;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;
    final (status, colour) = switch (holding) {
      _Holding.listed => (
        l10n.commonSol(solAmount(nft.rarity.lamports)),
        Veil.bone,
      ),
      _Holding.held => (l10n.vaultHeld, light),
      _Holding.loose => (l10n.vaultLoose, light),
      _Holding.absorbed => (l10n.vaultAbsorbed, Veil.ash),
    };

    return Semantics(
      button: true,
      label: '${nft.title}, ${rarityName(l10n, nft.rarity)}, $status',
      excludeSemantics: true,
      child: GestureDetector(
        key: Key('vault-nft-${nft.id}'),
        behavior: HitTestBehavior.opaque,
        onTap: onOpen,
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 7),
          child: Row(
            children: [
              Opacity(
                opacity: holding == _Holding.absorbed ? 0.35 : 1,
                child: NftThumbnail(nft: nft, size: 48),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(nft.title, style: Veil.body(size: 15)),
                    const SizedBox(height: 3),
                    Text(
                      rarityName(l10n, nft.rarity).toUpperCase(),
                      style: Veil.label(size: 10),
                    ),
                  ],
                ),
              ),
              Text(status, style: Veil.figure(color: colour, size: 13)),
            ],
          ),
        ),
      ),
    );
  }
}

/// One NFT in full: its image, what it is worth, and what can be done with
/// it now.
class _NftSheet extends HookConsumerWidget {
  const _NftSheet({required this.nft});

  final Nft nft;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).requireValue;
    final holding = _holdingOf(player, nft);
    final busy = useState(false);
    final rarity = nft.rarity;

    Future<void> act(Future<void> Function() request) async {
      busy.value = true;

      final done = await attempt(context, request);

      if (!context.mounted) {
        return;
      }

      busy.value = false;

      // Releasing sends the player back to watch for the husk.
      if (done && holding == _Holding.held) {
        Navigator.of(context).popUntil((route) => route.isFirst);
      }
    }

    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(24, 24, 24, 24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                NftThumbnail(nft: nft, size: 132),
                const SizedBox(width: 18),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        rarityName(l10n, rarity).toUpperCase(),
                        style: Veil.label(),
                      ),
                      const SizedBox(height: 8),
                      Text(nft.title, style: Veil.display(28)),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            LedgerRow(label: l10n.vaultWard, value: '${rarity.ward}'),
            LedgerRow(
              label: l10n.vaultWorth,
              value: l10n.vaultWorthValue(rarity.xp, rarity.anima),
            ),
            LedgerRow(
              label: l10n.vaultMint,
              value:
                  '${nft.mockMint.substring(0, 4)}…${nft.mockMint.substring(40)}',
            ),
            const SizedBox(height: 18),
            switch (holding) {
              _Holding.listed => LedgerButton(
                key: const Key('vault-buy'),
                label: l10n.vaultBuy,
                detail: l10n.commonSol(solAmount(rarity.lamports)),
                onPressed: busy.value
                    ? null
                    : () => act(
                        () => ref.read(playerProvider.notifier).acquire(nft),
                      ),
              ),
              _Holding.held => Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.vaultReleaseNote,
                    style: Veil.body(color: Veil.ash, size: 13),
                  ),
                  const SizedBox(height: 12),
                  LedgerButton(
                    key: const Key('vault-release'),
                    label: l10n.vaultRelease,
                    onPressed: busy.value
                        ? null
                        : () => act(
                            () =>
                                ref.read(playerProvider.notifier).release(nft),
                          ),
                  ),
                ],
              ),
              _Holding.loose => LedgerButton(
                label: l10n.vaultLoose,
                onPressed: null,
              ),
              _Holding.absorbed => LedgerButton(
                label: l10n.vaultAbsorbed,
                onPressed: null,
              ),
            },
          ],
        ),
      ),
    );
  }
}
