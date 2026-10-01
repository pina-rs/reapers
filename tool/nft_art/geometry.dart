/// Points, paths and shapes for the stand-in NFT artwork.
///
/// Everything here is measured in design units on a 512 by 512 square with
/// the origin at the top left and `y` pointing down, so a collection can be
/// laid out once and rendered at any resolution.
library;

import 'dart:math' as math;

/// The side of the design square every collection is laid out on.
const double designSize = 512;

/// The vertical axis symmetric subjects are mirrored about.
const double centreX = designSize / 2;

/// A point or an offset on the design square.
final class Vec {
  const Vec(this.x, this.y);

  /// The offset [radius] long at [angle] radians. Because `y` points down, a
  /// growing angle turns clockwise on screen, starting from "right".
  factory Vec.polar(double radius, double angle) =>
      Vec(radius * math.cos(angle), radius * math.sin(angle));

  static const zero = Vec(0, 0);

  final double x;
  final double y;

  double get length => math.sqrt(x * x + y * y);

  Vec operator +(Vec other) => Vec(x + other.x, y + other.y);

  Vec operator -(Vec other) => Vec(x - other.x, y - other.y);

  Vec operator *(double factor) => Vec(x * factor, y * factor);

  /// The point [amount] of the way from here to [other].
  Vec towards(Vec other, double amount) => this + (other - this) * amount;
}

/// A closed outline, or the spine of a stroke, as a run of straight segments.
///
/// Curves are flattened into enough segments that the corners disappear once
/// the canvas is downsampled.
typedef Path = List<Vec>;

/// The area covered by any of [paths] and by none of [holes].
///
/// Each path is filled on its own before they are combined, so overlapping or
/// mirrored outlines join into one silhouette instead of cancelling out.
final class Shape {
  const Shape(this.paths, {this.holes = const []});

  final List<Path> paths;
  final List<Path> holes;

  /// This shape together with its reflection across the vertical centre line.
  Shape get withMirror => Shape(
    [...paths, for (final path in paths) path.mirrored],
    holes: [...holes, for (final hole in holes) hole.mirrored],
  );
}

/// The moves that reuse one outline in several places.
extension PathTransforms on Path {
  /// The reflection of this path across the vertical centre line.
  Path get mirrored => [
    for (final point in this) Vec(designSize - point.x, point.y),
  ];

  Path shifted(double dx, double dy) => [
    for (final point in this) Vec(point.x + dx, point.y + dy),
  ];

  /// This path turned clockwise by [radians] around [origin].
  Path rotatedAbout(Vec origin, double radians) {
    final cosine = math.cos(radians);
    final sine = math.sin(radians);

    return [
      for (final point in this)
        Vec(
          origin.x +
              (point.x - origin.x) * cosine -
              (point.y - origin.y) * sine,
          origin.y +
              (point.x - origin.x) * sine +
              (point.y - origin.y) * cosine,
        ),
    ];
  }
}

/// Wraps a single outline as a shape.
extension PathShape on Path {
  Shape get shape => Shape([this]);
}

/// How many segments keep a curve of [radius] smooth at the build's
/// supersampling without wasting time on tiny details.
int _segmentsFor(double radius) => (radius * 1.5).clamp(16, 288).ceil();

Path circle(Vec centre, double radius) => ellipse(centre, radius, radius);

/// An ellipse, optionally turned clockwise by [tilt] radians.
Path ellipse(Vec centre, double radiusX, double radiusY, {double tilt = 0}) {
  final segments = _segmentsFor(math.max(radiusX, radiusY));
  final outline = [
    for (var index = 0; index < segments; index++)
      Vec(
        centre.x + radiusX * math.cos(index * math.pi * 2 / segments),
        centre.y + radiusY * math.sin(index * math.pi * 2 / segments),
      ),
  ];

  return tilt == 0 ? outline : outline.rotatedAbout(centre, tilt);
}

Path rect(double left, double top, double width, double height) => [
  Vec(left, top),
  Vec(left + width, top),
  Vec(left + width, top + height),
  Vec(left, top + height),
];

/// A rectangle whose corners are quarter circles of [radius].
Path roundedRect(
  double left,
  double top,
  double width,
  double height,
  double radius,
) {
  final corner = math.min(radius, math.min(width, height) / 2);
  final right = left + width;
  final bottom = top + height;

  return [
    ...arc(Vec(right - corner, top + corner), corner, -math.pi / 2, 0),
    ...arc(Vec(right - corner, bottom - corner), corner, 0, math.pi / 2),
    ...arc(Vec(left + corner, bottom - corner), corner, math.pi / 2, math.pi),
    ...arc(Vec(left + corner, top + corner), corner, math.pi, math.pi * 1.5),
  ];
}

/// The open run of points along a circle from angle [from] to angle [to].
Path arc(Vec centre, double radius, double from, double to) {
  final sweep = (to - from).abs();
  final segments = math.max(
    2,
    (_segmentsFor(radius) * sweep / (math.pi * 2)).ceil(),
  );

  return [
    for (var index = 0; index <= segments; index++)
      centre + Vec.polar(radius, from + (to - from) * index / segments),
  ];
}

/// A pie slice: the centre joined to the arc between [from] and [to].
Path wedge(Vec centre, double radius, double from, double to) => [
  centre,
  ...arc(centre, radius, from, to),
];

/// A ring between two radii, as one shape with a hole.
Shape ring(Vec centre, double outer, double inner) =>
    Shape([circle(centre, outer)], holes: [circle(centre, inner)]);

