import 'dart:math' as math;
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';

import '../../pose/ar_pose.dart';

/// One look at the room: how far away everything in a picture is, and where
/// the phone was pointing when it was taken.
final class DepthFrame {
  const DepthFrame({
    required this.width,
    required this.height,
    required this.depths,
    required this.pose,
    required this.tanHalfWidth,
    required this.tanHalfHeight,
  });

  final int width;
  final int height;

  /// How far away each pixel is, in metres along the direction the camera
  /// faced, row by row from the top.
  final Float32List depths;

  /// Where the phone was pointing.
  final ArPose pose;

  /// How much the picture takes in: the tangent of half its view across and
  /// of half its view up and down.
  final double tanHalfWidth;
  final double tanHalfHeight;
}

/// The room as seen from above, put together from depth pictures: where the
/// floor is, how far the way is clear in each direction, and what stands at
/// the end of each way: how tall it is, whether its top is level enough to
/// stand on, and whether there is room behind it to hide.
///
/// Each picture becomes a cloud of points around the player. For a husk
/// that walks, the floor is found in every picture on its own: the lowest
/// level surface with any size to it. Everything in the picture is then
/// measured in multiples of how far that floor is below the eye, and only
/// the floor's depth, taken across all the pictures, is trusted in metres.
/// A depth model reads a whole picture long or short together, so this
/// keeps one picture's floor and furniture in step with the next one's, and
/// it puts a husk's feet on the floor the camera shows even when the
/// model's metres are off.
///
/// This holds no camera and no model, only arithmetic, so that it can be
/// tested against rooms drawn by hand.
final class RoomMap {
  RoomMap({required this.vessel})
    : _clear = Uint16List(HuntPlan.slices * _rings),
      _taken = Uint16List(HuntPlan.slices * _rings),
      _seen = Uint32List(HuntPlan.slices),
      _floorAt = Uint16List(HuntPlan.slices * _rings),
      _solid = Uint16List(HuntPlan.slices * _rings),
      _heights = Uint16List(HuntPlan.slices * _rings * _heightBins),
      _levelHeights = Uint16List(HuntPlan.slices * _rings * _heightBins);

  /// The husk the room is being read for: how it gets about decides what
  /// counts as in its way, and its size decides where it fits.
  final Vessel vessel;

  /// Rings of the map, counted out from the player. For a walker each is
  /// [_ringSize] of the eye's height above the floor wide; for a flier,
  /// that many metres.
  static const _rings = 90;
  static const _ringSize = 0.07;

  /// Every other pixel each way is enough to map a room with.
  static const _stride = 2;

  /// Points a slice needs before it counts as read.
  static const _pointsToRead = 60;

  /// Points in one ring of one slice that make it floor, or make it taken.
  static const _pointsOfFloor = 3;
  static const _pointsInTheWay = 5;

  /// Nearer than this, in metres, is the player's own feet and hands.
  static const _selfMetres = 0.6;

  /// How far off the floor a point may be and still be floor: a little, and
  /// a little more for every metre away, where depth is less sure.
  static const _floorSlack = 0.07;
  static const _floorSlackPerMetre = 0.045;

  /// The floor is looked for in 2 cm steps between the lowest and highest a
  /// phone can be held above one.
  static const _floorStep = 0.02;
  static const _floorNearest = HuntPlan.minEyeCentimetres / 100;
  static const _floorFarthest = HuntPlan.maxEyeCentimetres / 100;

  final Uint16List _clear;
  final Uint16List _taken;
  final Uint32List _seen;
  final _floors = <double>[];

  // What stands on the floor, whatever the husk. Every length is in
  // multiples of the floor's depth in its own picture, like the rings.

  /// Points of floor in each cell.
  final Uint16List _floorAt;

  /// Points of anything standing on the floor in each cell.
  final Uint16List _solid;

  /// How high those points are, cell by cell, in bins of [_heightBin], and
  /// the same for the points among them that lie on a level surface.
  final Uint16List _heights;
  final Uint16List _levelHeights;

  static const _heightBins = 40;
  static const _heightBin = 0.04;

  /// Points in the highest bin that make it the top of something.
  static const _pointsOfTop = 3;

