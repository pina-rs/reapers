/// Builds the game's music from the recordings it is made of.
///
///     dart tool/build_music.dart
///
/// Nothing here is composed or generated. Each piece is a recording somebody
/// made and released for anyone to use, fetched from where its maker put it,
/// checked against a known checksum, and re-encoded at a size a phone can
/// carry. Who made each one, and on what terms, is in
/// `reapers_flutter/assets/music/CREDITS.md`, and the game's colophon says
/// the same to the player: all three ask for that credit.
///
/// The originals are kept in `art/build/music`, which is not committed. The
/// encoded pieces are, so nobody needs to run this to build the game. Run it
/// to change a piece or how it is encoded. It needs `ffmpeg` and `curl`.
library;

import 'dart:io';

/// One piece of music: where the original is, and what it becomes.
typedef _Piece = ({
  String name,
  String url,
  String original,
  String sha256,
  double gainDecibels,
});

const _originals = 'art/build/music';
const _encoded = 'reapers_flutter/assets/music';

const _pieces = <_Piece>[
  (
    // Kevin MacLeod, Ossuary 1 - A Beginning. CC BY 4.0. "At the
    // intersection of relaxation and horror": a slow drone, uneasy but
    // something to come back to.
    name: 'sanctum',
    url: 'https://incompetech.com/music/royalty-free/mp3-royaltyfree/Ossuary%201%20-%20A%20Beginning.mp3',
    original: 'macleod_ossuary_1.mp3',
    sha256: '2c3140481edd0c6ab5ff21b46198a97755822e74e5fd0c78f2e7b456cece3cf8',
    gainDecibels: -1,
  ),
  (
    // Kevin MacLeod, Penumbra. CC BY 4.0. Something in the shadows that
    // cannot quite be heard. Brought down to sit with the others.
    name: 'hunt',
    url: 'https://incompetech.com/music/royalty-free/mp3-royaltyfree/Penumbra.mp3',
    original: 'macleod_penumbra.mp3',
    sha256: '55bcea6dbd758694862a8c3de03c4f293cce2c13bd752fa570886d8849f43cfb',
    gainDecibels: -4,
  ),
  (
    // Kevin MacLeod, Oppressive Gloom. CC BY 4.0. Almost a death march.
    name: 'rift',
    url: 'https://incompetech.com/music/royalty-free/mp3-royaltyfree/Oppressive%20Gloom.mp3',
    original: 'macleod_oppressive_gloom.mp3',
    sha256: 'd630686bde6c92f832fb0cb16f6ce9009bf502c7db701139329af2a7683b1020',
    gainDecibels: -3,
  ),
];

Future<void> main() async {
  Directory(_originals).createSync(recursive: true);
  Directory(_encoded).createSync(recursive: true);

  for (final piece in _pieces) {
    final original = File('$_originals/${piece.original}');

    if (!original.existsSync()) {
      stdout.writeln('fetching ${piece.original}');
      await _run('curl', [
        '--fail',
        '--location',
        '--silent',
        '--show-error',
        '--output',
        original.path,
        piece.url,
      ]);
    }

    final checksum = await _sha256(original.path);

    if (checksum != piece.sha256) {
      stderr.writeln(
        '${original.path} is not the recording this piece was chosen as: '
        'its checksum is $checksum. Delete it to fetch it again.',
      );
      exit(1);
    }

    final encoded = '$_encoded/${piece.name}.ogg';

    // Stereo at 44.1 kHz, about 160 kbit/s: what a phone's speaker or a pair
    // of earphones can give back. Nothing is done to the sound itself but
    // a plain change of level.
    await _run('ffmpeg', [
      '-loglevel',
      'error',
      '-y',
      '-i',
      original.path,
      '-map',
      '0:a:0',
      '-map_metadata',
      '-1',
      '-af',
      'volume=${piece.gainDecibels}dB',
      '-ar',
      '44100',
      '-ac',
      '2',
      '-c:a',
      'libvorbis',
      '-q:a',
      '5',
      encoded,
    ]);

    final megabytes = File(encoded).lengthSync() / (1024 * 1024);

    stdout.writeln('wrote  $encoded (${megabytes.toStringAsFixed(1)} MB)');
  }
}

Future<void> _run(String executable, List<String> arguments) async {
  final process = await Process.start(
    executable,
    arguments,
    mode: ProcessStartMode.inheritStdio,
  );
  final code = await process.exitCode;

  if (code != 0) {
    stderr.writeln('$executable failed.');
    exit(code);
  }
}

Future<String> _sha256(String path) async {
  final result = await Process.run('shasum', ['-a', '256', path]);

  if (result.exitCode != 0) {
    stderr.writeln('Could not checksum $path: ${result.stderr}');
    exit(result.exitCode);
  }

  return (result.stdout as String).split(' ').first;
}
