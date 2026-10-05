import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../hunt/hunt_arena.dart';
import '../hunt/hunt_status.dart';
import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../src/backend/game_backend.dart';
import '../src/backend/rift_session.dart';
import 'place/rift_placer.dart';
import 'place/whereabouts.dart';
import 'place/location_tracker.dart';
import 'place/location_notice.dart';
import '../sound/score.dart';
import '../sound/soundtrack.dart';
import '../src/state/player.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';

/// One rift, from the circle gathering to the last cut.
///
/// Each reaper fights the husk through their own phone, exactly as in a lone
/// hunt. What the circle shares is the ward, the moments it has to hold
/// still, and whether enough of it cut in the end.
class RiftScreen extends HookConsumerWidget {
  const RiftScreen({super.key, required this.site, required this.session});

  final RiftSite site;

  /// This reaper's seat in the rift. The screen leaves it when it closes.
  final RiftSession session;

  /// A reaper who joins a fight already under way plays at most this much of
  /// it standing still to fall into step with the circle.
  static const _maxCatchUpTicks = 2 * HuntRules.tickRate;

  /// Ticks between one report of this reaper's standing and the next.
  static const _reportEveryTicks = 6;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    // Who the reaper was on the way in. Sealing the rift changes the player,
    // and the fight must not change under them when it does.
    final entered = useMemoized(
      () => ref.read(playerProvider).requireValue,
      const [],
    );
    final arena = useMemoized(
      () => HuntArena(vessel: site.vessel, relics: site.relics),
      const [],
    );
    final loaded = useState(false);
    final frame = useState<RiftFrame?>(null);
    final ended = useState<RiftFrame?>(null);
    final failure = useState<Object?>(null);
    final banner = useState<String?>(null);
    final seat = useState<RiftSeat?>(null);
    final lastReportTick = useRef(-_reportEveryTicks);
    final reportedEnd = useRef(false);
    // Where the rift stands for this reaper, and where the husk was last
    // put because of it.
    final placer = useMemoized(RiftPlacer.new, const []);
    final tracker = useMemoized(
      () => LocationTracker(
        source: ref.read(whereaboutsProvider),
        share: ref.read(gameBackendProvider).locate,
      ),
      const [],
    );
    useListenable(tracker);
    placer.fix = tracker.fix;
    final lifecycle = useAppLifecycleState();
    useEffect(() {
      if (lifecycle == null || lifecycle == AppLifecycleState.resumed) {
        tracker.start();
      } else {
        tracker.stop();
      }
      return null;
    }, [lifecycle, tracker]);
    useEffect(() => tracker.dispose, [tracker]);
    final placed = useRef<RiftSighting?>(null);
    final everPlaced = useRef(false);