/// A star with [points] spikes reaching [outer], with valleys at [inner].
Path burst(
  Vec centre,
  double outer,
  double inner,
  int points, {
  double phase = -math.pi / 2,
}) => [
  for (var index = 0; index < points * 2; index++)
    centre +
        Vec.polar(
          index.isEven ? outer : inner,
          phase + index * math.pi / points,
        ),
];

/// A gear outline: [teeth] flat-topped teeth rising from [inner] to [outer].
Path cog(Vec centre, double outer, double inner, int teeth) {
  final step = math.pi * 2 / teeth;

  return [
    for (var tooth = 0; tooth < teeth; tooth++) ...[
      centre + Vec.polar(inner, (tooth - 0.5) * step),
      centre + Vec.polar(outer, (tooth - 0.3) * step),
      centre + Vec.polar(outer, (tooth + 0.3) * step),
      centre + Vec.polar(inner, (tooth + 0.5) * step),
    ],
  ];
}

/// A smooth closed outline passing through every one of [knots].
///
/// Organic silhouettes such as wings and ears are far easier to author as a
/// handful of points on the outline than as Bézier handles.
Path blob(List<Vec> knots, {int steps = 14}) {
  final count = knots.length;

  return [
    for (var index = 0; index < count; index++)
      for (var step = 0; step < steps; step++)
        _catmullRom(
          knots[(index - 1 + count) % count],
          knots[index],
          knots[(index + 1) % count],
          knots[(index + 2) % count],
          step / steps,
        ),
  ];
}

/// A smooth open line passing through every one of [knots].
Path curve(List<Vec> knots, {int steps = 14}) {
  final last = knots.length - 1;

  return [
    for (var index = 0; index < last; index++)
      for (var step = 0; step < steps; step++)
        _catmullRom(
          knots[math.max(index - 1, 0)],
          knots[index],
          knots[index + 1],
          knots[math.min(index + 2, last)],
          step / steps,
        ),
    knots[last],
  ];
}

Vec _catmullRom(Vec before, Vec from, Vec to, Vec after, double t) {
  final t2 = t * t;
  final t3 = t2 * t;

  return (from * 2 +
          (to - before) * t +
          (before * 2 - from * 5 + to * 4 - after) * t2 +
          (from * 3 - before - to * 3 + after) * t3) *
      0.5;
}

/// Builds an outline from straight lines and Bézier curves, for silhouettes
/// that need sharp corners a [blob] would round away.
final class PathBuilder {
  PathBuilder(Vec start) : _points = [start];

  static const _steps = 20;

  final List<Vec> _points;

  void lineTo(Vec end) => _points.add(end);

  void quadTo(Vec control, Vec end) => cubicTo(
    _points.last.towards(control, 2 / 3),
    end.towards(control, 2 / 3),
    end,
  );

  void cubicTo(Vec control1, Vec control2, Vec end) {
    final start = _points.last;

    for (var step = 1; step <= _steps; step++) {
      final t = step / _steps;
      final u = 1 - t;

      _points.add(
        start * (u * u * u) +
            control1 * (3 * u * u * t) +
            control2 * (3 * u * t * t) +
            end * (t * t * t),
      );
    }
  }

  Path build() => List.unmodifiable(_points);
}

/// The band around [spine] whose width at each point comes from [widthAt],
/// which is given how far along the spine the point lies, from 0 to 1.
Path ribbon(Path spine, double Function(double t) widthAt) {
  final last = spine.length - 1;
  final left = <Vec>[];
  final right = <Vec>[];

  for (var index = 0; index <= last; index++) {
    final tangent =
        spine[math.min(index + 1, last)] - spine[math.max(index - 1, 0)];
    final length = tangent.length;

    if (length == 0) {
      continue;
    }

    final normal = Vec(-tangent.y / length, tangent.x / length);
    final half = widthAt(index / last) / 2;

    left.add(spine[index] + normal * half);
    right.add(spine[index] - normal * half);
  }

  return [...left, ...right.reversed];
}

/// A round-capped line of constant [width] along [spine].
///
/// A joint also gets a round cap wherever the spine turns sharply, because a
/// plain ribbon pinches at a corner.
Shape stroke(Path spine, double width) {
  final last = spine.length - 1;

  return Shape([
    ribbon(spine, (_) => width),
    circle(spine.first, width / 2),
    circle(spine.last, width / 2),
    for (var index = 1; index < last; index++)
      if (_isCorner(spine[index - 1], spine[index], spine[index + 1]))
        circle(spine[index], width / 2),
  ]);
}

bool _isCorner(Vec before, Vec at, Vec after) {
  final incoming = at - before;
  final outgoing = after - at;
  final lengths = incoming.length * outgoing.length;

  if (lengths == 0) {
    return false;
  }

  return (incoming.x * outgoing.x + incoming.y * outgoing.y) / lengths < 0.9;
}

/// A band of [width] centred on a closed [outline].
///
/// Clipped to the outline's own shape it becomes an inner border, which is
/// how the dark margin of a moth's wing is drawn.
Shape outlineStroke(Path outline, double width) {
  final count = outline.length;
  final outer = <Vec>[];
  final inner = <Vec>[];

  for (var index = 0; index < count; index++) {
    final tangent =
        outline[(index + 1) % count] - outline[(index - 1 + count) % count];
    final length = tangent.length;

    if (length == 0) {
      continue;
    }

    final normal = Vec(-tangent.y / length, tangent.x / length) * (width / 2);

    outer.add(outline[index] + normal);
    inner.add(outline[index] - normal);
  }

  // One loop out and the other back, joined by a seam that is walked in both
  // directions and so cancels. The band is filled whichever way the outline
  // winds, which a path with a hole could not promise.
  return Shape([
    [...outer, outer.first, inner.first, ...inner.reversed],
  ]);
}
