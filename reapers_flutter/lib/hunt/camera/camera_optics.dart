import 'dart:math' as math;

import 'package:camera/camera.dart';
import 'package:flutter/services.dart';

/// How much of the world a picture takes in, as the tangent of half its view
/// along each side. A point at depth 1 on the long side's edge is
/// [tanHalfLong] off the middle.
typedef LensView = ({double tanHalfLong, double tanHalfShort});

/// The camera's lens and sensor, which together decide how wide it sees.
///
/// The hunt needs this twice over: to draw a husk through the same lens the
/// world behind it was photographed with, so that it holds its place when
/// the phone turns, and to turn a picture of the room into distances.
final class CameraOptics {
  const CameraOptics({
    required this.focalLength,
    required this.sensorLong,
    required this.sensorShort,
  });

  /// A phone's main camera, near enough, for a phone that will not describe
  /// its own: about 62 degrees along the long side of a 4:3 sensor.
  static const typical = CameraOptics(
    focalLength: 5,
    sensorLong: 6,
    sensorShort: 4.5,
  );

  /// Asks the phone about [camera]'s lens, or answers null where the phone
  /// has no way to say. Only Android answers so far.
  static Future<CameraOptics?> describe(CameraDescription camera) async {
    try {
      final traits = await _channel.invokeMapMethod<String, double>(
        'describe',
        {'camera': camera.name},
      );

      if (traits == null) {
        return null;
      }

      final width = traits['sensorWidth']!;
      final height = traits['sensorHeight']!;

      return CameraOptics(
        focalLength: traits['focalLength']!,
        sensorLong: math.max(width, height),
        sensorShort: math.min(width, height),
      );
    } on MissingPluginException {
      return null;
    }
  }

  static const _channel = MethodChannel('reapers/lens');

  /// All three in millimetres.
  final double focalLength;
  final double sensorLong;
  final double sensorShort;

  /// What a picture [longSide] by [shortSide] pixels takes in.
  ///
  /// A picture shaped differently from the sensor is cut from the middle of
  /// it, keeping all of whichever side it can.
  LensView view({required double longSide, required double shortSide}) {
    final shape = longSide / shortSide;

    if (shape >= sensorLong / sensorShort) {
      final tanHalfLong = sensorLong / (2 * focalLength);

      return (tanHalfLong: tanHalfLong, tanHalfShort: tanHalfLong / shape);
    }

    final tanHalfShort = sensorShort / (2 * focalLength);

    return (tanHalfLong: tanHalfShort * shape, tanHalfShort: tanHalfShort);
  }
}

/// The full vertical field of view, in radians, of an upright picture seen
/// through [lens] once it has been scaled to cover a view [viewWidth] by
/// [viewHeight], as the camera feed is.
double coveringFieldOfView(
  LensView lens, {
  required double viewWidth,
  required double viewHeight,
}) {
  final viewShape = viewWidth / viewHeight;
  final pictureShape = lens.tanHalfShort / lens.tanHalfLong;

  // A view narrower than the picture shows its whole height and loses the
  // sides; a wider one shows its whole width and loses top and bottom.
  final tanHalfVertical = viewShape <= pictureShape
      ? lens.tanHalfLong
      : lens.tanHalfShort / viewShape;

  return 2 * math.atan(tanHalfVertical);
}
