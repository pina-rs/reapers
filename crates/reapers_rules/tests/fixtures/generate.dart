// Writes the golden fixtures the Rust rules are checked against: what the
// Dart rules in reapers_shared say, for every rule that can be asked a
// question with a definite answer.
//
// Retired with the Dart code on 8 November 2026, and kept as it was: it
// imports reapers_shared, which is no longer in this tree, so it cannot run
// here. The fixtures stay as they are, as the record of what the rules did
// when they were Dart. README.md, beside this file, gives the git command
// that checks out the last commit with the Dart package, where it runs from
// the repository root with the pinned SDK:
//
//     fvm dart run crates/reapers_rules/tests/fixtures/generate.dart
//
// ignore_for_file: avoid_print
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:reapers_shared/reapers_shared.dart';

import '../../../../reapers_shared/test/support/rift_table.dart';
import '../../../../reapers_shared/test/support/rooms.dart';
import '../../../../reapers_shared/test/support/sigil_tracer.dart';
import '../../../../reapers_shared/tool/pacing.dart' as pacing;
import '../../../../reapers_shared/tool/rift_pacing.dart' as rift_pacing;
import '../../../../reapers_shared/tool/soul_pacing.dart' as soul_pacing;

final _here = File.fromUri(Platform.script).parent;

void main() {
  _write('seeded_random.json', _seededRandom());
  _write('genomes.json', _genomes());
  _write('feeding.json', _feeding());
  _write('progression.json', _progression());
  _write('catalogue.json', _catalogue());
  _write('stats.json', _stats());
  _write('hunts.json', _hunts());
  _write('hunt_logs.json', _huntLogs());
  _write('sigils.json', _sigils());
  _write('geo.json', _geo());
  _write('rifts.json', _rifts());
  _write('player.json', _player());
  _writeText('pacing.txt', _printed(pacing.main));
  _writeText('rift_pacing.txt', _printed(rift_pacing.main));
  _writeText('soul_pacing.txt', _printed(soul_pacing.main));
}

void _write(String name, Object value) {
  File('${_here.path}/$name').writeAsStringSync(
    '${const JsonEncoder.withIndent(' ').convert(value)}\n',
  );
  print('wrote $name');
}

void _writeText(String name, String text) {
  File('${_here.path}/$name').writeAsStringSync(text);
  print('wrote $name');
}

/// Everything [body] prints, one line each.
String _printed(void Function() body) {
  final out = StringBuffer();

  runZoned(
    body,
    zoneSpecification: ZoneSpecification(
      print: (_, _, _, line) => out.writeln(line),
    ),
  );

  return out.toString();
}

/// The 64 bits of [value], in hex: how a double is compared exactly.
String _bits(double value) {
  final data = ByteData(8)..setFloat64(0, value);

  return data.getUint32(0).toRadixString(16).padLeft(8, '0') +
      data.getUint32(4).toRadixString(16).padLeft(8, '0');
}

/// FNV-1a over [bytes], 64 bits, in hex.
String _digest(List<int> bytes) {
  var hash = 0xcbf29ce484222325;

  for (final byte in bytes) {
    hash ^= byte & 0xFF;
    hash *= 0x100000001b3;
  }

  return ((hash >> 32) & 0xFFFFFFFF).toRadixString(16).padLeft(8, '0') +
      (hash & 0xFFFFFFFF).toRadixString(16).padLeft(8, '0');
}

// Seeded random --------------------------------------------------------

const _seeds = [
  0,
  1,
  7,
  42,
  -1,
  0x7FFFFFFF,
  0x80000000,
  0xFFFFFFFF,
  0x100000000,
  0xDEADBEEF,
  1 << 40 | 5,
  -123456789012,
];

Object _seededRandom() => {
  'streams': [
    for (final seed in _seeds)
      () {
        final uints = SeededRandom(seed);
        final doubles = SeededRandom(seed);
        final ints = SeededRandom(seed);
        final ranges = SeededRandom(seed);
        final weighted = SeededRandom(seed);

        return {
          'seed': seed,
          'uint32': [for (var i = 0; i < 64; i++) uints.nextUint32()],
          'double': [for (var i = 0; i < 16; i++) doubles.nextDouble()],
          'int': [
            for (final max in [1, 2, 3, 7, 10, 100, 1000, 1 << 20, 1 << 31])
              ints.nextInt(max),
          ],
          'range': [
            for (var i = 0; i < 8; i++) _bits(ranges.nextRange(-2.5, 7.25)),
          ],
          'weighted': [
            for (var i = 0; i < 16; i++)
              weighted.pickWeighted([25, 50, 20, 5, 0, 0.5]),
          ],
        };
      }(),
  ],
  'mixSeed': [
    for (final values in [
      <int>[],
      [0],
      [1, 2, 3],
      [7, 0x5EA1],
      [-1, 0xFEED],
      [1 << 40, 5, -6],
      [0xDEADBEEF, 0x5AADE, 2],
    ])
      {'values': values, 'seed': mixSeed(values)},
  ],
  'hashText': [
    for (final text in [
      '',
      'a',
      'look:thugbirdz:3',
      'mint:degenApes:8',
      'Vesper of the Last Furrow',
      'ŝ—😀',
    ])
      {'text': text, 'hash': hashText(text)},
  ],
};

// Genomes --------------------------------------------------------------

