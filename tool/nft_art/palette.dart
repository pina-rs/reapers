/// Colours for the stand-in NFT artwork.
library;

/// An opaque colour, written as `0xRRGGBB`.
final class Rgb {
  const Rgb(this.hex);

  /// A colour from a hue in turns (0 to 1), a saturation and a brightness.
  ///
  /// Only the iridescent treatments need this: they sweep the hue across a
  /// shape, which cannot be written as a list of fixed swatches.
  factory Rgb.hsv(double hue, double saturation, double value) {
    final sector = (hue % 1) * 6;
    final chroma = value * saturation;
    final second = chroma * (1 - (sector % 2 - 1).abs());
    final floor = value - chroma;

    final (red, green, blue) = switch (sector.floor()) {
      0 => (chroma, second, 0.0),
      1 => (second, chroma, 0.0),
      2 => (0.0, chroma, second),
      3 => (0.0, second, chroma),
      4 => (second, 0.0, chroma),
      _ => (chroma, 0.0, second),
    };

    return Rgb.channels(
      ((red + floor) * 255).round(),
      ((green + floor) * 255).round(),
      ((blue + floor) * 255).round(),
    );
  }

  const Rgb.channels(int red, int green, int blue)
    : hex = red << 16 | green << 8 | blue;

  static const white = Rgb(0xFFFFFF);
  static const black = Rgb(0x000000);

  final int hex;

  int get red => hex >> 16 & 0xFF;

  int get green => hex >> 8 & 0xFF;

  int get blue => hex & 0xFF;

  /// The colour [amount] of the way from this one to [other].
  Rgb mix(Rgb other, double amount) => Rgb.channels(
    _blend(red, other.red, amount),
    _blend(green, other.green, amount),
    _blend(blue, other.blue, amount),
  );

  /// This colour pushed toward white, for highlights.
  Rgb tint(double amount) => mix(white, amount);

  /// This colour pushed toward black, for shadows and seams.
  Rgb shade(double amount) => mix(black, amount);

  static int _blend(int from, int to, double amount) =>
      (from + (to - from) * amount.clamp(0, 1)).round();
}
