/// Builds the depth model the game reads rooms with.
///
///     dart tool/build_depth_model.dart
///
/// The model is Depth Anything V2 (small, metric, indoor), fetched from its
/// authors' repository at a pinned revision and converted by
/// `tool/depth_model/export.py` into the one file the game ships:
/// `reapers_flutter/assets/depth/depth_anything_v2_metric_indoor_small.onnx`.
///
/// The conversion is Python because the model's own tooling is. This tool is
/// what runs it, through `uv`, which fetches the pinned Python packages into
/// a throwaway environment and leaves nothing installed. The first run
/// downloads about 100 MB of weights and rather more of PyTorch.
///
/// The result is committed, like the 3D models, so nobody needs to run this
/// to build the game. Run it to change the picture size or the model.
library;

import 'dart:io';

const _script = 'tool/depth_model/export.py';
const _model =
    'reapers_flutter/assets/depth/depth_anything_v2_metric_indoor_small.onnx';

/// The picture the model is converted for. These must match `DepthModel` in
/// the game, which cuts camera frames down to exactly this size.
const _width = 252;
const _height = 448;

/// A converted model weighs about 27 MB. One far from that was not
/// converted the way the game expects.
const _smallestBytes = 20 * 1024 * 1024;
const _largestBytes = 40 * 1024 * 1024;

Future<void> main() async {
  final uv = await Process.run('uv', [
    '--version',
  ]).then((result) => result.exitCode == 0, onError: (Object _) => false);

  if (!uv) {
    stderr.writeln(
      'uv is needed to run the conversion: https://docs.astral.sh/uv/',
    );
    exit(69);
  }

  stdout.writeln('converting the depth model for $_width x $_height pictures');

  final conversion = await Process.start('uv', [
    'run',
    _script,
    _model,
    '--width',
    '$_width',
    '--height',
    '$_height',
  ], mode: ProcessStartMode.inheritStdio);
  final code = await conversion.exitCode;

  if (code != 0) {
    stderr.writeln('The conversion failed.');
    exit(code);
  }

  final bytes = File(_model).lengthSync();

  if (bytes < _smallestBytes || bytes > _largestBytes) {
    stderr.writeln(
      '$_model is ${_megabytes(bytes)} MB, which is not what a converted '
      'model weighs.',
    );
    exit(1);
  }

  stdout.writeln('wrote  $_model (${_megabytes(bytes)} MB)');
}

String _megabytes(int bytes) => (bytes / (1024 * 1024)).toStringAsFixed(1);