Object _genomes() => {
  'shuffles': [
    for (final seed in [
      for (var seed = 0; seed < 300; seed++) seed,
      ..._seeds,
    ])
      () {
        final genome = ReaperGenome.shuffle(seed);
        final soul = ReaperSoul.fromGenome(genome);

        return {
          'seed': seed,
          'code': genome.code,
          'name': genome.name,
          'gift': soul.gift.name,
          'temperament': soul.temperament.name,
        };
      }(),
  ],
  'parses': [
    for (final code in [
      'R1-har-h0-m0-r0-p0-o0-a2-s500-g500-z400-t500-n0-e0',
      'R1-mou-h2-m2-r3-p2-o3-a3-s1000-g0-z1000-t0-n39-e7',
      'R1-cho-h1-m1-r1-p1-o1-a1-s1-g2-z3-t4-n5-e6',
      'R1-hol-h+1-m0-r0-p0-o0-a0-s0x10-g0-z0-t0-n0-e0',
      'R1-har-h3-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      'R1-xxx-h0-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      'R2-har-h0-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      'R1-har-h0-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0',
      'R1-har-m0-h0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      'R1-har-h-1-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      'R1-wei-h0-m0-r0-p0-o4-a0-s0-g0-z0-t0-n0-e0',
      'R1-fer-h0-m0-r0-p0-o0-a0-s1001-g0-z0-t0-n0-e0',
      'R1-fer-h0-m0-r0-p0-o0-a0-s0-g0-z0-t0-n40-e0',
      'R1-fer-h0-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e8',
      'R1-fer-h 1-m0-r0-p0-o0-a0-s0-g0-z0-t0-n0-e0',
      '',
    ])
      () {
        try {
          return {'code': code, 'parsed': ReaperGenome.parse(code).code};
        } on FormatException {
          return {'code': code, 'parsed': null};
        }
      }(),
  ],
};

// Feeding and progression ----------------------------------------------

Object _feeding() => {
  'rolls': [for (var seed = 0; seed < 2000; seed++) Feeding.roll(seed).name],
  'bigSeeds': [
    for (final seed in _seeds) {'seed': seed, 'feeding': Feeding.roll(seed).name},
  ],
  'expectedShare': Feeding.expectedShare,
};

Object _progression() => {
  'levels': [
    for (var level = 1; level <= Progression.levelCap; level++)
      {
        'level': level,
        'xpToNext': Progression.xpToNext(level),
        'xpForLevel': Progression.xpForLevel(level),
      },
  ],
  'xp': [
    for (var xp = 0; xp < 40000; xp += 137)
      {
        'xp': xp,
        'level': Progression.levelForXp(xp),
        'progress': Progression.levelProgress(xp),
      },
  ],
};

// Catalogue ------------------------------------------------------------

Object _catalogue() => {
  'nfts': [
    for (final nft in marketListings)
      {
        'id': nft.id,
        'rarity': nft.rarity.name,
        'title': nft.title,
        'imageKey': nft.imageKey,
        'mockMint': nft.mockMint,
        'look': nft.look.toJson(),
      },
  ],
  'nftParses': [
    for (final id in [
      'thugbirdz:3',
      'degenApes:8',
      'solarians:1',
      'solarians:0',
      'solarians:9',
      'solarians:+2',
      'solarians:0x2',
      'solarians: 2',
      'solarians',
      'solarians:2:3',
      'nobody:1',
    ])
      () {
        try {
          return {'id': id, 'parsed': Nft.parse(id).id};
        } on FormatException {
          return {'id': id, 'parsed': null};
        } on ArgumentError {
          return {'id': id, 'parsed': null};
        }
      }(),
  ],
  'weapons': [
    for (final weapon in armoryWeapons)
      {
        'id': weapon.id,
        'levelRequired': weapon.levelRequired,
        'prices': {
          for (final reaperClass in ReaperClass.values)
            reaperClass.code: weapon.priceFor(reaperClass),
        },
      },
  ],
  'weaponParses': [
    for (final id in [
      'scythe.sworn',
      'greatsword.hollowed',
      'scythe',
      'scythe.sworn.extra',
      'sickle.sworn',
      'scythe.rusty',
    ])
      () {
        try {
          return {'id': id, 'parsed': Weapon.parse(id).id};
        } on FormatException {
          return {'id': id, 'parsed': null};
        } on ArgumentError {
          return {'id': id, 'parsed': null};
        }
      }(),
  ],
  'times': [
    for (final text in [
      '2026-10-01T12:00:00.000Z',
      '2026-10-01T12:00:00.123456Z',
      '2026-10-01T13:30:00+01:30',
      '1969-12-31T23:59:59.999Z',
      '2000-02-29T00:00:00Z',
    ])
      {
        'text': text,
        'micros': DateTime.parse(text).microsecondsSinceEpoch,
        'iso': DateTime.parse(text).toUtc().toIso8601String(),
      },
  ],
};

// Stats ----------------------------------------------------------------

Map<String, Object?> _statsJson(HuntStats stats) => {
  'strikeDamage': _bits(stats.strikeDamage),
  'reachRadians': _bits(stats.reachRadians),
  'swingTicks': stats.swingTicks,
  'resolve': stats.resolve,
  'captureTicks': stats.captureTicks,
  'reapToleranceRadians': _bits(stats.reapToleranceRadians),
  'flinchSpeed': _bits(stats.flinchSpeed),
  'parrySpeed': _bits(stats.parrySpeed),
  'perk': stats.perk.name,
  'gift': stats.gift?.name,
  'soulRank': stats.soulRank,
  'knockTicks': stats.knockTicks,
  'glanceShare': _bits(stats.glanceShare),
};

