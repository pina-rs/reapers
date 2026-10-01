/// Tin Saints: wind-up tin-toy saints, each a bust facing forward with a halo
/// behind a riveted head.
library;

import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'canvas.dart';
import 'geometry.dart';
import 'palette.dart';

/// Paints the saint for [nft].
void paintTinSaint(Canvas canvas, Nft nft) {
  final saint = _Saint.of(nft);

  _paintBackdrop(canvas, saint);
  _paintHalo(canvas, saint);
  _paintBust(canvas, saint);
  _paintTopper(canvas, saint);
  _paintHead(canvas, saint);
  _paintCrown(canvas, saint);
  _paintFace(canvas, saint);
}

/// Where the halo sits. Every saint shares it, which is what makes the set
/// read as one collection at thumbnail size.
const _haloCentre = Vec(centreX, 212);
const _haloRadius = 172.0;

enum _HeadShape { round, square, bell }

enum _Eyes { dots, washers, slits, lids }

enum _Mouth { slot, smile, hum, grille }

enum _Topper { key, antenna, loop, crown }

enum _Halo { plate, ring, rays, twinRing, gear }

enum _Backdrop { flat, band, niche, sunburst, night }

/// The lithographed colours of one saint.
final class _Enamel {
  const _Enamel({
    required this.backdrop,
    required this.backdropAccent,
    required this.tin,
    required this.trim,
    required this.halo,
    required this.ink,
    required this.metal,
    required this.blush,
  });

  final Rgb backdrop;
  final Rgb backdropAccent;

  /// The head.
  final Rgb tin;

  /// The shoulders, painted in a second colour as tin toys usually are.
  final Rgb trim;
  final Rgb halo;

  /// The stamped eyes and mouth.
  final Rgb ink;

  /// Keys, antennae and rivets.
  final Rgb metal;
  final Rgb blush;
}

const _cream = Rgb(0xF4E6C6);
const _navy = Rgb(0x1E2742);
const _mustard = Rgb(0xF0B73C);
const _tomato = Rgb(0xD94B3D);

/// The everyday colourways, cycled through by serial.
const _enamels = [
  _Enamel(
    backdrop: Rgb(0x2E8B88),
    backdropAccent: Rgb(0x237370),
    tin: _tomato,
    trim: _cream,
    halo: _mustard,
    ink: _navy,
    metal: _navy,
    blush: Rgb(0xF29B7F),
  ),
  _Enamel(
    backdrop: Rgb(0xEDB63E),
    backdropAccent: Rgb(0xDB9C2A),
    tin: Rgb(0x2F7396),
    trim: _tomato,
    halo: _cream,
    ink: _cream,
    metal: _navy,
    blush: Rgb(0x5FA3C2),
  ),
  _Enamel(
    backdrop: Rgb(0xE2644F),
    backdropAccent: Rgb(0xC94C3B),
    tin: _cream,
    trim: Rgb(0x2E8B88),
    halo: _mustard,
    ink: _navy,
    metal: _navy,
    blush: Rgb(0xF0A08C),
  ),
  _Enamel(
    backdrop: Rgb(0x8FC7CC),
    backdropAccent: Rgb(0x6FAFB6),
    tin: Rgb(0xEC9A33),
    trim: Rgb(0x2B3A67),
    halo: _cream,
    ink: _navy,
    metal: _navy,
    blush: Rgb(0xF6C36B),
  ),
  _Enamel(
    backdrop: Rgb(0x26315C),
    backdropAccent: Rgb(0x1B2447),
    tin: Rgb(0x8FD0B4),
    trim: _tomato,
    halo: _mustard,
    ink: _navy,
    metal: _navy,
    blush: Rgb(0xF2A7A0),
  ),
  _Enamel(
    backdrop: Rgb(0x2F5D50),
    backdropAccent: Rgb(0x244A3F),
    tin: Rgb(0xE88D93),
    trim: _cream,
    halo: _cream,
    ink: _navy,
    metal: _navy,
    blush: Rgb(0xF6B9B0),
  ),
];

