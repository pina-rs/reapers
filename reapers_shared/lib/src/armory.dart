import 'reaper_class.dart';

/// How worked a weapon is. Each temper is the same weapon after more has
/// happened to it, never a different weapon.
enum Temper {
  /// As issued: wood, rope and plain iron.
  shrouded(strike: 1.0, levelRequired: 1, price: 0),

  /// Dyed dark, shod in iron and carved with marks.
  sworn(strike: 1.3, levelRequired: 4, price: 220),

  /// Charred and pitted, with the edge burning through.
  scorched(strike: 1.7, levelRequired: 9, price: 640),

  /// Barely there: an outline around a lit edge.
  hollowed(strike: 2.2, levelRequired: 15, price: 1500);

  const Temper({
    required this.strike,
    required this.levelRequired,
    required this.price,
  });

  /// Multiplier on every strike's ward damage.
  final double strike;

  /// The reaper level needed to buy this temper.
  final int levelRequired;

  /// Cost in anima. The shrouded temper of a class's own weapon is free.
  final int price;
}

/// One buyable weapon: a [kind] at a [temper].
final class Weapon {
  const Weapon(this.kind, this.temper);

  /// Reads a weapon back from its [id], such as `scythe.sworn`.
  factory Weapon.parse(String id) {
    final parts = id.split('.');

    if (parts.length != 2) {
      throw FormatException('Not a weapon id', id);
    }

    return Weapon(
      WeaponKind.values.byName(parts[0]),
      Temper.values.byName(parts[1]),
    );
  }

  final WeaponKind kind;

  final Temper temper;

  String get id => '${kind.name}.${temper.name}';

  WeaponProfile get profile => weaponProfiles[kind]!;

  /// What another class pays to pick up this kind for the first time. A
  /// class's own kind costs nothing at the shrouded temper.
  static const crossClassPrice = 120;

  /// The price in anima for a reaper of [reaperClass].
  int priceFor(ReaperClass reaperClass) {
    if (temper != Temper.shrouded) {
      return temper.price;
    }

    return reaperClass.weapon == kind ? 0 : crossClassPrice;
  }

  @override
  bool operator ==(Object other) =>
      other is Weapon && other.kind == kind && other.temper == temper;

  @override
  int get hashCode => Object.hash(kind, temper);

  @override
  String toString() => 'Weapon($id)';
}

/// How one kind of weapon handles, whatever its temper.
final class WeaponProfile {
  const WeaponProfile({
    required this.strike,
    required this.reachDegrees,
    required this.swingTicks,
  });

  /// Multiplier on strike damage.
  final double strike;

  /// Added to the tap radius around a seal, in degrees of view.
  final double reachDegrees;

  /// Ticks between one strike landing and the next being allowed.
  final int swingTicks;
}

/// The handling of each weapon kind.
///
/// Damage per second is kept close across kinds: a slow weapon hits harder,
/// a fast one reaches less. What differs is the rhythm a hunt is played at.
const weaponProfiles = {
  WeaponKind.scythe: WeaponProfile(
    strike: 1.15,
    reachDegrees: 1.6,
    swingTicks: 26,
  ),
  WeaponKind.lanternPole: WeaponProfile(
    strike: 1.0,
    reachDegrees: 0.8,
    swingTicks: 22,
  ),
  WeaponKind.balanceFlail: WeaponProfile(
    strike: 1.0,
    reachDegrees: 0.4,
    swingTicks: 24,
  ),
  WeaponKind.spear: WeaponProfile(
    strike: 1.45,
    reachDegrees: 0,
    swingTicks: 32,
  ),
  WeaponKind.chainWhip: WeaponProfile(
    strike: 0.8,
    reachDegrees: 0.6,
    swingTicks: 16,
  ),
  WeaponKind.bell: WeaponProfile(
    strike: 1.3,
    reachDegrees: 2.4,
    swingTicks: 28,
  ),
};

/// Every weapon the armory sells.
final List<Weapon> armoryWeapons = [
  for (final kind in WeaponKind.values)
    for (final temper in Temper.values) Weapon(kind, temper),
];

/// The four states of a reaper's outfit.
///
/// "Darker" means the same outfit after something happened to it: the cloth
/// is dyed, then charred, then partly gone, while the reaper's one light
/// burns brighter. An outfit never changes how a hunt plays.
enum Outfit {
  /// Undyed cloth, rope and wood.
  shrouded(levelRequired: 1, price: 0),

  /// Dyed dark, with iron fittings and carved marks.
  sworn(levelRequired: 5, price: 300),

  /// Charred cloth and pitted metal.
  scorched(levelRequired: 12, price: 900),

  /// Parts of the figure are absent; the light is at its brightest.
  hollowed(levelRequired: 20, price: 2400);

  const Outfit({required this.levelRequired, required this.price});

  /// The reaper level needed to buy this outfit.
  final int levelRequired;

  /// Cost in anima.
  final int price;
}
