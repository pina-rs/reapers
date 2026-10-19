import 'package:reapers_shared/reapers_shared.dart';

/// A kitchen: a counter 92 cm high runs across the room ahead, from 35
/// degrees left to 35 degrees right, with its front 2.2 m away and room to
/// hide behind it 3.2 m out. Its top is level. Everywhere else the floor is
/// clear for 4.5 m.
HuntPlan kitchen({int counterCentimetres = 92, bool ledge = true}) {
  bool counter(int slice) => degrees(HuntPlan.bearingOf(slice)).abs() < 35;

  return HuntPlan(
    centreCentidegrees: 0,
    halfWidthCentidegrees: 8000,
    reachDecimetres: [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        counter(slice) ? 22 : 45,
    ],
    coverCentimetres: [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        counter(slice) ? counterCentimetres : 0,
    ],
    hideDecimetres: [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        counter(slice) ? 32 : 0,
    ],
    ledges: [
      for (var slice = 0; slice < HuntPlan.slices; slice++)
        ledge && counter(slice),
    ],
  );
}