    useEffect(() {
      SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);

      return () => SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    }, const []);

    // A rift has its own music, held back while the circle gathers.
    final soundtrack = ref.watch(soundtrackProvider);

    useEffect(() {
      soundtrack.play(Score.rift, mix: Mix.waiting);

      return () => soundtrack.play(Score.sanctum);
    }, [soundtrack]);

    useEffect(() {
      var closed = false;

      Future<void> load() async {
        try {
          await arena.load();

          if (!closed) {
            loaded.value = true;
          }
        } catch (error) {
          if (!closed) {
            failure.value = error;
          }
        }
      }

      /// The rift is over: read what it paid before showing how it ended.
      Future<void> end(RiftFrame last) async {
        try {
          if (last.phase == RiftPhase.sealed) {
            await ref.read(playerProvider.notifier).refresh();
          }

          if (!closed) {
            ended.value = last;
          }
        } catch (error) {
          if (!closed) {
            failure.value = error;
          }
        }
      }

      void onFrame(RiftFrame next) {
        frame.value = next;

        final fighting =
            next.phase == RiftPhase.fight || next.phase == RiftPhase.reap;

        if (seat.value == null) {
          final standing = placer.observe(
            next,
            facing: arena.pose.forward.yaw,
            north: arena.poseSource.north,
          );

          if (standing != null) {
            session.report(
              RiftReport(ready: next.you.ready, standing: standing),
            );
          }
        }

        if (fighting && seat.value == null) {
          // Once the fight is on the husk stays where it was last seen:
          // moving it would turn the reaper's own blade away from it.
          final fight = RiftSeat(
            seed: next.seed,
            site: site,
            wardMax: next.wardMax,
            loadout: entered.loadout,
            sighting: placer.sighting,
          )..catchUp(math.min(next.tick, _maxCatchUpTicks));

          arena.engage(fight.sim);
          seat.value = fight;
        } else if (seat.value == null &&
            (!everPlaced.value || !identical(placed.value, placer.sighting))) {
          // Nothing is being fought yet, but the husk already stands where
          // it will be fought.
          everPlaced.value = true;
          placed.value = placer.sighting;
          arena.rest(
            HuntSim.rift(
              seed: next.seed,
              vessel: site.vessel,
              ward: 1,
              stats: HuntStats.derive(entered.loadout),
              sighting: placer.sighting,
            ).pose,
          );
        }

        seat.value?.sync(ward: next.ward, counted: next.you.dealt);

        if (next.phase.isOver) {
          unawaited(end(next));
        }
      }

      final listening = session.frames.listen(
        onFrame,
        onError: (Object error) {
          if (!closed) {
            failure.value = error;
          }
        },
        onDone: () {
          if (!closed && frame.value?.phase.isOver != true) {
            failure.value = const BackendUnavailable('Rift stream ended early');
          }
        },
      );

      unawaited(load());

      return () {
        closed = true;
        unawaited(listening.cancel());
        session.leave();
        arena.dispose();
      };
    }, const []);

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
      final fight = seat.value;

      if (fight == null) {
        return;
      }

      soundtrack.shape(huntMix(fight.sim));

      fight.tally(events);

      for (final event in events) {
        knockFor(event);
        banner.value = huntBannerFor(l10n, event) ?? banner.value;
      }

      final due =
          fight.sim.tick - lastReportTick.value >= _reportEveryTicks ||
          fight.sim.isOver;

      if (!due || reportedEnd.value) {
        return;
      }

      lastReportTick.value = fight.sim.tick;
      reportedEnd.value = fight.sim.isOver;
      session.report(fight.report);
    }

    if (failure.value != null) {
      return Scaffold(
        body: LedgerNotice(
          title: l10n.riftLostTitle,
          body: l10n.riftLostBody(_reason(l10n, failure.value!)),
          action: l10n.commonBack,
          onAction: () => Navigator.of(context).pop(),
        ),
      );
    }

    final current = frame.value;
    final fight = seat.value;

    return Scaffold(
      backgroundColor: Veil.ink,
      body: Stack(
        fit: StackFit.expand,
        children: [
          HuntArenaView(
            arena: arena,
            loaded: loaded.value,
            running: fight != null,
            onEvents: onEvents,
            showSealOrder:
                entered.loadout.reaperClass.perk == ClassPerk.measure,
          ),
          if (loaded.value &&
              fight != null &&
              current != null &&
              ended.value == null)
            SafeArea(
              child: IgnorePointer(
                child: ListenableBuilder(
                  listenable: arena.frame,
                  builder: (context, _) => HuntStatus(
                    title: site.title,
                    sim: fight.sim,
                    light: arena.rig.light,
                    followsDevice: arena.poseSource.followsDevice,
                    banner: banner.value,
                    company: _Circle(frame: current, light: arena.rig.light),
                    afterword: switch (fight.sim.outcome) {
                      HuntOutcome(grade: _?) => l10n.riftCutMade,
                      HuntOutcome(escape: EscapeCause.resolve) =>
                        l10n.riftCastOut,
                      HuntOutcome(escape: EscapeCause.reap) =>
                        l10n.riftCutMissed,
                      _ => '',
                    },
                  ),
                ),
              ),
            ),
          if (fight == null)
            _Gathering(
              site: site,
              frame: current,
              location: LocationNotice(tracker: tracker),
              placing: _placing(
                l10n,
                anchor: current?.anchor,
                fix: placer.fix,
                north: arena.poseSource.north,
                stillLooking: placer.stillLooking,
              ),
              onReady: () => session.report(const RiftReport(ready: true)),
            ),
          if (ended.value case final last?)
            _Reckoning(
              site: site,
              frame: last,
              levelBefore: entered.level,
              levelAfter: ref.watch(playerProvider).requireValue.level,
            ),
        ],
      ),
    );
  }

  /// Where the rift stands for this reaper, in words for the lobby.
  static String _placing(
    AppLocalizations l10n, {
    required GeoPoint? anchor,
    required GeoFix? fix,
    required double? north,
    required bool stillLooking,
  }) {
    if (fix == null) {
      return stillLooking ? l10n.riftFinding : l10n.riftUnplaced;
    }

    // Knowing where it is, a phone with no compass still cannot say which
    // way the rift lies from there.
    if (north == null) {
      return l10n.riftNoCompass;
    }

    // What the phone measured, for whoever is testing how well it knows.
    final measured = kReleaseMode
        ? ''
        : ' ${l10n.riftMeasured(fix.accuracy.round(), fix.at.latitude.toStringAsFixed(5), fix.at.longitude.toStringAsFixed(5))}';

    if (anchor == null) {
      return '${l10n.riftFinding}$measured';
    }

    final metres = fix.at.distanceTo(anchor);

    return metres > RiftRules.sightMetres
        ? '${l10n.riftStandsFar}$measured'
        : '${l10n.riftStands(metres.round())}$measured';
  }

  /// Why the rift was lost, in the player's words where there are any.
  static String _reason(AppLocalizations l10n, Object failure) =>
      switch (failure) {
        RuleBreach(:final refusal) => refusalMessage(l10n, refusal),
        BackendUnavailable() => l10n.commonUnreachable,
        _ => '$failure',
      };
}

