/// Brine Hounds: sea-dogs. A hound's head in profile, long in the snout and
/// heavy in the ear, kitted out as a sailor against the sea.
library;

import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'canvas.dart';
import 'geometry.dart';
import 'palette.dart';

/// Paints the hound for [nft].
void paintBrineHound(Canvas canvas, Nft nft) {
  final hound = _Hound.of(nft);

  _paintSea(canvas, hound);
  _paintHead(canvas, hound);
  _paintNeckwear(canvas, hound);
  _paintFace(canvas, hound);
  _paintEar(canvas, hound);
  _paintHeadwear(canvas, hound);
  _paintPipe(canvas, hound);
  _paintSurf(canvas, hound);
}

enum _Headwear { none, gobCap, bachi, captain, souwester, coral }

enum _Neckwear { collar, stripes, kerchief, pearls }

enum _Ear { long, short }

enum _Coat { blaze, plain, mask, spotted }

enum _Sea { bubbles, swell, moonlit, abyss }

/// The colours of one hound.
final class _Livery {
  const _Livery({
    required this.sea,
    required this.seaDeep,
    required this.foam,
    required this.fur,
    required this.ear,
    required this.pale,
    required this.ink,
    required this.cloth,
  });

  final Rgb sea;

  /// The darker water of waves and depths.
  final Rgb seaDeep;

  /// Bubbles, and the moon over a moonlit sea.
  final Rgb foam;
  final Rgb fur;
  final Rgb ear;

  /// The lighter fur of a blaze down the muzzle and throat.
  final Rgb pale;

  /// Eye, nose and mouth.
  final Rgb ink;

  /// The collar or kerchief.
  final Rgb cloth;
}

const _navy = Rgb(0x1B2A4A);
const _white = Rgb(0xF7F3E8);
const _red = Rgb(0xD9483B);
const _gold = Rgb(0xF2BE3E);
const _oilskin = Rgb(0xF5C431);
const _coral = Rgb(0xF0606B);
const _briar = Rgb(0x7A4526);

/// The everyday colourways, cycled through by serial.
const _liveries = [
  _Livery(
    sea: Rgb(0x2A9D9A),
    seaDeep: Rgb(0x1F7F80),
    foam: Rgb(0xBDE8E0),
    fur: Rgb(0xDDA667),
    ear: Rgb(0x8A5230),
    pale: Rgb(0xF4DDB4),
    ink: Rgb(0x2A1A14),
    cloth: _red,
  ),
  _Livery(
    sea: Rgb(0x1F6FA5),
    seaDeep: Rgb(0x175888),
    foam: Rgb(0xBFE0F2),
    fur: Rgb(0xF0DFC0),
    ear: Rgb(0xB95C36),
    pale: Rgb(0xFBF3E0),
    ink: Rgb(0x2A1A14),
    cloth: _red,
  ),
  _Livery(
    sea: Rgb(0x67C7C1),
    seaDeep: Rgb(0x45ACA9),
    foam: Rgb(0xE2F7F2),
    fur: Rgb(0x6F4832),
    ear: Rgb(0x3F2718),
    pale: Rgb(0xC99B6B),
    ink: Rgb(0x1E120C),
    cloth: _white,
  ),
  _Livery(
    sea: Rgb(0x3FAE8A),
    seaDeep: Rgb(0x2E8F70),
    foam: Rgb(0xD3F2E2),
    fur: Rgb(0xC2643A),
    ear: Rgb(0x7A341E),
    pale: Rgb(0xF0C9A0),
    ink: Rgb(0x2A1A14),
    cloth: _navy,
  ),
  _Livery(
    sea: Rgb(0x155E75),
    seaDeep: Rgb(0x0F4A5E),
    foam: Rgb(0xA9DCE6),
    fur: Rgb(0xD5DCE2),
    ear: Rgb(0x5B6672),
    pale: Rgb(0xF4F6F7),
    ink: Rgb(0x1B2026),
    cloth: _red,
  ),
  _Livery(
    sea: Rgb(0x4C86B8),
    seaDeep: Rgb(0x386C9A),
    foam: Rgb(0xD5E8F5),
    fur: Rgb(0x57483C),
    ear: Rgb(0x2E2520),
    pale: Rgb(0xB79A7A),
    ink: Rgb(0x15110E),
    cloth: _red,
  ),
];