/// Weathered copper, kept for rare saints.
const _patina = _Enamel(
  backdrop: Rgb(0x4A2459),
  backdropAccent: Rgb(0x66317A),
  tin: Rgb(0x5DBBA3),
  trim: Rgb(0xC8703F),
  halo: Rgb(0xEFA35F),
  ink: Rgb(0x26143A),
  metal: Rgb(0x26143A),
  blush: Rgb(0x9BDCC6),
);

/// Gold on a night sky, kept for the epic saint.
const _gilt = _Enamel(
  backdrop: Rgb(0x14173A),
  backdropAccent: Rgb(0x232A66),
  tin: Rgb(0xF0B32E),
  trim: Rgb(0xDB971F),
  halo: Rgb(0xFFF1C2),
  ink: Rgb(0x3B2106),
  metal: Rgb(0x3B2106),
  blush: Rgb(0xFFDE85),
);

const _jewel = Rgb(0xE23B57);

/// The traits of one saint, chosen from its serial and rarity.
final class _Saint {
  const _Saint({
    required this.enamel,
    required this.head,
    required this.eyes,
    required this.mouth,
    required this.topper,
    required this.halo,
    required this.backdrop,
    required this.capped,
    required this.jewelled,
    required this.random,
  });

  factory _Saint.of(Nft nft) {
    final index = nft.serial - 1;
    final rarity = nft.rarity;
    final exalted = rarity.index >= Rarity.epic.index;

    return _Saint(
      enamel: switch (rarity) {
        Rarity.rare => _patina,
        Rarity.epic => _gilt,
        Rarity.common || Rarity.uncommon => _enamels[index % _enamels.length],
      },
      head: _HeadShape.values[index % _HeadShape.values.length],
      eyes: _Eyes.values[index % _Eyes.values.length],
      // Stepping the mouth out of phase with the eyes keeps any two saints
      // from sharing a whole face.
      mouth: _Mouth.values[(index * 3 + index ~/ 4) % _Mouth.values.length],
      topper: switch (rarity) {
        Rarity.uncommon when index.isOdd => _Topper.crown,
        Rarity.rare => _Topper.loop,
        _ => index.isEven ? _Topper.key : _Topper.antenna,
      },
      halo: switch (rarity) {
        Rarity.common => index.isEven ? _Halo.plate : _Halo.ring,
        Rarity.uncommon => index.isEven ? _Halo.rays : _Halo.ring,
        Rarity.rare => _Halo.twinRing,
        Rarity.epic => _Halo.gear,
      },
      backdrop: switch (rarity) {
        Rarity.rare => _Backdrop.sunburst,
        Rarity.epic => _Backdrop.night,
        Rarity.common || Rarity.uncommon => _Backdrop.values[index % 3],
      },
      capped: rarity == Rarity.common && index.isOdd,
      jewelled: exalted,
      random: SeededRandom(hashText('art:${nft.id}')),
    );
  }

  final _Enamel enamel;
  final _HeadShape head;
  final _Eyes eyes;
  final _Mouth mouth;
  final _Topper topper;
  final _Halo halo;
  final _Backdrop backdrop;

  /// Whether the head above the seam is painted in the trim colour, like a
  /// cap, which half of the common saints are.
  final bool capped;

  /// Whether a gem is set in the forehead, which only the epic saint has.
  final bool jewelled;
  final SeededRandom random;

  _HeadGeometry get geometry => switch (head) {
    _HeadShape.round => _roundHead,
    _HeadShape.square => _squareHead,
    _HeadShape.bell => _bellHead,
  };
}

/// The outline of a head and where its features sit on it.
final class _HeadGeometry {
  const _HeadGeometry({
    required this.outline,
    required this.top,
    required this.browHalfWidth,
    required this.seam,
    required this.eyeLine,
    required this.mouthLine,
    required this.glint,
  });

  final Path outline;

  /// The highest point of the head, where a key or antenna is planted.
  final double top;

  /// Half the width of the head a little below [top], where a crown rests.
  final double browHalfWidth;

  /// The height of the riveted join across the forehead.
  final double seam;
  final double eyeLine;
  final double mouthLine;

