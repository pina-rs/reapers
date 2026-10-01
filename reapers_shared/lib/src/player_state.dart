import 'armory.dart';
import 'hunt/hunt_event.dart';
import 'hunt/hunt_input.dart';
import 'hunt/hunt_sim.dart';
import 'hunt/hunt_stats.dart';
import 'nft_catalogue.dart';
import 'progression.dart';
import 'reaper_genome.dart';
import 'rift/rift_room.dart';
import 'rift/rift_rules.dart';
import 'rift/rift_site.dart';
import 'seeded_random.dart';

/// Why a request was refused.
enum Refusal {
  /// The reaper is already bound; the oath is taken once.
  oathAlreadyTaken,

  /// Only an oathbound reaper may do this.
  oathRequired,

  /// The market does not list this NFT.
  notListed,

  /// The player already holds, has released or has absorbed this NFT.
  alreadyHeld,

  /// The NFT is not in the player's vault.
  notInVault,

  /// The wallet does not hold enough SOL.
  insufficientFunds,

  /// The reaper does not hold enough anima.
  insufficientAnima,

  /// The reaper's level is too low.
  levelTooLow,

  /// The player has not bought this item.
  notOwned,

  /// The player already owns this item.
  alreadyOwned,

  /// Too many husks are already loose.
  veilCrowded,

  /// No such husk is loose.
  unknownSighting,

  /// The husk has not surfaced yet.
  notSurfaced,

  /// The submitted hunt does not replay to a finished hunt.
  huntUnproven,

  /// The reaper has absorbed nothing from this rift's collection yet.
  notAttuned,
}

/// A request the rules refused.
///
/// The game and the server raise the same exception from the same code, so
/// an offline player and an online one are told the same thing.
final class RuleBreach implements Exception {
  const RuleBreach(this.refusal);

  final Refusal refusal;

  @override
  String toString() => 'RuleBreach(${refusal.name})';
}

/// A husk loose in the veil, waiting to be hunted.
final class Sighting {
  const Sighting({
    required this.id,
    required this.nft,
    required this.seed,
    required this.surfacesAt,
  });

  factory Sighting.fromJson(Map<String, dynamic> json) => Sighting(
    id: json['id'] as int,
    nft: Nft.parse(json['nft'] as String),
    seed: json['seed'] as int,
    surfacesAt: DateTime.parse(json['surfacesAt'] as String),
  );

  /// Unique among one player's sightings.
  final int id;

  final Nft nft;

  /// The seed of every hunt for this husk.
  final int seed;

  /// When the husk shows itself. Before then it cannot be hunted.
  final DateTime surfacesAt;

  bool hasSurfaced(DateTime now) => !now.isBefore(surfacesAt);

  Map<String, dynamic> toJson() => {
    'id': id,
    'nft': nft.id,
    'seed': seed,
    'surfacesAt': surfacesAt.toIso8601String(),
  };
}

/// An NFT the reaper has absorbed, kept in the codex.
final class Absorption {
  const Absorption({required this.nft, required this.grade, required this.at});

  factory Absorption.fromJson(Map<String, dynamic> json) => Absorption(
    nft: Nft.parse(json['nft'] as String),
    grade: ReapGrade.values.byName(json['grade'] as String),
    at: DateTime.parse(json['at'] as String),
  );

  final Nft nft;

  final ReapGrade grade;

  final DateTime at;

  Map<String, dynamic> toJson() => {
    'nft': nft.id,
    'grade': grade.name,
    'at': at.toIso8601String(),
  };
}

/// What a judged hunt came to: the verdict, what it paid, and the player
/// afterwards.
final class HuntSettlement {
  const HuntSettlement({
    required this.state,
    required this.outcome,
    required this.xp,
    required this.anima,
  });

  /// Reads a settlement written by [toJson].
  factory HuntSettlement.fromJson(Map<String, dynamic> json) => HuntSettlement(
    state: PlayerState.fromJson((json['state'] as Map).cast<String, dynamic>()),
    outcome: HuntOutcome.fromJson(
      (json['outcome'] as Map).cast<String, dynamic>(),
    ),
    xp: json['xp'] as int,
    anima: json['anima'] as int,
  );

  final PlayerState state;

  final HuntOutcome outcome;

  /// Experience the hunt paid.
  final int xp;

  /// Anima the hunt paid.
  final int anima;

  Map<String, dynamic> toJson() => {
    'state': state.toJson(),
    'outcome': outcome.toJson(),
    'xp': xp,
    'anima': anima,
  };

  /// A settlement never changes, so a copy of it is itself. Serverpod asks
  /// for this of every value it carries between the server and the game.
  HuntSettlement copyWith() => this;
}

