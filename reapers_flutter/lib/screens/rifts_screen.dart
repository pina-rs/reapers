import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../rift/rift_screen.dart';
import '../src/backend/rift_session.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/attempt.dart';
import '../widgets/ledger.dart';
import '../widgets/ledger_page.dart';

/// Every rift there is, where each one opens, and whether it will open for
/// this reaper.
class RiftsScreen extends HookConsumerWidget {
  const RiftsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final player = ref.watch(playerProvider).requireValue;
    final entering = useState<RiftSite?>(null);

    Future<void> enter(RiftSite site) async {
      RiftSession? session;

      entering.value = site;

      await attempt(context, () async {
        session = await ref.read(playerProvider.notifier).enterRift(site);
      });

      if (!context.mounted) {
        session?.leave();
        return;
      }

      entering.value = null;

      final entered = session;

      if (entered == null) {
        return;
      }

      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => RiftScreen(site: site, session: entered),
        ),
      );
    }

    return LedgerPage(
      overline: l10n.riftsOverline,
      title: l10n.riftsTitle,
      children: [
        Text(l10n.riftsStandIn, style: Veil.body(color: Veil.ash, size: 13)),
        const SizedBox(height: 8),
        Text(
          l10n.riftsLocationNote,
          style: Veil.body(color: Veil.ash, size: 13),
        ),
        for (final site in RiftSite.values)
          _SiteBlock(
            site: site,
            attuned: player.attunedTo(site),
            entering: entering.value == site,
            onEnter: entering.value == null ? () => enter(site) : null,
          ),
      ],
    );
  }
}

/// One rift: the place, whose dead come through it, and the way in.
class _SiteBlock extends HookWidget {
  const _SiteBlock({
    required this.site,
    required this.attuned,
    required this.entering,
    required this.onEnter,
  });

  final RiftSite site;
  final bool attuned;
  final bool entering;
  final VoidCallback? onEnter;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;

    return Padding(
      padding: const EdgeInsets.only(top: 26),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  l10n
                      .riftsSite(site.city, site.collection.title)
                      .toUpperCase(),
                  style: Veil.label(size: 10),
                ),
              ),
              Text(
                _coordinates(site),
                style: Veil.figure(color: Veil.ash, size: 11),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(site.title, style: Veil.display(28)),
          const SizedBox(height: 12),
          if (attuned)
            LedgerButton(
              key: Key('rift-enter-${site.name}'),
              label: entering ? l10n.riftEntering : l10n.riftsOpen,
              onPressed: onEnter,
            )
          else
            Text(
              l10n.riftsAttune(site.collection.title),
              key: Key('rift-shut-${site.name}'),
              style: Veil.body(color: Veil.ash, size: 14),
            ),
        ],
      ),
    );
  }

  /// The site's position as it would be written on a chart.
  static String _coordinates(RiftSite site) {
    String part(double value, String positive, String negative) =>
        '${value.abs().toStringAsFixed(4)}° ${value >= 0 ? positive : negative}';

    return '${part(site.latitude, 'N', 'S')}  ${part(site.longitude, 'E', 'W')}';
  }
}
