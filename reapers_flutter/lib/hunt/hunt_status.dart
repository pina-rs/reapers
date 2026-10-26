import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/soul_lookups.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';

/// Fresh feedback with its own lifetime, even when two blows say the same
/// thing. Routine strikes use a quieter treatment than phase changes.
final class HuntBanner {
  const HuntBanner({required this.text, this.isStrike = false});

  final String text;
  final bool isStrike;
}

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
    required this.turnSpeed,
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

  /// What just happened, or null. Phase changes speak louder than strikes.
  final HuntBanner? banner;

  /// The phone's current turning speed, in radians per second.
  final double turnSpeed;

  /// Whoever else is in the fight, shown under the reaper's own standing.
  final Widget? company;

  /// The line shown once this reaper's own part is over, when the fight
  /// around them is not.
  final String afterword;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final seconds = (sim.fightTicksLeft / HuntRules.tickRate).ceil();
    final strike = banner?.isStrike ?? false;
    final bannerText =
        strike && sim.phase != HuntPhase.fight && sim.phase != HuntPhase.charge
        ? null
        : banner?.text;
    final reducedMotion = MediaQuery.disableAnimationsOf(context);

    final prompt = switch (sim.phase) {
      HuntPhase.stalk when turnSpeed > HuntRules.startleSpeed =>
        l10n.huntPromptTooFast,
      HuntPhase.stalk when sim.steadiness > 0.05 => l10n.huntPromptSteady,
      HuntPhase.stalk =>
        followsDevice ? l10n.huntPromptStalk : l10n.huntPromptStalkDrag,
      HuntPhase.fight when sim.footing == Footing.hidden && !sim.leaping =>
        l10n.huntPromptHidden,
      HuntPhase.fight when sim.footing == Footing.ledge && !sim.leaping =>
        l10n.huntPromptPerched,
      HuntPhase.fight => l10n.huntPromptStrike,
      HuntPhase.windup => l10n.huntPromptWindup,
      HuntPhase.shriek => l10n.huntPromptHold,
      HuntPhase.charge => l10n.huntPromptCharge,
      HuntPhase.broken when turnSpeed > HuntRules.reapSpeedLimit =>
        l10n.huntPromptReapTooFast,
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
              if (sim.phase != HuntPhase.stalk && !sim.isOver)
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
          if (sim.cadence > 0 && !sim.isOver) ...[
            const SizedBox(height: 6),
            LinearProgressIndicator(
              value: sim.cadenceTicksLeft / HuntRules.cadenceWindowTicks,
              minHeight: 2,
              color: light,
              backgroundColor: Veil.hairline,
              semanticsLabel: l10n.huntCadence(sim.cadence),
            ),
          ],
          if (company case final company?) ...[
            const SizedBox(height: 12),
            company,
          ],
          const SizedBox(height: 30),
          AnimatedSwitcher(
            duration: Duration(milliseconds: reducedMotion ? 0 : 180),
            child: bannerText == null
                ? const SizedBox.shrink()
                : Semantics(
                    key: ValueKey(bannerText),
                    liveRegion: !strike,
                    child: Text(
                      bannerText,
                      textAlign: TextAlign.center,
                      style: strike ? Veil.body(size: 18) : Veil.display(40),
                    ),
                  ),
          ),
          const Spacer(),
          if (sim.phase == HuntPhase.fight || sim.phase == HuntPhase.charge)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: _HuntGauge(
                fraction: 1 - sim.strikeTicksLeft / sim.stats.swingTicks,
                colour: light,
                label: sim.strikeTicksLeft == 0
                    ? l10n.huntBladeReady
                    : l10n.huntBladeRecovering,
              ),
            ),
          if (sim.phase == HuntPhase.shriek)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: _HuntGauge(
                fraction: (turnSpeed / sim.stats.flinchSpeed).clamp(0, 1),
                colour: turnSpeed > sim.stats.flinchSpeed ? Veil.wound : light,
                label: turnSpeed > sim.stats.flinchSpeed
                    ? l10n.huntMotionUnsteady
                    : l10n.huntMotionSteady,
              ),
            ),
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
            style:
                sim.phase == HuntPhase.shriek || sim.phase == HuntPhase.charge
                ? Veil.display(30)
                : Veil.body(size: 16),
          ),
        ],
      ),
    );
  }
}

/// Keeps a movement instruction readable above its meter, including with
/// enlarged type. The narrower ledger label column is for short nouns.
class _HuntGauge extends HookWidget {
  const _HuntGauge({
    required this.fraction,
    required this.colour,
    required this.label,
  });

  final double fraction;
  final Color colour;
  final String label;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Text(label.toUpperCase(), style: Veil.label()),
      const SizedBox(height: 6),
      LinearProgressIndicator(
        value: fraction,
        color: colour,
        backgroundColor: Veil.hairline,
        minHeight: 3,
        borderRadius: BorderRadius.zero,
        semanticsLabel: label,
      ),
    ],
  );
}

/// Feedback for an event the rules have already decided, or null when the
/// event needs no words.
HuntBanner? huntBannerFor(AppLocalizations l10n, HuntEvent event) {
  final text = switch (event) {
    GiftStirred(:final gift) => giftName(l10n, gift),
    StalkEnded(:final ambush) => ambush ? l10n.huntAmbush : l10n.huntNoticed,
    Flinched(:final forgiven) =>
      forgiven ? l10n.huntSteadfast : l10n.huntFlinched,
    HeldFirm() => l10n.huntHeldFirm,
    Parried() => l10n.huntParried,
    Struck() => l10n.huntStruck,
    Broken() => l10n.huntBroken,
    ReapRecovered() => l10n.huntRecovered,
    ArtLanded(hit: false) => l10n.huntArtMissed,
    ArtLanded(art: Art.cinder) => l10n.huntBurning,
    ArtLanded(art: Art.rime) => l10n.huntFrozen,
    ArtLanded(art: Art.gale) => l10n.huntGusted,
    IceShattered() => l10n.huntShattered,
    StrikeLanded(:final damage) => l10n.huntStrikeHit(damage.round()),
    StrikeGlanced() => l10n.huntStrikeGlanced,
    StrikeWhiffed() => l10n.huntStrikeMissed,
    ArtThrown() ||
    Leapt() ||
    SealOpened() ||
    ShriekWindup() ||
    ShriekBegan() ||
    ChargeBegan() ||
    Relocating() ||
    ReapBegan() ||
    HuskAbsorbed() ||
    HuskEscaped() => null,
  };

  if (text == null) {
    return null;
  }

  return HuntBanner(
    text: text,
    isStrike:
        event is StrikeLanded ||
        event is StrikeGlanced ||
        event is StrikeWhiffed,
  );
}

/// Knocks in the player's hand for [event]: the phone is the blade, so what
/// the blade meets should be felt.
void knockFor(HuntEvent event) {
  final knock = switch (event) {
    StrikeLanded(empowered: true) ||
    ShriekBegan() ||
    Parried() ||
    Broken() ||
    HuskAbsorbed() ||
    HuskEscaped() ||
    ArtLanded(hit: true) ||
    IceShattered() => HapticFeedback.heavyImpact,
    StalkEnded() ||
    HeldFirm() ||
    ChargeBegan() ||
    ArtThrown() => HapticFeedback.mediumImpact,
    StrikeLanded() => HapticFeedback.lightImpact,
    StrikeGlanced() => HapticFeedback.selectionClick,
    Flinched() || Struck() => HapticFeedback.vibrate,
    GiftStirred() ||
    ArtLanded() ||
    Leapt() ||
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
