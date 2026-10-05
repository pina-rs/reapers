// Runs the native arrival journey in a separate iOS app container. The
// normal simulator install and its books must survive this test.
import 'dart:io';

Future<void> main(List<String> arguments) async {
  if (arguments.length != 1) {
    stderr.writeln('Usage: fvm dart tool/test_arrival.dart <iOS device id>');
    exitCode = 64;
    return;
  }
  final project = File('reapers_flutter/ios/Runner.xcodeproj/project.pbxproj');
  final original = await project.readAsString();
  final isolated = original.replaceAll(
    'PRODUCT_BUNDLE_IDENTIFIER = com.reapers.app;',
    'PRODUCT_BUNDLE_IDENTIFIER = com.reapers.app.arrivalTest;',
  );
  if (isolated == original) {
    throw StateError('Could not isolate the arrival test app identifier.');
  }
  final backup = File('.dart_tool/arrival-test/project.pbxproj');
  await backup.parent.create(recursive: true);
  await backup.writeAsString(original);
  await project.writeAsString(isolated);
  try {
    final process = await Process.start(
      'fvm',
      [
        'flutter',
        'drive',
        '--no-pub',
        '--driver=test_driver/arrival.dart',
        '--target=integration_test/arrival_test.dart',
        '-d',
        arguments.single,
      ],
      workingDirectory: 'reapers_flutter',
      mode: ProcessStartMode.inheritStdio,
    );
    exitCode = await process.exitCode;
  } finally {
    // Do not overwrite an unrelated edit made while the test was running.
    if (await project.readAsString() != isolated) {
      throw StateError(
        'Project changed during the test. Original saved at ${backup.path}.',
      );
    }
    await project.writeAsString(original);
  }
}
