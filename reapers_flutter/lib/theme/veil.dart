import 'package:flutter/material.dart';

/// The look of everything that is not 3D: a funeral ledger, set in bone on
/// tinted black, ruled with hairlines.
///
/// There is one colour, and it is not chosen here. Each reaper burns with a
/// single light, and the interface borrows it: a harvester's screens are
/// lit bone-white, a ferryman's amber, a chooser's red. Pass that light to
/// [Veil.theme].
abstract final class Veil {
  /// The page: a blue-black, never a neutral one.
  static const ink = Color(0xFF07080B);

  /// A surface raised off the page.
  static const inkRaised = Color(0xFF0F1116);

  /// Text and rules.
  static const bone = Color(0xFFE9E4D4);

  /// Text that gives way to other text.
  static const ash = Color(0xFF8E897D);

  /// A rule or border that is barely there.
  static const hairline = Color(0x2EE9E4D4);

  /// A loss, a refusal, a flinch.
  static const wound = Color(0xFFD4553F);

  /// The light of a reaper nobody has bound yet.
  static const unbound = Color(0xFFBFC8CF);

  static const displayFamily = 'Fraunces';
  static const bodyFamily = 'DM Sans';

  /// The widest a column of content grows, so screens hold their shape on a
  /// tablet.
  static const contentWidth = 520.0;

  /// The theme for a player whose reaper burns with [light].
  static ThemeData theme(Color light) {
    final scheme = ColorScheme.dark(
      primary: light,
      onPrimary: ink,
      secondary: light,
      onSecondary: ink,
      surface: ink,
      onSurface: bone,
      surfaceContainerHighest: inkRaised,
      outline: hairline,
      outlineVariant: hairline,
      error: wound,
      onError: ink,
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: scheme,
      scaffoldBackgroundColor: ink,
      fontFamily: bodyFamily,
      splashFactory: NoSplash.splashFactory,
      highlightColor: Colors.transparent,
      textTheme: _text,
      dividerTheme: const DividerThemeData(color: hairline, thickness: 1),
      pageTransitionsTheme: const PageTransitionsTheme(
        builders: {
          TargetPlatform.iOS: FadeForwardsPageTransitionsBuilder(),
          TargetPlatform.android: FadeForwardsPageTransitionsBuilder(),
          TargetPlatform.macOS: FadeForwardsPageTransitionsBuilder(),
        },
      ),
    );
  }

  /// Large words: names, titles, numerals. Fraunces at its sharpest cut.
  static TextStyle display(
    double size, {
    Color color = bone,
    double weight = 420,
  }) => TextStyle(
    fontFamily: displayFamily,
    fontSize: size,
    height: 0.98,
    letterSpacing: -size * 0.022,
    color: color,
    fontVariations: [
      FontVariation('wght', weight),
      FontVariation('opsz', size.clamp(9, 144)),
      const FontVariation('SOFT', 0),
      const FontVariation('WONK', 1),
    ],
  );

  /// Small capitals in the margin: labels, units, column heads.
  static TextStyle label({Color color = ash, double size = 11}) => TextStyle(
    fontFamily: bodyFamily,
    fontSize: size,
    height: 1.2,
    letterSpacing: size * 0.22,
    color: color,
    fontVariations: const [FontVariation('wght', 600)],
  );

  /// Running text.
  static TextStyle body({Color color = bone, double size = 15}) => TextStyle(
    fontFamily: bodyFamily,
    fontSize: size,
    height: 1.42,
    color: color,
    fontVariations: const [FontVariation('wght', 400)],
  );

  /// A figure in a table: body face, every digit the same width.
  static TextStyle figure({Color color = bone, double size = 15}) => TextStyle(
    fontFamily: bodyFamily,
    fontSize: size,
    height: 1.2,
    color: color,
    fontFeatures: const [FontFeature.tabularFigures()],
    fontVariations: const [FontVariation('wght', 500)],
  );

  static final _text = TextTheme(
    displayLarge: display(64),
    displayMedium: display(44),
    displaySmall: display(32),
    headlineMedium: display(26),
    titleLarge: display(22),
    bodyLarge: body(size: 16),
    bodyMedium: body(),
    bodySmall: body(size: 13, color: ash),
    labelLarge: label(color: bone, size: 12),
    labelMedium: label(),
    labelSmall: label(size: 10),
  );
}

/// The numerals a ledger counts in.
String romanNumeral(int value) {
  if (value <= 0) {
    return 'N';
  }

  const symbols = [
    (1000, 'M'),
    (900, 'CM'),
    (500, 'D'),
    (400, 'CD'),
    (100, 'C'),
    (90, 'XC'),
    (50, 'L'),
    (40, 'XL'),
    (10, 'X'),
    (9, 'IX'),
    (5, 'V'),
    (4, 'IV'),
    (1, 'I'),
  ];
  final buffer = StringBuffer();
  var remaining = value;

  for (final (amount, symbol) in symbols) {
    while (remaining >= amount) {
      buffer.write(symbol);
      remaining -= amount;
    }
  }

  return buffer.toString();
}