/// A white, spotted hound under a full moon, kept for rare hounds.
const _moonlight = _Livery(
  sea: Rgb(0x121C42),
  seaDeep: Rgb(0x0B1230),
  foam: Rgb(0xF3E2AC),
  fur: Rgb(0xEEF1F4),
  ear: Rgb(0x2B3350),
  pale: Rgb(0xFFFFFF),
  ink: Rgb(0x161A30),
  cloth: _red,
);

/// A golden hound in the deep, kept for the epic hound.
const _treasure = _Livery(
  sea: Rgb(0x0C2340),
  seaDeep: Rgb(0x071628),
  foam: Rgb(0xFFE9A6),
  fur: Rgb(0xF4C64E),
  ear: Rgb(0xD08F22),
  pale: Rgb(0xFFE9A6),
  ink: Rgb(0x3B2106),
  cloth: _white,
);

/// The traits of one hound, chosen from its serial and rarity.
final class _Hound {
  const _Hound({
    required this.livery,
    required this.headwear,
    required this.neckwear,
    required this.ear,
    required this.coat,
    required this.sea,
    required this.eyepatch,
    required this.pipe,
    required this.random,
  });

  factory _Hound.of(Nft nft) {
    final index = nft.serial - 1;
    final rarity = nft.rarity;

    // A common hound wears one plain piece of kit, cycling bare, cap,
    // eyepatch, pipe. The rarer hounds are told by their hats instead.
    final kit = rarity == Rarity.common ? index % 4 : -1;

    return _Hound(
      livery: switch (rarity) {
        Rarity.rare => _moonlight,
        Rarity.epic => _treasure,
        Rarity.common || Rarity.uncommon => _liveries[index % _liveries.length],
      },
      headwear: switch (rarity) {
        Rarity.common => kit == 1 ? _Headwear.gobCap : _Headwear.none,
        Rarity.uncommon =>
          index.isEven ? _Headwear.captain : _Headwear.souwester,
        Rarity.rare => _Headwear.bachi,
        Rarity.epic => _Headwear.coral,
      },
      neckwear: switch (rarity) {
        Rarity.epic => _Neckwear.pearls,
        Rarity.rare => _Neckwear.kerchief,
        Rarity.common ||
        Rarity.uncommon => index.isEven ? _Neckwear.collar : _Neckwear.stripes,
      },
      ear: index.isEven ? _Ear.long : _Ear.short,
      coat: switch (rarity) {
        Rarity.rare => _Coat.spotted,
        Rarity.epic => _Coat.plain,
        Rarity.common || Rarity.uncommon => _Coat.values[index % 3],
      },
      sea: switch (rarity) {
        Rarity.rare => _Sea.moonlit,
        Rarity.epic => _Sea.abyss,
        Rarity.common ||
        Rarity.uncommon => index.isEven ? _Sea.bubbles : _Sea.swell,
      },
      eyepatch: kit == 2,
      pipe: kit == 3 || (rarity == Rarity.uncommon && index.isOdd),
      random: SeededRandom(hashText('art:${nft.id}')),
    );
  }

  final _Livery livery;
  final _Headwear headwear;
  final _Neckwear neckwear;
  final _Ear ear;
  final _Coat coat;
  final _Sea sea;
  final bool eyepatch;
  final bool pipe;
  final SeededRandom random;
}

const _eye = Vec(300, 222);

/// Where a hat sits: the crown of the skull.
const _crown = Vec(234, 138);

/// How far a hat is tipped back, in radians, to follow the slope of the
/// skull.
const _hatTilt = -0.14;

/// The head and neck in profile, facing right. Every hound shares this
/// silhouette, so the long snout is what the collection is known by.
final Path _head = _headOutline();

