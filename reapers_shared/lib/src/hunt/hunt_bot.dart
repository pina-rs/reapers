import '../arts.dart';
import '../vec3.dart';
import 'hunt_event.dart';
import 'hunt_input.dart';
import 'hunt_sim.dart';
import 'hunt_stats.dart';
import 'husk_pose.dart';

/// A simulated player: plays a hunt the way a careful person would.
///
/// The bot reads only what a player can see: where the husk is, which seals
/// are open, and what phase the hunt is in. It turns at a human pace, taps
/// open seals as fast as the weapon allows, holds still through a shriek and
/// sweeps the reap arc at a steady speed. A bot whose reaper knows an art
/// throws it when it is what the moment calls for.
///
/// It exists for three jobs: the rules' own tests and pacing checks, the
/// game's autoplay (which lets a hunt be watched without being played), and
/// filling a rift with company when too few reapers have gathered.
final class HuntBot {
  HuntBot(
    this.sim, {
    this.turnSpeed = 1.2,
    this.reapSpeed = 0.9,
    this.reactionTicks = 0,
    this.missEvery = 0,
    this.chargeNerve = 0.4,
  }) : _openSince = List.filled(sim.vessel.seals.length, -1);

  /// A bot with a person's limits: it needs a moment to notice a seal has
  /// opened, and now and then it taps wide.
  HuntBot.human(HuntSim sim) : this(sim, reactionTicks: 16, missEvery: 7);

  final HuntSim sim;

  /// How fast the bot turns toward a target, in radians a second.
  final double turnSpeed;

  /// How fast the bot sweeps the reap arc, in radians a second.
  final double reapSpeed;

  /// Ticks a seal must have been open before the bot taps it.
  final int reactionTicks;

  /// Every this-many taps lands wide of the husk. Zero means never.
  final int missEvery;

  /// How far through a charge the bot lets the husk come before it swings,
  /// from 0 to 1. Above 1 it never swings, and takes the blow.
  final double chargeNerve;

  final List<int> _openSince;
  int _taps = 0;

  final recorded = <HuntInput>[];
  final events = <HuntEvent>[];

  Vec3 _aim = Vec3.forward;
  double _reapTarget = 0;

  /// Plays until the hunt ends or [maxTicks] pass.
  HuntOutcome? play({int maxTicks = HuntLog.maxTicks}) {
    while (!sim.isOver && sim.tick < maxTicks) {
      events.addAll(stepOnce());
    }

    return sim.outcome;
  }

  /// Advances one tick and returns its events.
  List<HuntEvent> stepOnce() {
    final input = decide();

    recorded.add(input);

    return sim.step(input);
  }

  /// Where the bot is looking, after its last decision.
  Vec3 get aim => _aim;

  /// What the bot does this tick. Whoever calls this must feed the result
  /// to the hunt, since the bot assumes each decision is acted on.
  HuntInput decide() {
    switch (sim.phase) {
      case HuntPhase.stalk:
        return _turnToward(sim.pose.direction, speed: turnSpeed);
      case HuntPhase.shriek:
        return _hold();
      case HuntPhase.fight || HuntPhase.windup:
        return _wantsToThrow ? _cast() : _fight();
      case HuntPhase.charge:
        return _wantsToThrow && sim.chargeProgress < chargeNerve
            ? _cast()
            : _cut();
      case HuntPhase.broken:
        return _reap();
      case HuntPhase.absorbed || HuntPhase.escaped:
        return _hold();
    }
  }

