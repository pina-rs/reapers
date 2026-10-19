import 'dart:math' as math;

import '../arts.dart';
import '../reaper_class.dart';
import '../reaper_soul.dart';
import '../rift/rift_room.dart';
import '../rift/rift_rules.dart';
import '../seeded_random.dart';
import '../vec3.dart';
import '../vessel.dart';
import 'art_flight.dart';
import 'difficulty.dart';
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
  ///
  /// [ward] and [stats] are the husk's and the reaper's own. The hunt eases
  /// or stiffens them for [difficulty] itself, so that the phone and the
  /// server cannot disagree about what a difficulty means.
  HuntSim({
    required int seed,
    required Vessel vessel,
    required int ward,
    required HuntStats stats,
    HuntPlan plan = HuntPlan.open,
    Difficulty difficulty = Difficulty.normal,
  }) : this._(
         seed: seed,
         vessel: vessel,
         ward: ward * difficulty.ward,
         stats: stats.at(difficulty),
         plan: plan,
         difficulty: difficulty,
         inRift: false,
       );

  /// One reaper's place in a rift.
  ///
  /// The husk stands dead ahead and the fight starts at once: there is
  /// nothing to stalk. Its [ward] belongs to the whole circle, so whoever
  /// runs the rift keeps this reaper's view of it current with [shareWard].
  /// Every reaper given the same [seed] hears the husk shriek at the same
  /// moments, and there is one reap window, not two.
  ///
  /// [sighting] is where this reaper sees the husk: the way to the place on
  /// the earth the whole circle is looking at. Without one it stands dead
  /// ahead of them, at its usual distance.
  HuntSim.rift({
    required int seed,
    required Vessel vessel,
    required double ward,
    required HuntStats stats,
    RiftSighting? sighting,
  }) : this._(
         seed: seed,
         vessel: vessel,
         ward: ward,
         stats: stats,
         plan: HuntPlan.open,
         difficulty: Difficulty.normal,
         inRift: true,
         sighting: sighting,
       );

  HuntSim._({
    required this.seed,
    required this.vessel,
    required this.ward,
    required this.stats,
    required this.plan,
    required this.difficulty,
    required this.inRift,
    RiftSighting? sighting,
  }) : wardMax = ward,
       resolve = stats.resolve,
       reapAttemptsLeft = inRift
           ? 1
           : HuntRules.reapAttempts + difficulty.reaps,
       _schedule = SeededRandom(mixSeed([seed, 0x5EA1])),
       _shrieks = SeededRandom(mixSeed([seed, 0x5C81])),
       _path = SeededRandom(mixSeed([seed, 0x9A7B])),
       _charges = SeededRandom(mixSeed([seed, 0xC4A6])),
       _cover = SeededRandom(mixSeed([seed, 0xC07E])),
       artCharges = stats.art == null ? 0 : ArtRules.startingCharges,
       _openUntil = List.filled(vessel.seals.length, 0),
       _shutUntil = List.filled(vessel.seals.length, 0),
       _openedOrder = List.filled(vessel.seals.length, 0) {
    if (!plan.isWellFormed) {
      throw ArgumentError.value(plan, 'plan', 'is not one a phone could make');
    }

    _reapDirection = _path.nextChance(0.5) ? 1 : -1;
    _bobPhase = _path.nextRange(0, 2 * math.pi);

    // How far out it starts, as a share of the room it has wherever it
    // turns out to be.
    final outward = _path.nextRange(0, 1);

    _height = _restingHeight();
    _fromGround = _floor;
    _toGround = _floor;
    _nextAmbleTick = _restTicks();

    if (inRift) {
      // A rift's husk stands where the circle sees it, or dead ahead, and
      // faces this reaper.
      _bearing = sighting?.bearing ?? 0;
      _range = sighting?.metres ?? _rangeAt(_bearing, outward);
      _heading = wrapAngle(_bearing + math.pi);
      phase = HuntPhase.fight;
      _scheduleShriek();

      return;
    }

    // With the whole circle to hide in, a husk starts somewhere the reaper
    // has to turn to find. Inside a plan it starts anywhere in it.
    _bearing = plan.isWholeCircle
        ? (_path.nextChance(0.5) ? 1 : -1) * radians(_path.nextRange(55, 180))
        : plan.clamp(plan.centre + _path.nextRange(-0.8, 0.8) * plan.halfWidth);
    _range = _rangeAt(_bearing, outward);
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
    if (!log.plan.isWellFormed) {
      return null;
    }

    final sim = HuntSim(
      seed: seed,
      vessel: vessel,
      ward: ward,
      stats: stats,
      plan: log.plan,
      difficulty: log.difficulty,
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

  /// The numbers the hunt is played with, already eased or stiffened for
  /// [difficulty].
  final HuntStats stats;

  /// Where the husk may go.
  final HuntPlan plan;

  /// How forgiving the hunt is.
  final Difficulty difficulty;

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

  /// Decides when the husk charges, and nothing else.
  final SeededRandom _charges;

  /// Decides when the husk makes for cover or a ledge, and which. Kept
  /// apart from [_path] so that a room with nothing in it plays exactly as
  /// one did before the room's furniture was known.
  final SeededRandom _cover;

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

  /// What the husk stands on or behind, and, while [_leaping], what it is
  /// leaping to and how high the leap carries it above the straight line.
  Footing _footing = Footing.open;
  Footing _toFooting = Footing.open;
  bool _leaping = false;
  double _arc = 0;

  /// The surface under the husk, in metres from the eye, where it set off
  /// from and where it is going: for the view to put its shadow on.
  double _fromGround = 0;
  double _toGround = 0;

  // Arts ---------------------------------------------------------------

  /// Throws the reaper has ready.
  int artCharges;

  int _rechargeTicks = 0;
  int _gatherTicks = 0;
  Aim? _castAim;
  final _flights = <ArtFlight>[];
  int _burnUntil = 0;
  double _burnPerTick = 0;
  int _frozenUntil = 0;

  /// Until when the husk keeps to the open: burning, or shaken by wind.
  int _exposedUntil = 0;

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
  bool _giftStrikeReady = false;
  bool _passageReady = false;
  bool _returnReady = false;
  int _giftUses = 0;
  int _orderedHits = 0;
  int _refrainTolls = 0;
  int _echoCadence = 0;
  int _holds = 0;
  int _parries = 0;
  int _hitStreak = 0;
  int _maxHitStreak = 0;

  HuntDeeds get deeds =>
      HuntDeeds(holds: _holds, parries: _parries, maxCadence: _maxHitStreak);

  // Shriek -------------------------------------------------------------

  int _phaseEndTick = 0;
  int _shriekStartTick = 0;
  int _nextShriekTick = 0;
  bool _flinchedThisShriek = false;
  bool _steadfastSpent = false;
  int _fightTicks = 0;

  // Charge -------------------------------------------------------------

  int _nextChargeTick = -1;
  int _chargeStartTick = 0;
  int _staggeredUntil = 0;

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

  int get _fightLimitTicks => inRift
      ? RiftRules.fightLimitTicks
      : (HuntRules.fightLimitTicks * difficulty.patience).round();

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
    // A husk in the air never hangs quite still. One perched on a ledge
    // does.
    final bob = vessel.gait == Gait.air && _footing == Footing.open && !_leaping
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
      footing: _leaping ? _toFooting : _footing,
      leaping: _leaping,
      ground: _ground,
    );
  }

  /// What the husk stands on or behind.
  Footing get footing => _footing;

  /// Whether the husk is in the air between one footing and the next.
  bool get leaping => _leaping;

  /// The floor, in metres from the eye.
  double get _floor => -plan.eyeHeight;

  double? get _ground {
    final aloft = vessel.gait != Gait.ground;

    if (!_leaping) {
      return aloft && _footing == Footing.open ? null : _fromGround;
    }

    if (aloft) {
      return null;
    }

    final along = smoothstep((tick - _moveStart) / _moveTicks);

    return _fromGround + (_toGround - _fromGround) * along;
  }

  /// Arts in the air.
  List<ArtFlight> get flights => List.unmodifiable(_flights);

  /// How far the art in hand is gathered, from 0 to 1. At 1 a flick throws
  /// it.
  double get artGathered =>
      stats.art == null ? 0 : _gatherTicks / ArtRules.gatherTicks;

  /// Ticks until the next throw is ready, as a share of the wait.
  double get artRecharge => switch (stats.art) {
    final art? when artCharges < art.maxCharges =>
      _rechargeTicks / art.rechargeTicks,
    _ => 0,
  };

  /// Whether the husk is burning.
  bool get burning => _fighting && tick < _burnUntil;

  /// Whether the husk is frozen where it stands.
  bool get frozen => _fighting && tick < _frozenUntil;

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
  bool sealOpen(int index) {
    if (!_fighting || tick >= _openUntil[index]) {
      return false;
    }

    final current = pose;

    return current.sealFacesPlayer(index) && !_behindCover(current, index);
  }

  /// Whether seal [index] of a husk at [current] is behind something in the
  /// room, out of sight and out of reach.
  bool _behindCover(HuskPose current, int index) =>
      plan.hides(current.sealPosition(index));

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
      case HuntPhase.fight ||
          HuntPhase.windup ||
          HuntPhase.shriek ||
          HuntPhase.charge:
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

    // An art thrown at a husk that has not noticed the reaper goes unseen
    // until it lands. Then the husk knows, and the ambush is given up.
    _stepFlights(events);

    if (phase != HuntPhase.stalk) {
      return;
    }

    _stepArt(input, events);

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
    _scheduleCharge();
    events.add(StalkEnded(ambush: ambush));

    if (ambush) {
      _exposeAllSeals(events);

      switch (stats.gift) {
        case ReaperGift.passage:
          _passageReady = true;
          events.add(const GiftStirred(ReaperGift.passage));
        case ReaperGift.foreknowledge:
          for (var seal = 0; seal < vessel.seals.length; seal++) {
            _openUntil[seal] +=
                SoulRules.foreknowledgeTicksPerStrength * stats.giftStrength;
          }
          events.add(const GiftStirred(ReaperGift.foreknowledge));
        case ReaperGift.hunger:
          cadence = stats.giftStrength;
          _lastHitTick = tick;
          events.add(const GiftStirred(ReaperGift.hunger));
        case ReaperGift.hush:
          _nextShriekTick +=
              SoulRules.hushTicksPerStrength * stats.giftStrength;
          events.add(const GiftStirred(ReaperGift.hush));
        default:
          break;
      }

      return;
    }

    // It has seen the reaper, and it runs, unless it is in no state to.
    if (!_leaping && tick >= _frozenUntil && tick >= _staggeredUntil) {
      _bolt(events);
    }
  }

  // Fight --------------------------------------------------------------

  bool get _fighting =>
      phase == HuntPhase.fight ||
      phase == HuntPhase.windup ||
      phase == HuntPhase.shriek ||
      phase == HuntPhase.charge;

  void _stepFight(HuntInput input, List<HuntEvent> events) {
    _move(events);
    _fightTicks++;

    if (_fightTicks > _fightLimitTicks) {
      _escape(events, EscapeCause.time);
      return;
    }

    if (tick - _lastHitTick > HuntRules.cadenceWindowTicks) {
      cadence = 0;
      _hitStreak = 0;
    }

    _stepBurn();
    _stepFlights(events);

    if (ward <= 0) {
      _break(events);
      return;
    }

    _stepShriek(input, events);

    if (isOver || phase == HuntPhase.shriek) {
      _stepArt(input, events);
      return;
    }

    _stepCharge(input, events);

    if (isOver) {
      return;
    }

    _stepArt(input, events);
    _stepSeals(events);
    _stepStrikes(input, events);

    if (ward <= 0) {
      _break(events);
    }
  }

  void _stepShriek(HuntInput input, List<HuntEvent> events) {
    // A husk in mid-leap lands before it draws breath.
    if (phase == HuntPhase.fight && tick >= _nextShriekTick && !_leaping) {
      _echoCadence = cadence;
      phase = HuntPhase.windup;
      _phaseEndTick = tick + HuntRules.windupTicks;
      events.add(const ShriekWindup());

      // A husk shrieks at the reaper's face, not from behind the sofa: it
      // comes out over the top as it draws breath.
      if (_footing == Footing.hidden && !_leaping) {
        _leaveCover(events, mayStay: false);
      }

      return;
    }

    if (phase == HuntPhase.windup && tick >= _phaseEndTick && !_leaping) {
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
    // Having just been told to hold still, the reaper is given a moment
    // before being told to swing.
    if (_nextChargeTick >= 0) {
      _nextChargeTick = math.max(
        _nextChargeTick,
        tick + HuntRules.chargeClearTicks,
      );
    }
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
    _holds++;
    _exposeAllSeals(events);

    // Stillness steadies the hand: an unflinching hold readies a throw.
    if (stats.art case final art? when artCharges < art.maxCharges) {
      artCharges++;
    }

    if (stats.perk == ClassPerk.chosenBlow) {
      _chosenBlowReady = true;
    }

    if (stats.perk == ClassPerk.toll) {
      _bonusCaptureTicks += HuntRules.tollTicks;
    }

    switch (stats.gift) {
      case ReaperGift.aftermath || ReaperGift.verdict || ReaperGift.keening:
        _giftStrikeReady = true;
      case ReaperGift.anchor:
        if (_giftUses < stats.giftStrength && resolve < stats.resolve) {
          resolve++;
          _giftUses++;
          events.add(const GiftStirred(ReaperGift.anchor));
        }
      case ReaperGift.echo:
        cadence = math.min(
          HuntRules.chainCadenceCap,
          _echoCadence + stats.giftStrength,
        );
        _lastHitTick = tick;
        events.add(const GiftStirred(ReaperGift.echo));
      default:
        break;
    }
  }

  void _flinch(List<HuntEvent> events) {
    _flinchedThisShriek = true;
    cadence = 0;
    _hitStreak = 0;

    final steadfast = stats.perk == ClassPerk.steadfast && !_steadfastSpent;
    final returned = !steadfast && _returnReady;
    final forgiven = steadfast || returned;

    if (steadfast) {
      _steadfastSpent = true;
    } else if (returned) {
      _returnReady = false;
      events.add(const GiftStirred(ReaperGift.returnLight));
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

  // Charge -------------------------------------------------------------

  /// How far through its charge the husk is, from 0 to 1. Zero when it is
  /// not charging.
  double get chargeProgress => phase == HuntPhase.charge
      ? ((tick - _chargeStartTick) / _chargeTicks).clamp(0.0, 1.0)
      : 0;

  /// Whether the husk stands stunned after a charge was turned aside.
  bool get staggered => _fighting && tick < _staggeredUntil;

  int get _chargeTicks => (HuntRules.chargeTicks * difficulty.tell).round();

  /// Whether this husk charges at all: one that is rooted to the spot does
  /// not, and neither does a rift's, which is not any one reaper's to face.
  bool get _mayCharge => !inRift && vessel.gait != Gait.still;

  void _scheduleCharge() {
    if (!_mayCharge) {
      return;
    }

    final gap = HuntRules.chargeGapMaxTicks - HuntRules.chargeGapMinTicks;

    _nextChargeTick =
        tick + HuntRules.chargeGapMinTicks + _charges.nextInt(gap + 1);
  }

  void _stepCharge(HuntInput input, List<HuntEvent> events) {
    if (phase == HuntPhase.charge) {
      final swinging =
          tick - _chargeStartTick >= HuntRules.chargeGraceTicks &&
          input.turnRadiansPerSecond >= stats.parrySpeed;

      if (swinging) {
        _parry(events);
      } else if (tick >= _phaseEndTick) {
        _strikeReaper(events);
      }

      return;
    }

    final due =
        phase == HuntPhase.fight &&
        _nextChargeTick >= 0 &&
        tick >= _nextChargeTick &&
        tick >= _staggeredUntil &&
        tick >= _frozenUntil;

    if (!due || _leaping) {
      return;
    }

    // It has to be out in the open to run at anyone: it comes down or out
    // first, and the charge waits for it to land.
    if (_footing != Footing.open) {
      _leaveCover(events, mayStay: false);
      return;
    }

    // With a shriek about to start, the charge waits until it is over.
    if (_nextShriekTick - tick < _chargeTicks + HuntRules.chargeClearTicks) {
      return;
    }

    phase = HuntPhase.charge;
    _chargeStartTick = tick;
    _phaseEndTick = tick + _chargeTicks;
    _nextBoltTick = -1;

    // Straight at the reaper, to within arm's reach. One in the air comes
    // down to eye level as it comes.
    _fromBearing = _bearing;
    _fromRange = _range;
    _fromHeight = _height;
    _byBearing = 0;
    _byRange = math.min(0, HuntRules.chargeMetres - _range);
    _byHeight = vessel.gait == Gait.air ? -_height : 0;
    _moveStart = tick;
    _moveTicks = _chargeTicks;
    events.add(const ChargeBegan());
  }

  /// The reaper swung in time. The husk is knocked out of its run and
  /// stands stunned where it got to, every seal in view open.
  void _parry(List<HuntEvent> events) {
    _endCharge();
    _moveStart = -1;
    _staggeredUntil = tick + HuntRules.staggerTicks;
    _nextBoltTick = _staggeredUntil;
    events.add(const Parried());
    _parries++;
    _exposeAllSeals(events);

    if (stats.gift == ReaperGift.reprisal) {
      _chosenBlowReady = true;
      _giftStrikeReady = true;
      events.add(const GiftStirred(ReaperGift.reprisal));
    }
  }

  /// The charge got through. It costs the reaper resolve, and the husk is
  /// off again at once.
  void _strikeReaper(List<HuntEvent> events) {
    if (_passageReady) {
      _passageReady = false;
      _parry(events);
      // This is the gift's protection, not an input the player performed.
      _parries--;
      _staggeredUntil =
          tick + SoulRules.passageStaggerTicksPerStrength * stats.giftStrength;
      _nextBoltTick = _staggeredUntil;
      events.add(const GiftStirred(ReaperGift.passage));
      return;
    }

    _endCharge();
    resolve--;
    cadence = 0;
    _hitStreak = 0;
    events.add(Struck(resolveLeft: resolve));

    if (resolve <= 0) {
      _escape(events, EscapeCause.resolve);

      return;
    }

    _bolt(events);
  }

  void _endCharge() {
    phase = HuntPhase.fight;
    _scheduleCharge();
    // A shriek does not come straight on top of a charge.
    _nextShriekTick = math.max(
      _nextShriekTick,
      tick + HuntRules.chargeClearTicks,
    );
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
              current.sealFacesPlayer(index) &&
              !_behindCover(current, index))
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
    final firstStrike = _ambushStrikeReady;
    final inOrder = sealRank(seal) == 0;
    final aftermath = _giftStrikeReady && stats.gift == ReaperGift.aftermath;

    if (_ambushStrikeReady) {
      _ambushStrikeReady = false;
      scale *= HuntRules.ambushStrike;
      empowered = true;
    }

    if (firstStrike && stats.gift == ReaperGift.firstLight) {
      scale *= 1 + SoulRules.giftStrikePerStrength * stats.giftStrength;
      events.add(const GiftStirred(ReaperGift.firstLight));
    }

    if (_chosenBlowReady) {
      _chosenBlowReady = false;
      scale *= HuntRules.chosenBlowStrike;
      empowered = true;
    }

    if (stats.perk == ClassPerk.measure && inOrder) {
      scale *= HuntRules.measureStrike;
      empowered = true;
    }

    final verdict = stats.gift == ReaperGift.verdict && inOrder;
    final promised =
        _giftStrikeReady &&
        (verdict ||
            stats.gift == ReaperGift.keening ||
            stats.gift == ReaperGift.reprisal);
    final execution =
        stats.gift == ReaperGift.lastWord &&
        cadence >= SoulRules.executionCadence &&
        ward <= wardMax * SoulRules.executionWardShare;

    if (promised || execution) {
      scale *= 1 + SoulRules.giftStrikePerStrength * stats.giftStrength;
      empowered = true;
      if (promised) _giftStrikeReady = false;
      events.add(GiftStirred(stats.gift!));
    }

    if (aftermath) {
      _giftStrikeReady = false;
      empowered = true;
      events.add(const GiftStirred(ReaperGift.aftermath));
    }

    if (frozen) {
      scale *= stats.art?.shatter ?? 1;
      empowered = true;
      _frozenUntil = tick;
      events.add(const IceShattered());
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

        if (aftermath) {
          _openUntil[other] = math.max(
            _openUntil[other],
            tick +
                HuntRules.exposedTicks +
                SoulRules.gatheringTicksPerStrength * stats.giftStrength,
          );
        } else {
          _shutSeal(other);
        }
        cleaved.add(other);
        final share =
            HuntRules.cleaveShare +
            (firstStrike && stats.gift == ReaperGift.furrow
                ? SoulRules.furrowCleavePerStrength * stats.giftStrength
                : 0);
        damage += stats.strikeDamage * scale * share;
      }
    }

    if (firstStrike && stats.gift == ReaperGift.furrow && cleaved.isNotEmpty) {
      empowered = true;
      events.add(const GiftStirred(ReaperGift.furrow));
    }

    ward = math.max(0, ward - damage);
    _knock();
    cadence++;
    _hitStreak++;
    _maxHitStreak = math.max(_maxHitStreak, _hitStreak);
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
    _afterGiftStrike(seal, inOrder: inOrder, events: events);
  }

  /// Conditional powers keep their own counters, never a second simulation.
  void _afterGiftStrike(
    int seal, {
    required bool inOrder,
    required List<HuntEvent> events,
  }) {
    switch (stats.gift) {
      case ReaperGift.gathering:
        if (cadence % SoulRules.gatheringHits == 0) {
          _shutUntil[seal] = tick;
          _openSeal(
            seal,
            ticks:
                HuntRules.sealCooldownTicks +
                SoulRules.gatheringTicksPerStrength * stats.giftStrength,
            events: events,
          );
          events.add(const GiftStirred(ReaperGift.gathering));
        }
      case ReaperGift.returnLight:
        if (cadence >= SoulRules.returnCadence &&
            !_returnReady &&
            _giftUses < stats.giftStrength) {
          _returnReady = true;
          _giftUses++;
          events.add(const GiftStirred(ReaperGift.returnLight));
        }
      case ReaperGift.equilibrium:
        _orderedHits = inOrder ? _orderedHits + 1 : 0;
        if (_orderedHits == SoulRules.equilibriumHits) {
          _orderedHits = 0;
          for (var other = 0; other < vessel.seals.length; other++) {
            _shutUntil[other] = tick;
            _openSeal(
              other,
              ticks: math.max(
                _openUntil[other] - tick,
                HuntRules.sealCooldownTicks +
                    SoulRules.equilibriumTicksPerStrength * stats.giftStrength,
              ),
              events: events,
            );
          }
          events.add(const GiftStirred(ReaperGift.equilibrium));
        }
      case ReaperGift.refrain:
        if (cadence % SoulRules.refrainHits == 0 &&
            _refrainTolls < SoulRules.refrainMaxTolls) {
          _bonusCaptureTicks +=
              SoulRules.refrainTicksPerStrength * stats.giftStrength;
          _refrainTolls++;
          events.add(const GiftStirred(ReaperGift.refrain));
        }
      default:
        break;
    }
  }

  /// A heavy blade stops the husk in its tracks.
  void _knock() {
    if (stats.knockTicks == 0 || phase != HuntPhase.fight || _leaping) {
      return;
    }

    _staggeredUntil = math.max(_staggeredUntil, tick + stats.knockTicks);
    _halt();
  }

  void _resolveMiss(Vec3 ray, HuskPose current, List<HuntEvent> events) {
    // A blow at the part of a husk that is behind the counter hits the
    // counter.
    final blocked = plan.hides(ray * current.distance);

    if (blocked || !current.covers(ray, pad: stats.reachRadians * 0.5)) {
      _hitStreak = 0;
      if (stats.gift == ReaperGift.unbroken &&
          cadence > 0 &&
          _giftUses < stats.giftStrength) {
        _giftUses++;
        events.add(const GiftStirred(ReaperGift.unbroken));
      } else {
        cadence = 0;
      }
      events.add(const StrikeWhiffed());
      return;
    }

    final damage = stats.strikeDamage * stats.glanceShare;

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
    // A husk broken in mid-leap still lands, and hangs open where it does.
    final landing = _leaping ? _destination : pose.position;

    phase = HuntPhase.broken;
    ward = 0;
    _halt();
    _bufferedTap = null;
    _flights.clear();
    _burnUntil = tick;
    _frozenUntil = tick;
    _arcYaw = landing.yaw;
    _arcPitch = landing.pitch;
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

    if (_leaping) {
      _continueMove();
    }

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
    _scheduleCharge();
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
      deeds: deeds,
    );
    events.add(HuskAbsorbed(grade));
  }

  void _escape(List<HuntEvent> events, EscapeCause cause) {
    phase = HuntPhase.escaped;
    outcome = HuntOutcome.escaped(
      escape: cause,
      ambushed: ambushed,
      ticks: tick + 1,
      deeds: deeds,
    );
    events.add(HuskEscaped(cause));
  }

  // Arts ---------------------------------------------------------------

  /// Gathers, throws and readies the reaper's art.
  ///
  /// A thumb held on the art gathers it while the phone is steady, and the
  /// art goes wherever the phone was last steady on. Once it is gathered, a
  /// flick of the phone throws it. Letting go before that loses nothing.
  void _stepArt(HuntInput input, List<HuntEvent> events) {
    final art = stats.art;

    if (art == null) {
      return;
    }

    if (artCharges < art.maxCharges) {
      _rechargeTicks++;

      if (_rechargeTicks >= art.rechargeTicks) {
        artCharges++;
        _rechargeTicks = 0;
      }
    } else {
      _rechargeTicks = 0;
    }

    if (!input.focus || artCharges == 0) {
      _gatherTicks = 0;
      _castAim = null;
      return;
    }

    final speed = input.turnRadiansPerSecond;
    final aim = _castAim;
    final gathered = _gatherTicks >= ArtRules.gatherTicks;

    // Nothing is thrown into a shriek: the flick is a flinch, and the art
    // stays gathered in the hand.
    if (gathered && aim != null && speed >= ArtRules.throwSpeed) {
      if (phase == HuntPhase.shriek) {
        return;
      }

      _throw(art, aim, events);
      return;
    }

    if (speed <= HuntRules.steadySpeed) {
      _gatherTicks = math.min(ArtRules.gatherTicks, _gatherTicks + 1);
      _castAim = input.aim;
    }
  }

  void _throw(ArtSkill art, Aim aim, List<HuntEvent> events) {
    final ray = aim.direction;
    final current = pose;
    final atHusk = switch (art.art) {
      Art.gale => ray.angleTo(current.direction) <= ArtRules.gustAngle,
      Art.cinder || Art.rime => current.covers(ray, pad: stats.reachRadians),
    };
    const from = Vec3(0, -ArtRules.handDrop, 0);
    final target = atHusk ? current.position : _landingAlong(ray);

    artCharges--;
    _gatherTicks = 0;
    _castAim = null;
    _flights.add(
      ArtFlight(
        art: art.art,
        from: from,
        target: target,
        homing: atHusk && art.art.homing,
        thrownAt: tick,
        ticks: math.max(
          ArtRules.shortestFlightTicks,
          ((target - from).length / art.art.speed * HuntRules.tickRate).round(),
        ),
      ),
    );
    events.add(ArtThrown(art.art));
  }

  /// Where an art thrown along [ray] at nothing comes down: on the floor, or
  /// against the front of whatever stands in the way, or at the end of its
  /// throw. Thrown over the top of something, it comes down behind it.
  Vec3 _landingAlong(Vec3 ray) {
    var metres = ArtRules.farthestMetres;

    if (ray.y < 0) {
      metres = math.min(metres, -plan.eyeHeight / ray.y);
    }

    final across = math.sqrt(ray.x * ray.x + ray.z * ray.z);
    final cover = plan.coverAt(ray.yaw);

    if (cover > 0 && across > 0) {
      final front =
          plan.reachDecimetres[HuntPlan.sliceOf(ray.yaw)] / 10 / across;

      if (front < metres && ray.y * front < cover - plan.eyeHeight) {
        metres = front;
      }
    }

    return ray * metres;
  }

  void _stepFlights(List<HuntEvent> events) {
    if (_flights.isEmpty) {
      return;
    }

    final landing = [
      for (final flight in _flights)
        if (tick >= flight.landsAt) flight,
    ];

    _flights.removeWhere((flight) => tick >= flight.landsAt);

    for (final flight in landing) {
      _artLands(flight, events);
    }
  }

  void _artLands(ArtFlight flight, List<HuntEvent> events) {
    final art = stats.art!;
    final current = pose;
    final at = flight.homing ? current.position : flight.target;
    final hit = switch (flight.art) {
      // Fire bursts where it comes down, whatever is in front of it: it was
      // lobbed over the top.
      Art.cinder =>
        (current.position - at).length <=
            vessel.halfWidth + ArtRules.burstMetres,
      // Ice flies straight, and breaks on whatever the husk is behind.
      Art.rime => flight.homing && !plan.hides(current.position),
      // Wind goes round things.
      Art.gale => flight.homing,
    };

    if (!hit) {
      events.add(ArtLanded(art: flight.art, at: at, hit: false));

      if (phase == HuntPhase.stalk) {
        _beginFight(events, ambush: false);
      }

      return;
    }

    final damage = stats.strikeDamage * art.strike;

    ward = math.max(0, ward - damage);
    events.add(ArtLanded(art: flight.art, at: at, hit: true, damage: damage));

    switch (flight.art) {
      case Art.cinder:
        _burnUntil = tick + art.burnTicks;
        _burnPerTick = stats.strikeDamage * art.burnStrike / art.burnTicks;
        _frozenUntil = tick;
        _exposedUntil = math.max(_exposedUntil, _burnUntil);

        // It runs, burning, and not into cover.
        if (phase == HuntPhase.fight && tick >= _staggeredUntil) {
          _nextBoltTick = tick + ArtRules.burnBoltTicks;
        }
      case Art.rime:
        _freeze(art, events);
      case Art.gale:
        _gust(art, events);
    }

    if (phase == HuntPhase.stalk) {
      _beginFight(events, ambush: false);
    }
  }

  void _stepBurn() {
    if (tick < _burnUntil) {
      ward = math.max(0, ward - _burnPerTick);
    }
  }

  /// Ice holds the husk where it stands, every seal in view open. A charge
  /// stops dead.
  void _freeze(ArtSkill art, List<HuntEvent> events) {
    if (phase == HuntPhase.charge) {
      _endCharge();
      _moveStart = -1;
    }

    _frozenUntil = tick + art.freezeTicks;
    _burnUntil = tick;
    _halt();

    for (var index = 0; index < vessel.seals.length; index++) {
      _openSeal(index, ticks: art.freezeTicks, events: events);
    }
  }

  /// Wind knocks the husk off its ledge or out from behind its cover, turns
  /// a charge aside, and leaves it stunned, every seal in view open.
  void _gust(ArtSkill art, List<HuntEvent> events) {
    if (phase == HuntPhase.charge) {
      _endCharge();
      _moveStart = -1;
      _staggeredUntil = tick + art.gustStunTicks;
    } else if (_leaping) {
      _staggeredUntil = _moveStart + _moveTicks + art.gustStunTicks;
    } else if (_footing != Footing.open) {
      _leaveCover(events, mayStay: false);
      _staggeredUntil = tick + _moveTicks + art.gustStunTicks;
    } else {
      _halt();
      _staggeredUntil = tick + art.gustStunTicks;
    }

    _nextBoltTick = _staggeredUntil;
    _exposedUntil = _staggeredUntil + ArtRules.shakenTicks;
    _exposeAllSeals(events);
  }

  // Movement -----------------------------------------------------------

  /// Moves the husk on by one tick: along the move it is making, or into a
  /// new one when it is time to amble or to bolt.
  void _move(List<HuntEvent> events) {
    if (vessel.gait == Gait.still) {
      return;
    }

    // A leap, once begun, comes down, whatever else is happening.
    if (_leaping) {
      _continueMove();

      return;
    }

    // Frozen, it does not even turn.
    if (tick < _frozenUntil) {
      _halt();

      return;
    }

    // Drawing breath, a husk stops where it is and turns on the reaper.
    // Stunned, it does the same.
    if (phase == HuntPhase.windup ||
        phase == HuntPhase.shriek ||
        tick < _staggeredUntil) {
      _halt();
      _turnToward(_bearing + math.pi);

      return;
    }

    // Charging, it comes straight on with its face to the reaper.
    if (phase == HuntPhase.charge) {
      final progress = ((tick - _moveStart) / _moveTicks).clamp(0.0, 1.0);

      _range = _fromRange + _byRange * progress * progress;
      _height = _fromHeight + _byHeight * progress;
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
      _amble(events);
    }
  }

  void _continueMove() {
    final progress = (tick - _moveStart) / _moveTicks;

    if (progress >= 1) {
      _bearing = wrapAngle(_fromBearing + _byBearing);
      _range = _leaping
          ? _fromRange + _byRange
          : math.min(_fromRange + _byRange, _farAt(_bearing));
      _height = _fromHeight + _byHeight;
      _moveStart = -1;

      if (_leaping) {
        _land();
      }

      _nextAmbleTick = tick + _restTicks();

      return;
    }

    final eased = smoothstep(progress);
    final along = _fromRange + _byRange * eased;

    _bearing = wrapAngle(_fromBearing + _byBearing * eased);
    // Whatever stands between where it set off and where it is going, it
    // keeps to this side of: it goes round the front of the sofa, not
    // through it. A leap goes over the top instead.
    _range = _leaping ? along : math.min(along, _farAt(_bearing));
    _height = _fromHeight + _byHeight * eased + _arc * 4 * eased * (1 - eased);

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

  /// A short, unhurried move to somewhere nearby. Now and then, where the
  /// room has somewhere better to be, it leaps up onto a ledge or behind
  /// something instead, the way a cat would.
  void _amble(List<HuntEvent> events) {
    if (_footing != Footing.open) {
      _leaveCover(events, mayStay: true);
      return;
    }

    final spots = _spots(
      around: _bearing,
      within: radians(HuntRules.coverSearchDegrees),
    );

    if (spots.isNotEmpty &&
        tick >= _exposedUntil &&
        _cover.nextChance(HuntRules.ambleCoverChance)) {
      _leapTo(_cover.pick(spots), events);
      return;
    }

    final by = radians(
      _path.nextRange(-HuntRules.ambleDegrees, HuntRules.ambleDegrees),
    );

    final toBearing = plan.clamp(_bearing + by);

    _setOff(
      toBearing: toBearing,
      toRange: _rangeAt(toBearing, _path.nextRange(0, 1)),
      speed: vessel.walkSpeed,
      events: events,
    );
  }

  /// A dash to somewhere well away from where the reaper is aiming: into
  /// cover or up out of reach when the room offers it, unless it is burning
  /// or the wind has shaken it, when it wants nothing but open floor.
  void _bolt(List<HuntEvent> events) {
    _nextBoltTick = -1;

    if (vessel.gait == Gait.still) {
      return;
    }

    if (tick >= _exposedUntil) {
      final spots = _spots(
        around: _bearing,
        within: radians(HuntRules.boltMaxDegrees),
        beyond: radians(HuntRules.boltMinDegrees / 2),
      );

      if (spots.isNotEmpty && _cover.nextChance(HuntRules.boltCoverChance)) {
        _leapTo(_cover.pick(spots), events);
        events.add(const Relocating());
        return;
      }
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

    _setOff(
      toBearing: toBearing,
      // It runs for the far half of the room it has there.
      toRange: _rangeAt(toBearing, _path.nextRange(0.5, 1)),
      speed: vessel.boltSpeed,
      events: events,
    );
    events.add(const Relocating());
  }

  /// Leaps down from a ledge, or out from behind cover, onto the open floor
  /// in front of it. A husk that [mayStay] is only ready to move, and may
  /// settle down where it is a while longer.
  void _leaveCover(List<HuntEvent> events, {required bool mayStay}) {
    if (mayStay && _cover.nextChance(HuntRules.stayCoverChance)) {
      _nextAmbleTick = tick + _restTicks();
      return;
    }

    final toBearing = plan.clamp(_bearing + radians(_cover.nextRange(-20, 20)));

    _setOff(
      toBearing: toBearing,
      toRange: _rangeAt(toBearing, _cover.nextRange(0.2, 0.8)),
      speed: vessel.boltSpeed * HuntRules.leapPace,
      toHeight: vessel.gait == Gait.ground
          ? null
          : _cover.nextRange(vessel.lowMetres, vessel.highMetres),
      events: events,
    );
  }

  void _leapTo(_Spot spot, List<HuntEvent> events) => _setOff(
    toBearing: spot.bearing,
    toRange: spot.range,
    speed: vessel.boltSpeed * HuntRules.leapPace,
    toward: spot.footing,
    toHeight: spot.height,
    events: events,
  );

  /// Sets off for [toBearing] and [toRange] at [speed] metres a second,
  /// ending at [toHeight], or at the height it rests at when that is not
  /// given. A move onto, off or over anything in the room is a leap.
  void _setOff({
    required double toBearing,
    required double toRange,
    required double speed,
    required List<HuntEvent> events,
    Footing toward = Footing.open,
    double? toHeight,
  }) {
    _fromBearing = _bearing;
    _fromRange = _range;
    _fromHeight = _height;
    _byBearing = wrapAngle(toBearing - _bearing);
    _byRange = toRange - _range;
    _byHeight = (toHeight ?? _restingHeight()) - _height;

    final meanRange = _range + _byRange / 2;
    final length = math.sqrt(
      math.pow(meanRange * _byBearing, 2) +
          math.pow(_byRange, 2) +
          math.pow(_byHeight, 2),
    );

    _moveTicks = math.max(
      12,
      (length / (speed * difficulty.pace) * HuntRules.tickRate).round(),
    );
    _moveStart = tick;

    if (_footing == Footing.open && toward == Footing.open) {
      return;
    }

    _leaping = true;
    _toFooting = toward;
    _toGround = toward == Footing.ledge
        ? _height + _byHeight - vessel.halfHeight
        : _floor;
    _arc = _leapArc(toward: toward, toBearing: toBearing);
    _moveTicks = math.max(HuntRules.leapMinTicks, _moveTicks);
    events.add(Leapt(toward));
  }

  /// How far above the straight line a leap rises at its height: a hop,
  /// and enough more that its feet clear the front edge of the counter or
  /// the table it leaps onto, off or over, where it passes it.
  double _leapArc({required Footing toward, required double toBearing}) {
    final arc = HuntRules.hopMetres + 0.45 * _byHeight.abs();
    final edge = _footing == Footing.open ? toBearing : _bearing;
    final front = plan.reachDecimetres[HuntPlan.sliceOf(edge)] / 10;
    final from = _range;
    final to = _range + _byRange;

    if ((from - front) * (to - front) >= 0) {
      return arc;
    }

    // How far through the leap it passes the edge, kept off the very ends,
    // where no arc could lift it in time.
    final passing = ((front - from) / (to - from)).clamp(0.2, 0.8);
    final line = _height + _byHeight * passing;
    final clear =
        _floor +
        plan.coverAt(edge) +
        HuntRules.leapClearance +
        vessel.halfHeight;

    return math.max(arc, (clear - line) / (4 * passing * (1 - passing)));
  }

  void _land() {
    _leaping = false;
    _footing = _toFooting;
    _fromGround = _toGround;
    _arc = 0;
  }

  /// Where the move under way ends.
  Vec3 get _destination {
    final bearing = _fromBearing + _byBearing;
    final range = _fromRange + _byRange;

    return Vec3(
      math.sin(bearing) * range,
      _fromHeight + _byHeight,
      math.cos(bearing) * range,
    );
  }

  /// Stops the husk where it stands. A leap cannot be stopped in the air.
  void _halt() {
    if (_moveStart < 0 || _leaping) {
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

  /// Places within [within] radians of the bearing [around], and no nearer
  /// to it than [beyond], where the room lets the husk leap up onto a ledge
  /// or hide behind something. A place counts only where the ledge or the
  /// cover is as wide as the husk.
  List<_Spot> _spots({
    required double around,
    required double within,
    double beyond = 0,
  }) {
    if (!plan.knowsCover || vessel.gait == Gait.still) {
      return const [];
    }

    final spots = <_Spot>[];
    final farthest = vessel.farMetres + HuntRules.hideBeyond;

    for (var slice = 0; slice < HuntPlan.slices; slice++) {
      final bearing = HuntPlan.bearingOf(slice);
      final away = wrapAngle(bearing - around).abs();

      if (away > within || away < beyond || !plan.contains(bearing)) {
        continue;
      }

      final front = plan.reachDecimetres[slice] / 10;
      final hide = plan.hideAt(slice);

      if (vessel.gait == Gait.ground &&
          hide != null &&
          hide >= HuntRules.closestMetres &&
          hide <= farthest &&
          _spans(slice, hide, (other) => _hidesBody(other, hide))) {
        spots.add(
          _Spot(
            bearing: bearing,
            range: hide,
            footing: Footing.hidden,
            height: _floor + vessel.halfHeight,
          ),
        );
      }

      final ledge = plan.ledgeAt(slice);
      final onLedge = front + vessel.halfWidth * HuntRules.ledgeInset;

      if (ledge != null &&
          ledge >= HuntRules.lowestLedge &&
          ledge <= vessel.leapMetres &&
          front >= HuntRules.closestMetres &&
          onLedge <= farthest &&
          _spans(slice, onLedge, (other) => _sameLedge(other, slice))) {
        spots.add(
          _Spot(
            bearing: bearing,
            range: onLedge,
            footing: Footing.ledge,
            height: _floor + ledge + vessel.halfHeight,
          ),
        );
      }
    }

    return spots;
  }

  /// Whether every slice a husk [range] metres out along slice [slice] would
  /// stand across passes [test].
  bool _spans(int slice, double range, bool Function(int other) test) {
    final half = math.atan2(vessel.halfWidth, range);
    final reach = (half / HuntPlan.sliceWidth).ceil();

    for (var step = -reach; step <= reach; step++) {
      if (!test((slice + step) % HuntPlan.slices)) {
        return false;
      }
    }

    return true;
  }

  /// Whether the cover along slice [other] stands in front of a husk hiding
  /// [range] metres out, and is tall enough to hide at least half of it.
  bool _hidesBody(int other, double range) =>
      plan.coverCentimetres[other] / 100 >= vessel.halfHeight &&
      plan.reachDecimetres[other] / 10 < range - vessel.halfWidth / 2;

  /// Whether slice [other] has the same ledge as slice [slice]: as high, to
  /// within a hand, and as far away, to within a stride.
  bool _sameLedge(int other, int slice) {
    final ledge = plan.ledgeAt(other);
    final height = plan.ledgeAt(slice)!;

    return ledge != null &&
        (ledge - height).abs() <= 0.1 &&
        (plan.reachDecimetres[other] - plan.reachDecimetres[slice]).abs() <= 4;
  }

  /// The farthest the husk may be along [bearing]: as far as it likes to
  /// keep, or, where something stands in the way, just this side of it. In
  /// a corner too tight even for that, it stands as close as it ever does.
  double _farAt(double bearing) => math.max(
    HuntRules.closestMetres,
    math.min(vessel.farMetres, plan.reach(bearing) - vessel.halfWidth),
  );

  /// The nearest it may be along [bearing]: as near as it likes to come, or
  /// right up against whatever leaves it no more room than that.
  double _nearAt(double bearing) =>
      math.min(vessel.nearMetres, _farAt(bearing));

  /// A place along [bearing], [outward] of the way from the nearest it may
  /// be there to the farthest.
  double _rangeAt(double bearing, double outward) {
    final near = _nearAt(bearing);

    return near + outward * (_farAt(bearing) - near);
  }

  /// How high the middle of the body is while it rests: standing on the
  /// floor, or somewhere in the band of air it keeps to.
  double _restingHeight() => switch (vessel.gait) {
    Gait.ground => vessel.halfHeight - plan.eyeHeight,
    Gait.air ||
    Gait.still => _path.nextRange(vessel.lowMetres, vessel.highMetres),
  };
}

/// Somewhere the room lets a husk be that it has to leap to.
final class _Spot {
  const _Spot({
    required this.bearing,
    required this.range,
    required this.footing,
    required this.height,
  });

  final double bearing;
  final double range;
  final Footing footing;

  /// The height of the middle of the husk's body there, from the eye.
  final double height;
}
