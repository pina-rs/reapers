import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import '../widgets/nft_thumbnail.dart';
import 'hunt_arena.dart';
import 'hunt_status.dart';
import 'scan/room_reader.dart';
import 'scan/scan_overlay.dart';

/// What a hunt is for: which husk, and who is hunting it.
final class HuntBrief {
  const HuntBrief({
    required this.nft,
    required this.seed,
    required this.loadout,
    this.sightingId,
  });

  final Nft nft;

  /// Decides where the husk hides and everything it does.
  final int seed;

  final HuntLoadout loadout;

  /// The loose husk being hunted, or null for a practice hunt that leaves
  /// nothing behind.
  final int? sightingId;

  bool get isPractice => sightingId == null;

  /// The hunt for this husk, in the part of the room [plan] covers.
  HuntSim newSim(HuntPlan plan) => HuntSim(
    seed: seed,
    vessel: nft.collection.vessel,
    ward: nft.rarity.ward,
    stats: HuntStats.derive(loadout),
    plan: plan,
  );
}

/// What a finished hunt came to, as the keeper of the books judged it.
final class HuntReward {
  const HuntReward({
    required this.outcome,
    required this.xp,
    required this.anima,
    required this.feeding,
    required this.signed,
    required this.levelBefore,
    required this.levelAfter,
  });

  /// A practice hunt: the phone's own verdict, and nothing changes hands.
  const HuntReward.practice(this.outcome)
    : xp = 0,
      anima = 0,
      feeding = null,
      signed = false,
      levelBefore = 0,
      levelAfter = 0;

  /// The verdict. For a real hunt this is the keeper's, which is the one
  /// that counts if it ever differs from what the phone saw.
  final HuntOutcome outcome;

  final int xp;
  final int anima;

  /// How well the reaper fed on the husk, or null when it fed on nothing.
  final Feeding? feeding;

  /// Whether the keeper signed a receipt for the absorption. A server does;
  /// a phone keeping its own books cannot.
  final bool signed;

  final int levelBefore;
  final int levelAfter;

  bool get levelledUp => levelAfter > levelBefore;
}

/// One hunt, from the first look around to the reap.
///
/// The reaper is never drawn here: the player is the reaper, and the phone
/// is what it sees through.
class HuntScreen extends HookWidget {
  const HuntScreen({super.key, required this.brief, required this.settle});

  final HuntBrief brief;

  /// Hands the finished hunt to whoever keeps the books, and returns what it
  /// paid. Called once, when the hunt ends.
  final Future<HuntReward> Function(HuntLog log, HuntOutcome outcome) settle;

  @override
  Widget build(BuildContext context) {
    final arena = useMemoized(
      () => HuntArena(vessel: brief.nft.collection.vessel, relics: [brief.nft]),
      [brief],
    );
    final reader = useMemoized(
      () => RoomReader(gait: brief.nft.collection.vessel.gait),
      [brief],
    );
    final loaded = useState(false);
    final begun = useState(false);
    final engaged = useState(false);
    final failure = useState<Object?>(null);
    final reward = useState<HuntReward?>(null);
    final gone = useValueListenable(arena.gone);
    final banner = useState<String?>(null);
    final l10n = context.l10n;

    /// Starts the hunt in the part of the room that has been read.
    void engage(HuntPlan plan) {
      arena.engage(brief.newSim(plan));
      engaged.value = true;
      banner.value = l10n.scanFound;
      unawaited(HapticFeedback.heavyImpact());
    }

    useEffect(() {
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);

      return () => SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    }, const []);

    useEffect(() {
      var cancelled = false;

      Future<void> load() async {
        try {
          await arena.load();

          if (!cancelled) {
            loaded.value = true;
          }
        } catch (error) {
          if (!cancelled) {
            failure.value = error;
          }
        }
      }

      unawaited(load());

      return () {
        cancelled = true;
        arena.dispose();
      };
    }, [arena]);

    // A banner names what just happened, then steps aside.
    useEffect(() {
      if (banner.value == null) {
        return null;
      }

      final timer = Timer(const Duration(milliseconds: 1300), () {
        banner.value = null;
      });

      return timer.cancel;
    }, [banner.value]);

