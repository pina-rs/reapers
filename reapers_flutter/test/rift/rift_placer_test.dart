import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/rift/place/rift_placer.dart';
import 'package:reapers_shared/reapers_shared.dart';

const _garden = GeoPoint(52.2, 0.12);

GeoFix _fix(GeoPoint at) =>
    GeoFix(at: at, accuracy: 4, observedAt: DateTime.now().toUtc());

/// A phone in a rift: its placer, where it is, which way its owner first
/// faced (which is what its own view calls straight ahead), and which way it
/// points now.
final class _Phone {
  _Phone({
    required GeoPoint at,
    required this.firstFacedDegrees,
    required this.pointsDegrees,
  }) {
    placer.fix = _fix(at);
  }

  final placer = RiftPlacer();

  /// The compass bearing of what this phone's view calls straight ahead.
  final double firstFacedDegrees;

  /// The compass bearing the phone points along now.
  double pointsDegrees;

  /// Where north lies in this phone's view.
  double get north => radians(-firstFacedDegrees);

  /// Which way the phone points, in its own view.
  double get facing => radians(pointsDegrees - firstFacedDegrees);

  /// Shows the phone [frame], and passes what it has to say on to [room].
  void see(RiftRoom room, int seat) {
    final standing = placer.observe(
      room.frameFor(seat),
      facing: facing,
      north: north,
    );

    if (standing != null) {
      room.report(seat, RiftReport(ready: false, standing: standing));
    }
  }

  /// Where on the earth this phone would be pointing, were it turned to
  /// the husk as it sees it.
  GeoPoint lookingAt() {
    final sighting = placer.sighting!;

    return placer.fix!.at.moved(
      bearing: sighting.bearing - north,
      metres: placer.fix!.at.distanceTo(_anchorSeen!),
    );
  }

  GeoPoint? _anchorSeen;
}

RiftRoom _room() => RiftRoom(site: RiftSite.londonBridge, seed: 3);

int _join(RiftRoom room, String name) =>
    room.join(RiftEntrant(name: name, loadout: HuntLoadout.borrowed));

