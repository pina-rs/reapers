import 'dart:convert';

import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

void main() {
  final now = DateTime.utc(2026, 10, 5);
  final husk = Nft.parse('degenApes:1');
  const cut = HuntOutcome.absorbed(
    grade: ReapGrade.clean,
    ambushed: false,
    ticks: 100,
  );
  PlayerState pair() => PlayerState.initial()
      .takeOath(seed: 11)
      .bindReaper(
        ReaperState.bound(
          id: 'verified-asset',
          genome: ReaperGenome.shuffle(22),
        ),
      );
  Matcher refused(Refusal value) => throwsA(
    isA<RuleBreach>().having((error) => error.refusal, 'refusal', value),
  );

  test('hunt experience and codex belong only to the reaper that hunted', () {
    final original = pair();
    final hunted = original
        .selectReaper('verified-asset')
        .acquire(husk)
        .release(husk, seed: 8, now: now)
        .settleHunt(
          1,
          cut,
          now: now.add(const Duration(minutes: 1)),
          feedingSeed: 3,
        );
    final starter = hunted.selectReaper(ReaperState.oathId);
    expect(hunted.xp, greaterThan(0));
    expect(hunted.codex.single.nft, husk);
    expect(starter.xp, 0);
    expect(starter.codex, isEmpty);
    expect(starter.anima, hunted.anima);
    expect(starter.holds(husk), isTrue);
    expect(() => starter.acquire(husk), refused(Refusal.alreadyHeld));
  });

  test('cannot switch away from loose hunts or select an unowned reaper', () {
    final loose = pair().acquire(husk).release(husk, seed: 1, now: now);
    expect(
      () => loose.selectReaper('verified-asset'),
      refused(Refusal.reaperBusy),
    );
    expect(
      loose.selectReaper(ReaperState.oathId).activeReaperId,
      ReaperState.oathId,
    );
    expect(() => pair().selectReaper('not-mine'), refused(Refusal.notOwned));
  });

  test('tradable reapers cannot earn growth from unverified rifts', () {
    expect(
      () => pair()
          .selectReaper('verified-asset')
          .entrantFor(RiftSite.values.first),
      refused(Refusal.riftUnproven),
    );
  });

  test('roster capacity is enforced by the shared rule', () {
    var player = PlayerState.initial().takeOath(seed: 11);
    for (var index = 0; index < PlayerState.maxNftReapers; index++) {
      player = player.bindReaper(
        ReaperState.bound(
          id: 'asset-$index',
          genome: ReaperGenome.shuffle(index),
        ),
      );
    }
    expect(
      () => player.bindReaper(
        ReaperState.bound(id: 'overflow', genome: ReaperGenome.shuffle(80)),
      ),
      refused(Refusal.rosterFull),
    );
  });

  test('reaper progression is immutable and equipped gear must be owned', () {
    final reaper = pair().reapers.first;
    expect(() => reaper.weapons.clear(), throwsUnsupportedError);
    expect(() => reaper.outfits.clear(), throwsUnsupportedError);
    expect(() => reaper.codex.clear(), throwsUnsupportedError);
    expect(() => reaper.copyWith(xp: -1), throwsFormatException);
    expect(
      () => reaper.copyWith(outfit: Outfit.hollowed),
      throwsFormatException,
    );
  });

  test('rift settlement credits the entrant even after selection changes', () {
    final result = pair()
        .selectReaper('verified-asset')
        .settleRift(xp: 400, anima: 40, reaperId: ReaperState.oathId);
    expect(result.xp, 0);
    expect(result.riftsSealed, 0);
    final entrant = result.selectReaper(ReaperState.oathId);
    expect(entrant.xp, 400);
    expect(entrant.riftsSealed, 1);
    expect(entrant.anima, 40);
  });

  test('losing ownership returns unfinished husks without growth', () {
    final owned = pair()
        .selectReaper('verified-asset')
        .acquire(husk)
        .release(husk, seed: 1, now: now);
    final refreshed = owned.reconcileReapers([]);
    expect(refreshed.activeReaperId, ReaperState.oathId);
    expect(refreshed.vault, [husk]);
    expect(refreshed.sightings, isEmpty);
    expect(refreshed.xp, 0);
    expect(refreshed.codex, isEmpty);
  });

  test(
    'NFT equipment and experience survive roster serialization independently',
    () {
      final original = pair()
          .selectReaper('verified-asset')
          .settleRift(xp: Progression.xpForLevel(20), anima: 10000)
          .buyOutfit(Outfit.scorched)
          .wear(Outfit.scorched);
      final restored = PlayerState.fromJson(
        (jsonDecode(jsonEncode(original.toJson())) as Map)
            .cast<String, dynamic>(),
      );
      expect(restored.toJson(), original.toJson());
      expect(restored.selectReaper(ReaperState.oathId).outfit, Outfit.shrouded);
      expect(restored.outfit, Outfit.scorched);
    },
  );

  test(
    'schema two preserves a grown reaper, equipment, currency and codex',
    () {
      final grown = pair()
          .acquire(husk)
          .release(husk, seed: 1, now: now)
          .settleHunt(
            1,
            cut,
            now: now.add(const Duration(minutes: 1)),
            feedingSeed: 5,
          );
      final json = grown.toJson();
      final legacy = {...json, ...grown.activeReaper!.toJson(), 'schema': 2}
        ..remove('reapers')
        ..remove('activeReaperId')
        ..remove('id');
      final restored = PlayerState.fromJson(legacy);
      expect(restored.reapers.length, 1);
      expect(restored.genome, grown.genome);
      expect(restored.xp, grown.xp);
      expect(restored.weapon, grown.weapon);
      expect(restored.codex.single.toJson(), grown.codex.single.toJson());
      expect(restored.lamports, grown.lamports);
      expect(restored.anima, grown.anima);
      expect(restored.toJson()['schema'], 3);
    },
  );

  test('unbound legacy save still opens without inventing a reaper', () {
    final legacy = {
      ...PlayerState.initial().toJson(),
      'schema': 2,
      'genome': null,
    };
    expect(PlayerState.fromJson(legacy).oathTaken, isFalse);
  });

  test('duplicate seats and an invalid selection cannot be read', () {
    final json = pair().toJson();
    expect(
      () => PlayerState.fromJson({...json, 'activeReaperId': 'absent'}),
      throwsFormatException,
    );
    expect(
      () => PlayerState.fromJson({
        ...json,
        'reapers': [
          pair().reapers.first.toJson(),
          pair().reapers.first.toJson(),
        ],
      }),
      throwsFormatException,
    );
  });

  test(
    'preview birth trait edits keep names valid and reject out-of-range parts',
    () {
      final original = ReaperGenome.shuffle(11);
      for (final calling in ReaperClass.values) {
        final tried = original.copyWith(
          reaperClass: calling,
          head: 2,
          tatter: 1000,
        );
        expect(tried.name, isNotEmpty);
        expect(ReaperGenome.parse(tried.code), tried);
      }
      expect(() => original.copyWith(head: 9), throwsFormatException);
      expect(original, ReaperGenome.shuffle(11));
    },
  );
}
