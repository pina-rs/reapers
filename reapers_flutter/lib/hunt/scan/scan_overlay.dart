import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../l10n/l10n.dart';
import '../../theme/veil.dart';
import '../../widgets/ledger.dart';
import '../pose/ar_pose.dart';
import 'room_reader.dart';

/// What the player sees while the room is being read: that something is
/// near, what to do about it, and how much of the room is known so far.
class ScanOverlay extends HookWidget {
  const ScanOverlay({
    super.key,
    required this.reader,
    required this.pose,
    required this.followsDevice,
    required this.repaint,
  });

  final RoomReader reader;

  /// Where the phone points, read each time the dial is drawn.
  final ArPose Function() pose;

  /// Whether the view follows the phone itself rather than a drag.
  final bool followsDevice;

  /// Ticks once a frame.
  final Listenable repaint;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = Theme.of(context).colorScheme.primary;

    return IgnorePointer(
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(28, 20, 28, 30),
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
                  switch (reader.gait) {
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
                      reader: reader,
                      heading: pose().forward.yaw,
                      light: light,
                    ),
                  ),
                ),
                const SizedBox(height: 18),
                Center(
                  child: Text(
                    switch (reader.advice) {
                      ReadingAdvice.slowDown => l10n.scanSlower,
                      ReadingAdvice.lookLower => l10n.scanLower,
                      ReadingAdvice.lookHigher => l10n.scanHigher,
                      ReadingAdvice.keepGoing => l10n.scanReading,
                    },
                    key: const Key('scan-advice'),
                    style: Veil.body(size: 16),
                  ),
                ),
                const SizedBox(height: 14),
                LedgerMeter(
                  fraction: reader.progress,
                  colour: light,
                  label: l10n.scanProgress,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// The room from above, with the player in the middle: one tick for each
/// slice of the circle, lit once it has been read. The dial turns with the
/// phone, so straight ahead is always at the top.
class _DialPainter extends CustomPainter {
  _DialPainter({
    required this.reader,
    required this.heading,
    required this.light,
  });

  final RoomReader reader;
  final double heading;
  final Color light;

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

    for (var slice = 0; slice < RoomReader.slices; slice++) {
      final bearing =
          (slice + 0.5) / RoomReader.slices * 2 * math.pi - math.pi - heading;
      // Up the screen is straight ahead, and bearings grow to the right.
      final along = Offset(math.sin(bearing), -math.cos(bearing));
      final read = reader.read(slice);

      canvas.drawLine(
        centre + along * inner,
        centre + along * (inner + (outer - inner) * (0.45 + 0.55 * read)),
        stroke
          ..strokeWidth = read >= 1 ? 3 : 1.5
          ..color = Color.lerp(Veil.hairline, light, read)!,
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