/// Everything one player owns and has done.
///
/// The state is immutable. Each request is a method that returns the state
/// after it, or throws a [RuleBreach]; nothing here reads a clock or rolls a
/// die on its own. The offline game and the server both keep a player as one
/// of these and change it only through these methods, which is what keeps
/// the two from drifting apart.
final class PlayerState {
  const PlayerState({
    required this.genomeCode,
    required this.xp,
    required this.anima,
    required this.lamports,
    required this.weapons,
    required this.outfits,
    required this.weapon,
    required this.outfit,
    required this.vault,
    required this.sightings,
    required this.codex,
    required this.huntsLost,
    required this.riftsSealed,
    required this.nextSightingId,
  });

  /// A player who has not taken the oath: no reaper, a stocked wallet.
  factory PlayerState.initial() => const PlayerState(
    genomeCode: null,
    xp: 0,
    anima: 0,
    lamports: startingLamports,
    weapons: {},
    outfits: {Outfit.shrouded},
    weapon: null,
    outfit: Outfit.shrouded,
    vault: [],
    sightings: [],
    codex: [],
    huntsLost: 0,
    riftsSealed: 0,
    nextSightingId: 1,
  );

  /// Reads a state written by [toJson].
  ///
  /// Throws a [FormatException] when the document is from an unknown schema
  /// or names something that does not exist.
  factory PlayerState.fromJson(Map<String, dynamic> json) {
    if (json['schema'] != _schema) {
      throw FormatException('Unknown player schema', json['schema']);
    }

    final weapon = json['weapon'] as String?;

    return PlayerState(
      genomeCode: json['genome'] as String?,
      xp: json['xp'] as int,
      anima: json['anima'] as int,
      lamports: json['lamports'] as int,
      weapons: {
        for (final id in json['weapons'] as List) Weapon.parse(id as String),
      },
      outfits: {
        for (final name in json['outfits'] as List)
          Outfit.values.byName(name as String),
      },
      weapon: weapon == null ? null : Weapon.parse(weapon),
      outfit: Outfit.values.byName(json['outfit'] as String),
      vault: [for (final id in json['vault'] as List) Nft.parse(id as String)],
      sightings: [
        for (final sighting in json['sightings'] as List)
          Sighting.fromJson((sighting as Map).cast<String, dynamic>()),
      ],
      codex: [
        for (final entry in json['codex'] as List)
          Absorption.fromJson((entry as Map).cast<String, dynamic>()),
      ],
      huntsLost: json['huntsLost'] as int,
      riftsSealed: json['riftsSealed'] as int,
      nextSightingId: json['nextSightingId'] as int,
    );
  }

  /// The stand-in wallet a new player starts with: two and a half SOL.
  static const startingLamports = 2500000000;

  /// Husks that may be loose at once.
  static const maxSightings = 3;

  /// The shortest and longest wait, in seconds, before a released husk
  /// surfaces.
  static const surfaceMinSeconds = 6;
  static const surfaceMaxSeconds = 14;

  static const _schema = 1;

  /// The bound reaper's genome, or null before the oath.
  final String? genomeCode;

  final int xp;

  /// What a reaper spends in the armory. Earned by absorbing, never bought
  /// and never tradeable.
  final int anima;

  /// The stand-in wallet balance.
  final int lamports;

  /// Weapons bought so far.
  final Set<Weapon> weapons;

  /// Outfits bought so far.
  final Set<Outfit> outfits;

  /// The weapon in hand, or null before the oath.
  final Weapon? weapon;

  /// The outfit being worn.
  final Outfit outfit;

  /// NFTs held and not yet released.
  final List<Nft> vault;

  /// Husks loose in the veil.
  final List<Sighting> sightings;

  /// Every NFT absorbed, oldest first.
  final List<Absorption> codex;

  /// Hunts that ended with the husk escaping.
  final int huntsLost;

  /// Rifts this reaper helped seal.
  final int riftsSealed;

  final int nextSightingId;

  bool get oathTaken => genomeCode != null;

  /// The bound reaper, or the borrowed one before the oath.
  ReaperGenome get genome {
    final code = genomeCode;

    return code == null ? ReaperGenome.borrowed : ReaperGenome.parse(code);
  }

  int get level => Progression.levelForXp(xp);

  /// Collections with a full set absorbed.
  int get completedSets => NftCollection.values
      .where((collection) => absorbedFrom(collection) >= NftCollection.setSize)
      .length;

  /// How many of [collection] the reaper has absorbed.
  int absorbedFrom(NftCollection collection) =>
      codex.where((entry) => entry.nft.collection == collection).length;

  /// What the reaper takes into a hunt.
  HuntLoadout get loadout {
    final weapon = this.weapon;

    if (weapon == null) {
      return HuntLoadout.borrowed;
    }

    return HuntLoadout(
      reaperClass: genome.reaperClass,
      level: level,
      weapon: weapon,
      completedSets: completedSets,
    );
  }

