import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../theme/veil.dart';
import '../hunt_hud.dart';
import '../pose/ar_pose.dart';

/// How brightly the next light of [trace] shows this moment, from about a
/// half to 1: it breathes, slowly, so that the eye finds it.
double _breath(SigilTrace trace) =>
    0.62 + 0.38 * math.sin(trace.seconds * 2 * math.pi * 0.8);

/// The sigil hung in the room: its lights where they are around the player,
/// the thread drawn through those that have caught, the ring that rests on
/// them, and which way to turn when the next light is out of view.
///
/// It reads the trace and the pose every frame and decides nothing.
class SigilPainter extends CustomPainter {
  SigilPainter({
    required this.trace,
    required this.pose,
    required this.verticalFov,
    required this.light,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final SigilTrace trace;

  /// Where the phone points, read each time the sigil is drawn.
  final ArPose Function() pose;

  /// The vertical field of view the room is seen through, in radians.
  final double Function() verticalFov;

  /// The husk's own colour, which its sigil burns in.
  final Color light;

  final _numerals = <int, TextPainter>{};

  /// How far the thread from the last light to the ring is split, so that
  /// it bends as the room does when one end is far round the player.
  static const _threadSteps = 16;

  @override
  void paint(Canvas canvas, Size size) {
    final view = pose();
    final fov = verticalFov();
    final aspect = size.width / size.height;
    final centre = size.center(Offset.zero);
    final lights = trace.sigil.lights.length;
    final breath = _breath(trace);

    Offset? place(Vec3 direction) {
      final point = view.project(
        direction,
        verticalFov: fov,
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

    // How big a light's reach looks on screen.
    final reach =
        math.tan(trace.sigil.radius) / math.tan(fov / 2) * size.height / 2;
    final faint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round
      ..color = light.withValues(alpha: 0.2);
    final glow = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 9
      ..strokeCap = StrokeCap.round
      ..color = light.withValues(alpha: 0.16);
    final bright = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.4
      ..strokeCap = StrokeCap.round
      ..color = light.withValues(alpha: 0.95);

    // The whole of the sigil, faintly, so that its shape can be seen in the
    // room before it is traced.
    for (var step = math.max(0, trace.caught - 1); step + 1 < lights; step++) {
      _thread(
        canvas,
        place,
        trace.directionOf(step),
        trace.directionOf(step + 1),
        faint,
      );
    }

    // What has been traced.
    for (var step = 0; step + 1 < trace.caught; step++) {
      final from = trace.directionOf(step);
      final to = trace.directionOf(step + 1);

      _thread(canvas, place, from, to, glow);
      _thread(canvas, place, from, to, bright);
    }

    // The thread being drawn now, from the last light to the ring. It
    // fades, and reddens, as that light comes near to going out.
    if (trace.caught > 0 && !trace.isClosed) {
      final fraying = trace.fraying;

      _thread(
        canvas,
        place,
        trace.directionOf(trace.caught - 1),
        view.forward,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round
          ..color = Color.lerp(
            light,
            Veil.wound,
            fraying,
          )!.withValues(alpha: 0.85 * (1 - 0.7 * fraying)),
      );
    }

    for (var index = 0; index < lights; index++) {
      final at = place(trace.directionOf(index));

      if (at == null) {
        continue;
      }

      if (index < trace.caught) {
        canvas
          ..drawCircle(at, 15, Paint()..color = light.withValues(alpha: 0.22))
          ..drawCircle(at, 6.5, Paint()..color = light);
      } else if (index == trace.caught) {
        canvas
          ..drawCircle(
            at,
            reach,
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.6
              ..color = light.withValues(alpha: 0.7 * breath),
          )
          ..drawCircle(at, 5, Paint()..color = light.withValues(alpha: breath));
        _numeral(
          canvas,
          index,
          at.translate(reach * 0.72, -reach * 0.72),
          next: true,
        );
      } else {
        canvas.drawCircle(
          at,
          5,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.4
            ..color = light.withValues(alpha: 0.5),
        );
        _numeral(canvas, index, at.translate(10, -10), next: false);
      }
    }

    _paintRing(canvas, centre, view);

    if (!trace.isClosed) {
      final next = trace.directionOf(trace.caught);

      paintEdgePointer(
        canvas,
        size,
        target: next,
        onScreen: place(next),
        aim: view.forward,
        colour: light,
        pulse: breath,
      );
    }
  }

  /// The ring at the middle of the screen: what rests on a light. It fills
  /// as the light catches.
  void _paintRing(Canvas canvas, Offset centre, ArPose view) {
    final resting = trace.restsOn(view.forward);

    canvas.drawCircle(
      centre,
      18,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1
        ..color = Veil.bone.withValues(alpha: resting ? 0.85 : 0.45),
    );

    if (trace.fill <= 0) {
      return;
    }

    canvas.drawArc(
      Rect.fromCircle(center: centre, radius: 24),
      -math.pi / 2,
      math.pi * 2 * trace.fill,
      false,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..strokeCap = StrokeCap.round
        ..color = light,
    );
  }

  /// The stretch from [from] to [to], bent with the room. Wherever part of
  /// it is behind the player, only what is in front is drawn.
  void _thread(
    Canvas canvas,
    Offset? Function(Vec3) place,
    Vec3 from,
    Vec3 to,
    Paint paint,
  ) {
    final path = Path();
    var drawing = false;

    for (var step = 0; step <= _threadSteps; step++) {
      final share = step / _threadSteps;
      final at = place((from * (1 - share) + to * share).normalized);

      if (at == null) {
        drawing = false;
      } else if (drawing) {
        path.lineTo(at.dx, at.dy);
      } else {
        path.moveTo(at.dx, at.dy);
        drawing = true;
      }
    }

    canvas.drawPath(path, paint);
  }

  /// The light at [index]'s place in the order, centred on [at]: full
  /// when it is the [next] to catch, and dim when it is still to come.
  void _numeral(Canvas canvas, int index, Offset at, {required bool next}) {
    final painter = _numerals.putIfAbsent(
      index * 2 + (next ? 1 : 0),
      () => TextPainter(
        text: TextSpan(
          text: romanNumeral(index + 1),
          style: Veil.figure(
            color: Veil.bone.withValues(alpha: next ? 1 : 0.5),
            size: 13,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout(),
    );

    painter.paint(
      canvas,
      at.translate(-painter.width / 2, -painter.height / 2),
    );
  }

  @override
  bool shouldRepaint(SigilPainter oldDelegate) =>
      oldDelegate.trace != trace || oldDelegate.light != light;
}

/// The whole sigil at once, flat, as it would be drawn on paper: what the
/// player is tracing, how far they have got, and where they are pointing
/// now.
///
/// In the room only a few of the lights are in view at a time. This shows
/// them all.
class SigilGlyphPainter extends CustomPainter {
  SigilGlyphPainter({
    required this.trace,
    required this.pose,
    required this.light,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final SigilTrace trace;
  final ArPose Function() pose;
  final Color light;

  static const _inset = 10.0;

  @override
  void paint(Canvas canvas, Size size) {
    final lights = trace.sigil.lights;
    final yaws = [for (final light in lights) light.yaw];
    final pitches = [for (final light in lights) light.pitch];
    final (left, right) = (yaws.reduce(math.min), yaws.reduce(math.max));
    final (low, high) = (pitches.reduce(math.min), pitches.reduce(math.max));
    final breath = _breath(trace);

    // Across and up are kept to the same scale, so the glyph has the shape
    // the sigil has in the room.
    final scale = math.min(
      (size.width - 2 * _inset) / math.max(right - left, 1e-6),
      (size.height - 2 * _inset) / math.max(high - low, 1e-6),
    );
    final middle = Offset((left + right) / 2, (low + high) / 2);
    final centre = size.center(Offset.zero);

    Offset at(double yaw, double pitch) =>
        centre +
        Offset((yaw - middle.dx) * scale, -(pitch - middle.dy) * scale);

    Offset ofLight(int index) => at(lights[index].yaw, lights[index].pitch);

    for (var step = 0; step + 1 < lights.length; step++) {
      final traced = step + 1 < trace.caught;

      canvas.drawLine(
        ofLight(step),
        ofLight(step + 1),
        Paint()
          ..strokeWidth = traced ? 2.2 : 1
          ..strokeCap = StrokeCap.round
          ..color = light.withValues(alpha: traced ? 0.95 : 0.28),
      );
    }

    // Where the phone points now, held to the edge of the glyph when it is
    // off it, with the thread from the last light running to it.
    final view = pose().forward;
    final pointing = at(wrapAngle(view.yaw - trace.centre), view.pitch);
    final aim = Offset(
      pointing.dx.clamp(0, size.width),
      pointing.dy.clamp(0, size.height),
    );

    if (trace.caught > 0 && !trace.isClosed) {
      canvas.drawLine(
        ofLight(trace.caught - 1),
        aim,
        Paint()
          ..strokeWidth = 1.6
          ..strokeCap = StrokeCap.round
          ..color = Color.lerp(
            light,
            Veil.wound,
            trace.fraying,
          )!.withValues(alpha: 0.7),
      );
    }

    for (var index = 0; index < lights.length; index++) {
      final place = ofLight(index);

      if (index < trace.caught) {
        canvas.drawCircle(place, 4, Paint()..color = light);
      } else if (index == trace.caught) {
        canvas
          ..drawCircle(
            place,
            7,
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.4
              ..color = light.withValues(alpha: breath),
          )
          ..drawCircle(place, 2.5, Paint()..color = light);
      } else {
        canvas.drawCircle(
          place,
          3,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.2
            ..color = light.withValues(alpha: 0.55),
        );
      }
    }

    canvas
      ..drawLine(
        aim.translate(-4, 0),
        aim.translate(4, 0),
        Paint()
          ..strokeWidth = 1.4
          ..color = Veil.bone,
      )
      ..drawLine(
        aim.translate(0, -4),
        aim.translate(0, 4),
        Paint()
          ..strokeWidth = 1.4
          ..color = Veil.bone,
      );
  }

  @override
  bool shouldRepaint(SigilGlyphPainter oldDelegate) =>
      oldDelegate.trace != trace || oldDelegate.light != light;
}
