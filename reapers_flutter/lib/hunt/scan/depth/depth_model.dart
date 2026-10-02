import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/services.dart';
import 'package:flutter_onnxruntime/flutter_onnxruntime.dart';

/// The network that looks at one picture and says how far away everything
/// in it is.
///
/// It is Depth Anything V2 (small), fine-tuned by its authors to answer in
/// metres indoors, and it runs on the phone: no picture leaves it. See
/// docs/environment.md for where the model comes from and how it is built.
final class DepthModel {
  DepthModel._(this._session);

  /// Loads the model, or answers null on a platform that has nothing to run
  /// it with. The room is read without it then: see `GazeSurvey`.
  static Future<DepthModel?> load() async {
    try {
      final session = await OnnxRuntime().createSessionFromAsset(
        asset,
        options: OrtSessionOptions(
          intraOpNumThreads: threads,
          providers: [OrtProvider.CPU],
        ),
      );

      return DepthModel._(session);
    } on MissingPluginException {
      return null;
    }
  }

  static const asset =
      'assets/depth/depth_anything_v2_metric_indoor_small.onnx';

  /// The picture the model takes, in pixels. It was converted for exactly
  /// this size, which is the shape of a phone's camera frame held upright.
  static const width = 252;
  static const height = 448;

  /// Cores the model may use. The rest are left to draw the game.
  static const threads = 2;

  /// How much too long the model reads every distance in a picture taken
  /// through a lens whose focal length is [focalPixels] of the picture's
  /// own pixels.
  ///
  /// The model learnt its metres from pictures about twice as sharp as the
  /// ones it is shown here, and the smaller a room is drawn the farther away
  /// it takes it to be. Measured against rooms of known depth, what it says
  /// is long by a steady factor that follows this curve; see
  /// docs/environment.md for the measurements, and for why a wrong factor
  /// still leaves a husk's feet on the floor.
  static double overreach(double focalPixels) =>
      1.026 * math.pow(579 / focalPixels.clamp(200, 700), 0.48);

  final OrtSession _session;

  /// How far away the model takes each pixel of [pixels] to be, in metres
  /// along the direction the camera faces, row by row from the top. Divide
  /// by [overreach] for the distance itself.
  ///
  /// [pixels] is a picture [width] by [height], laid out as the model
  /// expects: see `CameraPicture`.
  Future<Float32List> estimate(Float32List pixels) async {
    final input = await OrtValue.fromList(pixels, [1, 3, height, width]);

    try {
      final outputs = await _session.run({'pixel_values': input});
      final output = outputs['predicted_depth']!;

      try {
        final values = await output.asFlattenedList();
        final depths = Float32List(values.length);

        for (var index = 0; index < values.length; index++) {
          depths[index] = (values[index] as num).toDouble();
        }

        return depths;
      } finally {
        await output.dispose();
      }
    } finally {
      await input.dispose();
    }
  }

  Future<void> close() => _session.close();
}
