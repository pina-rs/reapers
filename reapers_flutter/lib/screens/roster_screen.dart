import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:solana_kit/solana_kit.dart' as kit;
import 'package:url_launcher/url_launcher.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../l10n/soul_lookups.dart';
import '../solana/chain_state.dart';
import '../src/backend/game_backend.dart';
import '../src/backend/server_backend.dart';
import '../src/state/player.dart';
import '../studio/reaper_studio.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';

class RosterScreen extends HookConsumerWidget {
  const RosterScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).value;
    final chain = ref.watch(chainStatusProvider);
    final status = chain.value;
    final backend = ref.watch(gameBackendProvider);
    final busy = useState(false);
    final failure = useState<String?>(null);
    final asset = useTextEditingController();
    final canConnect =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

    Future<void> run(Future<void> Function() action) async {
      if (busy.value) return;
      busy.value = true;
      failure.value = null;
      try {
        await action();
      } on RuleBreach catch (error) {
        if (context.mounted) {
          failure.value = refusalMessage(l10n, error.refusal);
        }
      } on BackendUnavailable {
        if (context.mounted) failure.value = l10n.commonUnreachable;
      } on Exception {
        // Wallet cancellation, unavailable transports and launcher failures
        // surface a failed action, never a successful link or import.
        if (context.mounted) failure.value = l10n.commonFailedTitle;
      } finally {
        if (context.mounted) busy.value = false;
      }
    }

    return Scaffold(
      appBar: AppBar(title: Text(l10n.rosterTitle, style: Veil.display(26))),
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
            child: ListView(
              padding: const EdgeInsets.all(24),
              children: [
                Text(l10n.rosterIntro, style: Veil.body(color: Veil.ash)),
                const SizedBox(height: 16),
                if (player != null)
                  for (final reaper in player.reapers)
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(reaper.genome.name, style: Veil.display(22)),
                      subtitle: Text(
                        '${reaper.isNft ? l10n.rosterNft : l10n.rosterStarter} · ${l10n.studioLevelValue(reaper.level)}\n${giftName(l10n, reaper.soul.gift)} · ${temperamentName(l10n, reaper.soul.temperament)}',
                      ),
                      selected: reaper.id == player.activeReaperId,
                      trailing: reaper.id == player.activeReaperId
                          ? Tooltip(
                              message: l10n.rosterActive,
                              child: const Icon(Icons.check_circle_outline),
                            )
                          : null,
                      onTap: busy.value
                          ? null
                          : () => unawaited(
                              run(
                                () => ref
                                    .read(playerProvider.notifier)
                                    .selectReaper(reaper.id),
                              ),
                            ),
                    ),
                const SizedBox(height: 16),
                LedgerButton(
                  label: l10n.sanctumStudio,
                  detail: l10n.sanctumStudioDetail,
                  onPressed: () => unawaited(
                    Navigator.of(context).push(
                      MaterialPageRoute<void>(
                        builder: (_) =>
                            ReaperStudio(initialGenome: player?.genome),
                      ),
                    ),
                  ),
                ),
                LedgerHeading(l10n.rosterLinked),
                if (backend is! ServerBackend)
                  Text(l10n.rosterOffline, style: Veil.body(color: Veil.ash))
                else ...[
                  if (chain.isLoading) const LinearProgressIndicator(),
                  if (chain.hasError) ...[
                    Text(
                      l10n.commonUnreachable,
                      style: Veil.body(color: Veil.wound),
                    ),
                    TextButton(
                      onPressed: () => ref.invalidate(chainStatusProvider),
                      child: Text(l10n.commonRetry),
                    ),
                  ],
                  if (status != null) ...[
                    LedgerRow(label: l10n.rosterNetwork, value: status.chain),
                    if (status.wallet case final wallet?) ...[
                      SelectableText(wallet, style: Veil.figure(size: 12)),
                      if (status.lamports case final balance?)
                        LedgerRow(
                          label: l10n.rosterBalance,
                          value: solAmount(balance),
                        ),
                    ],
                    const SizedBox(height: 12),
                    Text(
                      l10n.rosterLinkHelp,
                      style: Veil.body(color: Veil.ash, size: 13),
                    ),
                    const SizedBox(height: 12),
                    if (!canConnect)
                      Text(
                        l10n.rosterWalletUnsupported,
                        style: Veil.body(color: Veil.ash, size: 13),
                      ),
                    LedgerButton(
                      label: l10n.rosterConnect,
                      onPressed:
                          busy.value ||
                              !canConnect ||
                              Uri.parse(status.identityUri).host
                                  .endsWith('.invalid')
                          ? null
                          : () => unawaited(
                              run(() async {
                                await backend.connectWallet(status);
                                await ref
                                    .read(playerProvider.notifier)
                                    .refresh();
                                ref.invalidate(chainStatusProvider);
                              }),
                            ),
                    ),
                    const SizedBox(height: 8),
                    LedgerButton(
                      label: l10n.rosterRefresh,
                      onPressed: busy.value
                          ? null
                          : () => unawaited(
                              run(() async {
                                await ref
                                    .read(playerProvider.notifier)
                                    .refresh();
                                ref.invalidate(chainStatusProvider);
                              }),
                            ),
                    ),
                    if (status.collection == null)
                      Padding(
                        padding: const EdgeInsets.only(top: 18),
                        child: Text(
                          l10n.rosterUnconfigured,
                          style: Veil.body(color: Veil.ash),
                        ),
                      )
                    else ...[
                      LedgerHeading(l10n.rosterImport),
                      Text(
                        l10n.rosterImportHelp,
                        style: Veil.body(color: Veil.ash, size: 13),
                      ),
                      const SizedBox(height: 12),
                      TextField(
                        controller: asset,
                        decoration: InputDecoration(
                          labelText: l10n.rosterAsset,
                        ),
                        autocorrect: false,
                        enableSuggestions: false,
                      ),
                      const SizedBox(height: 12),
                      LedgerButton(
                        label: l10n.rosterImport,
                        onPressed: busy.value || status.wallet == null
                            ? null
                            : () => unawaited(
                                run(() async {
                                  final address = asset.text.trim();
                                  if (!kit.isAddress(address)) {
                                    throw const RuleBreach(Refusal.notOwned);
                                  }
                                  await backend.importReaper(address);
                                  await ref
                                      .read(playerProvider.notifier)
                                      .refresh();
                                  if (context.mounted) asset.clear();
                                }),
                              ),
                      ),
                      if (status.marketplaceUri case final market?) ...[
                        const SizedBox(height: 18),
                        Text(
                          l10n.rosterBuyHelp,
                          style: Veil.body(color: Veil.ash, size: 13),
                        ),
                        const SizedBox(height: 12),
                        LedgerButton(
                          label: l10n.rosterBuy,
                          onPressed: busy.value
                              ? null
                              : () => unawaited(
                                  run(() async {
                                    final uri = Uri.parse(market);
                                    if (uri.scheme != 'https' ||
                                        !await launchUrl(
                                          uri,
                                          mode: LaunchMode.externalApplication,
                                        )) {
                                      throw const BackendUnavailable(
                                        'Marketplace did not open',
                                      );
                                    }
                                  }),
                                ),
                        ),
                      ],
                    ],
                  ],
                ],
                if (busy.value)
                  const Padding(
                    padding: EdgeInsets.only(top: 18),
                    child: LinearProgressIndicator(),
                  ),
                if (failure.value case final message?)
                  Padding(
                    padding: const EdgeInsets.only(top: 18),
                    child: Semantics(
                      liveRegion: true,
                      child: Text(message, style: Veil.body(color: Veil.wound)),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
