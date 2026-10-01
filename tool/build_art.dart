/// Builds every 3D model the game ships from its Blender source.
///
///     dart tool/build_art.dart              build everything
///     dart tool/build_art.dart reapers      build one script
///     dart tool/build_art.dart --preview    also render review sheets
///
/// The sources are the scripts in `art/blender`. Blender's authoring API is
/// Python, so those scripts are Python; this tool is what runs them, and it is
/// the only way models reach `reapers_flutter/assets/models`. Nothing in that
/// folder is edited by hand.
///
/// Review sheets land in `art/build/previews`, which is not committed.
library;

import 'dart:convert';
import 'dart:io';

import 'package:reapers_shared/reapers_shared.dart';

/// The Blender release the committed models were built with. A different
/// release may triangulate or export differently, so upgrade it on purpose.
const blenderRelease = '5.1';

const _scripts = ['reapers', 'weapons', 'husks'];
const _models = 'reapers_flutter/assets/models';
const _previews = 'art/build/previews';

Future<void> main(List<String> arguments) async {
  final preview = arguments.contains('--preview');
  final wanted = arguments.where((argument) => !argument.startsWith('--'));
  final scripts = wanted.isEmpty ? _scripts : wanted.toList();
  final unknown = scripts.where((script) => !_scripts.contains(script));

  if (unknown.isNotEmpty) {
    stderr.writeln('Unknown art script: ${unknown.join(', ')}');
    stderr.writeln('Known scripts: ${_scripts.join(', ')}');
    exit(64);
  }

  await _requireBlender();

  final specs = File('art/build/vessels.json');

  specs
    ..createSync(recursive: true)
    ..writeAsStringSync(jsonEncode(_vesselSpecs()));

  for (final script in scripts) {
    final source = File('art/blender/$script.py');

    if (!source.existsSync()) {
      stdout.writeln('skip   $script (no art/blender/$script.py yet)');
      continue;
    }

    stdout.writeln('build  $script');

    final exitCode = await _runBlender([
      '--background',
      '--python-exit-code',
      '1',
      '--python',
      source.path,
      '--',
      '--out',
      _models,
      '--specs',
      specs.path,
      if (preview) ...['--preview', _previews],
    ]);

    if (exitCode != 0) {
      stderr.writeln('Blender failed on $script (exit $exitCode).');
      exit(exitCode);
    }
  }
}

/// The body sizes and seal positions the rules use, for the art build.
///
/// The models are shaped to these numbers rather than the other way round, so
/// a seal is always drawn exactly where the rules say a strike lands.
Map<String, Object?> _vesselSpecs() => {
  for (final vessel in Vessel.values)
    vessel.name: {
      'halfWidth': vessel.halfWidth,
      'halfHeight': vessel.halfHeight,
      'seals': [
        for (final seal in vessel.seals)
          {
            'offset': [seal.offset.x, seal.offset.y, seal.offset.z],
            'orbitRadius': seal.orbitRadius,
            'orbitPhase': seal.orbitPhase,
          },
      ],
    },
};

Future<void> _requireBlender() async {
  final ProcessResult result;

  try {
    result = await Process.run('blender', ['--version']);
  } on ProcessException {
    stderr.writeln(
      'Blender is not on the PATH. Install Blender $blenderRelease.',
    );
    exit(69);
  }

  final version = '${result.stdout}'.split('\n').first.trim();

  if (!version.contains(' $blenderRelease.')) {
    stderr.writeln(
      'Found "$version" but the models are built with Blender '
      '$blenderRelease. Use that release, or update blenderRelease in '
      'tool/build_art.dart and rebuild every model.',
    );
    exit(69);
  }
}

/// Runs Blender, showing only the lines worth reading.
Future<int> _runBlender(List<String> arguments) async {
  final process = await Process.start('blender', arguments);
  final shown = RegExp(r'BUILT|Error|Traceback|File "|^\s+\S.*Error');

  Future<void> relay(Stream<List<int>> stream, IOSink sink) => stream
      .transform(utf8.decoder)
      .transform(const LineSplitter())
      .where(shown.hasMatch)
      .forEach(sink.writeln);

  await Future.wait([
    relay(process.stdout, stdout),
    relay(process.stderr, stderr),
  ]);

  return process.exitCode;
}
