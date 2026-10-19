/// The four numbers a reaper is rated on, each from 1 to 5.
///
/// Every class totals [ReaperStats.classTotal], so no shuffle result is simply
/// better than another: a class is a lean, never a lock.
final class ReaperStats {
  const ReaperStats({
    required this.strike,
    required this.reach,
    required this.capture,
    required this.resilience,
  });

  /// What every class's four ratings add up to.
  static const classTotal = 13;

  /// Ward removed by one landed strike.
  final int strike;

  /// How far from a seal a tap may land and still count.
  final int reach;

  /// How long a broken husk stays open to the reap.
  final int capture;

  /// How many flinches a reaper survives before the husk slips away.
  final int resilience;

  int get total => strike + reach + capture + resilience;
}

/// The one rule each class bends during a hunt.
enum ClassPerk {
  /// A strike also lands on every other open seal near the tap.
  cleave,

  /// The first flinch of a hunt costs no resolve.
  steadfast,

  /// Seals are numbered, and striking them in order doubles the strike.
  measure,

  /// The strike after an unflinching hold lands three times as hard.
  chosenBlow,

  /// Cadence builds to double strength instead of one and a half.
  chain,

  /// Each unflinching hold lengthens the reap window.
  toll,
}

/// The six kinds of reaper the shuffle can bind a player to.
///
/// The names are job titles on purpose: the classes borrow roles and tools
/// from death figures across cultures, never a named figure's likeness. See
/// docs/research/dark-games-and-ar-hunting.md.
enum ReaperClass {
  /// The field hand of death. A wide hat over a tall, narrow body.
  harvester(
    code: 'har',
    stats: ReaperStats(strike: 3, reach: 5, capture: 3, resilience: 2),
    perk: ClassPerk.cleave,
    weapon: WeaponKind.scythe,
  ),

  /// Carries the dead across and cannot be moved. A heavy cowl and a pole.
  ferryman(
    code: 'fer',
    stats: ReaperStats(strike: 3, reach: 2, capture: 3, resilience: 5),
    perk: ClassPerk.steadfast,
    weapon: WeaponKind.lanternPole,
  ),

  /// Measures a spirit before taking it. A crested mask and a balance.
  weigher(
    code: 'wei',
    stats: ReaperStats(strike: 4, reach: 2, capture: 4, resilience: 3),
    perk: ClassPerk.measure,
    weapon: WeaponKind.balanceFlail,
  ),

  /// Picks who falls, then strikes once. A one-sided feather mantle.
  chooser(
    code: 'cho',
    stats: ReaperStats(strike: 5, reach: 4, capture: 2, resilience: 2),
    perk: ClassPerk.chosenBlow,
    weapon: WeaponKind.spear,
  ),

  /// The rider with no head. An empty collar and a lantern at the hip.
  hollow(
    code: 'hol',
    stats: ReaperStats(strike: 4, reach: 3, capture: 2, resilience: 4),
    perk: ClassPerk.chain,
    weapon: WeaponKind.chainWhip,
  ),

  /// Announces a death before it arrives. A floor-length veil and a bell.
  mourner(
    code: 'mou',
    stats: ReaperStats(strike: 1, reach: 4, capture: 5, resilience: 3),
    perk: ClassPerk.toll,
    weapon: WeaponKind.bell,
  );

  const ReaperClass({
    required this.code,
    required this.stats,
    required this.perk,
    required this.weapon,
  });

  /// The three-letter tag used in a genome code.
  final String code;

  final ReaperStats stats;

  final ClassPerk perk;

  /// The kind of weapon this class starts with.
  final WeaponKind weapon;

  static ReaperClass fromCode(String code) => values.firstWhere(
    (reaperClass) => reaperClass.code == code,
    orElse: () => throw FormatException('Unknown reaper class "$code"'),
  );
}

/// The weapon shapes. Each class starts with one of the first six, and any
/// reaper may buy any of them. The greatsword and the axe are no class's
/// own: they are heavy, and every reaper comes to them from something else.
enum WeaponKind {
  scythe,
  lanternPole,
  balanceFlail,
  spear,
  chainWhip,
  bell,
  greatsword,
  axe;

  /// Whether this is one of the heavy weapons, which no class starts with.
  bool get isHeavy => this == greatsword || this == axe;
}