  /// Level points around the top that make it a ledge, and the share of the
  /// points there they must be.
  static const _pointsOfLedge = 12;
  static const _ledgeShare = 0.35;

  /// How far, in metres, a hiding husk keeps from the back of its cover and
  /// from what stands behind it.
  static const _hideGap = 0.1;

  /// Pictures taken in so far.
  int frames = 0;

  /// How far below the eye the floor was in the last picture taken in, in
  /// that picture's own metres. Each picture is read against its own
  /// floor, and this is how far to scale it to agree with [floorDepth].
  double? lastPictureFloor;

  /// How far the floor is below the player's eye, in metres: the middle of
  /// what every picture made of it. Null until a picture has shown floor.
  double? get floorDepth {
    if (_floors.isEmpty) {
      return null;
    }

    final sorted = [..._floors]..sort();

    return sorted[sorted.length ~/ 2];
  }

  /// Takes in one picture. Answers whether anything was learnt from it: a
  /// picture with no floor in it teaches nothing about where a walker can
  /// stand.
  bool add(DepthFrame frame) {
    final columns = frame.width ~/ _stride;
    final rows = frame.height ~/ _stride;
    final count = columns * rows;
    final xs = Float32List(count);
    final ys = Float32List(count);
    final zs = Float32List(count);
    final pose = frame.pose;

    for (var row = 0; row < rows; row++) {
      final up =
          (1 - (row * _stride + 0.5) / frame.height * 2) * frame.tanHalfHeight;

      for (var column = 0; column < columns; column++) {
        final across =
            ((column * _stride + 0.5) / frame.width * 2 - 1) *
            frame.tanHalfWidth;
        final depth =
            frame.depths[row * _stride * frame.width + column * _stride];
        final index = row * columns + column;

        xs[index] =
            depth * (pose.forward.x + pose.right.x * across + pose.up.x * up);
        ys[index] =
            depth * (pose.forward.y + pose.right.y * across + pose.up.y * up);
        zs[index] =
            depth * (pose.forward.z + pose.right.z * across + pose.up.z * up);
      }
    }

    final level = _levelPoints(xs, ys, zs, columns, rows);
    final floor = _floorOf(ys, level);
    final agreed = _floors.length < _picturesToAgree ? null : floorDepth;

    if (floor != null) {
      _floors.add(floor);
    }

    if (vessel.gait == Gait.ground) {
      if (floor == null) {
        return false;
      }

      // A picture with a table across it and hardly any floor takes the
      // table for the floor. Once a few pictures agree where the floor is,
      // one that puts it somewhere else altogether is not charted.
      if (agreed != null && (floor / agreed - 1).abs() > _floorLeeway) {
        return false;
      }

      _chartFloor(xs, ys, zs, floor);
    } else {
      _chartAir(xs, ys, zs, level);
    }

    if (floor != null &&
        (agreed == null || (floor / agreed - 1).abs() <= _floorLeeway)) {
      _chartFurniture(xs, ys, zs, level, floor);
    }

    lastPictureFloor = floor;
    frames++;
    _reckon();

    return true;
  }

  /// What stands at the end of the way along slice [index]: how tall it is,
  /// where a husk could hide behind it, and whether its top is a ledge. Null
  /// where the way ends for want of floor, or the slice has not been read.
  Furniture? furniture(int index) => _furniture[index];

  /// How well slice [index] has been seen, from 0 to 1.
  double read(int index) => _read[index];

  /// How far the way is clear along slice [index], in metres: to the first
  /// thing standing in it, or else to the end of the floor that was seen.
  /// Null until the slice has been read.
  double? reach(int index) => _reach[index];

  /// Whether slice [index] has been read and found tight: so little clear
  /// floor that the husk can only stand there as close as it ever comes.
  bool tight(int index) => switch (_reach[index]) {
    final metres? => metres - vessel.halfWidth < HuntRules.closestMetres,
    null => false,
  };

  /// How much of the circle has been read at all, in radians.
  double get coverage =>
      Iterable<int>.generate(
        HuntPlan.slices,
      ).where((slice) => read(slice) >= 1).length *
      HuntPlan.sliceWidth;