Map<String, Object?> _loadoutJson(HuntLoadout loadout) => {
  'class': loadout.reaperClass.code,
  'level': loadout.level,
  'weapon': loadout.weapon.id,
  'completedSets': loadout.completedSets,
  'gift': loadout.soul?.gift.name,
  'temperament': loadout.soul?.temperament.name,
  'soulRank': loadout.soulRank,
  'art': loadout.art?.art.name,
  'artRank': loadout.art?.rank,
};

Object _stats() {
  final cases = <Object>[];

  void add(HuntLoadout loadout, double leniency, Difficulty difficulty) {
    cases.add({
      'loadout': _loadoutJson(loadout),
      'leniency': leniency,
      'difficulty': difficulty.name,
      'stats': _statsJson(
        HuntStats.derive(loadout, leniency: leniency).at(difficulty),
      ),
    });
  }

  for (final reaperClass in ReaperClass.values) {
    for (final kind in WeaponKind.values) {
      for (final temper in Temper.values) {
        add(
          HuntLoadout(
            reaperClass: reaperClass,
            level: 7,
            weapon: Weapon(kind, temper),
            completedSets: 2,
          ),
          1,
          Difficulty.normal,
        );
      }
    }

    for (final temperament in [null, ...Temperament.values]) {
      for (final level in [1, 30]) {
        for (final leniency in [1.0, 1.03]) {
          final gift = ReaperGift.values[reaperClass.index * 3 + level % 3];

          add(
            HuntLoadout(
              reaperClass: reaperClass,
              level: level,
              weapon: Weapon(reaperClass.weapon, Temper.scorched),
              soul: temperament == null
                  ? null
                  : ReaperSoul(gift: gift, temperament: temperament),
              soulRank: level == 30 ? 3 : 0,
            ),
            leniency,
            Difficulty.normal,
          );
        }
      }
    }

    for (final difficulty in Difficulty.values) {
      add(
        HuntLoadout(
          reaperClass: reaperClass,
          level: 12,
          weapon: Weapon(reaperClass.weapon, Temper.sworn),
        ),
        1.03,
        difficulty,
      );
    }
  }

  return {
    'cases': cases,
    'riftWards': [
      for (final levels in [
        [1],
        [1, 1, 1, 1],
        [5, 12, 20, 30],
        [30, 1],
      ])
        {
          'levels': levels,
          'ward': _bits(
            RiftRules.wardFor([
              for (var index = 0; index < levels.length; index++)
                HuntStats.derive(
                  HuntLoadout(
                    reaperClass: ReaperClass
                        .values[index % ReaperClass.values.length],
                    level: levels[index],
                    weapon: Weapon(
                      ReaperClass
                          .values[index % ReaperClass.values.length]
                          .weapon,
                      Temper.values.lastWhere(
                        (temper) => temper.levelRequired <= levels[index],
                      ),
                    ),
                  ),
                ),
            ]),
          ),
        },
    ],
  };
}

// Hunts ----------------------------------------------------------------

Map<String, Object?> _planJson(HuntPlan plan) => {
  'centre': plan.centreCentidegrees,
  'halfWidth': plan.halfWidthCentidegrees,
  'eye': plan.eyeCentimetres,
  'reach': plan.reachDecimetres,
  'cover': plan.coverCentimetres,
  'hide': plan.hideDecimetres,
  'ledges': plan.ledges,
};

/// An event as one line of text, with every double as its bits.
String _event(HuntEvent event) => switch (event) {
  GiftStirred(:final gift) => 'GiftStirred:${gift.name}',
  StalkEnded(:final ambush) => 'StalkEnded:$ambush',
  StrikeLanded(
    :final seal,
    :final damage,
    :final cadence,
    :final empowered,
    :final cleaved,
  ) =>
    'StrikeLanded:$seal:${_bits(damage)}:$cadence:$empowered:'
        '${cleaved.join(',')}',
  StrikeGlanced(:final damage) => 'StrikeGlanced:${_bits(damage)}',
  StrikeWhiffed() => 'StrikeWhiffed',
  SealOpened(:final seal) => 'SealOpened:$seal',
  ShriekWindup() => 'ShriekWindup',
  ShriekBegan() => 'ShriekBegan',
  Flinched(:final resolveLeft, :final forgiven) =>
    'Flinched:$resolveLeft:$forgiven',
  HeldFirm() => 'HeldFirm',
  ChargeBegan() => 'ChargeBegan',
  Parried() => 'Parried',
  Struck(:final resolveLeft) => 'Struck:$resolveLeft',
  Relocating() => 'Relocating',
  Leapt(:final toward) => 'Leapt:${toward.name}',
  ArtThrown(:final art) => 'ArtThrown:${art.name}',
  ArtLanded(:final art, :final at, :final hit, :final damage) =>
    'ArtLanded:${art.name}:${_bits(at.x)}:${_bits(at.y)}:${_bits(at.z)}:'
        '$hit:${_bits(damage)}',
  IceShattered() => 'IceShattered',
  Broken() => 'Broken',
  ReapBegan() => 'ReapBegan',
  ReapRecovered(:final attemptsLeft) => 'ReapRecovered:$attemptsLeft',
  HuskAbsorbed(:final grade) => 'HuskAbsorbed:${grade.name}',
  HuskEscaped(:final cause) => 'HuskEscaped:${cause.name}',
};

