import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../l10n/l10n.dart';
import '../../theme/veil.dart';
import '../../widgets/ledger.dart';
import '../pose/ar_pose.dart';
import 'room_survey.dart';

/// What the player sees while the room is being read: that something is
/// near, what to do about it, and how much of the room is known so far.
class ScanOverlay extends HookWidget {
  const ScanOverlay({
    super.key,
    required this.survey,
    required this.gait,
    required this.pose,
    required this.followsDevice,
    required this.repaint,
    this.floorDepth,
    this.onSkip,
  });

  final RoomSurvey survey;

  /// How the husk that will appear gets about, which decides where the
  /// player is told to look.
  final Gait gait;

  /// Where the phone points, read each time the dial is drawn.
  final ArPose Function() pose;

  /// Whether the view follows the phone itself rather than a drag.
  final bool followsDevice;

  /// Ticks once a frame.
  final Listenable repaint;

  /// How far below the phone the floor has been found to be, in metres,
  /// read each frame. Null for a survey that cannot see the floor.
  final double? Function()? floorDepth;

  /// Starts the hunt without waiting for the room to be read. Offered only
  /// when it is given.
  final VoidCallback? onSkip;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;

    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(28, 20, 28, 30),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Only the way out takes a touch. Everything else lets it
            // through to whatever is underneath.
            Expanded(
              child: IgnorePointer(
                child: ListenableBuilder(
                  listenable: repaint,
                  builder: (context, _) => Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(l10n.scanLabel.toUpperCase(), style: Veil.label()),
                      const SizedBox(height: 8),
                      Text(l10n.scanTitle, style: Veil.display(40)),
                      const SizedBox(height: 12),
                      Text(
                        switch (gait) {
                          Gait.ground when followsDevice => l10n.scanFloor,
                          Gait.ground => l10n.scanFloorDrag,
                          _ when followsDevice => l10n.scanAir,
                          _ => l10n.scanAirDrag,
                        },
                        style: Veil.body(color: Veil.bone, size: 16),
                      ),
                      const Spacer(),
                      Center(
                        child: CustomPaint(
                          size: const Size.square(168),
                          painter: _DialPainter(
                            survey: survey,
                            heading: pose().forward.yaw,
                            light: light,
                          ),
                        ),
                      ),
                      const SizedBox(height: 18),
                      Center(
                        child: Text(
                          switch (survey.advice) {
                            ReadingAdvice.slowDown => l10n.scanSlower,
                            ReadingAdvice.lookLower => l10n.scanLower,
                            ReadingAdvice.lookHigher => l10n.scanHigher,
                            ReadingAdvice.keepGoing => l10n.scanReading,
                          },
                          key: const Key('scan-advice'),
                          textAlign: TextAlign.center,
                          style: Veil.body(size: 16),
                        ),
                      ),
                      if (floorDepth?.call() case final metres?) ...[
                        const SizedBox(height: 6),
                        Center(
                          child: Text(
                            l10n.scanFloorFound(metres.toStringAsFixed(2)),
                            key: const Key('scan-floor'),
                            style: Veil.figure(color: Veil.ash, size: 12),
                          ),
                        ),
                      ],
                      const SizedBox(height: 14),
                      LedgerMeter(
                        fraction: survey.progress,
                        colour: light,
                        label: l10n.scanProgress,
                      ),
                    ],
                  ),
                ),
              ),
            ),
            if (onSkip != null) ...[
              const SizedBox(height: 14),
              LedgerButton(
                key: const Key('scan-skip'),
                label: l10n.scanSkip,
                onPressed: onSkip,
                quiet: true,
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// The room from above, with the player in the middle: one spoke for each
/// slice of the circle, lit once it has been read. Where the survey can see,
/// a spoke is as long as the way is clear, so the dial takes the shape of
/// the room, and a slice too tight for the husk to keep its distance is
/// marked in red. The
/// dial turns with the phone, so straight ahead is always at the top.
class _DialPainter extends CustomPainter {
  _DialPainter({
    required this.survey,
    required this.heading,
    required this.light,
  });

  final RoomSurvey survey;
  final double heading;
  final Color light;

  /// The reach, in metres, that fills a spoke.
  static const _fullReach = 5.0;

  @override
  void paint(Canvas canvas, Size size) {
    final centre = size.center(Offset.zero);
    final outer = size.shortestSide / 2;
    final inner = outer * 0.72;
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.butt;

    canvas.drawCircle(
      centre,
      inner - 8,
      stroke
        ..strokeWidth = 1
        ..color = Veil.hairline,
    );

    for (var slice = 0; slice < HuntPlan.slices; slice++) {
      final bearing = HuntPlan.bearingOf(slice) - heading;
      // Up the screen is straight ahead, and bearings grow to the right.
      final along = Offset(math.sin(bearing), -math.cos(bearing));
      final read = survey.read(slice);
      final length = switch (survey.reach(slice)) {
        final metres? => (metres / _fullReach).clamp(0.15, 1.0),
        null => 0.45 + 0.55 * read,
      };

      canvas.drawLine(
        centre + along * inner,
        centre + along * (inner + (outer - inner) * length),
        stroke
          ..strokeWidth = read >= 1 ? 3 : 1.5
          ..color = survey.tight(slice)
              ? Veil.wound.withValues(alpha: 0.7)
              : Color.lerp(Veil.hairline, light, read)!,
      );
    }

    // Where the phone is looking now.
    final view = Path()
      ..moveTo(centre.dx, centre.dy - inner + 14)
      ..lineTo(centre.dx - 6, centre.dy - inner + 26)
      ..lineTo(centre.dx + 6, centre.dy - inner + 26)
      ..close();

    canvas.drawPath(view, Paint()..color = Veil.bone);
  }

  @override
  bool shouldRepaint(_DialPainter oldDelegate) => true;
}
