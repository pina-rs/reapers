// Builds native icon sizes from the chosen imagegen master, then runs the
// platform generators. Run from the root: fvm dart tool/build_branding.dart
// ignore_for_file: avoid_print
import 'dart:io';

Future<void> _run(
  String executable,
  List<String> arguments, {
  String? directory,
}) async {
  final process = await Process.start(
    executable,
    arguments,
    workingDirectory: directory,
    mode: ProcessStartMode.inheritStdio,
  );
  final code = await process.exitCode;
  if (code != 0) {
    throw ProcessException(
      executable,
      arguments,
      'Branding build failed',
      code,
    );
  }
}

Future<void> main(List<String> arguments) async {
  final source = arguments.isEmpty
      ? 'art/branding/quiet-cut.png'
      : arguments.single;
  if (!File(source).existsSync()) {
    throw ArgumentError('Missing logo master: $source');
  }
  const assets = 'reapers_flutter/assets/branding';
  Directory(assets).createSync(recursive: true);
  final provenance =
      'Origin: $source, sized and packed by tool/build_branding.dart. '
      'Generation prompts and cleanup direction: art/branding/prompts.json.';

  Future<void> canvas(
    String name,
    int canvas,
    int mark, {
    bool opaque = false,
    bool white = false,
  }) => _run('magick', [
    source,
    '-trim',
    '+repage',
    '-resize',
    '${mark}x$mark',
    if (white) ...[
      '-channel',
      'RGB',
      '-fill',
      'white',
      '-colorize',
      '100',
      '+channel',
    ],
    '-gravity',
    'center',
    '-background',
    opaque ? '#07080B' : 'none',
    '-extent',
    '${canvas}x$canvas',
    if (opaque) ...[
      '-background',
      '#07080B',
      '-alpha',
      'remove',
      '-alpha',
      'off',
    ],
    '-set',
    'impeccable:prompt',
    provenance,
    '$assets/$name.png',
  ]);

  await canvas('mark', 512, 512);
  await canvas('app_icon', 1024, 780, opaque: true);
  await canvas('adaptive_foreground', 1080, 600);
  await canvas('adaptive_monochrome', 1080, 600, white: true);
  // Android 12's 288dp artboard: 180dp mark inside its 192dp safe circle.
  await canvas('splash_mark', 1152, 720);

  for (final (density, size) in [
    ('mdpi', 24),
    ('hdpi', 36),
    ('xhdpi', 48),
    ('xxhdpi', 72),
    ('xxxhdpi', 96),
  ]) {
    final directory =
        'reapers_flutter/android/app/src/main/res/drawable-$density';
    Directory(directory).createSync(recursive: true);
    await _run('magick', [
      source,
      '-trim',
      '+repage',
      '-resize',
      '${size}x$size',
      '-channel',
      'RGB',
      '-fill',
      'white',
      '-colorize',
      '100',
      '+channel',
      '-gravity',
      'center',
      '-background',
      'none',
      '-extent',
      '${size}x$size',
      '-set',
      'impeccable:prompt',
      provenance,
      '$directory/ic_notification.png',
    ]);
  }
  await _run('fvm', [
    'dart',
    'run',
    'flutter_launcher_icons',
  ], directory: 'reapers_flutter');
  // 0.14.4 also matches this similarly named setting when selecting
  // AppIcon. Keep its boolean value; the icon catalogue name is separate.
  final project = File('reapers_flutter/ios/Runner.xcodeproj/project.pbxproj');
  await project.writeAsString(
    (await project.readAsString()).replaceAll(
      'ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = AppIcon;',
      'ASSETCATALOG_COMPILER_GENERATE_SWIFT_ASSET_SYMBOL_EXTENSIONS = YES;',
    ),
  );
  await _run('fvm', [
    'dart',
    'run',
    'flutter_native_splash:create',
  ], directory: 'reapers_flutter');
  // The generator leaves whitespace on an empty Android 12 branding line.
  for (final variant in ['values-v31', 'values-night-v31']) {
    final styles = File(
      'reapers_flutter/android/app/src/main/res/$variant/styles.xml',
    );
    await styles.writeAsString(
      (await styles.readAsString()).replaceAll(
        RegExp(r'[ \t]+$', multiLine: true),
        '',
      ),
    );
  }
  print('Branding built from $source.');
}