/// The circle during the fight: who is in it, how much of the ward each has
/// taken, and who has made their cut.
class _Circle extends HookWidget {
  const _Circle({required this.frame, required this.light});

  final RiftFrame frame;
  final Color light;

  @override
  Widget build(BuildContext context) {
    final fairShare = frame.wardMax / RiftRules.circle;
    final reaping = frame.phase == RiftPhase.reap;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            for (final (index, member) in frame.members.indexed) ...[
              if (index > 0) const SizedBox(width: 10),
              Expanded(
                child: _CircleSeat(
                  member: member,
                  you: index == frame.seat,
                  light: light,
                  share: fairShare <= 0
                      ? 0
                      : (member.dealt / fairShare).clamp(0.0, 1.0),
                ),
              ),
            ],
          ],
        ),
        if (reaping)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              context.l10n.riftCuts(frame.cuts, frame.cutsNeeded).toUpperCase(),
              style: Veil.label(color: Veil.bone),
            ),
          ),
      ],
    );
  }
}

class _CircleSeat extends HookWidget {
  const _CircleSeat({
    required this.member,
    required this.you,
    required this.light,
    required this.share,
  });

  final RiftMember member;
  final bool you;
  final Color light;

  /// How much of a fair share of the ward this member has taken, 0 to 1.
  final double share;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final name = you ? l10n.riftSeatYou : member.name;
    final colour = !member.standing
        ? Veil.wound
        : you
        ? Veil.bone
        : Veil.ash;

