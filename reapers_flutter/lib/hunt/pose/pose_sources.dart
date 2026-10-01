import 'dart:async';
import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:sensors_plus/sensors_plus.dart';

import 'ar_pose.dart';
import 'attitude_filter.dart';

/// Follows the phone itself, through its gyroscope and accelerometer.
final class DevicePoseSource implements PoseSource {
  DevicePoseSource() {
    _rotation = gyroscopeEventStream(
      samplingPeriod: SensorInterval.gameInterval,
    ).listen(_onRotation, onError: _onSensorError);
    _acceleration = accelerometerEventStream(
      samplingPeriod: SensorInterval.gameInterval,
    ).listen(_onAcceleration, onError: _onSensorError);
  }

  /// Gaps longer than this between gyroscope samples are a stall, not a
  /// turn, and are not integrated.
  static const _longestStep = Duration(milliseconds: 100);

  final _filter = AttitudeFilter();
  late final StreamSubscription<GyroscopeEvent> _rotation;
  late final StreamSubscription<AccelerometerEvent> _acceleration;
  DateTime? _lastRotation;

  /// Set when a sensor reports that it cannot be read.
  Object? failure;

  /// Whether the sensors have produced a usable pose yet.
  bool get isReady => _filter.isReady;

  @override
  ArPose get pose => _filter.isReady ? _filter.pose : ArPose.ahead;

  @override
  bool get followsDevice => true;

  @override
  void advance(double deltaSeconds) {}

  @override
  void drag(double dx, double dy) {}

  @override
  void dispose() {
    unawaited(_rotation.cancel());
    unawaited(_acceleration.cancel());
  }

  void _onRotation(GyroscopeEvent event) {
    final last = _lastRotation;

    _lastRotation = event.timestamp;

    if (last == null) {
      return;
    }

    final step = event.timestamp.difference(last);

    if (step <= Duration.zero || step > _longestStep) {
      return;
    }

    _filter.addRotation(
      event.x,
      event.y,
      event.z,
      step.inMicroseconds / Duration.microsecondsPerSecond,
    );
  }

  void _onAcceleration(AccelerometerEvent event) =>
      _filter.addAcceleration(event.x, event.y, event.z);

  void _onSensorError(Object error) => failure = error;
}

/// Follows a finger dragged across the screen, for anywhere the phone's own
/// motion cannot be read: a simulator, a desktop, a device with no gyroscope.
final class DragPoseSource implements PoseSource {
  /// How far up or down the view can be tilted.
  static final _pitchLimit = radians(80);

  /// How quickly the reported turn speed settles once the drag stops, as the
  /// time in seconds to lose most of it.
  static const _speedDecay = 0.09;

  double _yaw = 0;
  double _pitch = 0;
  double _pendingTurn = 0;
  double _turnSpeed = 0;

  @override
  ArPose get pose =>
      ArPose.looking(Vec3.fromAngles(_yaw, _pitch), turnSpeed: _turnSpeed);

  @override
  bool get followsDevice => false;

  @override
  void advance(double deltaSeconds) {
    if (deltaSeconds <= 0) {
      return;
    }

    final measured = _pendingTurn / deltaSeconds;
    final blend = 1 - math.exp(-deltaSeconds / _speedDecay);

    _pendingTurn = 0;
    _turnSpeed += (measured - _turnSpeed) * blend;
  }

  @override
  void drag(double dx, double dy) {
    _yaw = wrapAngle(_yaw + dx);
    _pitch = (_pitch + dy).clamp(-_pitchLimit, _pitchLimit);
    _pendingTurn += math.sqrt(dx * dx + dy * dy);
  }

  @override
  void dispose() {}
}
