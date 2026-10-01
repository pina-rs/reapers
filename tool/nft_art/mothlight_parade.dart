/// Mothlight Parade: moths seen from above with their wings spread, perfectly
/// symmetric, against a glow in the dark.
library;

import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'canvas.dart';
import 'geometry.dart';
import 'palette.dart';

/// Paints the moth for [nft].
void paintMoth(Canvas canvas, Nft nft) {
  final moth = _Moth.of(nft);

  _paintNight(canvas, moth);
  _paintWingPair(canvas, moth, moth.hindwingShape, moth.scales.hindwing);
  _paintWingPair(canvas, moth, moth.forewingShape, moth.scales.forewing);
  _paintAntennae(canvas, moth);
  _paintBody(canvas, moth);
}

/// The middle of the moth, which the glow is centred on.
const _heart = Vec(centreX, 252);

enum _Forewing { broad, swept, hooked }

enum _Hindwing { round, scalloped, tailed }

enum _Marking { eyespots, bands, veins }

enum _Antennae { thread, comb, feathered }

enum _Abdomen { striped, dotted, tipped }

enum _Sky { glow, bloodMoon, aurora }

/// The colours of one moth.
final class _Scales {
  const _Scales({
    required this.night,
    required this.glow,
    required this.forewing,
    required this.hindwing,
    required this.marking,
    required this.accent,
    required this.body,
  });

  final Rgb night;
  final Rgb glow;
  final Rgb forewing;
  final Rgb hindwing;

  /// The darker of the two pattern colours: margins, veins and pupils.
  final Rgb marking;

  /// The brighter pattern colour: bands and the rings of eyespots.
  final Rgb accent;
  final Rgb body;
}

/// The everyday colourways, cycled through by serial.
const _colourways = [
  _Scales(
    night: Rgb(0x15132E),
    glow: Rgb(0x6A58D8),
    forewing: Rgb(0xE9A84C),
    hindwing: Rgb(0xCB5A3F),
    marking: Rgb(0x2B1B3D),
    accent: Rgb(0xF7E9C6),
    body: Rgb(0xF7E9C6),
  ),
  _Scales(
    night: Rgb(0x0E1D2A),
    glow: Rgb(0x2596A0),
    forewing: Rgb(0xA9CB8A),
    hindwing: Rgb(0xEB8FA2),
    marking: Rgb(0x1D2F3A),
    accent: Rgb(0xF7F1D5),
    body: Rgb(0xF7F1D5),
  ),
  _Scales(
    night: Rgb(0x1E1028),
    glow: Rgb(0xA63A7A),
    forewing: Rgb(0xF0E4CB),
    hindwing: Rgb(0x7396C2),
    marking: Rgb(0x2A1B3D),
    accent: Rgb(0xD2553B),
    body: Rgb(0xF0E4CB),
  ),
  _Scales(
    night: Rgb(0x0F1C17),
    glow: Rgb(0x3A9460),
    forewing: Rgb(0xD9764B),
    hindwing: Rgb(0xE8BF52),
    marking: Rgb(0x1F3327),
    accent: Rgb(0xF6EBCB),
    body: Rgb(0xF6EBCB),
  ),
  _Scales(
    night: Rgb(0x101A33),
    glow: Rgb(0x3B86C8),
    forewing: Rgb(0x9FE3BE),
    hindwing: Rgb(0x6FC7A6),
    marking: Rgb(0x8F3F74),
    accent: Rgb(0xFFF3D6),
    body: Rgb(0xFFF3D6),
  ),
  _Scales(
    night: Rgb(0x1A1230),
    glow: Rgb(0x8250C8),
    forewing: Rgb(0xC9B6F2),
    hindwing: Rgb(0xF2B950),
    marking: Rgb(0x2A1B4D),
    accent: Rgb(0xFFF3D6),
    body: Rgb(0xFFF3D6),
  ),
];

/// Bone and blood against a red moon, kept for rare moths.
const _ghost = _Scales(
  night: Rgb(0x2A0D1C),
  glow: Rgb(0xB3273A),
  forewing: Rgb(0xF3EDE0),
  hindwing: Rgb(0xDCD3C2),
  marking: Rgb(0x1A0A12),
  accent: Rgb(0xD4313F),
  body: Rgb(0xF3EDE0),
);