/// One hunt to play: who, against what, where, and how the bot plays.
final class _HuntCase {
  _HuntCase({
    required this.name,
    required this.seed,
    required this.vessel,
    required this.rarity,
    required this.loadout,
    this.plan = HuntPlan.open,
    this.difficulty = Difficulty.normal,
    this.leniency = 1,
    this.turnSpeed = 1.2,
    this.reapSpeed = 0.9,
    this.reactionTicks = 16,
    this.missEvery = 7,
    this.chargeNerve = 0.4,
    this.idle = false,
    this.keepEvents = false,
  });

  final String name;
  final int seed;
  final Vessel vessel;
  final Rarity rarity;
  final HuntLoadout loadout;
  final HuntPlan plan;
  final Difficulty difficulty;
  final double leniency;
  final double turnSpeed;
  final double reapSpeed;
  final int reactionTicks;
  final int missEvery;
  final double chargeNerve;

  /// Whether the player does nothing at all.
  final bool idle;

  /// Whether to keep every event, not only their digest.
  final bool keepEvents;

  Object play() {
    final sim = HuntSim(
      seed: seed,
      vessel: vessel,
      ward: rarity.ward,
      stats: HuntStats.derive(loadout, leniency: leniency),
      plan: plan,
      difficulty: difficulty,
    );
    final bot = HuntBot(
      sim,
      turnSpeed: turnSpeed,
      reapSpeed: reapSpeed,
      reactionTicks: reactionTicks,
      missEvery: missEvery,
      chargeNerve: chargeNerve,
    );
    final inputs = <HuntInput>[];
    final lines = <String>[];
    final kinds = <String, int>{};
    final checkpoints = <Object>[];

    while (!sim.isOver && sim.tick < HuntLog.maxTicks) {
      final tick = sim.tick;
      final input = idle
          ? const HuntInput(aim: Aim.ahead, turnSpeed: 0)
          : bot.decide();

      inputs.add(input);

      for (final event in sim.step(input)) {
        final line = _event(event);

        lines.add('$tick|$line');
        kinds.update(line.split(':').first, (count) => count + 1,
            ifAbsent: () => 1);
      }

      if (sim.tick % 120 == 0 || sim.isOver) {
        final pose = sim.pose;

        checkpoints.add({
          'tick': sim.tick,
          'phase': sim.phase.name,
          'ward': _bits(sim.ward),
          'resolve': sim.resolve,
          'cadence': sim.cadence,
          'x': _bits(pose.position.x),
          'y': _bits(pose.position.y),
          'z': _bits(pose.position.z),
          'heading': _bits(pose.heading),
          'footing': pose.footing.name,
          'leaping': pose.leaping,
          'ground': pose.ground == null ? null : _bits(pose.ground!),
          'speed': _bits(sim.speed),
          'artCharges': sim.artCharges,
          'awareness': _bits(sim.awareness),
          'reapProgress': _bits(sim.reapProgress),
        });
      }
    }

    final log = HuntLog(inputs, plan: plan, difficulty: difficulty);
    final outcome = sim.outcome;
    final replayed = HuntSim.replay(
      seed: seed,
      vessel: vessel,
      ward: rarity.ward,
      stats: HuntStats.derive(loadout, leniency: leniency),
      log: log,
    );

    if (replayed?.toJson().toString() != outcome?.toJson().toString()) {
      throw StateError('$name does not replay');
    }

    return {
      'name': name,
      'seed': seed,
      'vessel': vessel.name,
      'rarity': rarity.name,
      'loadout': _loadoutJson(loadout),
      'plan': _planJson(plan),
      'difficulty': difficulty.name,
      'leniency': leniency,
      'bot': idle
          ? null
          : {
              'turnSpeed': turnSpeed,
              'reapSpeed': reapSpeed,
              'reactionTicks': reactionTicks,
              'missEvery': missEvery,
              'chargeNerve': chargeNerve,
            },
      'outcome': outcome?.toJson(),
      'ticks': sim.tick,
      'logDigest': _digest(log.encode()),
      'eventDigest': _digest(utf8.encode(lines.join('\n'))),
      'eventKinds': kinds,
      if (keepEvents) 'events': lines,
      'checkpoints': checkpoints,
    };
  }
}

HuntLoadout _loadout(
  ReaperClass reaperClass,
  int level, {
  WeaponKind? kind,
  Temper? temper,
  int completedSets = 0,
  ReaperSoul? soul,
  int soulRank = 0,
  ArtSkill? art,
}) => HuntLoadout(
  reaperClass: reaperClass,
  level: level,
  weapon: Weapon(
    kind ?? reaperClass.weapon,
    temper ??
        Temper.values.lastWhere((temper) => temper.levelRequired <= level),
  ),
  completedSets: completedSets,
  soul: soul,
  soulRank: soulRank,
  art: art,
);

