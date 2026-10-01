/// Builds the stand-in artwork for every NFT the market lists.
///
///     dart run tool/build_nft_art.dart
///
/// One 512 by 512 PNG per item lands in `reapers_flutter/assets/nfts`, named
/// by `Nft.imageKey`. The four collections are invented (see docs/nfts.md), so
/// their art is invented here too: each collection has one fixed composition
/// and a set of layered traits, the way generative profile-picture sets are
/// made. Nothing in the assets folder is edited by hand.
///
/// The build is deterministic. Traits come from an item's serial and rarity
/// and any scatter is seeded from its id, so rebuilding writes identical
/// files.
///
/// A contact sheet per collection lands in `art/build/previews`, which is not
/// committed. Each sheet shows the items at half size, then at the size of a
/// market thumbnail, then cropped to a circle, because an image has to read
/// in all three.
library;

import 'dart:io';

import 'package:image/image.dart' as img;
import 'package:reapers_shared/reapers_shared.dart';

import 'nft_art/brine_hounds.dart';
import 'nft_art/canvas.dart';
import 'nft_art/mothlight_parade.dart';
import 'nft_art/static_choir.dart';
import 'nft_art/tin_saints.dart';

const _assets = 'reapers_flutter/assets/nfts';
const _previews = 'art/build/previews';

/// The size the market list draws an item at.
const _thumbnailSize = 96;

void main() {
  final artworks = [
    for (final nft in marketListings) (nft: nft, image: _paint(nft)),
  ];
  var bytesWritten = 0;

  for (final artwork in artworks) {
    final png = img.encodePng(artwork.image, level: 9);

    File('$_assets/${artwork.nft.imageKey}.png')
      ..createSync(recursive: true)
      ..writeAsBytesSync(png);

    bytesWritten += png.length;
  }

  for (final collection in NftCollection.values) {
    final sheet = _contactSheet([
      for (final artwork in artworks)
        if (artwork.nft.collection == collection) artwork.image,
    ]);

    File('$_previews/nfts_${collection.name}.png')
      ..createSync(recursive: true)
      ..writeAsBytesSync(img.encodePng(sheet));
  }

  stdout.writeln(
    'Wrote ${artworks.length} images to $_assets '
    '(${(bytesWritten / 1024).round()} KiB) and '
    '${NftCollection.values.length} contact sheets to $_previews.',
  );

  _reportStrays();
}

/// Paints one item in its collection's composition.
///
/// The switch has no default on purpose: a collection added to the catalogue
/// stops this tool compiling until it has artwork.
img.Image _paint(Nft nft) {
  final canvas = Canvas();

  switch (nft.collection) {
    case NftCollection.tinSaints:
      paintTinSaint(canvas, nft);

    case NftCollection.mothlightParade:
      paintMoth(canvas, nft);

    case NftCollection.staticChoir:
      paintChorister(canvas, nft);

    case NftCollection.brineHounds:
      paintBrineHound(canvas, nft);
  }

  return canvas.render();
}

const _sheetWidth = 1084;
const _sheetGutter = 12;

/// Lays [artworks] out in three strips: half size, market thumbnails, and
/// the thumbnails cropped to a circle.
img.Image _contactSheet(List<img.Image> artworks) {
  const half = 256;

  final strips = [
    _Strip(half, [for (final artwork in artworks) _scaled(artwork, half)]),
    _Strip(_thumbnailSize, [
      for (final artwork in artworks) _scaled(artwork, _thumbnailSize),
    ]),
    _Strip(_thumbnailSize, [
      for (final artwork in artworks)
        img.copyCropCircle(
          _scaled(artwork, _thumbnailSize).convert(numChannels: 4),
        ),
    ]),
  ];
  final sheet = img.Image(
    width: _sheetWidth,
    height: strips.fold(_sheetGutter, (sum, strip) => sum + strip.height),
  );
  var top = _sheetGutter;

  img.fill(sheet, color: img.ColorRgb8(24, 24, 28));

  for (final strip in strips) {
    strip.placeOn(sheet, top);
    top += strip.height;
  }

  return sheet;
}

/// A run of equally sized tiles on a contact sheet, wrapped to its width.
final class _Strip {
  const _Strip(this.tile, this.images);

  final int tile;
  final List<img.Image> images;

  int get _columns => (_sheetWidth - _sheetGutter) ~/ (tile + _sheetGutter);

  /// The height of the strip, including the gutter below it.
  int get height => (images.length / _columns).ceil() * (tile + _sheetGutter);

  void placeOn(img.Image sheet, int top) {
    for (final (index, image) in images.indexed) {
      img.compositeImage(
        sheet,
        image,
        dstX: _sheetGutter + index % _columns * (tile + _sheetGutter),
        dstY: top + index ~/ _columns * (tile + _sheetGutter),
      );
    }
  }
}

img.Image _scaled(img.Image artwork, int size) => img.copyResize(
  artwork,
  width: size,
  height: size,
  interpolation: img.Interpolation.average,
);

/// Warns about images left behind by items the catalogue no longer lists, so
/// the assets folder cannot quietly drift from it.
void _reportStrays() {
  final expected = {for (final nft in marketListings) '${nft.imageKey}.png'};
  final strays = [
    for (final entity in Directory(_assets).listSync())
      if (entity is File && !expected.contains(entity.uri.pathSegments.last))
        entity.path,
  ]..sort();

  for (final stray in strays) {
    stderr.writeln('Not in the catalogue, delete it: $stray');
  }
}