Path _headOutline() {
  final builder = PathBuilder(const Vec(30, 530))
    ..cubicTo(const Vec(52, 440), const Vec(76, 356), const Vec(104, 292))
    ..cubicTo(const Vec(120, 196), const Vec(178, 134), const Vec(246, 130))
    ..cubicTo(const Vec(290, 128), const Vec(322, 150), const Vec(334, 188))
    ..cubicTo(const Vec(368, 194), const Vec(408, 206), const Vec(430, 224))
    ..cubicTo(const Vec(452, 238), const Vec(454, 268), const Vec(434, 282))
    ..cubicTo(const Vec(436, 300), const Vec(424, 314), const Vec(406, 318))
    ..cubicTo(const Vec(376, 328), const Vec(346, 330), const Vec(322, 346))
    ..cubicTo(const Vec(318, 376), const Vec(300, 398), const Vec(282, 406))
    ..cubicTo(const Vec(290, 446), const Vec(316, 490), const Vec(352, 530));

  return builder.build();
}

/// The lower jaw, which shows as a darker lip beneath the muzzle.
Path _jawOutline() {
  final builder = PathBuilder(const Vec(316, 334))
    ..lineTo(const Vec(404, 316))
    ..cubicTo(const Vec(420, 320), const Vec(418, 338), const Vec(402, 342))
    ..cubicTo(const Vec(376, 352), const Vec(344, 358), const Vec(314, 374));

  return builder.build();
}

/// Paler fur along the underside of the muzzle and down the throat. It
/// overshoots the head on purpose and is clipped back to it.
final Path _blaze = blob(const [
  Vec(446, 264),
  Vec(400, 268),
  Vec(356, 292),
  Vec(322, 332),
  Vec(296, 386),
  Vec(256, 446),
  Vec(232, 540),
  Vec(410, 540),
  Vec(372, 404),
  Vec(404, 344),
  Vec(462, 312),
]);

Path _earOutline(_Ear ear) {
  final builder = switch (ear) {
    _Ear.long =>
      PathBuilder(const Vec(196, 160))
        ..cubicTo(const Vec(142, 150), const Vec(92, 214), const Vec(94, 300))
        ..cubicTo(const Vec(94, 372), const Vec(132, 410), const Vec(164, 388))
        ..cubicTo(const Vec(196, 360), const Vec(218, 270), const Vec(226, 206))
        ..cubicTo(
          const Vec(228, 180),
          const Vec(216, 164),
          const Vec(196, 160),
        ),
    _Ear.short =>
      PathBuilder(const Vec(188, 152))
        ..cubicTo(const Vec(148, 142), const Vec(112, 176), const Vec(120, 232))
        ..cubicTo(const Vec(128, 286), const Vec(172, 312), const Vec(198, 280))
        ..cubicTo(const Vec(216, 250), const Vec(228, 204), const Vec(222, 172))
        ..cubicTo(
          const Vec(216, 156),
          const Vec(202, 152),
          const Vec(188, 152),
        ),
  };

  return builder.build();
}

void _paintSea(Canvas canvas, _Hound hound) {
  final livery = hound.livery;

  canvas.background(livery.sea);

  switch (hound.sea) {
    case _Sea.bubbles:
      _paintBubbles(canvas, hound, count: 9);

    case _Sea.swell:
      _paintWaveRow(canvas, 400, 0, livery.seaDeep);
      _paintBubbles(canvas, hound, count: 6);

    case _Sea.moonlit:
      canvas.fill(circle(const Vec(240, 214), 200).shape, livery.foam);
      _paintWaveRow(canvas, 392, 0, livery.seaDeep);

    case _Sea.abyss:
      _paintLightShafts(canvas, livery.foam);
      _paintBubbles(canvas, hound, count: 11);
  }
}

/// Slanting shafts of light from the surface far above.
void _paintLightShafts(Canvas canvas, Rgb color) {
  for (final (index, x) in const [60.0, 210.0, 380.0].indexed) {
    final width = 70.0 + index * 24;

    canvas.fill(
      [
        Vec(x, 0),
        Vec(x + width, 0),
        Vec(x + width - 140, designSize),
        Vec(x - 160, designSize),
      ].shape,
      color,
      opacity: 0.13,
    );
  }
}

