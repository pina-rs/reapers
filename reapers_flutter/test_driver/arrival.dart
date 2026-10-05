import 'dart:io';

import 'package:integration_test/integration_test_driver_extended.dart';

Future<void> main() => integrationDriver(
  responseDataCallback: null,
  onScreenshot: (name, bytes, [arguments]) async {
    final screenshot = File('../.impeccable/review/$name.png');
    await screenshot.parent.create(recursive: true);
    await screenshot.writeAsBytes(bytes);
    return bytes.isNotEmpty;
  },
);
