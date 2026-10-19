import 'package:reapers_shared/reapers_shared.dart';
import 'package:test/test.dart';

import 'support/rooms.dart';

HuntStats _stats({Art? art, int rank = 1, WeaponKind? weapon}) =>
    HuntStats.derive(
      HuntLoadout(
        reaperClass: ReaperClass.harvester,
        level: 6,
        weapon: Weapon(weapon ?? WeaponKind.scythe, Temper.shrouded),
        art: art == null ? null : ArtSkill(art, rank),
      ),
    );

HuntSim _sim({
  Art? art,
  int rank = 1,
  WeaponKind? weapon,
  HuntPlan plan = HuntPlan.open,
  Vessel vessel = Vessel.ape,
  int seed = 3,
}) => HuntSim(
  seed: seed,
  vessel: vessel,
  ward: Rarity.epic.ward,
  stats: _stats(art: art, rank: rank, weapon: weapon),
  plan: plan,
);

/// Steps [sim] with [input] until [until] holds, and returns what happened.
List<HuntEvent> _run(
  HuntSim sim,
  HuntInput Function(HuntSim sim) input, {
  required bool Function() until,
  int ticks = HuntLog.maxTicks,
}) {
  final events = <HuntEvent>[];
  final end = sim.tick + ticks;

  while (!until() && !sim.isOver && sim.tick < end) {
    events.addAll(sim.step(input(sim)));
  }

  return events;
}

HuntInput _steady(HuntSim sim, {bool focus = false}) => HuntInput.measured(
  aim: sim.pose.direction,
  turnRadiansPerSecond: 0,
  focus: focus,
);

HuntInput _focused(HuntSim sim) => _steady(sim, focus: true);

HuntInput _flick(HuntSim sim) => HuntInput.measured(
  aim: sim.pose.direction,
  turnRadiansPerSecond: ArtRules.throwSpeed + 0.1,
  focus: true,
);

/// Springs an ambush, so the fight starts with the husk standing still.
void _ambush(HuntSim sim) {
  _run(sim, _steady, until: () => sim.phase != HuntPhase.stalk);
  expect(sim.ambushed, isTrue);
}

/// Gathers the art steadily on the husk, then flicks it. Returns what the
/// flick did.
List<HuntEvent> _throw(HuntSim sim) {
  _run(
    sim,
    _focused,
    until: () => sim.artGathered >= 1,
    ticks: ArtRules.gatherTicks + 1,
  );

  return sim.step(_flick(sim));
}

/// Lets whatever is in the air come down, holding still.
List<HuntEvent> _land(HuntSim sim) =>
    _run(sim, _steady, until: () => sim.flights.isEmpty, ticks: 120);

/// A hunt in the kitchen whose husk has gone behind the counter before it
/// has noticed anyone, and stays there a while.
HuntSim _hiddenInKitchen({Art? art, int rank = 1}) {
  for (var seed = 1; seed < 200; seed++) {
    final sim = _sim(art: art, rank: rank, plan: kitchen(), seed: seed);

    // Looking away, so as not to spring an ambush.
    _run(
      sim,
      (sim) => const HuntInput(aim: Aim(18000, 0), turnSpeed: 0),
      until: () =>
          sim.footing == Footing.hidden && !sim.leaping && sim.speed == 0,
      ticks: 20 * HuntRules.tickRate,
    );

    final settled =
        sim.footing == Footing.hidden && !sim.leaping && sim.speed == 0;

    if (settled && sim.phase == HuntPhase.stalk) {
      return sim;
    }
  }

  fail('no husk hid in the kitchen');
}

