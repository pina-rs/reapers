import '../feeding.dart';
import '../nft_catalogue.dart';
import 'hunt_stats.dart';

/// Where an encounter is.
enum HuntPhase {
  /// The husk has not noticed the reaper. Moving slowly earns an ambush.
  stalk,

  /// Seals open and shut, and the reaper strikes them.
  fight,

  /// The husk is drawing breath. A shriek follows.
  windup,

  /// The husk is shrieking. The reaper holds still or loses resolve.
  shriek,

  /// The husk is running at the reaper. A swing of the phone turns it
  /// aside; without one it strikes, and the reaper loses resolve.
  charge,

  /// The ward is gone and the husk hangs open, waiting to be reaped.
  broken,

  /// The husk was reaped and absorbed. The hunt is over.
  absorbed,

  /// The husk slipped away. The hunt is over.
  escaped,
}

/// How cleanly the reap was cut.
enum ReapGrade {
  /// The cut tore: too fast, or too far from the arc.
  ragged(reward: 1.0),

  /// A steady cut along the arc.
  clean(reward: 1.25),

  /// A cut that barely left the arc at all.
  perfect(reward: 1.5);

  const ReapGrade({required this.reward});

  /// Multiplier on the experience and anima the husk is worth.
  final double reward;
}

/// Why a husk got away.
enum EscapeCause {
  /// The reaper flinched until no resolve was left.
  resolve,

  /// The fight ran past its limit.
  time,

  /// Too many reap windows closed unfinished.
  reap,
}

/// Something that happened during one tick, for the view to show and sound.
///
/// Events describe results the rules have already decided. The view never
/// feeds anything back from them.
sealed class HuntEvent {
  const HuntEvent();
}

/// The stalk is over and the fight has begun.
final class StalkEnded extends HuntEvent {
  const StalkEnded({required this.ambush});

  /// Whether the reaper steadied on the husk before it noticed.
  final bool ambush;
}

/// A strike landed on an open seal.
final class StrikeLanded extends HuntEvent {
  const StrikeLanded({
    required this.seal,
    required this.damage,
    required this.cadence,
    required this.empowered,
    this.cleaved = const [],
  });

  final int seal;

  /// Ward removed, including anything the strike cleaved.
  final double damage;

  /// Hits in a row, counting this one.
  final int cadence;

  /// Whether an ambush, a perk or striking in order strengthened it.
  final bool empowered;

  /// Further seals the same strike caught.
  final List<int> cleaved;
}

/// A strike hit the husk's body, away from any open seal.
final class StrikeGlanced extends HuntEvent {
  const StrikeGlanced({required this.damage});

  final double damage;
}

/// A strike hit nothing.
final class StrikeWhiffed extends HuntEvent {
  const StrikeWhiffed();
}

/// A seal opened.
final class SealOpened extends HuntEvent {
  const SealOpened(this.seal);

  final int seal;
}

/// The husk is drawing breath to shriek.
final class ShriekWindup extends HuntEvent {
  const ShriekWindup();
}

/// The shriek has started: hold still.
final class ShriekBegan extends HuntEvent {
  const ShriekBegan();
}

/// The reaper moved or struck during a shriek.
final class Flinched extends HuntEvent {
  const Flinched({required this.resolveLeft, required this.forgiven});

  final int resolveLeft;

  /// Whether a perk absorbed the flinch, so no resolve was lost.
  final bool forgiven;
}

/// The reaper held still through a whole shriek. Every seal opens.
final class HeldFirm extends HuntEvent {
  const HeldFirm();
}

/// The husk has turned on the reaper and is coming at them.
final class ChargeBegan extends HuntEvent {
  const ChargeBegan();
}

/// The reaper swung in time and turned the charge aside. The husk is
/// stunned, with every seal in view open.
final class Parried extends HuntEvent {
  const Parried();
}

/// The charge reached the reaper.
final class Struck extends HuntEvent {
  const Struck({required this.resolveLeft});

  final int resolveLeft;
}

/// The husk is darting to a new place.
final class Relocating extends HuntEvent {
  const Relocating();
}

/// The ward broke. The reap arc is showing.
final class Broken extends HuntEvent {
  const Broken();
}

/// The reaper's aim reached the start of the arc and the cut has begun.
final class ReapBegan extends HuntEvent {
  const ReapBegan();
}

/// The reap window closed unfinished and the husk pulled itself together.
final class ReapRecovered extends HuntEvent {
  const ReapRecovered({required this.attemptsLeft});

  final int attemptsLeft;
}

/// The husk was reaped.
final class HuskAbsorbed extends HuntEvent {
  const HuskAbsorbed(this.grade);

  final ReapGrade grade;
}

/// The husk slipped away.
final class HuskEscaped extends HuntEvent {
  const HuskEscaped(this.cause);

  final EscapeCause cause;
}

/// How a hunt ended.
final class HuntOutcome {
  const HuntOutcome.absorbed({
    required ReapGrade this.grade,
    required this.ambushed,
    required this.ticks,
  }) : escape = null;

  const HuntOutcome.escaped({
    required EscapeCause this.escape,
    required this.ambushed,
    required this.ticks,
  }) : grade = null;

  /// Reads an outcome written by [toJson].
  factory HuntOutcome.fromJson(Map<String, dynamic> json) {
    final grade = json['grade'] as String?;
    final ambushed = json['ambushed'] as bool;
    final ticks = json['ticks'] as int;

    if (grade != null) {
      return HuntOutcome.absorbed(
        grade: ReapGrade.values.byName(grade),
        ambushed: ambushed,
        ticks: ticks,
      );
    }

    return HuntOutcome.escaped(
      escape: EscapeCause.values.byName(json['escape'] as String),
      ambushed: ambushed,
      ticks: ticks,
    );
  }

  /// The grade of the reap, or null when the husk escaped.
  final ReapGrade? grade;

  /// Why the husk escaped, or null when it was absorbed.
  final EscapeCause? escape;

  /// Whether the fight opened with an ambush.
  final bool ambushed;

  /// How long the hunt ran.
  final int ticks;

  bool get absorbed => grade != null;

  Map<String, dynamic> toJson() => {
    'grade': grade?.name,
    'escape': escape?.name,
    'ambushed': ambushed,
    'ticks': ticks,
  };

  /// Experience earned from a husk of [rarity] that fed the reaper as
  /// [feeding] says.
  int xpFor(Rarity rarity, Feeding feeding) =>
      (_reward(rarity.xp) * feeding.share).round();

  /// Anima earned from a husk of [rarity]. The feeding does not change it.
  int animaFor(Rarity rarity) => _reward(rarity.anima).round();

  double _reward(int base) {
    final grade = this.grade;

    if (grade == null) {
      return 0;
    }

    final ambush = ambushed ? HuntRules.ambushReward : 1.0;

    return base * grade.reward * ambush;
  }
}
