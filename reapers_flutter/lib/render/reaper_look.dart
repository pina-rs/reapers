import 'dart:ui';

import 'package:reapers_shared/reapers_shared.dart';

/// How one named surface of a reaper is drawn.
///
/// The names are the material names the art build gives each part: `cloth`,
/// `bone`, `metal` and so on. A look turns a name into numbers for the
/// reaper shader (assets/materials/reaper.fmat).
final class SurfaceLook {
  const SurfaceLook({
    required this.tint,
    this.roughness = 0.9,
    this.metallic = 0,
    this.grime = 0.85,
    this.wear = 0.5,
    this.weave = 0.6,
    this.burns = true,
    this.glowStrength = 0,
  });

  final Color tint;

  final double roughness;

  final double metallic;

  /// How strongly painted grime and occlusion darken the surface.
  final double grime;

  /// How brightly raised edges catch the light.
  final double wear;

  /// How much fine grain breaks up the surface.
  final double weave;

  /// Whether the outfit's char and hollowing reach this surface. Bone and
  /// the reaper's light do not burn.
  final bool burns;

  /// How strongly the surface lights itself with the accent colour.
  final double glowStrength;
}

/// One colourway of a class: what the cloth is dyed and what colour the
/// reaper's single light burns.
final class Colourway {
  const Colourway({
    required this.cloth,
    required this.trim,
    required this.accent,
    required this.rim,
  });

  /// The dominant cloth, a tinted black. Never a neutral one.
  final Color cloth;

  /// The second cloth: a mantle, a hat, a wrap.
  final Color trim;

  /// The only saturated colour on the figure.
  final Color accent;

  /// The light that edges the figure from behind, so it separates from
  /// whatever the camera sees.
  final Color rim;
}

/// Everything needed to draw one reaper: a look for every surface, and how
/// far its outfit has burned and hollowed.
final class ReaperLook {
  const ReaperLook({
    required this.surfaces,
    required this.accent,
    required this.rim,
    required this.rimStrength,
    required this.ember,
    required this.emberStrength,
    required this.charLevel,
    required this.hollowLevel,
    required this.ink,
    required this.inkStrength,
    required this.inkSeed,
    required this.headless,
  });

  /// Resolves the look of [genome] wearing [outfit].
  factory ReaperLook.of(ReaperGenome genome, Outfit outfit) {
    final colourway = _omened(
      classColourways[genome.reaperClass]![genome.palette],
      genome.omen,
    );
    final state = _OutfitState.of(outfit);
    final cloth = _dye(colourway.cloth, state);
    final trim = _dye(colourway.trim, state);
    final gilded = genome.omen == Omen.gilded;
    final gaze = 0.7 + 0.6 * ReaperGenome.fraction(genome.gaze);
    final glow = state.glow * gaze * (genome.omen == Omen.eclipsed ? 1.6 : 1);

    return ReaperLook(
      accent: colourway.accent,
      rim: colourway.rim,
      rimStrength: state.rim,
      ember: const Color(0xFFFF6A1F),
      emberStrength: state.ember,
      charLevel: state.char,
      hollowLevel: state.hollow,
      ink: _lerp(
        _lerp(colourway.rim, const Color(0xFFE9E4D4), 0.55),
        colourway.accent,
        0.15,
      ),
      inkStrength: 1,
      inkSeed: _inkSeed(genome),
      headless: outfit == Outfit.hollowed,
      surfaces: {
        'cloth': SurfaceLook(tint: cloth, roughness: 0.95, weave: 0.8),
        'cloth_alt': SurfaceLook(tint: trim, roughness: 0.92, weave: 0.75),
        'feather': SurfaceLook(
          tint: _scale(cloth, 0.7),
          roughness: 0.55,
          wear: 0.8,
          weave: 0.4,
        ),
        'paper': SurfaceLook(
          tint: _lerp(trim, const Color(0xFFB8A888), 0.5),
          roughness: 0.8,
          weave: 0.5,
        ),
        'rope': SurfaceLook(
          tint: _lerp(trim, const Color(0xFF6E5B3A), 0.6),
          roughness: 0.95,
          wear: 0.7,
          weave: 0.9,
        ),
        'wood': const SurfaceLook(
          tint: Color(0xFF2A1D14),
          roughness: 0.8,
          wear: 0.6,
          weave: 0.5,
        ),
        'bone': SurfaceLook(
          tint: genome.omen == Omen.eclipsed
              ? const Color(0xFF4A4640)
              : const Color(0xFFB9AE95),
          roughness: 0.7,
          grime: 1,
          wear: 0.35,
          weave: 0.35,
          burns: false,
        ),
        'metal': SurfaceLook(
          tint: gilded ? const Color(0xFFB08A3C) : const Color(0xFF4A4C52),
          roughness: gilded ? 0.38 : 0.5,
          metallic: 1,
          wear: 0.9,
          weave: 0.25,
        ),
        'gold': const SurfaceLook(
          tint: Color(0xFFB08A3C),
          roughness: 0.4,
          metallic: 1,
          wear: 0.9,
          weave: 0.2,
        ),
        'accent': SurfaceLook(
          tint: const Color(0xFF000000),
          grime: 0,
          wear: 0,
          weave: 0,
          burns: false,
          glowStrength: glow,
        ),
        'spirit': SurfaceLook(
          tint: _lerp(const Color(0xFF111B24), colourway.accent, 0.15),
          roughness: 0.65,
          grime: 0,
          wear: 0,
          weave: 0,
          burns: false,
          glowStrength: glow * 0.08,
        ),
        'void': const SurfaceLook(
          tint: Color(0xFF010102),
          roughness: 1,
          grime: 0,
          wear: 0,
          weave: 0,
          burns: false,
        ),
      },
    );
  }

