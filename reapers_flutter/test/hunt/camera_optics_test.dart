import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/camera/camera_optics.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// The Seeker's main camera, as the phone itself describes it.
const _seeker = CameraOptics(
  focalLength: 5.24,
  sensorLong: 7.68,
  sensorShort: 5.76,
);

void main() {
  test('a picture the shape of the sensor takes in all of it', () {
    final lens = _seeker.view(longSide: 4000, shortSide: 3000);

    expect(lens.tanHalfLong, closeTo(7.68 / 2 / 5.24, 1e-9));
    expect(lens.tanHalfShort, closeTo(5.76 / 2 / 5.24, 1e-9));
  });

  test('a longer picture keeps the long side and loses some of the short', () {
    final lens = _seeker.view(longSide: 1280, shortSide: 720);

    expect(lens.tanHalfLong, closeTo(7.68 / 2 / 5.24, 1e-9));
    expect(lens.tanHalfShort, closeTo(7.68 / 2 / 5.24 * 720 / 1280, 1e-9));
  });

  test('a squarer picture keeps the short side and loses some of the long', () {
    final lens = _seeker.view(longSide: 1000, shortSide: 1000);

    expect(lens.tanHalfShort, closeTo(5.76 / 2 / 5.24, 1e-9));
    expect(lens.tanHalfLong, closeTo(5.76 / 2 / 5.24, 1e-9));
  });

  group('the feed scaled to cover the screen', () {
    final lens = _seeker.view(longSide: 1280, shortSide: 720);

    test('on a screen taller than the picture, shows its whole height', () {
      final fieldOfView = coveringFieldOfView(
        lens,
        viewWidth: 1200,
        viewHeight: 2670,
      );

      expect(degrees(fieldOfView), closeTo(72.5, 0.1));
    });

    test('on a screen squatter than the picture, shows its whole width', () {
      final fieldOfView = coveringFieldOfView(
        lens,
        viewWidth: 1000,
        viewHeight: 1000,
      );

      // The width is the picture's short side, and the view is as tall as
      // it is wide.
      expect(degrees(fieldOfView), closeTo(44.8, 0.1));
    });
  });

  test('a phone that will not say is taken to see about 62 degrees', () {
    final lens = CameraOptics.typical.view(longSide: 1280, shortSide: 720);
    final fieldOfView = coveringFieldOfView(
      lens,
      viewWidth: 402,
      viewHeight: 874,
    );

    expect(degrees(fieldOfView), closeTo(62, 0.2));
  });
}
