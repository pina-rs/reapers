import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:reapers_server/src/generated/protocol.dart';
import 'package:reapers_server/src/rift/rift_endpoint.dart';
import 'package:reapers_server/src/rift/rift_hall.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart' show UuidValue;
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

/// One reaper at a rift, played by the shared bot: it stands ready, keeps
/// its own fight in step with the frames the server sends, and reports how
/// that fight stands.
final class _Reaper {
  _Reaper(this.loadout);

  final HuntLoadout loadout;
  final reports = StreamController<RiftReport>();
  final frames = <RiftFrame>[];

  RiftSeat? _seat;
  HuntBot? _bot;

  RiftFrame get last => frames.last;

  /// Plays the rift that [stream] carries until the server ends it.
  Future<void> play(Stream<RiftFrame> stream) async {
    reports.add(const RiftReport(ready: true));

    await for (final frame in stream) {
      frames.add(frame);

      if (frame.phase == RiftPhase.gathering || frame.phase.isOver) {
        continue;
      }

      final seat = _seat ??= RiftSeat(
        seed: frame.seed,
        site: frame.site,
        wardMax: frame.wardMax,
        loadout: loadout,
      );
      final bot = _bot ??= HuntBot.human(seat.sim);

      while (seat.sim.tick < frame.tick && !seat.sim.isOver) {
        seat
          ..sync(ward: frame.ward, counted: frame.you.dealt)
          ..step(bot.decide());
      }

      reports.add(seat.report);
    }

    await reports.close();
  }
}

/// Takes [who] through the oath and one absorbed Tin Saint, which is what
/// opens the London Bridge rift to them.
Future<PlayerState> _attune(
  TestSessionBuilder sessionBuilder,
  TestEndpoints endpoints,
  TestSessionBuilder who,
  String id,
) async {
  const saint = 'degenApes:1';

  await endpoints.player.takeOath(who);
  await endpoints.player.acquire(who, saint);

  final released = await endpoints.player.release(who, saint);
  final session = sessionBuilder.build();
  final record = (await PlayerRecord.db.find(
    session,
    where: (table) => table.authUserId.equals(UuidValue.fromString(id)),
  )).single;
  final json = record.state.toJson();

  // Surface the husk now instead of waiting out the real delay.
  json['sightings'] = [
    for (final sighting in json['sightings'] as List)
      {...sighting as Map, 'surfacesAt': '2026-01-01T00:00:00.000Z'},
  ];
  await PlayerRecord.db.updateRow(
    session,
    record.copyWith(state: PlayerState.fromJson(json)),
  );

  final sighting = released.sightings.single;
  final bot = HuntBot(
    HuntSim(
      seed: sighting.seed,
      vessel: sighting.nft.collection.vessel,
      ward: sighting.nft.rarity.ward,
      stats: HuntStats.derive(released.loadout),
    ),
  )..play();
  final settlement = await endpoints.player.settleHunt(
    who,
    sighting.id,
    ByteData.sublistView(HuntLog(bot.recorded).encode()),
  );

  expect(settlement.outcome.absorbed, isTrue);

  return settlement.state;
}

/// A hall that plays a rift through in well under a second, by moving it on
/// half a second at every beat. A real rift is two minutes long.
RiftHall _quickHall() =>
    RiftHall(beat: const Duration(milliseconds: 4), ticksPerBeat: 30);

