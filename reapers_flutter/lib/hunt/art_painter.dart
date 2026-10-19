import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:reapers_shared/reapers_shared.dart';

import 'hunt_controller.dart';

/// Where a thrown art came down, kept for as long as its mark lasts.
final class ArtImpact {
  ArtImpact({required this.art, required this.at, required this.hit});

  final Art art;

  /// Where it came down, in metres from the eye.
  final Vec3 at;

  /// Whether it caught the husk.
  final bool hit;

  /// Seconds since it came down.
  double age = 0;

  static const lifetime = 0.8;

  bool get spent => age >= lifetime;

  /// How far through its mark it is, from 0 to 1.
  double get progress => (age / lifetime).clamp(0.0, 1.0);
}

/// The colours each art is drawn in: its bright core and the glow round it.
({Color core, Color glow}) artColours(Art art) => switch (art) {
  Art.cinder => (core: const Color(0xFFFFE2A0), glow: const Color(0xFFFF6A1A)),
  Art.rime => (core: const Color(0xFFF2FBFF), glow: const Color(0xFF7FCDFF)),
  Art.gale => (core: const Color(0xFFF1F7F2), glow: const Color(0xFF9FE3C9)),
};

/// Everything an art does that is drawn in the room: what is in the air,
/// where it came down, and the fire on a burning husk.
///
/// It is drawn with the husk and hidden with it: an ember lobbed behind the
/// counter goes out of sight behind the counter. Sizes are in metres, so an
/// ember far off is small. It decides nothing.
class ArtPainter extends CustomPainter {
  ArtPainter({
    required this.controller,
    required this.impacts,
    required this.verticalFov,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final HuntController controller;

  final List<ArtImpact> impacts;

  final double verticalFov;

  HuntSim get _sim => controller.sim;

  @override
  void paint(Canvas canvas, Size size) {
    final aspect = size.width / size.height;
    final pose = controller.pose;
    final perMetre = size.height / (2 * math.tan(verticalFov / 2));

    /// Where [point], in metres from the eye, is on the screen, and how
    /// many pixels a metre there is across.
    ({Offset at, double scale})? place(Vec3 point) {
      final distance = point.length;
      final projected = pose.project(
        point.normalized,
        verticalFov: verticalFov,
        aspect: aspect,
      );

      if (projected == null || distance < 0.05) {
        return null;
      }

      return (
        at: Offset(
          (projected.x + 1) / 2 * size.width,
          (1 - projected.y) / 2 * size.height,
        ),
        scale: perMetre / distance,
      );
    }

    if (_sim.burning) {
      _paintFlames(canvas, place);
    }

    for (final flight in _sim.flights) {
      _paintFlight(canvas, flight, place);
    }

    for (final impact in impacts) {
      _paintImpact(canvas, impact, place);
    }
  }

  void _paintFlight(
    Canvas canvas,
    ArtFlight flight,
    ({Offset at, double scale})? Function(Vec3) place,
  ) {
    final goal = flight.homing ? _sim.pose.position : flight.target;
    final colours = artColours(flight.art);
    final tick = _sim.tick;

    switch (flight.art) {
      case Art.cinder:
        // An ember, and the sparks it leaves behind it.
        for (var back = 6; back >= 0; back--) {
          final spot = place(flight.position(tick - back * 2, goal));

          if (spot == null) {
            continue;
          }

          final radius =
              (back == 0 ? 0.11 : 0.06 * (1 - back / 7)) * spot.scale;

          _glow(
            canvas,
            spot.at,
            radius * 2.6,
            colours.glow,
            0.55 - back * 0.06,
          );
          canvas.drawCircle(
            spot.at,
            radius,
            Paint()..color = colours.core.withValues(alpha: 1 - back * 0.13),
          );
        }
      case Art.rime:
        // A shard: a bright streak along its line of flight.
        final head = place(flight.position(tick, goal));
        final tail = place(flight.position(tick - 3, goal));

        if (head == null || tail == null) {
          return;
        }

        _glow(canvas, head.at, 0.16 * head.scale, colours.glow, 0.5);
        canvas
          ..drawLine(
            tail.at,
            head.at,
            Paint()
              ..color = colours.glow.withValues(alpha: 0.7)
              ..strokeWidth = 0.07 * head.scale
              ..strokeCap = StrokeCap.round,
          )
          ..drawLine(
            tail.at,
            head.at,
            Paint()
              ..color = colours.core
              ..strokeWidth = 0.025 * head.scale
              ..strokeCap = StrokeCap.round,
          );
      case Art.gale:
        // A knot of wind: arcs that turn as they go.
        final spot = place(flight.position(tick, goal));

        if (spot == null) {
          return;
        }

        final turn = tick * 0.35;

        for (var arc = 0; arc < 3; arc++) {
          final radius = (0.18 + arc * 0.09) * spot.scale;

          canvas.drawArc(
            Rect.fromCircle(center: spot.at, radius: radius),
            turn + arc * 2.1,
            2.2,
            false,
            Paint()
              ..color = colours.core.withValues(alpha: 0.75 - arc * 0.18)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 0.025 * spot.scale
              ..strokeCap = StrokeCap.round,
          );
        }
    }
  }

  void _paintImpact(
    Canvas canvas,
    ArtImpact impact,
    ({Offset at, double scale})? Function(Vec3) place,
  ) {
    final spot = place(impact.at);

    if (spot == null) {
      return;
    }

    final colours = artColours(impact.art);
    final along = impact.progress;
    final fade = 1 - along;
    final eased = Curves.easeOutCubic.transform(along);

    switch (impact.art) {
      case Art.cinder:
        final reach = ArtRules.burstMetres * spot.scale * eased;

        _glow(canvas, spot.at, reach * 1.2, colours.glow, 0.8 * fade);
        canvas.drawCircle(
          spot.at,
          reach,
          Paint()
            ..color = colours.core.withValues(alpha: 0.6 * fade)
            ..style = PaintingStyle.stroke
            ..strokeWidth = 0.05 * spot.scale * fade,
        );

        // Sparks thrown out from the burst.
        for (var spark = 0; spark < 12; spark++) {
          final angle = spark * 2 * math.pi / 12 + spark * 0.37;
          final out = reach * (0.6 + 0.5 * ((spark * 7) % 5) / 5);

          canvas.drawCircle(
            spot.at +
                Offset(math.cos(angle), math.sin(angle) - eased * 0.4) * out,
            0.025 * spot.scale * fade,
            Paint()..color = colours.core.withValues(alpha: fade),
          );
        }
      case Art.rime:
        // A star of ice where it struck; a scatter of splinters where it
        // broke on something else.
        final length = (impact.hit ? 0.45 : 0.25) * spot.scale * (0.4 + eased);
        final spikes = impact.hit ? 6 : 9;

        _glow(canvas, spot.at, length, colours.glow, 0.5 * fade);

        for (var spike = 0; spike < spikes; spike++) {
          final angle = spike * 2 * math.pi / spikes + 0.3;
          final end =
              spot.at +
              Offset(math.cos(angle), math.sin(angle)) *
                  length *
                  (impact.hit ? 1 : 0.5 + (spike % 3) / 4);

          canvas.drawLine(
            impact.hit ? spot.at : spot.at + (end - spot.at) * eased * 0.6,
            end,
            Paint()
              ..color = colours.core.withValues(alpha: fade)
              ..strokeWidth = 0.02 * spot.scale
              ..strokeCap = StrokeCap.round,
          );
        }
      case Art.gale:
        for (var ring = 0; ring < 2; ring++) {
          final grown = (eased - ring * 0.18).clamp(0.0, 1.0);

          canvas.drawCircle(
            spot.at,
            1.1 * spot.scale * grown,
            Paint()
              ..color = colours.core.withValues(alpha: 0.6 * fade)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 0.03 * spot.scale,
          );
        }
    }
  }

  /// Tongues of fire licking up the husk's body while it burns.
  void _paintFlames(
    Canvas canvas,
    ({Offset at, double scale})? Function(Vec3) place,
  ) {
    final pose = _sim.pose;
    final vessel = _sim.vessel;
    final seconds = _sim.tick / HuntRules.tickRate;
    final colours = artColours(Art.cinder);
    final across = Vec3(math.cos(pose.yaw), 0, -math.sin(pose.yaw));

    for (var tongue = 0; tongue < 6; tongue++) {
      // Each tongue rises and dies on its own beat.
      final beat = (seconds * (1.3 + tongue % 3 * 0.25) + tongue * 0.37) % 1;
      final side = ((tongue * 0.618) % 1) * 2 - 1;
      final rise = -0.3 + 1.4 * beat;
      final at =
          pose.position +
          across * (side * vessel.halfWidth * 0.75) +
          Vec3.up * (rise * vessel.halfHeight);
      final spot = place(at);

      if (spot == null) {
        continue;
      }

      final alpha = math.sin(beat * math.pi);
      final height = 0.14 * spot.scale * (1 - beat * 0.5);
      final width = height * 0.32;
      // A tongue of flame: round at the foot, drawn up to a point.
      final tip = spot.at - Offset(0, height * 0.6);
      final flame = Path()
        ..moveTo(tip.dx, tip.dy)
        ..quadraticBezierTo(
          spot.at.dx + width,
          spot.at.dy,
          spot.at.dx,
          spot.at.dy + height * 0.4,
        )
        ..quadraticBezierTo(spot.at.dx - width, spot.at.dy, tip.dx, tip.dy);

      _glow(canvas, spot.at, height * 0.8, colours.glow, 0.3 * alpha);
      canvas.drawPath(
        flame,
        Paint()..color = colours.core.withValues(alpha: 0.6 * alpha),
      );
    }
  }

  static void _glow(
    Canvas canvas,
    Offset at,
    double radius,
    Color colour,
    double alpha,
  ) {
    if (radius <= 0 || alpha <= 0) {
      return;
    }

    canvas.drawCircle(
      at,
      radius,
      Paint()
        ..shader = RadialGradient(
          colors: [
            colour.withValues(alpha: alpha.clamp(0.0, 1.0)),
            colour.withValues(alpha: 0),
          ],
        ).createShader(Rect.fromCircle(center: at, radius: radius)),
    );
  }

  @override
  bool shouldRepaint(covariant ArtPainter oldDelegate) =>
      oldDelegate.controller != controller ||
      oldDelegate.verticalFov != verticalFov;
}
