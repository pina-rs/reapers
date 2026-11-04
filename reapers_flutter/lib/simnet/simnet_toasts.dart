import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../theme/veil.dart';
import 'simnet_chain.dart';
import 'simnet_state.dart';

/// Lays a slip over [child] each time a simnet transaction is confirmed, so
/// the chain is seen working wherever the player is: in the vault, or at the
/// end of a hunt as the husk burns.
///
/// In a build without a simnet this is [child] and nothing more.
class SimnetToasts extends HookConsumerWidget {
  const SimnetToasts({super.key, required this.child});

  /// How long a transaction shows as confirmed before it is finalized, and
  /// how long its slip stays up. Shorter than a cluster's finality, so the
  /// slip is gone before the player has to wait for it.
  static const finality = Duration(milliseconds: 1400);
  static const shown = Duration(milliseconds: 4200);

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chain = ref.watch(simnetChainProvider);
    final current = useState<SimnetTransaction?>(null);
    final finalized = useState(false);

    useEffect(() {
      final subscription = chain?.posted.listen((transaction) {
        current.value = transaction;
        finalized.value = false;
      });

      return subscription?.cancel;
    }, [chain]);

    useEffect(() {
      if (current.value == null) {
        return null;
      }

      final finalizing = Timer(finality, () => finalized.value = true);
      final hiding = Timer(shown, () => current.value = null);

      return () {
        finalizing.cancel();
        hiding.cancel();
      };
    }, [current.value?.signature]);

    if (chain == null) {
      return child;
    }

    final still = MediaQuery.disableAnimationsOf(context);

    return Stack(
      children: [
        child,
        SafeArea(
          child: Align(
            alignment: Alignment.topCenter,
            child: IgnorePointer(
              child: AnimatedSwitcher(
                duration: still
                    ? Duration.zero
                    : const Duration(milliseconds: 240),
                transitionBuilder: (child, animation) => FadeTransition(
                  opacity: animation,
                  child: SlideTransition(
                    position: Tween(
                      begin: const Offset(0, -0.6),
                      end: Offset.zero,
                    ).animate(animation),
                    child: child,
                  ),
                ),
                child: switch (current.value) {
                  final transaction? => _Slip(
                    key: ValueKey(transaction.signature),
                    transaction: transaction,
                    finalized: finalized.value,
                  ),
                  null => const SizedBox.shrink(),
                },
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// One transaction, as a slip of paper pinned to the top of the screen.
class _Slip extends HookWidget {
  const _Slip({
    super.key,
    required this.transaction,
    required this.finalized,
  });

  final SimnetTransaction transaction;
  final bool finalized;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;
    final instruction = simnetInstructionName(l10n, transaction.instruction);
    final status = finalized ? l10n.simnetFinalized : l10n.simnetConfirmed;
    final subject = [
      ?transaction.nft?.title,
      if (transaction.lamports > 0)
        l10n.commonSol(solAmount(transaction.lamports)),
    ].join(' · ');

    return Semantics(
      liveRegion: true,
      label: '${l10n.simnetOverline}, $instruction, $subject, $status',
      excludeSemantics: true,
      child: Container(
        key: const Key('simnet-slip'),
        constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
        margin: const EdgeInsets.fromLTRB(16, 8, 16, 0),
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
        decoration: BoxDecoration(
          color: Veil.inkRaised,
          border: Border.all(color: Veil.hairline),
        ),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    '${l10n.simnetOverline} · $instruction'.toUpperCase(),
                    style: Veil.label(color: light),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    subject,
                    style: Veil.body(size: 14),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 2),
                  Text(
                    [
                      status,
                      l10n.simnetSlotValue(transaction.slot),
                      if (transaction.instruction == SimnetInstruction.burn)
                        transaction.receiptVerifies
                            ? l10n.simnetReceiptPasses
                            : l10n.simnetReceiptFails,
                    ].join(' · '),
                    style: Veil.body(color: Veil.ash, size: 12),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 12),
            Text(
              shortAddress(transaction.signature),
              style: Veil.figure(color: Veil.ash, size: 12),
            ),
          ],
        ),
      ),
    );
  }
}
