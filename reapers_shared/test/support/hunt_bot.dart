import 'package:reapers_shared/reapers_shared.dart';

/// A hunt for a common husk with a level-one harvester.
HuntSim simFor({
  int seed = 7,
  Vessel vessel = Vessel.penguin,
  Rarity rarity = Rarity.common,
  ReaperClass reaperClass = ReaperClass.harvester,
  int level = 1,
  Temper temper = Temper.shrouded,
  double leniency = 1,
}) => HuntSim(
  seed: seed,
  vessel: vessel,
  ward: rarity.ward,
  stats: HuntStats.derive(
    HuntLoadout(
      reaperClass: reaperClass,
      level: level,
      weapon: Weapon(reaperClass.weapon, temper),
    ),
    leniency: leniency,
  ),
);

/// An input that does nothing: aim straight ahead, perfectly still.
const idle = HuntInput(aim: Aim.ahead, turnSpeed: 0);