  /// The plan for a hunt kept to [halfWidth] radians either side of the
  /// bearing [centre]: the floor where it was found, and how far the way is
  /// clear along every slice that has been read.
  ///
  /// A room is never too small to hunt in. Where there is little clear
  /// floor the husk stands closer, and the plan says how much there is.
  /// Nothing is known to be in the way along a slice that has not been
  /// read, so it is given the most a plan can say. A plan made before any
  /// slice was read says nothing about the room but the floor.
  HuntPlan planWithin(double centre, double halfWidth) => HuntPlan.between(
    centre - halfWidth,
    centre + halfWidth,
    eyeCentimetres: ((floorDepth ?? HuntRules.eyeHeight) * 100).round(),
    reachDecimetres: _surveyed ? _reachDecimetres : const [],
    coverCentimetres: _surveyed
        ? [
            for (final found in _furniture)
              found == null
                  ? 0
                  : (found.cover * 100).round().clamp(
                      1,
                      HuntPlan.maxCoverCentimetres,
                    ),
          ]
        : const [],
    hideDecimetres: _surveyed
        ? [
            for (var slice = 0; slice < HuntPlan.slices; slice++)
              _hideDecimetres(slice),
          ]
        : const [],
    ledges: _surveyed
        ? [for (final found in _furniture) found?.ledge ?? false]
        : const [],
  );

  bool get _surveyed => _reach.any((metres) => metres != null);

  List<int> get _reachDecimetres => [
    for (final metres in _reach)
      metres == null
          ? HuntPlan.maxReachDecimetres
          : (metres * 10).floor().clamp(0, HuntPlan.maxReachDecimetres),
  ];

  /// Where a husk hides behind the cover along [slice], as a plan records
  /// it: always beyond the reach, or nowhere.
  int _hideDecimetres(int slice) {
    final hide = _furniture[slice]?.hide;
    final reach = _reach[slice];

    if (hide == null || reach == null) {
      return 0;
    }

    final decimetres = (hide * 10).round();

    return decimetres > (reach * 10).floor() &&
            decimetres <= HuntPlan.maxReachDecimetres
        ? decimetres
        : 0;
  }

  // What the pictures so far add up to. Worked out once for each picture
  // taken in, not each time it is asked for.

  /// How well each slice has been read, from 0 to 1.
  List<double> _read = List.filled(HuntPlan.slices, 0);

  /// Each slice's reach in metres, or null where it has not been read.
  List<double?> _reach = List.filled(HuntPlan.slices, null);

  /// What stands at the end of each slice's reach.
  List<Furniture?> _furniture = List.filled(HuntPlan.slices, null);

  void _reckon() {
    // One ring of the map in metres.
    final unit = vessel.gait == Gait.ground ? floorDepth ?? 0 : 1;
    final clear = [
      for (var slice = 0; slice < HuntPlan.slices; slice++) _clearRings(slice),
    ];

    _read = [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        (_seen[slice] / _pointsToRead).clamp(0.0, 1.0),
    ];
    _reach = [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        _read[slice] < 1 ? null : clear[slice].rings * _ringSize * unit,
    ];
    _furniture = [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        _read[slice] < 1 ? null : _furnitureAlong(slice, clear[slice]),
    ];

    // A husk that flies keeps in front of a ledge it may come down onto, so
    // that it lands on the front of it.
    if (vessel.gait != Gait.ground) {
      _reach = [
        for (var slice = 0; slice < HuntPlan.slices; slice++)
          switch ((_reach[slice], _furniture[slice])) {
            (final reach?, final Furniture found) when found.front < reach =>
              found.front,
            (final reach, _) => reach,
          },
      ];
    }
  }

