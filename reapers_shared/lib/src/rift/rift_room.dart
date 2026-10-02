import 'dart:math' as math;

import '../armory.dart';
import '../geo.dart';
import '../hunt/hunt_bot.dart';
import '../hunt/hunt_event.dart';
import '../hunt/hunt_input.dart';
import '../hunt/hunt_sim.dart';
import '../hunt/hunt_stats.dart';
import '../reaper_class.dart';
import '../reaper_names.dart';
import '../seeded_random.dart';
import '../vec3.dart';
import 'rift_frame.dart';
import 'rift_rules.dart';
import 'rift_site.dart';

/// A reaper arriving at a rift: who they are and what they fight with.
/// Where one reaper sees a rift's husk: which way it lies from them, and how
/// far off it is drawn.
final class RiftSighting {
  const RiftSighting({required this.bearing, required this.metres});

  /// Where a reaper [standing] sees a rift that stands at [anchor], or null
  /// when they are too far away to be said to be there.
  ///
  /// [north] is where north lies in the reaper's own view, as a bearing in
  /// the space their hunt is played in: it is what turns a direction on the
  /// earth into a direction the phone can be pointed.
  static RiftSighting? of({
    required GeoPoint anchor,
    required GeoPoint standing,
    required double north,
  }) {
    final metres = standing.distanceTo(anchor);

    if (metres > RiftRules.sightMetres) {
      return null;
    }

    return RiftSighting(
      bearing: wrapAngle(north + standing.bearingTo(anchor)),
      metres: metres.clamp(RiftRules.nearestMetres, RiftRules.farthestMetres),
    );
  }

  /// The way to the husk in the reaper's own view, in radians to the right
  /// of where they first faced.
  final double bearing;

  /// How far away the husk is drawn.
  final double metres;
}

final class RiftEntrant {
  const RiftEntrant({required this.name, required this.loadout});

  final String name;

  final HuntLoadout loadout;
}

/// What sealing a rift paid one reaper.
typedef RiftReward = ({int xp, int anima});

/// One reaper's own fight inside a rift.
///
/// Each reaper faces the husk themselves, with their own seals to strike and
/// their own arc to cut. What they share is the ward: every strike anyone
/// lands comes off the same total.
final class RiftSeat {
  /// [sighting] is where this reaper sees the husk. Without one it stands
  /// dead ahead of them.
  RiftSeat({
    required int seed,
    required RiftSite site,
    required double wardMax,
    required HuntLoadout loadout,
    RiftSighting? sighting,
  }) : sim = HuntSim.rift(
         seed: seed,
         vessel: site.vessel,
         ward: wardMax,
         stats: HuntStats.derive(loadout),
         sighting: sighting,
       );

  final HuntSim sim;

  /// Ward this seat has removed since the fight began.
  double dealt = 0;

  /// Advances this reaper's fight by one tick.
  List<HuntEvent> step(HuntInput input) {
    final events = sim.step(input);

    tally(events);

    return events;
  }

  /// Counts the ward [events] removed toward this seat's total. [step] does
  /// this itself; it is for whoever advances [sim] some other way.
  void tally(List<HuntEvent> events) {
    for (final event in events) {
      switch (event) {
        case StrikeLanded(:final damage) || StrikeGlanced(:final damage):
          dealt += damage;
        default:
          break;
      }
    }
  }

  /// Brings this seat's view of the ward up to date.
  ///
  /// [ward] is the circle's ward as the rift last reported it, and [counted]
  /// is how much of this seat's own damage that figure already includes.
  /// Strikes the rift has not heard of yet are taken off as well, so a blow
  /// shows the moment it lands instead of when the news comes back.
  void sync({required double ward, required double counted}) {
    if (!sim.isOver) {
      sim.shareWard(ward - (dealt - counted));
    }
  }

  /// Plays [ticks] of standing still, to join a fight that has already
  /// started in step with everyone else in it.
  void catchUp(int ticks) {
    for (var tick = 0; tick < ticks && !sim.isOver; tick++) {
      step(const HuntInput(aim: Aim.ahead, turnSpeed: 0));
    }
  }

  /// This seat's standing, to send to the rift.
  RiftReport get report =>
      RiftReport(ready: true, dealt: dealt, outcome: sim.outcome);
}

/// One rift from the first reaper arriving to the last cut.
///
/// The room is the keeper of everything the circle shares: who is in it, how
/// much ward is left, and whether the rift was sealed. It reads no clock and
/// rolls no dice of its own: whoever hosts it, the game alone or the server
/// for many, calls [advance] as time passes and [report] as reapers act.
final class RiftRoom {
  RiftRoom({required this.site, required this.seed});

  final RiftSite site;

  /// Decides the husk's timetable and who the shades are.
  final int seed;

  final _members = <_Member>[];

