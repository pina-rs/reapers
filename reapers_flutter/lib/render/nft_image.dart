import 'dart:ui' as ui;

import 'package:flutter/services.dart';
import 'package:reapers_shared/reapers_shared.dart';

/// Where an NFT's portrait is bundled.
///
/// A portrait is a render of the NFT's own creature in its colours, made by
/// the art build. No collection's artwork is used; see docs/nfts.md.
String nftImageAsset(Nft nft) => 'assets/nfts/${nft.imageKey}.png';

/// An NFT's portrait, decoded, with the colour it would glow. The husk of a
/// rift wears these on its cubes.
final class NftImage {
  const NftImage({required this.image, required this.light});

  /// Loads and decodes [nft]'s image.
  static Future<NftImage> load(Nft nft, {AssetBundle? bundle}) async {
    final data = await (bundle ?? rootBundle).load(nftImageAsset(nft));
    final codec = await ui.instantiateImageCodec(data.buffer.asUint8List());
    final frame = await codec.getNextFrame();
    final pixels = await frame.image.toByteData(
      format: ui.ImageByteFormat.rawRgba,
    );

    return NftImage(
      image: frame.image,
      light: pixels == null
          ? _fallbackLight
          : dominantLight(pixels, frame.image.width, frame.image.height),
    );
  }

  /// The light of a husk whose image has no colour to speak of.
  static const _fallbackLight = ui.Color(0xFF9FD8E8);

  final ui.Image image;

  /// The most vivid colour in the image, brightened: what the husk's glow,
  /// smoke and seals are tinted with, so each husk belongs to its picture.
  final ui.Color light;

  /// Picks the glow colour for an image from its raw RGBA [pixels].
  ///
  /// Colours are weighed by how vivid they are, so a small bright detail
  /// outvotes a large dull background.
  static ui.Color dominantLight(ByteData pixels, int width, int height) {
    // A coarse grid is plenty: the answer is one colour.
    final step = (width * height / 4096).ceil().clamp(1, 1 << 20);
    var red = 0.0;
    var green = 0.0;
    var blue = 0.0;
    var weight = 0.0;

    for (var pixel = 0; pixel < width * height; pixel += step) {
      final r = pixels.getUint8(pixel * 4) / 255;
      final g = pixels.getUint8(pixel * 4 + 1) / 255;
      final b = pixels.getUint8(pixel * 4 + 2) / 255;
      final brightest = [r, g, b].reduce((a, b) => a > b ? a : b);
      final dullest = [r, g, b].reduce((a, b) => a < b ? a : b);
      final vividness = (brightest - dullest) * brightest;
      final share = vividness * vividness;

      red += r * share;
      green += g * share;
      blue += b * share;
      weight += share;
    }

    if (weight < 1e-6) {
      return _fallbackLight;
    }

    final r = red / weight;
    final g = green / weight;
    final b = blue / weight;
    final peak = [r, g, b].reduce((a, b) => a > b ? a : b);

    // Scale so the strongest channel is full: a glow is never dim.
    return ui.Color.from(
      alpha: 1,
      red: r / peak,
      green: g / peak,
      blue: b / peak,
    );
  }
}