/// Gold over wings that are shaded rather than flat, kept for the epic moth.
const _opal = _Scales(
  night: Rgb(0x08091C),
  glow: Rgb(0x4534A6),
  forewing: Rgb(0x59E0D0),
  hindwing: Rgb(0xC77DF2),
  marking: Rgb(0x14102E),
  accent: Rgb(0xF5CC5B),
  body: Rgb(0xF5CC5B),
);

/// The traits of one moth, chosen from its serial and rarity.
final class _Moth {
  const _Moth({
    required this.scales,
    required this.forewing,
    required this.hindwing,
    required this.marking,
    required this.antennae,
    required this.abdomen,
    required this.sky,
    required this.iridescent,
    required this.random,
  });

  factory _Moth.of(Nft nft) {
    final index = nft.serial - 1;
    final rarity = nft.rarity;
    final exalted = rarity.index >= Rarity.epic.index;

    return _Moth(
      scales: switch (rarity) {
        Rarity.rare => _ghost,
        Rarity.epic => _opal,
        Rarity.common ||
        Rarity.uncommon => _colourways[index % _colourways.length],
      },
      forewing: _Forewing.values[index % _Forewing.values.length],
      // Tails and feathered antennae are held back from common moths, so each
      // is a trait an uncommon moth can be told by.
      hindwing: switch (rarity) {
        Rarity.uncommon when index.isEven => _Hindwing.tailed,
        Rarity.epic => _Hindwing.tailed,
        _ => index.isEven ? _Hindwing.round : _Hindwing.scalloped,
      },
      marking: _Marking.values[(index + index ~/ 3) % _Marking.values.length],
      antennae: switch (rarity) {
        Rarity.common => index.isEven ? _Antennae.thread : _Antennae.comb,
        Rarity.uncommon when index.isEven => _Antennae.thread,
        _ => _Antennae.feathered,
      },
      abdomen: _Abdomen.values[(index + 1) ~/ 2 % _Abdomen.values.length],
      sky: switch (rarity) {
        Rarity.rare => _Sky.bloodMoon,
        Rarity.epic => _Sky.aurora,
        Rarity.common || Rarity.uncommon => _Sky.glow,
      },
      iridescent: exalted,
      random: SeededRandom(hashText('art:${nft.id}')),
    );
  }

  final _Scales scales;
  final _Forewing forewing;
  final _Hindwing hindwing;
  final _Marking marking;
  final _Antennae antennae;
  final _Abdomen abdomen;
  final _Sky sky;

  /// Whether the wings shift colour across their span, which only the epic
  /// moth's do.
  final bool iridescent;
  final SeededRandom random;

  _Wing get forewingShape => switch (forewing) {
    _Forewing.broad => _broadForewing,
    _Forewing.swept => _sweptForewing,
    _Forewing.hooked => _hookedForewing,
  };

  _Wing get hindwingShape => switch (hindwing) {
    _Hindwing.round => _roundHindwing,
    _Hindwing.scalloped => _scallopedHindwing,
    _Hindwing.tailed => _tailedHindwing,
  };
}

/// The right-hand wing of a pair. The left is always its mirror image.
final class _Wing {
  const _Wing({
    required this.outline,
    required this.root,
    required this.eyespot,
    required this.eyespotRadius,
    required this.bands,
    required this.veinFrom,
    required this.veinTo,
  });

  final Path outline;

  /// Where the wing joins the body: bands circle it and veins leave from it.
  final Vec root;
  final Vec eyespot;
  final double eyespotRadius;

  /// The distance from the root to the middle of each band.
  final List<double> bands;

  /// The fan of angles, in radians, that the veins spread across.
  final double veinFrom;
  final double veinTo;
}

const _forewingRoot = Vec(266, 230);
const _hindwingRoot = Vec(266, 252);

final _broadForewing = _Wing(
  outline: blob(const [
    Vec(266, 204),
    Vec(312, 146),
    Vec(376, 108),
    Vec(440, 104),
    Vec(474, 136),
    Vec(470, 194),
    Vec(436, 244),
    Vec(372, 272),
    Vec(302, 270),
  ]),
  root: _forewingRoot,
  eyespot: const Vec(398, 184),
  eyespotRadius: 34,
  bands: const [96, 150],
  veinFrom: -1.05,
  veinTo: 0.3,
);