List<_HuntCase> _huntCases() {
  final solo = [
    for (final vessel in Vessel.values)
      if (vessel.isSolo) vessel,
  ];
  const levels = [1, 5, 10, 16, 30];
  final cases = <_HuntCase>[];

  for (final reaperClass in ReaperClass.values) {
    for (var index = 0; index < solo.length; index++) {
      cases.add(
        _HuntCase(
          name: '${reaperClass.name} ${solo[index].name}',
          seed: 100 + reaperClass.index * 10 + index,
          vessel: solo[index],
          rarity: Rarity.values[(reaperClass.index + index) % 4],
          loadout: _loadout(reaperClass, levels[index]),
          keepEvents: reaperClass == ReaperClass.harvester && index == 0,
        ),
      );
    }
  }

  for (final gift in ReaperGift.values) {
    cases.add(
      _HuntCase(
        name: 'gift ${gift.name}',
        seed: 300 + gift.index,
        vessel: Vessel.penguin,
        rarity: Rarity.rare,
        loadout: _loadout(
          gift.reaperClass,
          20,
          temper: Temper.shrouded,
          soul: ReaperSoul(
            gift: gift,
            temperament: Temperament.values[gift.index % 4],
          ),
          soulRank: 3,
        ),
      ),
    );
  }

  final carriers = {
    Art.cinder: [Vessel.ape, Vessel.robot, Vessel.penguin],
    Art.rime: [Vessel.llama, Vessel.penguin, Vessel.ape],
    Art.gale: [Vessel.bird, Vessel.ape, Vessel.llama],
  };

  for (final art in Art.values) {
    for (var rank = 1; rank <= ArtRules.maxRank; rank++) {
      cases.add(
        _HuntCase(
          name: 'art ${art.name} $rank',
          seed: 400 + art.index * 10 + rank,
          vessel: carriers[art]![rank - 1],
          rarity: Rarity.uncommon,
          loadout: _loadout(
            ReaperClass.values[art.index * 2 + rank % 2],
            8,
            art: ArtSkill(art, rank),
          ),
          plan: kitchen(),
          reactionTicks: 0,
          missEvery: 0,
          keepEvents: art == Art.cinder && rank == 1,
        ),
      );
    }
  }

  cases
    ..add(
      _HuntCase(
        name: 'easy ape',
        seed: 501,
        vessel: Vessel.ape,
        rarity: Rarity.epic,
        loadout: _loadout(ReaperClass.mourner, 1),
        difficulty: Difficulty.easy,
      ),
    )
    ..add(
      _HuntCase(
        name: 'hard ape',
        seed: 502,
        vessel: Vessel.ape,
        rarity: Rarity.rare,
        loadout: _loadout(ReaperClass.chooser, 18),
        difficulty: Difficulty.hard,
      ),
    )
    ..add(
      _HuntCase(
        name: 'narrow plan',
        seed: 503,
        vessel: Vessel.llama,
        rarity: Rarity.common,
        loadout: _loadout(ReaperClass.weigher, 3),
        plan: HuntPlan.between(radians(-60), radians(50), eyeCentimetres: 120),
      ),
    )
    ..add(
      _HuntCase(
        name: 'surveyed plan',
        seed: 504,
        vessel: Vessel.robot,
        rarity: Rarity.uncommon,
        loadout: _loadout(ReaperClass.hollow, 6),
        plan: HuntPlan(
          centreCentidegrees: 1500,
          halfWidthCentidegrees: 9000,
          eyeCentimetres: 160,
          reachDecimetres: [
            for (var slice = 0; slice < HuntPlan.slices; slice++)
              12 + (slice * 7) % 30,
          ],
        ),
      ),
    )
    ..add(
      _HuntCase(
        name: 'kitchen ape',
        seed: 505,
        vessel: Vessel.ape,
        rarity: Rarity.rare,
        loadout: _loadout(ReaperClass.harvester, 12),
        plan: kitchen(),
      ),
    )
    ..add(
      _HuntCase(
        name: 'low counter bird',
        seed: 506,
        vessel: Vessel.bird,
        rarity: Rarity.common,
        loadout: _loadout(ReaperClass.ferryman, 2),
        plan: kitchen(counterCentimetres: 60, ledge: false),
      ),
    )
    ..add(
      _HuntCase(
        name: 'greatsword',
        seed: 507,
        vessel: Vessel.llama,
        rarity: Rarity.rare,
        loadout: _loadout(
          ReaperClass.ferryman,
          9,
          kind: WeaponKind.greatsword,
          temper: Temper.scorched,
        ),
      ),
    )
    ..add(
      _HuntCase(
        name: 'axe',
        seed: 508,
        vessel: Vessel.robot,
        rarity: Rarity.uncommon,
        loadout: _loadout(
          ReaperClass.mourner,
          3,
          kind: WeaponKind.axe,
          temper: Temper.shrouded,
        ),
      ),
    )
    ..add(
      _HuntCase(
        name: 'lenient with sets',
        seed: 509,
        vessel: Vessel.penguin,
        rarity: Rarity.epic,
        loadout: _loadout(ReaperClass.weigher, 14, completedSets: 3),
        leniency: 1.03,
      ),
    )
    ..add(
      _HuntCase(
        name: 'no nerve',
        seed: 510,
        vessel: Vessel.ape,
        rarity: Rarity.epic,
        loadout: _loadout(ReaperClass.harvester, 1),
        chargeNerve: 2,
        missEvery: 2,
      ),
    )
    ..add(
      _HuntCase(
        name: 'jittery',
        seed: 511,
        vessel: Vessel.bird,
        rarity: Rarity.rare,
        loadout: _loadout(ReaperClass.hollow, 4),
        turnSpeed: 2.5,
        reapSpeed: 3,
        reactionTicks: 40,
      ),
    )
    ..add(
      _HuntCase(
        name: 'idle',
        seed: 512,
        vessel: Vessel.penguin,
        rarity: Rarity.common,
        loadout: _loadout(ReaperClass.harvester, 1),
        idle: true,
      ),
    );

  return cases;
}