  /// Whether the player holds [nft] in any form.
  bool holds(Nft nft) =>
      vault.contains(nft) ||
      sightings.any((sighting) => sighting.nft == nft) ||
      codex.any((entry) => entry.nft == nft);

  /// Binds a reaper drawn from [seed].
  ///
  /// The caller supplies the seed, so the server decides it for an online
  /// player and nobody picks their own reaper.
  PlayerState takeOath({required int seed}) {
    if (oathTaken) {
      throw const RuleBreach(Refusal.oathAlreadyTaken);
    }

    final genome = ReaperGenome.shuffle(seed);
    final weapon = Weapon(genome.reaperClass.weapon, Temper.shrouded);

    return _copy(genomeCode: genome.code, weapons: {weapon}, weapon: weapon);
  }

  /// Buys [nft] from the stand-in market into the vault.
  PlayerState acquire(Nft nft) {
    _requireOath();

    if (holds(nft)) {
      throw const RuleBreach(Refusal.alreadyHeld);
    }

    if (lamports < nft.rarity.lamports) {
      throw const RuleBreach(Refusal.insufficientFunds);
    }

    return _copy(
      lamports: lamports - nft.rarity.lamports,
      vault: [...vault, nft],
    );
  }

  /// Releases [nft] from the vault into the veil, where it becomes a husk.
  ///
  /// [seed] decides when the husk surfaces and how every hunt for it plays.
  PlayerState release(Nft nft, {required int seed, required DateTime now}) {
    _requireOath();

    if (!vault.contains(nft)) {
      throw const RuleBreach(Refusal.notInVault);
    }

    if (sightings.length >= maxSightings) {
      throw const RuleBreach(Refusal.veilCrowded);
    }

    final wait =
        surfaceMinSeconds +
        SeededRandom(seed).nextInt(surfaceMaxSeconds - surfaceMinSeconds + 1);

    return _copy(
      vault: [
        for (final held in vault)
          if (held != nft) held,
      ],
      sightings: [
        ...sightings,
        Sighting(
          id: nextSightingId,
          nft: nft,
          seed: seed,
          surfacesAt: now.add(Duration(seconds: wait)),
        ),
      ],
      nextSightingId: nextSightingId + 1,
    );
  }

  /// The loose husk with [sightingId], ready to be hunted at [now].
  Sighting huntable(int sightingId, {required DateTime now}) {
    final sighting = sightings
        .where((candidate) => candidate.id == sightingId)
        .firstOrNull;

    if (sighting == null) {
      throw const RuleBreach(Refusal.unknownSighting);
    }

    if (!sighting.hasSurfaced(now)) {
      throw const RuleBreach(Refusal.notSurfaced);
    }

    return sighting;
  }

  /// Records how the hunt for [sightingId] ended.
  ///
  /// An absorbed husk leaves the veil for the codex and pays out. An escaped
  /// one stays loose, to be hunted again.
  PlayerState settleHunt(
    int sightingId,
    HuntOutcome outcome, {
    required DateTime now,
  }) {
    final sighting = huntable(sightingId, now: now);
    final grade = outcome.grade;

    if (grade == null) {
      return _copy(huntsLost: huntsLost + 1);
    }

    final rarity = sighting.nft.rarity;

    return _copy(
      xp: xp + outcome.xpFor(rarity),
      anima: anima + outcome.animaFor(rarity),
      sightings: [
        for (final loose in sightings)
          if (loose.id != sightingId) loose,
      ],
      codex: [
        ...codex,
        Absorption(nft: sighting.nft, grade: grade, at: now),
      ],
    );
  }

  /// Judges the hunt recorded in [log] and settles it.
  ///
  /// The log is replayed through the rules, so the verdict is whatever the
  /// rules say those inputs earn; nothing the player claims is believed. A
  /// log that stops before the hunt ends throws [Refusal.huntUnproven].
  ///
  /// The server passes a [leniency] a little above 1, so a hunt won by a
  /// hair on the phone is not judged lost over a rounding difference.
  HuntSettlement judgeHunt(
    int sightingId,
    HuntLog log, {
    required DateTime now,
    double leniency = 1,
  }) {
    final sighting = huntable(sightingId, now: now);

    if (log.inputs.length > HuntLog.maxTicks) {
      throw const RuleBreach(Refusal.huntUnproven);
    }

    final outcome = HuntSim.replay(
      seed: sighting.seed,
      vessel: sighting.nft.collection.vessel,
      ward: sighting.nft.rarity.ward,
      stats: HuntStats.derive(loadout, leniency: leniency),
      log: log,
    );

    if (outcome == null) {
      throw const RuleBreach(Refusal.huntUnproven);
    }

    return HuntSettlement(
      state: settleHunt(sightingId, outcome, now: now),
      outcome: outcome,
      xp: outcome.xpFor(sighting.nft.rarity),
      anima: outcome.animaFor(sighting.nft.rarity),
    );
  }

