import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:skribble/skribble.dart';

import '../theme/veil.dart';

/// The hand that rules every box in the ledger.
///
/// Skribble draws the line: two passes of a fine pen that bow away from the
/// corner and never quite meet. A seed fixes the wander for the life of a
/// box, so a rebuild redraws the same line and the ink does not crawl.
abstract final class Sketch {
  /// The width of a hand-drawn rule, in logical pixels.
  static const thickness = 1.15;

  /// A fine pen, held lightly: visible at a glance, never a marker stroke.
  static const _pen = RoughPen.fineliner;

  /// The wander of the hand. It is deliberately smaller than Skribble's
  /// default so a box still holds its shape in a dense ledger.
  static DrawConfig config(int seed) => DrawConfig.build(
    maxRandomnessOffset: 1.5,
    roughness: 0.7,
    bowing: 0.7,
    lineWobble: 0.85,
    curveFitting: 0.95,
    seed: seed,
    pen: _pen,
  );

  /// A sketched rule in [colour], deterministic for [seed].
  static RoughBoxDecoration box({
    Color colour = Veil.hairline,
    int seed = 7,
    double thickness = Sketch.thickness,
  }) => RoughBoxDecoration(
    borderStyle: RoughDrawingStyle(color: colour, width: thickness),
    drawConfig: config(seed),
    seed: seed,
  );

  /// A stable seed for [text], so two buttons with different labels are not
  /// drawn with the same wobble.
  static int seedFor(String text) {
    var hash = 0x811c9dc5;

    for (final unit in text.codeUnits) {
      hash = ((hash ^ unit) * 0x01000193) & 0x7fffffff;
    }

    return hash;
  }
}

/// A sketch of a box drawn around [child].
///
/// The line is painted over the child so a filled surface can sit beneath it,
/// and it takes no room from the layout: the child keeps exactly the size it
/// would have had.
class SketchFrame extends HookWidget {
  const SketchFrame({
    super.key,
    required this.child,
    this.colour = Veil.hairline,
    this.seed = 7,
    this.thickness = Sketch.thickness,
  });

  final Widget child;

  final Color colour;

  /// Fixes the wobble. The same seed always draws the same line.
  final int seed;

  final double thickness;

  @override
  Widget build(BuildContext context) {
    final decoration = useMemoized(
      () => Sketch.box(colour: colour, seed: seed, thickness: thickness),
      [colour, seed, thickness],
    );

    return DecoratedBox(
      decoration: decoration,
      position: DecorationPosition.foreground,
      child: child,
    );
  }
}