  /// What stands at the end of slice [slice]'s way, given how far the way is
  /// [clear].
  ///
  /// For a husk that walks, that is whatever stopped the way. For one that
  /// flies, it is the nearest level top below its air, which it may come
  /// down and perch on; anything else under it does not matter to it.
  Furniture? _furnitureAlong(int slice, ({int rings, bool stopped}) clear) {
    final unit = floorDepth;

    if (unit == null) {
      return null;
    }

    final ground = vessel.gait == Gait.ground;
    final front = ground
        ? (clear.stopped ? clear.rings : null)
        : _firstSolid(slice, from: 0);

    if (front == null) {
      return null;
    }

    // It runs back from its front for as long as there is something there,
    // over a gap of a ring at most.
    var back = front;

    for (var ring = front + 1, gap = 0; ring < _rings; ring++) {
      if (_isSolid(slice, ring)) {
        back = ring;
        gap = 0;
      } else if (++gap > 1) {
        break;
      }
    }

    final top = _topBin(slice, front, back);

    if (top == null) {
      return null;
    }

    final cover = (top + 1) * _heightBin * unit;
    final ledge = _isLedge(slice, front, back, top);

    if (!ground) {
      return ledge
          ? Furniture(
              front: front * _ringSize * unit,
              cover: cover,
              ledge: true,
            )
          : null;
    }

    return Furniture(
      front: front * _ringSize * unit,
      cover: cover,
      ledge: ledge,
      hide: _hideBehind(slice, back, unit),
    );
  }

  /// Where a husk can hide behind something whose far side is in ring
  /// [back] of slice [slice]: in the gap before whatever stands behind it,
  /// or just behind it where floor shows beyond it. Nowhere when nothing
  /// behind it was seen at all: a husk is not put somewhere unknown.
  double? _hideBehind(int slice, int back, double unit) {
    final behind = (back + 1) * _ringSize * unit;
    final next = _firstSolid(slice, from: back + 2);
    var floorBeyond = false;

    for (var ring = back + 1; ring < _rings && !floorBeyond; ring++) {
      floorBeyond = _floorAt[slice * _rings + ring] >= _pointsOfFloor;
    }

    final width = 2 * vessel.halfWidth;
    final end = next == null
        ? (floorBeyond ? behind + width + 3 * _hideGap : null)
        : next * _ringSize * unit;

    if (end == null || end - behind < width + _hideGap) {
      return null;
    }

    return math.min(
      behind + vessel.halfWidth + _hideGap,
      end - vessel.halfWidth - _hideGap / 2,
    );
  }

  bool _isSolid(int slice, int ring) {
    final cell = slice * _rings + ring;

    return _solid[cell] >= _pointsInTheWay && _solid[cell] >= _floorAt[cell];
  }

  int? _firstSolid(int slice, {required int from}) {
    for (var ring = from; ring < _rings; ring++) {
      if (_isSolid(slice, ring)) {
        return ring;
      }
    }

    return null;
  }

  /// The highest height bin with enough points in it, over rings [front] to
  /// [back] of [slice].
  int? _topBin(int slice, int front, int back) {
    for (var bin = _heightBins - 1; bin >= 0; bin--) {
      var points = 0;

      for (var ring = front; ring <= back; ring++) {
        points += _heights[(slice * _rings + ring) * _heightBins + bin];
      }

      if (points >= _pointsOfTop) {
        return bin;
      }
    }

    return null;
  }

  /// Whether the top of what stands over rings [front] to [back] of [slice],
  /// in bin [top], is a level surface: a table or a counter.
  bool _isLedge(int slice, int front, int back, int top) {
    var level = 0;
    var all = 0;

    for (var ring = front; ring <= back; ring++) {
      for (var bin = math.max(0, top - 1); bin <= top; bin++) {
        final at = (slice * _rings + ring) * _heightBins + bin;

        level += _levelHeights[at];
        all += _heights[at];
      }
    }

    return level >= _pointsOfLedge && level >= all * _ledgeShare;
  }

  /// How many rings out from the player slice [index] is clear, and whether
  /// that is because something stands in the next one. Otherwise it is as
  /// far as floor has been seen.
  ({int rings, bool stopped}) _clearRings(int index) {
    var floorEnd = 0;

    for (var ring = 0; ring < _rings; ring++) {
      final cell = index * _rings + ring;

      // Something stands here, and more of it than there is floor.
      if (_taken[cell] >= _pointsInTheWay && _taken[cell] >= _clear[cell]) {
        return (rings: ring, stopped: true);
      }

      if (_clear[cell] >= _pointsOfFloor) {
        floorEnd = ring + 1;
      }
    }

    // In the air nothing has to be underfoot: clear is clear, as far as the
    // map goes.
    return (
      rings: vessel.gait == Gait.ground ? floorEnd : _rings,
      stopped: false,
    );
  }

