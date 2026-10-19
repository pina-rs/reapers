import 'armory.dart';
import 'arts.dart';
import 'player_state.dart';
import 'progression.dart';
import 'reaper_genome.dart';
import 'reaper_soul.dart';

/// The progression that travels with one reaper, never with a selected slot.
/// NFT-backed ids are their Core asset addresses; the starter has a local id.
final class ReaperState {
  ReaperState({
    required this.id,
    required this.genomeCode,
    required this.xp,
    required Set<Weapon> weapons,
    required Set<Outfit> outfits,
    required this.weapon,
    required this.outfit,
    required List<Absorption> codex,
    required this.huntsLost,
    required this.riftsSealed,
    this.memory = const SoulMemory(),
    this.art,
  }) : weapons = Set.unmodifiable(weapons),
       outfits = Set.unmodifiable(outfits),
       codex = List.unmodifiable(codex) {
    ReaperGenome.parse(genomeCode);
    if (id.isEmpty ||
        xp < 0 ||
        huntsLost < 0 ||
        riftsSealed < 0 ||
        !weapons.contains(weapon) ||
        !outfits.contains(outfit) ||
        (art != null && artRank(art!) == 0)) {
      throw const FormatException('Invalid reaper record');
    }
  }

  static const oathId = 'oathbound';

  factory ReaperState.bound({
    required String id,
    required ReaperGenome genome,
  }) {
    final weapon = Weapon(genome.reaperClass.weapon, Temper.shrouded);
    return ReaperState(
      id: id,
      genomeCode: genome.code,
      xp: 0,
      weapons: {weapon},
      outfits: {Outfit.shrouded},
      weapon: weapon,
      outfit: Outfit.shrouded,
      codex: [],
      huntsLost: 0,
      riftsSealed: 0,
    );
  }

  factory ReaperState.fromJson(Map<String, dynamic> json) {
    final genome = ReaperGenome.parse(json['genome'] as String);
    final id = json['id'] as String;
    final xp = json['xp'] as int;
    return ReaperState(
      id: id,
      genomeCode: genome.code,
      xp: xp,
      weapons: {
        for (final id in json['weapons'] as List) Weapon.parse(id as String),
      },
      outfits: {
        for (final name in json['outfits'] as List)
          Outfit.values.byName(name as String),
      },
      weapon: Weapon.parse(json['weapon'] as String),
      outfit: Outfit.values.byName(json['outfit'] as String),
      codex: [
        for (final entry in json['codex'] as List)
          Absorption.fromJson((entry as Map).cast<String, dynamic>()),
      ],
      huntsLost: json['huntsLost'] as int,
      riftsSealed: json['riftsSealed'] as int,
      memory: json['memory'] == null
          ? const SoulMemory()
          : SoulMemory.fromJson(
              (json['memory'] as Map).cast<String, dynamic>(),
            ),
      art: switch (json['art']) {
        final String name => Art.values.byName(name),
        _ => null,
      },
    );
  }

  final String id;
  final String genomeCode;
  final int xp;
  final Set<Weapon> weapons;
  final Set<Outfit> outfits;
  final Weapon weapon;
  final Outfit outfit;
  final List<Absorption> codex;
  final int huntsLost;
  final int riftsSealed;
  final SoulMemory memory;

  /// The art the reaper has made ready to throw, out of those it knows.
  final Art? art;

  bool get isNft => id != oathId;
  ReaperGenome get genome => ReaperGenome.parse(genomeCode);
  int get level => Progression.levelForXp(xp);
  ReaperSoul get soul => ReaperSoul.fromGenome(genome);
  int get soulRank =>
      SoulRules.rank(level: level, memory: memory, gift: soul.gift);

  /// How deep [art] runs in this reaper: 0 until it has absorbed a husk
  /// that carries it. It is read from the codex, so it travels with the
  /// reaper and cannot be bought.
  int artRank(Art art) => Art.rankFor(
    codex
        .where((entry) => art.vessels.contains(entry.nft.collection.vessel))
        .length,
  );

  /// The art the reaper throws, at the rank it knows it.
  ArtSkill? get artSkill {
    final art = this.art;

    return art == null ? null : ArtSkill(art, artRank(art));
  }

  ReaperState copyWith({
    int? xp,
    Set<Weapon>? weapons,
    Set<Outfit>? outfits,
    Weapon? weapon,
    Outfit? outfit,
    List<Absorption>? codex,
    int? huntsLost,
    int? riftsSealed,
    SoulMemory? memory,
    Art? art,
  }) => ReaperState(
    id: id,
    genomeCode: genomeCode,
    xp: xp ?? this.xp,
    weapons: weapons ?? this.weapons,
    outfits: outfits ?? this.outfits,
    weapon: weapon ?? this.weapon,
    outfit: outfit ?? this.outfit,
    codex: codex ?? this.codex,
    huntsLost: huntsLost ?? this.huntsLost,
    riftsSealed: riftsSealed ?? this.riftsSealed,
    memory: memory ?? this.memory,
    art: art ?? this.art,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'genome': genomeCode,
    'xp': xp,
    'weapons': [for (final weapon in weapons) weapon.id],
    'outfits': [for (final outfit in outfits) outfit.name],
    'weapon': weapon.id,
    'outfit': outfit.name,
    'codex': [for (final entry in codex) entry.toJson()],
    'huntsLost': huntsLost,
    'riftsSealed': riftsSealed,
    'memory': memory.toJson(),
    'art': art?.name,
  };
}
