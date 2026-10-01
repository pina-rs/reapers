/// The drawing surface for the stand-in NFT artwork.
library;

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:image/image.dart' as img;

import 'geometry.dart';
import 'palette.dart';

/// Colours one point of the design square, for fills that are not flat.
typedef Shader = Rgb Function(double x, double y);

/// A square surface that fills shapes with hard edges at several times the
/// final resolution, then averages down to it.
///
/// The `image` package fills polygons without anti-aliasing, and by testing
/// every pixel against every edge. Scanline filling into an oversized buffer
/// and letting the package average it down gives smooth edges for a fraction
/// of the work.
final class Canvas {
  Canvas({int supersampling = 4})
    : this._(supersampling, designSize.toInt() * supersampling);

  Canvas._(this.supersampling, this._side)
    : _pixels = Uint8List(_side * _side * 3),
      _coverage = Uint8List(_side * _side);

  /// How many samples are taken along each axis for one final pixel.
  final int supersampling;

  /// The width and height of the oversized buffer, in samples.
  final int _side;

  /// The oversized picture, three bytes to a sample.
  final Uint8List _pixels;

  /// Scratch space marking the samples the shape being filled covers.
  final Uint8List _coverage;

  /// The samples drawing is currently confined to, or null for everywhere.
  Uint8List? _clip;

  /// Fills [shape] with one flat [color].
  void fill(Shape shape, Rgb color, {double opacity = 1}) =>
      shade(shape, (_, _) => color, opacity: opacity);

  /// Fills the whole surface with [color].
  void background(Rgb color) =>
      fill(rect(0, 0, designSize, designSize).shape, color);

  /// Runs [draw] with every fill confined to [shape], and to any clip that is
  /// already in force.
  void clip(Shape shape, void Function() draw) {
    final outer = _clip;
    final inner = Uint8List(_coverage.length);

    _cover(shape, inner);

    if (outer != null) {
      for (var index = 0; index < inner.length; index++) {
        inner[index] &= outer[index];
      }
    }

    _clip = inner;

    try {
      draw();
    } finally {
      _clip = outer;
    }
  }

  /// The finished picture at the design size.
  img.Image render() {
    final oversized = img.Image.fromBytes(
      width: _side,
      height: _side,
      bytes: _pixels.buffer,
      numChannels: 3,
    );

    return img.copyResize(
      oversized,
      width: designSize.toInt(),
      height: designSize.toInt(),
      interpolation: img.Interpolation.average,
    );
  }

  /// Fills [shape] with whatever [shader] returns at each point.
  void shade(Shape shape, Shader shader, {double opacity = 1}) {
    final bounds = _cover(shape, _coverage);
    final clip = _clip;
    final keep = 1 - opacity;

    for (var row = bounds.top; row <= bounds.bottom; row++) {
      final y = (row + 0.5) / supersampling;

      for (var column = bounds.left; column < bounds.right; column++) {
        final sample = row * _side + column;

        if (_coverage[sample] == 0) {
          continue;
        }

        _coverage[sample] = 0;

        if (clip != null && clip[sample] == 0) {
          continue;
        }

        final color = shader((column + 0.5) / supersampling, y);
        final offset = sample * 3;

        _pixels[offset] = _blend(_pixels[offset], color.red, keep);
        _pixels[offset + 1] = _blend(_pixels[offset + 1], color.green, keep);
        _pixels[offset + 2] = _blend(_pixels[offset + 2], color.blue, keep);
      }
    }
  }

  /// One channel of new paint laid over old, of which [keep] shows through.
  static int _blend(int below, int above, double keep) =>
      (below * keep + above * (1 - keep)).round();

  /// Marks the samples of [shape] in [target] and returns the box they fall in.
  _Bounds _cover(Shape shape, Uint8List target) {
    final bounds = _Bounds();

    for (final path in shape.paths) {
      _rasterise(path, target, 1, bounds);
    }

    for (final hole in shape.holes) {
      _rasterise(hole, target, 0, bounds);
    }

    return bounds;
  }

  /// Writes [value] into every sample of [target] whose centre lies inside
  /// [path], using the non-zero winding rule so a ribbon that crosses itself
  /// stays solid.
  void _rasterise(Path path, Uint8List target, int value, _Bounds bounds) {
    final count = path.length;

    if (count < 3) {
      return;
    }

    final xs = Float64List(count);
    final ys = Float64List(count);
    var top = double.infinity;
    var bottom = double.negativeInfinity;

    for (var index = 0; index < count; index++) {
      xs[index] = path[index].x * supersampling;
      ys[index] = path[index].y * supersampling;
      top = math.min(top, ys[index]);
      bottom = math.max(bottom, ys[index]);
    }

    final firstRow = math.max(0, (top - 0.5).ceil());
    final lastRow = math.min(_side - 1, (bottom - 0.5).floor());
    final crossings = Float64List(count);
    final directions = Int8List(count);

    for (var row = firstRow; row <= lastRow; row++) {
      final sampleY = row + 0.5;
      var found = 0;

      for (var index = 0; index < count; index++) {
        final next = index + 1 == count ? 0 : index + 1;
        final startY = ys[index];
        final endY = ys[next];

        if ((startY <= sampleY) == (endY <= sampleY)) {
          continue;
        }

        final x =
            xs[index] +
            (sampleY - startY) / (endY - startY) * (xs[next] - xs[index]);
        var slot = found;

        // Kept sorted as it is built: a row rarely crosses more than a
        // handful of edges, so an insertion beats sorting afterwards.
        while (slot > 0 && crossings[slot - 1] > x) {
          crossings[slot] = crossings[slot - 1];
          directions[slot] = directions[slot - 1];
          slot--;
        }

        crossings[slot] = x;
        directions[slot] = endY > startY ? 1 : -1;
        found++;
      }

      var winding = 0;

      for (var index = 0; index < found - 1; index++) {
        winding += directions[index];

        if (winding == 0) {
          continue;
        }

        final from = math.max(0, (crossings[index] - 0.5).ceil());
        final to = math.min(_side, (crossings[index + 1] - 0.5).ceil());

        if (to <= from) {
          continue;
        }

        target.fillRange(row * _side + from, row * _side + to, value);
        bounds.include(from, to, row);
      }
    }
  }
}

/// The box of samples a fill touched, so painting can skip the rest.
final class _Bounds {
  int left = 1 << 30;
  int right = 0;
  int top = 1 << 30;
  int bottom = -1;

  void include(int from, int to, int row) {
    left = math.min(left, from);
    right = math.max(right, to);
    top = math.min(top, row);
    bottom = math.max(bottom, row);
  }
}
