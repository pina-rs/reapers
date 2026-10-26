import 'dart:io';

/// Runs the Android hunt journey in its own install. Flutter drive removes
/// its target on exit, so it must never target the player's normal game.
Future<void> main(List<String> arguments) async {
  if (arguments.length != 1) {
    stderr.writeln('Usage: fvm dart tool/test_hunt.dart <Android device id>');
    exitCode = 64;
    return;
  }

  const package = 'com.reapers.app.huntTest';
  final device = arguments.single;
  final project = File('reapers_flutter/android/app/build.gradle.kts');
  final original = await project.readAsString();
  final isolated = original.replaceAll(
    'applicationId = "com.reapers.app"',
    'applicationId = "$package"',
  );

  if (isolated == original) {
    throw StateError('Could not isolate the hunt test application id.');
  }

  final backup = File('.dart_tool/hunt-test/build.gradle.kts');
  await backup.parent.create(recursive: true);
  await backup.writeAsString(original);
  await project.writeAsString(isolated);

  try {
    // Install first, then grant the test package camera permission. A new
    // install must not leave the Flutter test waiting on a native dialog.
    for (final command in [
      [
        'fvm',
        'flutter',
        'build',
        'apk',
        '--debug',
        '--no-pub',
        '--target-platform=android-arm64',
        '--target=integration_test/hunt_test.dart',
        '--dart-define=REAPERS_AUTOPLAY=true',
      ],
      [
        'adb',
        '-s',
        device,
        'install',
        '-r',
        'build/app/outputs/flutter-apk/app-debug.apk',
      ],
      [
        'adb',
        '-s',
        device,
        'shell',
        'pm',
        'grant',
        package,
        'android.permission.CAMERA',
      ],
      ['adb', '-s', device, 'shell', 'input', 'keyevent', 'KEYCODE_WAKEUP'],
      [
        'fvm',
        'flutter',
        'drive',
        '--no-pub',
        '--driver=test_driver/arrival.dart',
        '--target=integration_test/hunt_test.dart',
        '--dart-define=REAPERS_AUTOPLAY=true',
        '--use-application-binary=build/app/outputs/flutter-apk/app-debug.apk',
        '--keep-app-running',
        '-d',
        device,
      ],
      // Only this fixed test package may be removed, even if a build ever
      // produces an APK with the wrong application id.
      ['adb', '-s', device, 'uninstall', package],
    ]) {
      final process = await Process.start(
        command.first,
        command.skip(1).toList(),
        workingDirectory: 'reapers_flutter',
        mode: ProcessStartMode.inheritStdio,
      );
      final result = await process.exitCode;

      if (result != 0) {
        exitCode = result;
        return;
      }
    }
  } finally {
    if (await project.readAsString() != isolated) {
      throw StateError(
        'Project changed during the test. Original saved at ${backup.path}.',
      );
    }

    await project.writeAsString(original);
  }
}
