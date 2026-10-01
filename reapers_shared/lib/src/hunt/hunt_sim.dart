import 'dart:math' as math;

import '../reaper_class.dart';
import '../rift/rift_rules.dart';
import '../seeded_random.dart';
import '../vec3.dart';
import '../vessel.dart';
import 'hunt_event.dart';
import 'hunt_input.dart';
import 'hunt_plan.dart';
import 'hunt_stats.dart';
import 'husk_pose.dart';

/// One encounter between a reaper and a husk, advanced a tick at a time.
///
/// The simulation owns every result: where the husk is, whether a tap struck
/// a seal, whether a reaper flinched and how clean the reap was. It reads
/// nothing but its seed and the [HuntInput] of each tick, so the same seed
/// and the same inputs always end the same way, on a phone or on the server.
final class HuntSim {
  /// A lone hunt: the husk hides somewhere around the reaper, and its ward
  /// is theirs alone to break.
  HuntSim({
    required int seed,
    required Vessel vessel,
    required int ward,
    required HuntStats stats,
    HuntPlan plan = HuntPlan.open,
  }) : this._(
         seed: seed,
         vessel: vessel,
         ward: ward.toDouble(),
         stats: stats,
         plan: plan,
         inRift: false,
       );

  /// One reaper's place in a rift.
  ///
  /// The husk stands dead ahead and the fight starts at once: there is
  /// nothing to stalk. Its [ward] belongs to the whole circle, so whoever
  /// runs the rift keeps this reaper's view of it current with [shareWard].
  /// Every reaper given the same [seed] hears the husk shriek at the same
  /// moments, and there is one reap window, not two.
  HuntSim.rift({
    required int seed,
    required Vessel vessel,
    required double ward,
    required HuntStats stats,
  }) : this._(
         seed: seed,
         vessel: vessel,
         ward: ward,
         stats: stats,
         plan: HuntPlan.open,
         inRift: true,
       );

  HuntSim._({
    required this.seed,
    required this.vessel,
    required this.ward,
    required this.stats,
    required this.plan,
    required this.inRift,
  }) : wardMax = ward,
       resolve = stats.resolve,
       reapAttemptsLeft = inRift ? 1 : HuntRules.reapAttempts,
       _schedule = SeededRandom(mixSeed([seed, 0x5EA1])),
       _shrieks = SeededRandom(mixSeed([seed, 0x5C81])),
       _path = SeededRandom(mixSeed([seed, 0x9A7B])),
       _openUntil = List.filled(vessel.seals.length, 0),
       _shutUntil = List.filled(vessel.seals.length, 0),
       _openedOrder = List.filled(vessel.seals.length, 0) {
    if (!plan.isPlayable) {
      throw ArgumentError.value(plan, 'plan', 'is too narrow to hunt in');
    }

    _reapDirection = _path.nextChance(0.5) ? 1 : -1;
    _bobPhase = _path.nextRange(0, 2 * math.pi);
    _range = _path.nextRange(vessel.nearMetres, vessel.farMetres);
    _height = _restingHeight();
    _nextAmbleTick = _restTicks();

    if (inRift) {
      // A rift's husk stands dead ahead, facing the circle.
      _bearing = 0;
      _heading = math.pi;
      phase = HuntPhase.fight;
      _scheduleShriek();

      return;
    }

    // With the whole circle to hide in, a husk starts somewhere the reaper
    // has to turn to find. Inside a plan it starts anywhere in it.
    _bearing = plan.isWholeCircle
        ? (_path.nextChance(0.5) ? 1 : -1) * radians(_path.nextRange(55, 180))
        : plan.clamp(plan.centre + _path.nextRange(-0.8, 0.8) * plan.halfWidth);
    _heading = _path.nextRange(-math.pi, math.pi);
  }

  /// Replays [log] from the start and returns how the hunt ended, or null
  /// when the log stops before the hunt does.
  static HuntOutcome? replay({
    required int seed,
    required Vessel vessel,
    required int ward,
    required HuntStats stats,
    required HuntLog log,
  }) {
    if (!log.plan.isPlayable) {
      return null;
    }

    final sim = HuntSim(
      seed: seed,
      vessel: vessel,
      ward: ward,
      stats: stats,
      plan: log.plan,
    );

    for (final input in log.inputs) {
      sim.step(input);

      if (sim.isOver) {
        break;
      }
    }

    return sim.outcome;
  }

