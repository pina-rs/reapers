import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

import 'reaper_appearance.dart';

/// Garment targets are mutually exclusive: fitted tunics already include
/// shortening, so adding the spirit's short-robe target would shorten twice.
Map<String, double> reaperClothWeights(
  ReaperGenome genome,
  ReaperAppearance appearance,
  ReaperMotion motion,
  double seconds,
) {
  final cloth = reaperClothAt(motion, seconds);
  final humanoid = appearance.form == ReaperForm.humanoid;
  final tunic = appearance.clothing == ReaperClothing.tunic;
  // A grounded garment cannot drift sideways through the hips like an empty
  // spirit robe. Keep the secondary sway inside its fitted allowance.
  final drift = humanoid ? .35 : 1.0;
  return {
    'sway_x': cloth.swayX * drift,
    'sway_y': cloth.swayY * drift,
    'billow': cloth.billow,
    'gaunt': ReaperGenome.fraction(genome.gauntness),
    'long_hem': ReaperGenome.fraction(genome.tatter),
    'short_robe': !humanoid && tunic ? 1 : 0,
    'humanoid_robe': humanoid && !tunic ? 1 : 0,
    'humanoid_tunic': humanoid && tunic ? 1 : 0,
  };
}

/// Periodic secondary motion sampled from the same clock as the baked joints.
/// Genome morphs are intentionally separate so posing cannot erase a trait.
({double swayX, double swayY, double billow}) reaperClothAt(
  ReaperMotion motion,
  double seconds,
) {
  final phase = seconds / motion.seconds * math.pi * 2;
  final trailing = switch (motion) {
    ReaperMotion.sprint => 0.7,
    ReaperMotion.run => 0.45,
    _ => 0.2,
  };
  return (
    swayX: 0.35 * math.sin(phase),
    swayY: trailing + 0.2 * math.sin(phase),
    billow: 0.3 + 0.2 * math.sin(phase),
  );
}