  /// Which points lie on a level surface: a floor, a table top, a ceiling.
  ///
  /// The surface at a point faces along the cross product of the steps to
  /// the next point across and the next point down. It is level when that
  /// points nearly straight up or down.
  Uint8List _levelPoints(
    Float32List xs,
    Float32List ys,
    Float32List zs,
    int columns,
    int rows,
  ) {
    final level = Uint8List(xs.length);

    for (var row = 0; row < rows; row++) {
      for (var column = 0; column < columns; column++) {
        final at = row * columns + column;
        // At the last column and the last row the step is taken backwards.
        // Which way the cross product ends up pointing does not matter.
        final right = column + 1 < columns ? at + 1 : at - 1;
        final down = row + 1 < rows ? at + columns : at - columns;
        final ax = xs[right] - xs[at];
        final ay = ys[right] - ys[at];
        final az = zs[right] - zs[at];
        final bx = xs[down] - xs[at];
        final by = ys[down] - ys[at];
        final bz = zs[down] - zs[at];
        final nx = ay * bz - az * by;
        final ny = az * bx - ax * bz;
        final nz = ax * by - ay * bx;
        final length = math.sqrt(nx * nx + ny * ny + nz * nz);

        if (length > 0 && ny.abs() / length >= _levelShare) {
          level[at] = 1;
        }
      }
    }

    return level;
  }

  /// How far below the eye the floor in one picture is, or null when the
  /// picture shows none.
  ///
  /// The floor is the lowest level surface that is a fair share of the
  /// level surface in the picture: lower than the table tops, and bigger
  /// than the stray points below it that a depth model always leaves.
  double? _floorOf(Float32List ys, Uint8List level) {
    final steps = ((_floorFarthest - _floorNearest) / _floorStep).ceil();
    final tally = Float64List(steps);

    for (var index = 0; index < ys.length; index++) {
      final below = -ys[index];

      if (level[index] == 1 &&
          below >= _floorNearest &&
          below < _floorFarthest) {
        tally[((below - _floorNearest) / _floorStep).floor()]++;
      }
    }

    // A surface's points land a few steps either side of it, so each step
    // is taken together with its neighbours.
    final smooth = Float64List(steps);
    var most = 0.0;

    for (var step = 0; step < steps; step++) {
      for (
        var near = math.max(0, step - _floorBlur);
        near <= math.min(steps - 1, step + _floorBlur);
        near++
      ) {
        smooth[step] += tally[near];
      }

      most = math.max(most, smooth[step]);
    }

    if (most < _floorPoints) {
      return null;
    }

    for (var step = steps - 1; step >= 0; step--) {
      final isPeak =
          smooth[step] >= most * _floorShare &&
          smooth[step] >= _floorPoints &&
          smooth[step] >= smooth[math.max(0, step - 1)] &&
          smooth[step] >= smooth[math.min(steps - 1, step + 1)];

      if (!isPeak) {
        continue;
      }

      // The blur spreads a surface over the steps either side of it, so
      // the peak says roughly where the floor is and the points around it
      // say exactly.
      var points = 0.0;
      var depth = 0.0;

      for (
        var near = math.max(0, step - 2 * _floorBlur);
        near <= math.min(steps - 1, step + 2 * _floorBlur);
        near++
      ) {
        points += tally[near];
        depth += tally[near] * (_floorNearest + (near + 0.5) * _floorStep);
      }

      return depth / points;
    }

    return null;
  }

  /// How nearly straight up a surface has to face to be level.
  static const _levelShare = 0.9;

  /// Steps either side that are counted together with each step.
  static const _floorBlur = 2;

  /// Pictures that have to have found a floor before one that disagrees
  /// with them is set aside, and how far, as a share, it may disagree.
  static const _picturesToAgree = 3;
  static const _floorLeeway = 0.25;

  /// The fewest level points that make a floor, and the share of the
  /// biggest level surface a lower one must have to be the floor instead.
  static const _floorPoints = 120;
  static const _floorShare = 0.15;