    void onEvents(List<HuntEvent> events) {
      for (final event in events) {
        knockFor(event);
        banner.value = huntBannerFor(l10n, event) ?? banner.value;

        if (event is! HuskAbsorbed && event is! HuskEscaped) {
          continue;
        }

        final controller = arena.controller!;

        unawaited(
          settle(controller.log, controller.sim.outcome!).then(
            (paid) => reward.value = paid,
            onError: (Object error) => failure.value = error,
          ),
        );
      }
    }

    if (failure.value != null) {
      return Scaffold(
        body: LedgerNotice(
          title: l10n.huntFailedTitle,
          body: l10n.huntFailedBody('${failure.value}'),
          action: l10n.commonBack,
          onAction: () => Navigator.of(context).pop(),
        ),
      );
    }

    return Scaffold(
      backgroundColor: Veil.ink,
      body: Stack(
        fit: StackFit.expand,
        children: [
          HuntArenaView(
            arena: arena,
            loaded: loaded.value,
            running: engaged.value,
            onEvents: onEvents,
            // Before the hunt, the phone reads the room as the player looks
            // around. The husk appears once there is somewhere for it to be.
            onIdle: (deltaSeconds) {
              if (!begun.value || engaged.value) {
                return;
              }

              reader.observe(arena.pose, deltaSeconds);

              if (reader.plan case final plan?) {
                engage(plan);
              }
            },
            showSealOrder: brief.loadout.reaperClass.perk == ClassPerk.measure,
          ),
          if (loaded.value && begun.value && !engaged.value)
            ScanOverlay(
              reader: reader,
              pose: () => arena.pose,
              followsDevice: arena.poseSource.followsDevice,
              repaint: arena.frame,
            ),
          // While the room is read there is nothing to strike, so the whole
          // screen is for looking around.
          if (loaded.value && begun.value && !engaged.value)
            _LookAround(arena: arena),
          if (loaded.value && engaged.value)
            SafeArea(
              child: IgnorePointer(
                child: ListenableBuilder(
                  listenable: arena.frame,
                  builder: (context, _) => HuntStatus(
                    title: brief.nft.title,
                    sim: arena.controller!.sim,
                    light: arena.rig.light,
                    followsDevice: arena.poseSource.followsDevice,
                    banner: banner.value,
                  ),
                ),
              ),
            ),
          if (!begun.value)
            _Threshold(
              brief: brief,
              ready: loaded.value,
              onBegin: () {
                begun.value = true;

                // A hunt that plays itself has no room to read.
                if (HuntArena.autoplay) {
                  engage(HuntPlan.open);
                }
              },
            ),
          // The reckoning waits for the husk to finish coming apart: the
          // quiet after the reap is part of the hunt.
          if (reward.value case final paid? when gone)
            _Reckoning(brief: brief, reward: paid),
        ],
      ),
    );
  }
}

/// What stands between the player and the hunt: which husk it is, and the
/// one thing to remember before raising the phone.
class _Threshold extends HookWidget {
  const _Threshold({
    required this.brief,
    required this.ready,
    required this.onBegin,
  });

  final HuntBrief brief;
  final bool ready;
  final VoidCallback onBegin;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;