  RiftPhase phase = RiftPhase.gathering;

  /// Ticks since the fight began.
  int tick = 0;

  /// The ward the husk raised against this circle. Zero until the fight
  /// begins, since it depends on who came.
  double wardMax = 0;

  /// Where on the earth the rift stands: decided by the first reaper to say
  /// where they are, and the same for everyone after. Null until then, and
  /// for good if nobody in the circle can say.
  GeoPoint? anchor;

  int _gatherTicks = 0;
  int? _countdown;
  int _reapEndsTick = 0;

  /// A reap is given this long past its window before the room gives up on
  /// reapers who have not said how theirs ended.
  static const _reportGraceTicks = 45;

  /// Whether another reaper may still join.
  bool get isOpen =>
      phase == RiftPhase.gathering && _present.length < RiftRules.circle;

  /// Whether any reaper who joined is still here.
  bool get isAbandoned => _present.every((member) => member.shade);

  /// What is left of the ward.
  double get ward {
    var dealt = 0.0;

    for (final member in _members) {
      dealt += member.dealt;
    }

    return math.max(0, wardMax - dealt);
  }

  Iterable<_Member> get _present => _members.where((member) => !member.left);

  /// Seats [entrant] in the circle and returns their seat.
  int join(RiftEntrant entrant) {
    if (!isOpen) {
      throw StateError('This rift is not taking reapers');
    }

    _members.add(_Member(entrant, shade: false));

    return _members.length - 1;
  }

  /// Records that the reaper in [seat] has gone. What they dealt still
  /// counts, but they no longer stand in the circle. Once the rift is over
  /// there is nothing left to walk away from, and leaving changes nothing.
  void leave(int seat) {
    if (phase.isOver) {
      return;
    }

    _members[seat].left = true;

    if (phase == RiftPhase.gathering) {
      _countdown = null;
    }
  }

  /// Takes in what the reaper in [seat] says of their own fight.
  void report(int seat, RiftReport report) {
    final member = _members[seat];

    if (member.left || phase.isOver) {
      return;
    }

    member.ready = member.ready || report.ready;

    if (phase == RiftPhase.gathering) {
      // The rift takes its place while the circle gathers, and keeps it:
      // moving it mid-fight would turn every reaper's phone away from it.
      if (report.standing case final standing?) {
        anchor ??= site.anchorFor(standing);
      }

      return;
    }

    member.dealt = math.max(member.dealt, report.dealt);
    member.outcome ??= report.outcome;
  }

  /// Moves the rift on by [ticks] ticks.
  void advance(int ticks) {
    for (var passed = 0; passed < ticks && !phase.isOver; passed++) {
      switch (phase) {
        case RiftPhase.gathering:
          _gather();
        case RiftPhase.fight:
          _fight();
        case RiftPhase.reap:
          _reap();
        case RiftPhase.sealed || RiftPhase.failed:
          break;
      }
    }
  }

  /// The rift as the reaper in [seat] sees it right now.
  RiftFrame frameFor(int seat) {
    final shown = _present.toList();
    final reward = rewardFor(seat);

    return RiftFrame(
      site: site,
      seed: seed,
      phase: phase,
      seat: shown.indexOf(_members[seat]),
      tick: tick,
      ticksLeft: switch (phase) {
        RiftPhase.gathering =>
          _countdown ?? RiftRules.gatherLimitTicks - _gatherTicks,
        RiftPhase.fight => RiftRules.fightLimitTicks - tick,
        RiftPhase.reap => math.max(0, _reapEndsTick - tick),
        RiftPhase.sealed || RiftPhase.failed => 0,
      },
      countingDown: _countdown != null,
      ward: ward,
      wardMax: wardMax,
      members: [
        for (final member in shown)
          RiftMember(
            name: member.entrant.name,
            reaperClass: member.entrant.loadout.reaperClass,
            level: member.entrant.loadout.level,
            shade: member.shade,
            ready: member.ready,
            standing: member.standing,
            dealt: member.dealt,
            finished: member.outcome != null,
            cut: member.outcome?.grade,
          ),
      ],
      xp: reward.xp,
      anima: reward.anima,
      anchor: anchor,
    );
  }

  /// What the rift paid the reaper in [seat]. Nothing unless it was sealed
  /// and they stayed to the end.
  RiftReward rewardFor(int seat) {
    final member = _members[seat];

    if (phase != RiftPhase.sealed || member.left) {
      return (xp: 0, anima: 0);
    }

    final fairShare = wardMax / RiftRules.circle;
    final pull =
        RiftRules.idleShare +
        (1 - RiftRules.idleShare) * math.min(1, member.dealt / fairShare);
    final part = switch (member.outcome) {
      HuntOutcome(:final grade?) => grade.reward,
      _ when !member.standing => RiftRules.castOutShare,
      _ => RiftRules.uncutShare,
    };

    return (
      xp: (RiftRules.xp * pull * part).round(),
      anima: (RiftRules.anima * pull * part).round(),
    );
  }

