import 'armory.dart';
import 'feeding.dart';
import 'hunt/difficulty.dart';
import 'hunt/hunt_event.dart';
import 'hunt/hunt_input.dart';
import 'hunt/hunt_sim.dart';
import 'hunt/hunt_stats.dart';
import 'nft_catalogue.dart';
import 'reaper_genome.dart';
import 'reaper_state.dart';
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

  /// Tradable reapers require replayed rift results before earning growth.
  riftUnproven,

  /// The reaper has absorbed nothing from this rift's collection yet.
  notAttuned,

  /// A location is malformed, too coarse, stale or future-dated.
  locationUnusable,

  /// Finish loose hunts before taking another reaper.
  reaperBusy,

  /// The wallet proof is invalid, expired or already used.
  walletUnproven,

  /// NFT ownership requires a configured collection and online keeper.
  chainUnconfigured,

  /// The chain could not be read or returned an invalid response.
  chainUnavailable,

  /// The NFT roster has reached its bounded ownership-check capacity.
  rosterFull,
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
///
/// It records everything the keeper decided at that moment, so the entry is
/// also what a signed receipt is made from: see docs/nfts.md.
final class Absorption {
  const Absorption({
    required this.nft,
    required this.grade,
    required this.feeding,
    required this.xp,
    required this.anima,
    required this.nonce,
    required this.at,
    this.signature,
  });

  factory Absorption.fromJson(Map<String, dynamic> json) => Absorption(
    nft: Nft.parse(json['nft'] as String),
    grade: ReapGrade.values.byName(json['grade'] as String),
    feeding: Feeding.values.byName(json['feeding'] as String),
    xp: json['xp'] as int,
    anima: json['anima'] as int,
    nonce: json['nonce'] as int,
    at: DateTime.parse(json['at'] as String),
    signature: json['signature'] as String?,
  );

  final Nft nft;

  final ReapGrade grade;

  /// How well the reaper fed, as the keeper rolled it.
  final Feeding feeding;

  /// Experience the absorption paid.
  final int xp;

  /// Anima the absorption paid.
  final int anima;

  /// The seed the feeding was rolled from. With the NFT it makes this
  /// absorption unlike any other, which is what lets a receipt be used once.
  final int nonce;

  final DateTime at;

  /// The keeper's signature over this absorption, in base64, or null when
  /// the keeper was the phone itself: an offline game signs nothing.
  final String? signature;

  /// This absorption, signed.
  Absorption signedWith(String signature) => Absorption(
    nft: nft,
    grade: grade,
    feeding: feeding,
    xp: xp,
    anima: anima,
    nonce: nonce,
    at: at,
    signature: signature,
  );