  final Map<String, SurfaceLook> surfaces;

  final Color accent;

  final Color rim;

  final double rimStrength;

  /// The colour of the burning edge between cloth and char.
  final Color ember;

  final double emberStrength;

  /// How far char has climbed, from 0 (none) to 1.
  final double charLevel;

  /// How much of the figure has been eaten away, from 0 (none) to 1.
  final double hollowLevel;

  /// The line drawn around the figure.
  ///
  /// It is the reaper's own light rather than a plain black outline: on the
  /// veil, ink has to be brighter than the paper to read.
  final Color ink;

  /// How strongly the line is laid down.
  final double inkStrength;

  /// An offset into the line's noise, fixed by the genome, so every reaper is
  /// sketched by a slightly different hand. It never changes over time.
  final double inkSeed;

  /// Whether the head has gone, leaving only what the reaper wears on it.
  final bool headless;

  /// The look for a material the art build named [material], which may carry
  /// Blender's `.001` suffix.
  SurfaceLook surfaceFor(String material) =>
      surfaces[material.split('.').first] ?? surfaces['cloth']!;
}

/// The colour [genome]'s one light burns: its colourway's accent, as its omen
/// leaves it. The interface is lit with the same colour.
Color reaperLight(ReaperGenome genome) => _omened(
  classColourways[genome.reaperClass]![genome.palette],
  genome.omen,
).accent;

/// What an outfit does to every surface it reaches.
final class _OutfitState {
  const _OutfitState({
    required this.undyed,
    required this.shade,
    required this.char,
    required this.hollow,
    required this.ember,
    required this.glow,
    required this.rim,
  });

  factory _OutfitState.of(Outfit outfit) => switch (outfit) {
    Outfit.shrouded => const _OutfitState(
      undyed: 0.55,
      shade: 1,
      char: 0,
      hollow: 0,
      ember: 0,
      glow: 2.4,
      rim: 1.1,
    ),
    Outfit.sworn => const _OutfitState(
      undyed: 0,
      shade: 1,
      char: 0,
      hollow: 0,
      ember: 0,
      glow: 3.6,
      rim: 1.3,
    ),
    Outfit.scorched => const _OutfitState(
      undyed: 0,
      shade: 0.8,
      char: 0.5,
      hollow: 0,
      ember: 3.2,
      glow: 5.5,
      rim: 1.6,
    ),
    Outfit.hollowed => const _OutfitState(
      undyed: 0,
      shade: 0.6,
      char: 0.8,
      hollow: 0.42,
      ember: 2.2,
      glow: 9,
      rim: 1.9,
    ),
  };

  /// How far the cloth is pulled back toward raw, undyed cloth.
  final double undyed;

  /// Multiplier on the cloth's brightness.
  final double shade;

  final double char;

  final double hollow;

  final double ember;

  final double glow;

  final double rim;
}

/// Raw cloth: what a shrouded outfit looks like before any dye.
const _rawCloth = Color(0xFF5E5245);

/// A stable offset into the ink's noise field, from 0 to 8.
double _inkSeed(ReaperGenome genome) {
  var hash = 0x811c9dc5;

  for (final unit in genome.code.codeUnits) {
    hash = ((hash ^ unit) * 0x01000193) & 0xffffffff;
  }

  return (hash % 1024) / 128;
}

Color _dye(Color dyed, _OutfitState state) =>
    _scale(_lerp(dyed, _rawCloth, state.undyed), state.shade);

Colourway _omened(Colourway colourway, Omen omen) => switch (omen) {
  Omen.none => colourway,
  Omen.ashen => Colourway(
    cloth: _lerp(_grey(colourway.cloth), const Color(0xFF8A857C), 0.45),
    trim: _lerp(_grey(colourway.trim), const Color(0xFFB5AFA2), 0.5),
    accent: _lerp(colourway.accent, const Color(0xFFF2EEE4), 0.7),
    rim: const Color(0xFFBFC6CC),
  ),
  Omen.gilded => Colourway(
    cloth: colourway.cloth,
    trim: _lerp(colourway.trim, const Color(0xFF8A6A2A), 0.6),
    accent: const Color(0xFFFFC65A),
    rim: const Color(0xFFE0B870),
  ),
  Omen.eclipsed => Colourway(
    cloth: _scale(colourway.cloth, 0.3),
    trim: _scale(colourway.trim, 0.3),
    accent: colourway.accent,
    rim: colourway.accent,
  ),
};

