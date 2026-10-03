import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

/// What came of a simulated reaper tracing a sigil.
typedef TraceResult = ({bool closed, double seconds, int lost});

/// Traces [trace] the way a person would: turning toward the next light at
/// [turnSpeed] radians a second until the view is well inside it, then
/// holding the phone still, with [wobble] radians a second of the hand's
/// own tremor, until it catches.
///
/// Gives up after [limitSeconds].
TraceResult traceSigil(
  SigilTrace trace, {
  double turnSpeed = 1.0,
  double wobble = 0.15,
  Vec3? from,
  double limitSeconds = 60,
}) {
  const frame = 1 / 60;
  var aim = from ?? Vec3.fromAngles(trace.centre, radians(-25));
  var seconds = 0.0;
  var lost = 0;

  while (!trace.isClosed && seconds < limitSeconds) {
    final target = trace.directionOf(trace.caught);
    final away = aim.angleTo(target);
    final settled = away <= trace.sigil.radius * 0.4;
    final speed = settled ? wobble : turnSpeed;

    if (!settled) {
      aim = _turnToward(aim, target, math.min(away, turnSpeed * frame));
    }

    for (final event in trace.advance(
      aim,
      turnSpeed: speed,
      deltaSeconds: frame,
    )) {
      if (event is LightLost) {
        lost++;
      }
    }

    seconds += frame;
  }

  return (closed: trace.isClosed, seconds: seconds, lost: lost);
}

/// [from] turned [angle] radians toward [to], along the shortest way round.
Vec3 _turnToward(Vec3 from, Vec3 to, double angle) {
  final between = from.angleTo(to);

  if (between <= angle || between == 0) {
    return to;
  }

  final share = angle / between;
  final sine = math.sin(between);

  return (from * (math.sin((1 - share) * between) / sine) +
          to * (math.sin(share * between) / sine))
      .normalized;
}
