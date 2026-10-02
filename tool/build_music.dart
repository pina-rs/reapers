/// Builds the game's music from the recordings it is made of.
///
///     dart tool/build_music.dart
///
/// Nothing here is composed or generated. Each piece is a recording somebody
/// made and released for anyone to use, fetched from where its maker put it,
/// checked against a known checksum, and re-encoded at a size a phone can
/// carry. Who made each one, and on what terms, is in
/// `reapers_flutter/assets/music/CREDITS.md`, and the game's colophon says
/// the same to the player: two of the three ask for that credit.
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
    // J.S. Bach, Prelude in E-flat minor, BWV 853. Kimiko Ishizaka, piano.
    // Public domain. Quiet as recorded, so it is brought up to sit with the
    // others.
    name: 'sanctum',
    url: 'https://upload.wikimedia.org/wikipedia/commons/8/8a/Kimiko_Ishizaka_-_Bach-_Well-Tempered_Clavier%2C_Book_1_-_15_Prelude_No._8_in_E-flat_minor%2C_BWV_853.flac',
    original: 'bach_bwv_853_ishizaka.flac',
    sha256: '464f817f2e536ee2e7f323be27b47add369443a27fa6446e39bbc16ea593458f',
    gainDecibels: 6,
  ),
  (
    // Scott Buckley, Anabasis I. CC BY 4.0.
    name: 'hunt',
    url: 'https://www.scottbuckley.com.au/library/wp-content/uploads/2026/09/Anabasis_I.mp3',
    original: 'buckley_anabasis_i.mp3',
    sha256: '964e693611445d808d4cdb748006ae22c7fa8534b126acdca39bcc74de95d76c',
    gainDecibels: 0,
  ),
  (
    // Scott Buckley, Katabasis I. CC BY 4.0.
    name: 'rift',
    url: 'https://www.scottbuckley.com.au/library/wp-content/uploads/2026/09/Katabasis_I.mp3',
    original: 'buckley_katabasis_i.mp3',
    sha256: 'bdce0cfde5f900a60b5c7f769261d0f8438a77e6e6bce351071de0c6d25d0f58',
    gainDecibels: 0,
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
