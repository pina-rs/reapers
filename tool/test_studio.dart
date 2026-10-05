// Exercises the real renderer and soul controls in an isolated phone app.
// Neither the normal install nor its saved reapers may be replaced.
import 'dart:io';

Future<void> main(List<String> arguments) async {
  if (arguments.length != 2 || !['android', 'ios'].contains(arguments.first)) {
    stderr.writeln(
      'Usage: fvm dart tool/test_studio.dart <android|ios> <device id>',
    );
    exitCode = 64;
    return;
  }
  final android = arguments.first == 'android';
  final project = File(
    android
        ? 'reapers_flutter/android/app/build.gradle.kts'
        : 'reapers_flutter/ios/Runner.xcodeproj/project.pbxproj',
  );
  final original = await project.readAsString();
  final isolated = original.replaceAll(
    android
        ? 'applicationId = "com.reapers.app"'
        : 'PRODUCT_BUNDLE_IDENTIFIER = com.reapers.app;',
    android
        ? 'applicationId = "com.reapers.app.studio_test"'
        : 'PRODUCT_BUNDLE_IDENTIFIER = com.reapers.app.studioTest;',
  );
  if (isolated == original) {
    throw StateError('Could not isolate the studio test app identifier.');
  }
  final backup = File('.dart_tool/studio-test/${arguments.first}.project');
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
        '--target=integration_test/studio_test.dart',
        '-d',
        arguments.last,
      ],
      workingDirectory: 'reapers_flutter',
      mode: ProcessStartMode.inheritStdio,
    );
    exitCode = await process.exitCode;
  } finally {
    if (await project.readAsString() != isolated) {
      throw StateError(
        'Project changed during the test. Original saved at ${backup.path}.',
      );
    }
    await project.writeAsString(original);
  }
}