  final int seed;

  final Vessel vessel;

  final HuntStats stats;

  /// Where the husk may go.
  final HuntPlan plan;

  /// Whether this is one reaper's place in a rift rather than a lone hunt.
  final bool inRift;

  /// The ward the husk started with.
  final double wardMax;

  final SeededRandom _schedule;

  /// Decides when the husk shrieks, and nothing else. Kept apart from
  /// [_schedule], which a reaper's own strikes advance, so that everyone in
  /// a rift is told to hold still at the same moment.
  final SeededRandom _shrieks;

  /// Decides where the husk goes, and nothing else.
  final SeededRandom _path;

  // Movement -----------------------------------------------------------

  /// Where the husk is: its bearing round the player, how far away it is
  /// along the floor, and how high the middle of its body is above the
  /// player's eye.
  late double _bearing;
  late double _range;
  late double _height;

  /// The way its body faces.
  late double _heading;

  late final double _bobPhase;

  /// The move under way: where it started, how far it goes, when it began
  /// and how long it takes. [_moveStart] is negative while the husk rests.
  double _fromBearing = 0;
  double _fromRange = 0;
  double _fromHeight = 0;
  double _byBearing = 0;
  double _byRange = 0;
  double _byHeight = 0;
  int _moveStart = -1;
  int _moveTicks = 1;

  int _nextAmbleTick = 0;
  int _nextBoltTick = -1;

  // Seals --------------------------------------------------------------

  final List<int> _openUntil;
  final List<int> _shutUntil;
  final List<int> _openedOrder;
  int _openedCount = 0;

  // Strikes ------------------------------------------------------------

  int _nextStrikeTick = 0;
  Aim? _bufferedTap;
  int _bufferedAt = 0;
  int _lastHitTick = -HuntRules.cadenceWindowTicks - 1;
  bool _ambushStrikeReady = false;
  bool _chosenBlowReady = false;

  // Shriek -------------------------------------------------------------

  int _phaseEndTick = 0;
  int _shriekStartTick = 0;
  int _nextShriekTick = 0;
  bool _flinchedThisShriek = false;
  bool _steadfastSpent = false;
  int _fightTicks = 0;

  // Reap ---------------------------------------------------------------

  late final int _reapDirection;
  double _arcYaw = 0;
  double _arcPitch = 0;
  int _captureEndTick = 0;
  int _bonusCaptureTicks = 0;
  double _deviationSum = 0;
  int _reapTicks = 0;
  int _fastTicks = 0;

  // Readable state -----------------------------------------------------

  /// Ticks advanced so far.
  int tick = 0;

  HuntPhase phase = HuntPhase.stalk;

  /// What is left of the husk's ward.
  double ward;

  /// Flinches the reaper can still take.
  int resolve;

  /// How alarmed an unaware husk is, from 0 to 1. At 1 it notices.
  double awareness = 0;

  /// Ticks the reaper has spent steady on the husk during the stalk.
  int steadyTicks = 0;

  /// Whether the fight opened with an ambush.
  bool ambushed = false;

  /// Seal hits in a row.
  int cadence = 0;

  /// Unfinished reap windows the husk will still allow.
  int reapAttemptsLeft;

  /// Whether the aim has reached the start of the arc.
  bool reapBegun = false;

  /// How far along the arc the cut has travelled, from 0 to 1.
  double reapProgress = 0;

  /// How the hunt ended, or null while it is still running.
  HuntOutcome? outcome;

  bool get isOver => outcome != null;

  /// How close the reaper is to springing an ambush, from 0 to 1.
  double get steadiness =>
      (steadyTicks / HuntRules.ambushTicks).clamp(0.0, 1.0);

  /// Ticks of fighting left before the husk escapes.
  int get fightTicksLeft => math.max(0, _fightLimitTicks - _fightTicks);

  int get _fightLimitTicks =>
      inRift ? RiftRules.fightLimitTicks : HuntRules.fightLimitTicks;

  /// Sets the ward to what the whole circle has left of it.
  ///
  /// Only a rift's ward is shared, and only while it is still being fought
  /// over: once it has broken for this reaper it stays broken.
  void shareWard(double shared) {
    if (!inRift) {
      throw StateError('Only the ward of a rift is shared');
    }

    if (_fighting) {
      ward = math.max(0, shared);
    }
  }

