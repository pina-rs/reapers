import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/scan/depth/depth_model.dart';

void main() {
  group('how long the model reads', () {
    test('at the sharpness it was trained at, hardly at all', () {
      expect(DepthModel.overreach(579), closeTo(1.03, 0.01));
    });

    test(
      'through the Seeker\'s lens at the size it is shown, by two fifths',
      () {
        // 252 pixels across a view whose half-width has a tangent of 0.4122.
        expect(DepthModel.overreach(252 / (2 * 0.4122)), closeTo(1.39, 0.01));
      },
    );

    test('it matches what was measured at each size', () {
      // Focal length in pixels, and how long the model read against rooms
      // of known depth. See docs/environment.md.
      for (final (focal, measured) in [
        (438.0, 1.18),
        (344.0, 1.33),
        (282.0, 1.45),
      ]) {
        expect(DepthModel.overreach(focal), closeTo(measured, 0.03));
      }
    });

    test('a lens far outside what was measured is not guessed at', () {
      expect(DepthModel.overreach(50), DepthModel.overreach(200));
      expect(DepthModel.overreach(5000), DepthModel.overreach(700));
    });
  });
}
