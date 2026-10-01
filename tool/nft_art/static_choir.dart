/// Static Choir: singing faces made of television static, built from scan
/// rows and blocky pixels with the colour channels slipping apart.
library;

import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'canvas.dart';
import 'geometry.dart';
import 'palette.dart';

/// Paints the chorister for [nft].
void paintChorister(Canvas canvas, Nft nft) {
  final singer = _Singer.of(nft);

  _paintAir(canvas, singer);
  _paintBust(canvas, singer);
  _paintEyes(canvas, singer);
  _paintMouth(canvas, singer);
  _paintAerial(canvas, singer);
  _paintScanGaps(canvas, singer);
}

/// The height of one scan row. Every edge in the collection sits on a
/// multiple of half of it, which is what keeps the set looking like pixels.
const _pitch = 16.0;

/// The dark gap between scan rows.
const _gap = 4.0;

const _rows = designSize ~/ _pitch;

/// The rows the head occupies, from its crown to its chin.
const _crownRow = 4;
const _chinRow = 26;

/// The first row of each eye and of the mouth.
const _eyeRow = 11;
const _mouthRow = 17;

/// How many rows the tallest mouth opens across.
const _tallestMouth = 7;

/// How far each colour channel has slipped from the picture.
const _slip = 10.0;

/// The slipped channels. They are the same on every chorister, so the cyan
/// and red fringes act as the collection's signature.
const _cool = Rgb(0x1FD6E6);
const _warm = Rgb(0xFF3B6B);

enum _Skull { oval, block, dome }

enum _Eyes { bars, dots }

enum _Mouth { tall, wide, round, box }

enum _Extra { none, aerial, levels, halo }

enum _Air { noise, testCard }

/// The phosphor colours of one chorister.
final class _Signal {
  const _Signal({
    required this.night,
    required this.bright,
    required this.mid,
    required this.dim,
  });

  final Rgb night;
  final Rgb bright;
  final Rgb mid;
  final Rgb dim;
}

/// The everyday phosphors, cycled through by serial.
const _phosphors = [
  _Signal(
    night: Rgb(0x0B0E1A),
    bright: Rgb(0xE8F1F5),
    mid: Rgb(0xAFC0CC),
    dim: Rgb(0x73879A),
  ),
  _Signal(
    night: Rgb(0x08130F),
    bright: Rgb(0xC8F7C0),
    mid: Rgb(0x7FDB8C),
    dim: Rgb(0x3F9E5E),
  ),
  _Signal(
    night: Rgb(0x160D08),
    bright: Rgb(0xFFE2A0),
    mid: Rgb(0xF2AF4A),
    dim: Rgb(0xB8732A),
  ),
  _Signal(
    night: Rgb(0x150A1A),
    bright: Rgb(0xFFD0E4),
    mid: Rgb(0xF08CB8),
    dim: Rgb(0xA8528A),
  ),
  _Signal(
    night: Rgb(0x070D1F),
    bright: Rgb(0xCFE9FF),
    mid: Rgb(0x7FB6F0),
    dim: Rgb(0x4A78C0),
  ),
  _Signal(
    night: Rgb(0x0E0A1E),
    bright: Rgb(0xE2D4FF),
    mid: Rgb(0xA88CF0),
    dim: Rgb(0x6A52B8),
  ),
];

/// A clean white picture, kept for rare choristers and shown over a test card.
const _broadcast = _Signal(
  night: Rgb(0x0A0A12),
  bright: Rgb(0xFFFFFF),
  mid: Rgb(0xD9DCE6),
  dim: Rgb(0xA4A9BA),
);

/// Gold, kept for the epic chorister.
const _golden = _Signal(
  night: Rgb(0x120A1F),
  bright: Rgb(0xFFE08A),
  mid: Rgb(0xF2B93B),
  dim: Rgb(0xC17F1E),
);

/// The bars of a television test card, dimmed so a face stands out on them.
const _testCardBars = [
  Rgb(0x8C8C96),
  Rgb(0x8F8A2A),
  Rgb(0x1F8A8F),
  Rgb(0x2A8F3A),
  Rgb(0x8F2A8A),
  Rgb(0x8F2A32),
  Rgb(0x2A328F),
];

/// The traits of one chorister, chosen from its serial and rarity.
final class _Singer {
  _Singer({
    required this.signal,
    required this.skull,
    required this.eyes,
    required this.mouth,
    required this.extra,
    required this.air,
    required this.random,
  }) : shifts = _rowShifts(random);