  /// Ticks left in the current reap window.
  int get captureTicksLeft => math.max(0, _captureEndTick - tick);

  /// The length of the current reap window.
  int get captureTicks => stats.captureTicks + _bonusCaptureTicks;

  /// How far through the warning or the shriek the husk is, from 0 to 1.
  double get shriekProgress {
    final length = switch (phase) {
      HuntPhase.windup => HuntRules.windupTicks,
      HuntPhase.shriek => HuntRules.shriekTicks,
      _ => 0,
    };

    if (length == 0) {
      return 0;
    }

    return (1 - (_phaseEndTick - tick) / length).clamp(0.0, 1.0);
  }

  /// Where the husk is right now.
  HuskPose get pose {
    // A husk in the air never hangs quite still.
    final bob = vessel.gait == Gait.air
        ? 0.05 * math.sin(0.9 * tick / HuntRules.tickRate + _bobPhase)
        : 0.0;

    return HuskPose(
      vessel: vessel,
      position: Vec3(
        math.sin(_bearing) * _range,
        _height + bob,
        math.cos(_bearing) * _range,
      ),
      heading: _heading,
    );
  }

  /// How fast the husk is covering ground right now, in metres a second.
  /// Zero while it rests.
  double get speed {
    if (_moveStart < 0) {
      return 0;
    }

    final meanRange = _fromRange + _byRange / 2;
    final length = math.sqrt(
      math.pow(meanRange * _byBearing, 2) +
          math.pow(_byRange, 2) +
          math.pow(_byHeight, 2),
    );

    return length / (_moveTicks / HuntRules.tickRate);
  }

  /// Whether seal [index] can be struck right now.
  bool sealOpen(int index) =>
      _fighting && tick < _openUntil[index] && pose.sealFacesPlayer(index);

  /// The order seal [index] opened in among the seals open now, starting at
  /// 0 for the oldest, or null when it is shut.
  ///
  /// Only a reaper with the measure perk is shown this.
  int? sealRank(int index) {
    if (!sealOpen(index)) {
      return null;
    }

    var rank = 0;

    for (var other = 0; other < vessel.seals.length; other++) {
      if (sealOpen(other) && _openedOrder[other] < _openedOrder[index]) {
        rank++;
      }
    }

    return rank;
  }

  /// The point a fraction [progress] of the way along the reap arc.
  ///
  /// The arc bows upward in the middle, like the path of a swung blade.
  Vec3 reapArcPoint(double progress) => Vec3.fromAngles(
    _arcYaw + _reapDirection * HuntRules.reapHalfSpan * (2 * progress - 1),
    _arcPitchAt(progress),
  );

  /// Advances the hunt by one tick and returns what happened in it.
  List<HuntEvent> step(HuntInput input) {
    if (isOver) {
      return const [];
    }

    final events = <HuntEvent>[];

    switch (phase) {
      case HuntPhase.stalk:
        _stepStalk(input, events);
      case HuntPhase.fight || HuntPhase.windup || HuntPhase.shriek:
        _stepFight(input, events);
      case HuntPhase.broken:
        _stepReap(input, events);
      case HuntPhase.absorbed || HuntPhase.escaped:
        break;
    }

    tick++;

    return events;
  }

  // Stalk --------------------------------------------------------------

  void _stepStalk(HuntInput input, List<HuntEvent> events) {
    _move(events);

    final current = pose;
    final speed = input.turnRadiansPerSecond;
    final struck = input.taps.any(
      (tap) => current.covers(tap.direction, pad: stats.reachRadians),
    );

    // A reaper who strikes first has given up the ambush, and the strike
    // still counts.
    if (struck) {
      _beginFight(events, ambush: false);
      _stepStrikes(input, events);
      return;
    }

    if (speed > HuntRules.startleSpeed) {
      awareness +=
          (speed - HuntRules.startleSpeed) *
          HuntRules.startleGain /
          HuntRules.tickRate;
    } else {
      awareness = math.max(
        0,
        awareness - HuntRules.startleDecay / HuntRules.tickRate,
      );
    }

    final onTarget =
        input.aim.direction.angleTo(current.direction) <= HuntRules.ambushAim;

    if (onTarget && speed <= HuntRules.steadySpeed) {
      steadyTicks++;
    } else {
      steadyTicks = math.max(0, steadyTicks - 2);
    }

    if (steadyTicks >= HuntRules.ambushTicks) {
      _beginFight(events, ambush: true);
      return;
    }

    if (awareness >= 1 || tick >= HuntRules.stalkLimitTicks) {
      awareness = 1;
      _beginFight(events, ambush: false);
    }
  }

