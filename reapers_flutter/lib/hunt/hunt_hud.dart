import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../theme/veil.dart';
import 'hunt_controller.dart';

/// A mark left on the screen where a strike landed, for a fraction of a
/// second.
final class Slash {
  Slash({required this.at, required this.angle, required this.strength});

  final Offset at;

  /// The direction of the cut, in radians.
  final double angle;

  /// How hard the strike was, from 0 (a whiff) to 1 (an empowered hit).
  final double strength;

  double age = 0;

  static const lifetime = 0.26;

  bool get spent => age >= lifetime;
}

/// Everything drawn over the hunt that is anchored to the world: seals on
/// the husk, the reap arc, the mark of each strike, and where to look when
/// the husk is out of view.
///
/// It reads the rules and the pose every frame and decides nothing.
class HuntHudPainter extends CustomPainter {
  HuntHudPainter({
    required this.controller,
    required this.light,
    required this.verticalFov,
    required this.slashes,
    required this.showSealOrder,
    required this.absorbed,
    required this.wound,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final HuntController controller;

  /// The husk's own colour.
  final Color light;

  final double verticalFov;

  final List<Slash> slashes;

  /// Whether open seals are numbered in the order they opened.
  final bool showSealOrder;

  /// How far an absorbed husk has come apart, from 0 to 1. Zero until the
  /// husk is reaped.
  final double Function() absorbed;

  /// How fresh the last blow the reaper took is, from 1 (just struck) down
  /// to 0.
  final double Function() wound;

  /// The radius of a seal on screen. The target is at least 44 points
  /// across, however far away the husk is.
  static const sealRadius = 26.0;

  HuntSim get _sim => controller.sim;

  @override
  void paint(Canvas canvas, Size size) {
    final aspect = size.width / size.height;
    final pose = controller.pose;

    Offset? place(Vec3 direction) {
      final point = pose.project(
        direction,
        verticalFov: verticalFov,
        aspect: aspect,
      );

      if (point == null) {
        return null;
      }

      return Offset(
        (point.x + 1) / 2 * size.width,
        (1 - point.y) / 2 * size.height,
      );
    }

    final seconds = _sim.tick / HuntRules.tickRate;

    switch (_sim.phase) {
      case HuntPhase.stalk:
        _paintPointer(canvas, size, place);
        _paintStalkReticle(canvas, size);
      case HuntPhase.fight || HuntPhase.windup:
        _paintPointer(canvas, size, place);
        _paintSeals(canvas, place, seconds);
      case HuntPhase.shriek:
        _paintPointer(canvas, size, place);
        _paintHoldRing(canvas, size);
      case HuntPhase.charge:
        _paintPointer(canvas, size, place);
        _paintSeals(canvas, place, seconds);
        _paintChargeRing(canvas, size, place);
      case HuntPhase.broken:
        _paintReapArc(canvas, size, place, seconds);
      case HuntPhase.absorbed:
        _paintEssence(
          canvas,
          size,
          place(_sim.pose.direction) ?? size.center(Offset.zero),
        );
      case HuntPhase.escaped:
        break;
    }

    if (_sim.phase == HuntPhase.windup || _sim.phase == HuntPhase.shriek) {
      _paintShriekEdge(canvas, size);
    }

    _paintWound(canvas, size);

    _paintSlashes(canvas);
  }

  /// An arrow at the edge of the screen toward a husk that is out of view.
  void _paintPointer(Canvas canvas, Size size, Offset? Function(Vec3) place) {
    final husk = _sim.pose.direction;

    paintEdgePointer(
      canvas,
      size,
      target: husk,
      onScreen: place(husk),
      aim: controller.pose.forward,
      colour: light,
      pulse: 0.55 + 0.45 * math.sin(_sim.tick / 9),
    );
  }

  /// The ring at the middle of the screen that closes as the reaper steadies
  /// on an unaware husk, and frays as the husk grows alarmed.
  void _paintStalkReticle(Canvas canvas, Size size) {
    final centre = size.center(Offset.zero);
    final steady = _sim.steadiness;
    final alarm = _sim.awareness;
    final radius = 40 - 18 * Curves.easeOut.transform(steady);
    final colour = Color.lerp(Veil.bone, Veil.wound, alarm)!;
    final jitter = alarm * 3 * math.sin(_sim.tick * 1.7);

    canvas.drawCircle(
      centre.translate(jitter, 0),
      radius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1
        ..color = colour.withValues(alpha: 0.35),
    );
    canvas.drawArc(
      Rect.fromCircle(center: centre.translate(jitter, 0), radius: radius),
      -math.pi / 2,
      math.pi * 2 * steady,
      false,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.4
        ..strokeCap = StrokeCap.round
        ..color = colour,
    );
    canvas.drawCircle(centre, 1.6, Paint()..color = colour);
  }

  void _paintSeals(
    Canvas canvas,
    Offset? Function(Vec3) place,
    double seconds,
  ) {
    final pose = _sim.pose;

    for (var index = 0; index < _sim.vessel.seals.length; index++) {
      if (!_sim.sealOpen(index)) {
        continue;
      }

      final at = place(pose.sealDirection(index));

      if (at == null) {
        continue;
      }

      final rank = _sim.sealRank(index) ?? 0;
      final first = showSealOrder && rank == 0;
      final beat = 0.5 + 0.5 * math.sin(seconds * 5 + index * 1.7);
      final ring = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = first ? 2.6 : 1.8
        ..color = light.withValues(alpha: 0.75 + 0.25 * beat);

      canvas
        ..drawCircle(
          at,
          sealRadius * 1.5,
          Paint()
            ..shader =
                RadialGradient(
                  colors: [
                    light.withValues(alpha: 0.32),
                    light.withValues(alpha: 0),
                  ],
                ).createShader(
                  Rect.fromCircle(center: at, radius: sealRadius * 1.5),
                ),
        )
        ..drawCircle(at, sealRadius - 3 * beat, ring)
        ..drawCircle(at, 3.2, Paint()..color = Veil.bone);

      // Four ticks turning around the ring: a seal reads as a mechanism
      // that is open, not as a button.
      for (var tick = 0; tick < 4; tick++) {
        final angle = seconds * 1.4 + index + tick * math.pi / 2;
        final direction = Offset(math.cos(angle), math.sin(angle));

        canvas.drawLine(
          at + direction * (sealRadius + 3),
          at + direction * (sealRadius + 9),
          ring,
        );
      }

      if (showSealOrder) {
        _paintLabel(
          canvas,
          romanNumeral(rank + 1),
          at.translate(0, -sealRadius - 18),
        );
      }
    }
  }

  /// While the husk shrieks: a still ring that shows how much of the hold
  /// is left.
  void _paintHoldRing(Canvas canvas, Size size) {
    final centre = size.center(Offset.zero);
    final left = 1 - _sim.shriekProgress;

    canvas
      ..drawCircle(
        centre,
        46,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = Veil.bone.withValues(alpha: 0.3),
      )
      ..drawArc(
        Rect.fromCircle(center: centre, radius: 46),
        -math.pi / 2,
        math.pi * 2 * left,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 3
          ..strokeCap = StrokeCap.round
          ..color = Veil.bone,
      );
  }

  /// While the husk charges: a ring that closes on it as it comes. When
  /// the ring has closed, it has arrived.
  void _paintChargeRing(
    Canvas canvas,
    Size size,
    Offset? Function(Vec3) place,
  ) {
    final centre = place(_sim.pose.direction) ?? size.center(Offset.zero);
    final progress = _sim.chargeProgress;
    final radius = size.shortestSide * (0.62 - 0.44 * progress);

    canvas.drawCircle(
      centre,
      radius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1 + 3 * progress
        ..color = Veil.bone.withValues(alpha: 0.35 + 0.55 * progress),
    );
  }

  /// A blow the reaper took: the edges of the screen bruise, and the bruise
  /// fades. It swells once and goes; it does not flash.
  void _paintWound(Canvas canvas, Size size) {
    final strength = Curves.easeOut.transform(wound().clamp(0.0, 1.0));

    if (strength <= 0) {
      return;
    }

    final bounds = Offset.zero & size;

    canvas.drawRect(
      bounds,
      Paint()
        ..shader = RadialGradient(
          radius: 0.95,
          colors: [
            Veil.wound.withValues(alpha: 0),
            Veil.wound.withValues(alpha: 0.12 * strength),
            Veil.wound.withValues(alpha: 0.5 * strength),
          ],
          stops: const [0.3, 0.65, 1],
        ).createShader(bounds),
    );
  }

  /// A warning that closes in from the edges of the screen: the world goes
  /// dark around the husk. It swells and holds; it never flashes, and it is
  /// never red.
  void _paintShriekEdge(Canvas canvas, Size size) {
    final strength = _sim.phase == HuntPhase.shriek
        ? 1.0
        : Curves.easeIn.transform(_sim.shriekProgress);
    final bounds = Offset.zero & size;

    canvas.drawRect(
      bounds,
      Paint()
        ..shader = RadialGradient(
          radius: 0.9,
          colors: [
            Veil.ink.withValues(alpha: 0),
            Veil.ink.withValues(alpha: 0.25 * strength),
            Veil.ink.withValues(alpha: 0.94 * strength),
          ],
          stops: const [0.18, 0.5, 1],
        ).createShader(bounds),
    );
  }

  /// What is left of an absorbed husk, streaming down into the reaper.
  void _paintEssence(Canvas canvas, Size size, Offset from) {
    final amount = absorbed();

    if (amount <= 0 || amount >= 1) {
      return;
    }

    final into = Offset(size.width / 2, size.height + 40);
    final paint = Paint();

    for (var mote = 0; mote < _essenceMotes; mote++) {
      // Each mote leaves a little after the last and takes its own curve.
      final delay = mote / _essenceMotes * 0.55;
      final travel = ((amount - delay) / 0.45).clamp(0.0, 1.0);

      if (travel <= 0 || travel >= 1) {
        continue;
      }

      final spread = math.sin(mote * 12.9898) * 110;
      final eased = Curves.easeInCubic.transform(travel);
      final start =
          from + Offset(math.sin(mote * 4.1) * 70, math.cos(mote * 2.7) * 90);
      final bend = Offset(spread * math.sin(travel * math.pi), 0);

      paint.color = light.withValues(alpha: (1 - travel) * 0.9 + 0.1);
      canvas.drawCircle(
        Offset.lerp(start, into, eased)! + bend,
        2.6 * (1 - 0.5 * travel),
        paint,
      );
    }
  }

  static const _essenceMotes = 46;

  void _paintReapArc(
    Canvas canvas,
    Size size,
    Offset? Function(Vec3) place,
    double seconds,
  ) {
    const samples = 28;
    final points = [
      for (var sample = 0; sample <= samples; sample++)
        place(_sim.reapArcPoint(sample / samples)),
    ];
    final start = points.first;
    final reached = (_sim.reapProgress * samples).floor();

    void stroke(int from, int to, Paint paint) {
      final path = Path();
      var drawing = false;

      for (var index = from; index <= to; index++) {
        final point = points[index];

        if (point == null) {
          drawing = false;
          continue;
        }

        if (drawing) {
          path.lineTo(point.dx, point.dy);
        } else {
          path.moveTo(point.dx, point.dy);
          drawing = true;
        }
      }

      canvas.drawPath(path, paint);
    }

    final faint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round
      ..color = Veil.bone.withValues(alpha: 0.4);
    final cut = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 5
      ..strokeCap = StrokeCap.round
      ..color = light
      ..maskFilter = const MaskFilter.blur(BlurStyle.solid, 5);

    stroke(0, samples, faint);

    if (_sim.reapBegun) {
      stroke(0, reached, cut);
    }

    if (start != null && !_sim.reapBegun) {
      final beat = 0.5 + 0.5 * math.sin(seconds * 6);

      canvas
        ..drawCircle(
          start,
          14 + 8 * beat,
          faint..color = light.withValues(alpha: 1 - beat),
        )
        ..drawCircle(start, 7, Paint()..color = light);
    } else if (start == null) {
      _paintPointerTo(canvas, size, _sim.reapArcPoint(0));
    }

    // Where the reaper is aiming: the tip of the blade.
    final centre = size.center(Offset.zero);

    canvas
      ..drawCircle(
        centre,
        11,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = Veil.bone,
      )
      ..drawCircle(centre, 2, Paint()..color = Veil.bone);
  }

  void _paintPointerTo(Canvas canvas, Size size, Vec3 target) {
    final aim = controller.pose.forward;
    final angle = math.atan2(
      -(target.pitch - aim.pitch),
      wrapAngle(target.yaw - aim.yaw),
    );
    final at =
        size.center(Offset.zero) +
        Offset(
          math.cos(angle) * (size.width / 2 - 34),
          math.sin(angle) * (size.height / 2 - 120),
        );

    canvas.drawCircle(at, 8, Paint()..color = light);
  }

  void _paintSlashes(Canvas canvas) {
    for (final slash in slashes) {
      final life = (slash.age / Slash.lifetime).clamp(0.0, 1.0);
      final reach =
          (30 + 60 * slash.strength) * Curves.easeOutCubic.transform(life);
      final direction = Offset(math.cos(slash.angle), math.sin(slash.angle));
      final fade = 1 - life;

      canvas.drawLine(
        slash.at - direction * reach,
        slash.at + direction * reach,
        Paint()
          ..strokeWidth = (1.5 + 4 * slash.strength) * fade
          ..strokeCap = StrokeCap.round
          ..color = Color.lerp(light, Veil.bone, 0.6)!.withValues(alpha: fade)
          ..maskFilter = MaskFilter.blur(BlurStyle.solid, 3 * slash.strength),
      );
    }
  }

  void _paintLabel(Canvas canvas, String text, Offset at) {
    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: Veil.display(16, color: Veil.bone),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    painter.paint(canvas, at - Offset(painter.width / 2, painter.height / 2));
  }

  @override
  bool shouldRepaint(HuntHudPainter oldDelegate) => false;
}

/// An arrow at the edge of the screen pointing toward [target] when it is
/// out of view: [onScreen] is where it falls on the screen, if anywhere, and
/// [aim] is where the phone points. [pulse], from 0 to 1, is how brightly
/// it shows this frame. The arrow runs round the oval that fills [within],
/// or most of the view when that is not given.
void paintEdgePointer(
  Canvas canvas,
  Size size, {
  required Vec3 target,
  required Offset? onScreen,
  required Vec3 aim,
  required Color colour,
  required double pulse,
  Rect? within,
}) {
  final visible =
      onScreen != null && (Offset.zero & size).deflate(24).contains(onScreen);

  if (visible) {
    return;
  }

  final turn = wrapAngle(target.yaw - aim.yaw);
  final lift = target.pitch - aim.pitch;
  final angle = math.atan2(-lift, turn);
  final track =
      within ??
      Rect.fromCenter(
        center: size.center(Offset.zero),
        width: size.width - 68,
        height: size.height - 240,
      );
  final at =
      track.center +
      Offset(
        math.cos(angle) * track.width / 2,
        math.sin(angle) * track.height / 2,
      );
  final paint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 2.2
    ..strokeCap = StrokeCap.round
    ..strokeJoin = StrokeJoin.round;

  canvas
    ..save()
    ..translate(at.dx, at.dy)
    ..rotate(angle);

  for (final offset in [0.0, -11.0]) {
    canvas.drawPath(
      Path()
        ..moveTo(offset - 7, -11)
        ..lineTo(offset + 5, 0)
        ..lineTo(offset - 7, 11),
      paint..color = colour.withValues(alpha: pulse * (offset == 0 ? 1 : 0.4)),
    );
  }

  canvas.restore();
}