void main() {
  if (!File('config/passwords.yaml').existsSync()) {
    fail(
      'Missing config/passwords.yaml, which the server needs in the test run '
      'mode.\n\n'
      'Create it from the committed example:\n'
      '    cp config/passwords.example.yaml config/passwords.yaml',
    );
  }

  const playerId = '550e8400-e29b-41d4-a716-446655440000';
  final site = RiftSite.londonBridge.name;

  withServerpod('Given a rift', (sessionBuilder, endpoints) {
    final player = sessionBuilder.copyWith(
      authentication: AuthenticationOverride.authenticationInfo(
        playerId,
        const {},
      ),
    );
    setUp(() => RiftEndpoint.hall = _quickHall());

    Future<PlayerState> attune(TestSessionBuilder who, String id) =>
        _attune(sessionBuilder, endpoints, who, id);

    test('when nobody is signed in then the rift is shut', () async {
      final nobody = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.unauthenticated(),
      );

      await expectLater(
        endpoints.rift.enter(nobody, site, const Stream.empty()).drain<void>(),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
    });

    test(
      'when the reaper carries none of its dead then they are refused',
      () async {
        await endpoints.player.takeOath(player);

        await expectLater(
          endpoints.rift
              .enter(player, site, const Stream.empty())
              .drain<void>(),
          throwsA(
            isA<Refused>().having(
              (refused) => refused.refusal,
              'refusal',
              Refusal.notAttuned.name,
            ),
          ),
        );
      },
    );

    test('when the site does not exist then they are refused', () async {
      await attune(player, playerId);

      await expectLater(
        endpoints.rift
            .enter(player, 'nowhere', const Stream.empty())
            .drain<void>(),
        throwsA(isA<Refused>()),
      );
    });

    test(
      'when one reaper comes then shades fill the circle and sealing pays',
      () async {
        final before = await attune(player, playerId);
        final reaper = _Reaper(before.loadout);

        await reaper.play(
          endpoints.rift.enter(player, site, reaper.reports.stream),
        );

        final first = reaper.frames.first;
        final last = reaper.last;

        expect(first.phase, RiftPhase.gathering);
        expect(first.members.single.name, before.genome.name);
        expect(last.phase, RiftPhase.sealed);
        expect(last.members, hasLength(RiftRules.circle));
        expect(last.members.where((member) => member.shade), hasLength(3));
        expect(last.you.cut, isNotNull);
        expect(last.xp, greaterThan(0));

        final after = await endpoints.player.load(player);

        expect(after.xp, before.xp + last.xp);
        expect(after.anima, before.anima + last.anima);
        expect(after.riftsSealed, 1);
      },
    );

    test(
      'when a reaper never cuts then the rift is not sealed and pays nothing',
      () async {
        final before = await attune(player, playerId);
        final reports = StreamController<RiftReport>()
          ..add(const RiftReport(ready: true));
        final frames = await endpoints.rift
            .enter(player, site, reports.stream)
            .toList();

        await reports.close();

        expect(frames.last.phase, RiftPhase.failed);
        expect(frames.last.xp, 0);

        final after = await endpoints.player.load(player);

        expect(after.xp, before.xp);
        expect(after.riftsSealed, 0);
      },
    );
  });
  // Two reapers in one rift each open and settle their own books at the same
  // moment. The test tools cannot roll back concurrent transactions, so this
  // group writes for real and clears up after itself.
  withServerpod(
    'Given two reapers at one rift',
    rollbackDatabase: RollbackDatabase.disabled,
    (sessionBuilder, endpoints) {
      // These rows are really written, so they belong to nobody any other
      // test signs in as.
      const playerId = '7c9e6679-7425-40de-944b-e07fc1f90ae7';
      const otherId = '16fd2706-8baf-433b-82eb-8c7fada847da';

      final player = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.authenticationInfo(
          playerId,
          const {},
        ),
      );
      final other = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.authenticationInfo(
          otherId,
          const {},
        ),
      );

      setUp(() => RiftEndpoint.hall = _quickHall());

      tearDown(
        () => PlayerRecord.db.deleteWhere(
          sessionBuilder.build(),
          where: (table) => table.authUserId.inSet({
            UuidValue.fromString(playerId),
            UuidValue.fromString(otherId),
          }),
        ),
      );

      Future<PlayerState> attune(TestSessionBuilder who, String id) =>
          _attune(sessionBuilder, endpoints, who, id);

      test(
        'when two reapers come then they share one circle and one ward',
        () async {
          final first = _Reaper((await attune(player, playerId)).loadout);
          final second = _Reaper((await attune(other, otherId)).loadout);

          await Future.wait([
            first.play(
              endpoints.rift.enter(player, site, first.reports.stream),
            ),
            second.play(
              endpoints.rift.enter(other, site, second.reports.stream),
            ),
          ]);

          expect(first.last.phase, RiftPhase.sealed);
          expect(second.last.phase, RiftPhase.sealed);
          expect(first.last.seed, second.last.seed);
          expect(first.last.wardMax, second.last.wardMax);
          expect(
            first.last.members.where((member) => !member.shade),
            hasLength(2),
          );
          expect(first.last.seat, isNot(second.last.seat));
          expect(first.last.you.dealt, greaterThan(0));
          expect(second.last.you.dealt, greaterThan(0));
          expect((await endpoints.player.load(player)).riftsSealed, 1);
          expect((await endpoints.player.load(other)).riftsSealed, 1);
        },
      );
    },
  );
}