  void _beginFight(List<HuntEvent> events, {required bool ambush}) {
    phase = HuntPhase.fight;
    ambushed = ambush;
    _ambushStrikeReady = ambush;
    _scheduleShriek();
    events.add(StalkEnded(ambush: ambush));

    if (ambush) {
      _exposeAllSeals(events);

      return;
    }

    // It has seen the reaper, and it runs.
    _bolt(events);
  }

  // Fight --------------------------------------------------------------

  bool get _fighting =>
      phase == HuntPhase.fight ||
      phase == HuntPhase.windup ||
      phase == HuntPhase.shriek;

  void _stepFight(HuntInput input, List<HuntEvent> events) {
    _move(events);
    _fightTicks++;

    if (_fightTicks > _fightLimitTicks) {
      _escape(events, EscapeCause.time);
      return;
    }

    if (tick - _lastHitTick > HuntRules.cadenceWindowTicks) {
      cadence = 0;
    }

    _stepShriek(input, events);

    if (isOver || phase == HuntPhase.shriek) {
      return;
    }

    _stepSeals(events);
    _stepStrikes(input, events);

    if (ward <= 0) {
      _break(events);
    }
  }

  void _stepShriek(HuntInput input, List<HuntEvent> events) {
    if (phase == HuntPhase.fight && tick >= _nextShriekTick) {
      phase = HuntPhase.windup;
      _phaseEndTick = tick + HuntRules.windupTicks;
      events.add(const ShriekWindup());
      return;
    }

    if (phase == HuntPhase.windup && tick >= _phaseEndTick) {
      phase = HuntPhase.shriek;
      _shriekStartTick = tick;
      _phaseEndTick = tick + HuntRules.shriekTicks;
      _flinchedThisShriek = false;
      _bufferedTap = null;
      _openUntil.fillRange(0, _openUntil.length, tick);
      events.add(const ShriekBegan());
      return;
    }

    if (phase != HuntPhase.shriek) {
      return;
    }

    final pastGrace = tick - _shriekStartTick >= HuntRules.shriekGraceTicks;
    final moved =
        input.turnRadiansPerSecond > stats.flinchSpeed || input.taps.isNotEmpty;

    if (pastGrace && moved && !_flinchedThisShriek) {
      _flinch(events);
    }

    if (isOver || tick < _phaseEndTick) {
      return;
    }

    phase = HuntPhase.fight;
    _scheduleShriek();
    _nextBoltTick =
        tick +
        HuntRules.boltDelayMinTicks +
        _path.nextInt(
          HuntRules.boltDelayMaxTicks - HuntRules.boltDelayMinTicks + 1,
        );

    if (_flinchedThisShriek) {
      return;
    }

    events.add(const HeldFirm());
    _exposeAllSeals(events);

    if (stats.perk == ClassPerk.chosenBlow) {
      _chosenBlowReady = true;
    }

    if (stats.perk == ClassPerk.toll) {
      _bonusCaptureTicks += HuntRules.tollTicks;
    }
  }

  void _flinch(List<HuntEvent> events) {
    _flinchedThisShriek = true;
    cadence = 0;

    final forgiven = stats.perk == ClassPerk.steadfast && !_steadfastSpent;

    if (forgiven) {
      _steadfastSpent = true;
    } else {
      resolve--;
    }

    events.add(Flinched(resolveLeft: resolve, forgiven: forgiven));

    if (resolve <= 0) {
      _escape(events, EscapeCause.resolve);
    }
  }

  void _scheduleShriek() {
    final gap = HuntRules.shriekGapMaxTicks - HuntRules.shriekGapMinTicks;

    _nextShriekTick =
        tick + HuntRules.shriekGapMinTicks + _shrieks.nextInt(gap + 1);
  }

