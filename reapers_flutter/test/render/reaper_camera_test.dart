import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/render/reaper_camera.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:vector_math/vector_math.dart';

void main() {
  for (final native in [false, true]) {
    test('face camera looks forward and stays upright with native=$native', () {
      final transform = Matrix4.rotationY(math.pi / 2);
      final orbit = reaperFaceOrbit(
        0,
        calling: ReaperClass.harvester,
        native: native,
      );
      final camera = transform.transformed3(orbit.offset);
      final up = transform.rotate3(orbit.up);
      expect(camera.x, closeTo(native ? -1.15 : 1.15, 1e-6));
      expect(camera.y, closeTo(-.6, 1e-6));
      expect(camera.z, closeTo(0, 1e-6));
      expect(up.y, closeTo(1, 1e-6));
      expect(up.z, closeTo(0, 1e-6));
      final side = reaperFaceOrbit(
        math.pi / 2,
        calling: ReaperClass.ferryman,
        native: native,
      );
      expect(side.offset.x, closeTo(1.25, 1e-6));
      expect(side.offset.y, closeTo(-.04, 1e-6));
    });
  }
}
