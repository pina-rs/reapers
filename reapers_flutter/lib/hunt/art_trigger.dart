import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../l10n/l10n.dart';
import '../l10n/lookups.dart';
import '../theme/veil.dart';
import 'art_painter.dart';
import 'hunt_controller.dart';

/// Where the reaper's art is held: a thumb rests here to gather it while
/// the phone is held steady on where it should go, and a flick of the
/// phone throws it.
///
/// It sits in the lower corner, under the thumb of the hand that holds the
/// phone, so the other hand is free to strike.
class ArtTrigger extends HookWidget {
  const ArtTrigger({
    super.key,
    required this.art,
    required this.controller,
    required this.light,
    required this.repaint,
  });

  final ArtSkill art;

  final HuntController controller;

  /// The husk's own colour.
  final Color light;

  /// Ticks once a frame.
  final Listenable repaint;

  /// How wide the trigger is: well past the 44 points a target needs.
  static const diameter = 84.0;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final held = useState(false);
    // The hint stays until the art has been thrown once.
    final thrown = useRef(false);

    void hold(bool down) {
      if (held.value == down) {
        return;
      }

      held.value = down;
      controller.focus = down;

      if (down) {
        HapticFeedback.selectionClick();
      }
    }

    useEffect(
      () =>
          () => controller.focus = false,
      [controller],
    );

    return Positioned(
      right: 18,
      bottom: 132,
      child: Semantics(
        button: true,
        label: l10n.huntArtSemantics(artName(l10n, art.art)),
        child: Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: (_) => hold(true),
          onPointerUp: (_) => hold(false),
          onPointerCancel: (_) => hold(false),
          child: ListenableBuilder(
            listenable: repaint,
            builder: (context, _) {
              final sim = controller.sim;

              // Once the hunt is over there is nothing left to throw at.
              if (sim.isOver) {
                return const SizedBox.shrink();
              }

              if (sim.flights.isNotEmpty) {
                thrown.value = true;
              }

              return Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  SizedBox.square(
                    dimension: diameter,
                    child: CustomPaint(
                      painter: _TriggerPainter(
                        art: art,
                        charges: sim.artCharges,
                        gathered: held.value ? sim.artGathered : 0,
                        recharge: sim.artRecharge,
                        held: held.value,
                      ),
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    (thrown.value
                            ? artName(l10n, art.art)
                            : held.value && sim.artGathered >= 1
                            ? l10n.huntArtFlick
                            : l10n.huntArtHint)
                        .toUpperCase(),
                    style: Veil.label(
                      color: held.value ? artColours(art.art).glow : Veil.bone,
                    ),
                  ),
                ],
              );
            },
          ),
        ),
      ),
    );
  }
}

class _TriggerPainter extends CustomPainter {
  _TriggerPainter({
    required this.art,
    required this.charges,
    required this.gathered,
    required this.recharge,
    required this.held,
  });

  final ArtSkill art;
  final int charges;
  final double gathered;
  final double recharge;
  final bool held;

