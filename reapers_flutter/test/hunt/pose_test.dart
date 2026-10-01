import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/hunt/pose/ar_pose.dart';
import 'package:reapers_flutter/hunt/pose/attitude_filter.dart';
import 'package:reapers_flutter/hunt/pose/pose_sources.dart';
import 'package:reapers_shared/reapers_shared.dart';

const _gravity = 9.80665;
const _fov = 62 * math.pi / 180;
const _aspect = 9 / 19.5;

Matcher _near(Vec3 expected, [double tolerance = 1e-3]) => predicate<Vec3>(
  (actual) => (actual - expected).length <= tolerance,
  'is within $tolerance of $expected',
);

/// A filter that has settled on a phone held upright in portrait.
AttitudeFilter _upright() => AttitudeFilter()..addAcceleration(0, _gravity, 0);

/// Turns [filter] about a device axis at [speed] radians a second for
/// [seconds], in small steps as the sensor would deliver them.
void _turn(
  AttitudeFilter filter, {
  double x = 0,
  double y = 0,
  double z = 0,
  double seconds = 1,
}) {
  const step = 0.01;

  for (var elapsed = 0.0; elapsed < seconds - 1e-9; elapsed += step) {
    filter.addRotation(x, y, z, step);
  }
}

void main() {
  group('AttitudeFilter', () {
    test('a phone held upright looks straight ahead and level', () {
      final pose = _upright().pose;

      expect(pose.forward, _near(Vec3.forward));
      expect(pose.up, _near(Vec3.up));
      expect(pose.right, _near(const Vec3(1, 0, 0)));
    });

    test('is not ready until gravity has been felt', () {
      final filter = AttitudeFilter();

      expect(filter.isReady, isFalse);

      filter.addRotation(0, 1, 0, 0.01);

      expect(filter.isReady, isFalse);

      filter.addAcceleration(0, _gravity, 0);

      expect(filter.isReady, isTrue);
    });

    test('a phone lying face up looks at the ground', () {
      final pose = (AttitudeFilter()..addAcceleration(0, 0, _gravity)).pose;

      expect(pose.forward, _near(const Vec3(0, -1, 0)));
    });

    test('turning left swings the view to the left', () {
      final filter = _upright();

      // A quarter turn anticlockwise seen from above is a positive turn
      // about the phone's up axis.
      _turn(filter, y: math.pi / 2);

      expect(filter.pose.forward, _near(const Vec3(-1, 0, 0), 0.01));
      expect(filter.pose.up, _near(Vec3.up, 0.01));
    });

    test('tilting the top of the phone back looks up', () {
      final filter = _upright();

      _turn(filter, x: math.pi / 4);

      expect(filter.pose.forward.pitch, closeTo(math.pi / 4, 0.01));
      expect(filter.pose.forward.yaw, closeTo(0, 0.01));
    });

    test('the three directions stay at right angles', () {
      final filter = _upright();

      _turn(filter, x: 0.7, y: -1.3, z: 0.4, seconds: 2);

      final pose = filter.pose;

      expect(pose.forward.dot(pose.up), closeTo(0, 1e-6));
      expect(pose.forward.dot(pose.right), closeTo(0, 1e-6));
      expect(pose.up.dot(pose.right), closeTo(0, 1e-6));
      expect(pose.forward.length, closeTo(1, 1e-6));
    });

    test('gravity slowly corrects a tilt the gyroscope got wrong', () {
      final filter = _upright();

      // The gyroscope claims a tilt that never happened.
      _turn(filter, x: 0.3);

      final wrong = filter.pose.forward.pitch.abs();

      for (var sample = 0; sample < 400; sample++) {
        filter.addAcceleration(0, _gravity, 0);
      }

      expect(wrong, greaterThan(0.25));
      expect(filter.pose.forward.pitch.abs(), lessThan(0.01));
    });

    test('gravity never changes the heading', () {
      final filter = _upright();

      _turn(filter, y: 1);

      final heading = filter.pose.forward.yaw;

      for (var sample = 0; sample < 400; sample++) {
        filter.addAcceleration(0, _gravity, 0);
      }

      expect(filter.pose.forward.yaw, closeTo(heading, 1e-3));
    });

    test('a shake is not mistaken for gravity', () {
      final filter = _upright();

      for (var sample = 0; sample < 200; sample++) {
        filter.addAcceleration(14, 3, -6);
      }

      expect(filter.pose.forward, _near(Vec3.forward));
    });

    test('the turn speed follows the gyroscope and settles when it stops', () {
      final filter = _upright();

      _turn(filter, y: 2, seconds: 0.5);

      expect(filter.pose.turnSpeed, closeTo(2, 0.05));

      _turn(filter, seconds: 0.5);

      expect(filter.pose.turnSpeed, lessThan(0.01));
    });
  });

  group('ArPose', () {
    test('the middle of the screen is where the phone points', () {
      expect(
        ArPose.ahead.rayThrough(x: 0, y: 0, verticalFov: _fov, aspect: _aspect),
        _near(Vec3.forward),
      );
    });

    test('the top edge of the screen is half the field of view up', () {
      final ray = ArPose.ahead.rayThrough(
        x: 0,
        y: 1,
        verticalFov: _fov,
        aspect: _aspect,
      );

      expect(ray.pitch, closeTo(_fov / 2, 1e-9));
    });

    test('projecting a ray gives back the point it came through', () {
      final pose = (DragPoseSource()..drag(0.7, -0.3)).pose;

      for (final (x, y) in [(0.0, 0.0), (0.6, -0.4), (-0.9, 0.8)]) {
        final ray = pose.rayThrough(
          x: x,
          y: y,
          verticalFov: _fov,
          aspect: _aspect,
        );
        final point = pose.project(ray, verticalFov: _fov, aspect: _aspect)!;

        expect(point.x, closeTo(x, 1e-9));
        expect(point.y, closeTo(y, 1e-9));
      }
    });

    test('what is behind the player is not on the screen', () {
      expect(
        ArPose.ahead.project(
          const Vec3(0, 0, -1),
          verticalFov: _fov,
          aspect: _aspect,
        ),
        isNull,
      );
    });
  });

  group('DragPoseSource', () {
    test('a drag to the right turns the view to the right', () {
      final source = DragPoseSource()..drag(math.pi / 2, 0);

      expect(source.pose.forward, _near(const Vec3(1, 0, 0)));
      expect(source.pose.right, _near(const Vec3(0, 0, -1)));
      expect(source.followsDevice, isFalse);
    });

    test('the view cannot be tipped over backwards', () {
      final source = DragPoseSource()..drag(0, 10);

      expect(source.pose.forward.pitch, lessThan(math.pi / 2));
      expect(source.pose.up.y, greaterThan(0));
    });

    test('reports how fast it was dragged, then comes to rest', () {
      final source = DragPoseSource();

      for (var frame = 0; frame < 30; frame++) {
        source
          ..drag(0.03, 0)
          ..advance(1 / 60);
      }

      expect(source.pose.turnSpeed, closeTo(1.8, 0.1));

      for (var frame = 0; frame < 60; frame++) {
        source.advance(1 / 60);
      }

      expect(source.pose.turnSpeed, lessThan(0.01));
    });
  });
}
