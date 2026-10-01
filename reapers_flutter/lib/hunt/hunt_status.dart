import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';

/// The strip of the ledger that is always in view during a fight: what is
/// being fought, what is left of its ward and of the reaper's resolve, and
/// one line of instruction.
class HuntStatus extends HookWidget {
  const HuntStatus({
    super.key,
    required this.title,
    required this.sim,
    required this.light,
    required this.followsDevice,
    required this.banner,
    this.company,
    this.afterword = '',
  });

  /// What is being fought.
  final String title;

  final HuntSim sim;

  /// The husk's own colour.
  final Color light;

  /// Whether the view follows the phone itself rather than a drag.
  final bool followsDevice;

  /// What just happened, shown large for a moment, or null.
  final String? banner;

  /// Whoever else is in the fight, shown under the reaper's own standing.
  final Widget? company;

  /// The line shown once this reaper's own part is over, when the fight
  /// around them is not.
  final String afterword;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final seconds = (sim.fightTicksLeft / HuntRules.tickRate).ceil();

    final prompt = switch (sim.phase) {
      HuntPhase.stalk when sim.steadiness > 0.05 => l10n.huntPromptSteady,
      HuntPhase.stalk =>
        followsDevice ? l10n.huntPromptStalk : l10n.huntPromptStalkDrag,
      HuntPhase.fight => l10n.huntPromptStrike,
      HuntPhase.windup => l10n.huntPromptWindup,
      HuntPhase.shriek => l10n.huntPromptHold,
      HuntPhase.broken when sim.reapBegun => l10n.huntPromptSweep,
      HuntPhase.broken => l10n.huntPromptMark,
      HuntPhase.absorbed || HuntPhase.escaped => afterword,
    };

    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Expanded(
                child: Text(
                  title,
                  style: Veil.display(22),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (sim.phase != HuntPhase.stalk)
                Text(l10n.huntSeconds(seconds), style: Veil.figure(size: 13)),
            ],
          ),
          const SizedBox(height: 8),
          LedgerMeter(
            fraction: sim.ward / sim.wardMax,
            colour: light,
            label: l10n.huntWard,
          ),
          const SizedBox(height: 10),
          Row(
            children: [
              Text(l10n.huntResolve.toUpperCase(), style: Veil.label()),
              const SizedBox(width: 10),
              for (var pip = 0; pip < sim.stats.resolve; pip++)
                Padding(
                  padding: const EdgeInsets.only(right: 6),
                  child: LedgerPip(lit: pip < sim.resolve, colour: Veil.bone),
                ),
              const Spacer(),
              if (sim.cadence > 1)
                Text(
                  l10n.huntCadence(sim.cadence).toUpperCase(),
                  style: Veil.label(color: light),
                ),
            ],
          ),
          if (company case final company?) ...[
            const SizedBox(height: 12),
            company,
          ],
          const SizedBox(height: 30),
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 180),
            child: banner == null
                ? const SizedBox.shrink()
                : Text(
                    banner!,
                    key: ValueKey(banner),
                    textAlign: TextAlign.center,
                    style: Veil.display(40),
                  ),
          ),
          const Spacer(),
          if (sim.phase == HuntPhase.broken)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: LedgerMeter(
                fraction: sim.captureTicksLeft / sim.captureTicks,
                colour: Veil.bone,
                label: l10n.huntReapWindow,
              ),
            ),
          Text(
            prompt,
            textAlign: TextAlign.center,
            style: sim.phase == HuntPhase.shriek
                ? Veil.display(30)
                : Veil.body(size: 16),
          ),
        ],
      ),
    );
  }
}

/// The word to show large on the screen for [event], or null when the event
/// has none.
String? huntBannerFor(AppLocalizations l10n, HuntEvent event) =>
    switch (event) {
      StalkEnded(:final ambush) => ambush ? l10n.huntAmbush : l10n.huntNoticed,
      Flinched(:final forgiven) =>
        forgiven ? l10n.huntSteadfast : l10n.huntFlinched,
      HeldFirm() => l10n.huntHeldFirm,
      Broken() => l10n.huntBroken,
      ReapRecovered() => l10n.huntRecovered,
      StrikeLanded() ||
      StrikeGlanced() ||
      StrikeWhiffed() ||
      SealOpened() ||
      ShriekWindup() ||
      ShriekBegan() ||
      Relocating() ||
      ReapBegan() ||
      HuskAbsorbed() ||
      HuskEscaped() => null,
    };

/// Knocks in the player's hand for [event]: the phone is the blade, so what
/// the blade meets should be felt.
void knockFor(HuntEvent event) {
  final knock = switch (event) {
    StrikeLanded(empowered: true) ||
    ShriekBegan() ||
    Broken() ||
    HuskAbsorbed() ||
    HuskEscaped() => HapticFeedback.heavyImpact,
    StalkEnded() || HeldFirm() => HapticFeedback.mediumImpact,
    StrikeLanded() => HapticFeedback.lightImpact,
    StrikeGlanced() => HapticFeedback.selectionClick,
    Flinched() => HapticFeedback.vibrate,
    StrikeWhiffed() ||
    SealOpened() ||
    ShriekWindup() ||
    Relocating() ||
    ReapBegan() ||
    ReapRecovered() => null,
  };

  if (knock != null) {
    unawaited(knock());
  }
}