  void _stepSeals(List<HuntEvent> events) {
    final current = pose;
    var open = 0;

    for (var index = 0; index < vessel.seals.length; index++) {
      if (sealOpen(index)) {
        open++;
      }
    }

    while (open < math.min(HuntRules.openSeals, vessel.seals.length)) {
      // A seal only opens where the reaper can see it, so a turning body
      // lights its seals as they come round.
      final ready = [
        for (var index = 0; index < vessel.seals.length; index++)
          if (tick >= _openUntil[index] &&
              tick >= _shutUntil[index] &&
              current.sealFacesPlayer(index))
            index,
      ];

      if (ready.isEmpty) {
        return;
      }

      _openSeal(
        _schedule.pick(ready),
        ticks: 150 + _schedule.nextInt(110),
        events: events,
      );
      open++;
    }
  }

  void _openSeal(
    int index, {
    required int ticks,
    required List<HuntEvent> events,
  }) {
    final wasShut = tick >= _openUntil[index];

    _openUntil[index] = tick + ticks;

    if (!wasShut) {
      return;
    }

    _openedOrder[index] = _openedCount++;
    events.add(SealOpened(index));
  }

  void _exposeAllSeals(List<HuntEvent> events) {
    for (var index = 0; index < vessel.seals.length; index++) {
      _openSeal(index, ticks: HuntRules.exposedTicks, events: events);
    }
  }

  void _stepStrikes(HuntInput input, List<HuntEvent> events) {
    final buffered = _bufferedTap;

    if (buffered != null && tick - _bufferedAt > HuntRules.bufferTicks) {
      _bufferedTap = null;
    }

    if (buffered != null && _bufferedTap != null && tick >= _nextStrikeTick) {
      _bufferedTap = null;
      _resolveStrike(buffered, events);
    }

    for (final tap in input.taps) {
      if (tick >= _nextStrikeTick) {
        _resolveStrike(tap, events);
        continue;
      }

      // One slot: a later tap replaces an earlier one that is still waiting.
      _bufferedTap = tap;
      _bufferedAt = tick;
    }
  }

  void _resolveStrike(Aim tap, List<HuntEvent> events) {
    final ray = tap.direction;
    final current = pose;

    _nextStrikeTick = tick + stats.swingTicks;

    final seal = _nearestOpenSeal(ray, current, stats.reachRadians);

    if (seal == null) {
      _resolveMiss(ray, current, events);
      return;
    }

    final cap = stats.perk == ClassPerk.chain
        ? HuntRules.chainCadenceCap
        : HuntRules.cadenceCap;
    var scale = 1 + math.min(cadence, cap) * HuntRules.cadenceStep;
    var empowered = false;

    if (_ambushStrikeReady) {
      _ambushStrikeReady = false;
      scale *= HuntRules.ambushStrike;
      empowered = true;
    }

    if (_chosenBlowReady) {
      _chosenBlowReady = false;
      scale *= HuntRules.chosenBlowStrike;
      empowered = true;
    }

    if (stats.perk == ClassPerk.measure && sealRank(seal) == 0) {
      scale *= HuntRules.measureStrike;
      empowered = true;
    }

    var damage = stats.strikeDamage * scale;
    final cleaved = <int>[];

    _shutSeal(seal);

    if (stats.perk == ClassPerk.cleave) {
      final reach = stats.reachRadians * HuntRules.cleaveReach;

      for (var other = 0; other < vessel.seals.length; other++) {
        final caught =
            sealOpen(other) &&
            ray.angleTo(current.sealDirection(other)) <= reach;

        if (!caught) {
          continue;
        }

        _shutSeal(other);
        cleaved.add(other);
        damage += stats.strikeDamage * scale * HuntRules.cleaveShare;
      }
    }

    ward = math.max(0, ward - damage);
    cadence++;
    _lastHitTick = tick;
    events.add(
      StrikeLanded(
        seal: seal,
        damage: damage,
        cadence: cadence,
        empowered: empowered,
        cleaved: cleaved,
      ),
    );
  }

  void _resolveMiss(Vec3 ray, HuskPose current, List<HuntEvent> events) {
    if (!current.covers(ray, pad: stats.reachRadians * 0.5)) {
      cadence = 0;
      events.add(const StrikeWhiffed());
      return;
    }

    final damage = stats.strikeDamage * HuntRules.glanceShare;

    ward = math.max(0, ward - damage);
    events.add(StrikeGlanced(damage: damage));
  }