/// A scalloped row of waves whose crests sit at [top].
void _paintWaveRow(Canvas canvas, double top, double offset, Rgb color) {
  const crest = 44.0;

  canvas.fill(
    Shape([
      rect(0, top + crest * 0.6, designSize, designSize),
      for (var x = offset - crest; x < designSize + crest; x += crest * 2)
        circle(Vec(x, top + crest), crest),
    ]),
    color,
  );
}

/// The corners of the frame that neither the head nor any hat reaches.
const _pockets = [
  (left: 410.0, top: 30.0, width: 78.0, height: 112.0),
  (left: 24.0, top: 30.0, width: 76.0, height: 180.0),
  (left: 388.0, top: 372.0, width: 100.0, height: 110.0),
];

/// Rings of air, kept to the corners so they never cross the face and spaced
/// so they never cross each other.
void _paintBubbles(Canvas canvas, _Hound hound, {required int count}) {
  final random = hound.random;
  final placed = <({Vec at, double radius})>[];

  for (var attempt = 0; attempt < count * 6; attempt++) {
    if (placed.length == count) {
      break;
    }

    final pocket = _pockets[attempt % _pockets.length];
    final radius = random.nextRange(8, 22);
    final at = Vec(
      pocket.left + random.nextDouble() * pocket.width,
      pocket.top + random.nextDouble() * pocket.height,
    );

    if (placed.any(
      (other) => (other.at - at).length < other.radius + radius + 6,
    )) {
      continue;
    }

    placed.add((at: at, radius: radius));
    _paintBubble(canvas, at, radius, hound.livery.foam);
  }
}

void _paintBubble(Canvas canvas, Vec at, double radius, Rgb color) {
  canvas.fill(ring(at, radius, radius - 4.5), color);
  canvas.fill(
    circle(at + Vec(-radius * 0.34, -radius * 0.34), radius * 0.16).shape,
    color,
  );
}

void _paintHead(Canvas canvas, _Hound hound) {
  final livery = hound.livery;

  canvas.fill(_jawOutline().shape, livery.fur.shade(0.3));
  canvas.fill(_head.shape, livery.fur.shade(0.16));

  canvas.clip(_head.shape, () {
    // The lit side is the head again, nudged up and back, which leaves a
    // band of shadow under the muzzle and down the throat.
    canvas.fill(_head.shifted(-12, -12).shape, livery.fur);
    _paintCoat(canvas, hound);
    canvas.fill(circle(const Vec(444, 250), 27).shape, livery.ink);
  });

  canvas.fill(
    ellipse(const Vec(438, 240), 7, 4, tilt: 0.5).shape,
    livery.ink.tint(0.5),
  );
}

void _paintCoat(Canvas canvas, _Hound hound) {
  final livery = hound.livery;

  switch (hound.coat) {
    case _Coat.plain:
      break;

    case _Coat.blaze:
      canvas.fill(_blaze.shape, livery.pale);

    case _Coat.mask:
      canvas.fill(
        ellipse(_eye + const Vec(-6, 2), 52, 46, tilt: -0.3).shape,
        livery.ear,
      );

    case _Coat.spotted:
      final random = hound.random;

      for (var spot = 0; spot < 18; spot++) {
        final at = Vec(random.nextRange(60, 400), random.nextRange(150, 500));
        final radius = random.nextRange(9, 20);
        final tilt = random.nextDouble() * 3;

        canvas.fill(
          ellipse(at, radius, radius * 0.8, tilt: tilt).shape,
          livery.ear,
        );
      }
  }
}