  factory _Singer.of(Nft nft) {
    final index = nft.serial - 1;
    final rarity = nft.rarity;

    return _Singer(
      signal: switch (rarity) {
        Rarity.rare => _broadcast,
        Rarity.epic => _golden,
        Rarity.common ||
        Rarity.uncommon => _phosphors[index % _phosphors.length],
      },
      skull: _Skull.values[index % _Skull.values.length],
      eyes: _Eyes.values[index % _Eyes.values.length],
      mouth: _Mouth.values[(index + index ~/ 4) % _Mouth.values.length],
      extra: switch (rarity) {
        Rarity.common || Rarity.rare => _Extra.none,
        Rarity.uncommon => index.isEven ? _Extra.aerial : _Extra.levels,
        Rarity.epic => _Extra.halo,
      },
      air: rarity == Rarity.rare ? _Air.testCard : _Air.noise,
      random: SeededRandom(hashText('art:${nft.id}')),
    );
  }

  final _Signal signal;
  final _Skull skull;
  final _Eyes eyes;
  final _Mouth mouth;
  final _Extra extra;
  final _Air air;
  final SeededRandom random;

  /// How far each scan row has slipped sideways, by row.
  final List<double> shifts;

  /// Half the width of the bust on [row], which is at or below the crown.
  double halfWidth(int row) {
    if (row > _chinRow) {
      return _shoulderHalfWidths[math.min(
        row - _chinRow - 1,
        _shoulderHalfWidths.length - 1,
      )];
    }

    final along = (row - _crownRow + 0.5) / (_chinRow - _crownRow + 1);

    final width = switch (skull) {
      _Skull.oval => 152 * math.pow(1 - math.pow(2 * along - 1, 2), 0.42),
      _Skull.block => along < 0.05 || along > 0.95 ? 112.0 : 136.0,
      _Skull.dome when along < 0.45 =>
        156 * math.sqrt(1 - math.pow((0.45 - along) / 0.45, 2)),
      _Skull.dome => 156 - (along - 0.45) / 0.55 * 72,
    };

    return _snap(width.toDouble());
  }
}

/// The neck, then the shoulders widening to the bottom edge.
const _shoulderHalfWidths = [56.0, 144.0, 184.0, 208.0, 216.0];

/// Rounds [value] to the half-row grid.
double _snap(double value) => (value / (_pitch / 2)).round() * (_pitch / 2);

/// Slips a few rows sideways, and tears one short band of the forehead well
/// out of line, the way a weak signal does.
///
/// The rows that hold the eyes and the mouth stay put, so the face survives
/// the interference.
List<double> _rowShifts(SeededRandom random) {
  final shifts = [
    for (var row = 0; row < _rows; row++)
      !_holdsFeature(row) && random.nextChance(0.36)
          ? random.pick(const [-16.0, -8.0, 8.0, 16.0])
          : 0.0,
  ];
  final tear = _crownRow + 2 + random.nextInt(3);
  final distance = random.pick(const [-32.0, 32.0]);

  shifts[tear] = distance;
  shifts[tear + 1] = distance;

  return shifts;
}

/// Whether [row] crosses an eye socket or the mouth.
bool _holdsFeature(int row) {
  final inEyes = row >= _eyeRow - 1 && row <= _eyeRow + 2;
  final inMouth = row >= _mouthRow && row < _mouthRow + _tallestMouth;

  return inEyes || inMouth;
}

/// The background: static, or a test card behind it for rare choristers.
void _paintAir(Canvas canvas, _Singer singer) {
  final signal = singer.signal;
  final random = singer.random;

  canvas.background(signal.night);

  if (singer.air == _Air.testCard) {
    final width = designSize / _testCardBars.length;

    for (final (index, color) in _testCardBars.indexed) {
      canvas.fill(rect(index * width, 0, width + 1, designSize).shape, color);
    }
  }

  for (var row = 0; row < _rows; row++) {
    for (var column = 0; column < _rows; column++) {
      if (!random.nextChance(0.2)) {
        continue;
      }

      final speck = random.pickWeighted(const [10, 6, 1, 1]);
      final cell = rect(column * _pitch, row * _pitch, _pitch, _pitch).shape;

      switch (speck) {
        case 0:
          canvas.fill(cell, signal.bright, opacity: 0.07);

        case 1:
          canvas.fill(cell, signal.bright, opacity: 0.14);

        case 2:
          canvas.fill(cell, _cool, opacity: 0.5);

        default:
          canvas.fill(cell, _warm, opacity: 0.5);
      }
    }
  }

  switch (singer.extra) {
    case _Extra.halo:
      _paintHalo(canvas, singer);

    case _Extra.levels:
      _paintLevels(canvas, singer);

    case _Extra.none || _Extra.aerial:
      break;
  }
}

/// A ring of pixels behind the head, which only the epic chorister has.
void _paintHalo(Canvas canvas, _Singer singer) {
  const centre = Vec(centreX, 232);

  for (var row = 0; row < _rows; row++) {
    for (var column = 0; column < _rows; column++) {
      final cell = Vec((column + 0.5) * _pitch, (row + 0.5) * _pitch);
      final distance = (cell - centre).length;

      if (distance < 206 || distance > 236) {
        continue;
      }

      _paintSlipped(
        canvas,
        rect(column * _pitch, row * _pitch, _pitch, _pitch),
        singer.signal.bright,
      );
    }
  }
}

