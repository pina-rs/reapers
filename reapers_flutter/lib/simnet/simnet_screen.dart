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
import 'surfpool_rpc.dart';

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
    final local = ref.watch(surfpoolSetupProvider);

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
        if (local.rpc case final rpc?) ...[
          _SurfpoolPanel(
            key: ValueKey('surfpool-${chain.wallet}'),
            rpc: rpc,
            wallet: chain.wallet,
          ),
          const SizedBox(height: 18),
        ] else if (local.problem case final problem?) ...[
          Text(
            l10n.simnetLocalRpcConfig(problem),
            key: const Key('simnet-local-rpc-config'),
            style: Veil.body(color: Veil.wound, size: 13),
          ),
          const SizedBox(height: 18),
        ],
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

/// The development validator beside the simnet: a real RPC to check, and a
/// faucet that pays the burner wallet there without touching the game.
class _SurfpoolPanel extends HookConsumerWidget {
  const _SurfpoolPanel({super.key, required this.rpc, required this.wallet});

  final SurfpoolRpc rpc;
  final String wallet;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final faucet = useMemoized(
      () => SurfpoolFaucet(ref.read(sharedPreferencesProvider), rpc),
      [rpc],
    );
    final receipt = useState<SurfpoolFaucetReceipt?>(null);
    final genesis = useState<String?>(null);
    final balance = useState<int?>(null);
    final failure = useState<Object?>(null);
    final checking = useState(true);
    final airdropping = useState(false);
    final healthy = useState<bool?>(null);

    Future<void> check() async {
      if (!context.mounted) {
        return;
      }

      checking.value = true;
      failure.value = null;

      try {
        // Read before the validator: a record that cannot be read is the
        // panel's own problem, and must not be mistaken for a fresh slate.
        receipt.value = faucet.receiptFor(wallet);
        await rpc.health();
        final hash = await rpc.genesisHash();
        final funds = await rpc.balance(wallet);

        if (!context.mounted) {
          return;
        }

        healthy.value = true;
        genesis.value = hash;
        balance.value = funds;

        // A restarted Surfpool has a new genesis and has forgotten the old
        // airdrop, so the faucet is offered again for the new chain.
        if (receipt.value case final paid? when paid.genesis != hash) {
          receipt.value = null;
        }
      } catch (error) {
        if (!context.mounted) {
          return;
        }

        healthy.value = false;
        failure.value = error;
      } finally {
        if (context.mounted) {
          checking.value = false;
        }
      }
    }

    Future<void> fund() async {
      airdropping.value = true;
      failure.value = null;

      try {
        final paid = await faucet.fund(wallet);

        if (!context.mounted) {
          return;
        }

        receipt.value = paid;
        await check();
      } catch (error) {
        if (context.mounted) {
          failure.value = error;
        }
      } finally {
        if (context.mounted) {
          airdropping.value = false;
        }
      }
    }

    useEffect(() {
      unawaited(check());

      return null;
    }, [rpc]);

    final status = checking.value
        ? l10n.simnetLocalRpcChecking
        : healthy.value == true
        ? l10n.simnetLocalRpcHealthy
        : l10n.simnetLocalRpcUnreachable;

    return Column(
      key: const Key('simnet-local-rpc'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LedgerHeading(l10n.simnetLocalRpc),
        Text(
          l10n.simnetLocalRpcBody,
          style: Veil.body(color: Veil.ash, size: 13),
        ),
        const SizedBox(height: 6),
        _Address(
          label: l10n.simnetLocalRpcEndpoint,
          address: rpc.endpoint.toString(),
        ),
        LedgerRow(label: l10n.simnetLocalRpcHealth, value: status),
        if (genesis.value case final hash?)
          _Address(label: l10n.simnetLocalRpcGenesis, address: hash),
        if (balance.value case final lamports?)
          LedgerRow(
            label: l10n.simnetLocalRpcBalance,
            value: l10n.commonSol(solAmount(lamports)),
          ),
        const SizedBox(height: 8),
        Text(
          l10n.simnetLocalRpcAirdropBody,
          style: Veil.body(color: Veil.ash, size: 13),
        ),
        if (receipt.value case final paid?) ...[
          const SizedBox(height: 4),
          _Address(
            label: l10n.simnetLocalRpcSignature,
            address: paid.signature,
          ),
        ],
        if (failure.value case final error?) ...[
          const SizedBox(height: 6),
          Text(
            l10n.simnetLocalRpcFailure(_reason(error)),
            key: const Key('simnet-local-rpc-failure'),
            style: Veil.body(color: Veil.wound, size: 13),
          ),
        ],
        const SizedBox(height: 10),
        Row(
          children: [
            Expanded(
              child: LedgerButton(
                key: const Key('simnet-local-rpc-check'),
                label: l10n.simnetLocalRpcCheck,
                onPressed: checking.value ? null : () => unawaited(check()),
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: LedgerButton(
                key: const Key('simnet-local-rpc-airdrop'),
                label: receipt.value == null
                    ? l10n.simnetLocalRpcAirdrop(
                        solAmount(SurfpoolFaucet.lamports),
                      )
                    : receipt.value!.confirmed
                    ? l10n.simnetLocalRpcAirdropped
                    : l10n.simnetLocalRpcCheck,
                detail: receipt.value == null
                    ? null
                    : shortAddress(receipt.value!.signature),
                onPressed:
                    receipt.value?.confirmed != true && !airdropping.value
                    ? () => unawaited(fund())
                    : null,
              ),
            ),
          ],
        ),
      ],
    );
  }

  static String _reason(Object error) => switch (error) {
    SurfpoolRpcException(:final detail) => detail,
    _ => '$error',
  };
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