final _sweptForewing = _Wing(
  outline: _sweptForewingOutline(),
  root: _forewingRoot,
  eyespot: const Vec(388, 190),
  eyespotRadius: 30,
  bands: const [96, 150],
  veinFrom: -0.85,
  veinTo: 0.28,
);

final _hookedForewing = _Wing(
  outline: _hookedForewingOutline(),
  root: _forewingRoot,
  eyespot: const Vec(390, 192),
  eyespotRadius: 32,
  bands: const [96, 150],
  veinFrom: -1.0,
  veinTo: 0.3,
);

const _hindwingKnots = [
  Vec(264, 244),
  Vec(330, 256),
  Vec(400, 280),
  Vec(430, 326),
  Vec(414, 384),
  Vec(364, 424),
  Vec(306, 414),
  Vec(272, 360),
];

final _roundHindwing = _Wing(
  outline: blob(_hindwingKnots),
  root: _hindwingRoot,
  eyespot: const Vec(354, 336),
  eyespotRadius: 36,
  bands: const [84, 132],
  veinFrom: 0.3,
  veinTo: 1.45,
);

final _scallopedHindwing = _Wing(
  outline: _scalloped(blob(_hindwingKnots, steps: 24)),
  root: _hindwingRoot,
  eyespot: const Vec(354, 336),
  eyespotRadius: 36,
  bands: const [84, 132],
  veinFrom: 0.3,
  veinTo: 1.45,
);

final _tailedHindwing = _Wing(
  outline: _tailedHindwingOutline(),
  root: _hindwingRoot,
  eyespot: const Vec(356, 322),
  eyespotRadius: 32,
  bands: const [84, 132],
  veinFrom: 0.3,
  veinTo: 1.3,
);

/// A long triangular forewing with a sharp tip, like a hawk-moth's.
Path _sweptForewingOutline() {
  final builder = PathBuilder(const Vec(264, 206))
    ..cubicTo(const Vec(312, 156), const Vec(388, 112), const Vec(474, 112))
    ..cubicTo(const Vec(474, 170), const Vec(438, 226), const Vec(384, 258))
    ..cubicTo(const Vec(346, 276), const Vec(300, 272), const Vec(266, 252));

  return builder.build();
}

/// A forewing whose tip curls back into a hook.
Path _hookedForewingOutline() {
  final builder = PathBuilder(const Vec(264, 206))
    ..cubicTo(const Vec(304, 144), const Vec(372, 104), const Vec(440, 98))
    ..cubicTo(const Vec(466, 94), const Vec(482, 108), const Vec(474, 130))
    ..cubicTo(const Vec(456, 144), const Vec(460, 170), const Vec(462, 192))
    ..cubicTo(const Vec(452, 236), const Vec(404, 268), const Vec(336, 276))
    ..cubicTo(const Vec(304, 276), const Vec(282, 266), const Vec(266, 252));

  return builder.build();
}

/// A hindwing drawn out into a long tail.
Path _tailedHindwingOutline() {
  final builder = PathBuilder(const Vec(264, 244))
    ..cubicTo(const Vec(330, 246), const Vec(410, 266), const Vec(430, 318))
    ..cubicTo(const Vec(440, 364), const Vec(408, 392), const Vec(382, 404))
    ..cubicTo(const Vec(372, 436), const Vec(366, 466), const Vec(344, 492))
    ..cubicTo(const Vec(330, 468), const Vec(326, 440), const Vec(316, 416))
    ..cubicTo(const Vec(290, 400), const Vec(270, 360), const Vec(264, 300));

  return builder.build();
}

/// Pushes the outer margin of [outline] out into a row of rounded lobes.
///
/// The lobes fade in from the body side, so the wing root stays smooth where
/// it tucks under the abdomen.
Path _scalloped(Path outline) {
  final count = outline.length;

  return [
    for (var index = 0; index < count; index++)
      outline[index] +
          _outward(outline, index) * (_lobeHeight(index / count) * 13),
  ];
}