  Map<String, dynamic> toJson() => {
    'nft': nft.id,
    'grade': grade.name,
    'feeding': feeding.name,
    'xp': xp,
    'anima': anima,
    'nonce': nonce,
    'at': at.toIso8601String(),
    'signature': signature,
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
    required this.feeding,
  });

  /// Reads a settlement written by [toJson].
  factory HuntSettlement.fromJson(Map<String, dynamic> json) => HuntSettlement(
    state: PlayerState.fromJson((json['state'] as Map).cast<String, dynamic>()),
    outcome: HuntOutcome.fromJson(
      (json['outcome'] as Map).cast<String, dynamic>(),
    ),
    xp: json['xp'] as int,
    anima: json['anima'] as int,
    feeding: switch (json['feeding']) {
      final String name => Feeding.values.byName(name),
      _ => null,
    },
  );

  final PlayerState state;

  final HuntOutcome outcome;

  /// Experience the hunt paid.
  final int xp;

  /// Anima the hunt paid.
  final int anima;

  /// How well the reaper fed, or null when the husk got away.
  final Feeding? feeding;

  Map<String, dynamic> toJson() => {
    'state': state.toJson(),
    'outcome': outcome.toJson(),
    'xp': xp,
    'anima': anima,
    'feeding': feeding?.name,
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
    required this.reapers,
    required this.activeReaperId,
    required this.anima,
    required this.lamports,
    required this.vault,
    required this.sightings,
    required this.nextSightingId,
  });

  factory PlayerState.initial() => const PlayerState(
    reapers: [],
    activeReaperId: null,
    anima: 0,
    lamports: startingLamports,
    vault: [],
    sightings: [],
    nextSightingId: 1,
  );

  /// Schema two's one reaper migrates intact into the oathbound roster seat.
  factory PlayerState.fromJson(Map<String, dynamic> json) {
    final schema = json['schema'];
    if (schema != 2 && schema != _schema) {
      throw FormatException('Unknown player schema', schema);
    }
    final List<ReaperState> roster;
    final String? selected;
    if (schema == 2) {
      roster = json['genome'] == null
          ? []
          : [
              ReaperState.fromJson({...json, 'id': ReaperState.oathId}),
            ];
      selected = roster.firstOrNull?.id;
    } else {
      roster = [
        for (final entry in json['reapers'] as List)
          ReaperState.fromJson((entry as Map).cast<String, dynamic>()),
      ];
      selected = json['activeReaperId'] as String?;
    }
    if (roster.map((reaper) => reaper.id).toSet().length != roster.length ||
        (roster.isEmpty
            ? selected != null
            : !roster.any((reaper) => reaper.id == selected))) {
      throw const FormatException(
        'Invalid active reaper or duplicate roster seat',
      );
    }
    return PlayerState(
      reapers: List.unmodifiable(roster),
      activeReaperId: selected,
      anima: json['anima'] as int,
      lamports: json['lamports'] as int,
      vault: [for (final id in json['vault'] as List) Nft.parse(id as String)],
      sightings: [
        for (final entry in json['sightings'] as List)
          Sighting.fromJson((entry as Map).cast<String, dynamic>()),
      ],
      nextSightingId: json['nextSightingId'] as int,
    );
  }

  /// The stand-in wallet a new player starts with: three SOL, which is
  /// enough to buy every NFT the market lists.
  static const startingLamports = 3000000000;

  /// Husks that may be loose at once.
  static const maxSightings = 3;

  /// Keeps one ownership read bounded; the oathbound starter is additional.
  static const maxNftReapers = 32;

  /// The shortest and longest wait, in seconds, before a released husk
  /// surfaces.
  static const surfaceMinSeconds = 6;
  static const surfaceMaxSeconds = 14;

  static const _schema = 3;

  /// Each reaper owns its growth, codex and equipment. Anima and the vault
  /// belong to the player; switching never copies experience between reapers.
  final List<ReaperState> reapers;
  final String? activeReaperId;
  final int anima;

  /// Offline simulation balance; a linked Solana balance is read separately.
  final int lamports;
  final List<Nft> vault;
  final List<Sighting> sightings;
  final int nextSightingId;

  ReaperState? get activeReaper =>
      reapers.where((reaper) => reaper.id == activeReaperId).firstOrNull;
  String? get genomeCode => activeReaper?.genomeCode;
  int get xp => activeReaper?.xp ?? 0;
  Set<Weapon> get weapons => activeReaper?.weapons ?? const {};
  Set<Outfit> get outfits => activeReaper?.outfits ?? const {Outfit.shrouded};
  Weapon? get weapon => activeReaper?.weapon;
  Outfit get outfit => activeReaper?.outfit ?? Outfit.shrouded;
  List<Absorption> get codex => activeReaper?.codex ?? const [];
  int get huntsLost => activeReaper?.huntsLost ?? 0;
  int get riftsSealed => activeReaper?.riftsSealed ?? 0;
  bool get oathTaken => activeReaper != null;
  ReaperGenome get genome => activeReaper?.genome ?? ReaperGenome.borrowed;
  int get level => activeReaper?.level ?? 1;

  /// Registers a reaper whose NFT ownership has already been verified by
  /// the caller. There is deliberately no simulated SOL purchase here.
  PlayerState bindReaper(ReaperState reaper) {
    if (reapers.any((owned) => owned.id == reaper.id)) {
      throw const RuleBreach(Refusal.alreadyOwned);
    }
    if (reaper.isNft &&
        reapers.where((owned) => owned.isNft).length >= maxNftReapers) {
      throw const RuleBreach(Refusal.rosterFull);
    }
    return _copy(reapers: [...reapers, reaper]);
  }

  /// Loose husks belong to the current reaper until their hunts are finished.
  PlayerState selectReaper(String id) {
    if (!reapers.any((reaper) => reaper.id == id)) {
      throw const RuleBreach(Refusal.notOwned);
    }
    if (id != activeReaperId && sightings.isNotEmpty) {
      throw const RuleBreach(Refusal.reaperBusy);
    }
    return _copy(activeReaperId: id);
  }

  /// Refreshes NFT-backed seats from the authoritative asset ledger while
  /// retaining the non-tradable oathbound starter. A transferred seat leaves
  /// this roster; its progression stays in the server's asset ledger.
  PlayerState reconcileReapers(List<ReaperState> owned) {
    final roster = [
      for (final reaper in reapers)
        if (!reaper.isNft) reaper,
      ...owned,
    ];
    if (roster.map((reaper) => reaper.id).toSet().length != roster.length) {
      throw const RuleBreach(Refusal.alreadyOwned);
    }

    // A transferred reaper cannot keep hunting under another reaper's stats.
    // Return its unfinished husks to this player's vault without rewarding it.
    final lostActive = !roster.any((reaper) => reaper.id == activeReaperId);
    return _copy(
      reapers: roster,
      vault: lostActive
          ? [...vault, for (final sighting in sightings) sighting.nft]
          : vault,
      sightings: lostActive ? [] : sightings,
    );
  }

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
      reapers.any((reaper) => reaper.codex.any((entry) => entry.nft == nft));

  /// Binds a reaper drawn from [seed].
  ///
  /// The caller supplies the seed, so the server decides it for an online
  /// player and nobody picks their own reaper.
  PlayerState takeOath({required int seed}) {
    if (oathTaken) {
      throw const RuleBreach(Refusal.oathAlreadyTaken);
    }

    final genome = ReaperGenome.shuffle(seed);
    return _copy(
      reapers: [ReaperState.bound(id: ReaperState.oathId, genome: genome)],
    );
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
  ///
  /// [feedingSeed] decides how well the reaper feeds on an absorbed husk. The
  /// caller supplies it, so the server rolls for an online player and nobody
  /// picks their own luck.
  PlayerState settleHunt(
    int sightingId,
    HuntOutcome outcome, {
    required DateTime now,
    required int feedingSeed,
  }) {
    final sighting = huntable(sightingId, now: now);
    final grade = outcome.grade;

    if (grade == null) {
      return _copy(huntsLost: huntsLost + 1);
    }

    final rarity = sighting.nft.rarity;
    final feeding = Feeding.roll(feedingSeed);
    final absorption = Absorption(
      nft: sighting.nft,
      grade: grade,
      feeding: feeding,
      xp: outcome.xpFor(rarity, feeding),
      anima: outcome.animaFor(rarity),
      nonce: feedingSeed,
      at: now,
    );

    return _copy(
      xp: xp + absorption.xp,
      anima: anima + absorption.anima,
      sightings: [
        for (final loose in sightings)
          if (loose.id != sightingId) loose,
      ],
      codex: [...codex, absorption],
    );
  }

  /// Attaches the keeper's [signature] to the absorption of [nft].
  PlayerState signAbsorption(Nft nft, String signature) => _copy(
    codex: [
      for (final entry in codex)
        entry.nft == nft ? entry.signedWith(signature) : entry,
    ],
  );

  /// Judges the hunt recorded in [log] and settles it.
  ///
  /// The log is replayed through the rules, so the verdict is whatever the
  /// rules say those inputs earn; nothing the player claims is believed. A
  /// log that stops before the hunt ends throws [Refusal.huntUnproven].
  ///
  /// The server passes a [leniency] a little above 1, so a hunt won by a
  /// hair on the phone is not judged lost over a rounding difference.
  ///
  /// A hunt played at a difficulty outside [difficulties] is refused the
  /// same way. A keeper that pays for real allows only the game's own.
  HuntSettlement judgeHunt(
    int sightingId,
    HuntLog log, {
    required DateTime now,
    required int feedingSeed,
    double leniency = 1,
    Set<Difficulty> difficulties = const {
      Difficulty.easy,
      Difficulty.normal,
      Difficulty.hard,
    },
  }) {
    final sighting = huntable(sightingId, now: now);
    final provable =
        log.inputs.length <= HuntLog.maxTicks &&
        difficulties.contains(log.difficulty);

    if (!provable) {
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

    final settled = settleHunt(
      sightingId,
      outcome,
      now: now,
      feedingSeed: feedingSeed,
    );
    final absorption = outcome.absorbed ? settled.codex.last : null;

    return HuntSettlement(
      state: settled,
      outcome: outcome,
      xp: absorption?.xp ?? 0,
      anima: absorption?.anima ?? 0,
      feeding: absorption?.feeding,
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

    if (activeReaper!.isNft) {
      throw const RuleBreach(Refusal.riftUnproven);
    }

    if (!attunedTo(site)) {
      throw const RuleBreach(Refusal.notAttuned);
    }

    return RiftEntrant(name: genome.name, loadout: loadout);
  }

  /// Pays a reaper for helping to seal a rift.
  PlayerState settleRift({
    required int xp,
    required int anima,
    String? reaperId,
  }) {
    _requireOath();
    final id = reaperId ?? activeReaperId;
    if (!reapers.any((reaper) => reaper.id == id)) {
      throw const RuleBreach(Refusal.notOwned);
    }
    return _copy(
      reapers: [
        for (final reaper in reapers)
          reaper.id == id
              ? reaper.copyWith(
                  xp: reaper.xp + xp,
                  riftsSealed: reaper.riftsSealed + 1,
                )
              : reaper,
      ],
      anima: this.anima + anima,
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
    'reapers': [for (final reaper in reapers) reaper.toJson()],
    'activeReaperId': activeReaperId,
    'anima': anima,
    'lamports': lamports,
    'vault': [for (final nft in vault) nft.id],
    'sightings': [for (final sighting in sightings) sighting.toJson()],
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
    List<ReaperState>? reapers,
    String? activeReaperId,
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
  }) {
    final roster =
        reapers ??
        [
          for (final reaper in this.reapers)
            reaper.id == this.activeReaperId
                ? reaper.copyWith(
                    xp: xp,
                    weapons: weapons,
                    outfits: outfits,
                    weapon: weapon,
                    outfit: outfit,
                    codex: codex,
                    huntsLost: huntsLost,
                    riftsSealed: riftsSealed,
                  )
                : reaper,
        ];
    final preferred = activeReaperId ?? this.activeReaperId;
    final selected = roster.any((reaper) => reaper.id == preferred)
        ? preferred
        : roster.firstOrNull?.id;
    return PlayerState(
      reapers: List.unmodifiable(roster),
      activeReaperId: selected,
      anima: anima ?? this.anima,
      lamports: lamports ?? this.lamports,
      vault: vault ?? this.vault,
      sightings: sightings ?? this.sightings,
      nextSightingId: nextSightingId ?? this.nextSightingId,
    );
  }
}
