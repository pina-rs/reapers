import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../theme/veil.dart';
import 'pose/ar_pose.dart';

/// The spirit realm with no world behind it: drifting motes and a faint
/// horizon, for when there is no camera to look through.
///
/// The motes are fixed in the hunt's space, so they slide across the screen
/// as the view turns. That is what tells the player they are turning at all.
class VeilBackdrop extends HookWidget {
  const VeilBackdrop({
    super.key,
    required this.pose,
    required this.verticalFov,
    required this.repaint,
  });

  /// Read each time the backdrop paints.
  final ArPose Function() pose;

  final double verticalFov;

  /// Ticks once a frame.
  final Listenable repaint;

  @override
  Widget build(BuildContext context) => CustomPaint(
    painter: _VeilPainter(
      pose: pose,
      verticalFov: verticalFov,
      repaint: repaint,
    ),
    size: Size.infinite,
  );
}

class _VeilPainter extends CustomPainter {
  _VeilPainter({
    required this.pose,
    required this.verticalFov,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final ArPose Function() pose;
  final double verticalFov;

  /// Directions of the motes, scattered once over the whole sphere.
  static final _motes = _scatter(260);

  static List<({Vec3 direction, double size, double glow})> _scatter(
    int count,
  ) {
    final random = SeededRandom(0x7E11);

    return [
      for (var index = 0; index < count; index++)
        (
          direction: Vec3.fromAngles(
            random.nextRange(-math.pi, math.pi),
            math.asin(random.nextRange(-0.85, 0.95)),
          ),
          size: random.nextRange(0.6, 2.4),
          glow: random.nextRange(0.15, 0.8),
        ),
    ];
  }

  @override
  void paint(Canvas canvas, Size size) {
    final current = pose();
    final aspect = size.width / size.height;

    canvas.drawRect(Offset.zero & size, Paint()..color = Veil.ink);
    _paintHorizon(canvas, size, current);

    final paint = Paint();

    for (final mote in _motes) {
      final point = current.project(
        mote.direction,
        verticalFov: verticalFov,
        aspect: aspect,
      );

      if (point == null || point.x.abs() > 1.1 || point.y.abs() > 1.1) {
        continue;
      }

      paint.color = Veil.bone.withValues(alpha: mote.glow * 0.5);
      canvas.drawCircle(
        Offset((point.x + 1) / 2 * size.width, (1 - point.y) / 2 * size.height),
        mote.size,
        paint,
      );
    }
  }

  /// A band of cold light where the ground would meet the sky.
  void _paintHorizon(Canvas canvas, Size size, ArPose current) {
    final level = current.project(
      Vec3.fromAngles(current.forward.yaw, 0),
      verticalFov: verticalFov,
      aspect: size.width / size.height,
    );

    if (level == null) {
      return;
    }

    final y = (1 - level.y) / 2 * size.height;
    final band = Rect.fromLTRB(
      0,
      y - size.height * 0.7,
      size.width,
      y + size.height * 0.5,
    );

    canvas.drawRect(
      band,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            Color(0x00121824),
            Color(0xFF16202C),
            Color(0xFF0A0D13),
            Color(0x00050608),
          ],
          stops: [0, 0.58, 0.64, 1],
        ).createShader(band),
    );
  }

  @override
  bool shouldRepaint(_VeilPainter oldDelegate) => false;
}
