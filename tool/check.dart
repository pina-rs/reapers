// Runs every check a change has to pass before it reaches main: formatting,
// analysis and the tests of each package. Run it from the repository root:
//
//     dart tool/check.dart            # everything
//     dart tool/check.dart --fast     # skip the server tests, which need
//                                     # a database to start
//
// It stops at the first step that fails and exits with that step's code.
// `dart tool/merge.dart` runs the same checks, in full, before it lets a
// branch into main.
// ignore_for_file: avoid_print
import 'dart:io';

/// One command to run, and where.
typedef Step = ({String name, String directory, List<String> command});

/// How a run of the checks went.
typedef CheckReport = ({bool passed, List<String> steps, int exitCode});

const _packages = ['reapers_shared', 'reapers_flutter', 'reapers_server'];

/// Code Serverpod and the build tools write. It is formatted and analysed by
/// whatever generated it, not here.
const _generated = [
  'reapers_client',
  'reapers_server/lib/src/generated',
  'reapers_server/test/integration/test_tools',
  'reapers_flutter/lib/l10n/generated',
];

List<Step> _steps({required bool fast}) => [
  (
    name: 'format',
    directory: '.',
    command: [
      'dart',
      'format',
      '--output=none',
      '--set-exit-if-changed',
      ..._handWritten(),
    ],
  ),
  (name: 'analyze tool', directory: '.', command: ['dart', 'analyze', 'tool']),
  for (final package in _packages)
    (
      name: 'analyze $package',
      directory: package,
      command: ['dart', 'analyze', '--fatal-infos'],
    ),
  (
    name: 'test reapers_shared',
    directory: 'reapers_shared',
    command: ['dart', 'test'],
  ),
  (
    name: 'test reapers_flutter',
    directory: 'reapers_flutter',
    command: ['flutter', 'test'],
  ),
  if (!fast)
    (
      name: 'test reapers_server',
      directory: 'reapers_server',
      command: ['dart', 'test'],
    ),
];

/// Every Dart file a person wrote, which is what the formatter is held to.
List<String> _handWritten() {
  final roots = ['tool', ..._packages];
  final files = <String>[];

  for (final root in roots) {
    for (final entity in Directory(root).listSync(recursive: true)) {
      final path = entity.path;
      final skipped =
          entity is! File ||
          !path.endsWith('.dart') ||
          path.endsWith('.g.dart') ||
          path.contains('/.dart_tool/') ||
          path.contains('/build/') ||
          _generated.any(path.startsWith);

      if (!skipped) {
        files.add(path);
      }
    }
  }

  return files..sort();
}

/// Runs the checks from the repository root and reports how they went.
///
/// A full run needs `reapers_server/config/passwords.yaml`. When it is
/// missing the run fails before it starts, with the fix.
Future<CheckReport> runChecks({required bool fast}) async {
  if (!File('pubspec.yaml').existsSync() ||
      !Directory('reapers_shared').existsSync()) {
    stderr.writeln('Run this from the repository root.');

    return (passed: false, steps: const <String>[], exitCode: 64);
  }

  if (!fast && !File('reapers_server/config/passwords.yaml').existsSync()) {
    stderr.writeln(
      'The server tests need reapers_server/config/passwords.yaml, which is '
      'git-ignored.\nCreate it from the committed example:\n'
      '    cp reapers_server/config/passwords.example.yaml '
      'reapers_server/config/passwords.yaml',
    );

    return (passed: false, steps: const <String>[], exitCode: 66);
  }

  final passed = <String>[];

  for (final step in _steps(fast: fast)) {
    stdout.writeln('── ${step.name}');

    final process = await Process.start(
      step.command.first,
      step.command.skip(1).toList(),
      workingDirectory: step.directory,
      mode: ProcessStartMode.inheritStdio,
    );
    final code = await process.exitCode;

    if (code != 0) {
      stderr.writeln('✗ ${step.name} failed (exit $code)');

      return (passed: false, steps: passed, exitCode: code);
    }

    passed.add(step.name);
  }

  return (passed: true, steps: passed, exitCode: 0);
}

Future<void> main(List<String> arguments) async {
  final fast = arguments.contains('--fast');
  final watch = Stopwatch()..start();
  final report = await runChecks(fast: fast);

  if (!report.passed) {
    exit(report.exitCode);
  }

  print(
    '✓ all checks passed in ${watch.elapsed.inSeconds}s'
    '${fast ? ' (server tests skipped)' : ''}',
  );
}
