import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/ledger_page.dart';
import '../widgets/sketch.dart';
import 'simnet_backend.dart';
import 'simnet_chain.dart';
import 'simnet_state.dart';

/// The simnet's explorer: the burner wallet, the program's addresses, and
/// every transaction the wallet has signed, newest first.
class SimnetScreen extends HookConsumerWidget {
  const SimnetScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final chain = ref.watch(simnetChainProvider);
    final player = ref.watch(playerProvider).value;
    final transactions = ref.watch(simnetLedgerProvider).value ?? const [];

    if (chain == null || player == null) {
      return const SizedBox.shrink();
    }

    final burned = transactions
        .where(
          (transaction) => transaction.instruction == SimnetInstruction.burn,
        )
        .length;

    return LedgerPage(
      overline: l10n.simnetOverline,
      title: l10n.simnetTitle,
      trailing: Text(
        l10n.commonSol(solAmount(player.lamports)),
        style: Veil.figure(size: 14),
      ),
      children: [
        Text(l10n.simnetIntro, style: Veil.body(color: Veil.ash, size: 13)),
        const SizedBox(height: 12),
        LedgerRow(label: l10n.rosterNetwork, value: l10n.simnetNetworkName),
        LedgerRow(label: l10n.simnetBurned, value: '$burned', emphasised: true),
        _Address(label: l10n.simnetWallet, address: chain.wallet),
        _Address(label: l10n.simnetRoleProgram, address: SimnetChain.program),
        _Address(label: l10n.simnetRoleKeeper, address: SimnetChain.keeper),
        LedgerHeading(
          l10n.simnetTransactions,
          trailing: '${transactions.length}',
        ),
        if (transactions.isEmpty)
          Text(l10n.simnetEmpty, style: Veil.body(color: Veil.ash, size: 14)),
        for (final transaction in transactions.reversed)
          _TransactionRow(
            transaction: transaction,
            onOpen: () => showModalBottomSheet<void>(
              context: context,
              backgroundColor: Veil.inkRaised,
              isScrollControlled: true,
              shape: const RoundedRectangleBorder(),
              builder: (_) => _TransactionSheet(transaction: transaction),
            ),
          ),
        const SizedBox(height: 24),
        LedgerButton(
          key: const Key('simnet-reset'),
          label: l10n.simnetReset,
          detail: l10n.simnetResetDetail,
          quiet: true,
          onPressed: () => unawaited(_reset(context, ref)),
        ),
      ],
    );
  }

  /// Asks, then starts the simnet over and goes back to the threshold.
  static Future<void> _reset(BuildContext context, WidgetRef ref) async {
    final l10n = context.l10n;
    final confirmed = await showDialog<bool>(
      context: context,
      barrierColor: Veil.ink.withValues(alpha: 0.8),
      builder: (context) => Dialog(
        backgroundColor: Veil.inkRaised,
        shape: const RoundedRectangleBorder(),
        child: SketchFrame(
          seed: Sketch.seedFor(l10n.simnetReset),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(l10n.simnetReset.toUpperCase(), style: Veil.label()),
                const SizedBox(height: 12),
                Text(l10n.simnetResetBody, style: Veil.body()),
                const SizedBox(height: 22),
                LedgerButton(
                  key: const Key('simnet-reset-confirm'),
                  label: l10n.simnetReset,
                  onPressed: () => Navigator.of(context).pop(true),
                ),
                const SizedBox(height: 8),
                LedgerButton(
                  label: l10n.simnetCancel,
                  quiet: true,
                  onPressed: () => Navigator.of(context).pop(false),
                ),
              ],
            ),
          ),
        ),
      ),
    );

    if (confirmed != true || !context.mounted) {
      return;
    }

    final backend = ref.read(gameBackendProvider);

    if (backend is! SimnetBackend) {
      return;
    }

    await backend.reset();
    ref
      ..invalidate(simnetLedgerProvider)
      ..invalidate(playerProvider);

    if (context.mounted) {
      Navigator.of(context).popUntil((route) => route.isFirst);
    }
  }
}

/// An address in full, under its name, for copying.
class _Address extends HookWidget {
  const _Address({required this.label, required this.address});

  final String label;
  final String address;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 7),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label.toUpperCase(), style: Veil.label()),
        const SizedBox(height: 4),
        SelectableText(address, style: Veil.figure(size: 12)),
      ],
    ),
  );
}

class _TransactionRow extends HookWidget {
  const _TransactionRow({required this.transaction, required this.onOpen});

  final SimnetTransaction transaction;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;
    final instruction = simnetInstructionName(l10n, transaction.instruction);
    final subject =
        transaction.nft?.title ??
        l10n.commonSol(solAmount(transaction.lamports));

    return Semantics(
      button: true,
      label:
          '$instruction, $subject, ${l10n.simnetSlotValue(transaction.slot)}',
      excludeSemantics: true,
      child: GestureDetector(
        key: Key('simnet-tx-${transaction.signature}'),
        behavior: HitTestBehavior.opaque,
        onTap: onOpen,
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 9),
          child: Row(
            children: [
              SizedBox(
                width: 72,
                child: Text(
                  instruction.toUpperCase(),
                  style: Veil.label(color: light),
                ),
              ),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(subject, style: Veil.body(size: 14)),
                    const SizedBox(height: 2),
                    Text(
                      l10n.simnetSlotValue(transaction.slot),
                      style: Veil.body(color: Veil.ash, size: 12),
                    ),
                  ],
                ),
              ),
              Text(
                shortAddress(transaction.signature),
                style: Veil.figure(color: Veil.ash, size: 12),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// One transaction in full, as an explorer would show it.
class _TransactionSheet extends HookWidget {
  const _TransactionSheet({required this.transaction});

  final SimnetTransaction transaction;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final burn = transaction.instruction == SimnetInstruction.burn;

    return SafeArea(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              simnetInstructionName(
                l10n,
                transaction.instruction,
              ).toUpperCase(),
              style: Veil.label(),
            ),
            const SizedBox(height: 8),
            Text(
              transaction.nft?.title ??
                  l10n.commonSol(solAmount(transaction.lamports)),
              style: Veil.display(28),
            ),
            const SizedBox(height: 12),
            LedgerRow(label: l10n.simnetStatus, value: l10n.simnetFinalized),
            LedgerRow(label: l10n.simnetSlot, value: '${transaction.slot}'),
            if (transaction.lamports > 0)
              LedgerRow(
                label: l10n.simnetAmount,
                value: l10n.commonSol(solAmount(transaction.lamports)),
              ),
            _Address(
              label: l10n.simnetSignature,
              address: transaction.signature,
            ),
            LedgerHeading(l10n.simnetAccounts),
            for (final account in transaction.accounts)
              _Address(
                label: simnetRoleName(l10n, account.role),
                address: account.address,
              ),
            if (burn) ...[
              LedgerHeading(l10n.simnetReceipt),
              Text(
                l10n.simnetReceiptBody,
                style: Veil.body(color: Veil.ash, size: 13),
              ),
              const SizedBox(height: 8),
              LedgerRow(
                label: l10n.simnetReceipt,
                value: transaction.receiptVerifies
                    ? l10n.simnetReceiptPasses
                    : l10n.simnetReceiptFails,
                emphasised: transaction.receiptVerifies,
              ),
            ],
          ],
        ),
      ),
    );
  }
}