  /// The spine of the highlight that makes the head read as polished metal.
  final Path glint;
}

final _roundHead = _HeadGeometry(
  outline: circle(const Vec(centreX, 236), 126),
  top: 110,
  browHalfWidth: 84,
  seam: 168,
  eyeLine: 230,
  mouthLine: 294,
  glint: arc(const Vec(centreX, 236), 100, math.pi * 1.12, math.pi * 1.36),
);

final _squareHead = _HeadGeometry(
  outline: roundedRect(132, 116, 248, 240, 46),
  top: 116,
  browHalfWidth: 104,
  seam: 170,
  eyeLine: 232,
  mouthLine: 298,
  glint: arc(const Vec(190, 172), 32, math.pi * 1.02, math.pi * 1.5),
);

final _bellHead = _HeadGeometry(
  outline: _bellOutline(),
  top: 118,
  browHalfWidth: 74,
  seam: 196,
  eyeLine: 254,
  mouthLine: 312,
  glint: curve(const [Vec(198, 196), Vec(204, 160), Vec(226, 138)]),
);

/// A narrow dome that flares to a wide rim, like a hand bell.
Path _bellOutline() {
  final builder = PathBuilder(const Vec(176, 196))
    ..cubicTo(const Vec(176, 92), const Vec(336, 92), const Vec(336, 196))
    ..cubicTo(const Vec(336, 262), const Vec(376, 300), const Vec(396, 334))
    ..quadTo(const Vec(404, 360), const Vec(376, 360))
    ..lineTo(const Vec(136, 360))
    ..quadTo(const Vec(108, 360), const Vec(116, 334))
    ..cubicTo(const Vec(136, 300), const Vec(176, 262), const Vec(176, 196));

  return builder.build();
}

void _paintBackdrop(Canvas canvas, _Saint saint) {
  final enamel = saint.enamel;

  canvas.background(enamel.backdrop);

  switch (saint.backdrop) {
    case _Backdrop.flat:
      break;

    case _Backdrop.band:
      canvas.fill(
        rect(0, 336, designSize, designSize - 336).shape,
        enamel.backdropAccent,
      );

    case _Backdrop.niche:
      canvas.background(enamel.backdropAccent);
      canvas.fill(
        Shape([
          circle(_haloCentre, 206),
          rect(centreX - 206, _haloCentre.y, 412, designSize),
        ]),
        enamel.backdrop,
      );

    case _Backdrop.sunburst:
      _paintRays(canvas, enamel.backdropAccent, count: 12, spread: 0.5);

    case _Backdrop.night:
      _paintRays(canvas, enamel.backdropAccent, count: 16, spread: 0.34);
      _paintStars(canvas, saint);
  }
}

/// Wedges fanning out from behind the halo. [spread] is the share of each
/// step that a ray takes up.
void _paintRays(
  Canvas canvas,
  Rgb color, {
  required int count,
  required double spread,
}) {
  final step = math.pi * 2 / count;

  for (var ray = 0; ray < count; ray++) {
    final middle = ray * step - math.pi / 2;

    canvas.fill(
      wedge(
        _haloCentre,
        520,
        middle - step * spread / 2,
        middle + step * spread / 2,
      ).shape,
      color,
    );
  }
}

/// Four-pointed stars scattered outside the halo.
void _paintStars(Canvas canvas, _Saint saint) {
  final random = saint.random;

  for (var star = 0; star < 14; star++) {
    final at =
        _haloCentre +
        Vec.polar(random.nextRange(214, 330), random.nextRange(0, math.pi * 2));
    final size = random.nextRange(7, 15);

    canvas.fill(burst(at, size, size * 0.3, 4).shape, saint.enamel.halo);
  }
}