/// The unit normal pointing out of a clockwise [outline] at [index].
Vec _outward(Path outline, int index) {
  final count = outline.length;
  final tangent =
      outline[(index + 1) % count] - outline[(index - 1 + count) % count];

  return Vec(tangent.y, -tangent.x) * (1 / tangent.length);
}

/// How far a scallop bulges, from 0 to 1, at [along] of the way round a wing.
double _lobeHeight(double along) {
  const lobes = 5;
  const from = 0.28;
  const to = 0.86;

  if (along < from || along > to) {
    return 0;
  }

  return math.sin((along - from) / (to - from) * math.pi * lobes).abs();
}

void _paintNight(Canvas canvas, _Moth moth) {
  final scales = moth.scales;

  canvas.shade(rect(0, 0, designSize, designSize).shape, (x, y) {
    final dx = x - _heart.x;
    final dy = y - _heart.y;
    final nearness = (1 - math.sqrt(dx * dx + dy * dy) / 262).clamp(0.0, 1.0);

    return scales.night.mix(
      scales.glow,
      nearness * nearness * (3 - 2 * nearness),
    );
  });

  switch (moth.sky) {
    case _Sky.glow:
      break;

    case _Sky.bloodMoon:
      // The one hard-edged disc in the collection, where every other moth
      // has only a soft glow.
      canvas.fill(circle(_heart, 204).shape, scales.glow);
      canvas.fill(ring(_heart, 204, 196), scales.glow.tint(0.22));
      _paintStars(canvas, moth, count: 12);

    case _Sky.aurora:
      canvas.fill(ring(_heart, 236, 229), scales.accent);
      canvas.fill(ring(_heart, 218, 215), scales.accent, opacity: 0.6);
      _paintStars(canvas, moth, count: 16);
  }
}

/// Four-pointed stars in the corners, clear of the wings.
void _paintStars(Canvas canvas, _Moth moth, {required int count}) {
  final random = moth.random;

  for (var star = 0; star < count; star++) {
    final at =
        _heart +
        Vec.polar(random.nextRange(244, 330), random.nextRange(0, math.pi * 2));
    final size = random.nextRange(6, 13);

    canvas.fill(burst(at, size, size * 0.3, 4).shape, moth.scales.accent);
  }
}

void _paintWingPair(Canvas canvas, _Moth moth, _Wing wing, Rgb base) {
  final scales = moth.scales;
  final pair = wing.outline.shape.withMirror;

  // A sliver of shadow below each pair lifts the forewings off the hindwings
  // and the hindwings off the glow.
  canvas.fill(
    wing.outline.shifted(0, 7).shape.withMirror,
    scales.night,
    opacity: 0.55,
  );

  if (moth.iridescent) {
    canvas.shade(pair, (x, y) => _iridescence(x, y, wing.root));
  } else {
    canvas.fill(pair, base);
  }

  canvas.clip(pair, () {
    _paintMarking(canvas, moth, wing);
    canvas.fill(outlineStroke(wing.outline, 24).withMirror, scales.marking);
  });
}

/// A hue that drifts from sea-green at the body to pink at the wing tips.
Rgb _iridescence(double x, double y, Vec root) {
  final dx = (x - centreX).abs() - (root.x - centreX);
  final dy = y - root.y;
  final span = (math.sqrt(dx * dx + dy * dy) / 230).clamp(0.0, 1.0);

  return Rgb.hsv(0.46 + span * 0.46, 0.5, 1);
}

void _paintMarking(Canvas canvas, _Moth moth, _Wing wing) {
  final scales = moth.scales;

  switch (moth.marking) {
    case _Marking.eyespots:
      _paintEyespot(canvas, moth, wing);

    case _Marking.bands:
      for (final (index, distance) in wing.bands.indexed) {
        final width = index.isEven ? 22.0 : 12.0;

        canvas.fill(
          ring(
            wing.root,
            distance + width / 2,
            distance - width / 2,
          ).withMirror,
          index.isEven ? scales.accent : scales.marking,
        );
      }

    case _Marking.veins:
      const veins = 6;

      for (var vein = 0; vein < veins; vein++) {
        final angle =
            wing.veinFrom + (wing.veinTo - wing.veinFrom) * vein / (veins - 1);

        canvas.fill(
          stroke([wing.root, wing.root + Vec.polar(320, angle)], 6).withMirror,
          scales.marking,
        );
      }
  }

  // The exalted moths wear every marking at once.
  if (moth.iridescent && moth.marking != _Marking.eyespots) {
    _paintEyespot(canvas, moth, wing);
  }
}