void _paintNeckwear(Canvas canvas, _Hound hound) {
  final livery = hound.livery;

  canvas.clip(_head.shape, () {
    switch (hound.neckwear) {
      case _Neckwear.collar:
        canvas.fill(
          const [
            Vec(20, 420),
            Vec(340, 432),
            Vec(340, 470),
            Vec(20, 458),
          ].shape,
          livery.cloth,
        );

      case _Neckwear.stripes:
        canvas.fill(rect(0, 420, designSize, 120).shape, _white);

        for (var y = 434.0; y < designSize; y += 28) {
          canvas.fill(rect(0, y, designSize, 13).shape, _navy);
        }

      case _Neckwear.kerchief:
        canvas.fill(rect(0, 420, designSize, 120).shape, livery.cloth);
        canvas.fill(
          rect(0, 420, designSize, 13).shape,
          livery.cloth.shade(0.22),
        );

      case _Neckwear.pearls:
        for (var x = 62.0; x < 330; x += 30) {
          canvas.fill(
            circle(Vec(x, 436 + (x - 62) * 0.04), 14).shape,
            livery.cloth,
          );
        }
    }
  });

  if (hound.neckwear == _Neckwear.collar) {
    canvas.fill(circle(const Vec(282, 478), 18).shape, _gold);
    canvas.fill(circle(const Vec(277, 473), 5).shape, _gold.tint(0.6));
  }
}

void _paintFace(Canvas canvas, _Hound hound) {
  final livery = hound.livery;
  final lip = curve(const [
    Vec(414, 312),
    Vec(380, 322),
    Vec(346, 327),
    Vec(324, 342),
  ]);

  canvas.fill(stroke(lip, 5), livery.ink);

  for (final whisker in const [Vec(396, 290), Vec(412, 297), Vec(398, 305)]) {
    canvas.fill(circle(whisker, 3).shape, livery.ink);
  }

  if (hound.eyepatch) {
    canvas.clip(_head.shape, () {
      canvas.fill(
        stroke(const [Vec(352, 150), Vec(300, 222), Vec(150, 300)], 8),
        livery.ink,
      );
    });
    canvas.fill(ellipse(_eye, 28, 26, tilt: -0.3).shape, livery.ink);

    return;
  }

  final brow = curve([
    _eye + const Vec(-24, -24),
    _eye + const Vec(-2, -32),
    _eye + const Vec(22, -27),
  ]);

  // A pale eye-white keeps the eye readable on the darkest coats.
  canvas.fill(stroke(brow, 6), livery.fur.shade(0.45));
  canvas.fill(ellipse(_eye, 19, 17).shape, _white);
  canvas.fill(circle(_eye + const Vec(3, 1), 13).shape, livery.ink);
  canvas.fill(circle(_eye + const Vec(-2, -4), 4.5).shape, _white);
}

void _paintEar(Canvas canvas, _Hound hound) {
  final ear = _earOutline(hound.ear);

  canvas.fill(ear.shape, hound.livery.ear.shade(0.22));

  canvas.clip(ear.shape, () {
    canvas.fill(ear.shifted(-9, -6).shape, hound.livery.ear);
  });
}

void _paintHeadwear(Canvas canvas, _Hound hound) {
  switch (hound.headwear) {
    case _Headwear.none:
      break;

    case _Headwear.gobCap:
      _paintGobCap(canvas);

    case _Headwear.bachi:
      _paintBachi(canvas);

    case _Headwear.captain:
      _paintCaptainCap(canvas);

    case _Headwear.souwester:
      _paintSouwester(canvas);

    case _Headwear.coral:
      _paintCoralCrown(canvas);
  }
}

/// Tips a hat drawn upright on the crown back along the skull.
Shape _worn(Path path) => path.rotatedAbout(_crown, _hatTilt).shape;

/// The white cap with an upturned brim that sailors wear.
void _paintGobCap(Canvas canvas) {
  canvas.fill(
    _worn(roundedRect(_crown.x - 76, _crown.y - 88, 152, 84, 38)),
    _white,
  );
  canvas.fill(
    _worn(roundedRect(_crown.x - 98, _crown.y - 32, 196, 46, 18)),
    _white.shade(0.12),
  );
  canvas.fill(_worn(rect(_crown.x - 84, _crown.y - 14, 168, 9)), _navy);
}

/// A flat cap with a red pompom on top.
void _paintBachi(Canvas canvas) {
  canvas.fill(_worn(ellipse(_crown + const Vec(0, -42), 112, 32)), _white);
  canvas.fill(
    _worn(roundedRect(_crown.x - 90, _crown.y - 36, 180, 42, 10)),
    _navy,
  );
  canvas.fill(_worn(circle(_crown + const Vec(0, -72), 19)), _red);
}