  /// Charts one picture for a husk that walks. Every length is divided by
  /// [floor], the picture's own floor depth, before it goes on the map.
  void _chartFloor(
    Float32List xs,
    Float32List ys,
    Float32List zs,
    double floor,
  ) {
    final top = 2 * vessel.halfHeight + 0.2;

    for (var index = 0; index < xs.length; index++) {
      final away = math.sqrt(xs[index] * xs[index] + zs[index] * zs[index]);

      if (away < _selfMetres) {
        continue;
      }

      final ring = (away / floor / _ringSize).floor();

      if (ring >= _rings) {
        continue;
      }

      final above = floor + ys[index];
      final slack = _floorSlack + _floorSlackPerMetre * away;
      final slice = _sliceOf(xs[index], zs[index]);
      final cell = slice * _rings + ring;

      if (above.abs() <= slack) {
        _clear[cell]++;
        _seen[slice]++;
      } else if (above > slack && above < top) {
        _taken[cell]++;
        _seen[slice]++;
      }
    }
  }

  /// Charts what stands on the floor in one picture, for any husk: every
  /// length is divided by [floor], the picture's own floor depth.
  ///
  /// Only what is no higher than the eye is charted. Anything taller is a
  /// wall as far as a husk is concerned, and a ceiling is not furniture.
  void _chartFurniture(
    Float32List xs,
    Float32List ys,
    Float32List zs,
    Uint8List level,
    double floor,
  ) {
    for (var index = 0; index < xs.length; index++) {
      final away = math.sqrt(xs[index] * xs[index] + zs[index] * zs[index]);

      if (away < _selfMetres || ys[index] > 0) {
        continue;
      }

      final ring = (away / floor / _ringSize).floor();

      if (ring >= _rings) {
        continue;
      }

      final above = floor + ys[index];
      final slack = _floorSlack + _floorSlackPerMetre * away;
      final cell = _sliceOf(xs[index], zs[index]) * _rings + ring;

      if (above.abs() <= slack) {
        _floorAt[cell]++;
        continue;
      }

      if (above < slack) {
        continue;
      }

      final bin = (above / floor / _heightBin).floor().clamp(
        0,
        _heightBins - 1,
      );

      _solid[cell]++;
      _heights[cell * _heightBins + bin]++;

      if (level[index] == 1) {
        _levelHeights[cell * _heightBins + bin]++;
      }
    }
  }

  /// Charts one picture for a husk that flies: what matters is whatever
  /// stands up into the band of air it keeps to.
  ///
  /// A level surface is left out. A ceiling or a floor that the band
  /// brushes is not a wall, and would otherwise close off the whole room.
  void _chartAir(
    Float32List xs,
    Float32List ys,
    Float32List zs,
    Uint8List level,
  ) {
    final low = vessel.lowMetres - vessel.halfHeight - 0.1;
    final high = vessel.highMetres + vessel.halfHeight + 0.1;

    for (var index = 0; index < xs.length; index++) {
      final away = math.sqrt(xs[index] * xs[index] + zs[index] * zs[index]);
      final inTheBand =
          away >= _selfMetres &&
          level[index] == 0 &&
          ys[index] >= low &&
          ys[index] <= high;

      if (!inTheBand) {
        continue;
      }

      final slice = _sliceOf(xs[index], zs[index]);
      final ring = (away / _ringSize).floor();

      _seen[slice]++;

      if (ring < _rings) {
        _taken[slice * _rings + ring]++;
      }
    }
  }

  static int _sliceOf(double x, double z) =>
      ((math.atan2(x, z) + math.pi) / HuntPlan.sliceWidth).floor().clamp(
        0,
        HuntPlan.slices - 1,
      );
}

/// Something standing in the room, at the end of one slice's way.
final class Furniture {
  const Furniture({
    required this.front,
    required this.cover,
    required this.ledge,
    this.hide,
  });

  /// How far its front is from the player, in metres.
  final double front;

  /// How high its top is above the floor, in metres.
  final double cover;

  /// Whether its top is level enough to stand on.
  final bool ledge;

  /// How far out a husk could hide behind it, in metres, or null where
  /// there is no room behind it, or nothing behind it was seen.
  final double? hide;
}
