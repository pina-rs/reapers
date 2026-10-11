import 'dart:math' as math;

import 'package:reapers_shared/reapers_shared.dart';

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