/// A peaked cap with a badge, which marks an uncommon hound.
void _paintCaptainCap(Canvas canvas) {
  final peak = blob([
    _crown + const Vec(40, 2),
    _crown + const Vec(96, -4),
    _crown + const Vec(150, 24),
    _crown + const Vec(110, 30),
    _crown + const Vec(60, 16),
  ]);

  canvas.fill(_worn(ellipse(_crown + const Vec(4, -46), 116, 38)), _white);
  canvas.fill(
    _worn(roundedRect(_crown.x - 88, _crown.y - 38, 184, 42, 10)),
    _navy,
  );
  canvas.fill(_worn(peak), _navy.shade(0.4));
  canvas.fill(_worn(circle(_crown + const Vec(72, -18), 13)), _gold);
}

/// A yellow oilskin hat, short at the front and long down the back, which
/// marks an uncommon hound.
void _paintSouwester(Canvas canvas) {
  final dome = [
    for (final point in arc(Vec.zero, 1, math.pi, math.pi * 2))
      _crown + Vec(point.x * 88, 6 + point.y * 92),
  ];
  final brim = curve([
    _crown + const Vec(132, 22),
    _crown + const Vec(70, -4),
    _crown + const Vec(-30, 2),
    _crown + const Vec(-112, 40),
    _crown + const Vec(-160, 104),
  ]);

  canvas.fill(_worn(dome), _oilskin);
  canvas.fill(
    stroke(arc(_crown + const Vec(0, 6), 60, math.pi * 1.2, math.pi * 1.45), 8),
    _oilskin.tint(0.5),
  );
  canvas.fill(
    ribbon(brim, (t) => 18 + 22 * math.sin(math.pi * t)).shape,
    _oilskin.shade(0.16),
  );
}

/// Branches of coral growing from the crown, which only the epic hound has.
void _paintCoralCrown(Canvas canvas) {
  const branches = [
    [Vec(0, 4), Vec(-6, -40), Vec(-2, -92)],
    [Vec(-4, -44), Vec(-34, -66), Vec(-40, -100)],
    [Vec(-2, -50), Vec(26, -72), Vec(30, -106)],
    [Vec(-50, 14), Vec(-70, -24), Vec(-96, -52)],
    [Vec(-66, -16), Vec(-64, -50), Vec(-72, -72)],
    [Vec(50, 2), Vec(70, -30), Vec(96, -60)],
    [Vec(66, -24), Vec(62, -56), Vec(70, -84)],
  ];

  for (final branch in branches) {
    final spine = curve([for (final offset in branch) _crown + offset]);

    canvas.fill(stroke(spine, 15), _coral);
    canvas.fill(circle(spine.last, 10).shape, _coral.tint(0.3));
  }
}

/// A bubble pipe hanging from the corner of the mouth.
void _paintPipe(Canvas canvas, _Hound hound) {
  if (!hound.pipe) {
    return;
  }

  const bowl = Vec(446, 358);

  canvas.fill(
    stroke(const [Vec(388, 324), Vec(432, 368)], 10),
    _briar.shade(0.35),
  );
  canvas.fill(roundedRect(bowl.x - 20, bowl.y - 26, 40, 50, 12).shape, _briar);
  canvas.fill(
    roundedRect(bowl.x - 24, bowl.y - 30, 48, 12, 6).shape,
    _briar.tint(0.3),
  );

  const bubbles = [
    (offset: Vec(24, -54), radius: 8.0),
    (offset: Vec(36, -94), radius: 11.0),
    (offset: Vec(30, -140), radius: 14.0),
  ];

  for (final bubble in bubbles) {
    _paintBubble(
      canvas,
      bowl + bubble.offset,
      bubble.radius,
      hound.livery.foam,
    );
  }
}

/// A last row of waves in front of the neck, so the hound sits in the water
/// rather than on top of it.
void _paintSurf(Canvas canvas, _Hound hound) {
  if (hound.sea == _Sea.bubbles || hound.sea == _Sea.abyss) {
    return;
  }

  _paintWaveRow(canvas, 468, 44, hound.livery.seaDeep.shade(0.2));
}