  // Gathering ----------------------------------------------------------

  void _gather() {
    _gatherTicks++;

    final present = _present.toList();
    final countdown = _countdown;

    if (countdown != null) {
      _countdown = countdown - 1;

      if (countdown <= 1) {
        _begin();
      }

      return;
    }

    if (present.isNotEmpty && present.every((member) => member.ready)) {
      _countdown = RiftRules.countdownTicks;
      return;
    }

    // Nobody waits forever on a reaper who walked away with the app open.
    if (present.isNotEmpty && _gatherTicks >= RiftRules.gatherLimitTicks) {
      _countdown = RiftRules.countdownTicks;
    }
  }

  void _begin() {
    final reapers = _present.toList();

    if (reapers.isEmpty) {
      phase = RiftPhase.failed;
      return;
    }

    var levels = 0;

    for (final reaper in reapers) {
      levels += reaper.entrant.loadout.level;
    }

    final shadeLevel = math.max(
      1,
      (levels / reapers.length).round() - RiftRules.shadeLevelGap,
    );

    for (var empty = reapers.length; empty < RiftRules.circle; empty++) {
      _members.add(
        _Member(_shade(empty, shadeLevel), shade: true)..ready = true,
      );
    }

    wardMax = RiftRules.wardFor([
      for (final member in _present) HuntStats.derive(member.entrant.loadout),
    ]);

    for (final member in _present.where((member) => member.shade)) {
      final seat = RiftSeat(
        seed: seed,
        site: site,
        wardMax: wardMax,
        loadout: member.entrant.loadout,
      );

      member
        ..seat = seat
        // A shade is slower to see a seal and wider of the mark than a
        // careful person, so company is always worth more than shades.
        ..bot = HuntBot(seat.sim, reactionTicks: 34, missEvery: 3);
    }

    _countdown = null;
    phase = RiftPhase.fight;
    tick = 0;
  }

  /// The shade that fills seat [index] when nobody else does.
  RiftEntrant _shade(int index, int level) {
    final random = SeededRandom(mixSeed([seed, 0x5AADE, index]));
    final reaperClass = random.pick(ReaperClass.values);
    final temper = Temper.values.lastWhere(
      (temper) => temper.levelRequired <= level,
    );

    return RiftEntrant(
      name: random.pick(reaperGivenNames),
      loadout: HuntLoadout(
        reaperClass: reaperClass,
        level: level,
        weapon: Weapon(reaperClass.weapon, temper),
      ),
    );
  }

  // Fight and reap -----------------------------------------------------

  void _fight() {
    tick++;
    _stepShades();

    if (ward <= 0) {
      phase = RiftPhase.reap;
      _reapEndsTick = tick + RiftRules.reapWindowTicks;
      return;
    }

    final outOfTime = tick >= RiftRules.fightLimitTicks;
    final nobodyLeft = !_present.any((member) => member.standing);

    if (outOfTime || nobodyLeft) {
      phase = RiftPhase.failed;
    }
  }

  void _reap() {
    tick++;
    _stepShades();

    final standing = _present.where((member) => member.standing).toList();
    final everyoneDone = standing.every((member) => member.outcome != null);

    if (!everyoneDone && tick < _reapEndsTick + _reportGraceTicks) {
      return;
    }

    final cuts = _present
        .where((member) => member.outcome?.grade != null)
        .toList();
    final enough = cuts.length >= RiftFrame.cutsNeededFor(standing.length);
    // Shades can break a ward, but only the living can shut a rift: at least
    // one of the cuts has to be a person's.
    final byTheLiving = cuts.any((member) => !member.shade);

    phase = enough && byTheLiving ? RiftPhase.sealed : RiftPhase.failed;
  }

  void _stepShades() {
    final shared = ward;

    for (final member in _members) {
      final seat = member.seat;
      final bot = member.bot;

      if (seat == null || bot == null || seat.sim.isOver) {
        continue;
      }

      seat
        ..sync(ward: shared, counted: seat.dealt)
        ..step(bot.decide());
      member
        ..dealt = seat.dealt
        ..outcome = seat.sim.outcome;
    }
  }
}

final class _Member {
  _Member(this.entrant, {required this.shade});

  final RiftEntrant entrant;
  final bool shade;

  bool ready = false;
  bool left = false;
  double dealt = 0;
  HuntOutcome? outcome;

  /// A shade's own fight, played by the room. Null for a person, whose fight
  /// is played on their phone.
  RiftSeat? seat;
  HuntBot? bot;

  /// Still in the fight: here, and not cast out by flinching.
  bool get standing => !left && outcome?.escape != EscapeCause.resolve;
}
