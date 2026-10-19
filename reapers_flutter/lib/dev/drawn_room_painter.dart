import 'package:flutter/material.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../hunt/pose/ar_pose.dart';
import '../theme/veil.dart';
import 'drawn_room.dart';

/// Draws a [DrawnRoom] behind the hunt, where a camera's picture would be:
/// a faint grid for the floor and solid blocks for what stands on it, so a
/// husk can be seen to stand on it, leap onto it and hide behind it.
class DrawnRoomPainter extends CustomPainter {
  DrawnRoomPainter({
    required this.room,
    required this.pose,
    required this.verticalFov,
    required Listenable repaint,
  }) : super(repaint: repaint);

  final DrawnRoom room;
  final ArPose Function() pose;
  final double verticalFov;

  @override
  void paint(Canvas canvas, Size size) {
    final current = pose();
    final aspect = size.width / size.height;

    Offset? place(Vec3 point) {
      final projected = current.project(
        point.normalized,
        verticalFov: verticalFov,
        aspect: aspect,
      );

      if (projected == null || projected.x.abs() > 4 || projected.y.abs() > 4) {
        return null;
      }

      return Offset(
        (projected.x + 1) / 2 * size.width,
        (1 - projected.y) / 2 * size.height,
      );
    }

    final floor = -room.eyeHeight;
    final grid = Paint()
      ..color = Veil.bone.withValues(alpha: 0.12)
      ..strokeWidth = 1;

    // The floor, in half-metre squares out to the walls, each line drawn in
    // short pieces so that the part behind the player is left out.
    for (var line = -room.wall; line <= room.wall + 1e-9; line += 0.5) {
      _line(
        canvas,
        place,
        Vec3(line, floor, -room.wall),
        Vec3(line, floor, room.wall),
        grid,
      );
      _line(
        canvas,
        place,
        Vec3(-room.wall, floor, line),
        Vec3(room.wall, floor, line),
        grid,
      );
    }

    final sides = [...room.blocks]
      ..sort((one, other) => other.near.compareTo(one.near));

    for (final block in sides) {
      _paintBlock(canvas, place, block, floor);
    }
  }

  void _paintBlock(
    Canvas canvas,
    Offset? Function(Vec3) place,
    Block block,
    double floor,
  ) {
    final top = floor + block.height;
    final faces = <List<Vec3>>[
      // Only the faces that look toward the player.
      if (block.near > 0)
        [
          Vec3(block.left, floor, block.near),
          Vec3(block.right, floor, block.near),
          Vec3(block.right, top, block.near),
          Vec3(block.left, top, block.near),
        ],
      if (block.left > 0)
        [
          Vec3(block.left, floor, block.near),
          Vec3(block.left, floor, block.far),
          Vec3(block.left, top, block.far),
          Vec3(block.left, top, block.near),
        ],
      if (block.right < 0)
        [
          Vec3(block.right, floor, block.near),
          Vec3(block.right, floor, block.far),
          Vec3(block.right, top, block.far),
          Vec3(block.right, top, block.near),
        ],
      if (top < 0)
        [
          Vec3(block.left, top, block.near),
          Vec3(block.right, top, block.near),
          Vec3(block.right, top, block.far),
          Vec3(block.left, top, block.far),
        ],
    ];

    for (final (index, face) in faces.indexed) {
      final isTop = top < 0 && index == faces.length - 1;
      final fill = Paint()..color = Color(isTop ? 0xFF2A2E36 : 0xFF1A1D23);

      // Drawn as small tiles, so the part in view is drawn even when a far
      // corner of the face is behind the player or well off the screen.
      const tiles = 12;
      final across = (face[1] - face[0]) * (1 / tiles);
      final up = (face[3] - face[0]) * (1 / tiles);

      for (var row = 0; row < tiles; row++) {
        for (var column = 0; column < tiles; column++) {
          final corner =
              face[0] + across * column.toDouble() + up * row.toDouble();
          final corners = [
            place(corner),
            place(corner + across),
            place(corner + across + up),
            place(corner + up),
          ];

          if (corners.contains(null)) {
            continue;
          }

          canvas.drawPath(
            Path()..addPolygon(corners.cast<Offset>(), true),
            fill..isAntiAlias = false,
          );
        }
      }

      final edge = Paint()
        ..color = Veil.bone.withValues(alpha: 0.35)
        ..strokeWidth = 1.2;

      for (var side = 0; side < 4; side++) {
        _line(canvas, place, face[side], face[(side + 1) % 4], edge);
      }
    }
  }

  static void _line(
    Canvas canvas,
    Offset? Function(Vec3) place,
    Vec3 from,
    Vec3 to,
    Paint paint,
  ) {
    const pieces = 24;
    var last = place(from);

    for (var piece = 1; piece <= pieces; piece++) {
      final next = place(from + (to - from) * (piece / pieces));

      if (last != null && next != null) {
        canvas.drawLine(last, next, paint);
      }

      last = next;
    }
  }

  @override
  bool shouldRepaint(covariant DrawnRoomPainter oldDelegate) =>
      oldDelegate.room != room || oldDelegate.verticalFov != verticalFov;
}