Object _hunts() => [for (final hunt in _huntCases()) hunt.play()];

// Hunt logs ------------------------------------------------------------

Object _huntLogs() {
  final sim = HuntSim(
    seed: 77,
    vessel: Vessel.ape,
    ward: Rarity.common.ward,
    stats: HuntStats.derive(_loadout(ReaperClass.chooser, 4)),
    plan: kitchen(),
  );
  final bot = HuntBot.human(sim)..play();
  final log = HuntLog(bot.recorded, plan: kitchen());
  final tapped = HuntLog(
    [
      const HuntInput(aim: Aim(-18000, 9000), turnSpeed: 0xFFFF),
      HuntInput(
        aim: const Aim(120, -45),
        turnSpeed: 1234,
        taps: const [Aim(1, 2), Aim(-3, -4)],
        focus: true,
      ),
    ],
    plan: HuntPlan.between(radians(-80), radians(80), eyeCentimetres: 99),
    difficulty: Difficulty.hard,
  );
  final whole = log.encode();
  final small = tapped.encode();

  String? decodes(List<int> bytes) {
    try {
      HuntLog.decode(Uint8List.fromList(bytes));
      return null;
    } on FormatException catch (error) {
      return error.message;
    }
  }

  return {
    'logs': [
      for (final (name, encoded) in [('kitchen hunt', whole), ('taps', small)])
        () {
          final decoded = HuntLog.decode(encoded);

          return {
            'name': name,
            'bytes': base64.encode(encoded),
            'ticks': decoded.inputs.length,
            'plan': _planJson(decoded.plan),
            'difficulty': decoded.difficulty.name,
            'firstInputs': [
              for (final input in decoded.inputs.take(3))
                {
                  'aim': [
                    input.aim.yawCentidegrees,
                    input.aim.pitchCentidegrees,
                  ],
                  'turnSpeed': input.turnSpeed,
                  'taps': [
                    for (final tap in input.taps)
                      [tap.yawCentidegrees, tap.pitchCentidegrees],
                  ],
                  'focus': input.focus,
                },
            ],
          };
        }(),
    ],
    'replay': {
      'seed': 77,
      'vessel': 'ape',
      'rarity': 'common',
      'loadout': _loadoutJson(_loadout(ReaperClass.chooser, 4)),
      'bytes': base64.encode(whole),
      'outcome': HuntSim.replay(
        seed: 77,
        vessel: Vessel.ape,
        ward: Rarity.common.ward,
        stats: HuntStats.derive(_loadout(ReaperClass.chooser, 4)),
        log: HuntLog.decode(whole),
      )?.toJson(),
      'truncatedOutcome': HuntSim.replay(
        seed: 77,
        vessel: Vessel.ape,
        ward: Rarity.common.ward,
        stats: HuntStats.derive(_loadout(ReaperClass.chooser, 4)),
        log: HuntLog(bot.recorded.take(400).toList(), plan: kitchen()),
      )?.toJson(),
    },
    'malformed': [
      for (final (name, bytes) in [
        ('empty', <int>[]),
        ('short', small.sublist(0, 5)),
        ('version', [5, ...small.sublist(1)]),
        ('difficulty', [...small.sublist(0, 7), 3, ...small.sublist(8)]),
        ('reach', [...small.sublist(0, 6), 5, ...small.sublist(7)]),
        ('cover', [...small.sublist(0, 8), 7, ...small.sublist(9)]),
        ('mid plan', whole.sublist(0, 100)),
        ('mid tick', small.sublist(0, small.length - 3)),
        ('mid tap', small.sublist(0, small.length - 2)),
        (
          'flags',
          [...small.sublist(0, 9 + 6), 2, ...small.sublist(9 + 7)],
        ),
        (
          'aim',
          [...small.sublist(0, 9), 0x50, 0x46, ...small.sublist(11)],
        ),
        (
          'tap',
          [...small.sublist(0, small.length - 4), 0x50, 0x46, 0, 0],
        ),
      ])
        {'name': name, 'bytes': base64.encode(bytes), 'error': decodes(bytes)},
    ],
  };
}

// Sigils ---------------------------------------------------------------

Object _sigils() => {
  'draws': [
    for (final rarity in Rarity.values)
      for (final gait in Gait.values)
        for (final difficulty in Difficulty.values)
          for (var seed = 0; seed < 8; seed++)
            () {
              final sigil = Sigil.draw(
                seed: seed * 131 + 3,
                shape: rarity.sigil,
                gait: gait,
                difficulty: difficulty,
              );

              return {
                'rarity': rarity.name,
                'gait': gait.name,
                'difficulty': difficulty.name,
                'seed': seed * 131 + 3,
                'lights': [
                  for (final light in sigil.lights)
                    [_bits(light.yaw), _bits(light.pitch)],
                ],
                'radius': _bits(sigil.radius),
                'holdSeconds': _bits(sigil.holdSeconds),
                'fraySeconds': sigil.fraySeconds == null
                    ? null
                    : _bits(sigil.fraySeconds!),
                'crossings': sigil.crossings,
                'halfArc': _bits(sigil.halfArc),
              };
            }(),
  ],
  'traces': [
    for (final rarity in Rarity.values)
      for (final gait in [Gait.ground, Gait.air])
        for (final (turnSpeed, wobble) in [(1.0, 0.15), (0.6, 0.5)])
          for (var seed = 0; seed < 3; seed++)
            () {
              final trace = SigilTrace(
                Sigil.draw(seed: seed, shape: rarity.sigil, gait: gait),
                centre: radians(10.0 * seed),
              );
              final result = traceSigil(
                trace,
                turnSpeed: turnSpeed,
                wobble: wobble,
              );

              return {
                'rarity': rarity.name,
                'gait': gait.name,
                'seed': seed,
                'turnSpeed': turnSpeed,
                'wobble': wobble,
                'closed': result.closed,
                'seconds': _bits(result.seconds),
                'lost': result.lost,
                'caught': trace.caught,
              };
            }(),
  ],
};

