import 'armory.dart';
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
  }) : weapons = Set.unmodifiable(weapons),
       outfits = Set.unmodifiable(outfits),
       codex = List.unmodifiable(codex) {
    ReaperGenome.parse(genomeCode);
    if (id.isEmpty ||
        xp < 0 ||
        huntsLost < 0 ||
        riftsSealed < 0 ||
        !weapons.contains(weapon) ||
        !outfits.contains(outfit)) {
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

  bool get isNft => id != oathId;
  ReaperGenome get genome => ReaperGenome.parse(genomeCode);
  int get level => Progression.levelForXp(xp);
  ReaperSoul get soul => ReaperSoul.fromGenome(genome);
  int get soulRank =>
      SoulRules.rank(level: level, memory: memory, gift: soul.gift);

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
  };
}