  int? _nearestOpenSeal(Vec3 ray, HuskPose current, double reach) {
    int? nearest;
    var nearestAngle = reach;

    for (var index = 0; index < vessel.seals.length; index++) {
      if (!sealOpen(index)) {
        continue;
      }

      final angle = ray.angleTo(current.sealDirection(index));

      if (angle <= nearestAngle) {
        nearest = index;
        nearestAngle = angle;
      }
    }

    return nearest;
  }

  void _shutSeal(int index) {
    _openUntil[index] = tick;
    _shutUntil[index] = tick + HuntRules.sealCooldownTicks;
  }

  // Break and reap -----------------------------------------------------

  void _break(List<HuntEvent> events) {
    final current = pose;

    phase = HuntPhase.broken;
    ward = 0;
    _halt();
    _bufferedTap = null;
    _arcYaw = current.yaw;
    _arcPitch = current.pitch;
    _captureEndTick = tick + captureTicks;
    reapBegun = false;
    reapProgress = 0;
    _deviationSum = 0;
    _reapTicks = 0;
    _fastTicks = 0;
    events.add(const Broken());
  }

  void _stepReap(HuntInput input, List<HuntEvent> events) {
    final aim = input.aim.direction;

    if (!reapBegun && aim.angleTo(reapArcPoint(0)) <= HuntRules.reapStartAim) {
      reapBegun = true;
      events.add(const ReapBegan());
    }

    if (reapBegun) {
      _trackCut(aim, input.turnRadiansPerSecond);
    }

    if (reapProgress >= 1) {
      _absorb(events);
      return;
    }

    if (tick < _captureEndTick) {
      return;
    }

    reapAttemptsLeft--;

    if (reapAttemptsLeft <= 0) {
      _escape(events, EscapeCause.reap);
      return;
    }

    phase = HuntPhase.fight;
    ward = wardMax * HuntRules.recoverShare;
    cadence = 0;
    // It pulls itself together and is off at once.
    _nextBoltTick = tick + 30;
    _scheduleShriek();
    events.add(ReapRecovered(attemptsLeft: reapAttemptsLeft));
  }

  void _trackCut(Vec3 aim, double speed) {
    final along =
        wrapAngle(aim.yaw - _arcYaw) *
            _reapDirection /
            (2 * HuntRules.reapHalfSpan) +
        0.5;
    final clamped = along.clamp(0.0, 1.0);

    // The cut only ever moves forward: drifting back loses nothing.
    reapProgress = math.max(reapProgress, clamped);
    _deviationSum += (aim.pitch - _arcPitchAt(clamped)).abs();
    _reapTicks++;

    if (speed > HuntRules.reapSpeedLimit) {
      _fastTicks++;
    }
  }

  double _arcPitchAt(double progress) =>
      _arcPitch + radians(5) * 4 * progress * (1 - progress) - radians(2.5);

  void _absorb(List<HuntEvent> events) {
    final deviation = _deviationSum / _reapTicks;
    final fastShare = _fastTicks / _reapTicks;
    final tolerance = stats.reapToleranceRadians;

    final grade = switch ((deviation, fastShare)) {
      (final d, final f) when d > tolerance || f > 0.2 => ReapGrade.ragged,
      (final d, final f)
          when d <= tolerance * HuntRules.perfectShare && f <= 0.05 =>
        ReapGrade.perfect,
      _ => ReapGrade.clean,
    };

    phase = HuntPhase.absorbed;
    outcome = HuntOutcome.absorbed(
      grade: grade,
      ambushed: ambushed,
      ticks: tick + 1,
    );
    events.add(HuskAbsorbed(grade));
  }

  void _escape(List<HuntEvent> events, EscapeCause cause) {
    phase = HuntPhase.escaped;
    outcome = HuntOutcome.escaped(
      escape: cause,
      ambushed: ambushed,
      ticks: tick + 1,
    );
    events.add(HuskEscaped(cause));
  }

  // Movement -----------------------------------------------------------

  /// Moves the husk on by one tick: along the move it is making, or into a
  /// new one when it is time to amble or to bolt.
  void _move(List<HuntEvent> events) {
    if (vessel.gait == Gait.still) {
      return;
    }

    // Drawing breath, a husk stops where it is and turns on the reaper.
    if (phase == HuntPhase.windup || phase == HuntPhase.shriek) {
      _halt();
      _turnToward(_bearing + math.pi);

      return;
    }

    if (_moveStart >= 0) {
      _continueMove();

      return;
    }

    final bolting =
        phase == HuntPhase.fight && _nextBoltTick >= 0 && tick >= _nextBoltTick;

    if (bolting) {
      _bolt(events);
    } else if (tick >= _nextAmbleTick) {
      _amble();
    }
  }