void main() {
  test('a stale or coarse fix cannot place the shared husk', () {
    final now = DateTime.utc(2026, 10, 5);
    final room = _room();
    final seat = _join(room, 'One');
    final placer = RiftPlacer(now: () => now);
    final precise = GeoFix(at: _garden, accuracy: 4, observedAt: now);
    for (final fix in [
      precise.copyWith(accuracy: 80),
      precise.copyWith(observedAt: now.subtract(const Duration(seconds: 31))),
    ]) {
      placer.fix = fix;
      expect(placer.observe(room.frameFor(seat), facing: 0, north: 0), isNull);
      expect(placer.sighting, isNull);
    }
    placer.fix = precise;
    expect(placer.observe(room.frameFor(seat), facing: 0, north: 0), isNotNull);
  });

  test('the first phone to find itself plants the rift ahead of it', () {
    final room = _room();
    final seat = _join(room, 'One');
    final phone = _Phone(at: _garden, firstFacedDegrees: 20, pointsDegrees: 70);

    phone.see(room, seat);

    final anchor = room.anchor!;

    expect(_garden.distanceTo(anchor), closeTo(RiftRules.plantMetres, 1e-3));
    expect(degrees(_garden.bearingTo(anchor)), closeTo(70, 1e-3));
  });

  test('it then sees the rift where it was pointing', () {
    final room = _room();
    final seat = _join(room, 'One');
    final phone = _Phone(at: _garden, firstFacedDegrees: 20, pointsDegrees: 70);

    phone
      ..see(room, seat)
      ..see(room, seat);

    // It was pointing 50° to the right of where it first faced.
    expect(degrees(phone.placer.sighting!.bearing), closeTo(50, 0.01));
    expect(phone.placer.sighting!.metres, closeTo(25, 0.01));
  });

  test('two phones side by side end up pointing at the same spot', () {
    final room = _room();
    final first = _join(room, 'One');
    final second = _join(room, 'Two');
    final one = _Phone(at: _garden, firstFacedDegrees: 20, pointsDegrees: 70);
    final other = _Phone(
      at: _garden.moved(bearing: radians(150), metres: 4),
      firstFacedDegrees: -110,
      pointsDegrees: 200,
    );

    for (var frame = 0; frame < 3; frame++) {
      one.see(room, first);
      other.see(room, second);
    }

    final anchor = room.anchor!;

    one._anchorSeen = anchor;
    other._anchorSeen = anchor;

    expect(one.lookingAt().distanceTo(anchor), lessThan(0.05));
    expect(other.lookingAt().distanceTo(anchor), lessThan(0.05));
    // The second phone started out facing somewhere else entirely, so the
    // rift lies in a different part of its own view.
    expect(
      wrapAngle(
        one.placer.sighting!.bearing - other.placer.sighting!.bearing,
      ).abs(),
      greaterThan(radians(90)),
    );
  });

  test('once the rift has its place, nobody else is asked for theirs', () {
    final room = _room();
    final first = _join(room, 'One');
    final second = _join(room, 'Two');
    final one = _Phone(at: _garden, firstFacedDegrees: 0, pointsDegrees: 0);
    final other = _Phone(
      at: _garden.moved(bearing: 1, metres: 5),
      firstFacedDegrees: 0,
      pointsDegrees: 180,
    );

    one.see(room, first);

    final anchor = room.anchor;

    expect(
      other.placer.observe(
        room.frameFor(second),
        facing: other.facing,
        north: other.north,
      ),
      isNull,
    );
    expect(room.anchor, anchor);
  });

  test('a phone that has not found itself says so, then stops waiting', () {
    final room = _room();
    final seat = _join(room, 'One');
    final placer = RiftPlacer();

    expect(placer.stillLooking, isTrue);

    for (var frame = 0; frame < RiftPlacer.framesToFindItself; frame++) {
      expect(
        placer.observe(room.frameFor(seat), facing: 0, north: 0),
        isNull,
      );
    }

    expect(placer.stillLooking, isFalse);
    expect(placer.sighting, isNull);
    expect(room.anchor, isNull);
  });

  test('a phone with no compass tells the rift nothing', () {
    final room = _room();
    final seat = _join(room, 'One');
    final placer = RiftPlacer()..fix = _fix(_garden);

    expect(placer.observe(room.frameFor(seat), facing: 0, north: null), isNull);
    expect(placer.sighting, isNull);
  });

  test('it tells the rift again only every half second', () {
    // A rift that never takes a place: nobody is listening on the other end.
    final frame = _room()
      ..join(const RiftEntrant(name: 'One', loadout: HuntLoadout.borrowed));
    final placer = RiftPlacer()..fix = _fix(_garden);
    var told = 0;

    for (var seen = 0; seen < 3 * RiftPlacer.tellEveryFrames; seen++) {
      if (placer.observe(frame.frameFor(0), facing: 0, north: 0) != null) {
        told++;
      }
    }

    expect(told, 3);
  });

  test('a fix that wanders moves the sighting gently', () {
    final room = _room();
    final seat = _join(room, 'One');
    final phone = _Phone(at: _garden, firstFacedDegrees: 0, pointsDegrees: 0);

    phone
      ..see(room, seat)
      ..see(room, seat);

    final before = phone.placer.sighting!.bearing;

    // The phone now thinks it is five metres to the east.
    phone.placer.fix = _fix(_garden.moved(bearing: radians(90), metres: 5));
    phone.see(room, seat);

    final after = phone.placer.sighting!.bearing;
    final whole = RiftSighting.of(
      anchor: room.anchor!,
      standing: phone.placer.fix!.at,
      north: phone.north,
    )!.bearing;

    expect(wrapAngle(whole - before).abs(), greaterThan(radians(5)));
    expect(
      wrapAngle(after - before).abs(),
      closeTo(wrapAngle(whole - before).abs() * RiftPlacer.easing, 1e-6),
    );
  });

  test('a rift in another town is seen dead ahead', () {
    final room = _room()..anchor = RiftSite.londonBridge.location;
    final seat = _join(room, 'One');
    final phone = _Phone(at: _garden, firstFacedDegrees: 0, pointsDegrees: 0);

    phone.see(room, seat);

    expect(phone.placer.sighting, isNull);
  });
}
