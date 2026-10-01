import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../hunt/start_hunt.dart';
import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../render/reaper_stage.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import 'armory_screen.dart';
import 'codex_screen.dart';
import 'rifts_screen.dart';
import 'vault_screen.dart';

/// Home: the player's reaper, standing in the dark, and everything that can
/// be done from here.
class SanctumScreen extends HookConsumerWidget {
  const SanctumScreen({super.key, required this.player});

  final PlayerState player;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final genome = player.genome;
    final light = Theme.of(context).colorScheme.primary;
    final now = _useClock();
    final sighted = _useNewlySighted(player, now);

    void open(Widget screen) => unawaited(
      Navigator.of(
        context,
      ).push(MaterialPageRoute<void>(builder: (_) => screen)),
    );

    return Scaffold(
      body: Stack(
        fit: StackFit.expand,
        children: [
          Column(
            children: [
              Expanded(
                flex: 11,
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    ReaperStage(
                      genome: genome,
                      weapon: player.weapon!,
                      outfit: player.outfit,
                    ),
                    const IgnorePointer(child: _StageFoot()),
                    IgnorePointer(
                      child: SafeArea(
                        bottom: false,
                        child: Padding(
                          padding: const EdgeInsets.fromLTRB(24, 14, 24, 10),
                          child: _Masthead(player: player),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              Expanded(
                flex: 10,
                child: Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(
                      maxWidth: Veil.contentWidth,
                    ),
                    // The list fades out under the reaper's name as it
                    // scrolls, instead of being cut off by it.
                    child: ShaderMask(
                      blendMode: BlendMode.dstIn,
                      shaderCallback: (bounds) => const LinearGradient(
                        begin: Alignment.topCenter,
                        end: Alignment.bottomCenter,
                        colors: [Color(0x00FFFFFF), Color(0xFFFFFFFF)],
                        stops: [0, 0.045],
                      ).createShader(bounds),
                      child: ListView(
                        padding: const EdgeInsets.fromLTRB(24, 20, 24, 28),
                        children: [
                          LedgerMeter(
                            fraction: Progression.levelProgress(player.xp),
                            colour: light,
                            label: l10n.commonLevel,
                          ),
                          LedgerHeading(
                            l10n.sanctumLoose,
                            trailing:
                                '${player.sightings.length}/${PlayerState.maxSightings}',
                          ),
                          if (player.sightings.isEmpty)
                            Text(
                              l10n.sanctumNothingLoose,
                              style: Veil.body(color: Veil.ash, size: 14),
                            ),
                          for (final sighting in player.sightings)
                            _SightingRow(
                              sighting: sighting,
                              now: now,
                              onHunt: () => startHunt(context, ref, sighting),
                            ),
                          const SizedBox(height: 18),
                          LedgerButton(
                            key: const Key('sanctum-vault'),
                            label: l10n.sanctumVault,
                            detail: l10n.sanctumVaultDetail(
                              player.vault.length,
                            ),
                            onPressed: () => open(const VaultScreen()),
                          ),
                          const SizedBox(height: 8),
                          LedgerButton(
                            key: const Key('sanctum-armory'),
                            label: l10n.sanctumArmory,
                            detail: l10n.armoryAnima(player.anima),
                            onPressed: () => open(const ArmoryScreen()),
                          ),
                          const SizedBox(height: 8),
                          LedgerButton(
                            key: const Key('sanctum-codex'),
                            label: l10n.sanctumCodex,
                            detail: l10n.sanctumCodexDetail(
                              player.codex.length,
                            ),
                            onPressed: () => open(const CodexScreen()),
                          ),
                          const SizedBox(height: 8),
                          LedgerButton(
                            key: const Key('sanctum-rifts'),
                            label: l10n.sanctumRifts,
                            detail: l10n.sanctumRiftsDetail(player.riftsSealed),
                            onPressed: () => open(const RiftsScreen()),
                          ),
                          const SizedBox(height: 8),
                          LedgerButton(
                            key: const Key('sanctum-practice'),
                            label: l10n.sanctumPractice,
                            detail: l10n.sanctumPracticeDetail,
                            quiet: true,
                            onPressed: () => startPracticeHunt(context, ref),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
          SafeArea(
            child: Align(
              alignment: Alignment.topCenter,
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 260),
                transitionBuilder: (child, animation) => SlideTransition(
                  position: Tween(
                    begin: const Offset(0, -1.2),
                    end: Offset.zero,
                  ).animate(animation),
                  child: child,
                ),
                child: sighted == null
                    ? const SizedBox.shrink()
                    : _SightedBanner(
                        key: ValueKey(sighted.id),
                        sighting: sighted,
                        onHunt: () => startHunt(context, ref, sighted),
                      ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// The time, refreshed each second, for the countdown on a surfacing husk.
DateTime _useClock() {
  final now = useState(DateTime.now().toUtc());

  useEffect(() {
    final timer = Timer.periodic(const Duration(seconds: 1), (_) {
      now.value = DateTime.now().toUtc();
    });

    return timer.cancel;
  }, const []);

  return now.value;
}

/// The husk that has just surfaced, for a few seconds after it does.
///
/// This is the in-game "husk sighted" notice. It only fires for a husk that
/// was seen waiting first, so reopening the sanctum does not announce husks
/// the player already knows about.
Sighting? _useNewlySighted(PlayerState player, DateTime now) {
  final waiting = useRef(<int>{});
  final announced = useState<Sighting?>(null);

  useEffect(() {
    for (final sighting in player.sightings) {
      if (!sighting.hasSurfaced(now)) {
        waiting.value.add(sighting.id);
        continue;
      }

      if (waiting.value.remove(sighting.id)) {
        announced.value = sighting;
        unawaited(HapticFeedback.heavyImpact());
      }
    }

    return null;
  }, [now, player.sightings.length]);

  useEffect(() {
    if (announced.value == null) {
      return null;
    }

    final timer = Timer(const Duration(seconds: 5), () {
      announced.value = null;
    });

    return timer.cancel;
  }, [announced.value?.id]);

  final current = announced.value;
  final stillLoose =
      current != null &&
      player.sightings.any((sighting) => sighting.id == current.id);

  return stillLoose ? current : null;
}

/// The reaper's name and standing, set over the stage.
class _Masthead extends HookWidget {
  const _Masthead({required this.player});

  final PlayerState player;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final genome = player.genome;
    final light = Theme.of(context).colorScheme.primary;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text(l10n.sanctumOverline.toUpperCase(), style: Veil.label()),
            const Spacer(),
            Text(
              l10n.commonSol(solAmount(player.lamports)),
              style: Veil.figure(color: Veil.ash, size: 12),
            ),
          ],
        ),
        const Spacer(),
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    className(l10n, genome.reaperClass).toUpperCase(),
                    style: Veil.label(color: light, size: 12),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    genome.name,
                    key: const Key('sanctum-name'),
                    style: Veil.display(30),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            Semantics(
              label: l10n.commonLevel,
              value: '${player.level}',
              excludeSemantics: true,
              child: Text(
                romanNumeral(player.level),
                style: Veil.display(64, color: light, weight: 300),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _StageFoot extends HookWidget {
  const _StageFoot();

  @override
  Widget build(BuildContext context) => const DecoratedBox(
    decoration: BoxDecoration(
      gradient: LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [Color(0x0007080B), Color(0x0007080B), Veil.ink],
        stops: [0, 0.6, 1],
      ),
    ),
  );
}

/// One husk loose in the veil: still surfacing, or sighted and huntable.
class _SightingRow extends HookWidget {
  const _SightingRow({
    required this.sighting,
    required this.now,
    required this.onHunt,
  });

  final Sighting sighting;
  final DateTime now;
  final VoidCallback onHunt;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final surfaced = sighting.hasSurfaced(now);
    final wait = sighting.surfacesAt.difference(now).inSeconds + 1;

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: LedgerButton(
        key: Key('sanctum-sighting-${sighting.id}'),
        label: sighting.nft.title,
        detail: surfaced
            ? '${l10n.sanctumHunt.toUpperCase()}  →'
            : l10n.sanctumSurfacing(wait),
        onPressed: surfaced ? onHunt : null,
      ),
    );
  }
}

/// "Husk sighted": the notice that drops in when a released husk surfaces.
class _SightedBanner extends HookWidget {
  const _SightedBanner({
    super.key,
    required this.sighting,
    required this.onHunt,
  });

  final Sighting sighting;
  final VoidCallback onHunt;

  @override
  Widget build(BuildContext context) {
    final light = Theme.of(context).colorScheme.primary;

    return Semantics(
      liveRegion: true,
      button: true,
      child: GestureDetector(
        key: const Key('sanctum-sighted-banner'),
        onTap: onHunt,
        child: Container(
          margin: const EdgeInsets.fromLTRB(16, 8, 16, 0),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          decoration: BoxDecoration(
            color: Veil.inkRaised,
            border: Border.all(color: light),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              LedgerPip(lit: true, colour: light),
              const SizedBox(width: 12),
              Flexible(
                child: Text(
                  context.l10n.sanctumSightedBanner(sighting.nft.title),
                  style: Veil.body(size: 14),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