// Geo ------------------------------------------------------------------

Object _geo() {
  final random = SeededRandom(0x6E0);
  final pairs = <Object>[];

  for (var index = 0; index < 200; index++) {
    final from = GeoPoint(
      random.nextRange(-89, 89),
      random.nextRange(-180, 180),
    );
    final near = index.isEven;
    final to = GeoPoint(
      near
          ? from.latitude + random.nextRange(-0.002, 0.002)
          : random.nextRange(-89, 89),
      near
          ? from.longitude + random.nextRange(-0.002, 0.002)
          : random.nextRange(-180, 180),
    );
    final bearing = random.nextRange(-3.2, 3.2);
    final metres = random.nextRange(0, near ? 300 : 3000000);
    final moved = from.moved(bearing: bearing, metres: metres);
    final north = random.nextRange(-3.2, 3.2);
    final sighting = RiftSighting.of(anchor: to, standing: from, north: north);

    pairs.add({
      'from': [from.latitude, from.longitude],
      'to': [to.latitude, to.longitude],
      'distance': _bits(from.distanceTo(to)),
      'bearing': _bits(from.bearingTo(to)),
      'move': [bearing, metres],
      'moved': [_bits(moved.latitude), _bits(moved.longitude)],
      'north': north,
      'sighting': sighting == null
          ? null
          : [_bits(sighting.bearing), _bits(sighting.metres)],
    });
  }

  return {
    'pairs': pairs,
    'anchors': [
      for (final site in RiftSite.values)
        for (final (metres, facing) in [(40.0, 0.3), (2000.0, -1.2)])
          () {
            final at = site.location.moved(bearing: 1, metres: metres);
            final anchor = site.anchorFor(
              GeoStanding(at: at, facing: facing, accuracy: 5),
            );

            return {
              'site': site.name,
              'at': [at.latitude, at.longitude],
              'facing': facing,
              'anchor': [_bits(anchor.latitude), _bits(anchor.longitude)],
            };
          }(),
    ],
  };
}

// Rifts ----------------------------------------------------------------

Object _rifts() {
  final rifts = <Object>[];

  Object play(String name, RiftTable table, {int? walkAwayAt}) {
    final frames = <Object>[];

    table.begin();

    while (!table.room.phase.isOver) {
      if (walkAwayAt != null && table.room.tick == walkAwayAt) {
        table.walkAway(table.seats.length - 1);
      }

      table.step();

      if (table.room.tick % 600 == 0) {
        frames.add(table.room.frameFor(table.seats.first).toJson());
      }
    }

    return {
      'name': name,
      'phase': table.room.phase.name,
      'tick': table.room.tick,
      'wardMax': _bits(table.room.wardMax),
      'ward': _bits(table.room.ward),
      'rewards': [
        for (final seat in table.seats)
          {
            'xp': table.room.rewardFor(seat).xp,
            'anima': table.room.rewardFor(seat).anima,
          },
      ],
      'final': table.room.frameFor(table.seats.first).toJson(),
      'frames': frames,
    };
  }

  for (final level in [1, 6, 12, 20]) {
    for (final people in [1, 2, 4]) {
      for (final seed in [1, 2]) {
        final reapers = [
          for (var person = 0; person < people; person++)
            entrant(
              name: reaperGivenNames[person],
              reaperClass:
                  ReaperClass.values[(seed + person) % ReaperClass.values.length],
              level: level,
            ),
        ];

        rifts.add({
          'site': RiftSite.values[(level + people) % RiftSite.values.length]
              .name,
          'seed': seed,
          'reapers': [
            for (final reaper in reapers)
              {
                'name': reaper.name,
                'class': reaper.loadout.reaperClass.code,
                'level': reaper.loadout.level,
              },
          ],
          'walkAwayAt': null,
          'play': play(
            'level $level, $people people, seed $seed',
            RiftTable(
              reapers: reapers,
              site: RiftSite.values[(level + people) % RiftSite.values.length],
              seed: seed,
            ),
          ),
        });
      }
    }
  }

  final reapers = [
    entrant(name: 'Morrow', reaperClass: ReaperClass.ferryman, level: 9),
    entrant(name: 'Thren', reaperClass: ReaperClass.mourner, level: 9),
  ];

  rifts.add({
    'site': RiftSite.brooklynBridge.name,
    'seed': 31,
    'reapers': [
      for (final reaper in reapers)
        {
          'name': reaper.name,
          'class': reaper.loadout.reaperClass.code,
          'level': reaper.loadout.level,
        },
    ],
    'walkAwayAt': 900,
    'play': play(
      'one walks away',
      RiftTable(reapers: reapers, site: RiftSite.brooklynBridge, seed: 31),
      walkAwayAt: 900,
    ),
  });

  // The shades a room seats, and what they fight with.
  final room = RiftRoom(site: RiftSite.shibuyaCrossing, seed: 0x51DE);
  final seat = room.join(entrant(level: 13));

  room
    ..report(seat, const RiftReport(ready: true))
    ..advance(RiftRules.countdownTicks + 1);

  return {
    'rifts': rifts,
    'shades': room.frameFor(seat).toJson(),
  };
}

