import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/rift/place/location_tracker.dart';
import 'package:reapers_flutter/rift/place/whereabouts.dart';
import 'package:reapers_flutter/src/backend/game_backend.dart';
import 'package:reapers_shared/reapers_shared.dart';

final class _Phone implements Whereabouts {
  final readings = StreamController<GeoFix>.broadcast(sync: true);
  @override
  Stream<GeoFix> watch() => readings.stream;
}

void main() {
  final start = DateTime.utc(2026, 10, 5, 12);
  GeoFix fix(DateTime now, {double accuracy = 4}) => GeoFix(
    at: const GeoPoint(51.5, -.08),
    accuracy: accuracy,
    observedAt: now,
  );

  test('only precise fixes are shared, at most once every five seconds', () {
    fakeAsync((time) {
      final phone = _Phone();
      final sent = <GeoFix?>[];
      DateTime now() => start.add(time.elapsed);
      final tracker = LocationTracker(
        source: phone,
        share: (fix) async {
          sent.add(fix);
        },
        now: now,
      )..start();
      phone.readings.add(fix(now(), accuracy: 80));
      time.flushMicrotasks();
      expect(tracker.inaccurate, isTrue);
      expect(sent, isEmpty);
      phone.readings.add(fix(now().add(const Duration(milliseconds: 1))));
      time.flushMicrotasks();
      expect(sent, hasLength(1));
      time.elapse(const Duration(seconds: 4));
      phone.readings.add(fix(now()));
      time.flushMicrotasks();
      expect(sent, hasLength(1));
      time.elapse(const Duration(seconds: 1));
      time.flushMicrotasks();
      expect(sent, hasLength(2));
      tracker.dispose();
      time.flushMicrotasks();
      unawaited(phone.readings.close());
    });
  });

  test(
    'stale readings are cleared; a foreground stop cancels the GPS listener',
    () {
      fakeAsync((time) {
        final phone = _Phone();
        final sent = <GeoFix?>[];
        DateTime now() => start.add(time.elapsed);
        final tracker = LocationTracker(
          source: phone,
          share: (fix) async {
            sent.add(fix);
          },
          now: now,
        )..start();
        phone.readings.add(fix(now()));
        time.flushMicrotasks();
        time.elapse(const Duration(seconds: 31));
        time.flushMicrotasks();
        expect(tracker.stale, isTrue);
        expect(tracker.fix, isNull);
        expect(sent.last, isNull);
        tracker.stop();
        time.flushMicrotasks();
        expect(phone.readings.hasListener, isFalse);
        final count = sent.length;
        time.elapse(const Duration(minutes: 1));
        expect(sent, hasLength(count));
        tracker.start();
        phone.readings.add(fix(now()));
        time.flushMicrotasks();
        expect(tracker.fix, isNotNull);
        tracker.dispose();
        time.flushMicrotasks();
        unawaited(phone.readings.close());
      });
    },
  );

  test(
    'stop serializes a clear after an in-flight upload, even after disposal',
    () {
      fakeAsync((time) {
        final phone = _Phone();
        final first = Completer<void>();
        final sent = <GeoFix?>[];
        final tracker = LocationTracker(
          source: phone,
          share: (fix) {
            sent.add(fix);
            return sent.length == 1 ? first.future : Future.value();
          },
          now: () => start,
        )..start();
        phone.readings.add(fix(start));
        tracker.dispose();
        expect(sent, hasLength(1));
        first.complete();
        time.flushMicrotasks();
        expect(sent, hasLength(2));
        expect(sent.last, isNull);
        unawaited(phone.readings.close());
      });
    },
  );

  test('permission and network failures are visible and can recover', () {
    fakeAsync((time) {
      final phone = _Phone();
      var reachable = false;
      final tracker = LocationTracker(
        source: phone,
        share: (fix) async {
          if (!reachable) throw const BackendUnavailable('test outage');
        },
        now: () => start,
      )..start();
      phone.readings.addError(const LocationFailure(LocationIssue.blocked));
      time.flushMicrotasks();
      expect(tracker.issue, LocationIssue.blocked);
      expect(tracker.sharingFailed, isTrue);
      reachable = true;
      tracker.retry();
      phone.readings.add(fix(start));
      time.flushMicrotasks();
      expect(tracker.issue, isNull);
      expect(tracker.sharingFailed, isFalse);
      expect(tracker.fix, isNotNull);
      tracker.dispose();
      time.flushMicrotasks();
      unawaited(phone.readings.close());
    });
  });
}
