import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../hunt/hunt_arena.dart';
import '../hunt/pose/ar_pose.dart';
import '../hunt/start_hunt.dart';
import '../hunt/veil_backdrop.dart';
import '../l10n/l10n.dart';
import '../theme/veil.dart';
import '../widgets/ledger.dart';
import 'oath_screen.dart';

/// The first thing an unbound player sees: the name, the premise, and the
/// two ways in.
class ThresholdScreen extends HookConsumerWidget {
  const ThresholdScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final drift = useAnimationController(duration: const Duration(minutes: 4))
      ..repeat();
    final arrival = useAnimationController(
      duration: const Duration(milliseconds: 1600),
    )..forward();

    return Scaffold(
      body: Stack(
        fit: StackFit.expand,
        children: [
          // The veil turns slowly behind the page, so it is never still.
          VeilBackdrop(
            verticalFov: huntFieldOfView,
            repaint: drift,
            pose: () => ArPose.looking(
              Vec3.fromAngles(drift.value * math.pi * 2, 0.12),
            ),
          ),
          CustomPaint(painter: _BladePainter(arrival)),
          SafeArea(
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: Veil.contentWidth),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(28, 24, 28, 28),
                  child: FadeTransition(
                    opacity: CurvedAnimation(
                      parent: arrival,
                      curve: const Interval(0.35, 1, curve: Curves.easeOut),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          l10n.thresholdOverline.toUpperCase(),
                          style: Veil.label(),
                        ),
                        const Spacer(),
                        Text(
                          l10n.appTitle,
                          key: const Key('threshold-title'),
                          style: Veil.display(92, weight: 360),
                        ),
                        const SizedBox(height: 18),
                        Text(
                          l10n.thresholdTagline,
                          style: Veil.body(color: Veil.ash, size: 16),
                        ),
                        const SizedBox(height: 34),
                        LedgerButton(
                          key: const Key('threshold-oath'),
                          label: l10n.thresholdOath,
                          detail: l10n.thresholdOathDetail,
                          onPressed: () => Navigator.of(context).push(
                            MaterialPageRoute<void>(
                              builder: (_) => const OathScreen(),
                            ),
                          ),
                        ),
                        const SizedBox(height: 4),
                        LedgerButton(
                          key: const Key('threshold-shade'),
                          label: l10n.thresholdShade,
                          detail: l10n.thresholdShadeDetail,
                          quiet: true,
                          onPressed: () => startPracticeHunt(context, ref),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// One long blade drawn across the page as it opens: the same arc the
/// player will later sweep with their phone.
class _BladePainter extends CustomPainter {
  _BladePainter(this.arrival) : super(repaint: arrival);

  final Animation<double> arrival;

  @override
  void paint(Canvas canvas, Size size) {
    final drawn = Curves.easeInOutCubic.transform(
      (arrival.value / 0.7).clamp(0.0, 1.0),
    );
    final spine = Path()
      ..moveTo(size.width * 1.08, size.height * 0.05)
      ..cubicTo(
        size.width * 0.55,
        size.height * 0.02,
        size.width * 0.02,
        size.height * 0.2,
        size.width * -0.06,
        size.height * 0.52,
      );
    final edge = Path()
      ..moveTo(size.width * 1.08, size.height * 0.13)
      ..cubicTo(
        size.width * 0.62,
        size.height * 0.12,
        size.width * 0.16,
        size.height * 0.26,
        size.width * -0.06,
        size.height * 0.52,
      );

    for (final (path, colour, width) in [
      (spine, Veil.hairline, 1.0),
      (edge, Veil.bone.withValues(alpha: 0.85), 1.4),
    ]) {
      final metric = path.computeMetrics().first;

      canvas.drawPath(
        metric.extractPath(0, metric.length * drawn),
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = width
          ..color = colour,
      );
    }
  }

  @override
  bool shouldRepaint(_BladePainter oldDelegate) => false;
}