// The player -----------------------------------------------------------

Object _player() {
  final steps = <Object>[];
  final now = DateTime.utc(2026, 10, 1, 12);
  var state = PlayerState.initial();

  void record(String name, PlayerState Function() request) {
    try {
      state = request();
      steps.add({'step': name, 'state': state.toJson()});
    } on RuleBreach catch (breach) {
      steps.add({'step': name, 'refusal': breach.refusal.name});
    }
  }

  const ape = Nft(collection: NftCollection.degenApes, serial: 2);
  const bird = Nft(collection: NftCollection.thugbirdz, serial: 8);
  const penguin = Nft(collection: NftCollection.peskyPenguins, serial: 7);

  record('release before the oath', () => state.release(ape, seed: 1, now: now));
  record('take the oath', () => state.takeOath(seed: 0xBEEF));
  record('take it again', () => state.takeOath(seed: 1));
  record('acquire an ape', () => state.acquire(ape));
  record('acquire it again', () => state.acquire(ape));
  record('acquire a bird', () => state.acquire(bird));
  record('acquire a penguin', () => state.acquire(penguin));
  record('release the ape', () => state.release(ape, seed: 41, now: now));
  record('release the bird', () => state.release(bird, seed: 42, now: now));

  final settlements = <Object>[];

  for (final sighting in [...state.sightings]) {
    final sim = HuntSim(
      seed: sighting.seed,
      vessel: sighting.nft.collection.vessel,
      ward: sighting.nft.rarity.ward,
      stats: HuntStats.derive(state.loadout),
    );
    final bot = HuntBot.human(sim)..play();
    final later = now.add(const Duration(minutes: 1));

    try {
      final settlement = state.judgeHunt(
        sighting.id,
        HuntLog(bot.recorded),
        now: later,
        feedingSeed: sighting.seed * 3 + 1,
        leniency: 1.03,
      );

      state = settlement.state;
      settlements.add(settlement.toJson());
      steps.add({'step': 'judge ${sighting.nft.id}', 'state': state.toJson()});
    } on RuleBreach catch (breach) {
      steps.add({
        'step': 'judge ${sighting.nft.id}',
        'refusal': breach.refusal.name,
      });
    }
  }

  record('release the penguin', () => state.release(penguin, seed: 43, now: now));
  record('hunt too early', () {
    state.huntable(state.sightings.last.id, now: now);
    return state;
  });
  record(
    'settle an escape',
    () => state.settleHunt(
      state.sightings.last.id,
      const HuntOutcome.escaped(
        escape: EscapeCause.time,
        ambushed: false,
        ticks: 4500,
      ),
      now: now.add(const Duration(minutes: 2)),
      feedingSeed: 5,
    ),
  );
  record(
    'settle an absorption',
    () => state.settleHunt(
      state.sightings.last.id,
      const HuntOutcome.absorbed(
        grade: ReapGrade.perfect,
        ambushed: true,
        ticks: 1800,
        deeds: HuntDeeds(holds: 2, parries: 1, maxCadence: 6),
      ),
      now: now.add(const Duration(minutes: 3)),
      feedingSeed: 0xFEEDFACE,
    ),
  );
  record('sign the penguin', () => state.signAbsorption(penguin, 'c2lnbmVk'));
  record('buy a heavy weapon too soon', () {
    return state.buyWeapon(const Weapon(WeaponKind.axe, Temper.shrouded));
  });
  record('settle a rift', () => state.settleRift(xp: 400, anima: 900));
  record(
    'buy an axe',
    () => state.buyWeapon(const Weapon(WeaponKind.axe, Temper.shrouded)),
  );
  record(
    'wield it',
    () => state.wield(const Weapon(WeaponKind.axe, Temper.shrouded)),
  );
  record('buy an outfit', () => state.buyOutfit(Outfit.sworn));
  record('wear it', () => state.wear(Outfit.sworn));
  record('attune rime', () => state.attune(Art.rime));
  record('attune gale', () => state.attune(Art.gale));
  record('enter a rift', () {
    state.entrantFor(RiftSite.londonBridge);
    return state;
  });

  final schemaTwo = {
    'schema': 2,
    'genome': ReaperGenome.shuffle(5).code,
    'xp': 500,
    'weapons': ['scythe.shrouded', 'scythe.sworn', 'scythe.sworn'],
    'outfits': ['shrouded'],
    'weapon': 'scythe.sworn',
    'outfit': 'shrouded',
    'codex': [],
    'huntsLost': 2,
    'riftsSealed': 1,
    'anima': 70,
    'lamports': 1000,
    'vault': ['sollamas:4'],
    'sightings': [],
    'nextSightingId': 3,
  };

  return {
    'steps': steps,
    'settlements': settlements,
    'loadout': _loadoutJson(state.loadout),
    'entrant': () {
      try {
        return state.entrantFor(RiftSite.londonBridge).name;
      } on RuleBreach catch (breach) {
        return breach.refusal.name;
      }
    }(),
    'schemaTwo': schemaTwo,
    'schemaTwoMigrated': PlayerState.fromJson(schemaTwo).toJson(),
  };
}
