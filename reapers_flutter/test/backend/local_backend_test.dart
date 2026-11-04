import 'dart:math';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:reapers_flutter/src/backend/local_backend.dart';
import 'package:reapers_flutter/src/backend/rift_session.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/rift_reaper.dart';

Matcher refuses(Refusal refusal) => throwsA(
  isA<RuleBreach>().having((breach) => breach.refusal, 'refusal', refusal),
);

const _saint = Nft(collection: NftCollection.degenApes, serial: 1);

void main() {
  late SharedPreferences preferences;
  late DateTime now;

  LocalBackend keeper() =>
      LocalBackend(preferences, clock: () => now, random: Random(5));

  /// Plays the hunt for the one loose husk with the shared bot and hands the
  /// record to [backend].
  Future<HuntSettlement> hunt(LocalBackend backend) async {
    final state = await backend.load();
    final sighting = state.sightings.single;
    final bot = HuntBot(
      HuntSim(
        seed: sighting.seed,
        vessel: sighting.nft.collection.vessel,
        ward: sighting.nft.rarity.ward,
        stats: HuntStats.derive(state.loadout),
      ),
    )..play();

    return backend.settleHunt(sighting.id, HuntLog(bot.recorded));
  }

  /// An oathbound player who has absorbed one Tin Saint.
  Future<LocalBackend> attuned() async {
    final backend = keeper();

    await backend.takeOath();
    await backend.acquire(_saint);
    await backend.release(_saint);
    now = now.add(const Duration(minutes: 1));

    expect((await hunt(backend)).outcome.absorbed, isTrue);

    return backend;
  }

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    preferences = await SharedPreferences.getInstance();
    now = DateTime.utc(2026, 10, 1, 12);
  });

  test(
    'a first launch finds an unbound player with a stocked wallet',
    () async {
      final state = await keeper().load();

      expect(state.oathTaken, isFalse);
      expect(state.lamports, PlayerState.startingLamports);
    },
  );

  test('everything the player does is still there after a restart', () async {
    final backend = keeper();
    final bound = await backend.takeOath();

    await backend.acquire(_saint);

    final reopened = await keeper().load();

    expect(reopened.genomeCode, bound.genomeCode);
    expect(reopened.vault, [_saint]);
    expect(reopened.toJson(), (await backend.load()).toJson());
  });

  test('a refusal changes nothing that was stored', () async {
    final backend = keeper();

    await backend.takeOath();

    final before = preferences.getString(LocalBackend.defaultStorageKey);

    await expectLater(backend.release(_saint), refuses(Refusal.notInVault));

    expect(preferences.getString(LocalBackend.defaultStorageKey), before);
  });

  test(
    'a save from a format this build does not know is not misread',
    () async {
      await preferences.setString(
        LocalBackend.defaultStorageKey,
        '{"schema": 99}',
      );

      await expectLater(keeper().load(), throwsFormatException);
    },
  );

  test('a husk cannot be hunted before it surfaces', () async {
    final backend = keeper();

    await backend.takeOath();
    await backend.acquire(_saint);
    await backend.release(_saint);

    await expectLater(hunt(backend), refuses(Refusal.notSurfaced));
  });

  test('an absorbed husk pays and is written in the codex', () async {
    final backend = await attuned();
    final state = await backend.load();

    expect(state.codex.single.nft, _saint);
    expect(state.sightings, isEmpty);
    expect(state.xp, greaterThan(0));
    expect(state.anima, greaterThan(0));
  });

  group('rifts', () {
    test('a rift is shut to a reaper who carries none of its dead', () async {
      final backend = keeper();

      await backend.takeOath();

      await expectLater(
        backend.enterRift(RiftSite.londonBridge),
        refuses(Refusal.notAttuned),
      );
    });

    test('a sealed rift pays once, and the books are settled before the '
        'last frame', () async {
      final backend = await attuned();
      final before = await backend.load();

      fakeAsync((async) {
        var paid = 0;
        late final RiftReaper reaper;
        final room = RiftRoom(site: RiftSite.londonBridge, seed: 11);
        final session = LocalRiftSession(
          room: room,
          seat: room.join(before.entrantFor(RiftSite.londonBridge)),
          microseconds: () => async.elapsed.inMicroseconds,
          onSealed: (reward) async {
            paid++;

            // The reaper has not been shown the end yet.
            expect(reaper.last.phase.isOver, isFalse);
            expect(reward.xp, greaterThan(0));
          },
        );

        reaper = RiftReaper(session, before.loadout);
        async.elapse(const Duration(seconds: 140));

        expect(reaper.done, isTrue);
        expect(reaper.last.phase, RiftPhase.sealed);
        expect(reaper.last.members, hasLength(RiftRules.circle));
        expect(reaper.last.you.cut, isNotNull);
        expect(reaper.last.xp, greaterThan(0));
        expect(paid, 1);
      });
    });

    test(
      'a rift the reaper never cuts is not sealed and pays nothing',
      () async {
        final backend = await attuned();
        final before = await backend.load();

        fakeAsync((async) {
          var paid = 0;
          final room = RiftRoom(site: RiftSite.londonBridge, seed: 11);
          final reaper = RiftReaper(
            LocalRiftSession(
              room: room,
              seat: room.join(before.entrantFor(RiftSite.londonBridge)),
              microseconds: () => async.elapsed.inMicroseconds,
              onSealed: (_) async => paid++,
            ),
            before.loadout,
            cuts: false,
          );

          async.elapse(const Duration(seconds: 140));

          expect(reaper.done, isTrue);
          expect(reaper.last.phase, RiftPhase.failed);
          expect(reaper.last.xp, 0);
          expect(paid, 0);
        });
      },
    );

    test('leaving stops the rift and closes the stream', () async {
      final backend = await attuned();
      final before = await backend.load();

      fakeAsync((async) {
        final room = RiftRoom(site: RiftSite.londonBridge, seed: 11);
        final session = LocalRiftSession(
          room: room,
          seat: room.join(before.entrantFor(RiftSite.londonBridge)),
          microseconds: () => async.elapsed.inMicroseconds,
          onSealed: (_) async => fail('an abandoned rift pays nobody'),
        );
        final reaper = RiftReaper(session, before.loadout);

        async.elapse(const Duration(seconds: 10));
        session.leave();
        async.flushMicrotasks();

        final frames = reaper.frames.length;

        async.elapse(const Duration(seconds: 200));

        expect(reaper.done, isTrue);
        expect(reaper.frames, hasLength(frames));
        expect(room.isAbandoned, isTrue);
        expect(async.periodicTimerCount, 0);
      });
    });

    test('a phone that slept does not wake to a rift that ran without it', () {
      fakeAsync((async) {
        var micros = 0;
        final room = RiftRoom(site: RiftSite.londonBridge, seed: 11);
        final seat = room.join(
          RiftEntrant(name: 'Vesper', loadout: HuntLoadout.borrowed),
        );
        final session = LocalRiftSession(
          room: room,
          seat: seat,
          microseconds: () => micros,
          onSealed: (_) async {},
        );
        final frames = <RiftFrame>[];

        session
          ..frames.listen(frames.add)
          ..report(const RiftReport(ready: true));

        // Ten minutes pass between two beats.
        micros = const Duration(minutes: 10).inMicroseconds;
        async.elapse(const Duration(milliseconds: 60));

        expect(frames.last.phase, RiftPhase.gathering);

        session.leave();
        async.flushMicrotasks();
      });
    });

    test('the keeper pays a sealed rift into the stored books', () async {
      final before = await (await attuned()).load();

      fakeAsync((async) {
        final opened = now;
        final backend = LocalBackend(
          preferences,
          clock: () => opened.add(async.elapsed),
          random: Random(5),
        );
        late final RiftReaper reaper;
        late final PlayerState after;

        backend
            .enterRift(RiftSite.londonBridge)
            .then((session) => reaper = RiftReaper(session, before.loadout));
        async
          ..flushMicrotasks()
          ..elapse(const Duration(seconds: 140));
        backend.load().then((state) => after = state);
        async.flushMicrotasks();

        expect(reaper.last.phase, RiftPhase.sealed);
        expect(after.xp, before.xp + reaper.last.xp);
        expect(after.anima, before.anima + reaper.last.anima);
        expect(after.riftsSealed, 1);
      });
    });

    test('a rift walked out of leaves the books as they were', () async {
      final backend = await attuned();
      final before = await backend.load();
      final session = await backend.enterRift(RiftSite.londonBridge);

      session.leave();

      expect((await backend.load()).toJson(), before.toJson());
    });
  });
}
