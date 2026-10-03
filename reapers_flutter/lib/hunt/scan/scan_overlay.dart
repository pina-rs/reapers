import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../l10n/l10n.dart';
import '../../theme/veil.dart';
import '../../widgets/ledger.dart';
import '../pose/ar_pose.dart';
import 'room_survey.dart';
import 'sigil_painter.dart';

/// What the player reads while tracing a husk's sigil: that something is
/// near, what to do about it, the whole sigil in small, and, on a phone
/// that sees, what it has made of the room so far.
///
/// The sigil itself is drawn in the room by [SigilPainter]. This lays the
/// words over it.
class ScanOverlay extends HookWidget {
  const ScanOverlay({
    super.key,
    required this.trace,
    required this.pose,
    required this.followsDevice,
    required this.light,
    required this.repaint,
    this.survey,
    this.floorDepth,
    this.onSkip,
  });

  final SigilTrace trace;

  /// Where the phone points, read each time the overlay is drawn.
  final ArPose Function() pose;

  /// Whether the view follows the phone itself rather than a drag.
  final bool followsDevice;

  /// The husk's own colour, which its sigil burns in.
  final Color light;

  /// Ticks once a frame.
  final Listenable repaint;

  /// The reading of the room going on as the sigil is traced, or null on a
  /// phone that cannot see the room.
  final RoomSurvey? survey;

  /// How far below the phone the floor has been found to be, in metres,
  /// read each frame. Null for a survey that cannot see the floor.
  final double? Function()? floorDepth;

  /// Starts the hunt without tracing the rest of the sigil. Offered only
  /// when it is given.
  final VoidCallback? onSkip;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final survey = this.survey;

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
                        followsDevice ? l10n.sigilHow : l10n.sigilHowDrag,
                        style: Veil.body(color: Veil.bone, size: 16),
                      ),
                      const SizedBox(height: 16),
                      CustomPaint(
                        key: const Key('sigil-glyph'),
                        size: const Size(200, 76),
                        painter: SigilGlyphPainter(
                          trace: trace,
                          pose: pose,
                          light: light,
                          repaint: repaint,
                        ),
                      ),
                      const Spacer(),
                      Row(
                        crossAxisAlignment: CrossAxisAlignment.end,
                        children: [
                          if (survey != null) ...[
                            CustomPaint(
                              size: const Size.square(92),
                              painter: _DialPainter(
                                survey: survey,
                                heading: pose().forward.yaw,
                                arc: (
                                  centre: trace.centre,
                                  halfWidth: trace.sigil.halfArc,
                                ),
                                light: light,
                              ),
                            ),
                            const SizedBox(width: 18),
                          ],
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  _status(l10n),
                                  key: const Key('sigil-status'),
                                  style: Veil.body(size: 16),
                                ),
                                if (floorDepth?.call() case final metres?) ...[
                                  const SizedBox(height: 6),
                                  Text(
                                    l10n.scanFloorFound(
                                      metres.toStringAsFixed(2),
                                    ),
                                    key: const Key('scan-floor'),
                                    style: Veil.figure(
                                      color: Veil.ash,
                                      size: 12,
                                    ),
                                  ),
                                ],
                              ],
                            ),
                          ),
                        ],
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

  /// One line on how the trace is going: a light about to go out, a light
  /// that will not fill because the phone is moving, or how many have
  /// caught.
  String _status(AppLocalizations l10n) {
    final view = pose();

    if (trace.fraying > 0.5) {
      return l10n.sigilSlipping;
    }

    if (trace.restsOn(view.forward) &&
        view.turnSpeed > HuntRules.sigilSteadySpeed) {
      return l10n.sigilSteady;
    }

    return l10n.sigilCaught(trace.caught, trace.sigil.lights.length);
  }
}

/// The room from above, with the player in the middle: one spoke for each
/// slice of the circle, lit once it has been read. A spoke is as long as
/// the way is clear, so the dial takes the shape of the room, and a slice
/// too tight for the husk to keep its distance is marked in red. A thin arc
/// round the outside is where the husk will keep to. The dial turns with
/// the phone, so straight ahead is always at the top.
class _DialPainter extends CustomPainter {
  _DialPainter({
    required this.survey,
    required this.heading,
    required this.arc,
    required this.light,
  });

  final RoomSurvey survey;
  final double heading;

  /// The arc the sigil reads, which the husk will keep to.
  final ({double centre, double halfWidth}) arc;

  final Color light;

  /// The reach, in metres, that fills a spoke.
  static const _fullReach = 5.0;

  @override
  void paint(Canvas canvas, Size size) {
    final centre = size.center(Offset.zero);
    final outer = size.shortestSide / 2 - 4;
    final inner = outer * 0.62;
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.butt;

    canvas.drawCircle(
      centre,
      inner - 5,
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
        null => 0.3,
      };

      canvas.drawLine(
        centre + along * inner,
        centre + along * (inner + (outer - inner) * length),
        stroke
          ..strokeWidth = read >= 1 ? 2 : 1
          ..color = survey.tight(slice)
              ? Veil.wound.withValues(alpha: 0.7)
              : Color.lerp(Veil.hairline, light, read)!,
      );
    }

    // Angles on a canvas start at three o'clock and grow clockwise, and a
    // bearing starts at twelve and grows to the right.
    canvas.drawArc(
      Rect.fromCircle(center: centre, radius: outer + 3),
      arc.centre - arc.halfWidth - heading - math.pi / 2,
      arc.halfWidth * 2,
      false,
      stroke
        ..strokeWidth = 1
        ..color = light.withValues(alpha: 0.6),
    );

    // Where the phone is looking now.
    final view = Path()
      ..moveTo(centre.dx, centre.dy - inner + 10)
      ..lineTo(centre.dx - 5, centre.dy - inner + 19)
      ..lineTo(centre.dx + 5, centre.dy - inner + 19)
      ..close();

    canvas.drawPath(view, Paint()..color = Veil.bone);
  }

  @override
  bool shouldRepaint(_DialPainter oldDelegate) => true;
}