  @override
  void paint(Canvas canvas, Size size) {
    final centre = size.center(Offset.zero);
    final radius = size.shortestSide / 2 - 4;
    final colours = artColours(art.art);
    final ready = charges > 0;

    // The well the art sits in.
    canvas.drawCircle(
      centre,
      radius,
      Paint()..color = Veil.ink.withValues(alpha: held ? 0.75 : 0.55),
    );
    canvas.drawCircle(
      centre,
      radius,
      Paint()
        ..color = (ready ? colours.glow : Veil.ash).withValues(alpha: 0.6)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2,
    );

    // It brightens as it gathers.
    if (gathered > 0) {
      canvas.drawCircle(
        centre,
        radius * (0.55 + 0.45 * gathered),
        Paint()
          ..shader = RadialGradient(
            colors: [
              colours.glow.withValues(alpha: 0.55 * gathered),
              colours.glow.withValues(alpha: 0),
            ],
          ).createShader(Rect.fromCircle(center: centre, radius: radius)),
      );
      canvas.drawArc(
        Rect.fromCircle(center: centre, radius: radius),
        -math.pi / 2,
        2 * math.pi * gathered,
        false,
        Paint()
          ..color = colours.core
          ..style = PaintingStyle.stroke
          ..strokeWidth = 3.5
          ..strokeCap = StrokeCap.round,
      );
    }

    // The next throw readying, faintly, inside the rim.
    if (recharge > 0) {
      canvas.drawArc(
        Rect.fromCircle(center: centre, radius: radius - 6),
        -math.pi / 2,
        2 * math.pi * recharge,
        false,
        Paint()
          ..color = Veil.ash.withValues(alpha: 0.6)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5,
      );
    }

    _paintGlyph(
      canvas,
      centre,
      radius * 0.42,
      ready ? colours.core : Veil.ash,
    );

    // One diamond for every throw that is ready, under the glyph.
    for (var pip = 0; pip < art.maxCharges; pip++) {
      final at =
          centre + Offset((pip - (art.maxCharges - 1) / 2) * 11, radius * 0.62);
      final diamond = Path()
        ..moveTo(at.dx, at.dy - 4)
        ..lineTo(at.dx + 4, at.dy)
        ..lineTo(at.dx, at.dy + 4)
        ..lineTo(at.dx - 4, at.dy)
        ..close();

      canvas.drawPath(
        diamond,
        Paint()
          ..color = pip < charges ? colours.core : Veil.ash
          ..style = pip < charges ? PaintingStyle.fill : PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  /// The art's mark: a flame, a six-pointed star of ice, a curl of wind.
  void _paintGlyph(Canvas canvas, Offset centre, double size, Color colour) {
    final stroke = Paint()
      ..color = colour
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.2
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final at = centre - Offset(0, size * 0.25);

    switch (art.art) {
      case Art.cinder:
        final flame = Path()
          ..moveTo(at.dx, at.dy - size)
          ..cubicTo(
            at.dx + size * 0.9,
            at.dy - size * 0.1,
            at.dx + size * 0.7,
            at.dy + size * 0.8,
            at.dx,
            at.dy + size * 0.8,
          )
          ..cubicTo(
            at.dx - size * 0.7,
            at.dy + size * 0.8,
            at.dx - size * 0.8,
            at.dy,
            at.dx - size * 0.2,
            at.dy - size * 0.35,
          )
          ..quadraticBezierTo(
            at.dx - size * 0.05,
            at.dy + size * 0.1,
            at.dx,
            at.dy - size,
          );

        canvas.drawPath(flame, stroke);
      case Art.rime:
        for (var arm = 0; arm < 6; arm++) {
          final angle = arm * math.pi / 3;
          final tip = at + Offset(math.cos(angle), math.sin(angle)) * size;
          final mid = at + (tip - at) * 0.6;
          final side =
              Offset(math.cos(angle + 0.9), math.sin(angle + 0.9)) *
              size *
              0.25;
          final other =
              Offset(math.cos(angle - 0.9), math.sin(angle - 0.9)) *
              size *
              0.25;

          canvas
            ..drawLine(at, tip, stroke)
            ..drawLine(mid, mid + side, stroke)
            ..drawLine(mid, mid + other, stroke);
        }
      case Art.gale:
        for (var curl = 0; curl < 3; curl++) {
          final y = at.dy + (curl - 1) * size * 0.55;
          final reach = size * (1 - curl * 0.18);
          final wind = Path()
            ..moveTo(at.dx - reach, y)
            ..lineTo(at.dx + reach * 0.45, y)
            ..arcToPoint(
              Offset(at.dx + reach * 0.45, y - size * 0.4),
              radius: Radius.circular(size * 0.2),
              clockwise: false,
            );

          canvas.drawPath(wind, stroke);
        }
    }
  }

  @override
  bool shouldRepaint(covariant _TriggerPainter old) =>
      old.art != art ||
      old.charges != charges ||
      old.gathered != gathered ||
      old.recharge != recharge ||
      old.held != held;
}
