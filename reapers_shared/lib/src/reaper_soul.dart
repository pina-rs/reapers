import 'hunt/hunt_event.dart';
import 'reaper_class.dart';
import 'reaper_genome.dart';

/// The kind of successful hunt that teaches a birth gift.
enum SoulPath { unseen, stillness, rhythm }

/// A fixed, common disposition, with a benefit and a cost.
/// Rare omens and colours never improve these odds.
enum Temperament { patient, keen, stalwart, hungry }

/// Original powers belonging to the six offices of death. The ordering is
/// part of the R1 genome contract: append a new version, never reorder this.
enum ReaperGift {
  furrow(ReaperClass.harvester, SoulPath.unseen),
  aftermath(ReaperClass.harvester, SoulPath.stillness),
  gathering(ReaperClass.harvester, SoulPath.rhythm),
  passage(ReaperClass.ferryman, SoulPath.unseen),
  anchor(ReaperClass.ferryman, SoulPath.stillness),
  returnLight(ReaperClass.ferryman, SoulPath.rhythm),
  foreknowledge(ReaperClass.weigher, SoulPath.unseen),
  verdict(ReaperClass.weigher, SoulPath.stillness),
  equilibrium(ReaperClass.weigher, SoulPath.rhythm),
  firstLight(ReaperClass.chooser, SoulPath.unseen),
  reprisal(ReaperClass.chooser, SoulPath.stillness),
  lastWord(ReaperClass.chooser, SoulPath.rhythm),
  hunger(ReaperClass.hollow, SoulPath.unseen),
  echo(ReaperClass.hollow, SoulPath.stillness),
  unbroken(ReaperClass.hollow, SoulPath.rhythm),
  hush(ReaperClass.mourner, SoulPath.unseen),
  keening(ReaperClass.mourner, SoulPath.stillness),
  refrain(ReaperClass.mourner, SoulPath.rhythm);

  const ReaperGift(this.reaperClass, this.path);

  final ReaperClass reaperClass;
  final SoulPath path;
}

/// Birth identity derived only from permanent R1 fields. No extra random
/// roll, client claim or NFT metadata can change a reaper's power.
final class ReaperSoul {
  const ReaperSoul({required this.gift, required this.temperament});

  factory ReaperSoul.fromGenome(ReaperGenome genome) => ReaperSoul(
    gift:
        ReaperGift.values[genome.reaperClass.index * SoulRules.giftsPerClass +
            (genome.givenName * 17 + genome.epithet * 31 + genome.gaze) %
                SoulRules.giftsPerClass],
    temperament:
        Temperament.values[(genome.stature + genome.gauntness + genome.tatter) %
            Temperament.values.length],
  );

  final ReaperGift gift;
  final Temperament temperament;
}

/// Replay observations, never submitted separately by the player. Counters
/// describe an encounter; a memory awards at most one mark per path per win.
final class HuntDeeds {
  const HuntDeeds({this.holds = 0, this.parries = 0, this.maxCadence = 0});

  factory HuntDeeds.fromJson(Map<String, dynamic> json) {
    final holds = json['holds'] as int;
    final parries = json['parries'] as int;
    final cadence = json['maxCadence'] as int;

    if (holds < 0 || parries < 0 || cadence < 0) {
      throw const FormatException('Negative hunt deed');
    }

    return HuntDeeds(holds: holds, parries: parries, maxCadence: cadence);
  }

  final int holds;
  final int parries;
  final int maxCadence;

  Map<String, dynamic> toJson() => {
    'holds': holds,
    'parries': parries,
    'maxCadence': maxCadence,
  };
}

/// Earned history belongs to the reaper and travels with its canonical NFT
/// record. Old saves begin without invented deeds; XP and equipment survive.
final class SoulMemory {
  const SoulMemory({this.unseen = 0, this.stillness = 0, this.rhythm = 0});

  factory SoulMemory.fromJson(Map<String, dynamic> json) {
    final unseen = json['unseen'] as int;
    final stillness = json['stillness'] as int;
    final rhythm = json['rhythm'] as int;

    if (unseen < 0 || stillness < 0 || rhythm < 0) {
      throw const FormatException('Negative soul memory');
    }

    return SoulMemory(unseen: unseen, stillness: stillness, rhythm: rhythm);
  }

  final int unseen;
  final int stillness;
  final int rhythm;

  int marks(SoulPath path) => switch (path) {
    SoulPath.unseen => unseen,
    SoulPath.stillness => stillness,
    SoulPath.rhythm => rhythm,
  };

  /// Losing or abandoning a hunt grants no marks. Waiting for repeated
  /// shrieks cannot farm more marks than a single well-played absorption.
  SoulMemory remember(HuntOutcome outcome) {
    if (!outcome.absorbed) return this;

    return SoulMemory(
      unseen: unseen + (outcome.ambushed ? 1 : 0),
      stillness: stillness + (outcome.deeds.holds > 0 ? 1 : 0),
      rhythm:
          rhythm +
          (outcome.deeds.maxCadence >= SoulRules.cadenceForMark ? 1 : 0),
    );
  }

  Map<String, dynamic> toJson() => {
    'unseen': unseen,
    'stillness': stillness,
    'rhythm': rhythm,
  };
}

/// Every new progression tunable. Read docs/gameplay.md and the identity
/// research before changing these playtest starting values.
abstract final class SoulRules {
  static const giftsPerClass = 3;
  static const cadenceForMark = 5;
  static const rankLevels = [1, 5, 12, 20];
  static const rankMarks = [0, 3, 12, 30];
  static const maxRank = 3;
  static const patientCaptureTicks = 30;
  static const patientSwingTicks = 2;
  static const keenReachDegrees = 0.6;
  static const keenCaptureTicks = 24;
  static const stalwartStrike = 0.95;
  static const hungryStrike = 1.08;
  static const hungryFlinch = 0.92;
  static const giftStrikePerStrength = 0.25;
  static const furrowCleavePerStrength = 0.15;
  static const foreknowledgeTicksPerStrength = 30;
  static const passageStaggerTicksPerStrength = 30;
  static const equilibriumHits = 3;
  static const equilibriumTicksPerStrength = 24;
  static const gatheringHits = 3;
  static const gatheringTicksPerStrength = 24;
  static const executionWardShare = 0.35;
  static const executionCadence = 3;
  static const returnCadence = 5;
  static const hushTicksPerStrength = 30;
  static const refrainHits = 3;
  static const refrainTicksPerStrength = 15;
  static const refrainMaxTolls = 6;

  /// Level and demonstrated practice are both needed. Money, rarity and
  /// feeding luck alone cannot awaken a gift's deeper behaviour.
  static int rank({
    required int level,
    required SoulMemory memory,
    required ReaperGift gift,
  }) {
    var rank = 0;

    for (var next = 1; next <= maxRank; next++) {
      if (level < rankLevels[next] ||
          memory.marks(gift.path) < rankMarks[next]) {
        break;
      }

      rank = next;
    }

    return rank;
  }
}
