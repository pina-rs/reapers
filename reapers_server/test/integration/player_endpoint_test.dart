import 'dart:io';
import 'dart:typed_data';

import 'package:reapers_server/src/generated/protocol.dart';
import 'package:reapers_server/src/receipts/absorption_receipt.dart';
import 'package:reapers_shared/reapers_shared.dart';
import 'package:serverpod/serverpod.dart' show UuidValue;
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
  const saint = 'degenApes:1';

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

    GeoFix measured({
      double accuracy = 4,
      GeoPoint at = const GeoPoint(51.5, -.08),
      Duration age = Duration.zero,
    }) => GeoFix(
      at: at,
      accuracy: accuracy,
      observedAt: DateTime.now().toUtc().subtract(age),
    );

    test(
      'location belongs to the signed-in player and survives book changes',
      () async {
        await endpoints.player.load(player);
        final fix = measured();
        await endpoints.player.updateLocation(player, fix);
        await endpoints.player.takeOath(player);
        expect((await endpoints.player.location(player))!.at, fix.at);
        expect(await endpoints.player.location(other), isNull);
        await endpoints.player.updateLocation(player, null);
        expect(await endpoints.player.location(player), isNull);
        expect((await endpoints.player.load(player)).oathTaken, isTrue);
      },
    );

    test(
      'location rejects coarse, malformed, stale and future readings',
      () async {
        for (final fix in [
          measured(accuracy: 80),
          measured(accuracy: 0),
          measured(at: const GeoPoint(95, 0)),
          measured(age: const Duration(seconds: 31)),
          measured(age: const Duration(seconds: -10)),
        ]) {
          await expectLater(
            endpoints.player.updateLocation(player, fix),
            refuses(Refusal.locationUnusable),
          );
        }
        expect(await endpoints.player.location(player), isNull);
      },
    );

    test('out-of-order uploads cannot replace a newer location', () async {
      final older = measured(age: const Duration(seconds: 2));
      final newer = measured(at: const GeoPoint(51.501, -.08));
      await endpoints.player.updateLocation(player, newer);
      await endpoints.player.updateLocation(player, older);
      expect((await endpoints.player.location(player))!.at, newer.at);
    });

    test('an expired stored fix is not returned', () async {
      await endpoints.player.load(player);
      final session = sessionBuilder.build();
      final record = (await PlayerRecord.db.find(session)).single;
      await PlayerRecord.db.updateRow(
        session,
        record.copyWith(location: measured(age: const Duration(minutes: 1))),
      );
      expect(await endpoints.player.location(player), isNull);
    });

    test('location read, write and clear all require authentication', () async {
      final nobody = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.unauthenticated(),
      );
      await expectLater(
        endpoints.player.location(nobody),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
      await expectLater(
        endpoints.player.updateLocation(nobody, measured()),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
      await expectLater(
        endpoints.player.updateLocation(nobody, null),
        throwsA(isA<ServerpodUnauthenticatedException>()),
      );
    });

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

        for (final id in ['', 'nonsense', 'degenApes:99', 'noSuchSet:1']) {
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
        expect(
          stored.activeReaper!.memory.toJson(),
          const SoulMemory().remember(settlement.outcome).toJson(),
        );
      },
    );

    test(
      'old combat rules receive an update refusal without changing growth',
      () async {
        final state = await withLooseHusk();
        final bytes = play(state, state.sightings.single).encode()..[0] = 4;
        await expectLater(
          endpoints.player.settleHunt(
            player,
            state.sightings.single.id,
            ByteData.sublistView(bytes),
          ),
          refuses(Refusal.huntVersionUnsupported),
        );
        expect((await endpoints.player.load(player)).toJson(), state.toJson());
      },
    );

    test(
      'when a husk is absorbed then the server rolls the feeding and signs it',
      () async {
        final before = await withLooseHusk();
        final sighting = before.sightings.single;
        final settlement = await endpoints.player.settleHunt(
          player,
          sighting.id,
          packed(play(before, sighting)),
        );
        final entry = settlement.state.codex.single;
        final feeding = settlement.feeding!;

        expect(entry.feeding, feeding);
        expect(Feeding.roll(entry.nonce), feeding, reason: 'rolled, not sent');
        expect(
          settlement.xp,
          settlement.outcome.xpFor(sighting.nft.rarity, feeding),
        );
        expect(entry.xp, settlement.xp);

        final signer = await ReceiptSigner.fromSecret(
          ReceiptSigner.developmentSecret,
        );
        final message = await AbsorptionReceipt.message(
          playerId: playerId,
          absorption: entry,
        );

        expect(entry.signature, isNotNull);
        expect(await signer.verifies(message, entry.signature!), isTrue);
        expect(
          (await endpoints.player.load(player)).codex.single.signature,
          entry.signature,
          reason: 'the receipt is kept with the books',
        );

        // What is signed is the whole of what was decided. Change any of it
        // and the signature no longer holds.
        for (final (label, forged) in [
          (
            'a richer feeding',
            Absorption(
              nft: entry.nft,
              grade: entry.grade,
              feeding: Feeding.feast,
              xp: entry.xp * 2,
              anima: entry.anima,
              nonce: entry.nonce,
              at: entry.at,
            ),
          ),
          (
            'another NFT',
            Absorption(
              nft: const Nft(collection: NftCollection.degenApes, serial: 2),
              grade: entry.grade,
              feeding: entry.feeding,
              xp: entry.xp,
              anima: entry.anima,
              nonce: entry.nonce,
              at: entry.at,
            ),
          ),
        ]) {
          expect(
            await signer.verifies(
              await AbsorptionReceipt.message(
                playerId: playerId,
                absorption: forged,
              ),
              entry.signature!,
            ),
            isFalse,
            reason: label,
          );
        }

        expect(
          await signer.verifies(
            await AbsorptionReceipt.message(
              playerId: otherId,
              absorption: entry,
            ),
            entry.signature!,
          ),
          isFalse,
          reason: 'a receipt belongs to one player',
        );
      },
    );

    test('when asked then the server gives the key receipts are checked '
        'against', () async {
      final signer = await ReceiptSigner.fromSecret(
        ReceiptSigner.developmentSecret,
      );
      final key = await endpoints.player.receiptKey(player);

      expect(key, hasLength(64));
      expect(
        key,
        [
          for (final byte in signer.publicKey)
            byte.toRadixString(16).padLeft(2, '0'),
        ].join(),
      );
    });

    test('a receipt is laid out as the documentation says', () async {
      final at = DateTime.utc(2026, 10, 1, 12);
      final message = await AbsorptionReceipt.message(
        playerId: playerId,
        absorption: Absorption(
          nft: const Nft(collection: NftCollection.thugbirdz, serial: 3),
          grade: ReapGrade.perfect,
          feeding: Feeding.rich,
          xp: 0x01020304,
          anima: 77,
          nonce: 0xCAFEF00D,
          at: at,
        ),
      );
      final data = ByteData.sublistView(message);

      expect(message, hasLength(AbsorptionReceipt.length));
      expect(String.fromCharCodes(message.sublist(0, 8)), 'RPRSABS1');
      expect(message.sublist(8, 40), isNot(everyElement(0)));
      expect(message.sublist(40, 72), isNot(message.sublist(8, 40)));
      expect(data.getUint8(72), ReapGrade.perfect.index);
      expect(data.getUint8(73), Feeding.rich.index);
      expect(message.sublist(74, 78), [4, 3, 2, 1], reason: 'little-endian');
      expect(data.getUint32(78, Endian.little), 77);
      expect(
        data.getInt64(82, Endian.little),
        at.millisecondsSinceEpoch ~/ 1000,
      );
      expect(data.getUint64(90, Endian.little), 0xCAFEF00D);
    });

    test('a different secret signs with a different key', () async {
      final ours = await ReceiptSigner.fromSecret(
        ReceiptSigner.developmentSecret,
      );
      final theirs = await ReceiptSigner.fromSecret('some other secret');
      final message = Uint8List.fromList(List.filled(98, 7));

      expect(theirs.publicKey, isNot(ours.publicKey));
      expect(await ours.verifies(message, await theirs.sign(message)), isFalse);
      expect(await ours.verifies(message, await ours.sign(message)), isTrue);
    });

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
        (
          'played in a corner too small to hunt in',
          packed(
            HuntLog(
              whole.inputs,
              plan: const HuntPlan(
                centreCentidegrees: 0,
                halfWidthCentidegrees: 1500,
              ),
            ),
          ),
        ),
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

  withServerpod(
    'Concurrent game and location requests',
    rollbackDatabase: RollbackDatabase.disabled,
    (sessionBuilder, endpoints) {
      const id = 'dcc079e3-0056-4a1b-8b12-8a8bf4aa0e93';
      final player = sessionBuilder.copyWith(
        authentication: AuthenticationOverride.authenticationInfo(id, const {}),
      );
      setUp(() => endpoints.player.takeOath(player));
      tearDown(
        () => PlayerRecord.db.deleteWhere(
          sessionBuilder.build(),
          where: (table) => table.authUserId.equals(UuidValue.fromString(id)),
        ),
      );

      test(
        'location uploads preserve purchases arriving at the same time',
        () async {
          final fix = GeoFix(
            at: const GeoPoint(51.5, -.08),
            accuracy: 4,
            observedAt: DateTime.now().toUtc(),
          );
          await Future.wait<Object?>([
            endpoints.player.acquire(player, saint),
            endpoints.player.updateLocation(player, fix),
          ]);
          expect((await endpoints.player.load(player)).vault.single.id, saint);
          expect((await endpoints.player.location(player))!.at, fix.at);
        },
      );
    },
  );
}
