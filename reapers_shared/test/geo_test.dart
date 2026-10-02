import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

/// London Bridge, where one of the rifts is.
const _bridge = GeoPoint(51.5079, -0.0877);

GeoStanding _standing(GeoPoint at, {double facingDegrees = 0}) =>
    GeoStanding(at: at, facing: radians(facingDegrees), accuracy: 4);

void main() {
  group('places on the earth', () {
    test('a degree of latitude is about 111 km', () {
      expect(
        const GeoPoint(51, 0).distanceTo(const GeoPoint(52, 0)),
        closeTo(111195, 60),
      );
    });

    test('a place is no distance from itself', () {
      expect(_bridge.distanceTo(_bridge), 0);
    });

    test('bearings run clockwise from north', () {
      expect(
        degrees(_bridge.bearingTo(const GeoPoint(51.6, -0.0877))),
        closeTo(0, 1e-6),
      );
      expect(
        degrees(_bridge.bearingTo(const GeoPoint(51.5079, 0.1))),
        closeTo(90, 0.2),
      );
      expect(
        degrees(_bridge.bearingTo(const GeoPoint(51.4, -0.0877))).abs(),
        closeTo(180, 1e-6),
      );
      expect(
        degrees(_bridge.bearingTo(const GeoPoint(51.5079, -0.3))),
        closeTo(-90, 0.2),
      );
    });

    test('moving somewhere and measuring back agree', () {
      for (final bearing in [0.0, 37.0, 90.0, 180.0, -120.0]) {
        for (final metres in [5.0, 25.0, 140.0, 2500.0]) {
          final there = _bridge.moved(
            bearing: radians(bearing),
            metres: metres,
          );

          expect(
            _bridge.distanceTo(there),
            closeTo(metres, metres * 1e-6 + 1e-6),
          );
          expect(
            wrapAngle(_bridge.bearingTo(there) - radians(bearing)).abs(),
            lessThan(1e-6),
            reason: '$metres m at $bearing°',
          );
        }
      }
    });

    test('a place survives being written down', () {
      final written = _standing(_bridge, facingDegrees: 33).toJson();
      final read = GeoStanding.fromJson(written);

      expect(read.at, _bridge);
      expect(read.facing, closeTo(radians(33), 1e-12));
      expect(read.accuracy, 4);
    });
  });

  group('where a rift stands', () {
    test('at its site, for a reaper who has come to it', () {
      final nearby = _bridge.moved(bearing: 1, metres: 120);

      expect(RiftSite.londonBridge.anchorFor(_standing(nearby)), _bridge);
    });

    test('in front of the first reaper, anywhere else', () {
      const garden = GeoPoint(52.2, 0.12);
      final anchor = RiftSite.londonBridge.anchorFor(
        _standing(garden, facingDegrees: 70),
      );

      expect(garden.distanceTo(anchor), closeTo(RiftRules.plantMetres, 1e-3));
      expect(degrees(garden.bearingTo(anchor)), closeTo(70, 1e-3));
    });

    test(
      'the first reaper to say where they are decides it for the circle',
      () {
        const garden = GeoPoint(52.2, 0.12);
        final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
        final first = room.join(
          const RiftEntrant(name: 'One', loadout: HuntLoadout.borrowed),
        );
        final second = room.join(
          const RiftEntrant(name: 'Two', loadout: HuntLoadout.borrowed),
        );

        expect(room.frameFor(first).anchor, isNull);

        room.report(
          second,
          RiftReport(
            ready: false,
            standing: _standing(garden, facingDegrees: 70),
          ),
        );

        final anchor = room.anchor!;

        // Someone else, standing a few metres off and facing another way,
        // does not move it.
        room.report(
          first,
          RiftReport(
            ready: false,
            standing: _standing(
              garden.moved(bearing: 2, metres: 6),
              facingDegrees: -140,
            ),
          ),
        );

        expect(room.anchor, anchor);
        expect(room.frameFor(first).anchor, anchor);
        expect(room.frameFor(second).anchor, anchor);
      },
    );

    test('it does not move once the fight has begun', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
      final seat = room.join(
        const RiftEntrant(name: 'One', loadout: HuntLoadout.borrowed),
      );

      room
        ..report(seat, const RiftReport(ready: true))
        ..advance(RiftRules.countdownTicks + 2);

      expect(room.phase, RiftPhase.fight);

      room.report(
        seat,
        RiftReport(
          ready: true,
          standing: _standing(const GeoPoint(52.2, 0.12)),
        ),
      );

      expect(room.anchor, isNull);
    });

    test('a frame carries it across the wire', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
      final seat = room.join(
        const RiftEntrant(name: 'One', loadout: HuntLoadout.borrowed),
      );

      room.report(
        seat,
        RiftReport(
          ready: false,
          standing: _standing(const GeoPoint(52.2, 0.12)),
        ),
      );

      final sent = RiftFrame.fromJson(room.frameFor(seat).toJson());
      final report = RiftReport.fromJson(
        RiftReport(
          ready: true,
          standing: _standing(const GeoPoint(52.2, 0.12), facingDegrees: 12),
        ).toJson(),
      );

      expect(sent.anchor, room.anchor);
      expect(report.standing!.at, const GeoPoint(52.2, 0.12));
      expect(
        RiftReport.fromJson(const RiftReport(ready: true).toJson()).standing,
        isNull,
      );
    });
  });

  group('where a reaper sees it', () {
    const garden = GeoPoint(52.2, 0.12);
    final anchor = garden.moved(bearing: radians(70), metres: 25);

    test('the one who planted it sees it where they were pointing', () {
      // Their phone pointed 70° east of north, which in their own view was
      // straight ahead: so north is 70° to their left.
      final sighting = RiftSighting.of(
        anchor: anchor,
        standing: garden,
        north: radians(-70),
      )!;

      expect(degrees(sighting.bearing), closeTo(0, 1e-3));
      expect(sighting.metres, closeTo(25, 1e-3));
    });

    test('two reapers side by side point at the same place', () {
      final beside = garden.moved(bearing: radians(160), metres: 3);

      // Each has north somewhere different in their own view, because each
      // started facing a different way.
      for (final north in [-70.0, 15.0, 170.0]) {
        final one = RiftSighting.of(
          anchor: anchor,
          standing: garden,
          north: radians(north),
        )!;
        final other = RiftSighting.of(
          anchor: anchor,
          standing: beside,
          north: radians(north),
        )!;

        // On the earth, the two directions meet at the rift.
        final meet = garden.moved(
          bearing: one.bearing - radians(north),
          metres: 25,
        );
        final meetOther = beside.moved(
          bearing: other.bearing - radians(north),
          metres: beside.distanceTo(anchor),
        );

        expect(meet.distanceTo(anchor), lessThan(0.01));
        expect(meetOther.distanceTo(anchor), lessThan(0.01));
      }
    });

    test('from across the rift it lies the other way', () {
      final beyond = garden.moved(bearing: radians(70), metres: 50);
      final sighting = RiftSighting.of(
        anchor: anchor,
        standing: beyond,
        north: 0,
      )!;

      expect(degrees(sighting.bearing), closeTo(-110, 0.01));
    });

    test('it is never drawn too near or too far to fight', () {
      final close = anchor.moved(bearing: 0, metres: 3);
      final far = anchor.moved(bearing: 0, metres: 120);

      expect(
        RiftSighting.of(anchor: anchor, standing: close, north: 0)!.metres,
        RiftRules.nearestMetres,
      );
      expect(
        RiftSighting.of(anchor: anchor, standing: far, north: 0)!.metres,
        RiftRules.farthestMetres,
      );
    });

    test('a reaper in another town is not there at all', () {
      expect(
        RiftSighting.of(anchor: anchor, standing: _bridge, north: 0),
        isNull,
      );
    });

    test('the husk stands where it is sighted, facing the reaper', () {
      final sim = HuntSim.rift(
        seed: 3,
        vessel: Vessel.reliquaryStack,
        ward: 1000,
        stats: HuntStats.derive(HuntLoadout.borrowed),
        sighting: RiftSighting(bearing: radians(40), metres: 30),
      );
      final flat = math.sqrt(
        math.pow(sim.pose.position.x, 2) + math.pow(sim.pose.position.z, 2),
      );

      expect(degrees(sim.pose.yaw), closeTo(40, 1e-9));
      expect(flat, closeTo(30, 1e-9));
      expect(sim.pose.facesPlayer, isTrue);
    });

    test('with no sighting it stands dead ahead, as before', () {
      final sim = HuntSim.rift(
        seed: 3,
        vessel: Vessel.reliquaryStack,
        ward: 1000,
        stats: HuntStats.derive(HuntLoadout.borrowed),
      );

      expect(sim.pose.yaw, 0);
      expect(sim.pose.position.z, closeTo(17, 1e-9));
    });
  });
}