  void _continueMove() {
    final progress = (tick - _moveStart) / _moveTicks;

    if (progress >= 1) {
      _bearing = wrapAngle(_fromBearing + _byBearing);
      _range = _fromRange + _byRange;
      _height = _fromHeight + _byHeight;
      _moveStart = -1;
      _nextAmbleTick = tick + _restTicks();

      return;
    }

    final eased = smoothstep(progress);

    _bearing = wrapAngle(_fromBearing + _byBearing * eased);
    _range = _fromRange + _byRange * eased;
    _height = _fromHeight + _byHeight * eased;

    // The body faces the way it is going: along the floor, the sum of going
    // round the player and going toward or away from them.
    final round = _range * _byBearing;

    _turnToward(
      math.atan2(
        _byRange * math.sin(_bearing) + round * math.cos(_bearing),
        _byRange * math.cos(_bearing) - round * math.sin(_bearing),
      ),
    );
  }

  /// A short, unhurried move to somewhere nearby.
  void _amble() {
    final by = radians(
      _path.nextRange(-HuntRules.ambleDegrees, HuntRules.ambleDegrees),
    );

    _setOff(
      toBearing: plan.clamp(_bearing + by),
      toRange: _path.nextRange(vessel.nearMetres, vessel.farMetres),
      speed: vessel.walkSpeed,
    );
  }

  /// A dash to somewhere well away from where the reaper is aiming.
  void _bolt(List<HuntEvent> events) {
    _nextBoltTick = -1;

    if (vessel.gait == Gait.still) {
      return;
    }

    final side = _path.nextChance(0.5) ? 1 : -1;
    final by = radians(
      _path.nextRange(HuntRules.boltMinDegrees, HuntRules.boltMaxDegrees),
    );
    // If one way runs out of room, it goes the other.
    final oneWay = plan.clamp(_bearing + side * by);
    final otherWay = plan.clamp(_bearing - side * by);
    final toBearing =
        wrapAngle(oneWay - _bearing).abs() >=
            wrapAngle(otherWay - _bearing).abs()
        ? oneWay
        : otherWay;
    final middle = (vessel.nearMetres + vessel.farMetres) / 2;

    _setOff(
      toBearing: toBearing,
      toRange: _path.nextRange(middle, vessel.farMetres),
      speed: vessel.boltSpeed,
    );
    events.add(const Relocating());
  }

  void _setOff({
    required double toBearing,
    required double toRange,
    required double speed,
  }) {
    _fromBearing = _bearing;
    _fromRange = _range;
    _fromHeight = _height;
    _byBearing = wrapAngle(toBearing - _bearing);
    _byRange = toRange - _range;
    _byHeight = _restingHeight() - _height;

    final meanRange = _range + _byRange / 2;
    final length = math.sqrt(
      math.pow(meanRange * _byBearing, 2) +
          math.pow(_byRange, 2) +
          math.pow(_byHeight, 2),
    );

    _moveTicks = math.max(12, (length / speed * HuntRules.tickRate).round());
    _moveStart = tick;
  }

  /// Stops the husk where it stands.
  void _halt() {
    if (_moveStart < 0) {
      return;
    }

    _moveStart = -1;
    _nextAmbleTick = tick + _restTicks();
  }

  void _turnToward(double heading) {
    final step = radians(HuntRules.turnDegreesPerTick);

    _heading = wrapAngle(
      _heading + wrapAngle(heading - _heading).clamp(-step, step),
    );
  }

  int _restTicks() =>
      HuntRules.restMinTicks +
      _path.nextInt(HuntRules.restMaxTicks - HuntRules.restMinTicks + 1);

  /// How high the middle of the body is while it rests: standing on the
  /// floor, or somewhere in the band of air it keeps to.
  double _restingHeight() => switch (vessel.gait) {
    Gait.ground => vessel.halfHeight - HuntRules.eyeHeight,
    Gait.air ||
    Gait.still => _path.nextRange(vessel.lowMetres, vessel.highMetres),
  };
}