  /// Whether the rift at [site] opens for this reaper.
  ///
  /// A rift answers only a reaper who already carries one of its dead, so
  /// the way to a collection's rift runs through that collection's NFTs.
  bool attunedTo(RiftSite site) =>
      absorbedFrom(site.collection) >= RiftRules.attunement;

  /// This reaper as they arrive at the rift at [site].
  RiftEntrant entrantFor(RiftSite site) {
    _requireOath();

    if (!attunedTo(site)) {
      throw const RuleBreach(Refusal.notAttuned);
    }

    return RiftEntrant(name: genome.name, loadout: loadout);
  }

  /// Pays a reaper for helping to seal a rift.
  PlayerState settleRift({required int xp, required int anima}) {
    _requireOath();

    return _copy(
      xp: this.xp + xp,
      anima: this.anima + anima,
      riftsSealed: riftsSealed + 1,
    );
  }

  /// Buys [weapon] with anima.
  PlayerState buyWeapon(Weapon weapon) {
    _requireOath();

    if (weapons.contains(weapon)) {
      throw const RuleBreach(Refusal.alreadyOwned);
    }

    final price = weapon.priceFor(genome.reaperClass);

    _requireLevel(weapon.temper.levelRequired);
    _requireAnima(price);

    return _copy(anima: anima - price, weapons: {...weapons, weapon});
  }

  /// Takes a bought [weapon] in hand.
  PlayerState wield(Weapon weapon) {
    if (!weapons.contains(weapon)) {
      throw const RuleBreach(Refusal.notOwned);
    }

    return _copy(weapon: weapon);
  }

  /// Buys [outfit] with anima.
  PlayerState buyOutfit(Outfit outfit) {
    _requireOath();

    if (outfits.contains(outfit)) {
      throw const RuleBreach(Refusal.alreadyOwned);
    }

    _requireLevel(outfit.levelRequired);
    _requireAnima(outfit.price);

    return _copy(anima: anima - outfit.price, outfits: {...outfits, outfit});
  }

  /// Puts on a bought [outfit].
  PlayerState wear(Outfit outfit) {
    if (!outfits.contains(outfit)) {
      throw const RuleBreach(Refusal.notOwned);
    }

    return _copy(outfit: outfit);
  }

  /// A player state never changes, so a copy of it is itself. Serverpod asks
  /// for this of every value it carries between the server and the game.
  PlayerState copyWith() => this;

  Map<String, dynamic> toJson() => {
    'schema': _schema,
    'genome': genomeCode,
    'xp': xp,
    'anima': anima,
    'lamports': lamports,
    'weapons': [for (final weapon in weapons) weapon.id],
    'outfits': [for (final outfit in outfits) outfit.name],
    'weapon': weapon?.id,
    'outfit': outfit.name,
    'vault': [for (final nft in vault) nft.id],
    'sightings': [for (final sighting in sightings) sighting.toJson()],
    'codex': [for (final entry in codex) entry.toJson()],
    'huntsLost': huntsLost,
    'riftsSealed': riftsSealed,
    'nextSightingId': nextSightingId,
  };

  void _requireOath() {
    if (!oathTaken) {
      throw const RuleBreach(Refusal.oathRequired);
    }
  }

  void _requireLevel(int required) {
    if (level < required) {
      throw const RuleBreach(Refusal.levelTooLow);
    }
  }

  void _requireAnima(int price) {
    if (anima < price) {
      throw const RuleBreach(Refusal.insufficientAnima);
    }
  }

  PlayerState _copy({
    String? genomeCode,
    int? xp,
    int? anima,
    int? lamports,
    Set<Weapon>? weapons,
    Set<Outfit>? outfits,
    Weapon? weapon,
    Outfit? outfit,
    List<Nft>? vault,
    List<Sighting>? sightings,
    List<Absorption>? codex,
    int? huntsLost,
    int? riftsSealed,
    int? nextSightingId,
  }) => PlayerState(
    genomeCode: genomeCode ?? this.genomeCode,
    xp: xp ?? this.xp,
    anima: anima ?? this.anima,
    lamports: lamports ?? this.lamports,
    weapons: weapons ?? this.weapons,
    outfits: outfits ?? this.outfits,
    weapon: weapon ?? this.weapon,
    outfit: outfit ?? this.outfit,
    vault: vault ?? this.vault,
    sightings: sightings ?? this.sightings,
    codex: codex ?? this.codex,
    huntsLost: huntsLost ?? this.huntsLost,
    riftsSealed: riftsSealed ?? this.riftsSealed,
    nextSightingId: nextSightingId ?? this.nextSightingId,
  );
}