void _paintHalo(Canvas canvas, _Saint saint) {
  final color = saint.enamel.halo;
  final groove = color.shade(0.16);

  switch (saint.halo) {
    case _Halo.plate:
      canvas.fill(circle(_haloCentre, _haloRadius).shape, color);
      canvas.fill(
        ring(_haloCentre, _haloRadius - 16, _haloRadius - 23),
        groove,
      );

    case _Halo.ring:
      canvas.fill(ring(_haloCentre, _haloRadius, _haloRadius - 36), color);

    case _Halo.twinRing:
      canvas.fill(ring(_haloCentre, _haloRadius + 6, _haloRadius - 18), color);
      canvas.fill(ring(_haloCentre, _haloRadius - 32, _haloRadius - 44), color);

    case _Halo.rays:
      canvas.fill(
        burst(_haloCentre, _haloRadius + 30, _haloRadius - 12, 18).shape,
        color,
      );
      canvas.fill(circle(_haloCentre, _haloRadius - 8).shape, color);
      canvas.fill(
        ring(_haloCentre, _haloRadius - 24, _haloRadius - 31),
        groove,
      );

    case _Halo.gear:
      canvas.fill(
        burst(_haloCentre, _haloRadius + 62, _haloRadius - 30, 16).shape,
        saint.enamel.tin,
      );
      canvas.fill(
        cog(_haloCentre, _haloRadius + 16, _haloRadius - 6, 16).shape,
        color,
      );
      canvas.fill(
        ring(_haloCentre, _haloRadius - 22, _haloRadius - 30),
        groove,
      );
  }
}

/// The neck and shoulders the head stands on.
void _paintBust(Canvas canvas, _Saint saint) {
  final enamel = saint.enamel;
  final shoulders = roundedRect(88, 398, 336, 200, 78);

  canvas.fill(rect(centreX - 38, 330, 76, 90).shape, enamel.tin.shade(0.34));
  canvas.fill(shoulders.shape, enamel.trim.shade(0.2));

  canvas.clip(shoulders.shape, () {
    canvas.fill(shoulders.shifted(-22, 0).shape, enamel.trim);
    canvas.fill(rect(0, 398, designSize, 26).shape, enamel.tin.shade(0.14));
  });

  for (final x in [centreX - 108, centreX, centreX + 108]) {
    _paintRivet(canvas, Vec(x, 458), 9, enamel.metal);
  }
}

void _paintRivet(Canvas canvas, Vec at, double radius, Rgb color) {
  canvas.fill(circle(at, radius).shape, color);
  canvas.fill(
    circle(at + Vec(-radius * 0.3, -radius * 0.3), radius * 0.36).shape,
    color.tint(0.55),
  );
}

/// Whatever is planted on top of the head.
void _paintTopper(Canvas canvas, _Saint saint) {
  final enamel = saint.enamel;
  final top = saint.geometry.top;

  switch (saint.topper) {
    case _Topper.key:
      canvas.fill(rect(centreX - 7, top - 44, 14, 56).shape, enamel.metal);
      canvas.fill(_keyWings(top - 60), enamel.metal);

    case _Topper.antenna:
      canvas.fill(rect(centreX - 5, top - 50, 10, 62).shape, enamel.metal);
      _paintRivet(canvas, Vec(centreX, top - 58), 17, enamel.metal);

    case _Topper.loop:
      canvas.fill(rect(centreX - 7, top - 30, 14, 42).shape, enamel.metal);
      canvas.fill(ring(Vec(centreX, top - 52), 28, 13), enamel.metal);

    case _Topper.crown:
      // A crown sits in front of the head, so it is painted after it.
      break;
  }
}

/// A tin crown pressed down over the brow, which marks an uncommon saint.
void _paintCrown(Canvas canvas, _Saint saint) {
  if (saint.topper != _Topper.crown) {
    return;
  }

  final top = saint.geometry.top;
  final half = saint.geometry.browHalfWidth;
  final band = top + 26;

  canvas.fill(
    [
      Vec(centreX - half, band + 30),
      Vec(centreX - half - 10, top - 44),
      Vec(centreX - half * 0.5, band - 22),
      Vec(centreX, top - 70),
      Vec(centreX + half * 0.5, band - 22),
      Vec(centreX + half + 10, top - 44),
      Vec(centreX + half, band + 30),
    ].shape,
    _mustard,
  );
  canvas.fill(
    rect(centreX - half, band + 18, half * 2, 12).shape,
    _mustard.shade(0.2),
  );

  for (final offset in [-half * 0.6, 0.0, half * 0.6]) {
    canvas.fill(circle(Vec(centreX + offset, band + 2), 9).shape, _jewel);
  }
}