void _paintEyespot(Canvas canvas, _Moth moth, _Wing wing) {
  final scales = moth.scales;
  final radius = wing.eyespotRadius;
  final glint = wing.eyespot + Vec(-radius * 0.16, -radius * 0.18);

  canvas.fill(circle(wing.eyespot, radius).shape.withMirror, scales.marking);
  canvas.fill(
    circle(wing.eyespot, radius * 0.74).shape.withMirror,
    scales.accent,
  );
  canvas.fill(
    circle(wing.eyespot, radius * 0.44).shape.withMirror,
    scales.marking,
  );
  canvas.fill(circle(glint, radius * 0.13).shape.withMirror, scales.accent);
}

void _paintAntennae(Canvas canvas, _Moth moth) {
  final scales = moth.scales;
  final spine = curve(const [Vec(263, 166), Vec(286, 122), Vec(324, 92)]);

  switch (moth.antennae) {
    case _Antennae.thread:
      canvas.fill(stroke(spine, 6).withMirror, scales.body);

    case _Antennae.comb:
      canvas.fill(stroke(spine, 6).withMirror, scales.body);

      for (var tooth = 3; tooth < spine.length; tooth += 4) {
        final along = spine[tooth] - spine[tooth - 1];
        final outward = Vec(along.y, -along.x) * (15 / along.length);

        canvas.fill(
          stroke([spine[tooth], spine[tooth] + outward], 5).withMirror,
          scales.body,
        );
      }

    case _Antennae.feathered:
      final plume = curve(const [Vec(263, 166), Vec(296, 116), Vec(348, 78)]);

      canvas.fill(
        ribbon(plume, (t) => 4 + 30 * math.sin(math.pi * t)).shape.withMirror,
        scales.body,
      );
      canvas.fill(stroke(plume, 4).withMirror, scales.marking);
  }
}

void _paintBody(Canvas canvas, _Moth moth) {
  final scales = moth.scales;
  final abdomen = ellipse(const Vec(centreX, 314), 27, 78);

  canvas.fill(abdomen.shape, scales.body);

  canvas.clip(abdomen.shape, () {
    switch (moth.abdomen) {
      case _Abdomen.striped:
        for (var y = 290.0; y < 400; y += 25) {
          canvas.fill(rect(0, y, designSize, 10).shape, scales.marking);
        }

      case _Abdomen.dotted:
        for (var y = 290.0; y < 380; y += 27) {
          canvas.fill(circle(Vec(centreX, y), 9).shape, scales.marking);
        }

      case _Abdomen.tipped:
        canvas.fill(rect(0, 296, designSize, 9).shape, scales.marking);
        canvas.fill(rect(0, 334, designSize, 80).shape, scales.marking);
    }
  });

  canvas.fill(_thorax(), scales.body);
  canvas.fill(ellipse(const Vec(centreX, 194), 30, 15).shape, scales.marking);
  canvas.fill(circle(const Vec(centreX, 174), 18).shape, scales.body);

  for (final side in [-1, 1]) {
    canvas.fill(
      circle(Vec(centreX + side * 14, 170), 7.5).shape,
      scales.marking,
    );
  }
}

/// A thorax with a tufted edge. The fur is what tells a moth from a
/// butterfly at a glance.
Shape _thorax() {
  const centre = Vec(centreX, 228);
  const tufts = 14;

  return Shape([
    ellipse(centre, 34, 46),
    for (var tuft = 0; tuft < tufts; tuft++)
      circle(
        Vec(
          centre.x + 34 * math.cos(tuft * math.pi * 2 / tufts),
          centre.y + 46 * math.sin(tuft * math.pi * 2 / tufts),
        ),
        9,
      ),
  ]);
}
