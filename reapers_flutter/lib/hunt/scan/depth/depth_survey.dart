import 'dart:async';

import 'package:camera/camera.dart';
import 'package:flutter/foundation.dart';
import 'package:reapers_shared/reapers_shared.dart';

import '../../camera/hunt_camera.dart';
import '../../pose/ar_pose.dart';
import '../room_survey.dart';
import 'camera_picture.dart';
import 'depth_model.dart';
import 'room_map.dart';

/// Reads the room by looking at it: camera frames go through the depth
/// model as the player sweeps the phone around, and what comes out is put
/// together into a map of the floor and what stands on it.
///
/// Nothing leaves the phone. What the hunt keeps of the room is the plan:
/// how high the phone was held, and how far the way is clear in each
/// direction.
final class DepthSurvey implements RoomSurvey {
  DepthSurvey({
    required this.camera,
    required this.model,
    required this.pose,
    required Vessel vessel,
  }) : _map = RoomMap(vessel: vessel);

  final HuntCamera camera;

  /// Closed by the survey when it is done.
  final DepthModel model;

  /// Where the phone points right now.
  final ArPose Function() pose;

  /// Turning faster than this, in radians a second, no picture is taken. A
  /// picture is pinned to where the phone pointed when it arrived, and the
  /// faster the phone turns the further out that is.
  static const steadyTurnSpeed = 0.7;

  /// Once this much of the circle has been read without finding room for
  /// the husk, the player is told to look for somewhere more open.
  static final _enoughToJudge = radians(200);

  final RoomMap _map;
  ReadingAdvice _posture = ReadingAdvice.keepGoing;
  bool _busy = false;
  bool _disposed = false;

  @override
  Object? failure;

  /// Pictures read so far.
  int get pictures => _map.frames;

  /// How far below the phone the floor has been found to be, in metres, or
  /// null before any floor has been seen.
  double? get floorDepth => _map.floorDepth;

  /// Starts taking pictures. Answers false when there is no camera to take
  /// them with.
  Future<bool> start() => camera.watch(_onFrame);

  @override
  ReadingAdvice get advice {
    if (_posture != ReadingAdvice.keepGoing) {
      return _posture;
    }

    return _map.plan == null && _map.coverage >= _enoughToJudge
        ? ReadingAdvice.findRoom
        : ReadingAdvice.keepGoing;
  }

  @override
  void observe(ArPose pose, double deltaSeconds) {
    _posture = postureAdvice(
      pose,
      _map.vessel.gait,
      steadyTurnSpeed: steadyTurnSpeed,
    );
  }

  @override
  double read(int index) => _map.read(index);

  @override
  double? reach(int index) => _map.reach(index);

  @override
  bool blocked(int index) => _map.blocked(index);

  @override
  double get progress => _map.progress;

  @override
  HuntPlan? get plan => _map.plan;

  @override
  HuntPlan get planUnread => _map.planUnread;

  @override
  void dispose() {
    if (_disposed) {
      return;
    }

    _disposed = true;
    unawaited(camera.stopWatching());

    // A picture still with the model is let finish, and closes it after.
    if (!_busy) {
      unawaited(model.close());
    }
  }

  void _onFrame(CameraImage frame) {
    final camera = this.camera.controller.value;

    if (_busy || _disposed || failure != null || camera == null) {
      return;
    }

    final taken = pose();
    final posture = postureAdvice(
      taken,
      _map.vessel.gait,
      steadyTurnSpeed: steadyTurnSpeed,
    );

    if (posture != ReadingAdvice.keepGoing) {
      return;
    }

    final picture = CameraPicture.from(
      frame,
      sensorOrientation: camera.description.sensorOrientation,
      width: DepthModel.width,
      height: DepthModel.height,
    );

    if (picture == null) {
      failure = UnsupportedError(
        'Camera frames arrive as ${frame.format.group.name}, turned '
        '${camera.description.sensorOrientation}°, which the room reading '
        'cannot take apart',
      );

      return;
    }

    final lens = this.camera.optics.view(
      longSide: frame.width > frame.height
          ? frame.width.toDouble()
          : frame.height.toDouble(),
      shortSide: frame.width > frame.height
          ? frame.height.toDouble()
          : frame.width.toDouble(),
    );

    _busy = true;
    unawaited(
      _read(
        picture,
        taken,
        tanHalfWidth: lens.tanHalfShort * picture.shareOfShortSide,
        tanHalfHeight: lens.tanHalfLong * picture.shareOfLongSide,
      ),
    );
  }

  Future<void> _read(
    CameraPicture picture,
    ArPose taken, {
    required double tanHalfWidth,
    required double tanHalfHeight,
  }) async {
    try {
      final depths = await model.estimate(picture.pixels);

      if (_disposed) {
        return;
      }

      final long = DepthModel.overreach(DepthModel.width / (2 * tanHalfWidth));

      for (var index = 0; index < depths.length; index++) {
        depths[index] /= long;
      }

      final learnt = _map.add(
        DepthFrame(
          width: DepthModel.width,
          height: DepthModel.height,
          depths: depths,
          pose: taken,
          tanHalfWidth: tanHalfWidth,
          tanHalfHeight: tanHalfHeight,
        ),
      );

      // A line for each picture, for whoever is working out why a room
      // reads the way it does. Never in a release build.
      if (!kReleaseMode) {
        debugPrint(
          'Room reading: picture ${learnt ? 'kept' : 'set aside'}, '
          'floor ${_map.floorDepth?.toStringAsFixed(2) ?? 'not found'} m '
          'down, ${degrees(_map.coverage).round()}° read, '
          '${(_map.progress * 100).round()}% of the room it needs',
        );
      }
    } on Object catch (error) {
      // Whatever stopped the model, the survey reads no more. The hunt
      // screen shows why instead of leaving the player sweeping for ever.
      failure = error;
    } finally {
      _busy = false;

      if (_disposed) {
        unawaited(model.close());
      }
    }
  }
}