/// The two pierced wings of a wind-up key, centred at height [y].
Shape _keyWings(double y) {
  final wing = ellipse(Vec(centreX + 30, y), 30, 22, tilt: -0.3);
  final eye = ellipse(Vec(centreX + 32, y), 13, 8, tilt: -0.3);

  return Shape([wing], holes: [eye]).withMirror;
}

void _paintHead(Canvas canvas, _Saint saint) {
  final enamel = saint.enamel;
  final geometry = saint.geometry;
  final head = geometry.outline;

  canvas.fill(head.shape, enamel.tin.shade(0.2));

  canvas.clip(head.shape, () {
    // The lit side is the head again, nudged toward the light, which leaves a
    // crescent of shadow along the far edge.
    canvas.fill(head.shifted(-20, -10).shape, enamel.tin);

    if (saint.capped) {
      canvas.fill(rect(0, 0, designSize, geometry.seam).shape, enamel.trim);
    }

    canvas.fill(
      rect(0, geometry.seam - 3, designSize, 6).shape,
      enamel.tin.shade(0.3),
    );

    for (final offset in [-96.0, -48.0, 0.0, 48.0, 96.0]) {
      _paintRivet(
        canvas,
        Vec(centreX + offset, geometry.seam),
        8,
        enamel.metal,
      );
    }
  });

  canvas.fill(stroke(geometry.glint, 13), enamel.tin.tint(0.55));
}

void _paintFace(Canvas canvas, _Saint saint) {
  final enamel = saint.enamel;
  final geometry = saint.geometry;

  for (final side in [-1, 1]) {
    final eye = Vec(centreX + side * 54, geometry.eyeLine);

    canvas.fill(
      circle(Vec(centreX + side * 88, geometry.eyeLine + 36), 17).shape,
      enamel.blush,
    );
    _paintEye(canvas, saint, eye);
  }

  _paintMouth(canvas, saint, Vec(centreX, geometry.mouthLine));

  if (saint.jewelled) {
    final brow = Vec(centreX, geometry.seam + 26);

    canvas.fill(burst(brow, 21, 12, 4).shape, enamel.ink);
    canvas.fill(burst(brow, 14, 8, 4).shape, _jewel);
  }
}

void _paintEye(Canvas canvas, _Saint saint, Vec at) {
  final ink = saint.enamel.ink;

  switch (saint.eyes) {
    case _Eyes.dots:
      canvas.fill(circle(at, 16).shape, ink);

    case _Eyes.washers:
      canvas.fill(ring(at, 22, 9), ink);

    case _Eyes.slits:
      canvas.fill(roundedRect(at.x - 25, at.y - 7, 50, 14, 7).shape, ink);

    case _Eyes.lids:
      canvas.fill(
        stroke(
          arc(at + const Vec(0, -10), 22, math.pi * 0.12, math.pi * 0.88),
          10,
        ),
        ink,
      );
  }
}

void _paintMouth(Canvas canvas, _Saint saint, Vec at) {
  final ink = saint.enamel.ink;

  switch (saint.mouth) {
    case _Mouth.slot:
      canvas.fill(roundedRect(at.x - 32, at.y - 7, 64, 14, 7).shape, ink);

    case _Mouth.smile:
      canvas.fill(
        stroke(
          arc(at + const Vec(0, -30), 40, math.pi * 0.22, math.pi * 0.78),
          10,
        ),
        ink,
      );

    case _Mouth.hum:
      canvas.fill(ring(at, 19, 8), ink);

    case _Mouth.grille:
      final grille = roundedRect(at.x - 42, at.y - 17, 84, 34, 10);

      canvas.fill(grille.shape, ink);

      for (final offset in [-20.0, 0.0, 20.0]) {
        canvas.fill(
          roundedRect(at.x + offset - 5, at.y - 10, 10, 20, 4).shape,
          saint.enamel.tin,
        );
      }
  }
}