void main() {
  group('learning an art', () {
    test('each creature carries an art, and the rift\'s carries none', () {
      for (final vessel in Vessel.values) {
        expect(Art.carriedBy(vessel), vessel.isSolo ? isNotNull : isNull);
      }

      expect(Art.carriedBy(Vessel.penguin), Art.rime);
      expect(Art.carriedBy(Vessel.ape), Art.cinder);
      expect(Art.carriedBy(Vessel.bird), Art.gale);
    });

    test('it deepens with every husk absorbed that carries it', () {
      expect(
        [
          for (var absorbed = 0; absorbed <= 11; absorbed++)
            Art.rankFor(absorbed),
        ],
        [0, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3],
      );
    });

    test('a deeper art holds more throws and readies them sooner', () {
      const first = ArtSkill(Art.rime, 1);
      const third = ArtSkill(Art.rime, 3);

      expect(third.maxCharges, greaterThan(first.maxCharges));
      expect(third.rechargeTicks, lessThan(first.rechargeTicks));
      expect(third.freezeTicks, greaterThan(first.freezeTicks));
      expect(third.shatter, greaterThan(first.shatter));
    });
  });

  group('throwing an art', () {
    test('it is gathered while the phone is steady, and a flick throws it', () {
      final sim = _sim(art: Art.cinder);

      _ambush(sim);

      // A flick before it is gathered throws nothing.
      sim.step(_focused(sim));
      expect(sim.step(_flick(sim)).whereType<ArtThrown>(), isEmpty);

      final events = _throw(sim);

      expect(events.whereType<ArtThrown>().single.art, Art.cinder);
      expect(sim.artCharges, 0);
      expect(sim.flights, hasLength(1));
    });

    test('turning while holding it does not gather it', () {
      final sim = _sim(art: Art.cinder);

      _ambush(sim);
      _run(
        sim,
        (sim) => HuntInput.measured(
          aim: sim.pose.direction,
          turnRadiansPerSecond: HuntRules.steadySpeed + 0.2,
          focus: true,
        ),
        until: () => false,
        ticks: 60,
      );

      expect(sim.artGathered, 0);
    });

    test('letting go before the flick loses nothing', () {
      final sim = _sim(art: Art.cinder);

      _ambush(sim);
      _run(sim, _focused, until: () => sim.artGathered >= 1, ticks: 60);
      sim.step(_steady(sim));

      expect(sim.artGathered, 0);
      expect(sim.artCharges, 1);
    });

    test('nothing is thrown into a shriek, and the flick is a flinch', () {
      final sim = _sim(art: Art.cinder);

      _ambush(sim);
      _run(sim, _focused, until: () => sim.phase == HuntPhase.shriek);
      _run(
        sim,
        _focused,
        until: () => false,
        ticks: HuntRules.shriekGraceTicks,
      );

      final events = sim.step(_flick(sim));

      expect(events.whereType<ArtThrown>(), isEmpty);
      expect(events.whereType<Flinched>(), hasLength(1));
    });

    test('a throw is readied again in time, and by holding firm', () {
      final sim = _sim(art: Art.rime);

      _ambush(sim);
      _throw(sim);
      expect(sim.artCharges, 0);

      _run(sim, _steady, until: () => sim.artCharges > 0, ticks: 2000);
      expect(sim.artCharges, 1);

      final held = _sim(art: Art.rime);

      _ambush(held);
      _throw(held);
      _run(held, _steady, until: () => held.phase == HuntPhase.shriek);
      _run(held, _steady, until: () => false, ticks: HuntRules.shriekTicks - 1);

      final before = held.artCharges;
      final events = _run(
        held,
        _steady,
        until: () => held.phase != HuntPhase.shriek,
      );

      expect(events.whereType<HeldFirm>(), hasLength(1));
      expect(held.artCharges, before + 1);
    });

    test('a reaper with no art has nothing to throw', () {
      final sim = _sim();

      _ambush(sim);
      _run(sim, _focused, until: () => false, ticks: 40);

      expect(sim.step(_flick(sim)).whereType<ArtThrown>(), isEmpty);
      expect(sim.artCharges, 0);
    });
  });

  group('fire', () {
    test('lands on a husk at rest, burns it, and sends it running', () {
      final sim = _sim(art: Art.cinder);

      _ambush(sim);

      final before = sim.ward;

      _throw(sim);

      final landed = _land(sim).whereType<ArtLanded>().single;

      expect(landed.hit, isTrue);
      expect(landed.damage, closeTo(sim.stats.strikeDamage * 2.0, 1e-9));
      expect(sim.burning, isTrue);

      final burning = sim.ward;
      final events = _run(sim, _steady, until: () => !sim.burning, ticks: 400);

      expect(burning - sim.ward, closeTo(sim.stats.strikeDamage * 1.5, 0.5));
      expect(before - sim.ward, greaterThan(sim.stats.strikeDamage * 3.4));
      expect(events.whereType<Relocating>(), isNotEmpty);
    });

    test('is lobbed over the counter onto a husk hiding behind it', () {
      final sim = _hiddenInKitchen(art: Art.cinder);

      // The top of its head shows over the counter; aim there.
      expect(sim.plan.hides(sim.pose.position), isTrue);

      final events = _throw(sim);

      expect(events.whereType<ArtThrown>(), hasLength(1));
      expect(_land(sim).whereType<ArtLanded>().single.hit, isTrue);
    });

    test('a burning husk does not run for cover', () {
      final sim = _sim(art: Art.cinder, plan: kitchen(), seed: 8);

      _ambush(sim);
      _throw(sim);
      _land(sim);

      final footings = <Footing>{};

      _run(sim, _steady, until: () => !sim.burning, ticks: 400);
      _run(sim, (sim) {
        if (sim.burning) footings.add(sim.footing);
        return _steady(sim);
      }, until: () => !sim.burning);

      expect(footings, isNot(contains(Footing.hidden)));
    });
  });

  group('ice', () {
    test('freezes the husk where it stands, every seal in view open', () {
      final sim = _sim(art: Art.rime);

      _ambush(sim);
      _run(sim, _steady, until: () => sim.speed > 0, ticks: 600);
      _throw(sim);

      final landed = _land(sim).whereType<ArtLanded>().single;

      expect(landed.hit, isTrue);
      expect(sim.frozen, isTrue);
      expect(sim.speed, 0);

      final at = sim.pose.position;

      _run(sim, _steady, until: () => false, ticks: 60);
      expect(sim.pose.position, at);
      expect([
        for (var seal = 0; seal < sim.vessel.seals.length; seal++)
          if (sim.pose.sealFacesPlayer(seal)) sim.sealOpen(seal),
      ], everyElement(isTrue));
    });

    test('the next blow shatters it, and lands harder', () {
      final sim = _sim(art: Art.rime);

      _ambush(sim);
      // Spend the ambush's doubled first strike first.
      _run(sim, _steady, until: () => false, ticks: sim.stats.swingTicks);
      _throw(sim);
      _land(sim);
      _run(sim, _steady, until: () => false, ticks: sim.stats.swingTicks);

      final seal = [
        for (var index = 0; index < sim.vessel.seals.length; index++)
          if (sim.sealOpen(index)) index,
      ].first;
      final events = sim.step(
        HuntInput(
          aim: Aim.fromDirection(sim.pose.direction),
          turnSpeed: 0,
          taps: [Aim.fromDirection(sim.pose.sealDirection(seal))],
        ),
      );
      final landed = events.whereType<StrikeLanded>().single;

      expect(events.whereType<IceShattered>(), hasLength(1));
      expect(landed.empowered, isTrue);
      expect(sim.frozen, isFalse);
    });

    test('stops a charge dead', () {
      final sim = _sim(art: Art.rime, rank: 2);

      _ambush(sim);
      _run(sim, _steady, until: () => sim.phase == HuntPhase.charge);
      _throw(sim);

      final events = _land(sim);

      expect(events.whereType<ArtLanded>().single.hit, isTrue);
      expect(events.whereType<Struck>(), isEmpty);
      expect(sim.phase, isNot(HuntPhase.charge));
      expect(sim.frozen, isTrue);
    });

    test('breaks on the counter in front of a husk hiding behind it', () {
      final sim = _hiddenInKitchen(art: Art.rime);

      _throw(sim);

      expect(_land(sim).whereType<ArtLanded>().single.hit, isFalse);
      expect(sim.frozen, isFalse);
    });
  });

  group('wind', () {
    test('blows a husk out from behind the counter and stuns it', () {
      final sim = _hiddenInKitchen(art: Art.gale);

      _throw(sim);

      final events = _land(sim);

      expect(events.whereType<ArtLanded>().single.hit, isTrue);
      expect(events.whereType<Leapt>().single.toward, Footing.open);

      _run(sim, _steady, until: () => !sim.leaping, ticks: 200);

      expect(sim.footing, Footing.open);
      expect(sim.staggered, isTrue);
    });

    test('turns a charge aside', () {
      final sim = _sim(art: Art.gale);

      _ambush(sim);
      _run(sim, _steady, until: () => sim.phase == HuntPhase.charge);
      _throw(sim);

      final events = _land(sim);

      expect(events.whereType<Struck>(), isEmpty);
      expect(sim.staggered, isTrue);
    });

    test('catches a husk it was only thrown near', () {
      final sim = _sim(art: Art.gale);

      _ambush(sim);
      _run(
        sim,
        (sim) => HuntInput.measured(
          aim: Vec3.fromAngles(sim.pose.yaw + radians(15), sim.pose.pitch),
          turnRadiansPerSecond: 0,
          focus: true,
        ),
        until: () => sim.artGathered >= 1,
        ticks: 40,
      );
      sim.step(_flick(sim));

      expect(_land(sim).whereType<ArtLanded>().single.hit, isTrue);
    });
  });

  group('the heavy weapons', () {
    HuntInput strikeOpenSeal(HuntSim sim) {
      for (var seal = 0; seal < sim.vessel.seals.length; seal++) {
        if (sim.sealOpen(seal)) {
          return HuntInput(
            aim: Aim.fromDirection(sim.pose.direction),
            turnSpeed: 0,
            taps: [Aim.fromDirection(sim.pose.sealDirection(seal))],
          );
        }
      }

      return _steady(sim);
    }

    test('a greatsword stops a husk in its tracks', () {
      final sim = _sim(weapon: WeaponKind.greatsword);

      _ambush(sim);
      _run(sim, _steady, until: () => sim.speed > 0, ticks: 900);
      expect(sim.speed, greaterThan(0), reason: 'it should move off');

      final events = sim.step(strikeOpenSeal(sim));

      expect(events.whereType<StrikeLanded>(), hasLength(1));
      expect(sim.staggered, isTrue);
      expect(sim.speed, 0);
    });

    test('an axe bites the body wherever it lands', () {
      final axe = _sim(weapon: WeaponKind.axe);
      // A spear reaches no further than an axe, so the same blow misses
      // every seal with both.
      final spear = _sim(weapon: WeaponKind.spear);

      for (final sim in [axe, spear]) {
        _ambush(sim);
      }

      double glance(HuntSim sim) {
        final pose = sim.pose;
        // Low on the body, well away from any seal.
        final body = (pose.position - Vec3.up * 0.5).normalized;
        final events = sim.step(
          HuntInput(
            aim: Aim.fromDirection(body),
            turnSpeed: 0,
            taps: [Aim.fromDirection(body)],
          ),
        );

        return events.whereType<StrikeGlanced>().single.damage /
            sim.stats.strikeDamage;
      }

      expect(glance(axe), closeTo(0.7, 1e-9));
      expect(glance(spear), closeTo(HuntRules.glanceShare, 1e-9));
    });

    test('they cost more than a class weapon and need a few levels', () {
      const greatsword = Weapon(WeaponKind.greatsword, Temper.shrouded);

      expect(greatsword.priceFor(ReaperClass.harvester), Weapon.heavyPrice);
      expect(greatsword.levelRequired, Weapon.heavyLevel);
      expect(
        const Weapon(WeaponKind.axe, Temper.scorched).levelRequired,
        Temper.scorched.levelRequired,
      );
    });
  });

  group('a hunt with an art', () {
    test('a careful player with an art brings husks down sooner', () {
      for (final art in Art.values) {
        var withArt = 0;
        var without = 0;

        for (var seed = 1; seed <= 10; seed++) {
          for (final (known, add) in [
            (art, (int ticks) => withArt += ticks),
            (null, (int ticks) => without += ticks),
          ]) {
            final sim = HuntSim(
              seed: seed,
              vessel: Vessel.ape,
              ward: Rarity.rare.ward,
              stats: _stats(art: known),
              plan: kitchen(),
            );

            HuntBot(sim).play();
            add(sim.tick);
          }
        }

        expect(withArt, lessThan(without), reason: art.name);
      }
    });

    test('replays to the same end, throws and all', () {
      for (final art in Art.values) {
        final sim = _sim(art: art, plan: kitchen(), seed: 11);
        final bot = HuntBot(sim)..play();

        expect(bot.events.whereType<ArtThrown>(), isNotEmpty, reason: art.name);

        final log = HuntLog.decode(
          HuntLog(bot.recorded, plan: sim.plan).encode(),
        );
        final replayed = HuntSim.replay(
          seed: 11,
          vessel: Vessel.ape,
          ward: Rarity.epic.ward,
          stats: _stats(art: art),
          log: log,
        );

        expect(replayed?.toJson(), sim.outcome?.toJson(), reason: art.name);
      }
    });
  });

  group('a reaper\'s arts', () {
    final now = DateTime.utc(2026, 10, 19);

    PlayerState absorbing(PlayerState state, Nft nft, {int seed = 1}) {
      final released = state.acquire(nft).release(nft, now: now, seed: seed);
      final sighting = released.sightings.last;

      return released.settleHunt(
        sighting.id,
        const HuntOutcome.absorbed(
          grade: ReapGrade.clean,
          ambushed: false,
          ticks: 600,
        ),
        now: sighting.surfacesAt,
        feedingSeed: seed,
      );
    }

    Nft of(NftCollection collection, [int index = 0]) => marketListings
        .where((nft) => nft.collection == collection)
        .elementAt(index);

    test('the first husk carrying an art teaches it, and makes it ready', () {
      var state = PlayerState.initial().takeOath(seed: 5);

      expect(state.loadout.art, isNull);

      state = absorbing(state, of(NftCollection.peskyPenguins));

      expect(state.activeReaper!.art, Art.rime);
      expect(state.loadout.art, const ArtSkill(Art.rime, 1));

      // A second art learned leaves the first one ready.
      state = absorbing(state, of(NftCollection.thugbirdz), seed: 2);

      expect(state.activeReaper!.art, Art.rime);
      expect(state.activeReaper!.artRank(Art.gale), 1);
      expect(state.attune(Art.gale).loadout.art, const ArtSkill(Art.gale, 1));
    });

    test('an art that has not been learned cannot be made ready', () {
      final state = PlayerState.initial().takeOath(seed: 5);

      expect(
        () => state.attune(Art.cinder),
        throwsA(
          isA<RuleBreach>().having(
            (breach) => breach.refusal,
            'refusal',
            Refusal.notOwned,
          ),
        ),
      );
    });

    test('the art a reaper has ready is kept with it', () {
      final state = absorbing(
        PlayerState.initial().takeOath(seed: 5),
        of(NftCollection.degenApes),
      );
      final restored = PlayerState.fromJson(state.toJson());

      expect(restored.activeReaper!.art, Art.cinder);
      expect(restored.loadout.art, const ArtSkill(Art.cinder, 1));
    });

    test('a saved reaper from before arts loads with none ready', () {
      final state = PlayerState.initial().takeOath(seed: 5);
      final json = state.toJson();

      for (final reaper in json['reapers'] as List) {
        (reaper as Map).remove('art');
      }

      expect(PlayerState.fromJson(json).activeReaper!.art, isNull);
    });
  });
}
