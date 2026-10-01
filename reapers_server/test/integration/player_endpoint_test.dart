import 'dart:io';
import 'dart:typed_data';

import 'package:reapers_server/src/generated/protocol.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'test_tools/serverpod_test_tools.dart';

Matcher refuses(Refusal refusal) => throwsA(
  isA<Refused>().having((refused) => refused.refusal, 'refusal', refusal.name),
);

ByteData packed(HuntLog log) => ByteData.sublistView(log.encode());

void main() {
  // The server reads its secrets from `config/passwords.yaml` in the test run
  // mode. That file is git-ignored, so a fresh clone has none and the suite
  // would otherwise sit waiting on a database it can never reach. Fail with
  // the fix instead.
  if (!File('config/passwords.yaml').existsSync()) {
    fail(
      'Missing config/passwords.yaml, which the server needs in the test run '
      'mode.\n\n'
      'Create it from the committed example:\n'
      '    cp config/passwords.example.yaml config/passwords.yaml',
    );
  }

  const playerId = '550e8400-e29b-41d4-a716-446655440000';
  const otherId = '6ba7b810-9dad-11d1-80b4-00c04fd430c8';
  const saint = 'tinSaints:1';

  withServerpod('Given a signed-in player', (sessionBuilder, endpoints) {
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

    /// Moves every loose husk's surfacing time into the past, so a test does
    /// not have to wait out the real delay.
    Future<void> surfaceEverything() async {
      final session = sessionBuilder.build();
      final record = (await PlayerRecord.db.find(session)).single;
      final json = record.state.toJson();

      json['sightings'] = [
        for (final sighting in json['sightings'] as List)
          {...sighting as Map, 'surfacesAt': '2026-01-01T00:00:00.000Z'},
      ];

      await PlayerRecord.db.updateRow(
        session,
        record.copyWith(state: PlayerState.fromJson(json)),
      );
    }

    /// Plays the hunt for [sighting] with the shared bot, as [state] would.
    HuntLog play(PlayerState state, Sighting sighting) {
      final bot = HuntBot(
        HuntSim(
          seed: sighting.seed,
          vessel: sighting.nft.collection.vessel,
          ward: sighting.nft.rarity.ward,
          stats: HuntStats.derive(state.loadout),
        ),
      )..play();

      return HuntLog(bot.recorded);
    }

    /// An oathbound player with [saint] released and surfaced.
    Future<PlayerState> withLooseHusk() async {
      await endpoints.player.takeOath(player);
      await endpoints.player.acquire(player, saint);
      await endpoints.player.release(player, saint);
      await surfaceEverything();

      return endpoints.player.load(player);
    }

    test('when nobody is signed in then every call is rejected', () async {
      final nobody = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.unauthenticated(),
      );

      await expectLater(
        endpoints.player.load(nobody),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
      await expectLater(
        endpoints.player.takeOath(nobody),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
    });

    test('when loading for the first time then the books are opened', () async {
      final state = await endpoints.player.load(player);

      expect(state.oathTaken, isFalse);
      expect(state.lamports, PlayerState.startingLamports);
      expect(
        await PlayerRecord.db.count(sessionBuilder.build()),
        1,
        reason: 'loading twice must not open a second set of books',
      );

      await endpoints.player.load(player);

      expect(await PlayerRecord.db.count(sessionBuilder.build()), 1);
    });

    test('when taking the oath then the server draws the reaper', () async {
      final state = await endpoints.player.takeOath(player);

      expect(state.oathTaken, isTrue);
      expect(state.weapon!.kind, state.genome.reaperClass.weapon);
      expect(
        (await endpoints.player.load(player)).genomeCode,
        state.genomeCode,
      );
      await expectLater(
        endpoints.player.takeOath(player),
        refuses(Refusal.oathAlreadyTaken),
      );
    });

    test('when two players act then each has their own books', () async {
      await endpoints.player.takeOath(player);

      expect((await endpoints.player.load(other)).oathTaken, isFalse);
      expect((await endpoints.player.load(player)).oathTaken, isTrue);
    });

    test(
      'when buying an NFT then the wallet pays and the vault holds it',
      () async {
        await endpoints.player.takeOath(player);

        final state = await endpoints.player.acquire(player, saint);

        expect(state.vault.single.id, saint);
        expect(
          state.lamports,
          PlayerState.startingLamports - Rarity.common.lamports,
        );
      },
    );

    test(
      'when the game sends an NFT that does not exist then it is refused',
      () async {
        await endpoints.player.takeOath(player);

        for (final id in ['', 'nonsense', 'tinSaints:99', 'noSuchSet:1']) {
          await expectLater(
            endpoints.player.acquire(player, id),
            refuses(Refusal.notListed),
            reason: id,
          );
        }
      },
    );

    test(
      'when releasing then the server picks the seed and the delay',
      () async {
        await endpoints.player.takeOath(player);
        await endpoints.player.acquire(player, saint);

        final state = await endpoints.player.release(player, saint);
        final sighting = state.sightings.single;

        expect(state.vault, isEmpty);
        expect(sighting.surfacesAt.isAfter(DateTime.now().toUtc()), isTrue);
        await expectLater(
          endpoints.player.settleHunt(
            player,
            sighting.id,
            packed(play(state, sighting)),
          ),
          refuses(Refusal.notSurfaced),
        );
      },
    );

    test(
      'when a played hunt is submitted then the replay absorbs the husk',
      () async {
        final before = await withLooseHusk();
        final sighting = before.sightings.single;
        final settlement = await endpoints.player.settleHunt(
          player,
          sighting.id,
          packed(play(before, sighting)),
        );

        expect(settlement.outcome.absorbed, isTrue);
        expect(settlement.xp, greaterThan(0));
        expect(settlement.state.codex.single.nft.id, saint);
        expect(settlement.state.sightings, isEmpty);

        final stored = await endpoints.player.load(player);

        expect(stored.xp, settlement.state.xp);
        expect(stored.codex.single.nft.id, saint);
      },
    );

    test(
      'when the same hunt is submitted twice then it only pays once',
      () async {
        final before = await withLooseHusk();
        final sighting = before.sightings.single;
        final log = packed(play(before, sighting));

        await endpoints.player.settleHunt(player, sighting.id, log);

        await expectLater(
          endpoints.player.settleHunt(player, sighting.id, log),
          refuses(Refusal.unknownSighting),
        );
      },
    );

    test('when the log is not a finished hunt then nothing is paid', () async {
      final before = await withLooseHusk();
      final sighting = before.sightings.single;
      final whole = play(before, sighting);
      final cut = HuntLog(whole.inputs.sublist(0, whole.inputs.length ~/ 2));

      for (final (label, log) in [
        ('empty', ByteData(0)),
        ('garbage', ByteData.sublistView(Uint8List.fromList([9, 9, 9, 9]))),
        ('cut short', packed(cut)),
      ]) {
        await expectLater(
          endpoints.player.settleHunt(player, sighting.id, log),
          refuses(Refusal.huntUnproven),
          reason: label,
        );
      }

      final after = await endpoints.player.load(player);

      expect(after.xp, 0);
      expect(after.sightings, hasLength(1));
    });

    test(
      'when a log from another hunt is submitted then it wins nothing',
      () async {
        final before = await withLooseHusk();
        final sighting = before.sightings.single;

        // The same play, recorded against a husk that hid somewhere else.
        final elsewhere = Sighting(
          id: sighting.id,
          nft: sighting.nft,
          seed: sighting.seed + 1,
          surfacesAt: sighting.surfacesAt,
        );

        Object? refusal;
        HuntSettlement? settlement;

        try {
          settlement = await endpoints.player.settleHunt(
            player,
            sighting.id,
            packed(play(before, elsewhere)),
          );
        } on Refused catch (refused) {
          refusal = refused.refusal;
        }

        // Either the replay never finishes, or it finishes with the husk
        // escaping. It must never absorb.
        expect(settlement?.outcome.absorbed ?? false, isFalse);
        expect(refusal ?? Refusal.huntUnproven.name, Refusal.huntUnproven.name);
        expect((await endpoints.player.load(player)).xp, 0);
      },
    );

    test(
      'when the armory is asked for what cannot be had then it refuses',
      () async {
        final state = await endpoints.player.takeOath(player);
        final sworn = Weapon(state.weapon!.kind, Temper.sworn);

        await expectLater(
          endpoints.player.buyWeapon(player, sworn.id),
          refuses(Refusal.levelTooLow),
        );
        await expectLater(
          endpoints.player.wield(player, sworn.id),
          refuses(Refusal.notOwned),
        );
        await expectLater(
          endpoints.player.wear(player, 'no-such-outfit'),
          refuses(Refusal.notOwned),
        );
        await expectLater(
          endpoints.player.buyWeapon(player, 'not a weapon'),
          refuses(Refusal.notOwned),
        );
        expect(
          (await endpoints.player.wear(player, Outfit.shrouded.name)).outfit,
          Outfit.shrouded,
        );
      },
    );
  });
}