    return Semantics(
      label: '$name, ${(share * 100).round()}%',
      excludeSemantics: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            name.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.clip,
            softWrap: false,
            style: Veil.label(color: colour, size: 9).copyWith(
              decoration: member.standing ? null : TextDecoration.lineThrough,
            ),
          ),
          const SizedBox(height: 4),
          Stack(
            children: [
              Container(height: 2, color: Veil.hairline),
              FractionallySizedBox(
                widthFactor: member.cut != null ? 1 : share,
                child: Container(
                  height: 2,
                  color: member.cut != null ? Veil.bone : light,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// Before the fight: where this is, who has come, and the one thing each
/// reaper has to say before it starts.
class _Gathering extends HookWidget {
  const _Gathering({
    required this.site,
    required this.frame,
    required this.placing,
    required this.location,
    required this.onReady,
  });

  final RiftSite site;

  /// Where the rift stands for this reaper, in words.
  final String placing;
  final Widget location;

  /// Null until the rift has answered.
  final RiftFrame? frame;

  final VoidCallback onReady;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;
    final frame = this.frame;
    final members = frame?.members ?? const <RiftMember>[];
    final ready = frame != null && frame.you.ready;
    final seconds = frame == null
        ? 0
        : (frame.ticksLeft / HuntRules.tickRate).ceil();

    return DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            Veil.ink.withValues(alpha: 0.35),
            Veil.ink.withValues(alpha: 0.92),
          ],
          stops: const [0.2, 0.62],
        ),
      ),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(28, 16, 28, 28),
          child: CustomScrollView(
            slivers: [
              SliverFillRemaining(
                hasScrollBody: false,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    LedgerBack(
                      onPressed: () => Navigator.of(context).pop(),
                      label: l10n.riftLeave,
                    ),
                    const Spacer(),
                    Text(l10n.riftLabel.toUpperCase(), style: Veil.label()),
                    const SizedBox(height: 8),
                    Text(site.title, style: Veil.display(44)),
                    const SizedBox(height: 10),
                    Text(
                      l10n.huntSafety,
                      style: Veil.body(color: Veil.ash, size: 14),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      placing,
                      key: const Key('rift-placing'),
                      style: Veil.body(size: 14),
                    ),
                    location,
                    LedgerHeading(
                      l10n.riftCircle,
                      trailing: '${members.length}/${RiftRules.circle}',
                    ),
                    for (var place = 0; place < RiftRules.circle; place++)
                      if (place < members.length)
                        _SeatRow(
                          name: place == frame!.seat
                              ? l10n.riftSeatYou
                              : members[place].name,
                          detail: members[place].shade
                              ? l10n.riftSeatShade
                              : '${className(l10n, members[place].reaperClass)} '
                                    '${romanNumeral(members[place].level)}',
                          status: members[place].ready
                              ? l10n.riftSeatReady
                              : l10n.riftSeatWaiting,
                          lit: members[place].ready,
                          light: light,
                        )
                      else
                        _SeatRow(
                          name: l10n.riftSeatEmpty,
                          detail: l10n.riftSeatEmptyDetail,
                          status: '',
                          lit: false,
                          light: light,
                          empty: true,
                        ),
                    const SizedBox(height: 18),
                    LedgerButton(
                      key: const Key('rift-ready'),
                      label: switch (frame) {
                        null => l10n.riftEntering,
                        RiftFrame(countingDown: true) => l10n.riftBegins(
                          seconds,
                        ),
                        _ when ready => l10n.riftWaiting,
                        _ => l10n.riftStandReady,
                      },
                      onPressed: frame == null || ready ? null : onReady,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SeatRow extends HookWidget {
  const _SeatRow({
    required this.name,
    required this.detail,
    required this.status,
    required this.lit,
    required this.light,
    this.empty = false,
  });

  final String name;
  final String detail;
  final String status;
  final bool lit;
  final Color light;
  final bool empty;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: Row(
      children: [
        LedgerPip(lit: lit, colour: light),
        const SizedBox(width: 14),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Veil.body(size: 15, color: empty ? Veil.ash : Veil.bone),
              ),
              const SizedBox(height: 2),
              Text(detail, style: Veil.body(color: Veil.ash, size: 12)),
            ],
          ),
        ),
        Text(
          status.toUpperCase(),
          style: Veil.label(color: lit ? light : Veil.ash, size: 10),
        ),
      ],
    ),
  );
}

/// The reckoning: whether the rift was shut, what each of the circle did,
/// and what it paid this reaper.
class _Reckoning extends HookWidget {
  const _Reckoning({
    required this.site,
    required this.frame,
    required this.levelBefore,
    required this.levelAfter,
  });

  final RiftSite site;
  final RiftFrame frame;
  final int levelBefore;
  final int levelAfter;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final sealed = frame.phase == RiftPhase.sealed;
    var dealt = 0.0;

    for (final member in frame.members) {
      dealt += member.dealt;
    }

    return ColoredBox(
      color: Veil.ink.withValues(alpha: 0.9),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Spacer(),
              Text(
                (sealed ? l10n.riftSealedLabel : l10n.riftFailedLabel)
                    .toUpperCase(),
                style: Veil.label(color: sealed ? Veil.ash : Veil.wound),
              ),
              const SizedBox(height: 10),
              Text(
                sealed ? l10n.riftSealedTitle : l10n.riftFailedTitle,
                key: const Key('rift-result'),
                style: Veil.display(46),
              ),
              const SizedBox(height: 14),
              Text(
                sealed ? l10n.riftSealedBody(site.title) : l10n.riftFailedBody,
                style: Veil.body(color: Veil.ash),
              ),
              LedgerHeading(l10n.riftCircle),
              for (final (index, member) in frame.members.indexed)
                LedgerRow(
                  label: index == frame.seat ? l10n.riftSeatYou : member.name,
                  value:
                      '${l10n.riftShare(dealt <= 0 ? 0 : (member.dealt / dealt * 100).round())}'
                      ' · '
                      '${switch (member) {
                        RiftMember(cut: _?) => l10n.riftMemberCut,
                        RiftMember(standing: false) => l10n.riftMemberCastOut,
                        _ => l10n.riftMemberNoCut,
                      }}',
                  emphasised: index == frame.seat,
                ),
              if (frame.xp > 0) ...[
                const SizedBox(height: 18),
                LedgerRow(label: l10n.commonExperience, value: '+${frame.xp}'),
                LedgerRow(label: l10n.commonAnima, value: '+${frame.anima}'),
                if (levelAfter > levelBefore)
                  LedgerRow(
                    label: l10n.commonLevel,
                    value: romanNumeral(levelAfter),
                    emphasised: true,
                  ),
              ],
              const SizedBox(height: 28),
              LedgerButton(
                key: const Key('rift-leave'),
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
