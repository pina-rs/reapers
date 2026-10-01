import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';

import 'pose/ar_pose.dart';
import 'pose/pose_sources.dart';

/// Runs one hunt: feeds the rules what the player does and keeps the record
/// the server will judge.
///
/// The display calls [frame] as often as it draws. The rules advance at
/// their own fixed rate, so the same play gives the same hunt on a 60 Hz
/// phone and a 120 Hz one.
base class HuntController {
  HuntController({required this.sim, required this.poseSource});

  final HuntSim sim;

  /// Where the phone points. Whoever made the controller owns it, so one
  /// pose source can outlive several fights.
  final PoseSource poseSource;

  /// Every tick's input so far, in order.
  final recorded = <HuntInput>[];

  final _pendingTaps = <Vec3>[];
  double _unspent = 0;

  static const _tickSeconds = 1 / HuntRules.tickRate;

  /// A stalled frame catches up at most this many ticks, so a hitch never
  /// turns into a burst the player could not have played.
  static const _maxTicksPerFrame = 6;

  /// Where the phone points right now.
  ArPose get pose => poseSource.pose;

  /// The record of this hunt, for the server to replay.
  HuntLog get log => HuntLog(List.unmodifiable(recorded));

  /// Registers a tap that points along [ray]. It is played on the next tick.
  void tap(Vec3 ray) => _pendingTaps.add(ray);

  /// Advances the hunt by [deltaSeconds] of real time and returns what
  /// happened, for the view to show and sound.
  List<HuntEvent> frame(double deltaSeconds) {
    poseSource.advance(deltaSeconds);

    if (sim.isOver) {
      return const [];
    }

    final events = <HuntEvent>[];

    _unspent += deltaSeconds;

    for (
      var ticks = 0;
      ticks < _maxTicksPerFrame && _unspent >= _tickSeconds && !sim.isOver;
      ticks++
    ) {
      final input = readInput();

      _unspent -= _tickSeconds;
      recorded.add(input);
      events.addAll(sim.step(input));
    }

    // Time the display could not keep up with is dropped, not owed.
    if (_unspent > _tickSeconds) {
      _unspent = 0;
    }

    return events;
  }

  /// What the player did this tick: where the phone points, how fast it is
  /// turning, and any taps since the last tick.
  @protected
  HuntInput readInput() {
    final current = pose;
    final input = HuntInput.measured(
      aim: current.forward,
      turnRadiansPerSecond: current.turnSpeed,
      taps: List.of(_pendingTaps),
    );

    _pendingTaps.clear();

    return input;
  }
}

/// A hunt that plays itself, with a simulated player in place of a person.
///
/// It is how a hunt can be watched from start to finish on a simulator, in
/// a test or in a recording, where there is no phone to turn. The simulated
/// player has a person's limits: it needs a moment to react, and it misses.
final class AutoplayHuntController extends HuntController {
  AutoplayHuntController({required super.sim})
    : _bot = HuntBot.human(sim),
      super(poseSource: DragPoseSource());

  final HuntBot _bot;
  double _turnSpeed = 0;

  @override
  ArPose get pose => ArPose.looking(_bot.aim, turnSpeed: _turnSpeed);

  @override
  HuntInput readInput() {
    final input = _bot.decide();

    _turnSpeed = input.turnRadiansPerSecond;

    return input;
  }
}