Color _lerp(Color a, Color b, double t) => Color.lerp(a, b, t)!;

Color _scale(Color color, double scale) => Color.from(
  alpha: color.a,
  red: color.r * scale,
  green: color.g * scale,
  blue: color.b * scale,
);

Color _grey(Color color) {
  final luma = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;

  return Color.from(alpha: color.a, red: luma, green: luma, blue: luma);
}

/// The three colourways of each class.
///
/// Each class has one dominant hue and one accent; see
/// docs/research/dark-games-and-ar-hunting.md for where they come from.
const classColourways = {
  // Straw ochre and rust on brown-black; a bone-white light.
  ReaperClass.harvester: [
    Colourway(
      cloth: Color(0xFF2B2119),
      trim: Color(0xFF7A5A2E),
      accent: Color(0xFFF3EBD8),
      rim: Color(0xFFB9A98A),
    ),
    Colourway(
      cloth: Color(0xFF2C1913),
      trim: Color(0xFF6E3520),
      accent: Color(0xFFFFE9C9),
      rim: Color(0xFFC79A7A),
    ),
    Colourway(
      cloth: Color(0xFF1E211B),
      trim: Color(0xFF5E5A34),
      accent: Color(0xFFE6F0D8),
      rim: Color(0xFF9FAE8E),
    ),
  ],
  // Verdigris and wet slate on green-black; an amber lantern.
  ReaperClass.ferryman: [
    Colourway(
      cloth: Color(0xFF16221F),
      trim: Color(0xFF2F5A50),
      accent: Color(0xFFFFB347),
      rim: Color(0xFF6FA596),
    ),
    Colourway(
      cloth: Color(0xFF1A1F26),
      trim: Color(0xFF3A4A5C),
      accent: Color(0xFFFFC56B),
      rim: Color(0xFF7D93AD),
    ),
    Colourway(
      cloth: Color(0xFF1F2118),
      trim: Color(0xFF4B5230),
      accent: Color(0xFFFF9A3D),
      rim: Color(0xFF8F9A6A),
    ),
  ],
  // Tarnished gold on blue-black; one cold white light in the mask.
  ReaperClass.weigher: [
    Colourway(
      cloth: Color(0xFF141926),
      trim: Color(0xFF6B5524),
      accent: Color(0xFFE8F4FF),
      rim: Color(0xFF7F92C2),
    ),
    Colourway(
      cloth: Color(0xFF1B1626),
      trim: Color(0xFF6B4A24),
      accent: Color(0xFFF3ECFF),
      rim: Color(0xFF9A86C7),
    ),
    Colourway(
      cloth: Color(0xFF10201F),
      trim: Color(0xFF5E5A2A),
      accent: Color(0xFFDFFBF4),
      rim: Color(0xFF6FB0A6),
    ),
  ],
  // Raven blue-black with red thread; a red spear tip.
  ReaperClass.chooser: [
    Colourway(
      cloth: Color(0xFF12141C),
      trim: Color(0xFF4A1414),
      accent: Color(0xFFFF3B2F),
      rim: Color(0xFF6C7CA8),
    ),
    Colourway(
      cloth: Color(0xFF16121A),
      trim: Color(0xFF55182A),
      accent: Color(0xFFFF4D6A),
      rim: Color(0xFF8A72A6),
    ),
    Colourway(
      cloth: Color(0xFF111A1C),
      trim: Color(0xFF4F2212),
      accent: Color(0xFFFF6A2A),
      rim: Color(0xFF6C9CA6),
    ),
  ],
  // Gunmetal on red-black; ember orange inside the collar.
  ReaperClass.hollow: [
    Colourway(
      cloth: Color(0xFF211416),
      trim: Color(0xFF3B3E46),
      accent: Color(0xFFFF7A1A),
      rim: Color(0xFFA07A7A),
    ),
    Colourway(
      cloth: Color(0xFF1C1418),
      trim: Color(0xFF45404A),
      accent: Color(0xFFFF9B2E),
      rim: Color(0xFF9A7A92),
    ),
    Colourway(
      cloth: Color(0xFF231A12),
      trim: Color(0xFF3F4244),
      accent: Color(0xFFFF5A1F),
      rim: Color(0xFFA88E72),
    ),
  ],
  // Washed grey-green on grey-black; pale silver in the bell.
  ReaperClass.mourner: [
    Colourway(
      cloth: Color(0xFF1C211F),
      trim: Color(0xFF55645C),
      accent: Color(0xFFDDEBFF),
      rim: Color(0xFFA5B8B0),
    ),
    Colourway(
      cloth: Color(0xFF1E1E24),
      trim: Color(0xFF5C5E70),
      accent: Color(0xFFEDE6FF),
      rim: Color(0xFFAAA8C4),
    ),
    Colourway(
      cloth: Color(0xFF211F1B),
      trim: Color(0xFF66604E),
      accent: Color(0xFFF2F4E2),
      rim: Color(0xFFB8B29A),
    ),
  ],
};
