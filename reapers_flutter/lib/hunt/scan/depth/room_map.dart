import 'dart:math' as math;
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';

import '../../pose/ar_pose.dart';
import '../room_survey.dart';

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
/// floor is, and how far the way is clear in each direction.
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
      _seen = Uint32List(HuntPlan.slices);

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

  /// The room a husk is given beyond the least it needs, in metres, so that
  /// rounding a plan to whole numbers cannot leave it short.
  static const _margin = 0.02;

  /// The floor is looked for in 2 cm steps between the lowest and highest a
  /// phone can be held above one.
  static const _floorStep = 0.02;
  static const _floorNearest = HuntPlan.minEyeCentimetres / 100;
  static const _floorFarthest = HuntPlan.maxEyeCentimetres / 100;

  final Uint16List _clear;
  final Uint16List _taken;
  final Uint32List _seen;
  final _floors = <double>[];

  /// Pictures taken in so far.
  int frames = 0;

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

    frames++;
    _reckon();

    return true;
  }

  /// How well slice [index] has been read, from 0 to 1.
  ///
  /// For a husk that walks, a slice is read once it has been followed out
  /// to something standing in it, or far enough along clear floor for the
  /// husk to stand there. A slice caught by the corner of a picture, with
  /// only the floor at the player's feet in view, is not read yet: nothing
  /// is known about it either way.
  double read(int index) => _read[index];

  /// How far the way is clear along slice [index], in metres: to the first
  /// thing standing in it, or else to the end of the floor that was seen.
  /// Null until the slice has been read.
  double? reach(int index) => _reach[index];

  /// Whether slice [index] has been read and found to leave the husk no
  /// room to stand anywhere across it.
  bool blocked(int index) =>
      read(index) >= 1 &&
      !_standable[2 * index] &&
      !_standable[2 * index + 1] &&
      !_standable[(2 * index + 2) % _marks];

  /// How much of the arc a husk needs has been found, from 0 to 1.
  double get progress {
    final (_, length) = widestRun(_standable);

    return ((length - 1) * _markWidth / RoomSurvey.neededArc).clamp(0.0, 1.0);
  }

  /// How much of the circle has been read at all, in radians.
  double get coverage =>
      Iterable<int>.generate(
        HuntPlan.slices,
      ).where((slice) => read(slice) >= 1).length *
      HuntPlan.sliceWidth;

  /// Where a husk may go, or null until an arc wide enough, with room all
  /// along it, has been read.
  HuntPlan? get plan {
    final (start, length) = widestRun(_standable);

    if ((length - 1) * _markWidth < RoomSurvey.neededArc) {
      return null;
    }

    if (length >= _marks) {
      return _room;
    }

    final from = _bearingOfMark(start);

    return HuntPlan.between(
      from,
      from + (length - 1) * _markWidth,
      eyeCentimetres: _room.eyeCentimetres,
      reachDecimetres: _room.reachDecimetres,
    );
  }

  /// The whole circle with the floor where it was found and nothing known
  /// of what stands on it: for a hunt begun before the room was read.
  HuntPlan get planUnread => HuntPlan(
    centreCentidegrees: 0,
    halfWidthCentidegrees: 18000,
    eyeCentimetres: _room.eyeCentimetres,
  );

  /// The circle is checked for standing room twice in every slice: at its
  /// edge and at its middle.
  static const _marks = 2 * HuntPlan.slices;
  static const _markWidth = HuntPlan.sliceWidth / 2;

  static double _bearingOfMark(int mark) =>
      wrapAngle(mark * _markWidth - math.pi);

  // What the pictures so far add up to. Worked out once for each picture
  // taken in, not each time it is asked for.

  /// How well each slice has been read, from 0 to 1.
  List<double> _read = List.filled(HuntPlan.slices, 0);

  /// Each slice's reach in metres, or null where it has not been read.
  List<double?> _reach = List.filled(HuntPlan.slices, null);

  /// The whole room as a plan would carry it.
  HuntPlan _room = HuntPlan.open;

  /// For each mark round the circle, whether the husk could stand there.
  List<bool> _standable = List.filled(_marks, false);

  void _reckon() {
    // One ring of the map in metres.
    final unit = vessel.gait == Gait.ground ? floorDepth ?? 0 : 1;
    final needed = HuntRules.closestMetres + vessel.halfWidth + _margin;
    final clear = [
      for (var slice = 0; slice < HuntPlan.slices; slice++) _clearRings(slice),
    ];

    _read = [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        math.min(
          (_seen[slice] / _pointsToRead).clamp(0.0, 1.0),
          clear[slice].stopped || vessel.gait != Gait.ground
              ? 1.0
              : (clear[slice].rings * _ringSize * unit / needed).clamp(
                  0.0,
                  1.0,
                ),
        ),
    ];
    _reach = [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        _read[slice] < 1 ? null : clear[slice].rings * _ringSize * unit,
    ];
    _room = HuntPlan(
      centreCentidegrees: 0,
      halfWidthCentidegrees: 18000,
      eyeCentimetres: ((floorDepth ?? HuntRules.eyeHeight) * 100).round(),
      reachDecimetres: [
        for (final metres in _reach)
          // Nothing is known to be in the way along a slice not read, so
          // it is given the most a plan can say.
          metres == null
              ? HuntPlan.maxReachDecimetres
              : (metres * 10).floor().clamp(0, HuntPlan.maxReachDecimetres),
      ],
    );

    _standable = [
      for (var mark = 0; mark < _marks; mark++)
        // A mark on the edge between two slices needs both of them read.
        read(mark ~/ 2) >= 1 &&
            (mark.isOdd || read((mark ~/ 2 - 1) % HuntPlan.slices) >= 1) &&
            _room.reach(_bearingOfMark(mark)) >= needed,
    ];
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
