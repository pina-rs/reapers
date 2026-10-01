import 'dart:convert';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';

Matcher refuses(Refusal refusal) => throwsA(
  isA<RuleBreach>().having((breach) => breach.refusal, 'refusal', refusal),
);

void main() {
  final now = DateTime.utc(2026, 10, 1, 12);
  final later = now.add(const Duration(minutes: 1));
  const saint = Nft(collection: NftCollection.tinSaints, serial: 1);
  const absorbed = HuntOutcome.absorbed(
    grade: ReapGrade.clean,
    ambushed: false,
    ticks: 1800,
  );

  PlayerState oathbound() => PlayerState.initial().takeOath(seed: 11);

  group('the oath', () {
    test('a new player has no reaper and plays with the borrowed one', () {
      final state = PlayerState.initial();

      expect(state.oathTaken, isFalse);
      expect(state.genome, ReaperGenome.borrowed);
      expect(state.loadout, same(HuntLoadout.borrowed));
      expect(state.level, 1);
    });

    test('binds the reaper the seed draws and arms it for its class', () {
      final state = oathbound();
      final genome = ReaperGenome.shuffle(11);

      expect(state.genome, genome);
      expect(state.weapon, Weapon(genome.reaperClass.weapon, Temper.shrouded));
      expect(state.weapons, {state.weapon});
      expect(state.loadout.reaperClass, genome.reaperClass);
    });

    test('is taken once', () {
      expect(
        () => oathbound().takeOath(seed: 12),
        refuses(Refusal.oathAlreadyTaken),
      );
    });

    test('is needed before the market opens', () {
      expect(
        () => PlayerState.initial().acquire(saint),
        refuses(Refusal.oathRequired),
      );
    });
  });

  group('the vault', () {
    test('buying an NFT spends SOL and holds it', () {
      final state = oathbound().acquire(saint);

      expect(state.vault, [saint]);
      expect(
        state.lamports,
        PlayerState.startingLamports - saint.rarity.lamports,
      );
    });

    test('an NFT is held once', () {
      expect(
        () => oathbound().acquire(saint).acquire(saint),
        refuses(Refusal.alreadyHeld),
      );
    });

    test('a wallet that cannot cover the price buys nothing', () {
      final short = PlayerState.fromJson({
        ...oathbound().toJson(),
        'lamports': saint.rarity.lamports - 1,
      });

      expect(() => short.acquire(saint), refuses(Refusal.insufficientFunds));
    });

    test('the starting wallet covers the whole stand-in market', () {
      var state = oathbound();

      for (final nft in marketListings) {
        state = state.acquire(nft);
      }

      expect(state.vault, hasLength(marketListings.length));
      expect(state.lamports, greaterThanOrEqualTo(0));
    });

    test('every listing has a price and an image', () {
      expect(marketListings, hasLength(32));

      for (final nft in marketListings) {
        expect(nft.rarity.lamports, greaterThan(0), reason: nft.id);
        expect(Nft.parse(nft.id), nft);
        expect(nft.mockMint, hasLength(44));
      }
    });
  });

  group('releasing', () {
    test('moves an NFT from the vault into the veil', () {
      final state = oathbound()
          .acquire(saint)
          .release(saint, seed: 5, now: now);
      final sighting = state.sightings.single;

      expect(state.vault, isEmpty);
      expect(sighting.nft, saint);
      expect(sighting.hasSurfaced(now), isFalse);
      expect(
        sighting.surfacesAt.difference(now).inSeconds,
        inInclusiveRange(
          PlayerState.surfaceMinSeconds,
          PlayerState.surfaceMaxSeconds,
        ),
      );
    });

    test('needs the NFT to be in the vault', () {
      expect(
        () => oathbound().release(saint, seed: 5, now: now),
        refuses(Refusal.notInVault),
      );
    });

    test('the veil holds only so many husks', () {
      var state = oathbound();

      for (var serial = 1; serial <= PlayerState.maxSightings + 1; serial++) {
        state = state.acquire(
          Nft(collection: NftCollection.tinSaints, serial: serial),
        );
      }

      for (var serial = 1; serial <= PlayerState.maxSightings; serial++) {
        state = state.release(
          Nft(collection: NftCollection.tinSaints, serial: serial),
          seed: serial,
          now: now,
        );
      }

      expect(
        () => state.release(
          const Nft(collection: NftCollection.tinSaints, serial: 4),
          seed: 9,
          now: now,
        ),
        refuses(Refusal.veilCrowded),
      );
      expect({for (final sighting in state.sightings) sighting.id}, {1, 2, 3});
    });

    test('a husk cannot be hunted before it surfaces', () {
      final state = oathbound()
          .acquire(saint)
          .release(saint, seed: 5, now: now);

      expect(() => state.huntable(1, now: now), refuses(Refusal.notSurfaced));
      expect(state.huntable(1, now: later).nft, saint);
      expect(
        () => state.huntable(2, now: later),
        refuses(Refusal.unknownSighting),
      );
    });
  });

  group('settling a hunt', () {
    PlayerState released() =>
        oathbound().acquire(saint).release(saint, seed: 5, now: now);

    test('an absorbed husk pays out and enters the codex', () {
      final state = released().settleHunt(1, absorbed, now: later);

      expect(state.sightings, isEmpty);
      expect(state.codex.single.nft, saint);
      expect(state.codex.single.grade, ReapGrade.clean);
      expect(state.xp, (Rarity.common.xp * 1.25).round());
      expect(state.anima, (Rarity.common.anima * 1.25).round());
      expect(state.holds(saint), isTrue);
    });

    test('an ambush and a perfect reap both pay more', () {
      const best = HuntOutcome.absorbed(
        grade: ReapGrade.perfect,
        ambushed: true,
        ticks: 1800,
      );

      expect(
        best.xpFor(Rarity.rare),
        (Rarity.rare.xp * 1.5 * HuntRules.ambushReward).round(),
      );
      expect(best.xpFor(Rarity.rare), greaterThan(absorbed.xpFor(Rarity.rare)));
    });

    test('an escaped husk stays loose and pays nothing', () {
      const escaped = HuntOutcome.escaped(
        escape: EscapeCause.time,
        ambushed: false,
        ticks: 4000,
      );
      final state = released().settleHunt(1, escaped, now: later);

      expect(state.sightings, hasLength(1));
      expect(state.codex, isEmpty);
      expect(state.xp, 0);
      expect(state.huntsLost, 1);
    });

    test('an absorbed NFT cannot be bought again', () {
      final state = released().settleHunt(1, absorbed, now: later);

      expect(() => state.acquire(saint), refuses(Refusal.alreadyHeld));
    });

    test('three of a collection complete its set and strengthen strikes', () {
      var state = oathbound();

      for (var serial = 1; serial <= NftCollection.setSize; serial++) {
        final nft = Nft(collection: NftCollection.tinSaints, serial: serial);

        state = state
            .acquire(nft)
            .release(nft, seed: serial, now: now)
            .settleHunt(state.nextSightingId, absorbed, now: later);
      }

      expect(state.completedSets, 1);
      expect(state.loadout.completedSets, 1);
      expect(
        HuntStats.derive(state.loadout).strikeDamage,
        greaterThan(
          HuntStats.derive(
            HuntLoadout(
              reaperClass: state.genome.reaperClass,
              level: state.level,
              weapon: state.weapon!,
            ),
          ).strikeDamage,
        ),
      );
    });
  });

  group('judging a hunt from its log', () {
    PlayerState released() =>
        oathbound().acquire(saint).release(saint, seed: 5, now: now);

    /// Plays the hunt for sighting 1 with the bot and returns its log.
    HuntLog played(PlayerState state) {
      final sighting = state.sightings.single;
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

    test('a played hunt is replayed and settled', () {
      final state = released();
      final settlement = state.judgeHunt(1, played(state), now: later);

      expect(settlement.outcome.absorbed, isTrue);
      expect(settlement.state.codex.single.nft, saint);
      expect(settlement.state.xp, settlement.xp);
      expect(settlement.xp, greaterThan(0));
    });

    test('the lenient replay the server runs agrees with the phone', () {
      final state = released();
      final log = played(state);
      final strict = state.judgeHunt(1, log, now: later);
      final lenient = state.judgeHunt(1, log, now: later, leniency: 1.03);

      expect(lenient.outcome.absorbed, strict.outcome.absorbed);
    });

    test('a log that stops before the hunt ends is refused', () {
      final state = released();
      final log = played(state);
      final cut = HuntLog(log.inputs.sublist(0, log.inputs.length ~/ 3));

      expect(
        () => state.judgeHunt(1, cut, now: later),
        refuses(Refusal.huntUnproven),
      );
    });

    test('an empty log wins nothing', () {
      expect(
        () => released().judgeHunt(1, HuntLog(const []), now: later),
        refuses(Refusal.huntUnproven),
      );
    });

    test('a log too long to be one hunt is refused before it is replayed', () {
      const still = HuntInput(aim: Aim.ahead, turnSpeed: 0);

      expect(
        () => released().judgeHunt(
          1,
          HuntLog(List.filled(HuntLog.maxTicks + 1, still)),
          now: later,
        ),
        refuses(Refusal.huntUnproven),
      );
    });
  });

  group('the armory', () {
    PlayerState rich({int level = 30}) => PlayerState.fromJson({
      ...oathbound().toJson(),
      'anima': 100000,
      'xp': Progression.xpForLevel(level),
    });

    test('a better temper costs anima and can be wielded', () {
      final start = rich();
      final weapon = Weapon(start.weapon!.kind, Temper.sworn);
      final state = start.buyWeapon(weapon).wield(weapon);

      expect(state.weapon, weapon);
      expect(state.anima, start.anima - Temper.sworn.price);
      expect(
        HuntStats.derive(state.loadout).strikeDamage,
        greaterThan(HuntStats.derive(start.loadout).strikeDamage),
      );
    });

    test("another class's weapon can be bought, at a price", () {
      final start = rich();
      final other = WeaponKind.values.firstWhere(
        (kind) => kind != start.genome.reaperClass.weapon,
      );
      final state = start.buyWeapon(Weapon(other, Temper.shrouded));

      expect(state.anima, start.anima - Weapon.crossClassPrice);
    });

    test('refuses what the reaper cannot afford, reach or already has', () {
      final poor = oathbound();
      final sworn = Weapon(poor.weapon!.kind, Temper.sworn);

      expect(() => poor.buyWeapon(sworn), refuses(Refusal.levelTooLow));
      expect(
        () => PlayerState.fromJson({
          ...poor.toJson(),
          'xp': Progression.xpForLevel(Temper.sworn.levelRequired),
        }).buyWeapon(sworn),
        refuses(Refusal.insufficientAnima),
      );
      expect(
        () => rich().buyWeapon(poor.weapon!),
        refuses(Refusal.alreadyOwned),
      );
      expect(() => poor.wield(sworn), refuses(Refusal.notOwned));
    });

    test('outfits are bought in the same way and only change the look', () {
      final start = rich();
      final state = start.buyOutfit(Outfit.scorched).wear(Outfit.scorched);

      expect(state.outfit, Outfit.scorched);
      expect(state.anima, start.anima - Outfit.scorched.price);
      expect(
        HuntStats.derive(state.loadout).strikeDamage,
        HuntStats.derive(start.loadout).strikeDamage,
      );
      expect(() => start.wear(Outfit.hollowed), refuses(Refusal.notOwned));
      expect(
        () => rich(level: 2).buyOutfit(Outfit.hollowed),
        refuses(Refusal.levelTooLow),
      );
    });

    test('every temper and outfit costs more than the one before', () {
      for (final values in [
        [for (final temper in Temper.values) temper.price],
        [for (final temper in Temper.values) temper.levelRequired],
        [for (final outfit in Outfit.values) outfit.price],
        [for (final outfit in Outfit.values) outfit.levelRequired],
      ]) {
        expect(values, [...values]..sort());
      }
    });
  });

  group('json', () {
    test('a played state survives a round trip through text', () {
      final state = oathbound()
          .acquire(saint)
          .release(saint, seed: 5, now: now)
          .settleHunt(1, absorbed, now: later)
          .acquire(const Nft(collection: NftCollection.brineHounds, serial: 8));
      final restored = PlayerState.fromJson(
        (jsonDecode(jsonEncode(state.toJson())) as Map).cast<String, dynamic>(),
      );

      expect(jsonEncode(restored.toJson()), jsonEncode(state.toJson()));
      expect(restored.genome, state.genome);
      expect(restored.codex.single.at, later);
    });

    test('an unknown schema is refused rather than guessed at', () {
      expect(
        () => PlayerState.fromJson({
          ...PlayerState.initial().toJson(),
          'schema': 2,
        }),
        throwsFormatException,
      );
    });
  });
}