  HuntInput _fight() {
    final pose = sim.pose;
    final input = _turnToward(pose.direction, speed: turnSpeed);
    final open = <int>[];

    for (var index = 0; index < sim.vessel.seals.length; index++) {
      if (!sim.sealOpen(index)) {
        _openSince[index] = -1;
        continue;
      }

      if (_openSince[index] < 0) {
        _openSince[index] = sim.tick;
      }

      if (sim.tick - _openSince[index] >= reactionTicks) {
        open.add(index);
      }
    }

    // Stop striking as the warning ends, so no tap lands inside the shriek.
    final nearShriek =
        sim.phase == HuntPhase.windup && sim.shriekProgress > 0.8;

    if (open.isEmpty || nearShriek || sim.tick % sim.stats.swingTicks != 0) {
      return input;
    }

    open.sort((a, b) => (sim.sealRank(a) ?? 0).compareTo(sim.sealRank(b) ?? 0));

    _taps++;

    final wide = missEvery > 0 && _taps % missEvery == 0;
    final target = wide
        ? Vec3.fromAngles(pose.yaw + radians(35), pose.pitch)
        : pose.sealDirection(open.first);

    return HuntInput(
      aim: input.aim,
      turnSpeed: input.turnSpeed,
      taps: [Aim.fromDirection(target)],
    );
  }

  /// Whether to reach for the art: a throw is ready, nothing is in the air,
  /// and the husk is where this art is for.
  bool get _wantsToThrow {
    final art = sim.stats.art;
    final ready =
        art != null &&
        sim.artCharges > 0 &&
        sim.flights.isEmpty &&
        !sim.frozen &&
        !(sim.phase == HuntPhase.windup && sim.shriekProgress > 0.4);

    if (!ready) {
      return false;
    }

    final away = sim.footing != Footing.open && !sim.leaping;

    return switch (art.art) {
      // An ember is lobbed at a place, so it is thrown at a husk at rest.
      Art.cinder => !sim.burning && sim.speed == 0 && !sim.leaping,
      Art.rime =>
        sim.phase == HuntPhase.charge || sim.artCharges == art.maxCharges,
      Art.gale =>
        away ||
            sim.phase == HuntPhase.charge ||
            sim.artCharges == art.maxCharges,
    };
  }

  /// Holds the art with one thumb and steadies on the husk, tapping with
  /// the other as it would anyway, then flicks.
  HuntInput _cast() {
    if (sim.artGathered >= 1 && _aim.angleTo(sim.pose.direction) < radians(4)) {
      return HuntInput.measured(
        aim: _aim,
        turnRadiansPerSecond: ArtRules.throwSpeed * 1.25,
        focus: true,
      );
    }

    final input = sim.phase == HuntPhase.charge
        ? _turnToward(sim.pose.direction, speed: turnSpeed)
        : _fight();

    return HuntInput(
      aim: input.aim,
      turnSpeed: input.turnSpeed,
      taps: input.taps,
      focus: true,
    );
  }

  HuntInput _reap() {
    if (!sim.reapBegun) {
      _reapTarget = 0;

      return _turnToward(sim.reapArcPoint(0), speed: turnSpeed);
    }

    _reapTarget = (_reapTarget + reapSpeed / HuntRules.tickRate / 0.9).clamp(
      0.0,
      1.05,
    );

    return _turnToward(sim.reapArcPoint(_reapTarget), speed: reapSpeed);
  }

  /// Meets a charge: keeps fighting until the husk is well on its way, then
  /// swings across it, hard enough to turn it aside.
  HuntInput _cut() {
    if (sim.chargeProgress < chargeNerve) {
      return _fight();
    }

    return _turnToward(
      Vec3.fromAngles(_aim.yaw + radians(40), _aim.pitch),
      speed: sim.stats.parrySpeed * 1.2,
    );
  }

  HuntInput _hold() => HuntInput.measured(aim: _aim, turnRadiansPerSecond: 0);

  /// Turns the aim toward [target] at no more than [speed], and reports the
  /// speed actually turned at.
  HuntInput _turnToward(Vec3 target, {required double speed}) {
    final angle = _aim.angleTo(target);
    final step = speed / HuntRules.tickRate;

    if (angle <= step) {
      final turned = angle * HuntRules.tickRate;

      _aim = target.normalized;

      return HuntInput.measured(aim: _aim, turnRadiansPerSecond: turned);
    }

    final share = step / angle;

    _aim = Vec3.fromAngles(
      _aim.yaw + wrapAngle(target.yaw - _aim.yaw) * share,
      _aim.pitch + (target.pitch - _aim.pitch) * share,
    );

    return HuntInput.measured(aim: _aim, turnRadiansPerSecond: speed);
  }
}
