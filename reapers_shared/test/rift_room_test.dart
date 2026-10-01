import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/hunt_bot.dart';
import 'support/rift_table.dart';

/// A person who fights well but never carries the cut through.
HuntBot _noCut(HuntSim sim, int person) =>
    HuntBot(sim, reactionTicks: 16, missEvery: 7, reapSpeed: 0);

/// A person who never strikes, but makes the cut when the ward breaks.
HuntBot _watcher(HuntSim sim) => HuntBot(sim, reactionTicks: 1 << 30);

/// The ticks at which [seat]'s husk began to shriek while [bot] played it.
List<int> _shriekTicks(RiftSeat seat, HuntBot bot, {required int ticks}) {
  final shrieks = <int>[];

  for (var tick = 0; tick < ticks; tick++) {
    final events = seat.step(bot.decide());

    if (events.any((event) => event is ShriekBegan)) {
      shrieks.add(tick);
    }
  }

  return shrieks;
}

void main() {
  group('gathering', () {
    test('the count starts once everyone present stands ready', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
      final first = room.join(entrant(name: 'Vesper'));
      final second = room.join(entrant(name: 'Morrow'));

      room
        ..report(first, const RiftReport(ready: true))
        ..advance(30);

      expect(room.frameFor(first).countingDown, isFalse);

      room
        ..report(second, const RiftReport(ready: true))
        ..advance(1);

      final frame = room.frameFor(first);

      expect(frame.countingDown, isTrue);
      expect(frame.ticksLeft, RiftRules.countdownTicks);
      expect(frame.phase, RiftPhase.gathering);
    });

    test('a reaper leaving stops the count', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
      final first = room.join(entrant());
      final second = room.join(entrant());

      room
        ..report(first, const RiftReport(ready: true))
        ..report(second, const RiftReport(ready: true))
        ..advance(10)
        ..leave(second);

      expect(room.frameFor(first).countingDown, isFalse);
      expect(room.frameFor(first).members, hasLength(1));
    });

    test('a rift does not wait forever on a reaper who is not ready', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3)
        ..join(entrant());

      room.advance(RiftRules.gatherLimitTicks + RiftRules.countdownTicks + 1);

      expect(room.phase, RiftPhase.fight);
    });

    test('a full circle and a rift already begun take nobody else', () {
      final full = RiftRoom(site: RiftSite.londonBridge, seed: 3);

      for (var seat = 0; seat < RiftRules.circle; seat++) {
        full.join(entrant());
      }

      expect(full.isOpen, isFalse);
      expect(() => full.join(entrant()), throwsStateError);

      final begun = RiftTable(reapers: [entrant()])..begin();

      expect(begun.room.isOpen, isFalse);
    });

    test('a rift everyone walked away from is abandoned', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);
      final seat = room.join(entrant());

      expect(room.isAbandoned, isFalse);

      room.leave(seat);

      expect(room.isAbandoned, isTrue);
    });
  });

  group('the circle', () {
    test('shades fill the seats nobody took', () {
      final table = RiftTable(reapers: [entrant(level: 6)])..begin();
      final frame = table.room.frameFor(table.seats.single);

      expect(frame.members, hasLength(RiftRules.circle));
      expect(frame.you.shade, isFalse);
      expect(
        frame.members.where((member) => member.shade),
        hasLength(RiftRules.circle - 1),
      );
      expect(
        frame.members.where((member) => member.shade).map((m) => m.level),
        everyElement(6 - RiftRules.shadeLevelGap),
      );
    });

    test('the same seed always sends the same shades', () {
      List<String> shades() {
        final table = RiftTable(reapers: [entrant()], seed: 21)..begin();

        return [
          for (final member in table.room.frameFor(0).members)
            '${member.name}/${member.reaperClass.code}',
        ];
      }

      expect(shades(), shades());
    });

    test('the ward is sized to the circle, not to a fixed number', () {
      final novices = RiftTable(
        reapers: [for (var i = 0; i < 4; i++) entrant()],
      )..begin();
      final veterans = RiftTable(
        reapers: [for (var i = 0; i < 4; i++) entrant(level: 20)],
      )..begin();

      expect(veterans.room.wardMax, greaterThan(novices.room.wardMax * 2));
    });
  });

  group('one fight, shared', () {
    test(
      'there is nothing to stalk: the husk is ahead and the fight is on',
      () {
        final seat = RiftSeat(
          seed: 5,
          site: RiftSite.londonBridge,
          wardMax: 1000,
          loadout: entrant().loadout,
        );

        expect(seat.sim.phase, HuntPhase.fight);
        expect(seat.sim.pose.yaw.abs(), lessThan(radians(1)));
      },
    );

    test('everyone hears the shriek at the same moment', () {
      RiftSeat seat(ReaperClass reaperClass) => RiftSeat(
        seed: 5,
        site: RiftSite.londonBridge,
        wardMax: 1e9,
        loadout: entrant(reaperClass: reaperClass).loadout,
      );

      final striker = seat(ReaperClass.harvester);
      final watcher = seat(ReaperClass.mourner);
      const ticks = 40 * HuntRules.tickRate;

      final struck = _shriekTicks(
        striker,
        HuntBot.human(striker.sim),
        ticks: ticks,
      );
      final watched = _shriekTicks(
        watcher,
        _watcher(watcher.sim),
        ticks: ticks,
      );

      expect(struck.length, greaterThanOrEqualTo(3));
      expect(striker.dealt, greaterThan(0));
      expect(watcher.dealt, 0);
      expect(struck, watched);
    });

    test('a strike anyone lands comes off the ward everyone sees', () {
      final table = RiftTable(reapers: [entrant(), entrant()])..begin();

      for (var tick = 0; tick < 12 * HuntRules.tickRate; tick++) {
        table.step();
      }

      final frame = table.room.frameFor(table.seats.first);
      var dealt = 0.0;

      for (final member in frame.members) {
        dealt += member.dealt;
      }

      expect(dealt, greaterThan(0));
      expect(frame.ward, closeTo(frame.wardMax - dealt, 1e-6));
    });

    test('the ward breaks for a reaper who never landed a blow', () {
      final table = RiftTable(
        reapers: [for (var person = 0; person < 4; person++) entrant()],
        play: (sim, person) => person == 0 ? _watcher(sim) : HuntBot.human(sim),
      )..play();

      expect(table.room.phase, RiftPhase.sealed);
      expect(table.room.frameFor(0).you.dealt, 0);
      expect(table.room.frameFor(0).you.cut, isNotNull);
    });

    test('a report can be repeated or arrive late without being recounted', () {
      final table = RiftTable(reapers: [entrant()])..begin();
      final seat = table.seats.single;

      table.room
        ..report(seat, const RiftReport(ready: true, dealt: 40))
        ..report(seat, const RiftReport(ready: true, dealt: 40))
        ..report(seat, const RiftReport(ready: true, dealt: 25));

      expect(table.room.frameFor(seat).you.dealt, 40);
    });

    test('a seat shows its own strikes before the room has heard of them', () {
      final seat = RiftSeat(
        seed: 5,
        site: RiftSite.londonBridge,
        wardMax: 1000,
        loadout: entrant().loadout,
      )..dealt = 60;

      // The room has counted 20 of this seat's 60 and others dealt 100.
      seat.sync(ward: 1000 - 100 - 20, counted: 20);

      expect(seat.sim.ward, 1000 - 100 - 60);
    });

    test('a lone hunt has no shared ward', () {
      expect(() => simFor().shareWard(10), throwsStateError);
    });
  });

  group('sealing', () {
    test('one careful reaper and three shades seal it with time to spare', () {
      final table = RiftTable(reapers: [entrant(level: 4)])..play();
      final frame = table.room.frameFor(0);

      expect(table.room.phase, RiftPhase.sealed);
      expect(table.room.tick, lessThan(RiftRules.fightLimitTicks));
      expect(frame.cuts, greaterThanOrEqualTo(frame.cutsNeeded));
      expect(frame.xp, greaterThan(0));
      expect(frame.anima, greaterThan(0));
    });

    test('four reapers break the ward sooner than one with shades', () {
      final alone = RiftTable(reapers: [entrant(level: 4)])..play();
      final together = RiftTable(
        reapers: [
          for (final reaperClass in ReaperClass.values.take(4))
            entrant(reaperClass: reaperClass, level: 4),
        ],
      )..play();

      expect(together.room.phase, RiftPhase.sealed);
      expect(
        together.room.tick,
        lessThan(alone.room.tick - 10 * HuntRules.tickRate),
      );
    });

    test('shades cannot shut a rift without a cut from the living', () {
      final table = RiftTable(reapers: [entrant(level: 4)], play: _noCut)
        ..play();
      final frame = table.room.frameFor(0);

      expect(frame.cuts, greaterThanOrEqualTo(frame.cutsNeeded));
      expect(table.room.phase, RiftPhase.failed);
      expect(frame.xp, 0);
    });

    test('half the circle, rounded up, has to cut', () {
      expect(RiftFrame.cutsNeededFor(4), 2);
      expect(RiftFrame.cutsNeededFor(3), 2);
      expect(RiftFrame.cutsNeededFor(2), 1);
      expect(RiftFrame.cutsNeededFor(1), 1);
      expect(RiftFrame.cutsNeededFor(0), 1);
    });

    test('a ward nobody breaks in time leaves the rift open', () {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);

      for (var seat = 0; seat < RiftRules.circle; seat++) {
        room
          ..join(entrant())
          ..report(seat, const RiftReport(ready: true));
      }

      room.advance(RiftRules.countdownTicks + RiftRules.fightLimitTicks + 2);

      expect(room.phase, RiftPhase.failed);
    });

    test('a circle everyone walked out of is left to its shades', () {
      final table = RiftTable(reapers: [entrant(), entrant()])..begin();

      for (final seat in table.seats) {
        table.room.leave(seat);
      }

      table.room.advance(1);

      // The shades fight on for nobody, so the room is free to be dropped.
      expect(table.room.isAbandoned, isTrue);
    });
  });

  group('what it pays', () {
    RiftRoom sealedWith({required RiftReport report}) {
      final room = RiftRoom(site: RiftSite.londonBridge, seed: 3);

      for (var seat = 0; seat < RiftRules.circle; seat++) {
        room
          ..join(entrant())
          ..report(seat, const RiftReport(ready: true));
      }

      room.advance(RiftRules.countdownTicks + 1);

      final share = room.wardMax / RiftRules.circle;
      const cut = HuntOutcome.absorbed(
        grade: ReapGrade.clean,
        ambushed: false,
        ticks: 1,
      );

      room.report(
        0,
        RiftReport(
          ready: true,
          dealt: report.dealt * share,
          outcome: report.outcome,
        ),
      );

      for (var seat = 1; seat < RiftRules.circle; seat++) {
        room.report(seat, RiftReport(ready: true, dealt: share, outcome: cut));
      }

      // The first seat may have dealt less than its share; the rest make up
      // the difference so the ward still breaks.
      room
        ..report(1, RiftReport(ready: true, dealt: share * 2, outcome: cut))
        ..advance(RiftRules.reapWindowTicks + 120);

      expect(room.phase, RiftPhase.sealed);

      return room;
    }

    const clean = HuntOutcome.absorbed(
      grade: ReapGrade.clean,
      ambushed: false,
      ticks: 1,
    );
    const perfect = HuntOutcome.absorbed(
      grade: ReapGrade.perfect,
      ambushed: false,
      ticks: 1,
    );
    const castOut = HuntOutcome.escaped(
      escape: EscapeCause.resolve,
      ambushed: false,
      ticks: 1,
    );
    const missed = HuntOutcome.escaped(
      escape: EscapeCause.reap,
      ambushed: false,
      ticks: 1,
    );

    test('a full share and a clean cut earn the stated reward', () {
      final room = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: clean),
      );

      expect(room.rewardFor(0), (
        xp: (RiftRules.xp * ReapGrade.clean.reward).round(),
        anima: (RiftRules.anima * ReapGrade.clean.reward).round(),
      ));
    });

    test('a better cut earns more', () {
      final better = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: perfect),
      );
      final worse = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: clean),
      );

      expect(better.rewardFor(0).xp, greaterThan(worse.rewardFor(0).xp));
    });

    test('pulling less than a fair share earns less, but never nothing', () {
      final idle = sealedWith(
        report: const RiftReport(ready: true, outcome: clean),
      );

      expect(
        idle.rewardFor(0).xp,
        (RiftRules.xp * RiftRules.idleShare * ReapGrade.clean.reward).round(),
      );
    });

    test('pulling more than a fair share earns no more than the whole', () {
      final eager = sealedWith(
        report: const RiftReport(ready: true, dealt: 1.5, outcome: clean),
      );

      expect(
        eager.rewardFor(0).xp,
        (RiftRules.xp * ReapGrade.clean.reward).round(),
      );
    });

    test('missing the cut and being cast out each cost a share', () {
      final uncut = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: missed),
      );
      final out = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: castOut),
      );

      expect(
        uncut.rewardFor(0).xp,
        (RiftRules.xp * RiftRules.uncutShare).round(),
      );
      expect(
        out.rewardFor(0).xp,
        (RiftRules.xp * RiftRules.castOutShare).round(),
      );
      expect(out.frameFor(0).you.standing, isFalse);
    });

    test('a reaper who walked away before the end is paid nothing', () {
      final table = RiftTable(
        reapers: [for (var person = 0; person < 4; person++) entrant()],
      )..begin();

      for (var tick = 0; tick < 10 * HuntRules.tickRate; tick++) {
        table.step();
      }

      table.walkAway(3);

      while (!table.room.phase.isOver) {
        table.step();
      }

      expect(table.room.phase, RiftPhase.sealed);
      expect(table.room.rewardFor(table.seats.last), (xp: 0, anima: 0));
      expect(table.room.rewardFor(table.seats.first).xp, greaterThan(0));
    });

    test('leaving once it is over changes nothing', () {
      final room = sealedWith(
        report: const RiftReport(ready: true, dealt: 1, outcome: clean),
      );
      final paid = room.rewardFor(0);

      room.leave(0);

      expect(room.rewardFor(0), paid);
      expect(room.frameFor(0).members, hasLength(RiftRules.circle));
    });
  });

  group('frames and reports', () {
    test('a frame survives being sent', () {
      final table = RiftTable(reapers: [entrant()])..play();
      final frame = table.room.frameFor(0);
      final copy = RiftFrame.fromJson(frame.toJson());

      expect(copy.toJson(), frame.toJson());
      expect(copy.phase, RiftPhase.sealed);
      expect(copy.you.cut, frame.you.cut);
    });

    test('a report survives being sent, with and without an outcome', () {
      const fighting = RiftReport(ready: true, dealt: 12.5);
      const done = RiftReport(
        ready: true,
        dealt: 80,
        outcome: HuntOutcome.absorbed(
          grade: ReapGrade.perfect,
          ambushed: false,
          ticks: 900,
        ),
      );

      expect(
        RiftReport.fromJson(fighting.toJson()).toJson(),
        fighting.toJson(),
      );
      expect(
        RiftReport.fromJson(done.toJson()).outcome!.grade,
        ReapGrade.perfect,
      );
    });
  });

  group('attunement', () {
    final now = DateTime.utc(2026, 10, 1);

    PlayerState bound() => PlayerState.initial().takeOath(seed: 4);

    PlayerState absorbed(PlayerState player, Nft nft) {
      final released = player.acquire(nft).release(nft, seed: 9, now: now);
      final sighting = released.sightings.single;

      return released.settleHunt(
        sighting.id,
        const HuntOutcome.absorbed(
          grade: ReapGrade.clean,
          ambushed: false,
          ticks: 600,
        ),
        now: sighting.surfacesAt,
        feedingSeed: 1,
      );
    }

    test('a rift stays shut to a reaper who carries none of its dead', () {
      final player = bound();

      expect(player.attunedTo(RiftSite.londonBridge), isFalse);
      expect(
        () => player.entrantFor(RiftSite.londonBridge),
        throwsA(
          isA<RuleBreach>().having(
            (breach) => breach.refusal,
            'refusal',
            Refusal.notAttuned,
          ),
        ),
      );
    });

    test('absorbing one of its collection opens it, and only it', () {
      final player = absorbed(
        bound(),
        const Nft(collection: NftCollection.degenApes, serial: 1),
      );

      expect(player.attunedTo(RiftSite.londonBridge), isTrue);
      expect(player.attunedTo(RiftSite.shibuyaCrossing), isFalse);
      expect(player.entrantFor(RiftSite.londonBridge).name, player.genome.name);
    });

    test('nobody enters a rift before the oath', () {
      expect(
        () => PlayerState.initial().entrantFor(RiftSite.londonBridge),
        throwsA(
          isA<RuleBreach>().having(
            (breach) => breach.refusal,
            'refusal',
            Refusal.oathRequired,
          ),
        ),
      );
    });

    test('sealing a rift is paid into the ledger', () {
      final player = bound().settleRift(xp: 320, anima: 140);

      expect(player.xp, 320);
      expect(player.anima, 140);
      expect(player.riftsSealed, 1);
    });
  });
}