    return ColoredBox(
      color: Veil.ink.withValues(alpha: 0.82),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              LedgerBack(onPressed: () => Navigator.of(context).pop()),
              const Spacer(),
              Text(
                (brief.isPractice
                        ? l10n.huntPracticeLabel
                        : l10n.huntSightedLabel)
                    .toUpperCase(),
                style: Veil.label(),
              ),
              const SizedBox(height: 10),
              Text(brief.nft.title, style: Veil.display(46)),
              const SizedBox(height: 18),
              Text(l10n.huntSafety, style: Veil.body(color: Veil.ash)),
              const SizedBox(height: 28),
              LedgerButton(
                key: const Key('hunt-begin'),
                label: ready ? l10n.huntBegin : l10n.commonLoading,
                onPressed: ready ? onBegin : null,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The reckoning: how the hunt ended and what it paid.
class _Reckoning extends HookWidget {
  const _Reckoning({required this.brief, required this.reward});

  final HuntBrief brief;
  final HuntReward reward;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final outcome = reward.outcome;
    final title = switch (outcome.grade) {
      ReapGrade.perfect => l10n.huntGradePerfect,
      ReapGrade.clean => l10n.huntGradeClean,
      ReapGrade.ragged => l10n.huntGradeRagged,
      null => l10n.huntEscapedTitle,
    };
    final detail = switch (outcome.escape) {
      EscapeCause.resolve => l10n.huntEscapedResolve,
      EscapeCause.time => l10n.huntEscapedTime,
      EscapeCause.reap => l10n.huntEscapedReap,
      null when brief.isPractice => l10n.huntPracticeDone,
      null => l10n.huntAbsorbed(brief.nft.title),
    };

    return ColoredBox(
      color: Veil.ink.withValues(alpha: 0.88),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: outcome.absorbed && !brief.isPractice
                    ? _AbsorbedPlate(nft: brief.nft)
                    : const SizedBox.shrink(),
              ),
              Text(
                (outcome.absorbed ? l10n.huntReapedLabel : l10n.huntLostLabel)
                    .toUpperCase(),
                style: Veil.label(
                  color: outcome.absorbed ? Veil.ash : Veil.wound,
                ),
              ),
              const SizedBox(height: 10),
              Text(
                title,
                key: const Key('hunt-result'),
                style: Veil.display(52),
              ),
              const SizedBox(height: 14),
              Text(detail, style: Veil.body(color: Veil.ash)),
              if (reward.xp > 0) ...[
                const SizedBox(height: 26),
                LedgerRow(label: l10n.commonExperience, value: '+${reward.xp}'),
                LedgerRow(label: l10n.commonAnima, value: '+${reward.anima}'),
                if (reward.feeding case final feeding?)
                  LedgerRow(
                    label: l10n.huntFeeding,
                    value: feedingName(l10n, feeding),
                    emphasised: feeding == Feeding.feast,
                  ),
                if (reward.feeding != null)
                  LedgerRow(
                    label: l10n.huntReceipt,
                    value: reward.signed
                        ? l10n.huntReceiptSigned
                        : l10n.huntReceiptUnsigned,
                  ),
                if (outcome.ambushed)
                  LedgerRow(
                    label: l10n.huntAmbush,
                    value: l10n.huntAmbushBonus,
                  ),
                if (reward.levelledUp)
                  LedgerRow(
                    label: l10n.commonLevel,
                    value: romanNumeral(reward.levelAfter),
                    emphasised: true,
                  ),
              ],
              const SizedBox(height: 30),
              LedgerButton(
                key: const Key('hunt-leave'),
                label: l10n.huntLeave,
                onPressed: () => Navigator.of(context).pop(),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// What was just absorbed, settling into the dark as a faded plate: the last
/// the player sees of the NFT before it is only a line in the codex.
class _AbsorbedPlate extends HookWidget {
  const _AbsorbedPlate({required this.nft});

  final Nft nft;

  @override
  Widget build(BuildContext context) {
    final settle = useAnimationController(
      duration: const Duration(milliseconds: 900),
    )..forward();
    final eased = CurvedAnimation(parent: settle, curve: Curves.easeOutCubic);

    return Center(
      child: FadeTransition(
        opacity: Tween(begin: 0.0, end: 0.6).animate(eased),
        child: ScaleTransition(
          scale: Tween(begin: 1.12, end: 1.0).animate(eased),
          child: NftThumbnail(nft: nft, size: 168),
        ),
      ),
    );
  }
}

/// Lets a finger look around where the phone cannot feel itself turn, while
/// the room is being read and there is nothing yet to strike.
class _LookAround extends HookWidget {
  const _LookAround({required this.arena});

  final HuntArena arena;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final radiansPerPoint = huntFieldOfView / constraints.biggest.height * 3;

      return GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanUpdate: (details) => arena.poseSource.drag(
          details.delta.dx * radiansPerPoint,
          -details.delta.dy * radiansPerPoint,
        ),
      );
    },
  );
}
