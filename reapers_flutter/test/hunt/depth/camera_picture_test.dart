import 'dart:typed_data';

import 'package:camera/camera.dart';
import 'package:camera_platform_interface/camera_platform_interface.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/scan/depth/camera_picture.dart';

/// A grey frame [width] by [height] in the layout an Android camera
/// streams, as bright at each pixel as [brightness] says.
CameraImage _nv21(
  int width,
  int height,
  int Function(int x, int y) brightness,
) {
  final bytes = Uint8List(width * height * 3 ~/ 2)
    // No colour: both halves of every colour pair sit at the midpoint.
    ..fillRange(width * height, width * height * 3 ~/ 2, 128);

  for (var y = 0; y < height; y++) {
    for (var x = 0; x < width; x++) {
      bytes[y * width + x] = brightness(x, y);
    }
  }

  return CameraImage.fromPlatformInterface(
    CameraImageData(
      format: const CameraImageFormat(ImageFormatGroup.nv21, raw: 17),
      width: width,
      height: height,
      planes: [
        CameraImagePlane(bytes: bytes, bytesPerRow: width, bytesPerPixel: 1),
      ],
    ),
  );
}

/// The brightness, from 0 to 255, that a picture's red value stands for.
double _brightness(double red) => (red * 0.229 + 0.485) * 255;

void main() {
  test('a rear camera\'s frame is stood upright', () {
    // The sensor lies on its side: the top of the world is along the
    // frame's left edge. Here the left edge is bright and the rest dark.
    final frame = _nv21(8, 4, (x, y) => x == 0 ? 200 : 40);
    final picture = CameraPicture.from(
      frame,
      sensorOrientation: 90,
      width: 4,
      height: 8,
    )!;

    for (var column = 0; column < 4; column++) {
      expect(_brightness(picture.pixels[column]), closeTo(200, 0.5));
      expect(_brightness(picture.pixels[4 + column]), closeTo(40, 0.5));
    }
  });

  test('upright, the top row of the frame runs down the right-hand side', () {
    final frame = _nv21(8, 4, (x, y) => y == 0 ? 200 : 40);
    final picture = CameraPicture.from(
      frame,
      sensorOrientation: 90,
      width: 4,
      height: 8,
    )!;

    for (var row = 0; row < 8; row++) {
      expect(_brightness(picture.pixels[row * 4 + 3]), closeTo(200, 0.5));
      expect(_brightness(picture.pixels[row * 4]), closeTo(40, 0.5));
    }
  });

  test('a sensor mounted the other way up is turned the other way', () {
    final frame = _nv21(8, 4, (x, y) => x == 0 ? 200 : 40);
    final picture = CameraPicture.from(
      frame,
      sensorOrientation: 270,
      width: 4,
      height: 8,
    )!;

    // The frame's left edge is now the bottom of the world.
    for (var column = 0; column < 4; column++) {
      expect(_brightness(picture.pixels[7 * 4 + column]), closeTo(200, 0.5));
      expect(_brightness(picture.pixels[column]), closeTo(40, 0.5));
    }
  });

  test('all three colours are laid out one plane after another', () {
    final picture = CameraPicture.from(
      _nv21(8, 4, (x, y) => 128),
      sensorOrientation: 90,
      width: 4,
      height: 8,
    )!;

    expect(picture.pixels, hasLength(3 * 4 * 8));
    // Grey is the same brightness in each colour, scaled by that colour's
    // own statistics.
    expect(picture.pixels[0], closeTo((128 / 255 - 0.485) / 0.229, 1e-3));
    expect(picture.pixels[32], closeTo((128 / 255 - 0.456) / 0.224, 1e-3));
    expect(picture.pixels[64], closeTo((128 / 255 - 0.406) / 0.225, 1e-3));
  });

  test('a frame of the picture\'s own shape is kept whole', () {
    final picture = CameraPicture.from(
      _nv21(32, 18, (x, y) => 90),
      sensorOrientation: 90,
      width: 9,
      height: 16,
    )!;

    expect(picture.shareOfLongSide, 1);
    expect(picture.shareOfShortSide, 1);
  });

  test('a squarer frame loses its sides and says how much', () {
    // A 4:3 frame cut to a 9:16 picture keeps its height and three
    // quarters of its width.
    final picture = CameraPicture.from(
      _nv21(16, 12, (x, y) => 90),
      sensorOrientation: 90,
      width: 9,
      height: 16,
    )!;

    expect(picture.shareOfLongSide, 1);
    expect(picture.shareOfShortSide, closeTo(0.75, 1e-9));
  });

  test('a layout it cannot read is refused', () {
    final frame = CameraImage.fromPlatformInterface(
      CameraImageData(
        format: const CameraImageFormat(ImageFormatGroup.jpeg, raw: 256),
        width: 8,
        height: 4,
        planes: [CameraImagePlane(bytes: Uint8List(32), bytesPerRow: 8)],
      ),
    );

    expect(
      CameraPicture.from(frame, sensorOrientation: 90, width: 4, height: 8),
      isNull,
    );
  });
}
