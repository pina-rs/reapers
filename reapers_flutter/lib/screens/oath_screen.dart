import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../render/reaper_look.dart';
import '../render/reaper_stage.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/attempt.dart';
import '../widgets/ledger.dart';
import '../widgets/reaper_stats.dart';

/// Where a player is bound to a reaper they did not choose.
///
/// Three moments: the oath is read, the shuffle turns, and the reaper it
/// drew steps out of the dark.
class OathScreen extends HookConsumerWidget {
  const OathScreen({super.key});

  /// The shuffle turns for at least this long, however fast the keeper
  /// answers, so the draw has weight.
  static const _suspense = Duration(milliseconds: 1900);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final turning = useState(false);
    final player = ref.watch(playerProvider).value;
    final drawn = player != null && player.oathTaken && !turning.value;

    Future<void> speak() async {
      turning.value = true;

      final bound = await attempt(
        context,
        () => Future.wait([
          ref.read(playerProvider.notifier).takeOath(),
          Future<void>.delayed(_suspense),
        ]),
      );

      if (bound) {
        unawaited(HapticFeedback.heavyImpact());
      }

      turning.value = false;
    }

    if (drawn) {
      return _Drawn(player: player);
    }

    return Scaffold(
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
            child: Padding(
              padding: const EdgeInsets.fromLTRB(28, 8, 28, 28),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (!turning.value)
                    LedgerBack(onPressed: () => Navigator.of(context).pop())
                  else
                    const SizedBox(height: 48),
                  const Spacer(),
                  Text(l10n.oathOverline.toUpperCase(), style: Veil.label()),
                  const SizedBox(height: 12),
                  if (turning.value)
                    const _Turning()
                  else ...[
                    Text(l10n.oathTitle, style: Veil.display(44)),
                    const SizedBox(height: 18),
                    Text(l10n.oathBody, style: Veil.body(color: Veil.ash)),
                  ],
                  const SizedBox(height: 34),
                  LedgerButton(
                    key: const Key('oath-speak'),
                    label: turning.value ? l10n.oathTurning : l10n.oathSpeak,
                    onPressed: turning.value ? null : speak,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// While the shuffle turns: names it might land on, passing too fast to
/// hold on to.
class _Turning extends HookWidget {
  const _Turning();

  @override
  Widget build(BuildContext context) {
    final index = useState(0);

    useEffect(() {
      final timer = Timer.periodic(const Duration(milliseconds: 90), (_) {
        index.value = (index.value + 7) % reaperGivenNames.length;
      });

      return timer.cancel;
    }, const []);

    return Text(
      reaperGivenNames[index.value],
      style: Veil.display(64, color: Veil.ash),
    );
  }
}

/// The reaper the shuffle drew, shown for the first time.
class _Drawn extends HookWidget {
  const _Drawn({required this.player});

  final PlayerState player;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final genome = player.genome;
    final light = reaperLight(genome);
    final reveal = useAnimationController(
      duration: const Duration(milliseconds: 2600),
    )..forward();

    Widget rising(double from, Widget child) => FadeTransition(
      opacity: CurvedAnimation(
        parent: reveal,
        curve: Interval(from, (from + 0.35).clamp(0, 1), curve: Curves.easeOut),
      ),
      child: child,
    );

    return Theme(
      data: Veil.theme(light),
      child: Scaffold(
        body: Stack(
          fit: StackFit.expand,
          children: [
            FadeTransition(
              opacity: CurvedAnimation(
                parent: reveal,
                curve: const Interval(0, 0.5, curve: Curves.easeIn),
              ),
              child: ReaperStage(
                genome: genome,
                weapon: player.weapon!,
                outfit: player.outfit,
                turnsOnItsOwn: true,
              ),
            ),
            const IgnorePointer(child: _Scrim()),
            SafeArea(
              child: Center(
                child: ConstrainedBox(
                  constraints: const BoxConstraints(
                    maxWidth: Veil.contentWidth,
                  ),
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(28, 24, 28, 28),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        rising(
                          0.3,
                          Text(
                            l10n.oathDrawnOverline.toUpperCase(),
                            style: Veil.label(),
                          ),
                        ),
                        const Spacer(),
                        rising(
                          0.45,
                          Text(
                            className(l10n, genome.reaperClass).toUpperCase(),
                            style: Veil.label(color: light, size: 13),
                          ),
                        ),
                        const SizedBox(height: 8),
                        rising(
                          0.5,
                          Text(
                            genome.name,
                            key: const Key('oath-name'),
                            style: Veil.display(40),
                          ),
                        ),
                        const SizedBox(height: 10),
                        rising(
                          0.58,
                          Text(
                            classLine(l10n, genome.reaperClass),
                            style: Veil.body(color: Veil.ash),
                          ),
                        ),
                        const SizedBox(height: 14),
                        rising(
                          0.64,
                          Column(
                            children: [
                              ReaperStatRows(stats: genome.reaperClass.stats),
                              LedgerRow(
                                label: perkName(l10n, genome.reaperClass.perk),
                                value: weaponName(
                                  l10n,
                                  genome.reaperClass.weapon,
                                ),
                              ),
                              if (genome.omen != Omen.none)
                                LedgerRow(
                                  label: l10n.oathOmen,
                                  value: omenName(l10n, genome.omen),
                                  emphasised: true,
                                ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 22),
                        rising(
                          0.65,
                          LedgerButton(
                            key: const Key('oath-enter'),
                            label: l10n.oathEnter,
                            onPressed: () => Navigator.of(
                              context,
                            ).popUntil((route) => route.isFirst),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Darkens the foot of a stage so the ledger over it stays readable.
class _Scrim extends HookWidget {
  const _Scrim();

  @override
  Widget build(BuildContext context) => const DecoratedBox(
    decoration: BoxDecoration(
      gradient: LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [Color(0x0007080B), Color(0x0007080B), Color(0xF207080B)],
        stops: [0, 0.42, 0.78],
      ),
    ),
  );
}