/// Fills [area] with [color] over a cyan copy slipped to the left and a red
/// copy slipped to the right, as if the colour channels had come apart.
void _paintSlipped(Canvas canvas, Path area, Rgb color, {double slip = _slip}) {
  canvas.fill(area.shifted(-slip, 0).shape, _cool);
  canvas.fill(area.shifted(slip, 0).shape, _warm);
  canvas.fill(area.shape, color);
}

/// Sound-level columns either side of the head.
void _paintLevels(Canvas canvas, _Singer singer) {
  const columns = [8.0, 40.0, 72.0];
  const base = 416.0;

  for (final (index, left) in columns.indexed) {
    final height = _pitch * (4 + singer.random.nextInt(9) + index * 3);
    final color = index.isEven ? _cool : _warm;

    canvas.fill(rect(left, base - height, 24, height).shape.withMirror, color);
  }
}

/// The head, neck and shoulders, row by row.
void _paintBust(Canvas canvas, _Singer singer) {
  final signal = singer.signal;
  final random = singer.random;

  for (var row = _crownRow; row < _rows; row++) {
    final half = singer.halfWidth(row);
    final left = centreX - half + singer.shifts[row];
    final top = row * _pitch;

    // A dark surround first, so the head keeps its edge against a busy
    // background.
    canvas.fill(
      rect(left - _slip - 8, top, half * 2 + _slip * 2 + 16, _pitch).shape,
      signal.night,
    );
    _paintSlipped(canvas, rect(left, top, half * 2, _pitch), signal.bright);

    for (var x = left; x < left + half * 2; x += _pitch) {
      final grain = random.pickWeighted(const [12, 5, 3]);

      if (grain == 0) {
        continue;
      }

      canvas.fill(
        rect(x, top, _pitch, _pitch).shape,
        grain == 1 ? signal.mid : signal.dim,
      );
    }
  }
}

void _paintEyes(Canvas canvas, _Singer singer) {
  final signal = singer.signal;
  final shift = singer.shifts[_eyeRow];
  final top = _eyeRow * _pitch;

  final (width, height) = switch (singer.eyes) {
    _Eyes.bars => (72.0, 16.0),
    _Eyes.dots => (32.0, 32.0),
  };

  for (final side in [-1, 1]) {
    final middle = centreX + side * 68 + shift;
    final socketTop = top - _pitch;

    canvas.fill(
      rect(middle - 48, socketTop, 96, _pitch * 2 + height).shape,
      signal.night,
    );
    _paintSlipped(
      canvas,
      rect(middle - width / 2, top, width, height),
      Rgb.white,
      slip: 6,
    );
  }
}

/// The open, singing mouth: a dark hole with a red throat low in it.
void _paintMouth(Canvas canvas, _Singer singer) {
  final halfWidths = switch (singer.mouth) {
    _Mouth.tall => const [24.0, 40.0, 48.0, 48.0, 48.0, 40.0, 24.0],
    _Mouth.wide => const [56.0, 80.0, 88.0, 88.0, 72.0, 48.0],
    _Mouth.round => const [32.0, 48.0, 56.0, 56.0, 48.0, 32.0],
    _Mouth.box => const [56.0, 56.0, 56.0, 56.0, 56.0],
  };
  final throatFrom = halfWidths.length - 3;

  for (final (index, half) in halfWidths.indexed) {
    final row = _mouthRow + index;
    final middle = centreX + singer.shifts[row];
    final top = row * _pitch;

    canvas.fill(
      rect(middle - half, top, half * 2, _pitch).shape,
      singer.signal.night,
    );

    if (index < throatFrom || index == halfWidths.length - 1) {
      continue;
    }

    canvas.fill(
      rect(middle - half + 16, top, half * 2 - 32, _pitch).shape,
      _warm,
    );
  }
}

/// A pair of rabbit-ear aerials stepping up and out from the crown.
void _paintAerial(Canvas canvas, _Singer singer) {
  if (singer.extra != _Extra.aerial) {
    return;
  }

  const steps = 3;

  for (final side in [-1, 1]) {
    for (var step = 0; step < steps; step++) {
      final middle = centreX + side * (40 + step * 32);
      final top = (_crownRow - 1 - step) * _pitch;

      _paintSlipped(
        canvas,
        rect(middle - 16, top, 32, _pitch),
        singer.signal.bright,
      );
    }

    final tip = centreX + side * (40 + steps * 32);

    canvas.fill(rect(tip - 24, 0, 48, _pitch).shape, _warm);
  }
}

/// The dark lines between scan rows, drawn over everything so that every
/// layer ends up made of the same bars.
void _paintScanGaps(Canvas canvas, _Singer singer) {
  for (var row = 0; row < _rows; row++) {
    canvas.fill(
      rect(0, (row + 1) * _pitch - _gap, designSize, _gap).shape,
      singer.signal.night,
      opacity: 0.85,
    );
  }
}
