import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

void main() {
  final now = DateTime.utc(2026, 10, 5, 12);
  final fix = GeoFix(
    at: const GeoPoint(51.5, -.08),
    accuracy: 4,
    observedAt: now,
  );

  test('measured fix survives wire and database serialization', () {
    final read = GeoFix.fromJson(fix.toJson());
    expect(read.at, fix.at);
    expect(read.accuracy, fix.accuracy);
    expect(read.observedAt, now);
    expect(read.isUsableAt(now), isTrue);
  });

  test(
    'freshness includes the boundary and tolerates only small clock skew',
    () {
      expect(fix.isUsableAt(now.add(RiftRules.locationMaxAge)), isTrue);
      expect(
        fix.isUsableAt(
          now.add(RiftRules.locationMaxAge + const Duration(microseconds: 1)),
        ),
        isFalse,
      );
      expect(fix.isUsableAt(now.subtract(RiftRules.locationClockSkew)), isTrue);
      expect(
        fix.isUsableAt(
          now.subtract(
            RiftRules.locationClockSkew + const Duration(microseconds: 1),
          ),
        ),
        isFalse,
      );
    },
  );

  test(
    'coarse, missing, nonfinite and out-of-range coordinates are unusable',
    () {
      for (final at in [
        const GeoPoint(91, 0),
        const GeoPoint(0, 181),
        const GeoPoint(double.nan, 0),
        const GeoPoint(0, double.infinity),
      ]) {
        expect(fix.copyWith(at: at).isUsableAt(now), isFalse);
      }
      for (final accuracy in [0.0, -1.0, double.nan, double.infinity, 25.01]) {
        expect(fix.copyWith(accuracy: accuracy).isUsableAt(now), isFalse);
      }
      expect(fix.copyWith(accuracy: 25).isUsableAt(now), isTrue);
    },
  );

  test('a live standing needs a fresh fix and a finite compass bearing', () {
    GeoStanding standing(double facing, DateTime? time) => GeoStanding(
      at: fix.at,
      accuracy: fix.accuracy,
      facing: facing,
      observedAt: time,
    );
    expect(standing(0, now).isUsableAt(now), isTrue);
    expect(standing(double.nan, now).isUsableAt(now), isFalse);
    expect(standing(0, null).isUsableAt(now), isFalse);
    expect(GeoStanding.fromJson(standing(1, now).toJson()).observedAt, now);
  });
}
