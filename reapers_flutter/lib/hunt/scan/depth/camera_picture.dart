import 'dart:typed_data';

import 'package:camera/camera.dart';

/// A camera frame cut down to what the depth model looks at: an upright
/// picture of a fixed size, with its colours scaled the way the model was
/// trained to expect.
final class CameraPicture {
  const CameraPicture({
    required this.pixels,
    required this.shareOfLongSide,
    required this.shareOfShortSide,
  });

  /// Red, then green, then blue, each a whole plane of
  /// `width * height` values, row by row from the top.
  final Float32List pixels;

  /// How much of the frame's long side, and of its short side, the picture
  /// kept: 1 for all of it, less when the frame had to be cropped to the
  /// picture's shape.
  final double shareOfLongSide;
  final double shareOfShortSide;

  /// Cuts an upright picture [width] by [height] from the middle of
  /// [frame].
  ///
  /// [sensorOrientation] is how far, in degrees clockwise, the frame has to
  /// be turned to stand upright on a phone held in portrait: 90 for nearly
  /// every rear camera. Answers null for a frame in a layout this does not
  /// read.
  static CameraPicture? from(
    CameraImage frame, {
    required int sensorOrientation,
    required int width,
    required int height,
  }) {
    final read = switch (frame.format.group) {
      ImageFormatGroup.nv21 => _readNv21,
      ImageFormatGroup.bgra8888 => _readBgra,
      _ => null,
    };

    if (read == null || (sensorOrientation != 90 && sensorOrientation != 270)) {
      return null;
    }

    // Upright, the frame's height runs across and its width runs down.
    final uprightWidth = frame.height;
    final uprightHeight = frame.width;
    final shape = width / height;
    final cropWidth = uprightWidth / uprightHeight > shape
        ? uprightHeight * shape
        : uprightWidth.toDouble();
    final cropHeight = cropWidth / shape;
    final left = (uprightWidth - cropWidth) / 2;
    final top = (uprightHeight - cropHeight) / 2;
    final step = cropWidth / width;

    final area = width * height;
    final pixels = Float32List(area * 3);
    final rgb = Float64List(3);

    for (var row = 0; row < height; row++) {
      for (var column = 0; column < width; column++) {
        var red = 0.0;
        var green = 0.0;
        var blue = 0.0;

        // Each pixel of the picture stands for a block of the frame, so it
        // takes the mean of a few points across that block. Taking one
        // point alone would turn fine detail into noise.
        for (var tap = 0; tap < _taps * _taps; tap++) {
          final uprightX = left + (column + (tap % _taps + 0.5) / _taps) * step;
          final uprightY = top + (row + (tap ~/ _taps + 0.5) / _taps) * step;
          final x = uprightX.floor().clamp(0, uprightWidth - 1);
          final y = uprightY.floor().clamp(0, uprightHeight - 1);

          if (sensorOrientation == 90) {
            read(frame, y, frame.height - 1 - x, rgb);
          } else {
            read(frame, frame.width - 1 - y, x, rgb);
          }

          red += rgb[0];
          green += rgb[1];
          blue += rgb[2];
        }

        final index = row * width + column;

        pixels[index] = (red * _scale - _mean[0]) / _spread[0];
        pixels[area + index] = (green * _scale - _mean[1]) / _spread[1];
        pixels[2 * area + index] = (blue * _scale - _mean[2]) / _spread[2];
      }
    }

    return CameraPicture(
      pixels: pixels,
      shareOfLongSide: cropHeight / uprightHeight,
      shareOfShortSide: cropWidth / uprightWidth,
    );
  }

  /// Points sampled along each side of a block.
  static const _taps = 2;

  /// Brings the sum of a block's points down to a mean from 0 to 1.
  static const _scale = 1 / (255 * _taps * _taps);

  /// The colour statistics the model was trained with.
  static const _mean = [0.485, 0.456, 0.406];
  static const _spread = [0.229, 0.224, 0.225];

  /// Reads the pixel at ([x], [y]) of a frame whose brightness is one plane
  /// and whose colour is a second, half the size, packed after it.
  static void _readNv21(CameraImage frame, int x, int y, Float64List rgb) {
    final plane = frame.planes.first;
    final bytes = plane.bytes;
    final stride = plane.bytesPerRow;
    final colour = stride * frame.height + (y >> 1) * stride + (x & ~1);
    final luma = bytes[y * stride + x].toDouble();
    final v = bytes[colour] - 128;
    final u = bytes[colour + 1] - 128;

    rgb[0] = (luma + 1.402 * v).clamp(0, 255);
    rgb[1] = (luma - 0.344136 * u - 0.714136 * v).clamp(0, 255);
    rgb[2] = (luma + 1.772 * u).clamp(0, 255);
  }

  static void _readBgra(CameraImage frame, int x, int y, Float64List rgb) {
    final plane = frame.planes.first;
    final at = y * plane.bytesPerRow + x * 4;

    rgb[0] = plane.bytes[at + 2].toDouble();
    rgb[1] = plane.bytes[at + 1].toDouble();
    rgb[2] = plane.bytes[at].toDouble();
  }
}
